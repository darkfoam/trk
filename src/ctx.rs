use std::io::IsTerminal;
use std::path::PathBuf;

use crate::clock::{self, Clock};
use crate::config::{Config, Toggle};
use crate::ui::style::Style;

/// Explicit context passed to commands: config, clock, paths, tty/color
/// settings (spec 11.1). No global mutable state.
pub struct Ctx {
    pub config: Config,
    pub config_path: Option<PathBuf>,
    pub clock: Box<dyn Clock>,
    pub color: bool,
    pub ascii: bool,
    pub wrap: usize,
    pub stdin_tty: bool,
    pub stdout_tty: bool,
    pub stderr_tty: bool,
}

impl Ctx {
    pub fn new(config: Config, config_path: Option<PathBuf>, color_flag: Option<Toggle>) -> Self {
        let stdin_tty = std::io::stdin().is_terminal();
        let stdout_tty = std::io::stdout().is_terminal();
        let stderr_tty = std::io::stderr().is_terminal();
        let color = resolve_color(config.color, color_flag, stdout_tty);
        let ascii = resolve_ascii(config.ascii);
        let wrap = resolve_wrap(config.wrap);
        Self {
            config,
            config_path,
            clock: clock::system_clock(),
            color,
            ascii,
            wrap,
            stdin_tty,
            stdout_tty,
            stderr_tty,
        }
    }

    pub fn style(&self) -> Style {
        Style::with_current(self.color, self.ascii, self.config.current_color)
    }

    /// A prompt is only shown when both stdin and stderr are terminals
    /// (spec 4.6).
    pub fn interactive(&self) -> bool {
        self.stdin_tty && self.stderr_tty
    }
}

fn resolve_color(config: Toggle, flag: Option<Toggle>, stdout_tty: bool) -> bool {
    let choice = flag.unwrap_or(config);
    match choice {
        Toggle::Never => false,
        Toggle::Always => true,
        Toggle::Auto => {
            if !stdout_tty {
                return false;
            }
            if std::env::var_os("NO_COLOR").is_some() {
                return false;
            }
            if std::env::var("TERM").map(|t| t == "dumb").unwrap_or(false) {
                return false;
            }
            true
        }
    }
}

fn resolve_ascii(config: Toggle) -> bool {
    if std::env::var("TRK_ASCII")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    match config {
        Toggle::Always => true,
        Toggle::Never => false,
        Toggle::Auto => {
            if cfg!(windows) {
                return false;
            }
            let locale = ["LC_ALL", "LC_CTYPE", "LANG"]
                .iter()
                .filter_map(|k| std::env::var(k).ok())
                .find(|v| !v.is_empty());
            match locale {
                Some(v) => {
                    let upper = v.to_ascii_uppercase();
                    !(upper.contains("UTF-8") || upper.contains("UTF8"))
                }
                None => true,
            }
        }
    }
}

fn resolve_wrap(config_wrap: usize) -> usize {
    match crossterm::terminal::size() {
        Ok((cols, _)) if cols > 0 => config_wrap.min(cols as usize),
        _ => config_wrap,
    }
}
