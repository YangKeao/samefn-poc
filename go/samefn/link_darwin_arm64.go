//go:build cgo

package samefn

// #cgo LDFLAGS: ${SRCDIR}/lib/darwin_arm64/libsamefn.a -lm
import "C"
