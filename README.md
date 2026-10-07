# trk

> adhd compliant task tracking

`trk` is a command-line tool for tracking what you are doing, and why, while
your work branches into side quests. It targets rabbit holes: you start a task,
discover you need another one, then another, and three hours later you cannot
remember why you came here. `trk` keeps the chain.

## The name

The program is called `trk` on purpose. It is three lowercase letters, needs no
shifted characters, and becomes muscle memory within days. Typing cost is a
first-class design constraint, and it applies to every command:

- Commands are short, lowercase, plain words that read like thoughts
  (`need`, `then`, `also`, `done`).
- No command requires quoting. Free text is taken from the remaining arguments
  as typed.
- No command requires a shifted symbol or an awkward key combination.
- The most common action takes the fewest keystrokes.
- A bare `trk` always answers "what was I doing, and why?".

## Core idea

Work is a tree of tasks under a goal. One task is the current task. Four moves
cover almost everything that happens mid-work:

1. "I need X to finish this." A child of the current task; you descend.
2. "Also, X." An unrelated top-level task; you stay where you are.
3. "Then, once this is done, X." A new parent above a task; you stay.
4. "Task T needs X." A child of some other task T; you stay.

A fifth action, finishing, walks you back up the chain and tells you where you
are and why.

## Quick start

```sh
trk goal new "Ship login fix"     # start a goal
trk also "Fix login bug"          # add a top-level task
trk need "Reproduce on staging"   # descend into what this needs
trk                               # what am I doing, and why?
trk done                          # finish and walk back up
trk start                         # live list in a terminal pane
```

The store is a single, hand-editable UTF-8 text file. It is safe to keep in git
and to back up with any tool. See `INSTRUCTIONS.md` for the full specification,
file format, and command reference.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -p xtask -- man           # regenerate man/trk.1.generated (flag skeleton)
cargo run -p xtask -- completions   # regenerate completions/
```

`man/trk.1` is the hand-written usage book (see `man trk`); `xtask man` writes a
generated flag skeleton to `man/trk.1.generated` so the two can be diffed for
drift.

The project targets stable Rust, edition 2024 (MSRV 1.89), and forbids unsafe
code. CI runs the gates above on Linux, macOS, and Windows.
