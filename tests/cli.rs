//! CLI integration tests using `assert_cmd`.

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use trk::store::format::parse;

struct Env {
    dir: TempDir,
}

impl Env {
    fn new() -> Self {
        Self {
            dir: TempDir::new().unwrap(),
        }
    }

    fn store(&self) -> PathBuf {
        self.dir.path().join("tasks.trk")
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("trk").unwrap();
        cmd.env("TRK_STORE", self.store())
            .env("TRK_CONFIG", self.dir.path().join("config.toml"))
            .env("TRK_NOW", "2026-10-07T09:00:00-05:00")
            .env("TRK_ASCII", "1")
            .env_remove("NO_COLOR");
        cmd
    }

    fn run(&self, args: &[&str]) -> assert_cmd::assert::Assert {
        let mut cmd = self.cmd();
        cmd.args(args);
        cmd.assert()
    }

    fn doc(&self) -> trk::model::Doc {
        parse(&fs::read_to_string(self.store()).unwrap()).unwrap()
    }
}

#[test]
fn add_then_and_build_a_tree() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "a"]).success();
    env.run(&["then", "b"]).success();
    env.run(&["and", "c"]).success();

    let doc = env.doc();
    let goal = doc.active_goal().unwrap();
    // t1 "a" is under t2 "b"; t3 "c" is a sibling of b.
    assert!(goal.find(2).unwrap().find(1).is_some());
    assert_eq!(goal.roots.len(), 2);
    assert_eq!(goal.roots[0].id, 2);
    assert_eq!(goal.roots[1].id, 3);
    assert_eq!(goal.last_added, Some(3));
}

#[test]
fn then_next_wraps_the_current_task() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "a"]).success();
    // The cursor stays on "a"; -n wraps the current task.
    env.run(&["then", "-n", "d"]).success();

    let doc = env.doc();
    let goal = doc.active_goal().unwrap();
    assert_eq!(goal.cursor, Some(1));
    assert_eq!(goal.roots[0].id, 2);
    assert!(goal.find(2).unwrap().find(1).is_some());
}

#[test]
fn list_shows_the_current_context() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "root"]).success();
    env.run(&["then", "parent"]).success();
    env.run(&["list"])
        .success()
        .stdout(predicate::str::contains("parent"))
        .stdout(predicate::str::contains("root"))
        .stdout(predicate::str::contains(">"));
}

#[test]
fn add_records_the_why() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "ship it", "-w", "because reasons"])
        .success()
        .stdout(predicate::str::contains("why: because reasons"));
    let task = env.doc().active_goal().unwrap().find(1).cloned().unwrap();
    assert_eq!(task.note, vec!["because reasons".to_string()]);
}

#[test]
fn before_inserts_between_tasks() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "a"]).success();
    env.run(&["add", "b"]).success();
    env.run(&["before", "c"]).success();
    let doc = env.doc();
    let goal = doc.active_goal().unwrap();
    let texts: Vec<&str> = goal.roots.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(texts, vec!["a", "c", "b"]);
}

#[test]
fn drop_with_subtasks_needs_confirmation() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "child"]).success();
    env.run(&["then", "parent"]).success();
    // Row 1 is the parent, which still has the "child" sub-task.
    env.cmd()
        .write_stdin("1\n")
        .args(["drop"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--yes"));
    assert_eq!(env.doc().active_goal().unwrap().roots.len(), 1);
}

#[test]
fn drop_with_yes_deletes_the_subtree() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "child"]).success();
    env.run(&["then", "parent"]).success();
    env.cmd()
        .write_stdin("1\n")
        .args(["drop", "--yes"])
        .assert()
        .success();
    let doc = env.doc();
    let goal = doc.active_goal().unwrap();
    assert!(goal.find(1).is_none(), "child gone");
    assert!(goal.find(2).is_none(), "parent gone");
}

#[test]
fn done_completes_the_current_task() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "finish me"]).success();
    env.run(&["done"]).success();
    let doc = env.doc();
    assert!(doc.active_goal().unwrap().find(1).unwrap().state.is_done());
}

#[test]
fn time_and_stop_accumulate() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "timed"]).success();

    let mut start = env.cmd();
    start
        .env("TRK_NOW", "2026-10-07T10:00:00-05:00")
        .args(["time"])
        .assert()
        .success()
        .stdout(predicate::str::contains("timing"));

    let mut stop = env.cmd();
    stop.env("TRK_NOW", "2026-10-07T10:05:00-05:00")
        .args(["stop"])
        .assert()
        .success()
        .stdout(predicate::str::contains("stopped"));

    let task = &env.doc().active_goal().unwrap().find(1).cloned().unwrap();
    assert_eq!(task.spent, 300);
    assert!(task.running.is_none());
}

#[test]
fn stop_without_a_timer_exits_4() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "idle"]).success();
    env.run(&["stop"]).code(4);
}

#[test]
fn drop_deletes_the_picked_task() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "keep"]).success();
    env.run(&["add", "delete me"]).success();
    env.cmd()
        .write_stdin("2\n")
        .args(["drop"])
        .assert()
        .success();
    let doc = env.doc();
    let goal = doc.active_goal().unwrap();
    assert!(goal.find(1).is_some());
    assert!(goal.find(2).is_none(), "t2 removed");
}

#[test]
fn switch_uses_the_picker() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "one"]).success();
    env.run(&["add", "two"]).success();
    env.cmd()
        .write_stdin("2\n")
        .args(["switch"])
        .assert()
        .success();
    assert_eq!(env.doc().active_goal().unwrap().cursor, Some(2));
}

#[test]
fn switch_goal_uses_the_picker() {
    let env = Env::new();
    env.run(&["new", "goal", "A"]).success();
    env.run(&["new", "goal", "B"]).success();
    env.cmd()
        .write_stdin("1\n")
        .args(["switch", "goal"])
        .assert()
        .success();
    assert_eq!(env.doc().active, Some(1));
}

#[test]
fn new_goal_without_a_title_needs_a_terminal() {
    let env = Env::new();
    env.run(&["new", "goal"]).code(2);
}

#[test]
fn list_goal_numbers_every_goal() {
    let env = Env::new();
    env.run(&["new", "goal", "One"]).success();
    env.run(&["new", "goal", "Two"]).success();
    env.run(&["list", "goal"])
        .success()
        .stdout(predicate::str::contains("One"))
        .stdout(predicate::str::contains("Two"));
}

#[test]
fn progress_shows_the_whole_tree() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "child"]).success();
    env.run(&["then", "parent"]).success();
    env.run(&["progress", "goal"])
        .success()
        .stdout(predicate::str::contains("parent"))
        .stdout(predicate::str::contains("child"));
}

#[test]
fn delete_goal_removes_it_and_its_tasks() {
    let env = Env::new();
    env.run(&["new", "goal", "A"]).success();
    env.run(&["add", "a task"]).success();
    env.run(&["new", "goal", "B"]).success();
    env.cmd()
        .write_stdin("1\n")
        .args(["delete", "goal", "--yes"])
        .assert()
        .success();
    let doc = env.doc();
    assert_eq!(doc.goals.len(), 1);
    assert_eq!(doc.goals[0].title, "B");
}

#[test]
fn delete_goal_without_yes_needs_a_terminal() {
    let env = Env::new();
    env.run(&["new", "goal", "A"]).success();
    env.cmd()
        .write_stdin("1\n")
        .args(["delete", "goal"])
        .assert()
        .code(2);
}

#[test]
fn blocked_done_exits_3_and_leaves_store_unchanged() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "child"]).success();
    env.run(&["then", "parent"]).success();
    // Switch to the parent, which still has an open child.
    env.cmd()
        .write_stdin("1\n")
        .args(["switch"])
        .assert()
        .success();
    let before = fs::read_to_string(env.store()).unwrap();
    env.run(&["done"])
        .code(3)
        .stderr(predicate::str::contains("open sub-tasks"));
    assert_eq!(fs::read_to_string(env.store()).unwrap(), before);
}

#[test]
fn done_without_a_current_task_exits_4() {
    let env = Env::new();
    env.run(&["new", "goal", "Empty"]).success();
    env.run(&["done"]).code(4);
}

#[test]
fn jot_and_inbox_drill_down() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "root"]).success();
    env.run(&["jot", "file me"]).success();
    // item 1, goal 1, then insert as a child of the "root" task (row 2).
    env.cmd()
        .write_stdin("1\n1\n2\n")
        .args(["inbox"])
        .assert()
        .success();
    let doc = env.doc();
    let goal = doc.active_goal().unwrap();
    assert!(
        goal.find(1).unwrap().find(2).is_some(),
        "the item became a child of root"
    );
}

#[test]
fn log_shows_completed_task() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "Write repro steps"]).success();
    env.run(&["done"]).success();
    env.run(&["log"])
        .success()
        .stdout(predicate::str::contains("Write repro steps"))
        .stdout(predicate::str::contains("done"));
}

#[test]
fn undo_restores_the_previous_store() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    env.run(&["add", "one"]).success();
    env.run(&["add", "two"]).success();
    env.run(&["undo"])
        .success()
        .stdout(predicate::str::contains("undid: add"));
    assert_eq!(env.doc().active_goal().unwrap().roots.len(), 1);
}

#[test]
fn parse_error_exits_1_and_leaves_file_untouched() {
    let env = Env::new();
    let broken = "trk 1\nnext g=1 t=2 i=1\n\ngoal g1 open 2026-10-07T09:00:00-05:00 G\n  [ ] t1 2026-10-07T09:01:00-05:00 x\n\t| tab\n";
    fs::write(env.store(), broken).unwrap();
    env.run(&["list"])
        .code(1)
        .stderr(predicate::str::is_match("line \\d+").unwrap());
    assert_eq!(fs::read_to_string(env.store()).unwrap(), broken);
}

#[test]
fn no_color_emits_no_ansi() {
    let env = Env::new();
    env.run(&["new", "goal", "G"]).success();
    let output = env
        .cmd()
        .env("NO_COLOR", "1")
        .args(["--color", "auto"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains('\u{1b}'), "no escape codes expected");
}

#[test]
fn unknown_subcommand_exits_2() {
    let env = Env::new();
    env.run(&["frobnicate"]).code(2);
}

#[test]
fn wrap_config_limits_output_width() {
    let env = Env::new();
    let config = format!(
        "store = {:?}\nbackups = {:?}\nwrap = 50\n",
        env.store().to_string_lossy(),
        env.dir.path().join("backups").to_string_lossy()
    );
    fs::write(env.dir.path().join("config.toml"), config).unwrap();
    env.run(&["new", "goal", "G"]).success();
    let long = "word ".repeat(40);
    env.run(&["add", &long]).success();
    let output = env.cmd().output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        assert!(
            trk::ui::wrap::width_of(line) <= 50,
            "line wider than 50: {line:?}"
        );
    }
}

#[cfg(unix)]
#[test]
fn completions_install_for_the_detected_shell() {
    let env = Env::new();
    let home = env.dir.path().join("home");
    fs::create_dir_all(&home).unwrap();
    env.cmd()
        .env("HOME", &home)
        .env("SHELL", "/bin/zsh")
        .arg("completions")
        .assert()
        .success()
        .stdout(predicate::str::contains("zsh completions"));
    assert!(home.join(".zfunc/_trk").exists());
}

#[cfg(unix)]
#[test]
fn completions_accept_an_explicit_shell() {
    let env = Env::new();
    let home = env.dir.path().join("home");
    fs::create_dir_all(&home).unwrap();
    env.cmd()
        .env("HOME", &home)
        .env_remove("SHELL")
        .args(["completions", "bash"])
        .assert()
        .success();
    assert!(
        home.join(".local/share/bash-completion/completions/trk")
            .exists()
    );
}

#[test]
fn completions_without_a_shell_is_a_usage_error() {
    let env = Env::new();
    env.cmd()
        .env_remove("SHELL")
        .arg("completions")
        .assert()
        .code(2);
}
