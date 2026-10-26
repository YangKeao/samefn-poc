//go:build cgo

package samefn

import (
	"bufio"
	"math"
	"os"
	"strconv"
	"strings"
	"testing"
)

func TestAdditionalMathMPFR(t *testing.T) {
	for _, name := range []string{"cos", "tan", "exp"} {
		f, e := os.Open("../../corpus/" + name + "_v1.tsv")
		if e != nil {
			t.Fatal(e)
		}
		var in, want []float64
		s := bufio.NewScanner(f)
		for s.Scan() {
			if strings.HasPrefix(s.Text(), "#") {
				continue
			}
			a, b, _ := strings.Cut(s.Text(), "\t")
			x, e := strconv.ParseUint(a, 16, 64)
			if e != nil {
				t.Fatal(e)
			}
			y, e := strconv.ParseUint(b, 16, 64)
			if e != nil {
				t.Fatal(e)
			}
			in = append(in, math.Float64frombits(x))
			want = append(want, math.Float64frombits(y))
		}
		if e := s.Err(); e != nil {
			t.Fatal(e)
		}
		f.Close()
		scalar := map[string]func(float64) float64{"cos": CosV1, "tan": TanV1, "exp": ExpV1}[name]
		batch := map[string]func([]float64, []float64){"cos": CosSliceV1, "tan": TanSliceV1, "exp": ExpSliceV1}[name]
		out := make([]float64, len(in))
		batch(out, in)
		inplace := append([]float64(nil), in...)
		batch(inplace, inplace)
		for i, x := range in {
			for _, got := range []float64{scalar(x), out[i], inplace[i]} {
				if !(math.IsNaN(got) && math.IsNaN(want[i])) && math.Float64bits(got) != math.Float64bits(want[i]) {
					t.Fatalf("%s input %x got %x want %x", name, math.Float64bits(x), math.Float64bits(got), math.Float64bits(want[i]))
				}
			}
		}
		batch(nil, nil)
	}
}
