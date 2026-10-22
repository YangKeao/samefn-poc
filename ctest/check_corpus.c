/* Checks the C interface (as TiFlash calls it) against the frozen corpus: the scalar
 * function and the in-place batch. Usage: check_corpus corpus/sin_v1.tsv */
#include <inttypes.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "samefn.h"

static uint64_t bits(double d) { uint64_t u; memcpy(&u, &d, 8); return u; }
static int same(double got, double want) { return (isnan(got) && isnan(want)) || bits(got) == bits(want); }

int main(int argc, char **argv) {
  FILE *f = fopen(argc > 1 ? argv[1] : "corpus/sin_v1.tsv", "r");
  if (!f) { perror("corpus"); return 2; }
  size_t cap = 16384, n = 0;
  double *x = malloc(cap * sizeof *x), *want = malloc(cap * sizeof *want);
  char line[128];
  while (fgets(line, sizeof line, f)) {
    if (line[0] == '#') continue;
    uint64_t a, b;
    if (sscanf(line, "%" SCNx64 "\t%" SCNx64, &a, &b) != 2) continue;
    if (n == cap) { cap *= 2; x = realloc(x, cap * sizeof *x); want = realloc(want, cap * sizeof *want); }
    memcpy(&x[n], &a, 8); memcpy(&want[n], &b, 8); n++;
  }
  fclose(f);
  long bad = 0;
  for (size_t i = 0; i < n; i++) bad += !same(samefn_sin_v1(x[i]), want[i]);
  samefn_sin_batch_v1(x, x, n); /* in place */
  for (size_t i = 0; i < n; i++) bad += !same(x[i], want[i]);
  printf("%s\nsin v1: %zu corpus entries, max_version(sin)=%u, mismatches: %ld\n",
         samefn_build_info(), n, samefn_max_version("sin"), bad);
  return bad ? 1 : 0;
}
