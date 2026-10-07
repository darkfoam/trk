use std::io::{BufRead, Write};

use crate::error::TrkError;
use crate::ui::style::Style;

#[derive(Clone, Debug)]
pub struct PickRow {
    pub id: u64,
    pub label: String,
    pub number: Option<usize>,
    pub current: bool,
}

#[derive(Clone, Debug)]
pub struct PickSpec {
    pub header: String,
    pub rows: Vec<PickRow>,
    pub initial: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickResult {
    Chosen(u64),
    Cancel,
}

pub trait Ui {
    fn ask(&mut self, prompt: &str) -> Result<Option<String>, TrkError>;
    fn pick(&mut self, spec: &PickSpec) -> Result<PickResult, TrkError>;
    fn is_interactive(&self) -> bool;
    fn warn(&mut self, message: &str);
    fn error(&mut self, message: &str);
}

/// Terminal-backed UI: line prompts on stderr, picker on the alternate screen.
pub struct TerminalUi {
    pub style: Style,
}

impl TerminalUi {
    pub fn new(style: Style) -> Self {
        Self { style }
    }
}

impl Ui for TerminalUi {
    fn ask(&mut self, prompt: &str) -> Result<Option<String>, TrkError> {
        let mut stderr = std::io::stderr();
        write!(stderr, "{prompt}")?;
        stderr.flush()?;
        let mut line = String::new();
        let n = std::io::stdin().lock().read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed.to_string()))
        }
    }

    fn pick(&mut self, spec: &PickSpec) -> Result<PickResult, TrkError> {
        if spec.rows.is_empty() {
            return Err(TrkError::Message("nothing to switch to".into()));
        }
        crate::tui::picker::run(spec, self.style)
    }

    fn is_interactive(&self) -> bool {
        true
    }

    fn warn(&mut self, message: &str) {
        let line = crate::ui::style::StyledLine::styled(
            format!("warning: {message}"),
            crate::ui::style::Role::Warning,
        );
        eprintln!("{}", crate::ui::style::render_line(&line, self.style));
    }

    fn error(&mut self, message: &str) {
        eprintln!("trk: {message}");
    }
}

/// Non-interactive UI: no prompts, pickers cancel (spec 4.6, 9.3).
pub struct NullUi;

impl Ui for NullUi {
    fn ask(&mut self, _prompt: &str) -> Result<Option<String>, TrkError> {
        Ok(None)
    }

    fn pick(&mut self, _spec: &PickSpec) -> Result<PickResult, TrkError> {
        Err(TrkError::Message("a picker needs a terminal".into()))
    }

    fn is_interactive(&self) -> bool {
        false
    }

    fn warn(&mut self, message: &str) {
        eprintln!("warning: {message}");
    }

    fn error(&mut self, message: &str) {
        eprintln!("trk: {message}");
    }
}
