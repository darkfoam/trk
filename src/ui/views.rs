use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate};

use super::style::{Role, Style, StyledLine};
use super::wrap;
use crate::model::tree;
use crate::model::{Doc, Goal, GoalStatus, Schedule, Task, TaskId, TaskState};

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

/// 7.1 Status view: what am I doing, and why.
pub fn status_view(doc: &Doc, width: usize, style: Style) -> Vec<StyledLine> {
    let Some(goal) = doc.active_goal() else {
        if doc.goals.is_empty() {
            return vec![
                plain("trk: nothing here yet"),
                plain("  add your first task: trk by <what to do>"),
                plain("  trk start opens the live list"),
            ];
        }
        return vec![plain("no goal yet. try: trk goal new <title>")];
    };

    let mut out = vec![styled(goal.title.clone(), Role::Title)];
    let Some(cursor) = goal.cursor else {
        out.push(plain("no current task"));
        if goal.has_open_tasks() {
            out.push(plain("try: trk switch"));
        } else {
            out.push(plain("try: trk goal done or trk by <task>"));
        }
        return out;
    };

    let path = tree::find_path(goal, cursor).unwrap_or_default();
    let current_depth = path.len();
    let ancestors: Vec<TaskId> = (0..path.len().saturating_sub(1))
        .filter_map(|i| tree::task_at(goal, &path[..=i]).map(|t| t.id))
        .collect();

    let shown_start = ancestors.len().saturating_sub(4);
    if ancestors.len() > 4 {
        out.push(plain(format!(
            "  {} {} more above",
            style.ellipsis(),
            ancestors.len() - 4
        )));
    }
    for (offset, id) in ancestors[shown_start..].iter().enumerate() {
        let depth = shown_start + offset + 1;
        let text = goal.find(*id).map(|t| t.text.clone()).unwrap_or_default();
        let first = indent(depth);
        wrap_into(&mut out, &text, width, &first, &first, Role::Task);
    }

    let cur_task = goal.find(cursor);
    let cur_text = cur_task.map(|t| t.text.clone()).unwrap_or_default();
    let first = format!("{}{} ", indent(current_depth), style.cursor());
    let cont = " ".repeat(wrap::width_of(&first));
    wrap_into(&mut out, &cur_text, width, &first, &cont, Role::Current);

    let note_indent = indent(current_depth + 1);
    if let Some(task) = cur_task {
        for note in &task.note {
            let first = format!("{note_indent}why: ");
            let cont = format!("{note_indent}     ");
            wrap_into(&mut out, note, width, &first, &cont, Role::Dim);
        }
        if task.state == TaskState::Stopped {
            let reason = task.stop_reason.clone().unwrap_or_default();
            let first = format!("{note_indent}{} stopped: ", style.stopped());
            let cont = format!("{note_indent}  ");
            wrap_into(&mut out, &reason, width, &first, &cont, Role::Warning);
        }
    }

    if let Some(task) = cur_task {
        let mut descendants = Vec::new();
        collect_open_lines(task, current_depth + 1, &mut descendants);
        const LIMIT: usize = 8;
        for (depth, text) in descendants.iter().take(LIMIT) {
            let first = format!("{}- ", indent(*depth));
            let cont = format!("{}  ", indent(*depth));
            wrap_into(&mut out, text, width, &first, &cont, Role::Task);
        }
        if descendants.len() > LIMIT {
            out.push(plain(format!(
                "{}  {} {} more",
                indent(current_depth + 1),
                style.ellipsis(),
                descendants.len() - LIMIT
            )));
        }
    }

    out
}

fn collect_open_lines(task: &Task, base_depth: usize, out: &mut Vec<(usize, String)>) {
    for child in &task.children {
        if child.state.is_unfinished() {
            out.push((base_depth, child.text.clone()));
            collect_open_lines(child, base_depth + 1, out);
        }
    }
}

/// 7.3 Why chain: the path from the goal root to the target.
pub fn why_view(doc: &Doc, width: usize, style: Style) -> Vec<StyledLine> {
    let Some(goal) = doc.active_goal() else {
        return vec![plain("no goal yet. try: trk goal new <title>")];
    };
    let Some(cursor) = goal.cursor else {
        return vec![plain("no current task")];
    };
    why_for(goal, cursor, width, style)
}

pub fn why_for(goal: &Goal, target: TaskId, width: usize, style: Style) -> Vec<StyledLine> {
    let Some(path) = tree::find_path(goal, target) else {
        return vec![plain("task not found")];
    };
    let mut out = vec![styled(goal.title.clone(), Role::Title)];
    for i in 0..path.len() {
        let Some(task) = tree::task_at(goal, &path[..=i]) else {
            continue;
        };
        let depth = i + 1;
        let marker = if i == path.len() - 1 {
            format!("{} ", style.cursor())
        } else {
            String::new()
        };
        let first = format!("{}{}", indent(depth), marker);
        let cont = " ".repeat(wrap::width_of(&first));
        let role = if i == path.len() - 1 {
            Role::Current
        } else {
            Role::Task
        };
        wrap_into(&mut out, &task.text, width, &first, &cont, role);
        let note_indent = indent(depth + 1);
        if task.note.is_empty() {
            out.push(styled(format!("{note_indent}why: (none)"), Role::Dim));
        } else {
            for note in &task.note {
                let first = format!("{note_indent}why: ");
                let cont = format!("{note_indent}     ");
                wrap_into(&mut out, note, width, &first, &cont, Role::Dim);
            }
        }
    }
    out
}

/// 7.2 List view.
pub fn list_view(
    doc: &Doc,
    width: usize,
    style: Style,
    all: bool,
    all_goals: bool,
) -> Vec<StyledLine> {
    if all_goals {
        let mut out = Vec::new();
        for (i, goal) in doc.goals.iter().enumerate() {
            if i > 0 {
                out.push(StyledLine::default());
            }
            out.push(styled(goal.title.clone(), Role::Title));
            out.extend(goal_list_lines(goal, width, style, all));
        }
        if out.is_empty() {
            out.push(plain("no goals yet"));
        }
        return out;
    }

    let Some(goal) = doc.active_goal() else {
        return vec![plain("no goal yet. try: trk goal new <title>")];
    };
    let mut out = vec![styled(goal.title.clone(), Role::Title)];
    out.extend(goal_list_lines(goal, width, style, all));
    out
}

fn digits(n: usize) -> usize {
    n.max(1).to_string().len()
}

fn goal_list_lines(goal: &Goal, width: usize, style: Style, all: bool) -> Vec<StyledLine> {
    let entries = goal.walk();
    let unfinished: Vec<TaskId> = entries
        .iter()
        .filter(|(t, _)| t.state.is_unfinished())
        .map(|(t, _)| t.id)
        .collect();
    let num_width = digits(unfinished.len()).max(2);
    let mut numbers: HashMap<TaskId, usize> = HashMap::new();
    for (i, id) in unfinished.iter().enumerate() {
        numbers.insert(*id, i + 1);
    }

    let mut out = Vec::new();
    let mut shown_any = false;
    for (task, depth) in &entries {
        let show = match &task.state {
            TaskState::Open | TaskState::Stopped => true,
            TaskState::Done(_) | TaskState::Dropped(_) => all || task.has_open_descendant(),
        };
        if !show {
            continue;
        }
        shown_any = true;
        let is_current = goal.cursor == Some(task.id);
        let marker = if is_current {
            format!("{} ", style.cursor())
        } else {
            String::new()
        };
        let ind = indent(*depth);
        let (number, role, suffix) = match &task.state {
            TaskState::Open | TaskState::Stopped => {
                let n = numbers.get(&task.id).copied().unwrap_or(0);
                let suffix = if matches!(task.state, TaskState::Stopped) {
                    format!(" {}", style.stopped())
                } else {
                    String::new()
                };
                let role = if is_current {
                    Role::Current
                } else {
                    Role::Task
                };
                (format!("{n:>num_width$}"), role, suffix)
            }
            TaskState::Done(_) => ("x".to_string(), Role::Dim, String::new()),
            TaskState::Dropped(_) => ("-".to_string(), Role::Dim, String::new()),
        };
        let first = format!("{number}  {ind}{marker}");
        let cont = " ".repeat(wrap::width_of(&first));
        let text = format!("{}{suffix}", task.text);
        wrap_into(&mut out, &text, width, &first, &cont, role);
    }
    if !shown_any {
        out.push(plain("no tasks yet"));
    }
    out
}

/// 7.5 Log view for a set of dates, newest first.
pub fn log_view(
    doc: &Doc,
    today: NaiveDate,
    days: &[NaiveDate],
    width: usize,
    style: Style,
) -> Vec<StyledLine> {
    let _ = style;
    let mut out = Vec::new();
    let mut first = true;
    for date in days {
        if !first {
            out.push(StyledLine::default());
        }
        first = false;
        let mut done: Vec<(&str, chrono::NaiveTime, &str)> = Vec::new();
        let mut added = 0usize;
        for goal in &doc.goals {
            for (task, _) in goal.walk() {
                if task.created.date_naive() == *date {
                    added += 1;
                }
                if let TaskState::Done(ts) = &task.state
                    && ts.date_naive() == *date
                {
                    done.push((goal.title.as_str(), ts.time(), task.text.as_str()));
                }
            }
        }
        done.sort_by_key(|(_, time, _)| *time);
        let weekday = date.weekday();
        out.push(styled(
            format!(
                "{} {}   {} done, {} added",
                weekday_abbrev(weekday),
                date.format("%Y-%m-%d"),
                done.len(),
                added
            ),
            Role::Title,
        ));
        if done.is_empty() {
            out.push(styled("  nothing finished", Role::Dim));
            continue;
        }
        for (goal, time, text) in done {
            let left = format!("  {}  {text}", time.format("%H:%M"));
            let right = format!("[{goal}]");
            let left_w = wrap::width_of(&left);
            let right_w = wrap::width_of(&right);
            if left_w + 1 + right_w <= width {
                let pad = width - right_w - left_w;
                out.push(plain(format!("{left}{}{right}", " ".repeat(pad))));
            } else {
                out.push(plain(left));
                out.push(plain(format!("    {right}")));
            }
        }
    }
    let _ = today;
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

/// 7.6 Agenda view across all goals.
pub fn agenda_view(
    doc: &Doc,
    now: DateTime<FixedOffset>,
    width: usize,
    style: Style,
) -> Vec<StyledLine> {
    let _ = (width, style);
    let today = now.date_naive();
    let now_time = now.time();
    struct Entry {
        date: NaiveDate,
        time: Option<chrono::NaiveTime>,
        text: String,
        goal: String,
    }
    let mut entries = Vec::new();
    for goal in &doc.goals {
        for (task, _) in goal.walk() {
            if !task.state.is_unfinished() {
                continue;
            }
            let Some(at) = &task.at else { continue };
            let (date, time) = match at {
                Schedule::Date(d) => (*d, None),
                Schedule::DateTime(ts) => (ts.date_naive(), Some(ts.time())),
            };
            entries.push(Entry {
                date,
                time,
                text: task.text.clone(),
                goal: goal.title.clone(),
            });
        }
    }
    if entries.is_empty() {
        return vec![plain("nothing scheduled")];
    }

    // A task is overdue when its date is past, or it is due earlier today.
    let is_overdue =
        |e: &Entry| e.date < today || (e.date == today && e.time.is_some_and(|t| t < now_time));

    let mut out = Vec::new();
    let overdue: Vec<&Entry> = entries.iter().filter(|e| is_overdue(e)).collect();
    if !overdue.is_empty() {
        out.push(styled("Overdue", Role::Title));
        let mut sorted: Vec<&&Entry> = overdue.iter().collect();
        sorted.sort_by_key(|e| (e.date, e.time));
        for e in sorted {
            out.push(plain(format!(
                "  {:<20} {:<30} [{}]",
                format!("{}", e.date.format("%a %Y-%m-%d")),
                e.text,
                e.goal
            )));
        }
    }

    let mut dates: Vec<NaiveDate> = entries.iter().map(|e| e.date).collect();
    dates.sort_unstable();
    dates.dedup();
    for date in dates {
        if date < today {
            continue;
        }
        // Overdue items already appeared above; everything else is upcoming.
        let mut day_entries: Vec<&Entry> = entries
            .iter()
            .filter(|e| e.date == date && !is_overdue(e))
            .collect();
        if day_entries.is_empty() {
            continue;
        }
        let label = if date == today {
            format!("Today  {}", date.format("%a %Y-%m-%d"))
        } else {
            date.format("%a %Y-%m-%d").to_string()
        };
        out.push(styled(label, Role::Title));
        day_entries.sort_by_key(|e| (e.time.is_none(), e.time));
        for e in day_entries {
            let time = e
                .time
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_else(|| "(no time)".to_string());
            out.push(plain(format!("  {:<20} {:<30} [{}]", time, e.text, e.goal)));
        }
    }
    out
}

/// 7.15-ish: inbox list.
pub fn inbox_view(doc: &Doc, width: usize, style: Style) -> Vec<StyledLine> {
    let _ = (width, style);
    if doc.inbox.is_empty() {
        return vec![plain("inbox is empty")];
    }
    doc.inbox
        .iter()
        .enumerate()
        .map(|(i, item)| {
            plain(format!(
                "{:>2}  {}  {}",
                i + 1,
                item.created.format("%Y-%m-%d"),
                item.text
            ))
        })
        .collect()
}

/// Goal list (8.15).
pub fn goal_list_view(doc: &Doc, style: Style) -> Vec<StyledLine> {
    if doc.goals.is_empty() {
        return vec![plain("no goals yet")];
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
                "{}{:>2}  {}{}  ({} open){}",
                marker,
                i + 1,
                goal.title,
                "",
                goal.count_open(),
                suffix
            ))
        })
        .collect()
}

pub fn days_back(today: NaiveDate, n: usize) -> Vec<NaiveDate> {
    (0..n).map(|i| today - Duration::days(i as i64)).collect()
}
