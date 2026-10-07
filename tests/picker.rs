//! Picker navigation state tests (spec 9.2, A14).

use trk::interact::PickRow;
use trk::tui::picker::PickerState;

fn rows() -> Vec<PickRow> {
    (1..=5)
        .map(|n| PickRow {
            id: n as u64,
            label: format!("row {n}"),
            number: Some(n as usize),
            current: n == 1,
        })
        .collect()
}

#[test]
fn movement_clamps_at_edges() {
    let mut state = PickerState::new(5, 0);
    state.move_by(-1, 5);
    assert_eq!(state.highlight, 0);
    state.move_by(99, 5);
    assert_eq!(state.highlight, 4);
    state.move_by(1, 5);
    assert_eq!(state.highlight, 4);
}

#[test]
fn digits_jump_to_number() {
    let mut state = PickerState::new(5, 0);
    state.push_digit('3', &rows());
    assert_eq!(state.highlight, 2);
    state.push_digit('1', &rows());
    assert_eq!(state.highlight, 2, "31 is not a row number");
    state.backspace();
    state.backspace();
    state.push_digit('5', &rows());
    assert_eq!(state.highlight, 4);
}

#[test]
fn home_and_end() {
    let mut state = PickerState::new(5, 2);
    state.home();
    assert_eq!(state.highlight, 0);
    state.end(5);
    assert_eq!(state.highlight, 4);
}

#[test]
fn enter_resolves_typed_number() {
    let mut state = PickerState::new(5, 0);
    state.push_digit('4', &rows());
    assert_eq!(state.resolve(&rows()), Some(3));
    assert_eq!(rows()[state.resolve(&rows()).unwrap()].id, 4);
}
