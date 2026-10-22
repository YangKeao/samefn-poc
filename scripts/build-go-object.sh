#!/bin/sh
# Turns this host's archive (go/samefn/lib/<goos>_<goarch>/libsamefn.a) into the object the Go
# package links, go/samefn/samefn_<goos>_<goarch>.syso: the archive prelinked into one object
# in which only the samefn_* symbols stay global, so the copy of Rust's standard library inside
# cannot clash with other code. Go and Bazel's rules_go link a package's .syso files, chosen by
# the platform in the file name, without any link flags pointing at a path.
set -eu
cd "$(dirname "$0")/.."
goos=$(go env GOOS); goarch=$(go env GOARCH)
archive=go/samefn/lib/${goos}_${goarch}/libsamefn.a
out=go/samefn/samefn_${goos}_${goarch}.syso
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
case "$goos" in
  darwin)
    echo '_samefn_*' > "$tmp/exports"
    # Apple's ld: with -r, the symbols not exported become static.
    ld -r -arch "$(uname -m)" -platform_version macos 12.0 12.0 -o "$tmp/samefn.o" \
      -force_load "$archive" -exported_symbols_list "$tmp/exports"
    strip -S -o "$out" "$tmp/samefn.o" ;;
  *)
    ld -r -o "$tmp/samefn.o" --whole-archive "$archive" --no-whole-archive
    objcopy --wildcard --keep-global-symbol='samefn_*' --strip-debug "$tmp/samefn.o" "$out" ;;
esac
ls -l "$out"
