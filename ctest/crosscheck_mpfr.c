/* Checks samefn_sin_v1 against MPFR (correctly rounded to nearest, with double subnormals)
 * on the inputs read from argv[1] (hex floats, '#' comments) and on N seeded random
 * inputs. Prints the count of differences; exit status 1 if any. */
#include <inttypes.h>
#include <math.h>
#include <mpfr.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "samefn.h"

static mpfr_t m;

static double ref_sin(double x) {
  mpfr_set_d(m, x, MPFR_RNDN);
  int inex = mpfr_sin(m, m, MPFR_RNDN);
  mpfr_subnormalize(m, inex, MPFR_RNDN);
  return mpfr_get_d(m, MPFR_RNDN);
}

static uint64_t bits(double d) { uint64_t u; memcpy(&u, &d, 8); return u; }
static double from_bits(uint64_t u) { double d; memcpy(&d, &u, 8); return d; }

static long checked, diffs;
static void check(double x) {
  double got = samefn_sin_v1(x), want = ref_sin(x);
  checked++;
  if (isnan(got) && isnan(want)) return;
  if (bits(got) != bits(want)) {
    if (diffs++ < 20)
      printf("DIFF x=%a got=%a want=%a\n", x, got, want);
  }
}

static uint64_t s = 0x9e3779b97f4a7c15u;
static uint64_t next(void) { s ^= s >> 12; s ^= s << 25; s ^= s >> 27; return s * 0x2545f4914f6cdd1du; }

int main(int argc, char **argv) {
  mpfr_set_emin(-1073);
  mpfr_set_emax(1024);
  mpfr_init2(m, 53);
  long n_random = argc > 2 ? atol(argv[2]) : 0;
  if (argc > 1 && strcmp(argv[1], "-") != 0) {
    FILE *f = fopen(argv[1], "r");
    if (!f) { perror(argv[1]); return 2; }
    char line[256];
    while (fgets(line, sizeof line, f)) {
      if (line[0] == '#' || line[0] == '\n') continue;
      double x = strtod(line, NULL);
      check(x); check(-x);
    }
    fclose(f);
  }
  long hard = checked;
  for (long i = 0; i < n_random; i++) {
    uint64_t r = next();
    double x;
    switch (i & 3) {
    case 0: case 1: x = from_bits(r); if (isnan(x)) continue; break;      /* any finite or inf */
    case 2: x = ((double)(r >> 11) * 0x1p-53 * 2 - 1) * 10; break;      /* [-10, 10) */
    default: x = ((double)(r >> 11) * 0x1p-53 * 2 - 1) * 1e6; break;   /* [-1e6, 1e6) */
    }
    check(x);
  }
  printf("checked %ld (hard cases incl. negated: %ld, random: %ld), differences: %ld\n",
         checked, hard, checked - hard, diffs);
  return diffs ? 1 : 0;
}
