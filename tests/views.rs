//! View rendering tests: exact snapshots and wrapping (spec 12.4).

use trk::store::format::parse;
use trk::ui::style::Style;
use trk::ui::views;
use trk::ui::wrap::width_of;

const GOLDEN: &str = include_str!("../tests/fixtures/step7.trk");

fn render(lines: Vec<trk::ui::style::StyledLine>) -> String {
    lines
        .iter()
        .map(|l| l.text())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn status_matches_spec_7_1() {
    let doc = parse(GOLDEN).unwrap();
    let got = render(views::status_view(&doc, 80, Style::new(false, true)));
    let expected = "\
Ship login fix
  Fix login bug
    Reproduce on staging
      Document the process
        > Get staging creds
          why: can't log in";
    assert_eq!(got, expected);
}

#[test]
fn list_matches_spec_7_2() {
    let doc = parse(GOLDEN).unwrap();
    let got = render(views::list_view(
        &doc,
        80,
        Style::new(false, true),
        false,
        false,
    ));
    let expected = "\
Ship login fix
 1  Fix login bug
 2    Reproduce on staging
 3      Document the process
 4        > Get staging creds
 5    Check auth logs
 6  Call internet company";
    assert_eq!(got, expected);
}

#[test]
fn why_matches_spec_7_3() {
    let doc = parse(GOLDEN).unwrap();
    let got = render(views::why_view(&doc, 80, Style::new(false, true)));
    let expected = "\
Ship login fix
  Fix login bug
    why: customers locked out
    Reproduce on staging
      why: need a failing case
      Document the process
        why: after the fix ships
        > Get staging creds
          why: can't log in";
    assert_eq!(got, expected);
}

#[test]
fn unicode_symbols_used_by_default() {
    let doc = parse(GOLDEN).unwrap();
    let got = render(views::status_view(&doc, 80, Style::new(false, false)));
    assert!(got.contains('\u{25ba}'), "cursor symbol: {got}");
    assert!(!got.contains('>'), "no ascii cursor");
}

#[test]
fn long_text_wraps_within_width() {
    let long = "x".repeat(300);
    let text = format!(
        "trk 1\nnext g=1 t=2 i=1\n\ngoal g1 open 2026-10-07T09:00:00-05:00 A very long goal title that keeps going\n  [ ] t1 2026-10-07T09:01:00-05:00 {long}\n"
    );
    let doc = parse(&text).unwrap();
    for width in [40usize, 60, 80] {
        for line in views::status_view(&doc, width, Style::new(false, true)) {
            assert!(
                width_of(&line.text()) <= width,
                "line wider than {width}: {:?}",
                line.text()
            );
        }
        for line in views::list_view(&doc, width, Style::new(false, true), false, false) {
            assert!(
                width_of(&line.text()) <= width,
                "list line wider than {width}: {:?}",
                line.text()
            );
        }
    }
}
