# AGENTS.md

## Read this first
- `INSTRUCTIONS.md` (1519 lines) is the authoritative spec for `trk` and is
  **gitignored**, so it will not appear in `git status`/diffs. Read it fully
  before writing code. If spec and instinct disagree, follow the spec.
- `trk` is being built from scratch. Current state is **M0 scaffold**: only
  `src/main.rs` "Hello, world!". Follow the build order in §11.9 (M0→M12); do
  not start a milestone before the previous one lands with passing tests and
  clean clippy.
- §6 (file format) and §8 (commands) are the contract; §13 acceptance
  scenarios are the required automated tests.

## Commands
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test` (single test: `cargo test <name>`)
- Stable Rust, edition 2024 (`Cargo.toml`). Declare an MSRV and test it in CI.
- `#![forbid(unsafe_code)]`. Binary crate named `trk`.
- CI matrix must cover Linux, macOS, Windows (§11.1).

## Testing quirks
- CLI integration tests use `TRK_STORE`, `TRK_CONFIG`, and `TRK_NOW` with temp
  dirs to make output deterministic (§12, §13). `TRK_NOW` is test-only.
- Dev stack: `assert_cmd`, `predicates`, `tempfile`, `insta`, `proptest`.
- Required beyond unit tests: parser/serializer round-trip property test,
  view snapshots at widths 40/60/80 plus ASCII symbols, CRLF fixture parsing,
  concurrency test, and the §13 scenarios incl. exit codes and stdout/stderr
  split.

## Architecture rules that are easy to get wrong
- Ops are pure: `(Doc, Request) -> Result<(Doc, Outcome), OpError>`. No I/O,
  no clock reads (pass `now` in), no prompts. Both CLI and TUI call the same
  ops so behavior cannot drift (§11.5).
- Views are pure: `(&Doc, Width, Style) -> Vec<StyledLine>`; no printing
  (§11.6).
- No global mutable state; pass a `Ctx` (config, clock, paths) explicitly.
- `thiserror` in library layers, `anyhow` only in `main`; map errors to exit
  codes in one place.
- Follow the module layout in §11.3; keep dependencies small, no async
  runtime, no database (§11.2).

## Conventions
- Spec prose and all program output wrap at 80 columns. Preserve that if you
  edit `INSTRUCTIONS.md`.
- Errors: `trk: <lowercase message>`; warnings: `warning: <msg>` in orange
  (ANSI 256-color 208). Exit codes: 3 blocked, 4 nothing to act on, 5 cancelled
  (§4.3).
- Commands are lowercase plain words; free text is collected from remaining
  positional args, no quoting (§4.1).
