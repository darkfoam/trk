use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate};

use super::style::{Role, Style, StyledLine};
use super::wrap;
use crate::model::ops::format_duration;
use crate::model::tree;
use crate::model::{Doc, GoalStatus, Task, TaskId, TaskState};

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

fn plain(text: impl Into<String>) -> StyledLine {
    StyledLine::plain(text)
}

fn styled(text: impl Into<String>, role: Role) -> StyledLine {
    StyledLine::styled(text, role)
}

fn wrap_into(
    out: &mut Vec<StyledLine>,
    text: &str,
    width: usize,
    first_indent: &str,
    rest_indent: &str,
    role: Role,
) {
    for line in wrap::wrap_text(text, width, first_indent, rest_indent) {
        out.push(styled(line, role));
    }
}

fn role_for(task: &Task, is_current: bool) -> Role {
    if is_current {
        Role::Current
    } else if task.state.is_done() || task.state.is_dropped() {
        Role::Done
    } else if matches!(task.state, TaskState::Stopped) {
        Role::Warning
    } else {
        Role::Task
    }
}

/// Time spent on a task, including a timer that is still running.
fn task_seconds(task: &Task, now: DateTime<FixedOffset>) -> i64 {
    let running = task
        .running
        .map(|s| (now - s).num_seconds().max(0))
        .unwrap_or(0);
    task.spent.max(0) + running
}

/// The main view (`trk list`): the current task, its siblings, and up to three
/// parents, with a pointer at the current task.
pub fn context_view(doc: &Doc, width: usize, style: Style) -> Vec<StyledLine> {
    let Some(goal) = doc.active_goal() else {
        if doc.goals.is_empty() {
            return vec![
                plain("trk: nothing here yet"),
                plain("  start a goal: trk new goal <title>"),
                plain("  add a task:   trk add <what to do>"),
            ];
        }
        return vec![plain("no goal selected. try: trk switch goal")];
    };

    let mut out = vec![styled(goal.title.clone(), Role::Title)];
    let Some(cursor) = goal.cursor else {
        out.push(plain("no current task"));
        push_level(&mut out, &goal.roots, 1, None, width, style);
        out.push(plain("try: trk switch"));
        return out;
    };

    let path = tree::find_path(goal, cursor).unwrap_or_default();
    let depth = path.len();
    let ancestors: Vec<TaskId> = (0..path.len().saturating_sub(1))
        .filter_map(|i| tree::task_at(goal, &path[..=i]).map(|t| t.id))
        .collect();

    if ancestors.len() > 3 {
        out.push(plain(format!(
            "  {} {} more above",
            style.ellipsis(),
            ancestors.len() - 3
        )));
    }
    let shown = ancestors.len().saturating_sub(3);
    for (offset, id) in ancestors[shown..].iter().enumerate() {
        let d = shown + offset + 1;
        let text = goal.find(*id).map(|t| t.text.clone()).unwrap_or_default();
        let first = indent(d);
        wrap_into(&mut out, &text, width, &first, &first, Role::Task);
    }

    let siblings: &[Task] = if depth <= 1 {
        &goal.roots
    } else {
        tree::task_at(goal, &path[..depth - 1])
            .map(|p| p.children.as_slice())
            .unwrap_or(&[])
    };
    push_level(&mut out, siblings, depth, Some(cursor), width, style);
    if let Some(task) = goal.find(cursor) {
        push_notes(&mut out, &task.note, depth + 1, width);
    }
    out
}

/// Render "why:" note lines, indented to sit under a task's text.
fn push_notes(out: &mut Vec<StyledLine>, notes: &[String], depth: usize, width: usize) {
    for note in notes {
        let first = format!("{}why: ", indent(depth));
        let cont = format!("{}     ", indent(depth));
        wrap_into(out, note, width, &first, &cont, Role::Dim);
    }
}

fn push_level(
    out: &mut Vec<StyledLine>,
    tasks: &[Task],
    depth: usize,
    current: Option<TaskId>,
    width: usize,
    style: Style,
) {
    for task in tasks {
        let is_current = current == Some(task.id);
        let marker = if is_current {
            format!("{} ", style.cursor())
        } else {
            String::new()
        };
        let first = format!("{}{}", indent(depth), marker);
        let cont = " ".repeat(wrap::width_of(&first));
        wrap_into(
            out,
            &task.text,
            width,
            &first,
            &cont,
            role_for(task, is_current),
        );
    }
}

/// `trk progress goal`: every task in the goal as one tree, with completed
/// tasks struck through and a pointer at the current task.
pub fn progress_view(
    doc: &Doc,
    now: DateTime<FixedOffset>,
    width: usize,
    style: Style,
) -> Vec<StyledLine> {
    let Some(goal) = doc.active_goal() else {
        if doc.goals.is_empty() {
            return vec![plain("no goals yet. try: trk new goal <title>")];
        }
        return vec![plain("no goal selected. try: trk switch goal")];
    };

    let mut out = vec![styled(
        format!(
            "{}   ({} open, {} done)",
            goal.title,
            goal.count_open(),
            goal.count_done()
        ),
        Role::Title,
    )];
    if goal.roots.is_empty() {
        out.push(plain("no tasks yet. try: trk add <what to do>"));
        return out;
    }

    for (task, depth) in goal.walk() {
        let is_current = goal.cursor == Some(task.id);
        let marker = if is_current {
            format!("{} ", style.cursor())
        } else {
            "  ".to_string()
        };
        let mut text = task.text.clone();
        let seconds = task_seconds(task, now);
        if seconds > 0 {
            text.push_str(&format!("  [{}]", format_duration(seconds)));
            if task.running.is_some() {
                text.push_str("(running)");
            }
        }
        let first = format!("{}{}", indent(depth), marker);
        let cont = " ".repeat(wrap::width_of(&first));
        wrap_into(
            &mut out,
            &text,
            width,
            &first,
            &cont,
            role_for(task, is_current),
        );
        push_notes(&mut out, &task.note, depth + 1, width);
    }
    out
}

/// `trk list goal`: every goal, numbered, with the active one marked.
pub fn goal_list_view(doc: &Doc, style: Style) -> Vec<StyledLine> {
    if doc.goals.is_empty() {
        return vec![plain("no goals yet. try: trk new goal <title>")];
    }
    doc.goals
        .iter()
        .enumerate()
        .map(|(i, goal)| {
            let marker = if doc.active == Some(goal.id) {
                format!("{} ", style.cursor())
            } else {
                "  ".to_string()
            };
            let done = matches!(goal.status, GoalStatus::Done(_));
            let suffix = if done { "  (done)" } else { "" };
            plain(format!(
                "{}{:>2}  {}  ({} open){}",
                marker,
                i + 1,
                goal.title,
                goal.count_open(),
                suffix
            ))
        })
        .collect()
}

/// `trk log`: what got finished, by day, newest first, with time spent.
pub fn log_view(
    doc: &Doc,
    days: &[NaiveDate],
    now: DateTime<FixedOffset>,
    width: usize,
    style: Style,
) -> Vec<StyledLine> {
    let mut out = Vec::new();
    let mut first = true;
    for date in days {
        if !first {
            out.push(StyledLine::default());
        }
        first = false;

        let mut done: Vec<(&str, chrono::NaiveTime, &Task)> = Vec::new();
        let mut added = 0usize;
        for goal in &doc.goals {
            for (task, _) in goal.walk() {
                if task.created.date_naive() == *date {
                    added += 1;
                }
                if let TaskState::Done(ts) = &task.state
                    && ts.date_naive() == *date
                {
                    done.push((goal.title.as_str(), ts.time(), task));
                }
            }
        }
        done.sort_by_key(|(_, time, _)| *time);

        let total: i64 = done.iter().map(|(_, _, t)| task_seconds(t, now)).sum();
        let mut header = format!(
            "{} {}   {} done, {} added",
            weekday_abbrev(date.weekday()),
            date.format("%Y-%m-%d"),
            done.len(),
            added
        );
        if total > 0 {
            header.push_str(&format!(", {}", format_duration(total)));
        }
        out.push(styled(header, Role::Title));

        if done.is_empty() {
            out.push(styled("  nothing finished", Role::Dim));
            continue;
        }
        for (goal, time, task) in done {
            let seconds = task_seconds(task, now);
            let time_part = if seconds > 0 {
                format!("{}  {:>7}", time.format("%H:%M"), format_duration(seconds))
            } else {
                format!("{}         ", time.format("%H:%M"))
            };
            let left = format!("  {time_part}  {}", task.text);
            let right = format!("[{goal}]");
            let left_w = wrap::width_of(&left);
            let right_w = wrap::width_of(&right);
            if left_w + 1 + right_w <= width {
                let pad = width - right_w - left_w;
                out.push(styled(
                    format!("{left}{}{right}", " ".repeat(pad)),
                    Role::Done,
                ));
            } else {
                out.push(styled(left, Role::Done));
                out.push(styled(format!("    {right}"), Role::Dim));
            }
        }
    }
    let _ = style;
    out
}

fn weekday_abbrev(day: chrono::Weekday) -> &'static str {
    match day {
        chrono::Weekday::Mon => "Mon",
        chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed",
        chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri",
        chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    }
}

pub fn days_back(today: NaiveDate, n: usize) -> Vec<NaiveDate> {
    (0..n).map(|i| today - Duration::days(i as i64)).collect()
}
