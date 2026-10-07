use anstyle::{Ansi256Color, AnsiColor, Style as AnsiStyle};

/// Semantic role for a span of output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Normal,
    Dim,
    Bold,
    /// The goal title: bold white.
    Title,
    /// The current task: bold, in the configured color.
    Current,
    /// Any other task: faded green, so the current task pops.
    Task,
    Warning,
    Error,
}

/// A named ANSI color, configurable via `current_color` in the config file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColorName {
    Black,
    Red,
    #[default]
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl ColorName {
    pub const NAMES: &[&str] = &[
        "black",
        "red",
        "green",
        "yellow",
        "blue",
        "magenta",
        "cyan",
        "white",
        "bright-black",
        "bright-red",
        "bright-green",
        "bright-yellow",
        "bright-blue",
        "bright-magenta",
        "bright-cyan",
        "bright-white",
    ];

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value.to_ascii_lowercase().as_str() {
            "black" => ColorName::Black,
            "red" => ColorName::Red,
            "green" => ColorName::Green,
            "yellow" => ColorName::Yellow,
            "blue" => ColorName::Blue,
            "magenta" => ColorName::Magenta,
            "cyan" => ColorName::Cyan,
            "white" => ColorName::White,
            "bright-black" => ColorName::BrightBlack,
            "bright-red" => ColorName::BrightRed,
            "bright-green" => ColorName::BrightGreen,
            "bright-yellow" => ColorName::BrightYellow,
            "bright-blue" => ColorName::BrightBlue,
            "bright-magenta" => ColorName::BrightMagenta,
            "bright-cyan" => ColorName::BrightCyan,
            "bright-white" => ColorName::BrightWhite,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ColorName::Black => "black",
            ColorName::Red => "red",
            ColorName::Green => "green",
            ColorName::Yellow => "yellow",
            ColorName::Blue => "blue",
            ColorName::Magenta => "magenta",
            ColorName::Cyan => "cyan",
            ColorName::White => "white",
            ColorName::BrightBlack => "bright-black",
            ColorName::BrightRed => "bright-red",
            ColorName::BrightGreen => "bright-green",
            ColorName::BrightYellow => "bright-yellow",
            ColorName::BrightBlue => "bright-blue",
            ColorName::BrightMagenta => "bright-magenta",
            ColorName::BrightCyan => "bright-cyan",
            ColorName::BrightWhite => "bright-white",
        }
    }

    fn to_ansi(self) -> AnsiColor {
        match self {
            ColorName::Black => AnsiColor::Black,
            ColorName::Red => AnsiColor::Red,
            ColorName::Green => AnsiColor::Green,
            ColorName::Yellow => AnsiColor::Yellow,
            ColorName::Blue => AnsiColor::Blue,
            ColorName::Magenta => AnsiColor::Magenta,
            ColorName::Cyan => AnsiColor::Cyan,
            ColorName::White => AnsiColor::White,
            ColorName::BrightBlack => AnsiColor::BrightBlack,
            ColorName::BrightRed => AnsiColor::BrightRed,
            ColorName::BrightGreen => AnsiColor::BrightGreen,
            ColorName::BrightYellow => AnsiColor::BrightYellow,
            ColorName::BrightBlue => AnsiColor::BrightBlue,
            ColorName::BrightMagenta => AnsiColor::BrightMagenta,
            ColorName::BrightCyan => AnsiColor::BrightCyan,
            ColorName::BrightWhite => AnsiColor::BrightWhite,
        }
    }
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
    pub current: ColorName,
}

impl Style {
    /// Default current-task color is green.
    pub fn new(color: bool, ascii: bool) -> Self {
        Self {
            color,
            ascii,
            current: ColorName::Green,
        }
    }

    pub fn with_current(color: bool, ascii: bool, current: ColorName) -> Self {
        Self {
            color,
            ascii,
            current,
        }
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

fn ansi_for(role: Role, style: Style) -> AnsiStyle {
    match role {
        Role::Normal => AnsiStyle::new(),
        Role::Dim => AnsiStyle::new().dimmed(),
        Role::Bold => AnsiStyle::new().bold(),
        Role::Title => AnsiStyle::new()
            .bold()
            .fg_color(Some(AnsiColor::White.into())),
        Role::Current => AnsiStyle::new()
            .bold()
            .fg_color(Some(style.current.to_ansi().into())),
        Role::Task => AnsiStyle::new()
            .dimmed()
            .fg_color(Some(AnsiColor::Green.into())),
        Role::Warning => AnsiStyle::new().fg_color(Some(Ansi256Color(208).into())),
        Role::Error => AnsiStyle::new().fg_color(Some(AnsiColor::Red.into())),
    }
}

/// Render a line to a string. When color is disabled no escape codes are
/// emitted.
pub fn render_line(line: &StyledLine, style: Style) -> String {
    if !style.color {
        return line.text();
    }
    let mut out = String::new();
    for span in &line.spans {
        let ansi = ansi_for(span.role, style);
        out.push_str(&ansi.render().to_string());
        out.push_str(&span.text);
        out.push_str(&ansi.render_reset().to_string());
    }
    out
}

pub fn render_lines(lines: &[StyledLine], style: Style) -> String {
    let mut out = String::new();
    for line in lines {
        out.push_str(&render_line(line, style));
        out.push('\n');
    }
    out
}
