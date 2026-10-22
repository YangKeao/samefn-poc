#!/bin/sh
# Verifies that the vendored CORE-MATH file is byte-identical to upstream at the pinned
# commit (third_party/core-math/PIN). With --fetch-wc DIR, also leaves upstream's list of
# hard-to-round sin inputs (sin.wc) in DIR for scripts/crosscheck-mpfr.sh.
set -eu
cd "$(dirname "$0")/.."
pin=third_party/core-math/PIN
get() { sed -n "s/^$1=//p" "$pin"; }
sha() { if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -c1-64; }
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
git -C "$tmp" init -q
git -C "$tmp" remote add origin "$(get url)"
git -C "$tmp" fetch -q --depth 1 origin "$(get commit)"
git -C "$tmp" checkout -q FETCH_HEAD
file=$(get file); want=$(get file_sha256)
up=$(sha "$tmp/$file"); ours=$(sha "third_party/core-math/$file")
echo "pinned  $want"; echo "upstream $up"; echo "vendored $ours"
[ "$up" = "$want" ] && [ "$ours" = "$want" ] || { echo "MISMATCH"; exit 1; }
if [ "${1:-}" = "--fetch-wc" ]; then cp "$tmp/src/binary64/sin/sin.wc" "$2/"; fi
echo "ok: third_party/core-math/$file is CORE-MATH $(get commit)"
