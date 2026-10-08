# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-10-08

### Added

- Pickers now work without a terminal: when one cannot be drawn, the numbered
  choices are printed to standard error and the number is read from standard
  input (an empty line accepts the highlighted default), so `trk then` default
  picking (and `-s`/`--switch` elsewhere) can be driven from a script or
  pipeline.
- The `trk then` picker starts on the most recently created task, so repeated
  `then` chains: wrap a task, then wrap that new task, and so on.

### Changed

- `trk then X` now picks the target by default instead of silently using the
  current task. Pass `-c`/`--current` to target the current task. The old
  `-s`/`--switch` flag is gone, since picking is now the default.

### Fixed

- `trk then X` now makes X the direct parent of the picked task, instead of the
  parent of the whole top-level branch containing it. Selecting a nested task no
  longer creates a new top-level task wrapping the entire tree.

## [0.2.1] - 2026-10-08

### Changed

- Rewrote `man/trk.1` as a tighter, complete usage book: removed the repeated
  per-command "thought process" prose in favour of one worked session, fixed
  stale references to the old `need` command, documented the one-letter command
  aliases, and expanded the explanation of `trk then` and how it differs from
  `trk add`.
- Trimmed `README.md` to an introduction, quick start, and command reference,
  deferring the full walkthrough to the man page; fixed an invalid `goal new -w`
  example and an incorrect alias listing.
- Regenerated `man/trk.1.generated` so its subcommand skeleton names the current
  `add` command instead of the removed `need`.
- The crate version now tracks the release tags. Earlier `0.2.x` tags built
  binaries that still reported `0.1.0` because `Cargo.toml` had not been bumped;
  it is now `0.2.1`.

### Fixed

- `trk inbox take -s/--switch` now actually picks a parent task inside the
  chosen goal, as documented. The flag was parsed but ignored, so the taken
  item was always added as a root; with `--new-goal` it now reports a usage
  error, since a fresh goal has no task to parent under.
- `trk goal switch N` and `trk goal reopen N` now use the same numbering as
  `trk goal list` (which numbers every goal). They previously numbered only the
  open goals, so the numbers diverged once a done goal existed, and `goal
  reopen` could never select a done goal at all.
- `trk agenda` no longer drops tasks scheduled for later today: they appear
  under a `Today` heading instead of vanishing. It also compares against the
  injected clock rather than the wall clock, so the view is deterministic under
  `TRK_NOW`.

## [0.2.0] - 2026-10-07

### Added

- `current_color` config key selecting the color of the current task (default
  `green`; also `red`, `yellow`, `blue`, `magenta`, `cyan`, `white`, and the
  `bright-*` variants).
- Colored output: the goal title is bold white, the current task is bold in
  `current_color`, and every other task is faded green so the current task
  pops. Done/dropped rows and notes are dim, warnings are orange, and errors
  are red. Applies to the CLI views and the `trk start` task pane.
- This `CHANGELOG.md`, maintained for every code change.

### Changed

- Renamed the `need` command to `add`: it adds a new task under the current
  task (or, with `-s`, a picked one) and descends onto it. The target-selection
  flag `-s/--switch` also accepts `-p` as a short alias.
- CI runs tests only on merges to `main`; the release build runs only after the
  tests pass, and no longer runs on pull requests or feature-branch pushes.
- Releases now publish the matching `CHANGELOG.md` section as the release body
  instead of generated notes.

### Fixed

- Release notes no longer repeat "Full Changelog" once per build-matrix job;
  the release is now created a single time after all assets are built.

## [0.1.1] - 2026-10-07

### Changed

- Renamed commands: `also` is now `by`, `add` is now `and`, and `pick` is now
  `switch`.
- The target-selection flag `-p/--pick` is now `-s/--switch`; `inbox take -s`
  became `-a/--activate`.
- `switch` and `goal switch` take a number or open a picker.

### Fixed

- `then` now wraps the current task's whole chain in a new parent and keeps the
  cursor on the current task, so repeated `then` builds a step-by-step plan and
  `done` walks back up it in order. Previously each `then` inserted a parent
  directly above the current task, which inverted the order.

## [0.1.0] - 2026-10-07

### Added

- Initial release: goals, tasks, the why chain, the inbox, scheduling, the
  agenda, the log, pickers, the `trk start` TUI, undo, shell completions, and
  the man page.

[Unreleased]: https://github.com/darkfoam/trk/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/darkfoam/trk/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/darkfoam/trk/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/darkfoam/trk/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/darkfoam/trk/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/darkfoam/trk/releases/tag/v0.1.0
