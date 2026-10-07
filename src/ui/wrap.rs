use textwrap::{Options, wrap};
use unicode_width::UnicodeWidthStr;

/// Display width of a string, ignoring ANSI escapes (callers pass plain text).
pub fn width_of(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// Wrap `text` to `width` columns using hanging indents. Returns at least one
/// line (possibly empty).
pub fn wrap_text(text: &str, width: usize, first_indent: &str, rest_indent: &str) -> Vec<String> {
    let width = width.max(1);
    let options = Options::new(width)
        .initial_indent(first_indent)
        .subsequent_indent(rest_indent);
    let lines: Vec<String> = wrap(text, options)
        .into_iter()
        .map(|c| c.into_owned())
        .collect();
    if lines.is_empty() {
        vec![first_indent.to_string()]
    } else {
        lines
    }
}

/// Truncate `text` to `width` columns, appending `ellipsis` when cut.
pub fn truncate(text: &str, width: usize, ellipsis: &str) -> String {
    if UnicodeWidthStr::width(text) <= width {
        return text.to_string();
    }
    let ell_width = UnicodeWidthStr::width(ellipsis);
    if width <= ell_width {
        return ellipsis.chars().take(width).collect();
    }
    let target = width - ell_width;
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > target {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push_str(ellipsis);
    out
}
