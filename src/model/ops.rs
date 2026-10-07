use chrono::{DateTime, FixedOffset};

use super::tree;
use super::{Doc, Goal, GoalStatus, Schedule, Task, TaskId, TaskState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Current,
    Task(TaskId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Need {
        text: String,
        why: Option<String>,
        target: Target,
        stay: bool,
    },
    Also {
        text: String,
        why: Option<String>,
    },
    Then {
        text: String,
        why: Option<String>,
        target: Target,
    },
    Add {
        text: String,
        why: Option<String>,
        target: Target,
    },
    Done {
        force: bool,
    },
    Drop {
        force: bool,
        why: Option<String>,
    },
    Stop {
        target: Target,
        reason: String,
    },
    Pick {
        target: Target,
    },
    Go,
    NoteAdd {
        target: Target,
        text: String,
    },
    NoteReplace {
        target: Target,
        text: String,
    },
    NoteClear {
        target: Target,
    },
    Rename {
        target: Target,
        text: String,
    },
    GoalNew {
        title: String,
        stay: bool,
    },
    GoalSwitch {
        id: u64,
    },
    GoalReopen {
        id: u64,
    },
    GoalDone {
        force: bool,
    },
    GoalRename {
        title: String,
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
        why: Option<String>,
        switch: bool,
    },
    Schedule {
        target: Target,
        at: Option<Schedule>,
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
    Why,
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
    #[error("blocked: {text} has {count} open sub-tasks")]
    Blocked { text: String, count: usize },
    #[error("blocked: goal {title} has {count} open tasks")]
    GoalBlocked { title: String, count: usize },
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
        Request::Need {
            text,
            why,
            target,
            stay,
        } => op_need(&mut doc, text, why, target, *stay, now)?,
        Request::Also { text, why } => op_also(&mut doc, text, why, now)?,
        Request::Then { text, why, target } => op_then(&mut doc, text, why, target, now)?,
        Request::Add { text, why, target } => op_add(&mut doc, text, why, target, now)?,
        Request::Done { force } => op_done(&mut doc, *force, now)?,
        Request::Drop { force, why } => op_drop(&mut doc, *force, why, now)?,
        Request::Stop { target, reason } => op_stop(&mut doc, target, reason)?,
        Request::Pick { target } => op_pick(&mut doc, target)?,
        Request::Go => op_go(&mut doc)?,
        Request::NoteAdd { target, text } => op_note_add(&mut doc, target, text)?,
        Request::NoteReplace { target, text } => op_note_replace(&mut doc, target, text)?,
        Request::NoteClear { target } => op_note_clear(&mut doc, target)?,
        Request::Rename { target, text } => op_rename(&mut doc, target, text)?,
        Request::GoalNew { title, stay } => op_goal_new(&mut doc, title, *stay, now)?,
        Request::GoalSwitch { id } => op_goal_switch(&mut doc, *id)?,
        Request::GoalReopen { id } => op_goal_reopen(&mut doc, *id)?,
        Request::GoalDone { force } => op_goal_done(&mut doc, *force, now)?,
        Request::GoalRename { title } => op_goal_rename(&mut doc, title)?,
        Request::Jot { text } => op_jot(&mut doc, text, now)?,
        Request::InboxDrop { id } => op_inbox_drop(&mut doc, *id)?,
        Request::InboxTake {
            id,
            goal,
            new_goal,
            parent,
            why,
            switch,
        } => op_inbox_take(
            &mut doc,
            *id,
            *goal,
            new_goal.as_deref(),
            *parent,
            why,
            *switch,
            now,
        )?,
        Request::Schedule { target, at } => op_schedule(&mut doc, target, at.clone())?,
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

fn push_why(task: &mut Task, why: &Option<String>) {
    if let Some(why) = why {
        let why = why.trim();
        if !why.is_empty() {
            task.note.push(why.to_string());
        }
    }
}

fn find_task_text(goal: &Goal, id: TaskId) -> String {
    goal.find(id).map(|t| t.text.clone()).unwrap_or_default()
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

fn push_why_message(messages: &mut Vec<Message>, goal: &Goal, id: TaskId) {
    if let Some(task) = goal.find(id) {
        for line in &task.note {
            messages.push(Message::dim(format!("  why: {line}")));
        }
    }
}

fn op_need(
    doc: &mut Doc,
    text: &str,
    why: &Option<String>,
    target: &Target,
    stay: bool,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let parent = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        match target {
            Target::Task(id) => {
                let task = goal.find(*id).ok_or(OpError::TargetNotFound(*id))?;
                if !task.state.is_unfinished() {
                    return Err(OpError::TargetNotActionable);
                }
                Some(*id)
            }
            Target::Current => goal.cursor,
        }
    };

    let id = doc.alloc_task();
    let mut task = Task::new(id, text.trim().to_string(), now);
    push_why(&mut task, why);

    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    match parent {
        Some(parent_id) => {
            let parent = goal.find_mut(parent_id).expect("parent exists");
            parent.children.push(task);
        }
        None => goal.roots.push(task),
    }

    if goal.cursor.is_none() || (!stay && matches!(target, Target::Current)) {
        // New task is a fresh task; no landing needed.
        goal.cursor = Some(id);
    }
    Ok(Outcome::status())
}

fn op_also(
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
    if goal.cursor.is_none() {
        goal.cursor = Some(id);
    }
    let mut outcome = Outcome::status();
    outcome
        .messages
        .push(Message::normal(format!("added: {}", text.trim())));
    Ok(outcome)
}

fn op_then(
    doc: &mut Doc,
    text: &str,
    why: &Option<String>,
    target: &Target,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let target_id = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        match target {
            Target::Current => goal.cursor.ok_or(OpError::NoCurrentTask)?,
            Target::Task(id) => {
                let t = goal.find(*id).ok_or(OpError::TargetNotFound(*id))?;
                if !t.state.is_unfinished() {
                    return Err(OpError::TargetNotActionable);
                }
                *id
            }
        }
    };

    let id = doc.alloc_task();
    let mut new_task = Task::new(id, text.trim().to_string(), now);
    push_why(&mut new_task, why);

    let path = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        tree::find_path(goal, target_id).ok_or(OpError::TargetNotFound(target_id))?
    };

    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    if path.len() == 1 {
        let idx = path[0];
        let old = goal.roots.remove(idx);
        new_task.children.push(old);
        goal.roots.insert(idx, new_task);
    } else {
        let last = *path.last().expect("non-empty path");
        let parent = tree::task_at_mut(goal, &path[..path.len() - 1]).expect("parent exists");
        let old = parent.children.remove(last);
        new_task.children.push(old);
        parent.children.insert(last, new_task);
    }

    let mut outcome = Outcome::status();
    if path.len() > 1 {
        let goal = doc.goal(goal_id).expect("active goal exists");
        if let Some(grandparent_id) = tree::parent_of(goal, target_id).flatten() {
            outcome.messages.push(Message::normal(format!(
                "also under: {}",
                find_task_text(goal, grandparent_id)
            )));
        }
    }
    Ok(outcome)
}

fn op_add(
    doc: &mut Doc,
    text: &str,
    why: &Option<String>,
    target: &Target,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let target_id = {
        let goal = doc.goal(goal_id).expect("active goal exists");
        match target {
            Target::Current => goal.cursor.ok_or(OpError::NoCurrentTask)?,
            Target::Task(id) => {
                let t = goal.find(*id).ok_or(OpError::TargetNotFound(*id))?;
                if !t.state.is_unfinished() {
                    return Err(OpError::TargetNotActionable);
                }
                *id
            }
        }
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
                        push_why_message(&mut messages, goal, leaf);
                    }
                    leaf
                } else {
                    messages.push(Message::normal(format!(
                        "back to: {}",
                        find_task_text(goal, parent)
                    )));
                    push_why_message(&mut messages, goal, parent);
                    Some(parent)
                }
            }
            None => match tree::next_open_root_after(goal, current) {
                Some(root) => {
                    let leaf = goal
                        .find(root)
                        .and_then(Task::first_open_leaf)
                        .unwrap_or(root);
                    messages.push(Message::normal(format!(
                        "next top-level task: {}",
                        find_task_text(goal, root)
                    )));
                    let _ = leaf;
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
        messages.push(Message::normal(
            "goal has no open tasks. when you are done: trk goal done",
        ));
    }

    Ok(Outcome {
        messages,
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn op_drop(
    doc: &mut Doc,
    force: bool,
    why: &Option<String>,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
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
        mark_subtree_dropped(task, now);
        task.stop_reason = None;
        if let Some(why) = why {
            let why = why.trim();
            if !why.is_empty() {
                task.note.push(format!("dropped: {why}"));
            }
        }
    }

    let mut messages = Vec::new();
    let next = match tree::nearest_open_ancestor(goal, current) {
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
                    push_why_message(&mut messages, goal, leaf);
                }
                leaf
            } else {
                messages.push(Message::normal(format!(
                    "back to: {}",
                    find_task_text(goal, parent)
                )));
                push_why_message(&mut messages, goal, parent);
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
    };

    match next {
        Some(id) => move_cursor(goal, id, &mut messages),
        None => goal.cursor = None,
    }

    if goal.cursor.is_none() && !goal.has_open_tasks() {
        messages.push(Message::normal(
            "goal has no open tasks. when you are done: trk goal done",
        ));
    }

    Ok(Outcome {
        messages,
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn mark_subtree_dropped(task: &mut Task, now: DateTime<FixedOffset>) {
    task.state = TaskState::Dropped(now);
    for child in &mut task.children {
        mark_subtree_dropped(child, now);
    }
}

fn op_stop(doc: &mut Doc, target: &Target, reason: &str) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let target_id = resolve_target(doc, target)?;
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    if let Some(task) = goal.find_mut(target_id) {
        task.state = TaskState::Stopped;
        task.stop_reason = Some(reason.trim().to_string());
    }
    Ok(Outcome::status())
}

fn op_pick(doc: &mut Doc, target: &Target) -> Result<Outcome, OpError> {
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

fn op_go(doc: &mut Doc) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let mut messages = Vec::new();
    let mut changed = false;
    {
        let goal = doc.goal_mut(goal_id).expect("active goal exists");
        let current = goal.cursor.ok_or(OpError::NoCurrentTask)?;
        if let Some(task) = goal.find_mut(current)
            && task.state == TaskState::Stopped
        {
            if let Some(reason) = task.stop_reason.take()
                && !reason.is_empty()
            {
                messages.push(Message::normal(format!("stopped earlier: {reason}")));
            }
            task.state = TaskState::Open;
            changed = true;
        }
        push_why_message(&mut messages, goal, current);
    }
    Ok(Outcome {
        messages,
        hint: Some(ViewHint::Why),
        changed,
    })
}

fn op_note_add(doc: &mut Doc, target: &Target, text: &str) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let id = resolve_target(doc, target)?;
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    if let Some(task) = goal.find_mut(id) {
        task.note.push(text.trim().to_string());
    }
    let mut outcome = Outcome::status();
    outcome.messages.push(Message::normal("noted."));
    Ok(outcome)
}

fn op_note_replace(doc: &mut Doc, target: &Target, text: &str) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let id = resolve_target(doc, target)?;
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    if let Some(task) = goal.find_mut(id) {
        task.note = vec![text.trim().to_string()];
    }
    let mut outcome = Outcome::status();
    outcome.messages.push(Message::normal("noted."));
    Ok(outcome)
}

fn op_note_clear(doc: &mut Doc, target: &Target) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let id = resolve_target(doc, target)?;
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    if let Some(task) = goal.find_mut(id) {
        task.note.clear();
    }
    let mut outcome = Outcome::status();
    outcome.messages.push(Message::normal("noted."));
    Ok(outcome)
}

fn op_rename(doc: &mut Doc, target: &Target, text: &str) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let id = resolve_target(doc, target)?;
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    if let Some(task) = goal.find_mut(id) {
        task.text = text.trim().to_string();
    }
    Ok(Outcome::status())
}

fn op_goal_new(
    doc: &mut Doc,
    title: &str,
    stay: bool,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
    let id = doc.alloc_goal();
    doc.goals.push(Goal {
        id,
        title: title.trim().to_string(),
        status: GoalStatus::Open,
        created: now,
        cursor: None,
        roots: Vec::new(),
    });
    if !stay {
        doc.active = Some(id);
    }
    let mut outcome = Outcome::status();
    outcome
        .messages
        .push(Message::normal(format!("goal: {}", title.trim())));
    Ok(outcome)
}

fn op_goal_switch(doc: &mut Doc, id: u64) -> Result<Outcome, OpError> {
    let goal = doc.goal(id).ok_or(OpError::GoalNotFound(id))?;
    if !goal.is_open() {
        return Err(OpError::GoalNotFound(id));
    }
    doc.active = Some(id);
    Ok(Outcome::status())
}

fn op_goal_reopen(doc: &mut Doc, id: u64) -> Result<Outcome, OpError> {
    let goal = doc.goal_mut(id).ok_or(OpError::GoalNotFound(id))?;
    goal.status = GoalStatus::Open;
    doc.active = Some(id);
    Ok(Outcome::status())
}

fn op_goal_done(
    doc: &mut Doc,
    force: bool,
    now: DateTime<FixedOffset>,
) -> Result<Outcome, OpError> {
    let id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let goal = doc.goal(id).ok_or(OpError::GoalNotFound(id))?;
    let open = goal.count_open();
    if open > 0 && !force {
        return Err(OpError::GoalBlocked {
            title: goal.title.clone(),
            count: open,
        });
    }
    let title = goal.title.clone();
    if let Some(goal) = doc.goal_mut(id) {
        goal.status = GoalStatus::Done(now);
        goal.cursor = None;
    }

    // Next open goal in file order, wrapping.
    let n = doc.goals.len();
    let start = doc.goals.iter().position(|g| g.id == id).unwrap_or(0);
    let mut next = None;
    for offset in 1..=n {
        let idx = (start + offset) % n;
        if doc.goals[idx].is_open() {
            next = Some(doc.goals[idx].id);
            break;
        }
    }
    doc.active = next;

    Ok(Outcome {
        messages: vec![Message::normal(format!("goal done: {title}"))],
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn op_goal_rename(doc: &mut Doc, title: &str) -> Result<Outcome, OpError> {
    let id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let goal = doc.goal_mut(id).ok_or(OpError::GoalNotFound(id))?;
    goal.title = title.trim().to_string();
    Ok(Outcome::status())
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
    why: &Option<String>,
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
            doc.goals.push(Goal {
                id: goal_id,
                title: title.trim().to_string(),
                status: GoalStatus::Open,
                created: now,
                cursor: None,
                roots: Vec::new(),
            });
            goal_id
        }
        (None, None) => return Err(OpError::NoActiveGoal),
    };

    let task_id = doc.alloc_task();
    let mut task = Task::new(task_id, item.text.clone(), now);
    push_why(&mut task, why);

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
    if switch {
        doc.active = Some(goal_id);
    }
    let goal = doc.goal_mut(goal_id).expect("goal exists");
    if goal.cursor.is_none() {
        goal.cursor = Some(task_id);
    }

    Ok(Outcome {
        messages: vec![Message::normal(format!("taken: {}", item.text))],
        hint: Some(ViewHint::Status),
        changed: true,
    })
}

fn op_schedule(doc: &mut Doc, target: &Target, at: Option<Schedule>) -> Result<Outcome, OpError> {
    let goal_id = doc.active.ok_or(OpError::NoActiveGoal)?;
    let id = resolve_target(doc, target)?;
    let goal = doc.goal_mut(goal_id).expect("active goal exists");
    if let Some(task) = goal.find_mut(id) {
        task.at = at;
    }
    Ok(Outcome::status())
}
