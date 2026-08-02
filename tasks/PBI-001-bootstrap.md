# PBI-001: Bootstrap the correctness-first workspace

- **Status:** Ready
- **Type:** Repository bootstrap; no production Collatz behavior

## Directive

Bootstrap the Rust and Lean4 workspace, deterministic validation entry points,
and empty test/benchmark structure needed for later engine implementation.
Before editing, present a file-level execution plan for approval. This PBI must
leave a buildable scaffold and must not implement the Collatz transition or run
algorithm.

This directive follows the local ASDLC
[`PBI Authoring` practice](../.asdlc/practices/pbi-authoring.md).

**Scope:**

- Add the root Rust workspace and pinned Rust toolchain configuration.
- Add a minimal `crates/collatz-engine/` library scaffold with module and public
  API placeholders sufficient to compile, but no transition arithmetic.
- Declare `rug` for the future GMP-backed engine and separate development
  dependencies for property testing and Criterion benchmarking.
- Add a pinned Lean4/Lake project under `proofs/` with a buildable namespace
  skeleton and no unproved theorem placeholders.
- Add compile-only smoke tests and a benchmark target that prove the harnesses
  are wired correctly without claiming feature behavior.
- Add CI for macOS Apple Silicon that runs formatting, static analysis, Rust
  tests, the Lean build, and benchmark compilation in correctness-first order.
- Update planning documentation in the same change if bootstrap discoveries
  alter paths, commands, or constraints.

**Out of scope:**

- The Collatz step function, bounded trajectory runner, or result computation.
- SIMD, GPU, parallel, distributed, CLI, persistence, visualization, or network
  functionality.
- Performance optimization or benchmark conclusions.
- Any claim about universal Collatz termination.

## Dependencies

- Blocked by: None.
- Must merge before: the first PBI that implements the Collatz engine contract.

## Context

Read, in order:

1. [`AGENTS.md`](../AGENTS.md)
2. [`specs/collatz-engine/spec.md`](../specs/collatz-engine/spec.md), especially
   Blueprint constraints and Contract guardrails
3. [`ARCHITECTURE.md`](../ARCHITECTURE.md), especially planned boundaries and
   dependency direction
4. [`docs/mathematical-definitions.md`](../docs/mathematical-definitions.md), to
   keep names ready for later formal correspondence
5. [`docs/adrs/README.md`](../docs/adrs/README.md), if a new architectural
   choice is discovered

## Verification

- [ ] The workspace resolves for `aarch64-apple-darwin` with pinned Rust and
  Lean4 toolchains recorded in version-controlled files.
- [ ] `cargo metadata --no-deps` describes the intended workspace and engine
  crate.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passes.
- [ ] `cargo test --workspace --all-features` passes smoke tests without
  claiming that any engine Contract scenario is implemented.
- [ ] `cd proofs && lake build` passes with no `sorry` or equivalent unproved
  theorem escape hatch.
- [ ] `cargo bench --workspace --no-run` compiles the Criterion harness without
  running or interpreting performance measurements.
- [ ] CI runs Rust and Lean correctness checks before benchmark compilation on
  a macOS Apple Silicon runner.
- [ ] The repository contains no implementation of `n / 2`, `3n + 1`, or
  bounded trajectory evaluation outside documentation and non-executable test
  descriptions.
- [ ] The commands and planned paths in `AGENTS.md` and `ARCHITECTURE.md` match
  the resulting scaffold.

## Refinement Protocol

- If toolchain setup changes only concrete versions or exact file names while
  preserving the architecture, update the affected pointers and commands in
  the same change.
- If setup requires changing engine semantics, numeric representations,
  platform scope, proof boundaries, or correctness-gate ordering, stop and
  request human review before editing the Spec.
- If setup introduces a significant choice with credible alternatives, add a
  proposed ADR following [`docs/adrs/README.md`](../docs/adrs/README.md).
- Record implementation discoveries in the living Spec; do not copy permanent
  design into this PBI.
