[![ci](https://github.com/darkfoam/trk/actions/workflows/ci.yml/badge.svg)](https://github.com/darkfoam/trk/actions/workflows/ci.yml)
[![release](https://github.com/darkfoam/trk/actions/workflows/release.yml/badge.svg)](https://github.com/darkfoam/trk/actions/workflows/release.yml)
[![latest release](https://img.shields.io/github/v/release/darkfoam/trk?sort=semver)](https://github.com/darkfoam/trk/releases/latest)

# trk

> adhd compliant task tracking

`trk` tracks what you are doing, and why, while your work branches into side
quests. It targets rabbit holes: you start a task, discover you need another,
then another, and three hours later you cannot remember why you came here.

It is one binary and one plain-text file. The file is human-readable,
hand-editable, safe to keep in git, and safe to back up with any tool.

## Install

### macOS

Download `trk-<version>-macos-<arch>.dmg` from the
[latest release](https://github.com/darkfoam/trk/releases/latest), open it, and
run the installer. It installs the binary, man page, and shell completions under
`/usr/local`, so `man trk` works.

The installer is unsigned, so macOS warns about an unidentified developer:
right-click the dmg and choose Open, or clear the quarantine flag first:

```sh
xattr -dr com.apple.quarantine ~/Downloads/trk-*-macos-*.dmg
```

Uninstall with:

```sh
sudo rm -f /usr/local/bin/trk /usr/local/share/man/man1/trk.1 \
  /usr/local/share/zsh/site-functions/_trk \
  /usr/local/share/bash-completion/completions/trk \
  /usr/local/share/fish/vendor_completions.d/trk.fish
sudo pkgutil --forget com.darkfoam.trk
```

### Build from source

The project targets stable Rust, edition 2024 (MSRV 1.89):

```sh
cargo build --release
install -m 0755 target/release/trk ~/.local/bin/trk
```

`cargo install --path .` is also supported; it installs to `~/.cargo/bin`.

Or use the convenience script, which builds the release, installs the binary to
`~/.local/bin`, installs the man page to `~/.local/share/man/man1`, and
refreshes the `whatis` index (no `sudo`):

```sh
scripts/install-local.sh
```

Override the destinations with `TRK_BIN_DIR` and `TRK_MAN_DIR` if you keep them
elsewhere.

Install shell completions (optional):

```sh
trk completions            # installs for your detected shell
```

## First run

The first time you run a command that needs the store, `trk` asks where to keep
your task list and its backups, then writes a config file. Press Enter to accept
the defaults. If you have no terminal, set `TRK_STORE` to skip the prompts, or
create the config file by hand (see [Configuration](#configuration)).

## Quick start

```sh
trk new goal "Ship login fix"   # start a goal, named for the outcome
trk add "Fix login bug"         # a top-level task under the goal
trk then "Reproduce on staging" # a parent of what you just added
trk and "Call internet company" # a sibling of what you just added
trk list                        # where am I, and what surrounds me?
trk time                        # start timing the current task
trk done                        # finish it (and stop the timer)
trk progress goal               # the whole goal as a tree
```

## How it works

Work is a **tree of tasks under a goal**. One goal is active at a time, and it
keeps its own **cursor**: the task you are currently on, plus a pointer to the
**most recently added** task.

Four commands add tasks, each relative to something already there:

| Command | Adds | Relative to |
|---|---|---|
| `trk add X` | a top-level task | the goal |
| `trk then X` | a parent of the last task you added | the most recently added task (`-n` for the current one) |
| `trk and X` | a sibling after the last task you added | the most recently added task |
| `trk before X` | a sibling before the last task you added | the most recently added task |

So `add` starts a new line of work, `then` wraps the last task in a parent
("first this, then that"), and `and`/`before` branch sideways. Because the most
recently added task is remembered, they chain across commands — and if you
realize a step was missing, `trk before X` slots it between the last two.

`add`, `then`, `and`, and `before` each take an optional **why**: `-w <why>`
supplies it inline, otherwise `trk` asks `why?` (press Enter for none, or pass
`--no-why`). The why is shown under the task in `trk list` and
`trk progress goal`.

`trk list` shows the current task, its siblings, and up to three of its parents,
with a pointer at the current task. `trk progress goal` shows the entire tree,
with completed tasks struck through. A parent cannot be completed while it still
has open sub-tasks: `trk done` refuses and exits 3 until you pass `--force`.

`trk time` records time against the current task; `trk stop` (or `trk done`)
stops the timer, and totals show up in `trk progress goal` and `trk log`.

For the complete walkthrough, see the man page (`man trk`) or
`trk <command> --help`.

## Command reference

| Command | What it does |
|---|---|
| `trk` | the current-task context view |
| `trk add <text>` | new top-level task under the goal |
| `trk then [-n] <text>` | new parent of the most recently added task (`-n`: current task) |
| `trk and <text>` | new sibling after the most recently added task |
| `trk before <text>` | new sibling before the most recently added task |
| `trk list` | current task, siblings, and up to three parents |
| `trk switch` | choose the current task with the picker |
| `trk done [-f]` | complete the current task; stop its timer |
| `trk time` | start timing the current task |
| `trk stop` | stop timing the current task |
| `trk drop [-y]` | delete a task with the picker (`-y` if it has sub-tasks) |
| `trk undo [N]` | undo the last N changes |
| `trk new goal [title]` | create a goal (prompts when the title is omitted) |
| `trk list goal` | list every goal |
| `trk switch goal` | choose the active goal with the picker |
| `trk progress goal` | the whole goal as a tree, completed tasks struck through |
| `trk delete goal [-y]` | delete a goal and its tasks (confirms, or `--yes`) |
| `trk jot <text>` | capture a task in the inbox |
| `trk inbox` | file inbox items with the picker (TAB creates a goal) |
| `trk log [when]` | what got finished, by day, with time spent |
| `trk completions [shell]` | install shell completions for your shell |

Aliases: `d` done, `l` list, `u` undo, `i` jot.

Without a terminal a picker prints its numbered choices to stderr and reads the
number from stdin (an empty line accepts the highlighted default), so it can be
scripted.

## Configuration

The config file is TOML, found at `$TRK_CONFIG`, else
`$XDG_CONFIG_HOME/trk/config.toml`, else `$HOME/.config/trk/config.toml`:

```toml
store = "~/.local/share/trk/tasks.trk"
backups = "~/.local/share/trk/backups"
color = "auto"           # auto | always | never
wrap = 80                # maximum output width in columns
undo_depth = 50          # number of undo snapshots kept
prompt_why = true        # ask "why?" when adding a task
ascii = "auto"           # auto | always | never
current_color = "green"  # color of the current task
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
Completed tasks are dim and struck through, warnings are orange, and errors are
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
