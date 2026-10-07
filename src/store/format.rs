use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::model::{Counters, Doc, Goal, GoalStatus, InboxItem, Schedule, Task, TaskState};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("line {line}: {reason}")]
pub struct ParseError {
    pub line: usize,
    pub reason: String,
}

impl ParseError {
    fn new(line: usize, reason: impl Into<String>) -> Self {
        Self {
            line,
            reason: reason.into(),
        }
    }
}

struct RawLine<'a> {
    no: usize,
    indent: usize,
    content: &'a str,
}

pub fn parse(text: &str) -> Result<Doc, ParseError> {
    let lines: Vec<&str> = text
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();

    let mut idx = 0usize;
    let header = lines
        .first()
        .copied()
        .ok_or_else(|| ParseError::new(1, "empty file"))?;
    let version = parse_header(header)?;
    idx += 1;

    let counters_line = lines
        .get(idx)
        .copied()
        .ok_or_else(|| ParseError::new(idx + 1, "missing next counters line"))?;
    let next = parse_counters(counters_line, idx + 1)?;
    idx += 1;

    let mut active = None;
    if let Some(line) = lines.get(idx)
        && let Some(rest) = line.strip_prefix("active ")
    {
        active = Some(parse_goal_id(rest, idx + 1)?);
        idx += 1;
    }

    let mut doc = Doc {
        version,
        next,
        active,
        inbox: Vec::new(),
        goals: Vec::new(),
    };

    while idx < lines.len() {
        let line = lines[idx];
        let no = idx + 1;
        if line.trim().is_empty() {
            idx += 1;
            continue;
        }
        if line.starts_with('#') {
            idx += 1;
            continue;
        }
        if line == "inbox" {
            idx += 1;
            while idx < lines.len() {
                let l = lines[idx];
                if !l.starts_with("  ") || l.starts_with("   ") {
                    break;
                }
                let content = &l[2..];
                if !content.starts_with('i') {
                    break;
                }
                doc.inbox.push(parse_inbox_item(content, idx + 1)?);
                idx += 1;
            }
            continue;
        }
        if line.starts_with("goal ") {
            let (goal, consumed) = parse_goal(&lines[idx..], idx + 1)?;
            doc.goals.push(goal);
            idx += consumed;
            continue;
        }
        return Err(ParseError::new(no, "unexpected line"));
    }

    doc.validate()
        .map_err(|reason| ParseError::new(0, reason))?;
    Ok(doc)
}

fn parse_header(line: &str) -> Result<u32, ParseError> {
    let rest = line
        .strip_prefix("trk ")
        .ok_or_else(|| ParseError::new(1, "expected `trk <version>`"))?;
    rest.parse::<u32>()
        .map_err(|_| ParseError::new(1, "invalid version"))
}

fn parse_counters(line: &str, no: usize) -> Result<Counters, ParseError> {
    let rest = line
        .strip_prefix("next ")
        .ok_or_else(|| ParseError::new(no, "expected `next g=.. t=.. i=..`"))?;
    let mut counters = Counters::default();
    for token in rest.split(' ') {
        let (key, value) = token
            .split_once('=')
            .ok_or_else(|| ParseError::new(no, "malformed counter"))?;
        let value: u64 = value
            .parse()
            .map_err(|_| ParseError::new(no, "invalid counter value"))?;
        match key {
            "g" => counters.g = value,
            "t" => counters.t = value,
            "i" => counters.i = value,
            other => return Err(ParseError::new(no, format!("unknown counter `{other}`"))),
        }
    }
    Ok(counters)
}

fn parse_goal_id(token: &str, no: usize) -> Result<u64, ParseError> {
    token
        .strip_prefix('g')
        .ok_or_else(|| ParseError::new(no, "expected goal id"))?
        .parse()
        .map_err(|_| ParseError::new(no, "invalid goal id"))
}

fn parse_task_id(token: &str, no: usize) -> Result<u64, ParseError> {
    token
        .strip_prefix('t')
        .ok_or_else(|| ParseError::new(no, "expected task id"))?
        .parse()
        .map_err(|_| ParseError::new(no, "invalid task id"))
}

fn parse_inbox_id(token: &str, no: usize) -> Result<u64, ParseError> {
    token
        .strip_prefix('i')
        .ok_or_else(|| ParseError::new(no, "expected inbox id"))?
        .parse()
        .map_err(|_| ParseError::new(no, "invalid inbox id"))
}

fn parse_ts(token: &str, no: usize) -> Result<DateTime<FixedOffset>, ParseError> {
    DateTime::parse_from_rfc3339(token).map_err(|_| ParseError::new(no, "invalid timestamp"))
}

fn parse_inbox_item(content: &str, no: usize) -> Result<InboxItem, ParseError> {
    let mut parts = content.splitn(3, ' ');
    let id = parse_inbox_id(parts.next().unwrap_or(""), no)?;
    let ts = parse_ts(parts.next().unwrap_or(""), no)?;
    let text = parts.next().unwrap_or("").to_string();
    Ok(InboxItem {
        id,
        created: ts,
        text,
    })
}

fn parse_goal(lines: &[&str], first_no: usize) -> Result<(Goal, usize), ParseError> {
    let header = lines[0];
    let mut parts = header.splitn(5, ' ');
    let _kw = parts.next();
    let id = parse_goal_id(parts.next().unwrap_or(""), first_no)?;
    let status_tok = parts.next().unwrap_or("");
    let created = parse_ts(parts.next().unwrap_or(""), first_no)?;
    let title = parts.next().unwrap_or("").to_string();

    let mut goal = Goal {
        id,
        title,
        status: GoalStatus::Open,
        created,
        cursor: None,
        roots: Vec::new(),
    };

    let mut consumed = 1usize;
    let mut body: Vec<RawLine> = Vec::new();
    while consumed < lines.len() {
        let line = lines[consumed];
        let indented = matches!(line.chars().next(), Some(' ') | Some('\t'));
        if line.is_empty() || line.starts_with('#') || !indented {
            break;
        }
        let no = first_no + consumed;
        let indent = count_indent(line, no)?;
        body.push(RawLine {
            no,
            indent,
            content: &line[indent..],
        });
        consumed += 1;
    }

    let mut done_ts: Option<DateTime<FixedOffset>> = None;
    let mut rest: Vec<RawLine> = Vec::new();
    for raw in body {
        if raw.indent == 2 && raw.content.starts_with("current ") {
            goal.cursor = Some(parse_task_id(&raw.content["current ".len()..], raw.no)?);
        } else if raw.indent == 2 && raw.content.starts_with("@ ") {
            for (key, value) in parse_attrs(&raw.content[2..], raw.no)? {
                match key.as_str() {
                    "done" => done_ts = Some(parse_ts(&value, raw.no)?),
                    other => {
                        return Err(ParseError::new(
                            raw.no,
                            format!("unknown goal attribute `{other}`"),
                        ));
                    }
                }
            }
        } else {
            rest.push(raw);
        }
    }

    match status_tok {
        "open" => {}
        "done" => {
            let ts = done_ts
                .ok_or_else(|| ParseError::new(first_no, "done goal needs `@ done=<timestamp>`"))?;
            goal.status = GoalStatus::Done(ts);
        }
        other => {
            return Err(ParseError::new(
                first_no,
                format!("unknown goal status `{other}`"),
            ));
        }
    }

    let mut idx = 0usize;
    parse_tasks(&rest, &mut idx, 2, &mut goal.roots)?;
    if idx != rest.len() {
        return Err(ParseError::new(rest[idx].no, "unexpected line"));
    }

    Ok((goal, consumed))
}

fn count_indent(line: &str, no: usize) -> Result<usize, ParseError> {
    if line.contains('\t') {
        return Err(ParseError::new(no, "tabs are not allowed"));
    }
    let count = line.bytes().take_while(|b| *b == b' ').count();
    if count % 2 != 0 {
        return Err(ParseError::new(no, "indentation must be a multiple of 2"));
    }
    Ok(count)
}

fn parse_tasks(
    lines: &[RawLine],
    idx: &mut usize,
    indent: usize,
    out: &mut Vec<Task>,
) -> Result<(), ParseError> {
    while *idx < lines.len() {
        let raw = &lines[*idx];
        if raw.indent != indent || !raw.content.starts_with('[') {
            break;
        }
        let (mut task, pending) = parse_task_header(raw.content, raw.no)?;
        *idx += 1;

        while *idx < lines.len() {
            let sub = &lines[*idx];
            if sub.indent <= indent {
                break;
            }
            if sub.indent != indent + 2 {
                return Err(ParseError::new(sub.no, "unexpected indentation"));
            }
            let content = sub.content;
            if content.starts_with('[') {
                parse_tasks(lines, idx, indent + 2, &mut task.children)?;
            } else if let Some(text) = strip_line_marker(content, '|') {
                task.note.push(text.to_string());
                *idx += 1;
            } else if let Some(text) = strip_line_marker(content, '!') {
                task.stop_reason = Some(text.to_string());
                *idx += 1;
            } else if let Some(attrs) = content.strip_prefix("@ ") {
                apply_task_attrs(&mut task, attrs, sub.no)?;
                *idx += 1;
            } else {
                return Err(ParseError::new(sub.no, "unknown line type"));
            }
        }

        finalize_task_state(&mut task, pending, raw.no)?;
        out.push(task);
    }
    Ok(())
}

fn strip_line_marker(content: &str, marker: char) -> Option<&str> {
    let rest = content.strip_prefix(marker)?;
    rest.strip_prefix(' ').or(Some(rest))
}

fn parse_task_header(content: &str, no: usize) -> Result<(Task, PendingState), ParseError> {
    let bytes = content.as_bytes();
    if bytes.len() < 5 || bytes[0] != b'[' || bytes[2] != b']' || bytes[3] != b' ' {
        return Err(ParseError::new(no, "malformed task line"));
    }
    let state = match bytes[1] {
        b' ' => PendingState::Open,
        b'x' => PendingState::Done,
        b'-' => PendingState::Dropped,
        b'!' => PendingState::Stopped,
        other => {
            return Err(ParseError::new(
                no,
                format!("unknown task state `{}`", other as char),
            ));
        }
    };
    let rest = &content[4..];
    let mut parts = rest.splitn(3, ' ');
    let id = parse_task_id(parts.next().unwrap_or(""), no)?;
    let created = parse_ts(parts.next().unwrap_or(""), no)?;
    let text = parts.next().unwrap_or("").to_string();
    let task = Task {
        id,
        text,
        state: TaskState::Open,
        created,
        note: Vec::new(),
        stop_reason: None,
        at: None,
        children: Vec::new(),
    };
    Ok((task, state))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingState {
    Open,
    Done,
    Dropped,
    Stopped,
}

fn apply_task_attrs(task: &mut Task, attrs: &str, no: usize) -> Result<(), ParseError> {
    for (key, value) in parse_attrs(attrs, no)? {
        match key.as_str() {
            "done" => {
                task.state = TaskState::Done(parse_ts(&value, no)?);
            }
            "dropped" => {
                task.state = TaskState::Dropped(parse_ts(&value, no)?);
            }
            "at" => {
                task.at = Some(parse_schedule(&value, no)?);
            }
            other => {
                return Err(ParseError::new(
                    no,
                    format!("unknown task attribute `{other}`"),
                ));
            }
        }
    }
    Ok(())
}

fn parse_attrs(text: &str, no: usize) -> Result<Vec<(String, String)>, ParseError> {
    let mut out = Vec::new();
    for token in text.split(' ') {
        if token.is_empty() {
            continue;
        }
        let (key, value) = token
            .split_once('=')
            .ok_or_else(|| ParseError::new(no, "malformed attribute"))?;
        out.push((key.to_string(), value.to_string()));
    }
    Ok(out)
}

fn parse_schedule(value: &str, no: usize) -> Result<Schedule, ParseError> {
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Ok(Schedule::Date(date));
    }
    Ok(Schedule::DateTime(parse_ts(value, no)?))
}

fn finalize_task_state(
    task: &mut Task,
    pending: PendingState,
    no: usize,
) -> Result<(), ParseError> {
    match pending {
        PendingState::Open => {
            if matches!(task.state, TaskState::Done(_) | TaskState::Dropped(_)) {
                return Err(ParseError::new(
                    no,
                    "task marked done or dropped without a state",
                ));
            }
            if task.stop_reason.is_some() {
                return Err(ParseError::new(no, "open task has a stop reason"));
            }
        }
        PendingState::Stopped => {
            if task.stop_reason.is_none() {
                return Err(ParseError::new(no, "stopped task needs a stop reason"));
            }
            if matches!(task.state, TaskState::Done(_) | TaskState::Dropped(_)) {
                return Err(ParseError::new(
                    no,
                    "stopped task has a completion attribute",
                ));
            }
            task.state = TaskState::Stopped;
        }
        PendingState::Done => {
            if !matches!(task.state, TaskState::Done(_)) {
                return Err(ParseError::new(no, "done task needs `@ done=<timestamp>`"));
            }
        }
        PendingState::Dropped => {
            if !matches!(task.state, TaskState::Dropped(_)) {
                return Err(ParseError::new(
                    no,
                    "dropped task needs `@ dropped=<timestamp>`",
                ));
            }
        }
    }
    Ok(())
}

pub fn serialize(doc: &Doc) -> String {
    let mut out = String::new();
    out.push_str(&format!("trk {}\n", doc.version));
    out.push_str(&format!(
        "next g={} t={} i={}\n",
        doc.next.g, doc.next.t, doc.next.i
    ));
    if let Some(active) = doc.active {
        out.push_str(&format!("active g{active}\n"));
    }

    if !doc.inbox.is_empty() {
        out.push('\n');
        out.push_str("inbox\n");
        for item in &doc.inbox {
            out.push_str(&format!(
                "  i{} {} {}\n",
                item.id,
                fmt_ts(&item.created),
                item.text
            ));
        }
    }

    for goal in &doc.goals {
        out.push('\n');
        write_goal(&mut out, goal);
    }

    out
}

fn write_goal(out: &mut String, goal: &Goal) {
    let status = match &goal.status {
        GoalStatus::Open => "open",
        GoalStatus::Done(_) => "done",
    };
    out.push_str(&format!(
        "goal g{} {} {} {}\n",
        goal.id,
        status,
        fmt_ts(&goal.created),
        goal.title
    ));
    if let Some(cursor) = goal.cursor {
        out.push_str(&format!("  current t{cursor}\n"));
    }
    if let GoalStatus::Done(ts) = &goal.status {
        out.push_str(&format!("  @ done={}\n", fmt_ts(ts)));
    }
    for root in &goal.roots {
        write_task(out, root, 0);
    }
}

fn write_task(out: &mut String, task: &Task, depth: usize) {
    let indent = "  ".repeat(depth + 1);
    let indent2 = "  ".repeat(depth + 2);
    let state = match &task.state {
        TaskState::Open => ' ',
        TaskState::Done(_) => 'x',
        TaskState::Dropped(_) => '-',
        TaskState::Stopped => '!',
    };
    out.push_str(&format!(
        "{indent}[{state}] t{} {} {}\n",
        task.id,
        fmt_ts(&task.created),
        task.text
    ));
    for note in &task.note {
        out.push_str(&format!("{indent2}| {note}\n"));
    }
    if let Some(reason) = &task.stop_reason {
        out.push_str(&format!("{indent2}! {reason}\n"));
    }
    match &task.state {
        TaskState::Done(ts) => out.push_str(&format!("{indent2}@ done={}\n", fmt_ts(ts))),
        TaskState::Dropped(ts) => out.push_str(&format!("{indent2}@ dropped={}\n", fmt_ts(ts))),
        _ => {}
    }
    if let Some(at) = &task.at {
        let value = match at {
            Schedule::Date(d) => d.format("%Y-%m-%d").to_string(),
            Schedule::DateTime(ts) => fmt_ts(ts),
        };
        out.push_str(&format!("{indent2}@ at={value}\n"));
    }
    for child in &task.children {
        write_task(out, child, depth + 1);
    }
}

fn fmt_ts(ts: &DateTime<FixedOffset>) -> String {
    ts.to_rfc3339()
}
