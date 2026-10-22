#!/bin/sh
# Builds ctest/check_corpus.c against the freshly built archive and runs it on the corpus.
set -eu
cd "$(dirname "$0")/.."
cargo build -q --release
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
cc -O2 -std=c11 -Iinclude ctest/check_corpus.c target/release/libsamefn.a $(./scripts/native-libs.sh) -o "$tmp/check"
"$tmp/check" corpus/sin_v1.tsv
