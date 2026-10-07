use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{execute, terminal};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style as TuiStyle};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::cli::{Cli, Command};
use crate::ctx::Ctx;
use crate::error::TrkError;
use crate::model::Doc;
use crate::model::TaskId;
use crate::model::ops::{Request, Target};
use crate::model::tree;
use crate::tui::input::Input;
use clap::Parser;

enum Mode {
    Normal,
    Command,
    Note,
    Stop,
    Help,
}

struct App<'a> {
    ctx: &'a Ctx,
    doc: Doc,
    highlight: TaskId,
    focus: bool,
    why_full: bool,
    mode: Mode,
    input: Input,
    message: Option<(String, Instant)>,
    history: Vec<String>,
    history_idx: Option<usize>,
    last_meta: (u64, u64),
    quit: bool,
    rows: Vec<TaskId>,
}

pub fn run(ctx: &Ctx, focus: bool) -> Result<(), TrkError> {
    let doc = crate::commands::load_doc(ctx)?;
    let mut app = App::new(ctx, doc, focus);

    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut tui = Terminal::new(backend)?;

    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(std::io::stdout(), terminal::LeaveAlternateScreen);
        previous(info);
    }));

    let result = app.event_loop(&mut tui);

    let _ = terminal::disable_raw_mode();
    let _ = execute!(tui.backend_mut(), terminal::LeaveAlternateScreen);
    let _ = tui.show_cursor();
    result
}

impl<'a> App<'a> {
    fn new(ctx: &'a Ctx, doc: Doc, focus: bool) -> Self {
        let highlight = doc.active_goal().and_then(|g| g.cursor).unwrap_or(0);
        let mut app = Self {
            ctx,
            doc,
            highlight,
            focus,
            why_full: false,
            mode: Mode::Normal,
            input: Input::new(),
            message: None,
            history: Vec::new(),
            history_idx: None,
            last_meta: (0, 0),
            quit: false,
            rows: Vec::new(),
        };
        app.refresh_rows();
        if app.highlight == 0 {
            app.highlight = app.rows.first().copied().unwrap_or(0);
        }
        app
    }

    fn refresh_rows(&mut self) {
        let mut rows = Vec::new();
        if let Some(goal) = self.doc.active_goal() {
            for (task, _) in goal.walk() {
                if task.state.is_unfinished() {
                    rows.push(task.id);
                }
            }
        }
        self.rows = rows;
    }

    fn event_loop(
        &mut self,
        tui: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), TrkError> {
        loop {
            tui.draw(|frame| self.render(frame))?;
            if event::poll(Duration::from_millis(self.ctx.config.poll_ms))? {
                match event::read()? {
                    Event::Key(key) if key.kind != event::KeyEventKind::Release => {
                        self.handle_key(key, tui)?;
                    }
                    _ => {}
                }
            } else {
                self.reload_if_changed();
            }
            if self.quit {
                return Ok(());
            }
        }
    }

    fn store_meta(&self) -> (u64, u64) {
        let meta = std::fs::metadata(&self.ctx.config.store).ok();
        match meta {
            Some(m) => {
                let mtime = m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                (mtime, m.len())
            }
            None => (0, 0),
        }
    }

    fn reload_if_changed(&mut self) {
        let meta = self.store_meta();
        if meta == self.last_meta {
            return;
        }
        self.last_meta = meta;
        match crate::commands::load_doc(self.ctx) {
            Ok(doc) => {
                self.doc = doc;
                self.refresh_rows();
                if !self.rows.contains(&self.highlight) {
                    self.highlight = self
                        .doc
                        .active_goal()
                        .and_then(|g| g.cursor)
                        .or_else(|| self.rows.first().copied())
                        .unwrap_or(0);
                }
            }
            Err(_) => {
                self.set_message("store unreadable");
            }
        }
    }

    fn set_message(&mut self, text: impl Into<String>) {
        self.message = Some((text.into(), Instant::now()));
    }

    fn move_highlight(&mut self, delta: isize) {
        if self.rows.is_empty() {
            return;
        }
        let idx = self
            .rows
            .iter()
            .position(|id| *id == self.highlight)
            .unwrap_or(0);
        let new = (idx as isize + delta).clamp(0, self.rows.len() as isize - 1) as usize;
        self.highlight = self.rows[new];
    }

    fn handle_key(
        &mut self,
        key: KeyEvent,
        tui: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), TrkError> {
        match self.mode {
            Mode::Help => {
                self.mode = Mode::Normal;
                return Ok(());
            }
            Mode::Command | Mode::Note | Mode::Stop => {
                return self.handle_input_key(key);
            }
            Mode::Normal => {}
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('c')
                if key.code == KeyCode::Char('q')
                    || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_highlight(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_highlight(-1),
            KeyCode::PageDown => self.move_highlight(10),
            KeyCode::PageUp => self.move_highlight(-10),
            KeyCode::Char('c') => {
                if let Some(current) = self.doc.active_goal().and_then(|g| g.cursor) {
                    self.highlight = current;
                }
            }
            KeyCode::Char('d') => {
                let id = self.highlight;
                self.apply(
                    "done",
                    vec![
                        Request::Switch {
                            target: Target::Task(id),
                        },
                        Request::Done { force: false },
                    ],
                )?;
            }
            KeyCode::Char('g') => {
                self.apply("go", vec![Request::Go])?;
            }
            KeyCode::Char('n') => {
                self.mode = Mode::Note;
                self.input.clear();
            }
            KeyCode::Char('s') => {
                self.mode = Mode::Stop;
                self.input.clear();
            }
            KeyCode::Char('u') => {
                self.undo()?;
            }
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.input.clear();
            }
            KeyCode::Char('?') => self.mode = Mode::Help,
            KeyCode::Char('w') => self.why_full = !self.why_full,
            KeyCode::Tab => self.focus = !self.focus,
            KeyCode::Char('r') => {
                self.last_meta = (0, 0);
                self.reload_if_changed();
                tui.clear()?;
            }
            KeyCode::Enter => {
                let id = self.highlight;
                self.apply(
                    "switch",
                    vec![Request::Switch {
                        target: Target::Task(id),
                    }],
                )?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> Result<(), TrkError> {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                let value = self.input.value();
                let mode = std::mem::replace(&mut self.mode, Mode::Normal);
                match mode {
                    Mode::Command => {
                        self.history.push(value.clone());
                        self.history_idx = None;
                        self.run_command_line(&value)?;
                    }
                    Mode::Note => {
                        self.apply(
                            "note",
                            vec![Request::NoteAdd {
                                target: Target::Task(self.highlight),
                                text: value,
                            }],
                        )?;
                    }
                    Mode::Stop => {
                        self.apply(
                            "stop",
                            vec![Request::Stop {
                                target: Target::Task(self.highlight),
                                reason: value,
                            }],
                        )?;
                    }
                    _ => {}
                }
            }
            KeyCode::Up if matches!(self.mode, Mode::Command) => {
                if let Some(idx) = self.history_idx {
                    if idx > 0 {
                        self.history_idx = Some(idx - 1);
                        if let Some(cmd) = self.history.get(idx - 1) {
                            self.input.set(cmd);
                        }
                    }
                } else if !self.history.is_empty() {
                    self.history_idx = Some(self.history.len() - 1);
                    if let Some(cmd) = self.history.last() {
                        self.input.set(cmd);
                    }
                }
            }
            KeyCode::Down if matches!(self.mode, Mode::Command) => {
                if let Some(idx) = self.history_idx {
                    if idx + 1 < self.history.len() {
                        self.history_idx = Some(idx + 1);
                        if let Some(cmd) = self.history.get(idx + 1) {
                            self.input.set(cmd);
                        }
                    } else {
                        self.history_idx = None;
                        self.input.clear();
                    }
                }
            }
            _ => {
                self.input.handle(key);
            }
        }
        Ok(())
    }

    fn run_command_line(&mut self, line: &str) -> Result<(), TrkError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }
        let mut argv: Vec<String> = vec!["trk".to_string()];
        argv.extend(parts.iter().map(|s| s.to_string()));
        let parsed = match Cli::try_parse_from(&argv) {
            Ok(cli) => cli,
            Err(e) => {
                let msg = e
                    .to_string()
                    .lines()
                    .next()
                    .unwrap_or("bad command")
                    .to_string();
                self.set_message(msg);
                return Ok(());
            }
        };
        let Some(command) = parsed.command else {
            return Ok(());
        };
        match command_to_requests(command) {
            Ok(reqs) => {
                self.apply("tui", reqs)?;
            }
            Err(msg) => self.set_message(msg),
        }
        Ok(())
    }

    fn apply(&mut self, label: &str, requests: Vec<Request>) -> Result<(), TrkError> {
        let now = crate::commands::clock_now(self.ctx);
        let mut doc = self.doc.clone();
        let mut messages = Vec::new();
        for request in &requests {
            let (next, outcome) = crate::commands::apply_doc(&doc, request, now)?;
            doc = next;
            for message in outcome.messages {
                messages.push(message.text);
            }
        }
        crate::commands::save_doc(self.ctx, label, &doc)?;
        self.doc = doc;
        self.last_meta = self.store_meta();
        self.refresh_rows();
        if !self.rows.contains(&self.highlight) {
            self.highlight = self
                .doc
                .active_goal()
                .and_then(|g| g.cursor)
                .or_else(|| self.rows.first().copied())
                .unwrap_or(0);
        }
        if let Some(last) = messages.last() {
            self.set_message(last.clone());
        }
        Ok(())
    }

    fn undo(&mut self) -> Result<(), TrkError> {
        if let Some(doc) = crate::commands::undo_doc(self.ctx, 1)? {
            self.doc = doc;
            self.last_meta = self.store_meta();
            self.refresh_rows();
            self.set_message("undone");
        } else {
            self.set_message("nothing to undo");
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        let show_why = area.height >= 7 && !self.why_full;
        let mut constraints = vec![Constraint::Length(1), Constraint::Min(3)];
        if show_why {
            constraints.push(Constraint::Length(if self.why_full { 8 } else { 4 }));
        }
        if area.height >= 10 {
            constraints.push(Constraint::Length(1));
        }
        constraints.push(Constraint::Length(1));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let (title, open) = self
            .doc
            .active_goal()
            .map(|g| (g.title.clone(), g.count_open()))
            .unwrap_or_else(|| ("(no goal)".to_string(), 0));
        let title_line = Line::from(vec![
            Span::styled(
                format!(" {title} "),
                TuiStyle::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("  {open} open")),
        ]);
        frame.render_widget(Paragraph::new(title_line), chunks[0]);

        self.render_tasks(frame, chunks[1]);

        let mut next = 2;
        if show_why {
            self.render_why(frame, chunks[next]);
            next += 1;
        }
        if area.height >= 10 {
            let text = self
                .message
                .as_ref()
                .filter(|(_, at)| at.elapsed() < Duration::from_secs(5))
                .map(|(m, _)| m.clone())
                .unwrap_or_default();
            frame.render_widget(
                Paragraph::new(text).style(TuiStyle::default().fg(Color::Yellow)),
                chunks[next],
            );
            next += 1;
        }
        self.render_input_or_footer(frame, chunks[next]);
    }

    fn render_tasks(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let mut lines: Vec<Line> = Vec::new();
        let Some(goal) = self.doc.active_goal() else {
            lines.push(Line::from("no goal yet. :by <task>"));
            frame.render_widget(Paragraph::new(lines), area);
            return;
        };
        for (task, depth) in goal.walk() {
            if task.state.is_unfinished() {
                let marker = if self.highlight == task.id {
                    "> "
                } else {
                    "  "
                };
                let cursor = if goal.cursor == Some(task.id) {
                    " (current)"
                } else {
                    ""
                };
                let is_current = goal.cursor == Some(task.id);
                let mut style = if is_current {
                    TuiStyle::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    TuiStyle::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::DIM)
                };
                if self.highlight == task.id {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                lines.push(Line::styled(
                    format!("{marker}{}{}{cursor}", "  ".repeat(depth), task.text),
                    style,
                ));
            } else if task.has_open_descendant() {
                lines.push(Line::styled(
                    format!("  {}{}", "  ".repeat(depth), task.text),
                    TuiStyle::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::DIM),
                ));
            }
        }
        if lines.is_empty() {
            lines.push(Line::from("no tasks yet. :by <task>"));
        }
        frame.render_widget(
            Paragraph::new(lines).block(Block::default().borders(Borders::NONE)),
            area,
        );
    }

    fn render_why(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let Some(goal) = self.doc.active_goal() else {
            return;
        };
        let target = self.highlight;
        let is_current = goal.cursor == Some(target);
        let mut lines: Vec<Line> = Vec::new();
        if let Some(task) = goal.find(target) {
            for note in &task.note {
                lines.push(Line::from(note.clone()));
            }
        }
        if let Some(path) = tree::find_path(goal, target) {
            for len in (1..path.len()).rev() {
                if let Some(anc) = tree::task_at(goal, &path[..len])
                    && let Some(note) = anc.note.first()
                {
                    lines.push(Line::styled(
                        format!("^ {}: {note}", anc.text),
                        TuiStyle::default().fg(Color::DarkGray),
                    ));
                }
            }
        }
        let title = if is_current {
            " why \u{b7} current "
        } else {
            " why \u{b7} highlighted "
        };
        frame.render_widget(
            Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title(title))
                .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn render_input_or_footer(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        match self.mode {
            Mode::Command => {
                frame.render_widget(Paragraph::new(format!(":{}", self.input.buffer)), area);
            }
            Mode::Note => {
                frame.render_widget(Paragraph::new(format!("note: {}", self.input.buffer)), area);
            }
            Mode::Stop => {
                frame.render_widget(
                    Paragraph::new(format!("why stopping? {}", self.input.buffer)),
                    area,
                );
            }
            Mode::Help => {
                frame.render_widget(
                    Paragraph::new("j/k move  enter switch  d done  s stop  g go  n note  u undo  : cmd  tab focus  w why  r reload  q quit"),
                    area,
                );
            }
            Mode::Normal => {
                frame.render_widget(
                    Paragraph::new(
                        "j/k move  \u{23ce} switch  d done  s stop  n note  : cmd  ? help  q quit",
                    ),
                    area,
                );
            }
        }
    }
}

fn command_to_requests(command: Command) -> Result<Vec<Request>, String> {
    let join = |words: &[String]| words.join(" ");
    let req = match command {
        Command::By(a) => Request::By {
            text: join(&a.text),
            why: a.why,
        },
        Command::Need(a) => Request::Need {
            text: join(&a.text),
            why: a.why,
            target: Target::Current,
            stay: a.stay,
        },
        Command::Then(a) => Request::Then {
            text: join(&a.text),
            why: a.why,
            target: Target::Current,
        },
        Command::And(a) => Request::And {
            text: join(&a.text),
            why: a.why,
            target: Target::Current,
        },
        Command::Done(a) => Request::Done { force: a.force },
        Command::Drop(a) => Request::Drop {
            force: a.force,
            why: a.why,
        },
        Command::Stop(a) => Request::Stop {
            target: Target::Current,
            reason: join(&a.reason),
        },
        Command::Go => Request::Go,
        Command::Note(a) => Request::NoteAdd {
            target: Target::Current,
            text: join(&a.text),
        },
        Command::Rename(a) => Request::Rename {
            target: Target::Current,
            text: join(&a.text),
        },
        Command::Goal(g) => match g.command {
            Some(crate::cli::GoalCommand::New(a)) => Request::GoalNew {
                title: join(&a.title),
                stay: a.stay,
            },
            Some(crate::cli::GoalCommand::Done(a)) => Request::GoalDone { force: a.force },
            Some(crate::cli::GoalCommand::Rename(a)) => Request::GoalRename {
                title: join(&a.title),
            },
            _ => return Err("that command is not available in the TUI".into()),
        },
        Command::Jot(a) => Request::Jot {
            text: join(&a.text),
        },
        _ => return Err("that command is not available in the TUI".into()),
    };
    Ok(vec![req])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::store::format::parse;
    use ratatui::backend::TestBackend;

    fn make_ctx() -> Ctx {
        let config = Config::with_store("/tmp/trk-tui-test.trk".into()).unwrap();
        Ctx::new(config, None, None)
    }

    #[test]
    fn renders_at_small_and_large_sizes() {
        let doc = parse(include_str!("../../tests/fixtures/step7.trk")).unwrap();
        for (width, height) in [(20u16, 6u16), (40, 12), (80, 24)] {
            let ctx = make_ctx();
            let mut app = App::new(&ctx, doc.clone(), false);
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| app.render(frame)).unwrap();
            let buffer = terminal.backend().buffer();
            let text: String = buffer.content().iter().map(|c| c.symbol()).collect();
            assert!(
                text.contains("Ship login fix"),
                "size {width}x{height} rendered: {text:?}"
            );
        }
    }
}
