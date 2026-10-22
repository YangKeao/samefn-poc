//go:build cgo && linux

package samefn

// The library itself is internal/obj/samefn_linux_<goarch>.syso; these are the system libraries it needs
// (from `cargo rustc -- --print native-static-libs`).

// #cgo LDFLAGS: -lgcc_s -lutil -lrt -lpthread -lm -ldl -lc
import "C"
