# AGENTS.md

> **Project:** Collatz Lab is a correctness-first laboratory for studying the
> standard Collatz map with two scalar Rust engines on macOS Apple Silicon.
> The bounded `u128` engine is the Rust reference implementation; the
> `rug::Integer` engine supplies arbitrary-precision arithmetic. Lean4 records
> and proves the local mathematical obligations before performance work is
> accepted.

## Sources of truth

Read these in order before changing engine behavior:

1. [`specs/collatz-engine/spec.md`](specs/collatz-engine/spec.md) — feature
   state and acceptance contract.
2. [`docs/mathematical-definitions.md`](docs/mathematical-definitions.md) —
   notation, step semantics, and proof boundary.
3. [`ARCHITECTURE.md`](ARCHITECTURE.md) — component boundaries and dependency
   direction.
4. The active file under `tasks/` — the authorized delta.

This repository uses lightweight, spec-anchored ASDLC. Keep behavior changes
and their living spec changes together, following the
[`Living Specs` practice](.asdlc/practices/living-specs.md). PBIs are pointers
to the current spec, not copies of it; follow
[`PBI Authoring`](.asdlc/practices/pbi-authoring.md).

## Toolchain

The commands below are the intended project entry points. They become
available when [`PBI-001`](tasks/PBI-001-bootstrap.md) is implemented; until
then, do not report them as passing.

| Intent | Command | Authority |
|---|---|---|
| Format | `cargo fmt --all -- --check` | Rust formatter configuration |
| Static analysis | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Cargo workspace and Clippy |
| Rust correctness | `cargo test --workspace --all-features` | Spec Contract and test suite |
| Lean correctness | `cd proofs && lake build` | Pinned Lean toolchain and proof sources |
| Benchmark build | `cargo bench --workspace --no-run` | Criterion benchmark targets |
| Benchmarks | `cargo bench --workspace` | Criterion reports on Apple Silicon |

## Judgment boundaries

**ALWAYS**

- Define or refine mathematical semantics and Lean4 obligations before
  optimizing an engine.
- Use checked arithmetic in the `u128` engine and exact arithmetic in the
  `rug::Integer` engine.
- Run proof and correctness gates before interpreting benchmark results.
- Add example, property-based, and cross-engine regression coverage for every
  behavior change.
- Update the relevant spec in the same change when observable behavior changes.

**ASK**

- Before changing the Collatz map, stopping convention, step-count convention,
  or public result/error semantics.
- Before adding dependencies beyond Rust, `rug`, the property-testing stack,
  the benchmarking stack, and Lean4 tooling.
- Before expanding the supported platform or execution model beyond scalar CPU
  execution on macOS Apple Silicon.

**NEVER**

- Treat finite computation as a proof of the Collatz conjecture.
- Hide arithmetic overflow, step-limit exhaustion, or invalid zero input.
- Accept a performance optimization that changes mathematical results or step
  accounting.

This file follows the local
[`AGENTS.md` practice](.asdlc/practices/agents-md-spec.md): it contains only
cross-cutting judgment and pointers; feature details remain in the Spec.
