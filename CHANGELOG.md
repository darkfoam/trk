# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/darkfoam/trk/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/darkfoam/trk/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/darkfoam/trk/releases/tag/v0.1.0
