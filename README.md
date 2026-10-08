[![ci](https://github.com/darkfoam/trk/actions/workflows/ci.yml/badge.svg)](https://github.com/darkfoam/trk/actions/workflows/ci.yml)
[![release](https://github.com/darkfoam/trk/actions/workflows/release.yml/badge.svg)](https://github.com/darkfoam/trk/actions/workflows/release.yml)
[![latest release](https://img.shields.io/github/v/release/darkfoam/trk?sort=semver)](https://github.com/darkfoam/trk/releases/latest)

# trk

> adhd compliant task tracking

`trk` is a command-line tool for tracking what you are doing, and why, while
your work branches into side quests. It targets rabbit holes: you start a task,
discover you need another one, then another, and three hours later you cannot
remember why you came here. `trk` keeps the chain.

It is one binary and one plain-text file. No time tracking, no network, no
accounts, no daemon. The file is human-readable, hand-editable, safe to keep in
git, and safe to back up with any tool.

## Install

Build from source. The project targets stable Rust, edition 2024 (MSRV 1.89):

```sh
cargo build --release
```

The binary is at `target/release/trk`. Install it onto your `PATH`:

```sh
install -m 0755 target/release/trk ~/.local/bin/trk
```

`~/.local/bin` is on `PATH` for most users on Linux and macOS. If it is not,
add it:

```sh
# bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
# zsh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
# fish
fish_add_path ~/.local/bin
```

Common alternatives on Unix-like systems:

- **macOS**: `~/.local/bin`, or `/opt/homebrew/bin` (Apple Silicon) and
  `/usr/local/bin` (Intel) if you use Homebrew.
- **Linux**: `~/.local/bin` per user, or `/usr/local/bin` system-wide with
  `sudo install -m 0755 target/release/trk /usr/local/bin/trk`.
- **FreeBSD**: `/usr/local/bin` (the ports convention), or `~/.local/bin` per
  user.

`cargo install --path .` is also supported; it installs to `~/.cargo/bin`.

Install the man page (optional):

```sh
install -d ~/.local/share/man/man1
install -m 0644 man/trk.1 ~/.local/share/man/man1/trk.1
man trk
```

Install shell completions (optional):

```sh
trk completions bash > ~/.local/share/bash-completion/completions/trk
trk completions zsh  > "${fpath[1]}/_trk"
trk completions fish > ~/.config/fish/completions/trk.fish
```

## First run

The first time you run a command that needs the store, `trk` asks where to keep
your task list and its backups, then writes a config file. Press Enter to accept
the defaults:

```
Where should trk keep your task list?
(a plain text file; you can back it up or put it in git)
path [~/.local/share/trk/tasks.trk]:

Where should trk keep its backups of the task list?
(undo snapshots; a folder, ideally somewhere you also back up)
folder [~/.local/share/trk/backups]:
```

The first task you create in an empty store triggers a short walkthrough that
asks `What *are* you working on?` and turns your answer into a goal.

If you have no terminal (for example in a script), set `TRK_STORE` to skip the
prompts, or create the config file by hand (see [Configuration](#configuration)).

## Quick start

```sh
trk goal new "Ship login fix"     # start a goal, named for the outcome
trk by "Fix login bug" -w "customers locked out"
trk add "Reproduce on staging"    # descend into what this needs
trk                               # what am I doing, and why?
trk done                          # finish and walk back up
trk start                         # live list in a terminal pane
```

`-w` records the why inline; without it, `trk` asks `why?`.

## How it works

Work is a **tree of tasks under a goal**. One goal is active at a time, and it
keeps its own **cursor**: the task you are currently on. Switching goals is
cheap and resumes exactly where you left off. A task's children are the things
needed to finish it, so the path from the goal down to your cursor is a literal
list of reasons — the **why chain**.

Four moves cover almost everything that happens mid-work:

| Move | Meaning | Cursor |
|---|---|---|
| `trk add X` | a child of the current task: "I need X to finish this" | descends |
| `trk by X` | an unrelated top-level task: "also, X" | stays |
| `trk and X` | a sibling right after the target: "and X, at this level" | stays |
| `trk then X` | a new parent wrapping the current branch: "then, X" | stays |

`add` records what you need *before* the current task; `then` records what
happens *after* it. Repeated `then` grows a plan forward from the first step
while your cursor stays put, and `trk done` walks you back up it one task at a
time.

A parent cannot be completed while it still has open descendants: `trk done`
refuses, lists what is left, and exits 3 until you pass `--force`. `trk goal
done` holds the same line for goals. This is the tool reminding you that "done"
still has loose ends.

For the complete walkthrough — every command, its options, the file format, and
a full worked session — see the man page (`man trk`) or
`trk <command> --help`.

## Command reference

| Command | What it does |
|---|---|
| `trk` | what am I doing, and why |
| `trk add <text>` | new child of current task; descend |
| `trk by <text>` | new top-level task; stay |
| `trk then <text>` | new parent wrapping the current branch; stay |
| `trk and <text>` | new sibling after target; stay |
| `trk done` | finish current; walk back up |
| `trk drop` | abandon current (reversible) |
| `trk stop [reason]` | leave current unfinished, record why |
| `trk go` | resume current; show how I got here |
| `trk why` | chain of notes from the root to the current task |
| `trk note [text]` | read or add note lines on a task |
| `trk rename <text>` | change a task's text |
| `trk switch [N]` | choose the current task |
| `trk list` | the whole task tree of the active goal |
| `trk undo [N]` | undo the last N changes |
| `trk goal ...` | show, list, new, switch, done, rename, reopen |
| `trk jot <text>` | quick capture to the inbox |
| `trk inbox ...` | view the inbox, take or drop items |
| `trk log [when]` | what got finished, by day |
| `trk at [when]` | schedule a task |
| `trk agenda` | scheduled tasks across all goals |
| `trk start` | live task list in a terminal pane |
| `trk config ...` | where the task list lives |
| `trk prompt` | one short line for a shell prompt |
| `trk completions <shell>` | print a shell completion script |

Aliases: `d` done, `u` undo, `l` list, `n` note, `w` why, `s` stop, `g` goal,
`i` jot.

Useful flags: `-s/--switch` chooses a target in a picker; `-w <why>` supplies
the why inline; `-n/--stay` keeps the cursor put; `-f/--force` overrides a
blocked `done`/`drop`/`goal done`; `--no-why` skips the why prompt.

## Configuration

The config file is TOML, found at `$TRK_CONFIG`, else
`$XDG_CONFIG_HOME/trk/config.toml`, else `$HOME/.config/trk/config.toml`:

```toml
store = "~/.local/share/trk/tasks.trk"
backups = "~/.local/share/trk/backups"
color = "auto"        # auto | always | never
wrap = 80             # maximum output width in columns
undo_depth = 50       # number of undo snapshots kept
prompt_why = true     # ask "why?" when creating a task
ascii = "auto"        # auto | always | never
current_color = "green"  # color of the current task
poll_ms = 500         # `trk start` change-detection interval
```

Environment variables:

- `TRK_CONFIG` — path to the config file.
- `TRK_STORE` — path to the store, overriding `store` in the config. Setting it
  with no config skips the first-run flow.
- `NO_COLOR` — disables color.
- `TRK_ASCII=1` — forces ASCII symbols instead of Unicode.

Exit codes: `0` success, `1` general error, `2` usage error, `3` blocked,
`4` nothing to act on, `5` cancelled.

## Colors

The goal title is bold white. The current task is bold in `current_color`
(default `green`). Every other task is faded green, so the current task pops.
Done and dropped rows and notes are dim, warnings are orange, and errors are
red. Color is enabled automatically on a terminal; force it with
`--color always` or `color = "always"`, and disable it with `--color never`,
`color = "never"`, or by setting `NO_COLOR`.

## Development

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test
cargo run -p xtask -- man           # regenerate man/trk.1.generated (flag skeleton)
cargo run -p xtask -- completions   # regenerate completions/
```

`man/trk.1` is the hand-written usage book (see `man trk`); `xtask man` writes a
generated flag skeleton to `man/trk.1.generated` so the two can be diffed for
drift.

CI runs the gates above on Linux, macOS, and Windows.
