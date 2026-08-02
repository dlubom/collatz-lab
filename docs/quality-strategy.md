# Quality Strategy

- **Status:** Accepted minimum for the MVP
- **Scope:** Mathematical core, two Rust engines, Lean model, and reproducible
  experiment records

This is a deliberately small correctness gate. It is not a claim that the MVP
is research-grade software, and it does not replace independent reproduction of
important mathematical observations.

## Minimal Quality Gate

The first complete reference engine is ready for review only after the following
commands have actually run successfully from the repository root, in this
order:

```bash
(cd lean && lake build)

cargo fmt --all -- --check

cargo clippy \
  --workspace \
  --all-targets \
  --all-features \
  -- -D warnings

cargo test --workspace

cargo llvm-cov \
  --workspace \
  --all-features
```

Once the reference module and mutation configuration exist, add:

```bash
cargo mutants --file crates/collatz-engine/src/reference.rs
```

Before the relevant PBI creates its toolchain and files, these commands are
intent, not passing evidence. A PBI closure report must list the literal
commands executed, exit status, and concise result. The statement “tests pass”
alone is not evidence of completion.

## Formatting

Every Rust change passes:

```bash
cargo fmt --all -- --check
```

Formatting is automatic and is not subject to reviewer taste.

## Static analysis and production-code rules

Every Rust change passes:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Each production crate root contains:

```rust
#![forbid(unsafe_code)]
```

Production domain code avoids `unwrap()`, `expect()`, `todo!()`,
`unimplemented!()`, uncontrolled `wrapping_*`, and implicit overflow
assumptions. A necessary exception requires a local explanatory comment and an
explicit PBI review justification. Broad `#[allow(...)]` annotations are not a
substitute for resolving a Clippy finding.

## Independent fixed examples

The reference engine uses fixed expectations that are not calculated by the
algorithm under test:

| Input | Classical steps to `1` | Trajectory peak |
|---:|---:|---:|
| 1 | 0 | 1 |
| 2 | 1 | 2 |
| 3 | 7 | 16 |
| 27 | 111 | 9232 |

Lean-checked small examples and the reviewed constants in
[`mathematical-definitions.md`](mathematical-definitions.md) provide separate
specification evidence. A test must not call the engine to generate its own
expected result.

Unit coverage separately checks:

- one classical even step and one classical odd step;
- terminal handling of input `1`;
- classical step counts and peak accounting;
- safe detection of a `u128` operation that cannot be represented;
- a small Mersenne generator;
- once implemented, compressed accounting and promotion to BigInt.

Every corrected defect leaves a regression test that fails without the fix.

## Coverage

The mathematical Collatz core has a minimum **90% line coverage** threshold.
The whole workspace publishes a coverage report but has no global hard
threshold in the MVP. The intended whole-workspace report is:

```bash
cargo llvm-cov --workspace --all-features
```

Once `collatz-engine` cleanly isolates the mathematical core, PBI-002 must make
this deterministic threshold command pass:

```bash
cargo llvm-cov \
  --package collatz-engine \
  --lib \
  --all-features \
  --fail-under-lines 90
```

If tool behavior shows that this command includes non-core adapters and cannot
enforce the intended boundary, implementation stops for a spec refinement;
the threshold is not silently weakened. Coverage proves execution, not
mathematical correctness, and tests without meaningful assertions do not count
as acceptable coverage improvement.

## Mutation testing

After the simple reference engine is complete, mutation testing is limited to:

```text
crates/collatz-engine/src/reference.rs
```

using:

```bash
cargo mutants --file crates/collatz-engine/src/reference.rs
```

No material mutant in the classical branch formula, terminal condition,
transition counter, overflow handling, or peak calculation may survive without
a recorded classification. Every survivor is classified as a test gap,
semantically equivalent mutant, unreachable code, tool limitation, or
explicitly deferred case with rationale. Test gaps are closed with meaningful
tests. Codex may not exclude mutations merely to make the command green, and
the MVP has no global mutation-score target.

## Lean gate

PBI-001 must make `(cd lean && lake build)` pass. Lean sources contain no
`sorry`; principal theorems do not depend on `sorryAx`; small examples are
checked by Lean; and review confirms that theorem statements match
[`mathematical-definitions.md`](mathematical-definitions.md).

Lean proves the mathematical model and stated local equivalences. It does not
prove the compiled Rust program or any external dependency.

## Engine and experiment verification

- The reference engine uses checked `u128` arithmetic and reports an
  unrepresentable transition without wrapping.
- The BigInt engine and hybrid promotion path are compared with the reference
  engine on their common domain.
- Every integration defect leaves a regression test.
- Key results are confirmed through at least two execution paths; a benchmark
  is never a correctness oracle.
- A result remains `needs-reproduction` until the procedure in
  [`experimental-methodology.md`](experimental-methodology.md) succeeds.

## PBI evidence and review

Every implementation PBI provides exact paths, deterministic commands, and the
expected result of each command. Closure requires:

1. the acceptance criteria checked against the living Spec;
2. the literal commands and observed outcomes recorded;
3. `git diff --check` and applicable quality gates passing;
4. living-spec updates included with observable behavior changes;
5. an independent review of contract, mathematics, tests, and surviving
   mutations in scope;
6. commits following the logical boundaries declared in the PBI.

## Deferred quality hardening

The following are consciously deferred and are not MVP completion gates:

- fuzzing, Miri, and sanitizers;
- fully executable Gherkin scenarios;
- an automated requirement-traceability matrix;
- mutation testing across the whole repository;
- multiple specialized agents;
- nightly and weekly pipelines;
- elaborate experiment manifests;
- automatically generated quality scorecards;
- automatic benchmark publication.

They may be proposed after the MVP exposes a concrete risk or maintenance need.
