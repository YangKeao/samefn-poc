//! samefn PoC: one correctly rounded `sin`, shared by TiDB, TiKV and TiFlash, so the
//! function returns the same bits on every engine and every CPU.
//!
//! Versioning: each version of a function is its own entry point and never changes. A
//! result may change only by adding a new version next to the old one; `corpus/` freezes
//! every version's outputs and the tests fail if one moves. Version 0 is each engine's own
//! pre-existing implementation and is not provided here. Callers choose the version
//! explicitly; there is no "latest" alias, so upgrading the library cannot change a result.

use std::ffi::{c_char, CStr};

extern "C" {
    fn samefn_core_math_sin_v1(x: f64) -> f64;
}

/// Description of this build, for start-up logs.
pub const BUILD_INFO: &str = concat!(
    "samefn-poc ",
    env!("CARGO_PKG_VERSION"),
    "; sin v1 = CORE-MATH 0ce0cbc15935 src/binary64/sin/sin.c"
);

pub mod sin {
    /// Highest version of `sin` in this build.
    pub const MAX_VERSION: u32 = 1;

    /// `sin` version 1: correctly rounded (CORE-MATH at the pinned commit).
    #[inline]
    pub fn v1(x: f64) -> f64 {
        // SAFETY: a pure function of its argument.
        unsafe { super::samefn_core_math_sin_v1(x) }
    }

    /// `output[i] = v1(input[i])` for every `i < input.len()`.
    pub fn batch_v1(input: &[f64], output: &mut [f64]) {
        assert!(output.len() >= input.len(), "samefn: output shorter than input");
        for (o, &x) in output.iter_mut().zip(input) {
            *o = v1(x);
        }
    }

    /// `values[i] = v1(values[i])`.
    pub fn batch_v1_in_place(values: &mut [f64]) {
        for v in values {
            *v = v1(*v);
        }
    }
}

/// Highest version of function `name` in this build, 0 if it is not provided.
pub fn max_version(name: &str) -> u32 {
    match name {
        "sin" => sin::MAX_VERSION,
        _ => 0,
    }
}

// C ABI, see include/samefn.h.

#[no_mangle]
pub extern "C" fn samefn_sin_v1(x: f64) -> f64 {
    sin::v1(x)
}

/// # Safety
/// `input` and `output` must be valid for `n` elements; they may be the same pointer.
#[no_mangle]
pub unsafe extern "C" fn samefn_sin_batch_v1(input: *const f64, output: *mut f64, n: usize) {
    for i in 0..n {
        // Read before write, so input == output works.
        let x = *input.add(i);
        *output.add(i) = sin::v1(x);
    }
}

/// # Safety
/// `name` must be a valid NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn samefn_max_version(name: *const c_char) -> u32 {
    if name.is_null() {
        return 0;
    }
    CStr::from_ptr(name).to_str().map_or(0, max_version)
}

#[no_mangle]
pub extern "C" fn samefn_build_info() -> *const c_char {
    concat!(
        "samefn-poc ",
        env!("CARGO_PKG_VERSION"),
        "; sin v1 = CORE-MATH 0ce0cbc15935 src/binary64/sin/sin.c\0"
    )
    .as_ptr()
    .cast()
}
