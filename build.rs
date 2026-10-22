//! Compiles the vendored CORE-MATH sin (third_party/core-math/PIN) with fixed, portable
//! flags. Correct rounding makes the results independent of the compiler and its flags,
//! except fast math, which csrc/guard.c rejects. No -march: the archive must run on any CPU
//! of its target.

fn main() {
    let sin = "third_party/core-math/src/binary64/sin/sin.c";
    let guard = "csrc/guard.c";
    for f in [sin, guard, "third_party/core-math/PIN"] {
        println!("cargo:rerun-if-changed={f}");
    }
    cc::Build::new()
        .file(sin)
        .file(guard)
        // Version 1's code under its own name, so a later version can link next to it.
        .define("cr_sin", "samefn_core_math_sin_v1")
        .flag_if_supported("-std=c11")
        .flag_if_supported("-ffp-contract=off")
        .flag_if_supported("-fno-fast-math")
        .warnings(false)
        .compile("samefn_core_math");
}
