//! Concurrency and undo-depth tests (spec 12.8, A11).

use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;

use tempfile::TempDir;
use trk::store::format::parse;

fn run(store: &Path, config: &Path, args: &[&str]) {
    let status = Command::new(env!("CARGO_BIN_EXE_trk"))
        .args(args)
        .env("TRK_STORE", store)
        .env("TRK_CONFIG", config)
        .env("TRK_NOW", "2026-10-07T09:00:00-05:00")
        .env("TRK_ASCII", "1")
        .env_remove("NO_COLOR")
        .status()
        .unwrap();
    assert!(status.success(), "command {:?} failed", args);
}

#[test]
fn concurrent_mutations_do_not_corrupt_the_store() {
    let dir = TempDir::new().unwrap();
    let store = dir.path().join("tasks.trk");
    let config = dir.path().join("config.toml");
    run(&store, &config, &["goal", "new", "G"]);

    let threads: Vec<_> = (0..2)
        .map(|worker| {
            let store = store.clone();
            let config = config.clone();
            thread::spawn(move || {
                for i in 0..15 {
                    run(&store, &config, &["by", &format!("w{worker}-{i}")]);
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }

    let doc = parse(&fs::read_to_string(&store).unwrap()).expect("store stays parseable");
    assert_eq!(
        doc.active_goal().unwrap().roots.len(),
        30,
        "all writes applied"
    );
}

#[test]
fn undo_depth_is_respected() {
    let dir = TempDir::new().unwrap();
    let store = dir.path().join("tasks.trk");
    let config = dir.path().join("config.toml");
    let config_text = format!(
        "store = {:?}\nbackups = {:?}\nundo_depth = 3\n",
        store.to_string_lossy(),
        dir.path().join("backups").to_string_lossy()
    );
    fs::write(&config, config_text).unwrap();

    run(&store, &config, &["goal", "new", "G"]);
    for i in 0..5 {
        run(&store, &config, &["by", &format!("t{i}")]);
    }
    for _ in 0..3 {
        run(&store, &config, &["undo"]);
    }
    // The fourth undo has nothing left.
    let output = Command::new(env!("CARGO_BIN_EXE_trk"))
        .args(["undo"])
        .env("TRK_STORE", &store)
        .env("TRK_CONFIG", &config)
        .env("TRK_NOW", "2026-10-07T09:00:00-05:00")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("nothing to undo"),
        "expected empty ring"
    );
}
