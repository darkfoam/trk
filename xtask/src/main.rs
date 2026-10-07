#![forbid(unsafe_code)]
//! Developer tasks: regenerate the man page and shell completions from the
//! same clap definitions the binary uses (spec 11.2, appendix C).

use std::fs;
use std::path::Path;

use clap::CommandFactory;
use clap_complete::{Shell, generate};
use clap_mangen::Man;

fn main() {
    let task = std::env::args().nth(1).unwrap_or_default();
    match task.as_str() {
        "man" => gen_man(),
        "completions" => gen_completions(),
        _ => {
            eprintln!("usage: xtask <man|completions>");
            std::process::exit(2);
        }
    }
}

fn gen_man() {
    // The installed `man/trk.1` is hand\-written (it is a full usage book, not
    // a flag dump). The generated skeleton is written beside it so the flag
    // list can be diffed for drift.
    let cmd = trk::cli::Cli::command();
    let man = Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer).expect("render man page");
    write_file(Path::new("man/trk.1.generated"), &buffer);
    println!("wrote man/trk.1.generated (hand-written man/trk.1 is authoritative)");
}

fn gen_completions() {
    let mut cmd = trk::cli::Cli::command();
    let shells = [
        (Shell::Bash, "completions/trk.bash"),
        (Shell::Zsh, "completions/_trk"),
        (Shell::Fish, "completions/trk.fish"),
        (Shell::PowerShell, "completions/trk.ps1"),
        (Shell::Elvish, "completions/trk.elv"),
    ];
    for (shell, path) in shells {
        let mut buffer = Vec::new();
        generate(shell, &mut cmd, "trk", &mut buffer);
        write_file(Path::new(path), &buffer);
        println!("wrote {path}");
    }
}

fn write_file(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create output directory");
    }
    fs::write(path, bytes).expect("write output file");
}
