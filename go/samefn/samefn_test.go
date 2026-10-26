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

func corpus(t *testing.T) (in, want []float64) {
	f, err := os.Open("../../corpus/sin_v1.tsv")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	s := bufio.NewScanner(f)
	for s.Scan() {
		line := s.Text()
		if strings.HasPrefix(line, "#") {
			continue
		}
		x, y, _ := strings.Cut(line, "\t")
		a, err1 := strconv.ParseUint(x, 16, 64)
		b, err2 := strconv.ParseUint(y, 16, 64)
		if err1 != nil || err2 != nil {
			t.Fatalf("bad corpus line %q", line)
		}
		in = append(in, math.Float64frombits(a))
		want = append(want, math.Float64frombits(b))
	}
	return in, want
}

func same(got, want float64) bool {
	return (math.IsNaN(got) && math.IsNaN(want)) || math.Float64bits(got) == math.Float64bits(want)
}

func TestSinV1Corpus(t *testing.T) {
	in, want := corpus(t)
	if len(in) < 10000 {
		t.Fatalf("corpus has %d entries", len(in))
	}
	for i, x := range in {
		if got := SinV1(x); !same(got, want[i]) {
			t.Fatalf("SinV1(%x) = %x, frozen %x", x, got, want[i])
		}
	}
	out := make([]float64, len(in))
	SinSliceV1(out, in)
	inPlace := append([]float64(nil), in...)
	SinSliceV1(inPlace, inPlace)
	for i := range in {
		if !same(out[i], want[i]) || !same(inPlace[i], want[i]) {
			t.Fatalf("SinSliceV1 entry %d differs from the corpus", i)
		}
	}
}

func TestVersions(t *testing.T) {
	if v := MaxVersion("sin"); v != 1 {
		t.Fatalf("MaxVersion(sin) = %d", v)
	}
	if v := MaxVersion("cos"); v != 1 {
		t.Fatalf("MaxVersion(cos) = %d", v)
	}
	t.Log(BuildInfo())
}
