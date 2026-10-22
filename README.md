# samefn-poc

A minimal proof of concept for [pingcap/tidb#72218](https://github.com/pingcap/tidb/pull/72218)
"docs: design for deterministic, correctly rounded math and regex across engines": one
correctly rounded, versioned `sin`, linked into TiDB, TiKV and TiFlash, so `SIN(x)` returns the
same bits whichever engine evaluates it and on whichever CPU. Tracking issue:
[pingcap/tidb#71807](https://github.com/pingcap/tidb/issues/71807).

## What is in it

| Path | What |
| --- | --- |
| `src/lib.rs` | Rust API (`samefn::sin::v1`, `batch_v1`, `batch_v1_in_place`) and the C ABI |
| `include/samefn.h` | C interface: `samefn_sin_v1`, `samefn_sin_batch_v1`, `samefn_max_version`, `samefn_build_info`; a copy sits next to the Go package |
| `go/samefn` | Go package (`SinV1`, `SinSliceV1`, `MaxVersion`, `BuildInfo`) over cgo |
| `go/samefn/internal/obj` | One prebuilt object per platform, `samefn_<goos>_<goarch>.syso`, imported only by the cgo side |
| `go/samefn/lib/<goos>_<goarch>` | The static archives, for C and C++ users |
| `third_party/core-math` | CORE-MATH's `sin.c`, vendored unmodified, pinned in `PIN` |
| `corpus/sin_v1.tsv` | Frozen outputs of `sin` version 1 for 10,000 inputs, as bit patterns |

How each engine uses it:

- **TiKV (Rust)** depends on this crate as a git dependency pinned to a commit and calls
  `samefn::sin::v1` directly. No foreign-function boundary.
- **TiFlash (C++)** links the static archive and calls `samefn_sin_batch_v1` once per column.
- **TiDB (Go)** imports `github.com/mjonss/samefn-poc/go/samefn`: `SinV1` for row evaluation,
  `SinSliceV1` once per chunk for vectorized evaluation. The code is a prebuilt object per
  platform, `internal/obj/samefn_<goos>_<goarch>.syso`, which `go build` and Bazel's rules_go
  both link by the platform in its name, with no link flag pointing at a path. Two details
  found in TiDB's CI: Go links a package's `.syso` files even without cgo, so only the cgo side
  imports `internal/obj`; and Gazelle does not list `.syso` files, so `internal/obj/BUILD.bazel`
  keeps them. Without cgo the package compiles but panics if called; it never falls back to
  Go's `math.Sin`.

## Versioning

- Each version of a function is its own entry point (`samefn_sin_v1`) and never changes. A
  result may change only by adding a new version next to the old one, for example
  `samefn_sin_v2`; both stay available.
- Callers choose the version explicitly. There is no "latest" alias, so upgrading the library
  cannot change a result.
- `samefn_max_version("sin")` reports the highest version a build provides, so an engine can
  say what it supports.
- Version 0 is each engine's own, pre-existing implementation and is not provided here.
- Out of scope here: choosing a version in the engines. In this PoC the engines call
  version 1 for `SIN` directly. Carrying the version from TiDB to TiKV and TiFlash needs a
  change to tipb, and is independent of how the library is built and linked.
- `corpus/sin_v1.tsv` freezes version 1. The tests in Rust, C and Go fail if any output moves,
  and CI checks the corpus file against `corpus/SHA256SUMS`.

## The pinned CORE-MATH

`sin` version 1 is CORE-MATH's `src/binary64/sin/sin.c` at commit
[`0ce0cbc159`](https://gitlab.inria.fr/core-math/core-math/-/commit/0ce0cbc15935e24c656280e5a46c34e89f301b7a)
(2026-10-20; the file last changed in `a6509c5`, 2026-10-08), sha256 `cd772a06...b1c2`.
`scripts/vendor-core-math.sh` fetches that commit and verifies the vendored file byte for byte.

### How it is compiled

`build.rs` compiles the file once per platform (operating system and architecture), not per CPU
model; nothing is recompiled at run time.

- No `-march`: only the architecture's baseline instructions are used, so one archive runs on
  every CPU of its platform. The x86_64 baseline has no FMA instruction, so the four explicit
  `__builtin_fma` calls in `sin.c` become calls to an `fma()` function. In the archive that
  is the copy in Rust's runtime (`compiler_builtins`), which uses the CPU's FMA instruction
  where there is one and an exact software version otherwise; both give the same bits. On
  arm64, FMA is part of the baseline and inlined.
- `-ffp-contract=off`: the compiler does not fuse a multiply and an add that the source writes
  separately, so the machine code performs exactly the operations in the source.
- `csrc/guard.c` fails the build under fast math, which may drop rounding steps.

TiDB and TiFlash link prebuilt code, TiDB the object and TiFlash the archive. TiKV compiles the
crate in its own build, with its own C compiler but this crate's `build.rs` flags (the `cc`
crate also passes on any x86 `-C target-feature` from TiKV's `RUSTFLAGS`). Correct rounding
gives every input exactly one right answer, so all these builds compute the same bits although
their machine code differs; tested with Apple clang and GCC 11.5.

Faster code for newer CPUs, such as `-march=x86-64-v3` with FMA and AVX2 inline, is a flag
change in `build.rs` with the same results, but that build would not run on older x86 CPUs.
For this `sin` the gain is small: on an i7-13700, for inputs in [-10, 10], 8.6 ns per call
with `-march=x86-64-v3` against 9.0 ns at the baseline. Building both variants and choosing
one at run time keeps it portable; the PoC does not.

## Verification

- **MPFR:** 0 differences on 13,949,476 inputs: all 1,975,927 upstream hard-to-round cases and
  their negations, plus 10,000,000 seeded random inputs (`scripts/crosscheck-mpfr.sh`).
- **An independent correctly rounded `sin`** (LLVM libc, from the earlier PoC kernel): 0
  differences on the corpus.
- **Corpus, through all three interfaces** (Rust, C scalar and in-place batch, Go scalar and
  slice): 0 differences on macOS arm64 (Apple clang) and Linux x86_64 (GCC 11.5).

## Cluster results

On lellen (Intel i7-13700, Rocky Linux 9.8): a tiup playground with one TiDB, PD, TiKV and
TiFlash, and 100,000 rows of generated doubles: `x` uniform in [-10, 10], `xl` = ±10^u with u
uniform in [-1, 12], `p` = 10^u with u uniform in [-3, 3]. Each expression is evaluated by TiDB,
pushed down to TiKV, and pushed down to TiFlash; EXPLAIN confirms which engine evaluated it,
and the results are compared as exact bits. The harness comes from an earlier, private PoC and
is not in this repository yet.

- **Before:** stock nightly TiDB, TiKV, PD and TiFlash of 2026-10-07.
- **After:** [tidb-samefn-poc](https://github.com/mjonss/tidb/tree/tidb-samefn-poc),
  [tikv-samefn-poc](https://github.com/mjonss/tikv/tree/tikv-samefn-poc) and
  [tiflash-samefn-poc](https://github.com/mjonss/tiflash/tree/tiflash-samefn-poc), with the
  same nightly PD.
- Between that nightly and the commits the branches start from, no engine's SIN code changed,
  so the difference below is samefn alone.

Rows that differ, of 100,000:

| Expression | Engines | Before | After |
| --- | --- | --- | --- |
| `SIN(x)` | TiDB vs TiKV | 24,695 (1 ULP) | **0** |
| `SIN(x)` | TiDB vs TiFlash, TiKV vs TiFlash | not pushed down | **0** |
| `SIN(xl)` | TiDB vs TiKV | 28,673 (up to 1,347 ULP) | **0** |
| `SIN(xl)` | TiDB vs TiFlash, TiKV vs TiFlash | not pushed down | **0** |
| `COS(x)`, not on samefn | TiDB vs TiKV | 26,890 (1 ULP) | 26,890 (1 ULP) |
| `POW(p, x)`, not on samefn | TiDB vs TiKV, TiDB vs TiFlash | 61,802 (8 ULP) | 61,802 (8 ULP) |

- TiDB now pushes SIN down to TiFlash (`mpp[tiflash]`), which it did not before.
- TiDB's own SIN results changed on 24,683 rows: Go's `math.Sin` is not correctly rounded.
  TiKV's changed on 136 rows (`SIN(xl)`: 173): Rust's `f64::sin` calls the platform libm
  (glibc 2.34 here), which is almost always correctly rounded. A stock TiFlash, which also
  calls glibc, differed from samefn on the same 136 and 173 rows.
- `COS` and `POW` are controls: not on samefn, their differences are unchanged.

## Build and test

```sh
cargo test --release              # Rust API against the corpus
./scripts/check-c.sh              # C interface against the corpus
./scripts/build-archive.sh        # archive and Go object for this host (go/samefn/lib/, go/samefn/internal/obj/)
go test ./go/samefn/              # Go package against the corpus
./scripts/crosscheck-mpfr.sh      # against MPFR (needs MPFR and GMP)
```

Committed archives, each with the Go object made from it (`scripts/build-go-object.sh`):
`darwin_arm64` (M3), `linux_amd64` (Rocky 9 on an i7-13700) and `linux_arm64` (Rocky 9 in an
arm64 Linux VM on the M3), all three tested there against the corpus. `darwin_amd64` is missing;
the Go package fails to compile there with a clear error. CI builds and tests Linux x86_64,
Linux arm64 and macOS arm64.

## Status

| Part | State |
| --- | --- |
| Shared library, `sin` v1 and its batch form | done |
| TiKV branch | done: [tikv-samefn-poc](https://github.com/mjonss/tikv/tree/tikv-samefn-poc); CI trial in [tikv/tikv#20215](https://github.com/tikv/tikv/pull/20215) (do not merge) |
| TiFlash branch | done: [tiflash-samefn-poc](https://github.com/mjonss/tiflash/tree/tiflash-samefn-poc); CI trial in [pingcap/tiflash#11144](https://github.com/pingcap/tiflash/pull/11144) (do not merge) |
| TiDB branch | done: [tidb-samefn-poc](https://github.com/mjonss/tidb/tree/tidb-samefn-poc), also pushes SIN down to TiFlash; CI trial in [pingcap/tidb#72219](https://github.com/pingcap/tidb/pull/72219) (do not merge) |
| Cluster run: TiDB, TiKV and TiFlash give the same bits | done: 0 differences (above) |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option. The vendored CORE-MATH file is MIT
([third_party/core-math/LICENSE](third_party/core-math/LICENSE)).
