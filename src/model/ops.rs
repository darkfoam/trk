use chrono::{DateTime, FixedOffset, NaiveDate};

use super::tree;
use super::{Doc, Goal, Schedule, Task, TaskId, TaskState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Current,
    Task(TaskId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// A new top-level task in the active goal.
    Add {
        text: String,
        why: Option<String>,
    },
    /// A new parent wrapping the most recently added task (or the current task,
    /// when `next` is set).
    Then {
        text: String,
        next: bool,
        why: Option<String>,
    },
    /// A new sibling right after the most recently added task.
    And {
        text: String,
        why: Option<String>,
    },
    /// A new sibling right before the most recently added task.
    Before {
        text: String,
        why: Option<String>,
    },
    /// Complete the current task (stopping its timer).
    Done {
        force: bool,
    },
    /// Start timing the current task.
    Time,
    /// Stop timing the current task.
    Stop,
    Switch {
        target: Target,
    },
    /// Permanently delete a task and its subtree.
    Drop {
        target: Target,
    },
    /// Assign (or clear) a date on a task, for the agenda.
    SetDate {
        target: Target,
        date: Option<NaiveDate>,
    },
    GoalNew {
        title: String,
    },
    GoalSwitch {
        id: u64,
    },
    GoalDelete {
        id: u64,
    },
    Jot {
        text: String,
    },
    InboxDrop {
        id: u64,
    },
    InboxTake {
        id: u64,
        goal: Option<u64>,
        new_goal: Option<String>,
        parent: Option<TaskId>,
        switch: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgKind {
    Normal,
    Dim,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub kind: MsgKind,
    pub text: String,
}

impl Message {
    pub fn normal(text: impl Into<String>) -> Self {
        Self {
            kind: MsgKind::Normal,
            text: text.into(),
        }
    }

    pub fn dim(text: impl Into<String>) -> Self {
        Self {
            kind: MsgKind::Dim,
            text: text.into(),
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            kind: MsgKind::Warning,
            text: text.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewHint {
    Status,
    None,
}

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub messages: Vec<Message>,
    pub hint: Option<ViewHint>,
    pub changed: bool,
}

impl Outcome {
    fn status() -> Self {
        Self {
            messages: Vec::new(),
            hint: Some(ViewHint::Status),
            changed: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OpError {
    #[error("no active goal")]
    NoActiveGoal,
    #[error("no current task")]
    NoCurrentTask,
    #[error("no recently added task; use trk add first")]
    NoRecentTask,
    #[error("no timer running")]
    NoTimer,
    #[error("blocked: {text} has {count} open sub-tasks")]
    Blocked { text: String, count: usize },
    #[error("task t{0} not found")]
    TargetNotFound(TaskId),
    #[error("target task is not open")]
    TargetNotActionable,
    #[error("goal {0} not found")]
    GoalNotFound(u64),
    #[error("inbox item i{0} not found")]
    InboxNotFound(u64),
}

/// Apply a fully-specified request, returning the new document and an outcome.
/// Pure: no I/O, no clock reads, no prompts.
pub fn apply(
    doc: &Doc,
    req: &Request,
    now: DateTime<FixedOffset>,
) -> Result<(Doc, Outcome), OpError> {
    let mut doc = doc.clone();
    let outcome = match req {
        Request::Add { text, why } => op_add(&mut doc, text, why, now)?,
        Request::Then { text, next, why } => op_then(&mut doc, text, *next, why, now)?,
        Request::And { text, why } => op_and(&mut doc, text, why, now)?,
        Request::Before { text, why } => op_before(&mut doc, text, why, now)?,
        Request::Done { force } => op_done(&mut doc, *force, now)?,
        Request::Time => op_time(&mut doc, now)?,
        Request::Stop => op_stop(&mut doc, now)?,
        Request::Switch { target } => op_switch(&mut doc, target)?,
        Request::Drop { target } => op_drop(&mut doc, target)?,
        Request::SetDate { target, date } => op_set_date(&mut doc, target, *date)?,
        Request::GoalNew { title } => op_goal_new(&mut doc, title, now)?,
        Request::GoalSwitch { id } => op_goal_switch(&mut doc, *id)?,
        Request::GoalDelete { id } => op_goal_delete(&mut doc, *id)?,
        Request::Jot { text } => op_jot(&mut doc, text, now)?,
        Request::InboxDrop { id } => op_inbox_drop(&mut doc, *id)?,
        Request::InboxTake {
            id,
            goal,
            new_goal,
            parent,
            switch,
        } => op_inbox_take(
            &mut doc,
            *id,
            *goal,
            new_goal.as_deref(),
            *parent,
            *switch,
            now,
        )?,
    };
    Ok((doc, outcome))
}

fn resolve_target(doc: &Doc, target: &Target) -> Result<TaskId, OpError> {
    let goal = doc.active_goal().ok_or(OpError::NoActiveGoal)?;
    match target {
        Target::Current => goal.cursor.ok_or(OpError::NoCurrentTask),
        Target::Task(id) => {
            let task = goal.find(*id).ok_or(OpError::TargetNotFound(*id))?;
            if !task.state.is_unfinished() {
                return Err(OpError::TargetNotActionable);
            }
            Ok(*id)
        }
    }
}

fn find_task_text(goal: &Goal, id: TaskId) -> String {
    goal.find(id).map(|t| t.text.clone()).unwrap_or_default()
}

fn first_unfinished(goal: &Goal) -> Option<TaskId> {
    goal.roots
        .iter()
        .find(|t| t.state.is_unfinished())
        .map(|t| t.id)
}

/// Record the optional "why" as a note line on the task.
fn push_why(task: &mut Task, why: &Option<String>) {
    if let Some(why) = why {
        let why = why.trim();
        if !why.is_empty() {
            task.note.push(why.to_string());
        }
    }
}

/// Landing rule 4.8: moving onto a stopped task reopens it and reports why.
fn land(goal: &mut Goal, id: TaskId, messages: &mut Vec<Message>) {
    if let Some(task) = goal.find_mut(id)
        && task.state == TaskState::Stopped
    {
        if let Some(reason) = task.stop_reason.take()
            && !reason.is_empty()
        {
            messages.push(Message::normal(format!("stopped earlier: {reason}")));
        }
        task.state = TaskState::Open;
    }
}

fn move_cursor(goal: &mut Goal, id: TaskId, messages: &mut Vec<Message>) {
    goal.cursor = Some(id);
    land(goal, id, messages);
}

/// Stop a running timer on `task`, folding the elapsed time into `spent`.
/// Returns the duration that was recorded, if a timer was running.
fn stop_timer(task: &mut Task, now: DateTime<FixedOffset>) -> Option<i64> {
    let start = task.running.take()?;
    let elapsed = (now - start).num_seconds().max(0);
    task.spent += elapsed;
    Some(elapsed)
}

pub fn format_duration(seconds: i64) -> String {
    let s = seconds.max(0);
    let h = s / 3600;
    let m = (s % 3600) / 60;
    if h > 0 {
        format!("{h}h{m:02}m")
    } else if m > 0 {
        format!("{m}m")
    } else {
        format!("{s}s")
    }
}

fn op_add(
    doc: &mut Doc,
    text: &str,
    why: &Option<String>,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let id = doc.alloc_task();
    let mut task = Task::new(id, text.trim().to_string(), now);
    push_why(&mut task, why);
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    goal.roots.push(task);
    goal.last_added = Some(id);
    goal.cursor = Some(id);
    Ok(Outcome::status())
}

fn op_then(
    doc: &mut Doc,
    text: &str,
    next: bool,
    why: &Option<String>,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let target_id = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        let anchor = if next {
            goal.cursor.ok_or(OpError::NoCurrentTask)?
        } else {
            goal.last_added.ok_or(OpError::NoRecentTask)?
        };
        let task = goal.find(anchor).ok_or(OpError::TargetNotFound(anchor))?;
        if !task.state.is_unfinished() {
            return Err(OpError::TargetNotActionable);
        }
        anchor
    };

    let id = doc.alloc_task();
    let mut new_task = Task::new(id, text.trim().to_string(), now);
    push_why(&mut new_task, why);

    // The new task becomes the direct parent of the target, so the target (and
    // everything below it) moves underneath the new task.
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    let path = tree::find_path(goal, target_id).ok_or(OpError::TargetNotFound(target_id))?;
    let (&last, parent_path) = path.split_last().expect("find_path never returns empty");
    let siblings = if parent_path.is_empty() {
        &mut goal.roots
    } else {
        &mut tree::task_at_mut(goal, parent_path)
            .ok_or(OpError::TargetNotFound(target_id))?
            .children
    };
    let old = siblings.remove(last);
    new_task.children.push(old);
    siblings.insert(last, new_task);
    goal.last_added = Some(id);
    Ok(Outcome::status())
}

fn op_and(
    doc: &mut Doc,
    text: &str,
    why: &Option<String>,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let target_id = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        let anchor = goal.last_added.ok_or(OpError::NoRecentTask)?;
        let task = goal.find(anchor).ok_or(OpError::TargetNotFound(anchor))?;
        if !task.state.is_unfinished() {
            return Err(OpError::TargetNotActionable);
        }
        anchor
    };

    let id = doc.alloc_task();
    let mut task = Task::new(id, text.trim().to_string(), now);
    push_why(&mut task, why);

    let path = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        tree::find_path(goal, target_id).ok_or(OpError::TargetNotFound(target_id))?
    };
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    if path.len() == 1 {
        let idx = path[0];
        goal.roots.insert(idx + 1, task);
    } else {
        let last = *path.last().expect("non-empty path");
        let parent = tree::task_at_mut(goal, &path[..path.len() - 1]).expect("parent exists");
        parent.children.insert(last + 1, task);
    }
    goal.last_added = Some(id);
    Ok(Outcome::status())
}

fn op_before(
    doc: &mut Doc,
    text: &str,
    why: &Option<String>,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let target_id = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        let anchor = goal.last_added.ok_or(OpError::NoRecentTask)?;
        let task = goal.find(anchor).ok_or(OpError::TargetNotFound(anchor))?;
        if !task.state.is_unfinished() {
            return Err(OpError::TargetNotActionable);
        }
        anchor
    };

    let id = doc.alloc_task();
    let mut task = Task::new(id, text.trim().to_string(), now);
    push_why(&mut task, why);

    let path = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        tree::find_path(goal, target_id).ok_or(OpError::TargetNotFound(target_id))?
    };
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    if path.len() == 1 {
        let idx = path[0];
        goal.roots.insert(idx, task);
    } else {
        let last = *path.last().expect("non-empty path");
        let parent = tree::task_at_mut(goal, &path[..path.len() - 1]).expect("parent exists");
        parent.children.insert(last, task);
    }
    goal.last_added = Some(id);
    Ok(Outcome::status())
}

fn op_done(doc: &mut Doc, force: bool, now: DateTime<FixedOffset>) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let current = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        goal.cursor.ok_or(OpError::NoCurrentTask)?
    };

    let goal = doc.goal(goal_id).expect("active goal exists");
    let (has_open, count) = {
        let task = goal.find(current).expect("cursor exists");
        let mut v = Vec::new();
        task.collect_open_descendants(&mut v, usize::MAX);
        (!v.is_empty(), v.len())
    };
    if has_open && !force {
        return Err(OpError::Blocked {
            text: find_task_text(goal, current),
            count,
        });
    }

    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    if let Some(task) = goal.find_mut(current) {
        stop_timer(task, now);
        task.state = TaskState::Done(now);
        task.stop_reason = None;
    }

    let mut messages = Vec::new();
    let next = if force && has_open {
        goal.find(current).and_then(Task::first_open_leaf)
    } else {
        match tree::nearest_open_ancestor(goal, current) {
            Some(parent) => {
                let parent_has_open = goal
                    .find(parent)
                    .map(|p| p.has_open_descendant())
                    .unwrap_or(false);
                if parent_has_open {
                    let leaf = goal.find(parent).and_then(Task::first_open_leaf);
                    if let Some(leaf) = leaf {
                        messages.push(Message::normal(format!(
                            "next: {}  (still needed for: {})",
                            find_task_text(goal, leaf),
                            find_task_text(goal, parent)
                        )));
                    }
                    leaf
                } else {
                    messages.push(Message::normal(format!(
                        "back to: {}",
                        find_task_text(goal, parent)
                    )));
                    Some(parent)
                }
            }
            None => match tree::next_open_root_after(goal, current) {
                Some(root) => {
                    messages.push(Message::normal(format!(
                        "next top-level task: {}",
                        find_task_text(goal, root)
                    )));
                    Some(root)
                }
                None => None,
            },
        }
    };

    match next {
        Some(id) => move_cursor(goal, id, &mut messages),
        None => goal.cursor = None,
    }

    if goal.cursor.is_none() && !goal.has_open_tasks() {
        messages.push(Message::normal("goal has no open tasks."));
    }

    Ok(Outcome {
        messages,
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn op_time(doc: &mut Doc, now: DateTime<FixedOffset>) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let current = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        goal.cursor.ok_or(OpError::NoCurrentTask)?
    };
    let text = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        find_task_text(goal, current)
    };
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    let task = goal.find_mut(current).expect("cursor exists");
    if let Some(start) = task.running {
        let elapsed = (now - start).num_seconds().max(0);
        return Ok(Outcome {
            messages: vec![Message::dim(format!(
                "already timing: {text}  ({})",
                format_duration(elapsed)
            ))],
            hint: Some(ViewHint::Status),
            changed: false,
        });
    }
    task.running = Some(now);
    Ok(Outcome {
        messages: vec![Message::normal(format!("timing: {text}"))],
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn op_stop(doc: &mut Doc, now: DateTime<FixedOffset>) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let current = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        goal.cursor.ok_or(OpError::NoCurrentTask)?
    };
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    let task = goal.find_mut(current).expect("cursor exists");
    let elapsed = stop_timer(task, now).ok_or(OpError::NoTimer)?;
    let text = task.text.clone();
    let total = task.spent;
    Ok(Outcome {
        messages: vec![Message::normal(format!(
            "stopped: {text}  (+{}, {} total)",
            format_duration(elapsed),
            format_duration(total)
        ))],
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn op_switch(doc: &mut Doc, target: &Target) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let id = resolve_target(doc, target)?;
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    let mut messages = Vec::new();
    let warning = goal.find(id).map(|task| {
        let mut descendants = Vec::new();
        task.collect_open_descendants(&mut descendants, usize::MAX);
        (!descendants.is_empty(), descendants.len())
    });
    if let Some((true, count)) = warning {
        messages.push(Message::warning(format!(
            "\"{}\" has {count} open sub-tasks and can't be completed until they are done.",
            find_task_text(goal, id)
        )));
    }
    move_cursor(goal, id, &mut messages);
    Ok(Outcome {
        messages,
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn op_drop(doc: &mut Doc, target: &Target) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let id = match target {
        Target::Current => {
            let goal = doc.goal(goal_id).expect("active goal exists");
            goal.cursor.ok_or(OpError::NoCurrentTask)?
        }
        Target::Task(id) => *id,
    };
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    let removed = tree::remove(goal, id).ok_or(OpError::TargetNotFound(id))?;
    let text = removed.text.clone();
    if goal.cursor.is_some_and(|c| goal.find(c).is_none()) {
        goal.cursor = first_unfinished(goal);
    }
    if goal.last_added.is_some_and(|l| goal.find(l).is_none()) {
        goal.last_added = None;
    }
    Ok(Outcome {
        messages: vec![Message::normal(format!("deleted: {text}"))],
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn op_set_date(
    doc: &mut Doc,
    target: &Target,
    date: Option<NaiveDate>,
) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let id = match target {
        Target::Current => {
            let goal = doc.goal(goal_id).expect("active goal exists");
            goal.cursor.ok_or(OpError::NoCurrentTask)?
        }
        Target::Task(id) => *id,
    };
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    let task = goal.find_mut(id).ok_or(OpError::TargetNotFound(id))?;
    task.at = date.map(Schedule::Date);
    let text = task.text.clone();
    let message = match date {
        Some(d) => format!("dated {text}: {}", d.format("%Y-%m-%d")),
        None => format!("cleared the date on {text}"),
    };
    Ok(Outcome {
        messages: vec![Message::normal(message)],
        hint: Some(ViewHint::None),
        changed: true,
    })
}

fn op_goal_new(doc: &mut Doc, title: &str, now: DateTime<FixedOffset>) -> Result<Outcome, OpError> {
    let id = doc.alloc_goal();
    doc.goals.push(Goal::new(id, title.trim().to_string(), now));
    doc.active = Some(id);
    Ok(Outcome {
        messages: vec![Message::normal(format!("goal: {}", title.trim()))],
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn op_goal_switch(doc: &mut Doc, id: u64) -> Result<Outcome, OpError> {
    doc.goal(id).ok_or(OpError::GoalNotFound(id))?;
    doc.active = Some(id);
    Ok(Outcome::status())
}

fn op_goal_delete(doc: &mut Doc, id: u64) -> Result<Outcome, OpError> {
    let idx = doc
        .goals
        .iter()
        .position(|g| g.id == id)
        .ok_or(OpError::GoalNotFound(id))?;
    let goal = doc.goals.remove(idx);
    if doc.active == Some(id) {
        doc.active = doc.goals.first().map(|g| g.id);
    }
    Ok(Outcome {
        messages: vec![Message::normal(format!("deleted goal: {}", goal.title))],
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn op_jot(doc: &mut Doc, text: &str, now: DateTime<FixedOffset>) -> Result<Outcome, OpError> {
    let id = doc.alloc_inbox();
    doc.inbox.push(super::InboxItem {
        id,
        created: now,
        text: text.trim().to_string(),
    });
    let count = doc.inbox.len();
    Ok(Outcome {
        messages: vec![Message::normal(format!(
            "jotted: {}  (inbox: {count})",
            text.trim()
        ))],
        hint: Some(ViewHint::None),
        changed: true,
    })
}

fn op_inbox_drop(doc: &mut Doc, id: u64) -> Result<Outcome, OpError> {
    let idx = doc
        .inbox
        .iter()
        .position(|i| i.id == id)
        .ok_or(OpError::InboxNotFound(id))?;
    let item = doc.inbox.remove(idx);
    Ok(Outcome {
        messages: vec![Message::normal(format!("dropped: {}", item.text))],
        hint: Some(ViewHint::None),
        changed: true,
    })
}

#[allow(clippy::too_many_arguments)]
fn op_inbox_take(
    doc: &mut Doc,
    id: u64,
    goal: Option<u64>,
    new_goal: Option<&str>,
    parent: Option<TaskId>,
    switch: bool,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
    let idx = doc
        .inbox
        .iter()
        .position(|i| i.id == id)
        .ok_or(OpError::InboxNotFound(id))?;
    let item = doc.inbox.remove(idx);

    let goal_id = match (goal, new_goal) {
        (Some(goal_id), _) => {
            if doc.goal(goal_id).is_none() {
                return Err(OpError::GoalNotFound(goal_id));
            }
            goal_id
        }
        (None, Some(title)) => {
            let goal_id = doc.alloc_goal();
            doc.goals
                .push(Goal::new(goal_id, title.trim().to_string(), now));
            goal_id
        }
        (None, None) => return Err(OpError::NoActiveGoal),
    };

    let task_id = doc.alloc_task();
    let task = Task::new(task_id, item.text.clone(), now);

    let goal = doc.goal_mut(goal_id).expect("goal exists");
    match parent {
        Some(parent_id) => {
            if let Some(parent) = goal.find_mut(parent_id) {
                parent.children.push(task);
            } else {
                goal.roots.push(task);
            }
        }
        None => goal.roots.push(task),
    }
    goal.last_added = Some(task_id);
    if goal.cursor.is_none() {
        goal.cursor = Some(task_id);
    }
    if switch {
        doc.active = Some(goal_id);
    }

    Ok(Outcome {
        messages: vec![Message::normal(format!("taken: {}", item.text))],
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::format::parse;

    fn ts(s: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(s).unwrap()
    }

    fn doc() -> Doc {
        parse(
            "\
trk 1
next g=1 t=1 i=1
active g1

goal g1 open 2026-10-07T09:00:00-05:00 G
",
        )
        .unwrap()
    }

    #[test]
    fn add_is_top_level_and_sets_last_added() {
        let d = doc();
        let (d, _) = apply(
            &d,
            &Request::Add {
                text: "a".into(),
                why: None,
            },
            ts("2026-10-07T10:00:00-05:00"),
        )
        .unwrap();
        let goal = d.active_goal().unwrap();
        assert_eq!(goal.roots.len(), 1);
        assert_eq!(goal.last_added, Some(1));
        assert_eq!(goal.cursor, Some(1));
    }

    #[test]
    fn then_wraps_last_added() {
        let d = doc();
        let (d, _) = apply(
            &d,
            &Request::Add {
                text: "a".into(),
                why: None,
            },
            ts("2026-10-07T10:00:00-05:00"),
        )
        .unwrap();
        let (d, _) = apply(
            &d,
            &Request::Then {
                text: "b".into(),
                next: false,
                why: None,
            },
            ts("2026-10-07T10:01:00-05:00"),
        )
        .unwrap();
        let goal = d.active_goal().unwrap();
        assert_eq!(goal.roots.len(), 1);
        assert_eq!(goal.roots[0].id, 2);
        assert_eq!(goal.roots[0].children[0].id, 1);
        assert_eq!(goal.last_added, Some(2));
    }

    #[test]
    fn and_is_a_sibling_of_last_added() {
        let d = doc();
        let (d, _) = apply(
            &d,
            &Request::Add {
                text: "a".into(),
                why: None,
            },
            ts("2026-10-07T10:00:00-05:00"),
        )
        .unwrap();
        let (d, _) = apply(
            &d,
            &Request::And {
                text: "b".into(),
                why: None,
            },
            ts("2026-10-07T10:01:00-05:00"),
        )
        .unwrap();
        let goal = d.active_goal().unwrap();
        assert_eq!(goal.roots.len(), 2);
        assert_eq!(goal.roots[0].id, 1);
        assert_eq!(goal.roots[1].id, 2);
        assert_eq!(goal.last_added, Some(2));
    }

    #[test]
    fn before_inserts_ahead_of_last_added() {
        let d = doc();
        let (d, _) = apply(
            &d,
            &Request::Add {
                text: "a".into(),
                why: None,
            },
            ts("2026-10-07T10:00:00-05:00"),
        )
        .unwrap();
        let (d, _) = apply(
            &d,
            &Request::Add {
                text: "b".into(),
                why: None,
            },
            ts("2026-10-07T10:01:00-05:00"),
        )
        .unwrap();
        let (d, _) = apply(
            &d,
            &Request::Before {
                text: "c".into(),
                why: None,
            },
            ts("2026-10-07T10:02:00-05:00"),
        )
        .unwrap();
        let goal = d.active_goal().unwrap();
        let texts: Vec<&str> = goal.roots.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["a", "c", "b"], "c lands between a and b");
        assert_eq!(goal.last_added, Some(3));
    }

    #[test]
    fn time_and_stop_accumulate() {
        let d = doc();
        let (d, _) = apply(
            &d,
            &Request::Add {
                text: "a".into(),
                why: None,
            },
            ts("2026-10-07T10:00:00-05:00"),
        )
        .unwrap();
        let (d, _) = apply(&d, &Request::Time, ts("2026-10-07T10:00:00-05:00")).unwrap();
        let (d, _) = apply(&d, &Request::Stop, ts("2026-10-07T10:05:00-05:00")).unwrap();
        let task = d.active_goal().unwrap().find(1).unwrap();
        assert_eq!(task.spent, 300);
        assert!(task.running.is_none());
    }

    #[test]
    fn done_stops_the_timer() {
        let d = doc();
        let (d, _) = apply(
            &d,
            &Request::Add {
                text: "a".into(),
                why: None,
            },
            ts("2026-10-07T10:00:00-05:00"),
        )
        .unwrap();
        let (d, _) = apply(&d, &Request::Time, ts("2026-10-07T10:00:00-05:00")).unwrap();
        let (d, _) = apply(
            &d,
            &Request::Done { force: false },
            ts("2026-10-07T10:10:00-05:00"),
        )
        .unwrap();
        let task = d.active_goal().unwrap().find(1).unwrap();
        assert_eq!(task.spent, 600);
        assert!(task.state.is_done());
    }

    #[test]
    fn drop_hard_deletes() {
        let d = doc();
        let (d, _) = apply(
            &d,
            &Request::Add {
                text: "a".into(),
                why: None,
            },
            ts("2026-10-07T10:00:00-05:00"),
        )
        .unwrap();
        let (d, _) = apply(
            &d,
            &Request::Drop {
                target: Target::Task(1),
            },
            ts("2026-10-07T10:01:00-05:00"),
        )
        .unwrap();
        let goal = d.active_goal().unwrap();
        assert!(goal.find(1).is_none());
        assert_eq!(goal.roots.len(), 0);
        assert!(goal.cursor.is_none());
    }

    #[test]
    fn goal_delete_removes_the_goal() {
        let d = doc();
        let (d, _) = apply(
            &d,
            &Request::GoalDelete { id: 1 },
            ts("2026-10-07T10:01:00-05:00"),
        )
        .unwrap();
        assert!(d.goals.is_empty());
        assert!(d.active.is_none());
    }
}
