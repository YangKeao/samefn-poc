//go:build cgo

// Package samefn calls samefn's versioned, correctly rounded functions, the same machine
// code TiKV and TiFlash run, through cgo. The code comes prebuilt, one object per platform
// (samefn_<goos>_<goarch>.syso), which Go and Bazel's rules_go link automatically.
//
// Each version of a function is its own entry point and never changes; callers pick the
// version explicitly. The functions compute values only: NULL handling, domain checks,
// warnings and errors stay with the caller. A NaN result's sign and payload are not part of
// the contract. Do not combine results with float arithmetic Go could fuse into an FMA.
package samefn

// #include <stdlib.h>
// #include "samefn.h"
import "C"

import "unsafe"

// SinV1 returns sin version 1 of x: correctly rounded.
func SinV1(x float64) float64 { return float64(C.samefn_sin_v1(C.double(x))) }

// SinSliceV1 sets dst[i] = SinV1(src[i]) for every i < len(src), in one call into the
// library. dst may be src (in place); other overlaps are not supported.
func SinSliceV1(dst, src []float64) {
	if len(dst) < len(src) {
		panic("samefn: dst shorter than src")
	}
	if len(src) == 0 {
		return
	}
	C.samefn_sin_batch_v1(
		(*C.double)(unsafe.Pointer(unsafe.SliceData(src))),
		(*C.double)(unsafe.Pointer(unsafe.SliceData(dst))),
		C.size_t(len(src)))
}

// MaxVersion returns the highest version of function name in the linked library, 0 if the
// library does not provide it.
func MaxVersion(name string) uint32 {
	cs := C.CString(name)
	defer C.free(unsafe.Pointer(cs))
	return uint32(C.samefn_max_version(cs))
}

// BuildInfo describes the linked library, for start-up logs.
func BuildInfo() string { return C.GoString(C.samefn_build_info()) }
