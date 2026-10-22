/* CORE-MATH is correctly rounded under any optimization level and floating-point
 * contraction, but not under fast math, which lets the compiler reassociate and drop
 * rounding steps. Fail the build if CFLAGS bring it in. */
#ifdef __FAST_MATH__
#error "samefn: CORE-MATH must not be built with -ffast-math"
#endif
#if defined(__FINITE_MATH_ONLY__) && __FINITE_MATH_ONLY__
#error "samefn: CORE-MATH must not be built with -ffinite-math-only"
#endif
typedef int samefn_guard_no_symbol;
