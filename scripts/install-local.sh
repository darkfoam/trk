#!/usr/bin/env bash
#
# Build trk and copy it (plus its man page) into place for the current user,
# overwriting whatever is already there. A plain build and copy: no installer
# packaging, no sudo.
#
#   bin  -> ~/.local/bin/trk
#   man  -> ~/.local/share/man/man1/trk.1
#
# Both directories are on the default PATH / manpath on most macOS and Linux
# systems. Override them with TRK_BIN_DIR and TRK_MAN_DIR.
#
# Usage: scripts/install-local.sh
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root_dir"

bindir="${TRK_BIN_DIR:-$HOME/.local/bin}"
mandir="${TRK_MAN_DIR:-$HOME/.local/share/man}"

cargo build --release

install -d "$bindir" "$mandir/man1"
install -m 0755 target/release/trk "$bindir/trk"
install -m 0644 man/trk.1 "$mandir/man1/trk.1"

# Refresh the whatis index so `man -k` finds the page. macOS ships
# /usr/libexec/makewhatis; Linux uses mandb. Best effort either way.
if command -v mandb >/dev/null 2>&1; then
  mandb "$mandir" >/dev/null 2>&1 || true
elif [ -x /usr/libexec/makewhatis ]; then
  /usr/libexec/makewhatis "$mandir" || true
fi

echo "installed $bindir/trk" >&2
echo "installed $mandir/man1/trk.1" >&2
