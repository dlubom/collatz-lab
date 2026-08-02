# PBI-003: Add Arbitrary Precision and Safe Promotion

- **Status:** Implemented; ready for review
- **Type:** Exact engine, hybrid execution, and differential verification

## Goal

Add a `rug::Integer` engine and a hybrid runner that promotes from `u128`
before overflow while preserving every classical value, counter, peak, first-
descent, and termination rule.

## Motivation

Selected inputs and intermediate values exceed 128 bits. Exact arithmetic is
required, but it must be introduced without weakening the simple bounded
reference or obscuring the promotion boundary.

## Dependencies

- Depends on: completed PBI-002 and accepted ADR-003.
- Must merge before: PBI-004.

## Context pointers

Read:

1. [`docs/adrs/ADR-003-arbitrary-precision-with-rug-and-gmp.md`](../docs/adrs/ADR-003-arbitrary-precision-with-rug-and-gmp.md)
2. [`specs/collatz-engine/spec.md`](../specs/collatz-engine/spec.md), Engine
   contract and correctness scenarios
3. [`docs/mathematical-definitions.md`](../docs/mathematical-definitions.md),
   sections 4, 8, and 10
4. [`ARCHITECTURE.md`](../ARCHITECTURE.md), engine responsibilities and
   dependency direction
5. [`docs/quality-strategy.md`](../docs/quality-strategy.md)

## Scope

- Add the reviewed, pinned `rug` dependency.
- Implement a standalone exact BigInt step and finite runner with the same
  public semantics as the bounded engine where outcomes overlap.
- Implement a hybrid numeric state that promotes the current value before an
  unrepresentable odd operation and never demotes.
- Record a promotion count of zero or one.
- Test known inputs `1`, `2`, `3`, and `27` through BigInt.
- Add common-domain generated differential tests.
- Add a boundary test comparing hybrid promotion with BigInt-from-start.
- Add small and medium special-number cases without external downloads.
- Add baseline Criterion targets for reference, BigInt, and hybrid execution;
  compile them and run only short declared local measurements.
- Preserve the reference module's coverage and mutation gate.

## Out of scope

- A custom BigInt, alternate large-integer library, or direct unsafe GMP FFI.
- Compression or multi-iteration optimization.
- Catalogs, controls, JSONL, CLI, or claims about experimental performance.
- New platforms, multithreading, SIMD, or GPU.

## Concrete files

```text
crates/collatz-engine/Cargo.toml
crates/collatz-engine/src/lib.rs
crates/collatz-engine/src/domain.rs
crates/collatz-engine/src/bigint.rs
crates/collatz-engine/src/hybrid.rs
crates/collatz-engine/tests/bigint_engine.rs
crates/collatz-engine/tests/differential.rs
crates/collatz-engine/benches/engines.rs
Cargo.lock
```

Reference-engine production logic changes only if a PBI-002 defect is found;
such a change requires a regression test and living-Spec review.

## Small tasks

1. Add and lock `rug`; verify the GMP-backed build on Apple Silicon.
2. Implement exact one-step and finite BigInt execution.
3. Reuse the public semantic contract without calling the reference engine.
4. Implement checked preflight and one-way promotion in the hybrid runner.
5. Add fixed known-value and invalid-input tests.
6. Add common-domain property/differential tests, including limits and peaks.
7. Test `u128::MAX`: reference reports overflow, hybrid produces exact
   `3 * u128::MAX + 1` with one step and one promotion, BigInt agrees.
8. Add basic declared benchmark cases and compile the harness.
9. Re-run coverage and reference mutation gates and add regression tests for
   every integration issue.

## Acceptance criteria

- [x] The BigInt engine evaluates the same known trajectories and metrics as the
  reference engine.
- [x] Common-domain generated inputs agree on values, counts, peak, first
  descent, and termination.
- [x] The hybrid runner checks before overflow and promotes the current value,
  not a wrapped result.
- [x] Promotion neither repeats nor omits a transition and occurs at most once.
- [x] `u128::MAX` produces the exact BigInt odd result under BigInt/hybrid and a
  typed overflow without a counted step under the reference engine.
- [x] Invalid zero input remains distinct from all finite run outcomes.
- [x] The BigInt engine never narrows intermediate values to `u128`.
- [x] Every integration defect leaves a regression test.
- [x] The mathematical core remains at or above 90% line coverage.
- [x] The reference mutation gate has no unexplained material survivor.
- [x] Criterion benchmarks compile on the supported Apple Silicon target and
  are not interpreted as correctness evidence.

## Deterministic verification commands

```bash
(cd lean && lake build)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --all-features
cargo llvm-cov --package collatz-engine --lib --all-features --fail-under-lines 90
cargo mutants --file crates/collatz-engine/src/reference.rs
cargo bench --workspace --no-run
cargo bench --workspace
git diff --check
```

Expected results:

- All builds, lint, tests, threshold coverage, benchmark compilation/execution,
  and diff checks exit `0`; workspace coverage and mutation analysis report
  evidence.
- Differential tests include representable prefixes and the promotion boundary.
- Coverage remains at least `90.00%` for the package/lib core.
- Mutation survivors, if equivalent or tool-limited, are explicitly classified
  and independently accepted; a resulting nonzero mutation-tool exit is
  reported honestly rather than called a passing command.
- Criterion emits measurements for reference, BigInt, and promotion cases on
  the documented Apple Silicon host; no benchmark number is a completion
  threshold or correctness criterion.

## Closure evidence

Observed on 2026-08-02 from the PBI topic branch on macOS 26.6 / Apple M4,
using Rust and Cargo 1.97.1, `rug` 1.30.0, `gmp-mpfr-sys` 1.7.1 with GMP 6.3.0,
Criterion 0.8.2, `cargo-llvm-cov` 0.8.7, and `cargo-mutants` 27.1.0:

- `cd lean && lake build` — exit `0`; all 1,049 Lean jobs completed.
- `/Users/dariuszlubomski/.cargo/bin/cargo fmt --all -- --check` — exit `0`;
  no formatting changes required.
- `/Users/dariuszlubomski/.cargo/bin/cargo clippy --workspace --all-targets
  --all-features -- -D warnings` — exit `0`; no warnings.
- `/Users/dariuszlubomski/.cargo/bin/cargo test --workspace` — exit `0`; 8
  library unit tests, 6 BigInt integration tests, 4 differential tests, 4
  reference property tests, and 17 reference integration tests passed; no
  failures or ignored tests.
- `/Users/dariuszlubomski/.cargo/bin/cargo llvm-cov --workspace
  --all-features` — exit `0`; reported `99.34%` workspace line coverage
  (`454/457` lines), with no global threshold applied.
- `/Users/dariuszlubomski/.cargo/bin/cargo llvm-cov --package collatz-engine
  --lib --all-features --fail-under-lines 90` — exit `0`; reported `98.47%`
  line coverage (`450/457` lines) for the package/lib target.
- `/Users/dariuszlubomski/.cargo/bin/cargo mutants --file
  crates/collatz-engine/src/reference.rs` — exit `0`; 15 mutants tested: 12
  caught, 3 compile-unviable function-return replacements, and 0 missed.
- `/Users/dariuszlubomski/.cargo/bin/cargo bench --workspace --no-run` — exit
  `0`; the library and `engines` Criterion targets compiled in the optimized
  bench profile.
- `/Users/dariuszlubomski/.cargo/bin/cargo bench --workspace` — exit `0`; five
  short declared reference, BigInt, hybrid, promotion, and special-form cases
  emitted Criterion measurements. Context and intervals are recorded in
  [`docs/benchmarking.md`](../docs/benchmarking.md) without treating timing as
  correctness evidence.
- `git diff --check` — exit `0`; no whitespace errors.

The first package/lib coverage attempt exposed that the new modules were only
reached through integration tests, while the threshold command runs library
unit tests. Targeted unit regressions now exercise the same private branch and
representation invariants, and the unchanged public integration tests retain
independent end-to-end coverage.

## Implementation refinements

- `PositiveInteger` enforces the positive arbitrary-precision input domain and
  exposes borrowed or owned `rug::Integer` values without narrowing.
- BigInt summaries preserve exact start, last, and observed-peak values while
  reusing the shared termination vocabulary and metric names.
- Hybrid summaries retain the active `u128` or BigInt representation explicitly
  and record a `u8` promotion count constrained by execution to zero or one.
- Criterion 0.8.2 writes raw measurements below crate-local build directories;
  those generated artifacts are ignored and remain outside version control.

## Risks

- Conversion may occur after a failed operation or alter step order.
- Shared types may accidentally make the exact engine call the reference path,
  weakening differential independence.
- `rug` ownership shortcuts may tempt narrowing or hidden allocation behavior.
- A platform-specific GMP build issue may appear.
- Tests may cover promotion without checking counters and peaks.

## Completion conditions

Both exact and hybrid paths pass the full gate, the reference path remains
simple and independently testable, promotion evidence is reviewed, and no
experiment/reporting layer is added.

## Independent review

The reviewer follows the `u128::MAX` transition manually, inspects the order of
preflight/conversion/arithmetic/counter update, confirms the BigInt engine is
algorithmically independent, checks dependency/build consequences against
ADR-003, and reviews all new differential assertions.

## Logical commit boundaries

1. `build(engine): add pinned rug dependency`
2. `feat(engine): add exact BigInt execution`
3. `feat(engine): add pre-overflow hybrid promotion`
4. `test(engine): add differential and boundary coverage`
5. `bench(engine): add scalar baseline harness`
6. `docs: record verified implementation refinements` only when required

## Refinement protocol

Idiomatic ownership and internal numeric abstractions may evolve. Stop for human
review before changing the promotion policy, standalone reference outcome,
public metrics, dependency choice, supported platform, or formal claim. Update
the Spec and, for a changed architectural choice, supersede the ADR.
