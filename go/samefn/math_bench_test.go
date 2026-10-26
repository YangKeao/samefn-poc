//go:build cgo

package samefn

import (
	"fmt"
	"math"
	"runtime"
	"testing"
)

var benchmarkSink float64

// Preallocated stable, column-dependent inputs; no RNG or allocations in timed loops.
func BenchmarkSharedMath(b *testing.B) {
	for _, op := range []struct {
		name           string
		native, scalar func(float64) float64
		batch          func([]float64, []float64)
	}{
		{"identity", func(x float64) float64 { return x }, IdentityV1, IdentitySliceV1},
		{"sin", math.Sin, SinV1, SinSliceV1}, {"cos", math.Cos, CosV1, CosSliceV1},
		{"tan", math.Tan, TanV1, TanSliceV1}, {"exp", math.Exp, ExpV1, ExpSliceV1},
	} {
		for _, n := range []int{1, 32, 1024} {
			for _, mode := range []string{"native", "scalar", "batch"} {
				b.Run(fmt.Sprintf("%s/n=%d/%s", op.name, n, mode), func(b *testing.B) {
					src := make([]float64, n)
					dst := make([]float64, n)
					for i := range src {
						src[i] = float64((i*15427+431)%200001)/10000 - 10
					}
					before := runtime.NumCgoCall()
					b.ReportAllocs()
					b.ResetTimer()
					switch mode {
					case "native":
						for k := 0; k < b.N; k++ {
							for i, x := range src {
								dst[i] = op.native(x)
							}
						}
					case "scalar":
						for k := 0; k < b.N; k++ {
							for i, x := range src {
								dst[i] = op.scalar(x)
							}
						}
					default:
						for k := 0; k < b.N; k++ {
							op.batch(dst, src)
						}
					}
					b.StopTimer()
					b.ReportMetric(float64(runtime.NumCgoCall()-before)/float64(b.N), "cgo/op")
					benchmarkSink = dst[n-1]
				})
			}
		}
	}
}
