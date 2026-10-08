#!/usr/bin/env bash
#
# Build a macOS installer for trk and wrap it in a .dmg.
#
# The payload is a component .pkg that installs, under /usr/local:
#   bin/trk                                      the binary
#   share/man/man1/trk.1                         the man page
#   share/zsh/site-functions/_trk                zsh completion
#   share/bash-completion/completions/trk        bash completion
#   share/fish/vendor_completions.d/trk.fish     fish completion
#
# Usage: scripts/package-macos.sh <version> <target-triple>
#   scripts/package-macos.sh 0.3.0 aarch64-apple-darwin
#
# The binary at target/<triple>/release/trk is used if present, otherwise it is
# built. Writes trk-<version>-macos-<arch>.dmg in the repo root.
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <version> <target-triple>" >&2
  exit 2
fi

version="$1"
triple="$2"

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root_dir"

bin="target/${triple}/release/trk"
if [ ! -x "$bin" ]; then
  echo "building trk for ${triple}..." >&2
  cargo build --release --locked --target "$triple"
fi
if [ ! -x "$bin" ]; then
  echo "error: $bin not found" >&2
  exit 1
fi

arch="${triple%%-*}"
pkg_name="trk-${version}-macos-${arch}.pkg"
dmg_name="trk-${version}-macos-${arch}.dmg"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

payload="$work/root"
install -d "$payload/usr/local/bin"
install -m 0755 "$bin" "$payload/usr/local/bin/trk"

install -d "$payload/usr/local/share/man/man1"
install -m 0644 man/trk.1 "$payload/usr/local/share/man/man1/trk.1"

install -d "$payload/usr/local/share/zsh/site-functions"
install -m 0644 completions/_trk "$payload/usr/local/share/zsh/site-functions/_trk"

install -d "$payload/usr/local/share/bash-completion/completions"
install -m 0644 completions/trk.bash \
  "$payload/usr/local/share/bash-completion/completions/trk"

install -d "$payload/usr/local/share/fish/vendor_completions.d"
install -m 0644 completions/trk.fish \
  "$payload/usr/local/share/fish/vendor_completions.d/trk.fish"

pkg="$work/$pkg_name"
pkgbuild \
  --root "$payload" \
  --identifier com.darkfoam.trk \
  --version "$version" \
  --install-location / \
  "$pkg" >/dev/null

staging="$work/dmg"
install -d "$staging"
cp "$pkg" "$staging/"

dmg="$root_dir/$dmg_name"
rm -f "$dmg"
hdiutil create \
  -volname "trk $version" \
  -srcfolder "$staging" \
  -ov -format UDZO -fs HFS+ \
  "$dmg" >/dev/null

echo "wrote $dmg"
