//go:build cgo && darwin

package samefn

// The library itself is samefn_darwin_<goarch>.syso; these are the system libraries it needs.

// #cgo LDFLAGS: -lm
import "C"
