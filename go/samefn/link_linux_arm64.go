//go:build cgo

package samefn

// #cgo LDFLAGS: ${SRCDIR}/lib/linux_arm64/libsamefn.a -lgcc_s -lutil -lrt -lpthread -lm -ldl -lc
import "C"
