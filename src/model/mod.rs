use chrono::{DateTime, FixedOffset, NaiveDate};

pub mod ops;
pub mod tree;

pub type GoalId = u64;
pub type TaskId = u64;
pub type InboxId = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    pub g: u64,
    pub t: u64,
    pub i: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskState {
    Open,
    Done(DateTime<FixedOffset>),
    Dropped(DateTime<FixedOffset>),
    Stopped,
}

impl TaskState {
    pub fn is_open(&self) -> bool {
        matches!(self, TaskState::Open)
    }

    /// Open or stopped: both count as unfinished.
    pub fn is_unfinished(&self) -> bool {
        matches!(self, TaskState::Open | TaskState::Stopped)
    }

    pub fn is_done(&self) -> bool {
        matches!(self, TaskState::Done(_))
    }

    pub fn is_dropped(&self) -> bool {
        matches!(self, TaskState::Dropped(_))
    }

    pub fn completion(&self) -> Option<DateTime<FixedOffset>> {
        match self {
            TaskState::Done(ts) | TaskState::Dropped(ts) => Some(*ts),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Schedule {
    Date(NaiveDate),
    DateTime(DateTime<FixedOffset>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub id: TaskId,
    pub text: String,
    pub state: TaskState,
    pub created: DateTime<FixedOffset>,
    pub note: Vec<String>,
    pub stop_reason: Option<String>,
    pub at: Option<Schedule>,
    pub children: Vec<Task>,
}

impl Task {
    pub fn new(id: TaskId, text: String, created: DateTime<FixedOffset>) -> Self {
        Self {
            id,
            text,
            state: TaskState::Open,
            created,
            note: Vec::new(),
            stop_reason: None,
            at: None,
            children: Vec::new(),
        }
    }

    pub fn find(&self, id: TaskId) -> Option<&Task> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(id))
    }

    pub fn find_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter_mut().find_map(|c| c.find_mut(id))
    }

    pub fn has_open_descendant(&self) -> bool {
        self.children
            .iter()
            .any(|c| c.state.is_unfinished() || c.has_open_descendant())
    }

    /// First unfinished leaf in document order (depth first).
    pub fn first_open_leaf(&self) -> Option<TaskId> {
        if self.children.is_empty() {
            if self.state.is_unfinished() {
                return Some(self.id);
            }
            return None;
        }
        for child in &self.children {
            if child.state.is_unfinished() {
                if let Some(id) = child.first_open_leaf() {
                    return Some(id);
                }
                return Some(child.id);
            }
        }
        None
    }

    /// Collect up to `limit` unfinished descendants' text, depth first.
    pub fn collect_open_descendants(&self, out: &mut Vec<String>, limit: usize) {
        for child in &self.children {
            if out.len() >= limit {
                return;
            }
            if child.state.is_unfinished() {
                out.push(child.text.clone());
            }
            child.collect_open_descendants(out, limit);
        }
    }

    pub fn count_open(&self) -> usize {
        let self_count = usize::from(self.state.is_unfinished());
        self_count + self.children.iter().map(Task::count_open).sum::<usize>()
    }

    pub fn count_done(&self) -> usize {
        let self_count = usize::from(self.state.is_done()) + usize::from(self.state.is_dropped());
        self_count + self.children.iter().map(Task::count_done).sum::<usize>()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GoalStatus {
    Open,
    Done(DateTime<FixedOffset>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Goal {
    pub id: GoalId,
    pub title: String,
    pub status: GoalStatus,
    pub created: DateTime<FixedOffset>,
    pub cursor: Option<TaskId>,
    pub roots: Vec<Task>,
}

impl Goal {
    pub fn is_open(&self) -> bool {
        matches!(self.status, GoalStatus::Open)
    }

    pub fn find(&self, id: TaskId) -> Option<&Task> {
        self.roots.iter().find_map(|r| r.find(id))
    }

    pub fn find_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        self.roots.iter_mut().find_map(|r| r.find_mut(id))
    }

    /// Depth-first list of every task with its depth.
    pub fn walk(&self) -> Vec<(&Task, usize)> {
        let mut out = Vec::new();
        for root in &self.roots {
            walk_task(root, 0, &mut out);
        }
        out
    }

    pub fn count_open(&self) -> usize {
        self.roots.iter().map(Task::count_open).sum()
    }

    pub fn count_done(&self) -> usize {
        self.roots.iter().map(Task::count_done).sum()
    }

    /// True when the goal has no unfinished tasks anywhere.
    pub fn has_open_tasks(&self) -> bool {
        self.count_open() > 0
    }
}

fn walk_task<'a>(task: &'a Task, depth: usize, out: &mut Vec<(&'a Task, usize)>) {
    out.push((task, depth));
    for child in &task.children {
        walk_task(child, depth + 1, out);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InboxItem {
    pub id: InboxId,
    pub created: DateTime<FixedOffset>,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Doc {
    pub version: u32,
    pub next: Counters,
    pub active: Option<GoalId>,
    pub inbox: Vec<InboxItem>,
    pub goals: Vec<Goal>,
}

impl Default for Doc {
    fn default() -> Self {
        Self::empty()
    }
}

impl Doc {
    pub fn empty() -> Self {
        Self {
            version: 1,
            next: Counters { g: 1, t: 1, i: 1 },
            active: None,
            inbox: Vec::new(),
            goals: Vec::new(),
        }
    }

    pub fn active_goal(&self) -> Option<&Goal> {
        let id = self.active?;
        self.goals.iter().find(|g| g.id == id)
    }

    pub fn active_goal_mut(&mut self) -> Option<&mut Goal> {
        let id = self.active?;
        self.goals.iter_mut().find(|g| g.id == id)
    }

    pub fn goal(&self, id: GoalId) -> Option<&Goal> {
        self.goals.iter().find(|g| g.id == id)
    }

    pub fn goal_mut(&mut self, id: GoalId) -> Option<&mut Goal> {
        self.goals.iter_mut().find(|g| g.id == id)
    }

    pub fn alloc_goal(&mut self) -> GoalId {
        let id = self.next.g;
        self.next.g += 1;
        id
    }

    pub fn alloc_task(&mut self) -> TaskId {
        let id = self.next.t;
        self.next.t += 1;
        id
    }

    pub fn alloc_inbox(&mut self) -> InboxId {
        let id = self.next.i;
        self.next.i += 1;
        id
    }

    /// Check the invariants from section 3. Returns a human-readable reason on
    /// failure; used in tests and debug builds after every write.
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err(format!("unsupported version {}", self.version));
        }

        let mut goal_ids = Vec::new();
        let mut task_ids = Vec::new();
        let mut inbox_ids = Vec::new();

        for item in &self.inbox {
            if inbox_ids.contains(&item.id) {
                return Err(format!("duplicate inbox id i{}", item.id));
            }
            inbox_ids.push(item.id);
        }

        for goal in &self.goals {
            if goal_ids.contains(&goal.id) {
                return Err(format!("duplicate goal id g{}", goal.id));
            }
            goal_ids.push(goal.id);

            for root in &goal.roots {
                validate_task(root, &mut task_ids)?;
            }
            if let Some(cursor) = goal.cursor {
                let valid = goal.find(cursor).is_some_and(|t| t.state.is_unfinished());
                if !valid {
                    return Err(format!(
                        "g{}: cursor t{cursor} is not an open or stopped task",
                        goal.id
                    ));
                }
            }
        }

        if let Some(active) = self.active {
            match self.goals.iter().find(|g| g.id == active) {
                None => return Err(format!("active goal g{active} does not exist")),
                Some(goal) if !goal.is_open() => {
                    return Err(format!("active goal g{active} is not open"));
                }
                _ => {}
            }
        }

        Ok(())
    }
}

fn validate_task(task: &Task, task_ids: &mut Vec<TaskId>) -> Result<(), String> {
    if task_ids.contains(&task.id) {
        return Err(format!("duplicate task id t{}", task.id));
    }
    task_ids.push(task.id);

    match &task.state {
        TaskState::Stopped => {
            if task.stop_reason.is_none() {
                return Err(format!("t{}: stopped task needs a stop reason", task.id));
            }
        }
        _ => {
            if task.stop_reason.is_some() {
                return Err(format!("t{}: stop reason on a non-stopped task", task.id));
            }
        }
    }

    for child in &task.children {
        validate_task(child, task_ids)?;
    }
    Ok(())
}

impl Goal {
    /// Whether `cursor_ok` should consider `id` a valid cursor target.
    pub fn cursor_target_valid(&self, id: TaskId) -> bool {
        self.find(id).is_some_and(|t| t.state.is_unfinished())
    }
}
