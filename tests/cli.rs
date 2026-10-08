//! CLI integration tests using `assert_cmd` (spec 12.5, 13).

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

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
}

fn build_main_scenario(env: &Env) {
    env.run(&["goal", "new", "Ship login fix"]).success();
    env.run(&["by", "Fix login bug", "-w", "customers locked out"])
        .success();
    env.run(&["add", "Reproduce on staging", "-w", "need a failing case"])
        .success();
    env.run(&["add", "Get staging creds", "-w", "can't log in"])
        .success();
    env.run(&["by", "Call internet company", "-w", "wifi flaky"])
        .success();
    env.run(&["switch", "1"]).success();
    env.run(&[
        "add",
        "-n",
        "Check auth logs",
        "-w",
        "might explain the 401s",
    ])
    .success();
    env.run(&["switch", "3"]).success();
    env.run(&["then", "Document the process", "-w", "after the fix ships"])
        .success();
}

#[test]
fn main_scenario_steps_1_to_7_produce_golden_fixture() {
    let env = Env::new();
    build_main_scenario(&env);
    let text = fs::read_to_string(env.store())
        .unwrap()
        .replace("\r\n", "\n");
    let golden = std::fs::read_to_string("tests/fixtures/step7.trk")
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(text, golden);
}

#[test]
fn main_scenario_done_and_undo() {
    let env = Env::new();
    build_main_scenario(&env);

    env.run(&["done"])
        .success()
        .stdout(predicate::str::contains("back to: Reproduce on staging"));
    env.run(&["done"])
        .success()
        .stdout(predicate::str::contains("next: Check auth logs"));
    env.run(&["done"])
        .success()
        .stdout(predicate::str::contains("back to: Fix login bug"));
    env.run(&["done"])
        .success()
        .stdout(predicate::str::contains("back to: Document the process"));
    env.run(&["done"])
        .success()
        .stdout(predicate::str::contains("next top-level task"));
    env.run(&["done"])
        .success()
        .stdout(predicate::str::contains("goal has no open tasks"));

    env.run(&["undo"])
        .success()
        .stdout(predicate::str::contains("undid: done"))
        .stdout(predicate::str::contains("Call internet company"));
}

#[test]
fn blocked_done_exits_3_and_leaves_store_unchanged() {
    let env = Env::new();
    build_main_scenario(&env);
    let before = fs::read_to_string(env.store()).unwrap();
    env.run(&["switch", "2"]).success();
    let before_pick = fs::read_to_string(env.store()).unwrap();
    let _ = before;
    env.run(&["done"])
        .code(3)
        .stderr(predicate::str::contains("open sub-tasks"));
    assert_eq!(fs::read_to_string(env.store()).unwrap(), before_pick);
}

#[test]
fn forced_done_marks_only_the_task() {
    let env = Env::new();
    build_main_scenario(&env);
    env.run(&["switch", "2"]).success();
    env.run(&["done", "-f"]).success();
    let text = fs::read_to_string(env.store()).unwrap();
    assert!(text.contains("[x] t1"), "{text}");
}

#[test]
fn no_current_task_exits_4() {
    let env = Env::new();
    env.run(&["goal", "new", "Empty"]).success();
    env.run(&["done"]).code(4);
}

#[test]
fn non_tty_need_creates_task_without_prompt() {
    let env = Env::new();
    env.run(&["goal", "new", "G"]).success();
    let mut cmd = env.cmd();
    cmd.write_stdin("")
        .args(["add", "do the thing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("do the thing"));
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
    env.run(&["goal", "new", "G"]).success();
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
fn jot_and_inbox_round_trip() {
    let env = Env::new();
    env.run(&["jot", "look into wal mode"])
        .success()
        .stdout(predicate::str::contains("jotted"));
    env.run(&["inbox"])
        .success()
        .stdout(predicate::str::contains("look into wal mode"));
}

#[test]
fn prompt_prints_nothing_without_goal() {
    let env = Env::new();
    env.run(&["prompt"])
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn goals_keep_their_own_cursor() {
    let env = Env::new();
    env.run(&["goal", "new", "One"]).success();
    env.run(&["by", "one task"]).success();
    env.run(&["goal", "new", "Two"]).success();
    env.run(&["by", "two task"]).success();
    env.run(&["goal", "switch", "1"])
        .success()
        .stdout(predicate::str::contains("one task"));
    env.run(&["goal", "switch", "2"])
        .success()
        .stdout(predicate::str::contains("two task"));
}

#[test]
fn config_show_and_path() {
    let env = Env::new();
    let config = format!(
        "store = {:?}\nbackups = {:?}\nwrap = 50\n",
        env.store().to_string_lossy(),
        env.dir.path().join("backups").to_string_lossy()
    );
    fs::write(env.dir.path().join("config.toml"), config).unwrap();
    env.run(&["config", "show"])
        .success()
        .stdout(predicate::str::contains("wrap:    50"));
    env.run(&["config", "path"])
        .success()
        .stdout(predicate::str::contains("config.toml"));
}

#[test]
fn at_schedules_and_agenda_lists() {
    let env = Env::new();
    env.run(&["goal", "new", "G"]).success();
    env.run(&["by", "Renew domain"]).success();
    env.run(&["at", "2026-10-09"]).success();
    env.run(&["agenda"])
        .success()
        .stdout(predicate::str::contains("Renew domain"))
        .stdout(predicate::str::contains("2026-10-09"));
}

#[test]
fn at_parses_relative_and_time() {
    let env = Env::new();
    env.run(&["goal", "new", "G"]).success();
    env.run(&["by", "task"]).success();
    env.run(&["at", "tomorrow", "3pm"]).success();
    let text = fs::read_to_string(env.store()).unwrap();
    assert!(text.contains("@ at=2026-10-08T15:00:00-05:00"), "{text}");
}

#[test]
fn at_rejects_bad_grammar_with_exit_2() {
    let env = Env::new();
    env.run(&["goal", "new", "G"]).success();
    env.run(&["by", "task"]).success();
    env.run(&["at", "whenever"]).code(2);
}

#[test]
fn log_shows_completed_task() {
    let env = Env::new();
    env.run(&["goal", "new", "G"]).success();
    env.run(&["by", "Write repro steps"]).success();
    env.run(&["done"]).success();
    env.run(&["log"])
        .success()
        .stdout(predicate::str::contains("Write repro steps"))
        .stdout(predicate::str::contains("done"));
}

#[test]
fn inbox_take_new_goal_creates_goal_and_task() {
    let env = Env::new();
    env.run(&["jot", "look into wal mode"]).success();
    env.run(&["inbox", "take", "1", "--new-goal", "New Goal"])
        .success();
    let text = fs::read_to_string(env.store()).unwrap();
    assert!(text.contains("New Goal"), "{text}");
    assert!(text.contains("look into wal mode"), "{text}");
    assert!(!text.contains("inbox"), "inbox emptied: {text}");
}

#[test]
fn prompt_prints_current_task() {
    let env = Env::new();
    env.run(&["goal", "new", "G"]).success();
    env.run(&["by", "ship it"]).success();
    env.run(&["prompt"])
        .success()
        .stdout(predicate::str::contains("ship it"));
}

#[test]
fn completions_emit_script() {
    let env = Env::new();
    env.run(&["completions", "bash"])
        .success()
        .stdout(predicate::str::contains("trk"));
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
    env.run(&["goal", "new", "G"]).success();
    let long = "word ".repeat(40);
    env.run(&["by", &long]).success();
    let output = env.cmd().output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        assert!(
            trk::ui::wrap::width_of(line) <= 50,
            "line wider than 50: {line:?}"
        );
    }
}

#[test]
fn agenda_shows_todays_upcoming_task() {
    // TRK_NOW pins "now" to 09:00, so 23:00 today is still upcoming and must
    // appear under Today rather than being dropped.
    let env = Env::new();
    env.run(&["goal", "new", "G"]).success();
    env.run(&["by", "Later today"]).success();
    env.run(&["at", "23:00"]).success();
    env.run(&["agenda"])
        .success()
        .stdout(predicate::str::contains("Today"))
        .stdout(predicate::str::contains("Later today"))
        .stdout(predicate::str::contains("23:00"));
}

#[test]
fn goal_switch_and_reopen_share_goal_list_numbers() {
    let env = Env::new();
    env.run(&["goal", "new", "One"]).success();
    env.run(&["by", "one task"]).success();
    env.run(&["done"]).success();
    env.run(&["goal", "done"]).success();
    env.run(&["goal", "new", "Two"]).success();
    env.run(&["by", "two task"]).success();

    // g1 is done and g2 is open; `goal list` numbers both from 1.
    env.run(&["goal", "list"])
        .success()
        .stdout(predicate::str::contains("(done)"));
    // switch refuses the done goal's number, reopen accepts it.
    env.run(&["goal", "switch", "1"]).code(2);
    env.run(&["goal", "reopen", "1"])
        .success()
        .stdout(predicate::str::contains("One"));
    // switching to the open goal's number still works.
    env.run(&["goal", "switch", "2"])
        .success()
        .stdout(predicate::str::contains("two task"));
}

#[test]
fn inbox_take_switch_honours_the_flag() {
    let env = Env::new();
    env.run(&["goal", "new", "G"]).success();
    env.run(&["by", "root task"]).success();
    env.run(&["jot", "filed thing"]).success();
    // With no terminal the picker cannot run, so -s must fail rather than
    // silently creating a root task.
    env.run(&["inbox", "take", "1", "--goal", "G", "-s"])
        .code(1)
        .stderr(predicate::str::contains("picker"));
}

#[test]
fn inbox_take_switch_needs_an_existing_goal() {
    let env = Env::new();
    env.run(&["jot", "thing"]).success();
    env.run(&["inbox", "take", "1", "--new-goal", "X", "-s"])
        .code(2);
}
