# PBI-003: Add Arbitrary Precision and Safe Promotion

- **Status:** Planned; blocked by PBI-002
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

- Blocked by: PBI-002 and accepted ADR-003.
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

- [ ] The BigInt engine evaluates the same known trajectories and metrics as the
  reference engine.
- [ ] Common-domain generated inputs agree on values, counts, peak, first
  descent, and termination.
- [ ] The hybrid runner checks before overflow and promotes the current value,
  not a wrapped result.
- [ ] Promotion neither repeats nor omits a transition and occurs at most once.
- [ ] `u128::MAX` produces the exact BigInt odd result under BigInt/hybrid and a
  typed overflow without a counted step under the reference engine.
- [ ] Invalid zero input remains distinct from all finite run outcomes.
- [ ] The BigInt engine never narrows intermediate values to `u128`.
- [ ] Every integration defect leaves a regression test.
- [ ] The mathematical core remains at or above 90% line coverage.
- [ ] The reference mutation gate has no unexplained material survivor.
- [ ] Criterion benchmarks compile on the supported Apple Silicon target and
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
