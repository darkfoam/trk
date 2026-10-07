# AGENTS.md

## Commands
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test` (single test: `cargo test <name>`)
- Stable Rust, edition 2024, MSRV 1.89 (`Cargo.toml`). `#![forbid(unsafe_code)]`.
  Binary crate named `trk`.
- CI matrix covers Linux, macOS, and Windows.

## Layout
- `src/model/` pure domain types, `ops` (mutations), `tree` helpers.
- `src/store/` format parser/serializer, atomic writes, locking, undo ring.
- `src/ui/` pure renderers returning `Vec<StyledLine>`, wrapping, styles.
- `src/commands/` CLI dispatch and interactivity.
- `src/tui/` `trk start` and the reusable picker.
- `tests/` integration tests; `man/` hand-written man page; `xtask/` generators.

## Architecture rules that are easy to get wrong
- Ops are pure: `(Doc, Request) -> Result<(Doc, Outcome), OpError>`. No I/O, no
  clock reads (pass `now` in), no prompts. CLI and TUI call the same ops so
  behavior cannot drift.
- Views are pure: `(&Doc, Width, Style) -> Vec<StyledLine>`; no printing.
- No global mutable state; pass a `Ctx` (config, clock, paths) explicitly.
- `thiserror` in library layers, `anyhow` only in `main`; map errors to exit
  codes in one place.
- Keep dependencies small; no async runtime, no database.

## Testing quirks
- CLI integration tests use `TRK_STORE`, `TRK_CONFIG`, and `TRK_NOW` with temp
  dirs to make output deterministic. `TRK_NOW` is test-only.
- Dev stack: `assert_cmd`, `predicates`, `tempfile`, `insta`, `proptest`.
- Coverage includes parser/serializer round-trip, view snapshots at widths
  40/60/80 plus ASCII symbols, CRLF parsing, concurrency, and exit codes with
  the stdout/stderr split.

## Conventions
- All program output wraps at 80 columns.
- Errors: `trk: <lowercase message>`; warnings: `warning: <msg>` in orange
  (ANSI 256-color 208). Exit codes: 3 blocked, 4 nothing to act on, 5 cancelled.
- Commands are lowercase plain words; free text is collected from remaining
  positional args, no quoting.
- `man/trk.1` is hand-written; `cargo run -p xtask -- man` writes a generated
  flag skeleton to `man/trk.1.generated` for drift-checking.
