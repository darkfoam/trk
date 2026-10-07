#![forbid(unsafe_code)]
//! `trk`: adhd compliant task tracking.

pub mod cli;
pub mod clock;
pub mod commands;
pub mod config;
pub mod ctx;
pub mod error;
pub mod interact;
pub mod model;
pub mod store;
pub mod tui;
pub mod ui;

use cli::Cli;

/// Entry point used by `main`. Returns the process exit code.
pub fn run(cli: Cli) -> i32 {
    match commands::dispatch(cli) {
        Ok(code) => code,
        Err(err) => {
            if let Some(warning) = err.blocked_warning() {
                eprintln!("warning: {warning}");
                return 3;
            }
            eprintln!("trk: {err}");
            err.exit_code()
        }
    }
}
