#!/bin/sh
# Builds the static archive for this host into go/samefn/lib/<goos>_<goarch>/libsamefn.a (for C
# and C++ users), then the Go package's object from it (scripts/build-go-object.sh). Prints the
# native libraries a link of the archive needs.
set -eu
cd "$(dirname "$0")/.."
goos=$(go env GOOS); goarch=$(go env GOARCH)
# macOS: the oldest release Go supports, so linking TiDB gives no version warnings.
export MACOSX_DEPLOYMENT_TARGET=12.0
cargo rustc --release --lib --crate-type staticlib -- --print native-static-libs 2>&1 | grep -i 'native-static-libs' || true
out=go/samefn/lib/${goos}_${goarch}
mkdir -p "$out"
cp target/release/libsamefn.a "$out/libsamefn.a"
case "$goos" in
  darwin) strip -S "$out/libsamefn.a" ;;
  *) strip --strip-debug "$out/libsamefn.a" ;;
esac
ls -l "$out/libsamefn.a"
./scripts/build-go-object.sh
