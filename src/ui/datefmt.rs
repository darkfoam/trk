use chrono::{Datelike, NaiveDate, Weekday};

/// Format a date with a small, moment/Obsidian-style token set:
/// `YYYY`, `YY`, `MM`, `M`, `DD`, `D`, `ddd` (Mon), and `dddd` (Monday).
/// Anything that is not a token is copied verbatim.
pub fn format_date(date: NaiveDate, fmt: &str) -> String {
    const TOKENS: [&str; 8] = ["YYYY", "dddd", "MM", "DD", "YY", "ddd", "M", "D"];
    let mut out = String::new();
    let mut rest = fmt;
    'outer: while !rest.is_empty() {
        for token in TOKENS {
            if rest.starts_with(token) {
                out.push_str(&expand(date, token));
                rest = &rest[token.len()..];
                continue 'outer;
            }
        }
        let ch = rest.chars().next().expect("non-empty");
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

fn expand(date: NaiveDate, token: &str) -> String {
    let year = date.year();
    let month = date.month();
    let day = date.day();
    match token {
        "YYYY" => format!("{year:04}"),
        "YY" => format!("{:02}", year.rem_euclid(100)),
        "MM" => format!("{month:02}"),
        "M" => month.to_string(),
        "DD" => format!("{day:02}"),
        "D" => day.to_string(),
        "ddd" => weekday_abbrev(date.weekday()).to_string(),
        "dddd" => weekday_full(date.weekday()).to_string(),
        _ => token.to_string(),
    }
}

/// Parse a weekday from its first three letters (case-insensitive): `Mon`,
/// `sun`, `TUE`, ...
pub fn parse_weekday(text: &str) -> Option<Weekday> {
    let lower = text.trim().to_ascii_lowercase();
    Some(match lower.get(..3)? {
        "mon" => Weekday::Mon,
        "tue" => Weekday::Tue,
        "wed" => Weekday::Wed,
        "thu" => Weekday::Thu,
        "fri" => Weekday::Fri,
        "sat" => Weekday::Sat,
        "sun" => Weekday::Sun,
        _ => return None,
    })
}

pub fn weekday_abbrev(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}

pub fn weekday_full(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn default_is_us_style() {
        assert_eq!(format_date(date(2026, 10, 7), "MM/DD/YYYY"), "10/07/2026");
    }

    #[test]
    fn supports_tokens_and_literals() {
        assert_eq!(format_date(date(2026, 1, 9), "D/M/YY"), "9/1/26");
        assert_eq!(
            format_date(date(2026, 10, 7), "ddd DD.MM.YYYY"),
            "Wed 07.10.2026"
        );
        assert_eq!(
            format_date(date(2026, 10, 7), "dddd, MM/DD/YYYY"),
            "Wednesday, 10/07/2026"
        );
    }

    #[test]
    fn parses_weekday_prefixes() {
        assert_eq!(parse_weekday("Mon"), Some(Weekday::Mon));
        assert_eq!(parse_weekday("SUN"), Some(Weekday::Sun));
        assert_eq!(parse_weekday(" tuesday "), Some(Weekday::Tue));
        assert_eq!(parse_weekday("nope"), None);
    }
}
