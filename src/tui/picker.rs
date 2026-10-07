use std::io::Write;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{cursor, execute, terminal};

use crate::error::TrkError;
use crate::interact::{PickResult, PickRow, PickSpec};
use crate::ui::style::Style;
use crate::ui::wrap;

/// Pure picker navigation state, split out so it can be unit tested without a
/// terminal (spec 9.2).
#[derive(Clone, Debug)]
pub struct PickerState {
    pub highlight: usize,
    pub typed: String,
}

impl PickerState {
    pub fn new(len: usize, initial: usize) -> Self {
        Self {
            highlight: initial.min(len.saturating_sub(1)),
            typed: String::new(),
        }
    }

    pub fn move_by(&mut self, delta: isize, len: usize) {
        if len == 0 {
            return;
        }
        self.highlight = (self.highlight as isize + delta).clamp(0, len as isize - 1) as usize;
    }

    pub fn home(&mut self) {
        self.highlight = 0;
    }

    pub fn end(&mut self, len: usize) {
        self.highlight = len.saturating_sub(1);
    }

    pub fn push_digit(&mut self, c: char, rows: &[PickRow]) {
        self.typed.push(c);
        if let Some(idx) = resolve(rows, &self.typed) {
            self.highlight = idx;
        }
    }

    pub fn backspace(&mut self) {
        self.typed.pop();
    }

    pub fn resolve(&self, rows: &[PickRow]) -> Option<usize> {
        resolve(rows, &self.typed)
    }
}

fn resolve(rows: &[PickRow], typed: &str) -> Option<usize> {
    if typed.is_empty() {
        return None;
    }
    let n: usize = typed.parse().ok()?;
    rows.iter().position(|r| r.number == Some(n))
}

/// Run the interactive picker on the alternate screen. Restores the terminal
/// on every exit path (spec 9.3).
pub fn run(spec: &PickSpec, style: Style) -> Result<PickResult, TrkError> {
    let mut out = std::io::stdout();
    terminal::enable_raw_mode()?;
    execute!(out, terminal::EnterAlternateScreen)?;
    let result = run_inner(spec, style, &mut out);
    let _ = execute!(out, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
    result
}

fn run_inner(
    spec: &PickSpec,
    style: Style,
    out: &mut std::io::Stdout,
) -> Result<PickResult, TrkError> {
    let mut state = PickerState::new(spec.rows.len(), spec.initial);
    loop {
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        if cols < 20 || rows < 6 {
            return Err(TrkError::Message("terminal too small".into()));
        }
        draw(spec, style, out, cols as usize, rows as usize, &state)?;

        let event = event::read()?;
        let Event::Key(key) = event else { continue };
        if key.kind == event::KeyEventKind::Release {
            continue;
        }
        let len = spec.rows.len();
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(PickResult::Cancel);
            }
            KeyCode::Esc | KeyCode::Char('q') => return Ok(PickResult::Cancel),
            KeyCode::Up | KeyCode::Char('k') => state.move_by(-1, len),
            KeyCode::Down | KeyCode::Char('j') => state.move_by(1, len),
            KeyCode::PageUp => state.move_by(-(rows as isize - 3), len),
            KeyCode::PageDown => state.move_by(rows as isize - 3, len),
            KeyCode::Home | KeyCode::Char('g') => state.home(),
            KeyCode::End | KeyCode::Char('G') => state.end(len),
            KeyCode::Char(c) if c.is_ascii_digit() => state.push_digit(c, &spec.rows),
            KeyCode::Backspace => state.backspace(),
            KeyCode::Enter => {
                if let Some(idx) = state.resolve(&spec.rows) {
                    state.highlight = idx;
                }
                return Ok(PickResult::Chosen(spec.rows[state.highlight].id));
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw(
    spec: &PickSpec,
    style: Style,
    out: &mut std::io::Stdout,
    cols: usize,
    rows: usize,
    state: &PickerState,
) -> Result<(), TrkError> {
    let highlight = state.highlight;
    let view_height = rows.saturating_sub(3).max(1);
    let start = if highlight < view_height {
        0
    } else {
        highlight + 1 - view_height
    };
    let end = (start + view_height).min(spec.rows.len());

    execute!(
        out,
        cursor::MoveTo(0, 0),
        terminal::Clear(terminal::ClearType::All)
    )?;
    let mut buf = String::new();
    buf.push_str(&wrap::truncate(&spec.header, cols, style.ellipsis()));
    buf.push_str("\r\n");
    for (i, row) in spec.rows[start..end].iter().enumerate() {
        let idx = start + i;
        let label = wrap::truncate(&row.label, cols.saturating_sub(1), style.ellipsis());
        if idx == highlight {
            buf.push_str("\x1b[7m");
            buf.push_str(&label);
            buf.push_str("\x1b[0m");
        } else {
            buf.push_str(&label);
        }
        buf.push_str("\r\n");
    }
    while buf.matches("\r\n").count() < rows.saturating_sub(1) {
        buf.push_str("\r\n");
    }
    let footer = format!(
        "j/k move  \u{23ce} pick  esc cancel{}",
        if state.typed.is_empty() {
            String::new()
        } else {
            format!("  go to: {}", state.typed)
        }
    );
    buf.push_str(&wrap::truncate(&footer, cols, style.ellipsis()));
    out.write_all(buf.as_bytes())?;
    out.flush()?;
    Ok(())
}
