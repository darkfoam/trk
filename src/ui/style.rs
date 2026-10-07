use anstyle::{Ansi256Color, AnsiColor, Style as AnsiStyle};

/// Semantic role for a span of output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Normal,
    Dim,
    Bold,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub role: Role,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct StyledLine {
    pub spans: Vec<Span>,
}

impl StyledLine {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            spans: vec![Span {
                text: text.into(),
                role: Role::Normal,
            }],
        }
    }

    pub fn styled(text: impl Into<String>, role: Role) -> Self {
        Self {
            spans: vec![Span {
                text: text.into(),
                role,
            }],
        }
    }

    pub fn push(&mut self, text: impl Into<String>, role: Role) {
        self.spans.push(Span {
            text: text.into(),
            role,
        });
    }

    /// Plain text with all styling removed (used by snapshots and tests).
    pub fn text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.iter().all(|s| s.text.is_empty())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color: bool,
    pub ascii: bool,
}

impl Style {
    pub fn new(color: bool, ascii: bool) -> Self {
        Self { color, ascii }
    }

    pub fn cursor(&self) -> &'static str {
        if self.ascii { ">" } else { "\u{25ba}" }
    }

    pub fn stopped(&self) -> &'static str {
        if self.ascii { "!" } else { "\u{23f8}" }
    }

    pub fn up(&self) -> &'static str {
        if self.ascii { "^" } else { "\u{2191}" }
    }

    pub fn ellipsis(&self) -> &'static str {
        if self.ascii { "..." } else { "\u{2026}" }
    }
}

fn ansi_for(role: Role) -> AnsiStyle {
    match role {
        Role::Normal => AnsiStyle::new(),
        Role::Dim => AnsiStyle::new().dimmed(),
        Role::Bold => AnsiStyle::new().bold(),
        Role::Warning => AnsiStyle::new().fg_color(Some(Ansi256Color(208).into())),
        Role::Error => AnsiStyle::new().fg_color(Some(AnsiColor::Red.into())),
    }
}

/// Render a line to a string. When `color` is false no escape codes are
/// emitted.
pub fn render_line(line: &StyledLine, color: bool) -> String {
    if !color {
        return line.text();
    }
    let mut out = String::new();
    for span in &line.spans {
        let style = ansi_for(span.role);
        out.push_str(&style.render().to_string());
        out.push_str(&span.text);
        out.push_str(&style.render_reset().to_string());
    }
    out
}

pub fn render_lines(lines: &[StyledLine], color: bool) -> String {
    let mut out = String::new();
    for line in lines {
        out.push_str(&render_line(line, color));
        out.push('\n');
    }
    out
}
