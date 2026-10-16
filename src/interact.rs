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
    /// Support TAB to create a new item (returns `PickResult::Create`).
    pub allow_new: bool,
    /// Support `a` to assign a date to the highlighted item (returns
    /// `PickResult::AssignDate`).
    pub assign_date: bool,
}

impl PickSpec {
    pub fn new(header: impl Into<String>, rows: Vec<PickRow>, initial: usize) -> Self {
        Self {
            header: header.into(),
            rows,
            initial,
            allow_new: false,
            assign_date: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickResult {
    Chosen(u64),
    Create,
    AssignDate(u64),
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
        if spec.rows.is_empty() && !spec.allow_new {
            return Err(TrkError::Message("nothing to choose from".into()));
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
    fn ask(&mut self, prompt: &str) -> Result<Option<String>, TrkError> {
        // Non-interactive, but still scriptable: read a line from stdin so
        // flows like `trk view`'s date prompt and `trk inbox`'s new-goal title
        // can be driven from a pipeline.
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
        if spec.rows.is_empty() && !spec.allow_new {
            return Err(TrkError::Message("nothing to choose from".into()));
        }
        let mut err = std::io::stderr();
        writeln!(err, "{}", spec.header)?;
        for row in &spec.rows {
            writeln!(err, "{}", row.label)?;
        }
        if spec.allow_new {
            writeln!(err, "n: new goal")?;
        }
        if spec.assign_date {
            writeln!(err, "a <n>: assign a date")?;
        }
        write!(err, "pick: ")?;
        err.flush()?;
        let mut line = String::new();
        let n = std::io::stdin().lock().read_line(&mut line)?;
        if n == 0 {
            return Ok(PickResult::Cancel);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if spec.rows.is_empty() {
                return Ok(PickResult::Create);
            }
            let idx = spec.initial.min(spec.rows.len() - 1);
            return Ok(PickResult::Chosen(spec.rows[idx].id));
        }
        if spec.allow_new && trimmed.eq_ignore_ascii_case("n") {
            return Ok(PickResult::Create);
        }
        if spec.assign_date
            && let Some(rest) = trimmed.strip_prefix("a ")
        {
            let number: usize = rest
                .trim()
                .parse()
                .map_err(|_| TrkError::Usage("pick: assign wants `a <number>`".into()))?;
            return spec
                .rows
                .iter()
                .find(|r| r.number == Some(number))
                .map(|r| PickResult::AssignDate(r.id))
                .ok_or_else(|| TrkError::Usage(format!("pick: no item number {number}")));
        }
        let number: usize = trimmed
            .parse()
            .map_err(|_| TrkError::Usage("pick: enter a number".into()))?;
        spec.rows
            .iter()
            .find(|r| r.number == Some(number))
            .map(|r| PickResult::Chosen(r.id))
            .ok_or_else(|| TrkError::Usage(format!("pick: no item number {number}")))
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
