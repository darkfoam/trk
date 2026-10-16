//! View rendering tests: exact snapshots and wrapping.

use chrono::{DateTime, FixedOffset};
use trk::store::format::parse;
use trk::ui::style::Style;
use trk::ui::views;
use trk::ui::wrap::width_of;

const BASIC: &str = include_str!("../tests/fixtures/basic.trk");

fn dt(s: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(s).unwrap()
}

fn render(lines: Vec<trk::ui::style::StyledLine>) -> String {
    lines
        .iter()
        .map(|l| l.text())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn context_shows_parents_siblings_and_pointer() {
    let doc = parse(BASIC).unwrap();
    let got = render(views::context_view(&doc, 80, Style::new(false, true)));
    let expected = "\
Ship login fix
  Fix login bug
    Reproduce on staging
      > Get staging creds
        why: can't log in";
    assert_eq!(got, expected);
}

#[test]
fn progress_strikes_done_and_shows_time() {
    let doc = parse(BASIC).unwrap();
    let got = render(views::progress_view(
        &doc,
        dt("2026-10-07T12:00:00-05:00"),
        80,
        Style::new(false, true),
    ));
    let expected = "\
Ship login fix   (3 open, 1 done)
  Fix login bug
    Reproduce on staging
    > Get staging creds
      why: can't log in
  Call internet company  [10m]";
    assert_eq!(got, expected);
}

#[test]
fn goal_list_numbers_and_marks_active() {
    let doc = parse(BASIC).unwrap();
    let got = render(views::goal_list_view(&doc, Style::new(false, true)));
    assert!(got.contains("1  Ship login fix"), "{got}");
    assert!(got.contains("(3 open)"), "{got}");
    assert!(got.starts_with('>'), "active goal marked: {got}");
}

#[test]
fn log_shows_finished_task_with_time() {
    let doc = parse(BASIC).unwrap();
    let today = chrono::NaiveDate::from_ymd_opt(2026, 10, 7).unwrap();
    let got = render(views::log_view(
        &doc,
        &[today],
        dt("2026-10-07T12:00:00-05:00"),
        "MM/DD/YYYY",
        80,
        Style::new(false, true),
    ));
    assert!(got.contains("Call internet company"), "{got}");
    assert!(got.contains("11:00"), "{got}");
    assert!(got.contains("10m"), "{got}");
    assert!(got.contains("Wed 10/07/2026"), "{got}");
}

#[test]
fn unicode_symbols_used_by_default() {
    let doc = parse(BASIC).unwrap();
    let got = render(views::context_view(&doc, 80, Style::new(false, false)));
    assert!(got.contains('\u{25ba}'), "cursor symbol: {got}");
    assert!(!got.contains('>'), "no ascii cursor");
}

#[test]
fn agenda_shows_only_this_week_grouped_by_date() {
    let text = "\
trk 1
next g=1 t=4 i=1
active g1

goal g1 open 2026-10-06T09:00:00-05:00 G
  [ ] t1 2026-10-07T09:00:00-05:00 due today
    @ at=2026-10-07
  [ ] t2 2026-10-07T09:00:00-05:00 due later this week
    @ at=2026-10-09
  [ ] t3 2026-10-07T09:00:00-05:00 due next week
    @ at=2026-10-20
";
    let doc = parse(text).unwrap();
    let today = chrono::NaiveDate::from_ymd_opt(2026, 10, 7).unwrap();
    let got = render(views::agenda_view(
        &doc,
        today,
        chrono::Weekday::Mon,
        "MM/DD/YYYY",
        80,
        Style::new(false, true),
    ));
    assert!(got.contains("Today  10/07/2026"), "{got}");
    assert!(got.contains("due today"), "{got}");
    assert!(got.contains("due later this week"), "{got}");
    assert!(got.contains("10/09/2026"), "{got}");
    assert!(!got.contains("due next week"), "outside the week: {got}");
    assert!(!got.contains("10/20/2026"), "outside the week: {got}");
}

#[test]
fn agenda_marks_today_in_the_agenda_color() {
    let text = "\
trk 1
next g=1 t=2 i=1
active g1

goal g1 open 2026-10-06T09:00:00-05:00 G
  [ ] t1 2026-10-07T09:00:00-05:00 due today
    @ at=2026-10-07
";
    let doc = parse(text).unwrap();
    let today = chrono::NaiveDate::from_ymd_opt(2026, 10, 7).unwrap();
    let lines = views::agenda_view(
        &doc,
        today,
        chrono::Weekday::Mon,
        "MM/DD/YYYY",
        80,
        Style::new(true, true),
    );
    let rendered = trk::ui::style::render_lines(&lines, Style::new(true, true));
    assert!(
        rendered.contains("\u{1b}[33m"),
        "today should be yellow: {rendered:?}"
    );
}

#[test]
fn long_text_wraps_within_width() {
    let long = "x".repeat(300);
    let text = format!(
        "trk 1\nnext g=1 t=2 i=1\n\ngoal g1 open 2026-10-07T09:00:00-05:00 A very long goal title that keeps going\n  current t1\n  [ ] t1 2026-10-07T09:01:00-05:00 {long}\n"
    );
    let doc = parse(&text).unwrap();
    for width in [40usize, 60, 80] {
        for line in views::context_view(&doc, width, Style::new(false, true)) {
            assert!(
                width_of(&line.text()) <= width,
                "line wider than {width}: {:?}",
                line.text()
            );
        }
        for line in views::progress_view(
            &doc,
            dt("2026-10-07T12:00:00-05:00"),
            width,
            Style::new(false, true),
        ) {
            assert!(
                width_of(&line.text()) <= width,
                "progress line wider than {width}: {:?}",
                line.text()
            );
        }
    }
}
