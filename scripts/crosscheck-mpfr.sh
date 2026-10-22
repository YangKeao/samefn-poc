#!/bin/sh
# Checks sin version 1 against MPFR on all of upstream's hard-to-round inputs (and their
# negations) plus N seeded random inputs (default 10,000,000). Needs MPFR and GMP.
set -eu
cd "$(dirname "$0")/.."
n=${1:-10000000}
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
./scripts/vendor-core-math.sh --fetch-wc "$tmp" >/dev/null
cargo build -q --release
inc=; lib=
if [ -d /opt/homebrew ]; then inc=-I/opt/homebrew/include; lib=-L/opt/homebrew/lib; fi
cc -O2 -std=c11 -Iinclude $inc ctest/crosscheck_mpfr.c target/release/libsamefn.a $lib \
  -lmpfr -lgmp $(./scripts/native-libs.sh) -o "$tmp/crosscheck"
"$tmp/crosscheck" "$tmp/sin.wc" "$n"
