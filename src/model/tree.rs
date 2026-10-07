use super::{Goal, Task, TaskId};

/// Path of child indices from the goal's roots to `id`, or `None`.
pub fn find_path(goal: &Goal, id: TaskId) -> Option<Vec<usize>> {
    fn rec(task: &Task, id: TaskId, path: &mut Vec<usize>) -> bool {
        if task.id == id {
            return true;
        }
        for (i, child) in task.children.iter().enumerate() {
            path.push(i);
            if rec(child, id, path) {
                return true;
            }
            path.pop();
        }
        false
    }
    for (i, root) in goal.roots.iter().enumerate() {
        let mut path = vec![i];
        if rec(root, id, &mut path) {
            return Some(path);
        }
    }
    None
}

pub fn task_at<'a>(goal: &'a Goal, path: &[usize]) -> Option<&'a Task> {
    let mut cur = goal.roots.get(*path.first()?)?;
    for &i in &path[1..] {
        cur = cur.children.get(i)?;
    }
    Some(cur)
}

fn task_mut_rec<'a>(task: &'a mut Task, path: &[usize]) -> Option<&'a mut Task> {
    if path.is_empty() {
        return Some(task);
    }
    task.children
        .get_mut(path[0])
        .and_then(|c| task_mut_rec(c, &path[1..]))
}

pub fn task_at_mut<'a>(goal: &'a mut Goal, path: &[usize]) -> Option<&'a mut Task> {
    let (&first, rest) = path.split_first()?;
    goal.roots
        .get_mut(first)
        .and_then(|r| task_mut_rec(r, rest))
}

/// `None` if the task does not exist; `Some(None)` for a root; otherwise
/// `Some(Some(parent))`.
pub fn parent_of(goal: &Goal, id: TaskId) -> Option<Option<TaskId>> {
    fn rec(task: &Task, id: TaskId, parent: Option<TaskId>) -> Option<Option<TaskId>> {
        if task.id == id {
            return Some(parent);
        }
        for child in &task.children {
            if let Some(found) = rec(child, id, Some(task.id)) {
                return Some(found);
            }
        }
        None
    }
    for root in &goal.roots {
        if let Some(found) = rec(root, id, None) {
            return Some(found);
        }
    }
    None
}

/// Nearest ancestor that is open or stopped.
pub fn nearest_open_ancestor(goal: &Goal, id: TaskId) -> Option<TaskId> {
    let path = find_path(goal, id)?;
    let mut best = None;
    for len in 1..path.len() {
        if let Some(anc) = task_at(goal, &path[..len])
            && anc.state.is_unfinished()
        {
            best = Some(anc.id);
        }
    }
    best
}

/// The next root after the root containing `id` that is unfinished, wrapping
/// around; used by `done`/`drop` cursor movement.
pub fn next_open_root_after(goal: &Goal, id: TaskId) -> Option<TaskId> {
    let path = find_path(goal, id)?;
    let start = path[0];
    let n = goal.roots.len();
    for offset in 1..=n {
        let idx = (start + offset) % n;
        if goal.roots[idx].state.is_unfinished() {
            return Some(goal.roots[idx].id);
        }
    }
    None
}

/// Unfinished tasks in `trk list` order (depth first).
pub fn unfinished_tasks(goal: &Goal) -> Vec<TaskId> {
    let mut out = Vec::new();
    for (task, _) in goal.walk() {
        if task.state.is_unfinished() {
            out.push(task.id);
        }
    }
    out
}
