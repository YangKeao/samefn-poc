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
| `include/samefn.h` | C interface: `samefn_sin_v1`, `samefn_sin_batch_v1`, `samefn_max_version`, `samefn_build_info` |
| `go/samefn` | Go package (`SinV1`, `SinSliceV1`, `MaxVersion`, `BuildInfo`) over cgo, with prebuilt archives in `lib/<goos>_<goarch>/` |
| `third_party/core-math` | CORE-MATH's `sin.c`, vendored unmodified, pinned in `PIN` |
| `corpus/sin_v1.tsv` | Frozen outputs of `sin` version 1 for 10,000 inputs, as bit patterns |

How each engine uses it:

- **TiKV (Rust)** depends on this crate as a git dependency pinned to a commit and calls
  `samefn::sin::v1` directly. No foreign-function boundary.
- **TiFlash (C++)** links the static archive and calls `samefn_sin_batch_v1` once per column.
- **TiDB (Go)** imports `github.com/mjonss/samefn-poc/go/samefn`: `SinV1` for row evaluation,
  `SinSliceV1` once per chunk for vectorized evaluation. Without cgo the package compiles but
  panics if called; it never falls back to Go's `math.Sin`.

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

TiDB and TiFlash link the prebuilt archive. TiKV compiles the crate in its own build, with its
own C compiler but this crate's `build.rs` flags (the `cc` crate also passes on any x86
`-C target-feature` from TiKV's `RUSTFLAGS`). Correct rounding gives every input exactly one
right answer, so all these builds compute the same bits although their machine code differs;
tested with Apple clang and GCC 11.5.

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

## Build and test

```sh
cargo test --release              # Rust API against the corpus
./scripts/check-c.sh              # C interface against the corpus
./scripts/build-archive.sh        # archive for this host, into go/samefn/lib/<goos>_<goarch>/
go test ./go/samefn/              # Go package against the corpus
./scripts/crosscheck-mpfr.sh      # against MPFR (needs MPFR and GMP)
```

Committed archives: `darwin_arm64` (M3), `linux_amd64` (Rocky 9 on an i7-13700) and
`linux_arm64` (Rocky 9 in an arm64 Linux VM on the M3), all three tested there against the
corpus. `darwin_amd64` is missing. CI builds and tests Linux x86_64, Linux arm64 and macOS
arm64.

## Status

| Part | State |
| --- | --- |
| Shared library, `sin` v1 and its batch form | done |
| TiKV branch | not started |
| TiFlash branch | not started |
| TiDB branch | not started |
| Cluster run: TiDB, TiKV and TiFlash give the same bits | not started |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option. The vendored CORE-MATH file is MIT
([third_party/core-math/LICENSE](third_party/core-math/LICENSE)).
