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

## The name

The program is called `trk` on purpose. It is three lowercase letters, needs no
shifted characters, and becomes muscle memory within days. Typing cost is a
first-class design constraint, and it applies to every command:

- Commands are short, lowercase, plain words that read like thoughts
  (`need`, `by`, `and`, `then`, `done`).
- No command requires quoting. Free text is taken from the remaining arguments
  as typed.
- No command requires a shifted symbol or an awkward key combination.
- The most common action takes the fewest keystrokes.
- A bare `trk` always answers "what was I doing, and why?".

## Core idea

Work is a tree of tasks under a goal. One task is the current task. Five moves
cover almost everything that happens mid-work:

1. "I need X to finish this." A child of the current task; you descend. (`need`)
2. "Also, X." An unrelated top-level task; you stay. (`by`)
3. "And, X." Another task at the same level as the current one. (`and`)
4. "Then, once this is done, X." A new parent that wraps the current chain; you
   stay. (`then`)
5. "Task T needs X." A child of some other task T; you stay. (`need -s`)

A fifth action, finishing, walks you back up the chain and tells you where you
are and why.

A **goal** is the big thing you are working toward; it has a title, a status,
and its own tree of tasks. Exactly one goal is active at a time, and every goal
remembers its own cursor, so switching goals resumes exactly where you left off.
A **task** is one concrete step; tasks nest, and a child is "something I need in
order to finish the parent". That single rule is what makes the **why chain**
work: the path from the goal root down to your current task is a literal list of
reasons. A parent cannot be completed while it has open children unless you
explicitly force it, which is the tool reminding you that "done" still has loose
ends.

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
trk goal new "Ship login fix"     # start a goal
trk by "Fix login bug"            # add a top-level task
trk need "Reproduce on staging"   # descend into what this needs
trk                               # what am I doing, and why?
trk done                          # finish and walk back up
trk start                         # live list in a terminal pane
```

## A worked session

This walks from an empty store to a finished goal, narrating the thought
process at each step. `$` marks the command; the rest is what you would be
thinking.

**Starting a goal.** I am about to fix a login problem. Before I touch code, I
name the outcome, not the first step:

```sh
$ trk goal new Ship login fix
goal: Ship login fix
Ship login fix
no current task
try: trk goal done or trk by <task>
```

**The first task.** The first concrete thing is the bug itself. It is top
level, so I use `by`, not `need`:

```sh
$ trk by Fix login bug -w "customers locked out"
added: Fix login bug
Ship login fix
  > Fix login bug
      why: customers locked out
```

The empty-cursor rule made this task current even though `by` normally stays
put. The `-w` flag recorded the why inline; without it, `trk` would have asked
`why?`.

**A prerequisite.** I cannot fix what I cannot see, so I descend:

```sh
$ trk need Reproduce on staging -w "need a failing case"
Ship login fix
  Fix login bug
    > Reproduce on staging
        why: need a failing case
```

The chain now reads like a sentence: reproducing on staging, because I need a
failing case, to fix the login bug, because customers are locked out.

**An unrelated interruption.** The wifi drops and I have to call the internet
company. That is not part of the login fix; it is a separate top-level task:

```sh
$ trk by Call internet company -w "wifi flaky"
added: Call internet company
```

My cursor did not move. The interruption was captured without derailing me. If
I had used `need`, it would have been buried under the current task, which would
be wrong.

**A sibling.** I realize I should also check the auth logs, and that belongs at
the same level as "Reproduce on staging", not underneath it. `and` puts a task
right after a target:

```sh
$ trk and Check auth logs -w "might explain the 401s"
```

If I want to add it under a different task instead, I can target one with `-s`:

```sh
$ trk need -s Check auth logs        # opens a picker of tasks to parent under
```

**A whole new goal, mid-work.** While debugging, the release manager pings me
about a completely different job. I do not want to lose my place, so I create a
new goal and switch to it:

```sh
$ trk goal new Release 1.4 -w
$ trk by Tag the release -w "blocked on QA"
```

`goal new` makes the new goal active. When I am done there, I switch back and I
am exactly where I was, because each goal keeps its own cursor:

```sh
$ trk goal switch 1
Ship login fix
  Fix login bug
    Reproduce on staging
      > Get staging creds
          why: can't log in
```

Use `trk goal new -n` (or `--stay`) if you want to create a goal without
switching to it, and `trk goal list` to see your goals.

**Reframing the work.** I realize the credential step sits inside a
documentation step that happens after shipping. `then` wraps the whole current
chain in a new parent:

```sh
$ trk then Document the process -w "after the fix ships"
Ship login fix
  Document the process
    Fix login bug
      Reproduce on staging
        > Get staging creds
          why: can't log in
```

My place is unchanged, but it now sits inside a larger, correct frame. Each
further `then` wraps the chain again, so I can keep adding later steps while I
stay on the first one.

**Blocked, then resumed.** I am waiting on a registrar, so I stop the task with
a reason instead of pretending it is done:

```sh
$ trk stop waiting on registrar
...
$ trk go
stopped earlier: waiting on registrar
Ship login fix
  Fix login bug
    ...
```

`stop` leaves the task unfinished and on purpose; `go` clears it and reprints
the why chain so I can resume with full context.

**Finishing and walking back up.** When a step is done, `trk done` finishes it
and moves me to the nearest thing that still needs me, telling me why:

```sh
$ trk done
back to: Reproduce on staging
  why: need a failing case
```

If a sibling still needed me it would have said `next:` and named it; if I
finished the last top-level task it would say `next top-level task:` or report
that the goal has no open tasks. Finishing a parent that still has open children
is blocked with exit code 3 until you pass `--force`.

**Jumping around.** `trk list` prints the tree with numbers, and `trk switch N`
jumps to one:

```sh
$ trk list
Ship login fix
 1  Document the process
 2    Fix login bug
 3      Reproduce on staging
 4        > Get staging creds
 5      Check auth logs
 6  Call internet company
$ trk switch 6
```

**A mistake.** I finished the wrong task:

```sh
$ trk undo
undid: done
```

There is no redo, but `trk undo 2` steps back further.

**A stray thought.** A thought arrives that does not belong anywhere yet. The
inbox captures it without touching a goal or my cursor:

```sh
$ trk jot look into sqlite wal mode
jotted: look into sqlite wal mode  (inbox: 1)
```

Later I file it into a goal:

```sh
$ trk inbox
 1  2026-10-07  look into sqlite wal mode
$ trk inbox take 1 --goal "Ship login fix"
```

**Scheduling and review.** `trk at` schedules a task; `trk agenda` shows
everything due across all goals; `trk log` shows what actually got finished:

```sh
$ trk at tomorrow 3pm
$ trk agenda
$ trk log 7
```

**Closing the goal.** When every task is done, I close the promise:

```sh
$ trk goal done
goal done: Ship login fix
```

## Command reference

| Command | What it does |
|---|---|
| `trk` | what am I doing, and why |
| `trk need <text>` | new child of current task; descend |
| `trk by <text>` | new top-level task; stay |
| `trk then <text>` | new parent wrapping the current chain; stay |
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
`i` inbox.

Useful flags: `-s/--switch` chooses a target in a picker; `-w <why>` supplies
the why inline; `-n/--stay` keeps the cursor put; `-f/--force` overrides a
blocked `done`/`drop`/`goal done`; `--no-why` skips the why prompt.

Run `trk <command> --help` for the full option list, or see `man trk` for the
complete usage book.

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
