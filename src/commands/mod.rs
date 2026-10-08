use std::fs;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

use crate::cli::*;
use crate::config::{self, Config, Toggle};
use crate::ctx::Ctx;
use crate::error::TrkError;
use crate::interact::{NullUi, PickResult, PickRow, PickSpec, TerminalUi, Ui};
use crate::model::ops::{self, Message, MsgKind, Outcome, Request, Target, ViewHint};
use crate::model::{Doc, Goal, Schedule, TaskState};
use crate::store::Store;
use crate::store::undo::UndoRing;
use crate::ui::{style, views};

pub fn dispatch(cli: Cli) -> Result<i32, TrkError> {
    let color_flag = cli.color.map(ColorArg::to_toggle);

    match cli.command {
        Some(Command::Completions(args)) => {
            print_completions(args);
            Ok(0)
        }
        Some(Command::Config(args)) => config_command(color_flag, args),
        Some(Command::Prompt(args)) => Ok(prompt_command(color_flag, args)),
        other => {
            let (config, path) = ensure_config(&other)?;
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
                    print_lines(&ctx, &views::status_view(&doc, ctx.wrap, ctx.style()));
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

fn ensure_config(command: &Option<Command>) -> Result<(Config, Option<PathBuf>), TrkError> {
    let _ = command;
    let path = config::config_path();
    if let Some(p) = &path
        && p.exists()
    {
        let (mut cfg, warnings) = Config::from_path(p)?;
        for warning in warnings {
            if let Some(store) = env_store() {
                let _ = store;
            }
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
        poll_ms: 500,
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

fn config_command(color_flag: Option<Toggle>, args: ConfigArgs) -> Result<i32, TrkError> {
    match args.command {
        Some(ConfigCommand::Path) => {
            let path = config::config_path().ok_or(config::ConfigError::NoHome)?;
            println!("{}", path.display());
            Ok(0)
        }
        Some(ConfigCommand::Show) | None => {
            if args.command.is_none() {
                // bare `trk config` runs the interactive flow
                if !std::io::stdin().is_terminal() {
                    let path = config::config_path()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "~/.config/trk/config.toml".into());
                    return Err(TrkError::Message(format!(
                        "config needs a terminal; create {path} by hand or set TRK_STORE"
                    )));
                }
                run_config_flow()?;
            }
            let (mut cfg, _) = match config::config_path() {
                Some(p) if p.exists() => Config::from_path(&p)?,
                _ => return Err(TrkError::Message("no config file yet".into())),
            };
            apply_store_override(&mut cfg)?;
            let path = config::config_path()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            println!("config:  {path}");
            println!("store:   {}", cfg.store.display());
            println!("backups: {}", cfg.backups.display());
            println!("color:   {}", cfg.color.as_str());
            println!("ascii:   {}", cfg.ascii.as_str());
            println!("current_color: {}", cfg.current_color.as_str());
            println!("wrap:    {}", cfg.wrap);
            println!("undo_depth: {}", cfg.undo_depth);
            println!("prompt_why: {}", cfg.prompt_why);
            println!("poll_ms: {}", cfg.poll_ms);
            let _ = color_flag;
            Ok(0)
        }
    }
}

fn ask_line(prompt: &str) -> Result<Option<String>, TrkError> {
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
        Some(ViewHint::Status) => print_lines(ctx, &views::status_view(doc, ctx.wrap, ctx.style())),
        Some(ViewHint::Why) => print_lines(ctx, &views::why_view(doc, ctx.wrap, ctx.style())),
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

fn now(ctx: &Ctx) -> DateTime<FixedOffset> {
    ctx.clock.now()
}

fn unfinished_rows(goal: &Goal, style: &style::Style) -> Vec<PickRow> {
    let mut rows = Vec::new();
    let mut n = 0;
    for (task, depth) in goal.walk() {
        if !task.state.is_unfinished() {
            continue;
        }
        n += 1;
        let mut label = format!("{:>2}  {}{}", n, "  ".repeat(depth), task.text);
        if matches!(task.state, TaskState::Stopped) {
            label.push(' ');
            label.push_str(style.stopped());
        }
        rows.push(PickRow {
            id: task.id,
            label,
            number: Some(n),
            current: goal.cursor == Some(task.id),
        });
    }
    rows
}

fn switch_target(
    ctx: &Ctx,
    ui: &mut dyn Ui,
    goal: &Goal,
    header: &str,
) -> Result<Target, TrkError> {
    let rows = unfinished_rows(goal, &ctx.style());
    let initial = rows.iter().position(|r| r.current).unwrap_or(0);
    let spec = PickSpec {
        header: header.to_string(),
        rows,
        initial,
    };
    match ui.pick(&spec)? {
        PickResult::Chosen(id) => Ok(Target::Task(id)),
        PickResult::Cancel => Err(TrkError::Cancelled),
    }
}

fn resolved_target(
    ctx: &Ctx,
    ui: &mut dyn Ui,
    switch: bool,
    header: &str,
) -> Result<Target, TrkError> {
    if !switch {
        return Ok(Target::Current);
    }
    let store = active_store(ctx);
    let doc = store.read()?;
    let goal = doc
        .active_goal()
        .ok_or(TrkError::Op(ops::OpError::NoActiveGoal))?;
    switch_target(ctx, ui, goal, header)
}

fn resolve_why(
    ctx: &Ctx,
    ui: &mut dyn Ui,
    why: Option<String>,
    no_why: bool,
) -> Result<Option<String>, TrkError> {
    if let Some(w) = why {
        return Ok(Some(w));
    }
    if no_why || !ctx.config.prompt_why || !ctx.interactive() {
        return Ok(None);
    }
    match ui.ask("why? ")? {
        Some(text) if !text.trim().is_empty() => Ok(Some(text)),
        _ => Ok(None),
    }
}

fn join_text(words: &[String]) -> String {
    words.join(" ")
}

fn run(ctx: &Ctx, ui: &mut dyn Ui, command: Command) -> Result<i32, TrkError> {
    match command {
        Command::Add(args) => {
            let text = join_text(&args.text);
            if text.trim().is_empty() {
                return Err(TrkError::Usage("add: give the task text".into()));
            }
            let target = resolved_target(ctx, ui, args.switch, "add under which task?")?;
            let why = resolve_why(ctx, ui, args.why, args.no_why)?;
            mutate(
                ctx,
                "add",
                &Request::Add {
                    text,
                    why,
                    target,
                    stay: args.stay,
                },
            )
        }
        Command::By(args) => {
            let text = join_text(&args.text);
            if text.trim().is_empty() {
                return Err(TrkError::Usage("by: give the task text".into()));
            }
            let why = resolve_why(ctx, ui, args.why, args.no_why)?;
            mutate(ctx, "by", &Request::By { text, why })
        }
        Command::Then(args) => {
            let text = join_text(&args.text);
            if text.trim().is_empty() {
                return Err(TrkError::Usage("then: give the task text".into()));
            }
            let target = resolved_target(ctx, ui, args.switch, "after which task?")?;
            let why = resolve_why(ctx, ui, args.why, args.no_why)?;
            mutate(ctx, "then", &Request::Then { text, why, target })
        }
        Command::And(args) => {
            let text = join_text(&args.text);
            if text.trim().is_empty() {
                return Err(TrkError::Usage("and: give the task text".into()));
            }
            let target = resolved_target(ctx, ui, args.switch, "next to which task?")?;
            let why = resolve_why(ctx, ui, args.why, args.no_why)?;
            mutate(ctx, "and", &Request::And { text, why, target })
        }
        Command::Done(args) => mutate(ctx, "done", &Request::Done { force: args.force }),
        Command::Drop(args) => {
            let why = resolve_why(ctx, ui, args.why, false)?;
            mutate(
                ctx,
                "drop",
                &Request::Drop {
                    force: args.force,
                    why,
                },
            )
        }
        Command::Stop(args) => {
            let target = resolved_target(ctx, ui, args.switch, "which task?")?;
            let mut reason = join_text(&args.reason);
            if reason.trim().is_empty()
                && ctx.interactive()
                && let Some(text) = ui.ask("why stopping? ")?
            {
                reason = text;
            }
            mutate(ctx, "stop", &Request::Stop { target, reason })
        }
        Command::Go => mutate(ctx, "go", &Request::Go),
        Command::Why(args) => {
            let target = resolved_target(ctx, ui, args.switch, "which task?")?;
            let store = active_store(ctx);
            let doc = store.read()?;
            let goal = doc
                .active_goal()
                .ok_or(TrkError::Op(ops::OpError::NoActiveGoal))?;
            let id = match target {
                Target::Current => goal
                    .cursor
                    .ok_or(TrkError::Op(ops::OpError::NoCurrentTask))?,
                Target::Task(id) => id,
            };
            print_lines(ctx, &views::why_for(goal, id, ctx.wrap, ctx.style()));
            Ok(0)
        }
        Command::Note(args) => {
            let target = resolved_target(ctx, ui, args.switch, "which task?")?;
            if let Some(replace) = args.replace {
                return mutate(
                    ctx,
                    "note",
                    &Request::NoteReplace {
                        target,
                        text: replace,
                    },
                );
            }
            if args.clear {
                return mutate(ctx, "note", &Request::NoteClear { target });
            }
            let text = join_text(&args.text);
            if text.trim().is_empty() {
                let store = active_store(ctx);
                let doc = store.read()?;
                let goal = doc
                    .active_goal()
                    .ok_or(TrkError::Op(ops::OpError::NoActiveGoal))?;
                let id = match target {
                    Target::Current => goal
                        .cursor
                        .ok_or(TrkError::Op(ops::OpError::NoCurrentTask))?,
                    Target::Task(id) => id,
                };
                if let Some(task) = goal.find(id) {
                    for note in &task.note {
                        println!("{note}");
                    }
                }
                return Ok(0);
            }
            mutate(ctx, "note", &Request::NoteAdd { target, text })
        }
        Command::Rename(args) => {
            let text = join_text(&args.text);
            if text.trim().is_empty() {
                return Err(TrkError::Usage("rename: give the new text".into()));
            }
            let target = resolved_target(ctx, ui, args.switch, "which task?")?;
            mutate(ctx, "rename", &Request::Rename { target, text })
        }
        Command::Switch(args) => switch_command(ctx, ui, args),
        Command::List(args) => {
            let store = active_store(ctx);
            let doc = store.read()?;
            print_lines(
                ctx,
                &views::list_view(&doc, ctx.wrap, ctx.style(), args.all, args.all_goals),
            );
            Ok(0)
        }
        Command::Undo(args) => undo_command(ctx, args),
        Command::Goal(args) => goal_command(ctx, ui, args),
        Command::Jot(args) => {
            let text = join_text(&args.text);
            if text.trim().is_empty() {
                return Err(TrkError::Usage("jot: give the text".into()));
            }
            mutate(ctx, "jot", &Request::Jot { text })
        }
        Command::Inbox(args) => inbox_command(ctx, ui, args),
        Command::Log(args) => log_command(ctx, args),
        Command::At(args) => at_command(ctx, ui, args),
        Command::Agenda => {
            let store = active_store(ctx);
            let doc = store.read()?;
            print_lines(
                ctx,
                &views::agenda_view(&doc, now(ctx), ctx.wrap, ctx.style()),
            );
            Ok(0)
        }
        Command::Start(args) => {
            crate::tui::run(ctx, args.focus)?;
            Ok(0)
        }
        Command::Prompt(_) => unreachable!("prompt handled in dispatch"),
        Command::Config(_) => unreachable!("config handled in dispatch"),
        Command::Completions(_) => unreachable!("completions handled in dispatch"),
    }
}

fn switch_command(ctx: &Ctx, ui: &mut dyn Ui, args: SwitchArgs) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let doc = store.read()?;
    let goal = doc
        .active_goal()
        .ok_or(TrkError::Op(ops::OpError::NoActiveGoal))?;
    let target = match args.index {
        Some(index) => {
            let rows = unfinished_rows(goal, &ctx.style());
            let row = rows
                .iter()
                .find(|r| r.number == Some(index))
                .ok_or_else(|| TrkError::Usage(format!("switch: no task number {index}")))?;
            Target::Task(row.id)
        }
        None => switch_target(ctx, ui, goal, "work on which task?")?,
    };
    mutate(ctx, "switch", &Request::Switch { target })
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
            print_lines(ctx, &views::status_view(&doc, ctx.wrap, ctx.style()));
            Ok(0)
        }
    }
}

fn goal_command(ctx: &Ctx, ui: &mut dyn Ui, args: GoalArgs) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let doc = store.read()?;
    match args.command {
        Some(GoalCommand::Show) | None => {
            let goal = doc
                .active_goal()
                .ok_or(TrkError::Op(ops::OpError::NoActiveGoal))?;
            println!("{}", goal.title);
            println!("id: g{}", goal.id);
            println!("state: {}", if goal.is_open() { "open" } else { "done" });
            println!("created: {}", goal.created.to_rfc3339());
            println!("open: {}  done: {}", goal.count_open(), goal.count_done());
            Ok(0)
        }
        Some(GoalCommand::List) => {
            print_lines(ctx, &views::goal_list_view(&doc, ctx.style()));
            Ok(0)
        }
        Some(GoalCommand::New(a)) => {
            let title = join_text(&a.title);
            if title.trim().is_empty() {
                return Err(TrkError::Usage("goal new: give a title".into()));
            }
            mutate(
                ctx,
                "goal new",
                &Request::GoalNew {
                    title,
                    stay: a.stay,
                },
            )
        }
        Some(GoalCommand::Switch(a)) => {
            let id = resolve_goal_index(&doc, a.index, ui, GoalFilter::Open)?;
            mutate(ctx, "goal switch", &Request::GoalSwitch { id })
        }
        Some(GoalCommand::Done(a)) => {
            mutate(ctx, "goal done", &Request::GoalDone { force: a.force })
        }
        Some(GoalCommand::Rename(a)) => {
            let title = join_text(&a.title);
            if title.trim().is_empty() {
                return Err(TrkError::Usage("goal rename: give a title".into()));
            }
            mutate(ctx, "goal rename", &Request::GoalRename { title })
        }
        Some(GoalCommand::Reopen(a)) => {
            let id = resolve_goal_index(&doc, a.index, ui, GoalFilter::Done)?;
            mutate(ctx, "goal reopen", &Request::GoalReopen { id })
        }
    }
}

/// Which goals a `goal switch`/`goal reopen` selection may target.
#[derive(Clone, Copy)]
enum GoalFilter {
    Open,
    Done,
}

/// Resolve a goal number or picker choice. Numbers are the ones shown by
/// `trk goal list`, which numbers every goal; the filter decides whether an
/// open or a done goal is acceptable.
fn resolve_goal_index(
    doc: &Doc,
    index: Option<usize>,
    ui: &mut dyn Ui,
    filter: GoalFilter,
) -> Result<u64, TrkError> {
    let wanted = |g: &Goal| match filter {
        GoalFilter::Open => g.is_open(),
        GoalFilter::Done => !g.is_open(),
    };
    match index {
        Some(n) if n >= 1 => {
            let goal = doc
                .goals
                .get(n - 1)
                .ok_or_else(|| TrkError::Usage(format!("no goal number {n}")))?;
            if !wanted(goal) {
                return Err(TrkError::Usage(match filter {
                    GoalFilter::Open => format!("goal {n} is not open"),
                    GoalFilter::Done => format!("goal {n} is not done"),
                }));
            }
            Ok(goal.id)
        }
        Some(n) => Err(TrkError::Usage(format!("no goal number {n}"))),
        None => {
            let rows: Vec<PickRow> = doc
                .goals
                .iter()
                .enumerate()
                .filter(|(_, g)| wanted(g))
                .map(|(i, g)| PickRow {
                    id: g.id,
                    label: format!("{:>2}  {}", i + 1, g.title),
                    number: Some(i + 1),
                    current: doc.active == Some(g.id),
                })
                .collect();
            if rows.is_empty() {
                return Err(TrkError::Usage(match filter {
                    GoalFilter::Open => "no open goals".into(),
                    GoalFilter::Done => "no done goals".into(),
                }));
            }
            let initial = rows.iter().position(|r| r.current).unwrap_or(0);
            let spec = PickSpec {
                header: match filter {
                    GoalFilter::Open => "switch to which goal?".into(),
                    GoalFilter::Done => "reopen which goal?".into(),
                },
                rows,
                initial,
            };
            match ui.pick(&spec)? {
                PickResult::Chosen(id) => Ok(id),
                PickResult::Cancel => Err(TrkError::Cancelled),
            }
        }
    }
}

fn inbox_command(ctx: &Ctx, ui: &mut dyn Ui, args: InboxArgs) -> Result<i32, TrkError> {
    match args.command {
        None => {
            let store = active_store(ctx);
            let doc = store.read()?;
            print_lines(ctx, &views::inbox_view(&doc, ctx.wrap, ctx.style()));
            Ok(0)
        }
        Some(InboxCommand::Drop(a)) => {
            let store = active_store(ctx);
            let doc = store.read()?;
            let id = match a.index {
                Some(n) => doc
                    .inbox
                    .get(n.saturating_sub(1))
                    .map(|i| i.id)
                    .ok_or_else(|| TrkError::Usage(format!("inbox drop: no item {n}")))?,
                None => {
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
                    let spec = PickSpec {
                        header: "drop which item?".into(),
                        rows,
                        initial: 0,
                    };
                    match ui.pick(&spec)? {
                        PickResult::Chosen(id) => id,
                        PickResult::Cancel => return Err(TrkError::Cancelled),
                    }
                }
            };
            mutate(ctx, "inbox drop", &Request::InboxDrop { id })
        }
        Some(InboxCommand::Take(a)) => {
            let store = active_store(ctx);
            let doc = store.read()?;
            let item = match a.index {
                Some(n) => doc
                    .inbox
                    .get(n.saturating_sub(1))
                    .cloned()
                    .ok_or_else(|| TrkError::Usage(format!("inbox take: no item {n}")))?,
                None if doc.inbox.len() == 1 => doc.inbox[0].clone(),
                None => {
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
                    let spec = PickSpec {
                        header: "take which item?".into(),
                        rows,
                        initial: 0,
                    };
                    let id = match ui.pick(&spec)? {
                        PickResult::Chosen(id) => id,
                        PickResult::Cancel => return Err(TrkError::Cancelled),
                    };
                    doc.inbox
                        .iter()
                        .find(|i| i.id == id)
                        .cloned()
                        .ok_or_else(|| TrkError::Message("item vanished".into()))?
                }
            };

            let (goal_id, new_goal) = if let Some(title) = &a.new_goal {
                (None, Some(title.clone()))
            } else if let Some(name) = &a.goal {
                let needle = name.to_ascii_lowercase();
                let matches: Vec<&Goal> = doc
                    .goals
                    .iter()
                    .filter(|g| g.is_open() && g.title.to_ascii_lowercase().contains(&needle))
                    .collect();
                match matches.len() {
                    1 => (Some(matches[0].id), None),
                    0 => return Err(TrkError::Usage(format!("no goal matches \"{name}\""))),
                    _ => {
                        return Err(TrkError::Usage(format!(
                            "\"{name}\" matches {} goals",
                            matches.len()
                        )));
                    }
                }
            } else {
                let rows: Vec<PickRow> = doc
                    .goals
                    .iter()
                    .filter(|g| g.is_open())
                    .enumerate()
                    .map(|(i, g)| PickRow {
                        id: g.id,
                        label: format!("{:>2}  {}", i + 1, g.title),
                        number: Some(i + 1),
                        current: doc.active == Some(g.id),
                    })
                    .collect();
                if rows.is_empty() {
                    return Err(TrkError::Usage(
                        "inbox take: no open goals; use --new-goal".into(),
                    ));
                }
                let initial = rows.iter().position(|r| r.current).unwrap_or(0);
                let spec = PickSpec {
                    header: "put it in which goal?".into(),
                    rows,
                    initial,
                };
                match ui.pick(&spec)? {
                    PickResult::Chosen(id) => (Some(id), None),
                    PickResult::Cancel => return Err(TrkError::Cancelled),
                }
            };

            let parent = if a.switch {
                let goal = goal_id.and_then(|id| doc.goal(id)).ok_or_else(|| {
                    TrkError::Usage("inbox take: --switch needs an existing goal with tasks".into())
                })?;
                match switch_target(ctx, ui, goal, "put it under which task?")? {
                    Target::Task(id) => Some(id),
                    Target::Current => None,
                }
            } else {
                None
            };

            let why = {
                if !ctx.config.prompt_why || !ctx.interactive() {
                    None
                } else {
                    match ui.ask("why? ")? {
                        Some(t) if !t.trim().is_empty() => Some(t),
                        _ => None,
                    }
                }
            };

            mutate(
                ctx,
                "inbox take",
                &Request::InboxTake {
                    id: item.id,
                    goal: goal_id,
                    new_goal,
                    parent,
                    why,
                    switch: a.activate,
                },
            )
        }
    }
}

fn log_command(ctx: &Ctx, args: LogArgs) -> Result<i32, TrkError> {
    let store = active_store(ctx);
    let doc = store.read()?;
    let today = now(ctx).date_naive();
    let days = parse_log_when(&join_text(&args.when), today)?;
    print_lines(
        ctx,
        &views::log_view(&doc, today, &days, ctx.wrap, ctx.style()),
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

fn at_command(ctx: &Ctx, ui: &mut dyn Ui, args: AtArgs) -> Result<i32, TrkError> {
    let target = resolved_target(ctx, ui, args.switch, "which task?")?;
    if args.clear {
        return mutate(ctx, "at", &Request::Schedule { target, at: None });
    }
    let spec = join_text(&args.when);
    let schedule = if spec.trim().is_empty() {
        if ctx.interactive() {
            let date = ui.ask("date (YYYY-MM-DD, today, tomorrow, mon..sun, +Nd): ")?;
            let time = ui.ask("time (HH:MM, Enter for none): ")?;
            let tokens: Vec<String> = date.into_iter().chain(time).collect::<Vec<String>>();
            parse_schedule_tokens(&tokens, now(ctx))?
        } else {
            return Err(TrkError::Usage("at: give a date".into()));
        }
    } else {
        parse_schedule_tokens(&args.when, now(ctx))?
    };
    mutate(
        ctx,
        "at",
        &Request::Schedule {
            target,
            at: Some(schedule),
        },
    )
}

fn parse_schedule_tokens(
    tokens: &[String],
    now: DateTime<FixedOffset>,
) -> Result<Schedule, TrkError> {
    let grammar = "at: date (today, tomorrow, mon..sun, YYYY-MM-DD, +Nd) and time (HH:MM, 3pm)";
    let lower: Vec<String> = tokens
        .iter()
        .map(|t| t.trim().to_ascii_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    match lower.len() {
        0 => Err(TrkError::Usage(grammar.into())),
        1 => {
            if let Some(date) = parse_date_token(&lower[0], now.date_naive()) {
                Ok(Schedule::Date(date))
            } else if let Some(time) = parse_time_token(&lower[0]) {
                time_only(time, now).ok_or_else(|| TrkError::Usage(grammar.into()))
            } else {
                Err(TrkError::Usage(grammar.into()))
            }
        }
        2 => {
            let date = parse_date_token(&lower[0], now.date_naive())
                .ok_or_else(|| TrkError::Usage(grammar.into()))?;
            let time =
                parse_time_token(&lower[1]).ok_or_else(|| TrkError::Usage(grammar.into()))?;
            combine(date, time, now).ok_or_else(|| TrkError::Usage(grammar.into()))
        }
        _ => Err(TrkError::Usage(grammar.into())),
    }
}

fn parse_date_token(token: &str, today: NaiveDate) -> Option<NaiveDate> {
    match token {
        "today" => return Some(today),
        "tomorrow" => return Some(today + chrono::Duration::days(1)),
        _ => {}
    }
    if let Some(rest) = token.strip_prefix('+')
        && let Some(days) = rest.strip_suffix('d')
        && let Ok(n) = days.parse::<i64>()
    {
        return Some(today + chrono::Duration::days(n));
    }
    let weekdays = [
        ("mon", 0u64),
        ("tue", 1),
        ("wed", 2),
        ("thu", 3),
        ("fri", 4),
        ("sat", 5),
        ("sun", 6),
    ];
    for (name, dow) in weekdays {
        if token.starts_with(name) {
            use chrono::Datelike;
            let current = today.weekday().num_days_from_monday() as u64;
            let mut delta = (dow + 7 - current) % 7;
            if delta == 0 {
                delta = 7;
            }
            return Some(today + chrono::Duration::days(delta as i64));
        }
    }
    NaiveDate::parse_from_str(token, "%Y-%m-%d").ok()
}

fn parse_time_token(token: &str) -> Option<NaiveTime> {
    if let Ok(t) = NaiveTime::parse_from_str(token, "%H:%M") {
        return Some(t);
    }
    let (digits, meridiem) = if let Some(d) = token.strip_suffix("am") {
        (d, "am")
    } else {
        let d = token.strip_suffix("pm")?;
        (d, "pm")
    };
    let mut hour: u32 = digits.parse().ok()?;
    if hour > 12 {
        return None;
    }
    if meridiem == "pm" && hour != 12 {
        hour += 12;
    }
    if meridiem == "am" && hour == 12 {
        hour = 0;
    }
    NaiveTime::from_hms_opt(hour, 0, 0)
}

fn combine(date: NaiveDate, time: NaiveTime, now: DateTime<FixedOffset>) -> Option<Schedule> {
    let naive = NaiveDateTime::new(date, time);
    now.offset()
        .from_local_datetime(&naive)
        .single()
        .map(Schedule::DateTime)
}

fn time_only(time: NaiveTime, now: DateTime<FixedOffset>) -> Option<Schedule> {
    let mut date = now.date_naive();
    let candidate = combine(date, time, now)?;
    if let Schedule::DateTime(dt) = &candidate
        && dt <= &now
    {
        date += chrono::Duration::days(1);
        return combine(date, time, now);
    }
    Some(candidate)
}

/// `trk prompt`: never fails, never prompts, never takes the lock (spec 8.19).
pub fn prompt_command(color_flag: Option<Toggle>, args: PromptArgs) -> i32 {
    let Some((config, path)) = load_config_quiet() else {
        return 0;
    };
    let ctx = Ctx::new(config, path, color_flag);
    let text = prompt_text(&ctx, args.max);
    print!("{text}");
    0
}

fn load_config_quiet() -> Option<(Config, Option<PathBuf>)> {
    let path = config::config_path();
    if let Some(p) = &path
        && p.exists()
    {
        let (mut cfg, _) = Config::from_path(p).ok()?;
        apply_store_override(&mut cfg).ok()?;
        return Some((cfg, Some(p.clone())));
    }
    if let Some(store) = env_store() {
        let mut cfg = Config::with_store(store).ok()?;
        apply_store_override(&mut cfg).ok()?;
        return Some((cfg, path));
    }
    None
}

pub fn undo_doc(ctx: &Ctx, n: usize) -> Result<Option<Doc>, TrkError> {
    let ring = undo_ring(ctx);
    match ring.pop(n)? {
        None => Ok(None),
        Some((_label, text)) => {
            let doc = crate::store::format::parse(&text)?;
            let store = active_store(ctx);
            let lock = store.lock()?;
            store.save(&doc)?;
            drop(lock);
            Ok(Some(doc))
        }
    }
}

fn print_completions(args: CompletionsArgs) {
    use clap::CommandFactory;
    use clap_complete::{Shell as CS, generate};
    let shell = match args.shell {
        Shell::Bash => CS::Bash,
        Shell::Zsh => CS::Zsh,
        Shell::Fish => CS::Fish,
        Shell::Powershell => CS::PowerShell,
        Shell::Elvish => CS::Elvish,
    };
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "trk", &mut std::io::stdout());
}

/// Best-effort prompt text for `trk prompt` (spec 8.19): never fails, never
/// takes the lock.
pub fn prompt_text(ctx: &Ctx, max: usize) -> String {
    let store = active_store(ctx);
    let Ok(doc) = store.read() else {
        return String::new();
    };
    let Some(goal) = doc.active_goal() else {
        return String::new();
    };
    let Some(cursor) = goal.cursor else {
        return String::new();
    };
    let Some(task) = goal.find(cursor) else {
        return String::new();
    };
    crate::ui::wrap::truncate(&task.text, max, ctx.style().ellipsis())
}

/// Read-only helpers for the TUI.
pub fn load_doc(ctx: &Ctx) -> Result<Doc, TrkError> {
    active_store(ctx).read()
}

pub fn save_doc(ctx: &Ctx, label: &str, doc: &Doc) -> Result<(), TrkError> {
    let store = active_store(ctx);
    let lock = store.lock()?;
    let before_text = fs::read_to_string(&store.path).ok();
    if let Some(before) = &before_text {
        undo_ring(ctx).snapshot(before, label)?;
    }
    store.save(doc)?;
    drop(lock);
    Ok(())
}

pub fn apply_doc(
    doc: &Doc,
    request: &Request,
    now: DateTime<FixedOffset>,
) -> Result<(Doc, Outcome), TrkError> {
    Ok(ops::apply(doc, request, now)?)
}

pub fn clock_now(ctx: &Ctx) -> DateTime<FixedOffset> {
    ctx.clock.now()
}

pub fn local_now() -> DateTime<FixedOffset> {
    Local::now().fixed_offset()
}
