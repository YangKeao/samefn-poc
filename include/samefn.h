/* samefn PoC: one correctly rounded, versioned sin() shared by TiDB, TiKV and TiFlash.
 *
 * Each version of a function is its own entry point and never changes; a result changes
 * only through a new version. Callers pick the version explicitly. */
#ifndef SAMEFN_H
#define SAMEFN_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* sin version 1: correctly rounded (CORE-MATH, pinned commit). */
double samefn_sin_v1(double x);

/* output[i] = samefn_sin_v1(input[i]) for i < n. input and output may be the same. */
void samefn_sin_batch_v1(const double *input, double *output, size_t n);

double samefn_cos_v1(double x);
void samefn_cos_batch_v1(const double *input, double *output, size_t n);
double samefn_tan_v1(double x);
void samefn_tan_batch_v1(const double *input, double *output, size_t n);
double samefn_exp_v1(double x);
void samefn_exp_batch_v1(const double *input, double *output, size_t n);
double samefn_identity_v1(double x);
void samefn_identity_batch_v1(const double *input, double *output, size_t n);

/* Highest version of function `name` ("sin") in this build; 0 if not provided. */
uint32_t samefn_max_version(const char *name);

/* Static description of this build, for start-up logs. */
const char *samefn_build_info(void);

#ifdef __cplusplus
}
#endif

#endif /* SAMEFN_H */
