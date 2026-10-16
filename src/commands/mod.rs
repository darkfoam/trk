use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::cli::*;
use crate::config::{self, Config, Toggle};
use crate::ctx::Ctx;
use crate::error::TrkError;
use crate::interact::{NullUi, PickResult, PickRow, PickSpec, TerminalUi, Ui};
use crate::model::ops::{self, Message, MsgKind, Outcome, Request, Target, ViewHint};
use crate::model::{Doc, Goal, Schedule};
use crate::store::Store;
use crate::store::undo::UndoRing;
use crate::ui::{style, views};

pub fn dispatch(cli: Cli) -> Result<i32, TrkError> {
    let color_flag = cli.color.map(ColorArg::to_toggle);

    match cli.command {
        Some(Command::Completions(args)) => completions_command(args),
        other => {
            let (config, path) = ensure_config()?;
            let ctx = Ctx::new(config, path, color_flag);
            let mut ui: Box<dyn Ui> = if ctx.interactive() {
                Box::new(TerminalUi::new(ctx.style()))
            } else {
                Box::new(NullUi)
            };
            match other {
                None => {
                    let store = active_store(&ctx);
                    let doc = store.read()?;
                    print_lines(&ctx, &views::context_view(&doc, ctx.wrap, ctx.style()));
                    Ok(0)
                }
                Some(command) => run(&ctx, ui.as_mut(), command),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Config resolution and first-run (spec 5).
// ---------------------------------------------------------------------------

fn env_store() -> Option<String> {
    std::env::var("TRK_STORE")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

fn ensure_config() -> Result<(Config, Option<PathBuf>), TrkError> {
    let path = config::config_path();
    if let Some(p) = &path
        && p.exists()
    {
        let (mut cfg, warnings) = Config::from_path(p)?;
        for warning in warnings {
            eprintln!("warning: {warning}");
        }
        apply_store_override(&mut cfg)?;
        return Ok((cfg, Some(p.clone())));
    }

    if let Some(store) = env_store() {
        let mut cfg = Config::with_store(store)?;
        apply_store_override(&mut cfg)?;
        return Ok((cfg, path));
    }

    if !std::io::stdin().is_terminal() {
        let shown = path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/trk/config.toml".into());
        return Err(TrkError::Message(format!(
            "config needs a terminal; create {shown} by hand or set TRK_STORE"
        )));
    }

    run_config_flow()?;
    let p = config::config_path().ok_or(config::ConfigError::NoHome)?;
    let (mut cfg, _) = Config::from_path(&p)?;
    apply_store_override(&mut cfg)?;
    Ok((cfg, Some(p)))
}

fn apply_store_override(cfg: &mut Config) -> Result<(), TrkError> {
    if let Some(store) = env_store() {
        cfg.override_store(store)?;
    }
    Ok(())
}

fn run_config_flow() -> Result<Config, TrkError> {
    let existing = config::config_path().and_then(|p| Config::from_path(&p).ok().map(|(c, _)| c));

    println!("Where should trk keep your task list?");
    println!("(a plain text file; you can back it up or put it in git)");
    let store_default = existing
        .as_ref()
        .map(|c| c.store_raw.clone())
        .or_else(|| {
            config::default_store()
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
        })
        .ok_or(config::ConfigError::NoHome)?;
    let store_raw = loop {
        match ask_line(&format!("path [{store_default}]: "))? {
            None => break store_default.clone(),
            Some(v) => {
                let expanded = config::expand(&v)?;
                if v.trim().is_empty() {
                    break store_default.clone();
                }
                if expanded.is_dir() {
                    println!("that is a directory; give a file path");
                    continue;
                }
                break v;
            }
        }
    };
    let store = config::expand(&store_raw)?;

    println!("Where should trk keep its backups of the task list?");
    println!("(undo snapshots; a folder, ideally somewhere you also back up)");
    let backups_default = existing
        .as_ref()
        .map(|c| c.backups_raw.clone())
        .unwrap_or_else(|| {
            store
                .parent()
                .map(|p| p.join("backups"))
                .unwrap_or_else(|| PathBuf::from("backups"))
                .to_string_lossy()
                .into_owned()
        });
    let backups_raw = loop {
        match ask_line(&format!("folder [{backups_default}]: "))? {
            None => break backups_default.clone(),
            Some(v) if v.trim().is_empty() => break backups_default.clone(),
            Some(v) => {
                let expanded = config::expand(&v)?;
                if expanded.is_file() {
                    println!("that is an existing file; give a folder path");
                    continue;
                }
                break v;
            }
        }
    };

    let config = Config {
        store_raw: store_raw.clone(),
        store: store.clone(),
        backups_is_default: false,
        backups_raw: backups_raw.clone(),
        backups: config::expand(&backups_raw)?,
        color: Toggle::Auto,
        wrap: 80,
        undo_depth: 50,
        prompt_why: true,
        ascii: Toggle::Auto,
        current_color: crate::ui::style::ColorName::default(),
        agenda_color: crate::ui::style::ColorName::Yellow,
    };

    if !config.store.exists() {
        if let Some(parent) = config.store.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            &config.store,
            crate::store::format::serialize(&Doc::empty()),
        )?;
    }
    fs::create_dir_all(&config.backups)?;

    let path = config::config_path().ok_or(config::ConfigError::NoHome)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, config.to_toml())?;

    println!("config:  {}", path.display());
    println!("store:   {}", config.store.display());
    println!("backups: {}", config.backups.display());
    Ok(config)
}

fn ask_line(prompt: &str) -> Result<Option<String>, TrkError> {
    use std::io::Write;
    let mut stdout = std::io::stdout();
    write!(stdout, "{prompt}")?;
    stdout.flush()?;
    let mut line = String::new();
    let n = std::io::stdin().read_line(&mut line)?;
    if n == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim_end_matches(['\n', '\r']).to_string()))
}

// ---------------------------------------------------------------------------
// Shared plumbing.
// ---------------------------------------------------------------------------

fn active_store(ctx: &Ctx) -> Store {
    Store::new(ctx.config.store.clone())
}

fn undo_ring(ctx: &Ctx) -> UndoRing {
    UndoRing::new(ctx.config.backups.clone(), ctx.config.undo_depth)
}

fn print_lines(ctx: &Ctx, lines: &[style::StyledLine]) {
    let rendered = style::render_lines(lines, ctx.style());
    print!("{rendered}");
}

fn emit_messages(ctx: &Ctx, messages: &[Message]) {
    for message in messages {
        let role = match message.kind {
            MsgKind::Normal => style::Role::Normal,
            MsgKind::Dim => style::Role::Dim,
            MsgKind::Warning => style::Role::Warning,
        };
        for line in crate::ui::wrap::wrap_text(&message.text, ctx.wrap, "", "") {
            let styled = style::StyledLine {
                spans: vec![style::Span { text: line, role }],
            };
            if message.kind == MsgKind::Warning {
                eprintln!("{}", style::render_line(&styled, ctx.style()));
            } else {
                println!("{}", style::render_line(&styled, ctx.style()));
            }
        }
    }
}

fn emit_outcome(ctx: &Ctx, doc: &Doc, outcome: &Outcome) {
    emit_messages(ctx, &outcome.messages);
    match outcome.hint {
        Some(ViewHint::Status) => {
            print_lines(ctx, &views::context_view(doc, ctx.wrap, ctx.style()));
        }
        Some(ViewHint::None) | None => {}
    }
}

fn mutate(ctx: &Ctx, label: &str, request: &Request) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let lock = store.lock()?;
    let before_text = fs::read_to_string(&store.path).ok();
    let doc = store.read()?;
    let now = ctx.clock.now();
    let (new_doc, outcome) = ops::apply(&doc, request, now)?;

    if outcome.changed {
        if let Some(before) = &before_text {
            undo_ring(ctx).snapshot(before, label)?;
        }
        new_doc
            .validate()
            .map_err(|e| TrkError::Message(format!("internal error: {e}")))?;
        store.save(&new_doc)?;
    }
    drop(lock);

    emit_outcome(ctx, &new_doc, &outcome);
    Ok(0)
}

fn join_text(words: &[String]) -> String {
    words.join(" ")
}

fn require_text(kind: &str, words: &[String]) -> Result<String, TrkError> {
    let text = join_text(words);
    if text.trim().is_empty() {
        return Err(TrkError::Usage(format!("{kind}: give the task text")));
    }
    Ok(text)
}

/// The optional "why" for a new task: taken from `--why`, else prompted for
/// when interactive (an empty line means none).
fn resolve_why(
    ctx: &Ctx,
    ui: &mut dyn Ui,
    why: Option<String>,
    no_why: bool,
) -> Result<Option<String>, TrkError> {
    if let Some(why) = why {
        let why = why.trim().to_string();
        return Ok(if why.is_empty() { None } else { Some(why) });
    }
    if no_why || !ctx.config.prompt_why || !ctx.interactive() {
        return Ok(None);
    }
    match ui.ask("why? ")? {
        Some(text) if !text.trim().is_empty() => Ok(Some(text)),
        _ => Ok(None),
    }
}

/// Rows for a picker: every task, or only the unfinished ones.
fn task_rows(goal: &Goal, unfinished_only: bool) -> Vec<PickRow> {
    let mut rows = Vec::new();
    let mut n = 0;
    for (task, depth) in goal.walk() {
        if unfinished_only && !task.state.is_unfinished() {
            continue;
        }
        n += 1;
        rows.push(PickRow {
            id: task.id,
            label: format!("{:>2}  {}{}", n, "  ".repeat(depth), task.text),
            number: Some(n),
            current: goal.cursor == Some(task.id),
        });
    }
    rows
}

fn goal_rows(doc: &Doc) -> Vec<PickRow> {
    doc.goals
        .iter()
        .enumerate()
        .map(|(i, g)| PickRow {
            id: g.id,
            label: format!("{:>2}  {}", i + 1, g.title),
            number: Some(i + 1),
            current: doc.active == Some(g.id),
        })
        .collect()
}

fn interactive_initial(rows: &[PickRow]) -> usize {
    rows.iter().position(|r| r.current).unwrap_or(0)
}

fn run(ctx: &Ctx, ui: &mut dyn Ui, command: Command) -> Result<i32, TrkError> {
    match command {
        Command::Add(args) => {
            let text = require_text("add", &args.text)?;
            let why = resolve_why(ctx, ui, args.why, args.no_why)?;
            mutate(ctx, "add", &Request::Add { text, why })
        }
        Command::Then(args) => {
            let text = require_text("then", &args.text)?;
            let why = resolve_why(ctx, ui, args.why, args.no_why)?;
            mutate(
                ctx,
                "then",
                &Request::Then {
                    text,
                    next: args.next,
                    why,
                },
            )
        }
        Command::And(args) => {
            let text = require_text("and", &args.text)?;
            let why = resolve_why(ctx, ui, args.why, args.no_why)?;
            mutate(ctx, "and", &Request::And { text, why })
        }
        Command::Before(args) => {
            let text = require_text("before", &args.text)?;
            let why = resolve_why(ctx, ui, args.why, args.no_why)?;
            mutate(ctx, "before", &Request::Before { text, why })
        }
        Command::Done(args) => mutate(ctx, "done", &Request::Done { force: args.force }),
        Command::Time => mutate(ctx, "time", &Request::Time),
        Command::Stop => mutate(ctx, "stop", &Request::Stop),
        Command::Switch(args) => match args.what {
            Some(SwitchWhat::Goal) => switch_goal(ctx, ui),
            None => switch_task(ctx, ui),
        },
        Command::Drop(args) => drop_command(ctx, ui, args),
        Command::List(args) => match args.what {
            Some(ListWhat::Goal) => {
                let store = active_store(ctx);
                let doc = store.read()?;
                print_lines(ctx, &views::goal_list_view(&doc, ctx.style()));
                Ok(0)
            }
            None => {
                let store = active_store(ctx);
                let doc = store.read()?;
                print_lines(ctx, &views::context_view(&doc, ctx.wrap, ctx.style()));
                Ok(0)
            }
        },
        Command::View => view_command(ctx, ui),
        Command::Undo(args) => undo_command(ctx, args),
        Command::New(args) => match args.what {
            NewWhat::Goal(a) => new_goal(ctx, ui, a),
        },
        Command::Progress(_) => {
            let store = active_store(ctx);
            let doc = store.read()?;
            print_lines(
                ctx,
                &views::progress_view(&doc, ctx.clock.now(), ctx.wrap, ctx.style()),
            );
            Ok(0)
        }
        Command::Delete(args) => match args.what {
            DeleteWhat::Goal(a) => delete_goal(ctx, ui, a),
        },
        Command::Agenda => agenda_command(ctx),
        Command::Jot(args) => {
            let text = require_text("jot", &args.text)?;
            mutate(ctx, "jot", &Request::Jot { text })
        }
        Command::Inbox => inbox_command(ctx, ui),
        Command::Log(args) => log_command(ctx, args),
        Command::Completions(_) => unreachable!("completions handled in dispatch"),
    }
}

fn switch_task(ctx: &Ctx, ui: &mut dyn Ui) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let doc = store.read()?;
    let goal = doc
        .active_goal()
        .ok_or(TrkError::Op(ops::OpError::NoActiveGoal))?;
    let rows = task_rows(goal, true);
    if rows.is_empty() {
        return Err(TrkError::Nothing("no open tasks to switch to".into()));
    }
    let initial = interactive_initial(&rows);
    let spec = PickSpec::new("work on which task?", rows, initial);
    match ui.pick(&spec)? {
        PickResult::Chosen(id) => mutate(
            ctx,
            "switch",
            &Request::Switch {
                target: Target::Task(id),
            },
        ),
        PickResult::Cancel => Err(TrkError::Cancelled),
        PickResult::Create | PickResult::AssignDate(_) => {
            unreachable!("switch does not allow new items")
        }
    }
}

fn switch_goal(ctx: &Ctx, ui: &mut dyn Ui) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let doc = store.read()?;
    if doc.goals.is_empty() {
        return Err(TrkError::Nothing("no goals to switch to".into()));
    }
    let rows = goal_rows(&doc);
    let initial = interactive_initial(&rows);
    let spec = PickSpec::new("switch to which goal?", rows, initial);
    match ui.pick(&spec)? {
        PickResult::Chosen(id) => mutate(ctx, "switch goal", &Request::GoalSwitch { id }),
        PickResult::Cancel => Err(TrkError::Cancelled),
        PickResult::Create | PickResult::AssignDate(_) => {
            unreachable!("switch does not allow new items")
        }
    }
}

fn drop_command(ctx: &Ctx, ui: &mut dyn Ui, args: DropArgs) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let doc = store.read()?;
    let goal = doc
        .active_goal()
        .ok_or(TrkError::Op(ops::OpError::NoActiveGoal))?;
    let rows = task_rows(goal, false);
    if rows.is_empty() {
        return Err(TrkError::Nothing("no tasks to delete".into()));
    }
    let initial = interactive_initial(&rows);
    let spec = PickSpec::new("delete which task?", rows, initial);
    let id = match ui.pick(&spec)? {
        PickResult::Chosen(id) => id,
        PickResult::Cancel => return Err(TrkError::Cancelled),
        PickResult::Create | PickResult::AssignDate(_) => {
            unreachable!("drop does not allow new items")
        }
    };

    let children = goal.find(id).map(|t| t.children.len()).unwrap_or(0);
    if children > 0 && !args.yes {
        if !ctx.interactive() {
            return Err(TrkError::Usage(
                "drop: pass --yes to delete a task with sub-tasks without a terminal".into(),
            ));
        }
        let prompt = format!("delete this task and its {children} sub-task(s)? (y/n) ");
        if !confirm(ui, &prompt)? {
            return Err(TrkError::Cancelled);
        }
    }

    mutate(
        ctx,
        "drop",
        &Request::Drop {
            target: Target::Task(id),
        },
    )
}

/// Ask a yes/no question. Anything but y/yes (including a dismissed prompt) is
/// a no.
fn confirm(ui: &mut dyn Ui, prompt: &str) -> Result<bool, TrkError> {
    Ok(match ui.ask(prompt)? {
        Some(answer) => matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes"),
        None => false,
    })
}

/// `trk view`: browse the whole task list, assigning dates with `a`.
fn view_command(ctx: &Ctx, ui: &mut dyn Ui) -> Result<i32, TrkError> {
    loop {
        let store = active_store(ctx);
        let doc = store.read()?;
        let goal = doc
            .active_goal()
            .ok_or(TrkError::Op(ops::OpError::NoActiveGoal))?;

        let mut rows = Vec::new();
        let mut n = 0;
        for (task, depth) in goal.walk() {
            n += 1;
            let mut label = format!("{:>2}  {}{}", n, "  ".repeat(depth), task.text);
            if let Some(at) = &task.at {
                let when = match at {
                    Schedule::Date(d) => d.format("%Y-%m-%d").to_string(),
                    Schedule::DateTime(ts) => ts.format("%Y-%m-%d %H:%M").to_string(),
                };
                label.push_str(&format!("  ({when})"));
            }
            rows.push(PickRow {
                id: task.id,
                label,
                number: Some(n),
                current: goal.cursor == Some(task.id),
            });
        }
        if rows.is_empty() {
            return Err(TrkError::Nothing("no tasks to view".into()));
        }

        let initial = interactive_initial(&rows);
        let mut spec = PickSpec::new("view", rows, initial);
        spec.assign_date = true;
        match ui.pick(&spec)? {
            PickResult::AssignDate(id) => {
                let today = ctx.clock.now().date_naive();
                let prompt = "date (today, tomorrow, +Nd, mon..sun, YYYY-MM-DD; - to clear): ";
                let Some(input) = ui.ask(prompt)? else {
                    continue;
                };
                let trimmed = input.trim();
                let date = if matches!(trimmed, "-" | "none" | "clear") {
                    None
                } else {
                    Some(parse_when_date(trimmed, today)?)
                };
                mutate(
                    ctx,
                    "view date",
                    &Request::SetDate {
                        target: Target::Task(id),
                        date,
                    },
                )?;
                continue;
            }
            // A view is not a mutation: picking or dismissing just closes it.
            PickResult::Chosen(_) | PickResult::Cancel => return Ok(0),
            PickResult::Create => unreachable!("view does not allow new items"),
        }
    }
}

/// Parse a date for `trk view`: today, tomorrow, `+Nd`, a weekday, or
/// `YYYY-MM-DD`. No time component.
fn parse_when_date(spec: &str, today: NaiveDate) -> Result<NaiveDate, TrkError> {
    use chrono::Datelike;
    let token = spec.trim().to_ascii_lowercase();
    if token == "today" {
        return Ok(today);
    }
    if token == "tomorrow" {
        return Ok(today + chrono::Duration::days(1));
    }
    if let Some(rest) = token.strip_prefix('+') {
        let days = rest.strip_suffix('d').unwrap_or(rest);
        if let Ok(n) = days.parse::<i64>() {
            return Ok(today + chrono::Duration::days(n));
        }
    }
    const WEEKDAYS: [(&str, i64); 7] = [
        ("mon", 0),
        ("tue", 1),
        ("wed", 2),
        ("thu", 3),
        ("fri", 4),
        ("sat", 5),
        ("sun", 6),
    ];
    if let Some(head) = token.get(..3) {
        for (name, dow) in WEEKDAYS {
            if head == name {
                let current = today.weekday().num_days_from_monday() as i64;
                let mut delta = (dow + 7 - current) % 7;
                if delta == 0 {
                    delta = 7;
                }
                return Ok(today + chrono::Duration::days(delta));
            }
        }
    }
    NaiveDate::parse_from_str(&token, "%Y-%m-%d")
        .map_err(|_| TrkError::Usage("date: today, tomorrow, +Nd, mon..sun, or YYYY-MM-DD".into()))
}

fn agenda_command(ctx: &Ctx) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let doc = store.read()?;
    let today = ctx.clock.now().date_naive();
    print_lines(ctx, &views::agenda_view(&doc, today, ctx.wrap, ctx.style()));
    Ok(0)
}

fn new_goal(ctx: &Ctx, ui: &mut dyn Ui, args: NewGoalArgs) -> Result<i32, TrkError> {
    let mut title = join_text(&args.title);
    if title.trim().is_empty() {
        if !ctx.interactive() {
            return Err(TrkError::Usage("new goal: give a title".into()));
        }
        title = ui.ask("goal title: ")?.unwrap_or_default();
    }
    if title.trim().is_empty() {
        return Err(TrkError::Usage("new goal: give a title".into()));
    }
    mutate(ctx, "new goal", &Request::GoalNew { title })
}

fn delete_goal(ctx: &Ctx, ui: &mut dyn Ui, args: DeleteGoalArgs) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let doc = store.read()?;
    if doc.goals.is_empty() {
        return Err(TrkError::Nothing("no goals to delete".into()));
    }
    let rows = goal_rows(&doc);
    let initial = interactive_initial(&rows);
    let spec = PickSpec::new("delete which goal?", rows, initial);
    let id = match ui.pick(&spec)? {
        PickResult::Chosen(id) => id,
        PickResult::Cancel => return Err(TrkError::Cancelled),
        PickResult::Create | PickResult::AssignDate(_) => {
            unreachable!("delete does not allow new items")
        }
    };
    if !args.yes {
        if !ctx.interactive() {
            return Err(TrkError::Usage(
                "delete goal: pass --yes to confirm without a terminal".into(),
            ));
        }
        if !confirm(ui, "delete this goal and all its tasks? (y/n) ")? {
            return Err(TrkError::Cancelled);
        }
    }
    mutate(ctx, "delete goal", &Request::GoalDelete { id })
}

fn inbox_command(ctx: &Ctx, ui: &mut dyn Ui) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let doc = store.read()?;
    if doc.inbox.is_empty() {
        return Err(TrkError::Nothing("inbox is empty".into()));
    }

    // 1. Which item?
    let rows: Vec<PickRow> = doc
        .inbox
        .iter()
        .enumerate()
        .map(|(i, item)| PickRow {
            id: item.id,
            label: format!("{:>2}  {}", i + 1, item.text),
            number: Some(i + 1),
            current: false,
        })
        .collect();
    let spec = PickSpec::new("take which item?", rows, 0);
    let item_id = match ui.pick(&spec)? {
        PickResult::Chosen(id) => id,
        PickResult::Cancel => return Err(TrkError::Cancelled),
        PickResult::Create | PickResult::AssignDate(_) => {
            unreachable!("inbox items do not allow new items")
        }
    };

    // 2. Which goal? (TAB creates a new one.)
    let rows = goal_rows(&doc);
    let initial = interactive_initial(&rows);
    let mut goal_spec = PickSpec::new("add it to which goal?", rows, initial);
    goal_spec.allow_new = true;
    let mut goal = None;
    let mut new_goal = None;
    match ui.pick(&goal_spec)? {
        PickResult::Chosen(id) => goal = Some(id),
        PickResult::Create => {
            let title = ui.ask("new goal title: ")?.unwrap_or_default();
            if title.trim().is_empty() {
                return Err(TrkError::Cancelled);
            }
            new_goal = Some(title);
        }
        PickResult::AssignDate(_) => unreachable!("inbox does not assign dates"),
        PickResult::Cancel => return Err(TrkError::Cancelled),
    }

    // 3. Drill down: insert as a child of a task, or at the top level.
    let mut parent = None;
    if let Some(gid) = goal
        && let Some(g) = doc.goal(gid)
        && !g.roots.is_empty()
    {
        let mut rows = vec![PickRow {
            id: 0,
            label: format!("{:>2}  (top level)", 1),
            number: Some(1),
            current: false,
        }];
        for (i, (task, depth)) in g.walk().iter().enumerate() {
            rows.push(PickRow {
                id: task.id,
                label: format!("{:>2}  {}{}", i + 2, "  ".repeat(*depth), task.text),
                number: Some(i + 2),
                current: g.cursor == Some(task.id),
            });
        }
        let spec = PickSpec::new("add it under which task?", rows, 0);
        match ui.pick(&spec)? {
            PickResult::Chosen(0) => parent = None,
            PickResult::Chosen(id) => parent = Some(id),
            PickResult::Cancel => return Err(TrkError::Cancelled),
            PickResult::Create | PickResult::AssignDate(_) => {}
        }
    }

    mutate(
        ctx,
        "inbox",
        &Request::InboxTake {
            id: item_id,
            goal,
            new_goal,
            parent,
            switch: true,
        },
    )
}

fn undo_command(ctx: &Ctx, args: UndoArgs) -> Result<i32, TrkError> {
    let n = args.count.unwrap_or(1);
    let ring = undo_ring(ctx);
    let store = active_store(ctx);
    match ring.pop(n)? {
        None => {
            println!("nothing to undo");
            Ok(0)
        }
        Some((label, text)) => {
            let doc = crate::store::format::parse(&text)?;
            let lock = store.lock()?;
            store.save(&doc)?;
            drop(lock);
            println!("undid: {label}");
            print_lines(ctx, &views::context_view(&doc, ctx.wrap, ctx.style()));
            Ok(0)
        }
    }
}

fn log_command(ctx: &Ctx, args: LogArgs) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let doc = store.read()?;
    let today = ctx.clock.now().date_naive();
    let days = parse_log_when(&join_text(&args.when), today)?;
    print_lines(
        ctx,
        &views::log_view(&doc, &days, ctx.clock.now(), ctx.wrap, ctx.style()),
    );
    Ok(0)
}

fn parse_log_when(spec: &str, today: NaiveDate) -> Result<Vec<NaiveDate>, TrkError> {
    let spec = spec.trim();
    if spec.is_empty() || spec.eq_ignore_ascii_case("today") {
        return Ok(vec![today]);
    }
    if spec.eq_ignore_ascii_case("yesterday") {
        return Ok(vec![today - chrono::Duration::days(1)]);
    }
    if let Ok(n) = spec.parse::<usize>() {
        return Ok(views::days_back(today, n.max(1)));
    }
    if let Ok(date) = NaiveDate::parse_from_str(spec, "%Y-%m-%d") {
        return Ok(vec![date]);
    }
    Err(TrkError::Usage(
        "log: when must be today, yesterday, N, or YYYY-MM-DD".into(),
    ))
}

// ---------------------------------------------------------------------------
// Shell completions.
// ---------------------------------------------------------------------------

fn completions_command(args: CompletionsArgs) -> Result<i32, TrkError> {
    let shell = match args.shell {
        Some(shell) => shell,
        None => detect_shell()?,
    };
    let path = install_completions(shell)?;
    println!("installed {} completions: {}", shell.name(), path.display());
    if matches!(shell, Shell::Zsh) {
        println!("  add to ~/.zshrc: fpath=(~/.zfunc $fpath)");
    }
    Ok(0)
}

fn detect_shell() -> Result<Shell, TrkError> {
    if let Ok(shell) = std::env::var("SHELL") {
        let name = Path::new(&shell)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if name.contains("zsh") {
            return Ok(Shell::Zsh);
        }
        if name.contains("bash") {
            return Ok(Shell::Bash);
        }
        if name.contains("fish") {
            return Ok(Shell::Fish);
        }
        if name.contains("pwsh") || name.contains("powershell") {
            return Ok(Shell::Powershell);
        }
        if name.contains("elvish") {
            return Ok(Shell::Elvish);
        }
        if name.contains("sh") {
            return Ok(Shell::Sh);
        }
    }
    Err(TrkError::Usage(
        "completions: could not detect the shell; pass sh, bash, zsh, fish, powershell, or elvish"
            .into(),
    ))
}

fn install_completions(shell: Shell) -> Result<PathBuf, TrkError> {
    use clap::CommandFactory;
    use clap_complete::{Shell as CS, generate};

    let cs = match shell {
        Shell::Sh | Shell::Bash => CS::Bash,
        Shell::Zsh => CS::Zsh,
        Shell::Fish => CS::Fish,
        Shell::Powershell => CS::PowerShell,
        Shell::Elvish => CS::Elvish,
    };
    let mut cmd = Cli::command();
    let mut buf = Vec::new();
    generate(cs, &mut cmd, "trk", &mut buf);

    let path = completion_path(shell)?;
    install_file(&path, &buf)?;
    Ok(path)
}

fn completion_path(shell: Shell) -> Result<PathBuf, TrkError> {
    let home = dirs::home_dir()
        .ok_or_else(|| TrkError::Message("no home directory; cannot install completions".into()))?;
    Ok(match shell {
        Shell::Sh | Shell::Bash => home.join(".local/share/bash-completion/completions/trk"),
        Shell::Zsh => home.join(".zfunc/_trk"),
        Shell::Fish => home.join(".config/fish/completions/trk.fish"),
        Shell::Powershell => home.join(".config/powershell/trk.ps1"),
        Shell::Elvish => home.join(".config/elvish/lib/trk.elv"),
    })
}

fn install_file(path: &Path, bytes: &[u8]) -> Result<(), TrkError> {
    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        if e.kind() != std::io::ErrorKind::PermissionDenied {
            return Err(e.into());
        }
        elevated_mkdir(parent)?;
    }
    match fs::write(path, bytes) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => elevated_write(path, bytes),
        Err(e) => Err(e.into()),
    }
}

#[cfg(unix)]
fn elevated_mkdir(dir: &Path) -> Result<(), TrkError> {
    let status = std::process::Command::new("sudo")
        .arg("mkdir")
        .arg("-p")
        .arg(dir)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(TrkError::Message("sudo mkdir failed".into()))
    }
}

#[cfg(not(unix))]
fn elevated_mkdir(_dir: &Path) -> Result<(), TrkError> {
    Err(TrkError::Message(
        "cannot create the completion directory: permission denied".into(),
    ))
}

#[cfg(unix)]
fn elevated_write(path: &Path, bytes: &[u8]) -> Result<(), TrkError> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new("sudo")
        .arg("tee")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;
    child
        .stdin
        .take()
        .expect("sudo tee has a piped stdin")
        .write_all(bytes)?;
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(TrkError::Message("sudo tee failed".into()))
    }
}

#[cfg(not(unix))]
fn elevated_write(_path: &Path, _bytes: &[u8]) -> Result<(), TrkError> {
    Err(TrkError::Message(
        "cannot write the completion file: permission denied".into(),
    ))
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ops::{Request, apply};
    use chrono::{DateTime, FixedOffset};

    fn dt(s: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(s).unwrap()
    }

    #[test]
    fn task_rows_number_only_unfinished_when_asked() {
        let doc =
            crate::store::format::parse(include_str!("../../tests/fixtures/basic.trk")).unwrap();
        let goal = doc.active_goal().unwrap();
        let all = task_rows(goal, false);
        let open = task_rows(goal, true);
        assert!(all.len() > open.len());
        assert!(open.iter().all(|r| r.number.is_some()));
    }

    #[test]
    fn require_text_rejects_empty() {
        assert!(require_text("add", &[]).is_err());
        assert_eq!(
            require_text("add", &["a".into(), "b".into()]).unwrap(),
            "a b"
        );
    }

    #[test]
    fn progress_marks_running_timer() {
        let doc =
            crate::store::format::parse(include_str!("../../tests/fixtures/basic.trk")).unwrap();
        let (doc, _) = apply(&doc, &Request::Time, dt("2026-10-07T10:00:00-05:00")).unwrap();
        let lines = views::progress_view(
            &doc,
            dt("2026-10-07T10:05:00-05:00"),
            80,
            style::Style::new(false, true),
        );
        let text: String = lines
            .iter()
            .map(|l| l.text())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("(running)"), "{text}");
    }
}
