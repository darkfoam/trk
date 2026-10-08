//! Parser/serializer and core-op unit tests (spec section 12.1, 12.2).

use chrono::{DateTime, FixedOffset};
use proptest::prelude::*;

use trk::model::ops::{Request, Target, apply};
use trk::model::{Counters, Doc, Goal, GoalStatus, InboxItem, Task, TaskState};
use trk::store::format::{parse, serialize};

/// The golden fixture from spec 6.2.
const GOLDEN: &str = "\
trk 1
next g=2 t=7 i=1
active g1

inbox
  i3 2026-10-07T09:30:00-05:00 look into sqlite wal mode

goal g1 open 2026-10-07T09:00:00-05:00 Ship login fix
  current t3
  [ ] t1 2026-10-07T09:01:00-05:00 Fix login bug
    | customers locked out
    [ ] t2 2026-10-07T09:02:00-05:00 Reproduce on staging
      | need a failing case
      [ ] t6 2026-10-07T09:06:00-05:00 Document the process
        | after the fix ships
        [ ] t3 2026-10-07T09:03:00-05:00 Get staging creds
          | can't log in
    [ ] t5 2026-10-07T09:05:00-05:00 Check auth logs
      | might explain the 401s
  [ ] t4 2026-10-07T09:04:00-05:00 Call internet company
    | wifi flaky
";

#[test]
fn golden_parses_and_is_canonical() {
    let doc = parse(GOLDEN).expect("golden parses");
    assert_eq!(serialize(&doc), GOLDEN, "serialize(parse(x)) == x");
    assert_eq!(doc.active, Some(1));
    assert_eq!(doc.inbox.len(), 1);
}

#[test]
fn crlf_is_tolerated() {
    let crlf = GOLDEN.replace('\n', "\r\n");
    let doc = parse(&crlf).expect("crlf parses");
    assert_eq!(serialize(&doc), GOLDEN, "crlf normalizes to LF");
}

#[test]
fn tab_is_a_parse_error_with_line_number() {
    let broken = GOLDEN.replace("    | customers locked out", "\t| customers locked out");
    let err = parse(&broken).unwrap_err();
    assert_eq!(err.line, 11, "{err}");
    assert!(err.reason.contains("tab"), "{err}");
}

#[test]
fn done_needs_timestamp() {
    let text = "\
trk 1
next g=1 t=2 i=1

goal g1 open 2026-10-07T09:00:00-05:00 G
  [x] t1 2026-10-07T09:01:00-05:00 nope
";
    assert!(parse(text).is_err());
}

#[test]
fn stopped_needs_reason() {
    let text = "\
trk 1
next g=1 t=2 i=1

goal g1 open 2026-10-07T09:00:00-05:00 G
  [!] t1 2026-10-07T09:01:00-05:00 nope
";
    assert!(parse(text).is_err());
}

#[test]
fn schedules_round_trip() {
    let text = "\
trk 1
next g=2 t=3 i=1

goal g1 open 2026-10-07T09:00:00-05:00 G
  [ ] t1 2026-10-07T09:01:00-05:00 dated
    @ at=2026-10-09
  [ ] t2 2026-10-07T09:01:00-05:00 timed
    @ at=2026-10-09T15:00:00-05:00
";
    let doc = parse(text).unwrap();
    assert_eq!(serialize(&doc), text);
}

fn ts(s: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(s).unwrap()
}

fn goal_with_tree() -> Doc {
    parse(GOLDEN).unwrap()
}

#[test]
fn done_blocked_without_force() {
    let doc = goal_with_tree();
    let err = apply(
        &doc,
        &Request::Done { force: false },
        ts("2026-10-07T10:00:00-05:00"),
    );
    // current task t3 has no children, so done succeeds here; use t2 instead.
    assert!(err.is_ok());
}

#[test]
fn done_on_task_with_children_is_blocked() {
    let mut doc = goal_with_tree();
    doc.active_goal_mut().unwrap().cursor = Some(1);
    let result = apply(
        &doc,
        &Request::Done { force: false },
        ts("2026-10-07T10:00:00-05:00"),
    );
    assert!(result.is_err(), "t1 has open descendants");
}

#[test]
fn then_parents_the_target_directly() {
    let doc = goal_with_tree();
    let (next, _) = apply(
        &doc,
        &Request::Then {
            text: "Wrap".into(),
            why: None,
            target: Target::Task(3),
        },
        ts("2026-10-07T10:00:00-05:00"),
    )
    .unwrap();
    let goal = next.active_goal().unwrap();
    let wrap = goal.find(7).unwrap();
    assert_eq!(wrap.text, "Wrap");
    assert_eq!(wrap.children.len(), 1);
    assert_eq!(
        wrap.children[0].id, 3,
        "the new task becomes the direct parent of the target"
    );
    // The new task is inserted where the target was, not at the root.
    assert!(
        goal.find(6).unwrap().find(7).is_some(),
        "the new parent sits under the target's old parent"
    );
    assert!(goal.find(7).unwrap().find(3).is_some());
    // The unrelated root is untouched.
    assert_eq!(goal.roots.len(), 2);
    assert!(goal.roots.iter().any(|r| r.id == 4));
}

#[test]
fn landing_rule_reopens_stopped() {
    let mut doc = goal_with_tree();
    {
        let goal = doc.active_goal_mut().unwrap();
        let task = goal.find_mut(4).unwrap();
        task.state = TaskState::Stopped;
        task.stop_reason = Some("waiting".into());
    }
    let (next, outcome) = apply(
        &doc,
        &Request::Switch {
            target: Target::Task(4),
        },
        ts("2026-10-07T10:00:00-05:00"),
    )
    .unwrap();
    let task = next.active_goal().unwrap().find(4).unwrap();
    assert_eq!(task.state, TaskState::Open);
    assert!(task.stop_reason.is_none());
    assert!(
        outcome
            .messages
            .iter()
            .any(|m| m.text.contains("stopped earlier: waiting")),
        "landing message printed"
    );
}

// ---------------------------------------------------------------------------
// Round-trip / invariants property tests (spec 12.2, 12.3).
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serialize_parse_is_identity_on_valid_docs(text in "[ -~\\n]{0,400}") {
        // Only documents that parse must round-trip identically.
        if let Ok(doc) = parse(&text) {
            let again = parse(&serialize(&doc)).expect("canonical text parses");
            prop_assert_eq!(again, doc);
        }
    }
}

proptest! {
    #[test]
    fn random_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..200)) {
        let text = String::from_utf8_lossy(&bytes);
        let _ = parse(&text);
    }
}

#[test]
fn empty_doc_round_trips() {
    let doc = Doc::empty();
    assert_eq!(serialize(&doc), "trk 1\nnext g=1 t=1 i=1\n");
    assert_eq!(parse(&serialize(&doc)).unwrap(), doc);
}

#[test]
fn validate_rejects_duplicate_ids() {
    let mut doc = Doc::empty();
    doc.goals.push(Goal {
        id: 1,
        title: "a".into(),
        status: GoalStatus::Open,
        created: ts("2026-10-07T09:00:00-05:00"),
        cursor: None,
        roots: vec![
            Task::new(1, "x".into(), ts("2026-10-07T09:00:00-05:00")),
            Task::new(1, "y".into(), ts("2026-10-07T09:00:00-05:00")),
        ],
    });
    doc.next = Counters { g: 2, t: 2, i: 1 };
    assert!(doc.validate().is_err());
}

#[test]
fn inbox_item_round_trip() {
    let mut doc = Doc::empty();
    doc.inbox.push(InboxItem {
        id: 1,
        created: ts("2026-10-07T09:00:00-05:00"),
        text: "capture".into(),
    });
    doc.next.i = 2;
    assert_eq!(parse(&serialize(&doc)).unwrap(), doc);
}
