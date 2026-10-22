//go:build !cgo

package samefn

// Built without cgo (CGO_ENABLED=0): no implementation is linked. These fail closed rather
// than fall back to Go's math package, whose results are not the same on every CPU.

const errNoCgo = "samefn: built without cgo; this binary must not evaluate samefn functions"

func SinV1(x float64) float64       { panic(errNoCgo) }
func SinSliceV1(dst, src []float64) { panic(errNoCgo) }
func MaxVersion(name string) uint32 { return 0 }
func BuildInfo() string             { return "samefn: built without cgo" }
