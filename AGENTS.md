# AGENTS.md

> **Project:** Collatz Lab is a correctness-first laboratory for studying the
> standard Collatz map with two scalar Rust engines on macOS Apple Silicon.
> The bounded `u128` engine is the Rust reference implementation; the
> `rug::Integer` engine supplies arbitrary-precision arithmetic. Lean4 records
> and proves the local mathematical obligations before performance work is
> accepted.

## Sources of truth

Read these in order before changing observable behavior:

1. [`specs/collatz-engine/spec.md`](specs/collatz-engine/spec.md) — feature
   state and acceptance contract.
2. [`docs/mathematical-definitions.md`](docs/mathematical-definitions.md) —
   notation, step semantics, and proof boundary.
3. [`ARCHITECTURE.md`](ARCHITECTURE.md) — component boundaries and dependency
   direction.
4. The active file under `tasks/` — the authorized delta.

Experiment work also reads
[`docs/experimental-methodology.md`](docs/experimental-methodology.md); quality
work reads [`docs/quality-strategy.md`](docs/quality-strategy.md). Do not copy
their permanent rules into a PBI.

This repository uses lightweight, spec-anchored ASDLC. Keep behavior changes
and their living spec changes together, following the
[`Living Specs` practice](.asdlc/practices/living-specs.md). PBIs are pointers
to the current spec, not copies of it; follow
[`PBI Authoring`](.asdlc/practices/pbi-authoring.md).

## Delivery workflow

Every repository-changing task is delivered through a topic branch and pull
request. Follow [`CONTRIBUTING.md`](CONTRIBUTING.md) for the complete workflow
and activate the tracked safeguards with `./scripts/install-git-hooks.sh` in
each clone.

- Start work from the current `origin/main` on a `codex/<description>` branch;
  use `codex/pbi-NNN-<description>` for an implementation PBI.
- Treat applicable verification, an intentional commit, push, and draft pull
  request as part of task completion.
- Keep merge as a separate human-reviewed action. Never merge merely because
  implementation and checks completed.
- If a required gate fails, report the exact blocker and do not represent the
  task as complete. Push an incomplete checkpoint only when explicitly asked.

## Toolchain

The commands below are intended project entry points. Lean becomes available in
[`PBI-001`](tasks/PBI-001-formal-reference-model.md), Rust and core quality gates
in [`PBI-002`](tasks/PBI-002-rust-reference-engine.md), and benchmark compilation
in [`PBI-003`](tasks/PBI-003-arbitrary-precision-engine.md). Until the owning PBI
is implemented, do not report its command as passing.

| Intent | Command | Authority |
|---|---|---|
| Format | `cargo fmt --all -- --check` | Rust formatter configuration |
| Static analysis | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Cargo workspace and Clippy |
| Rust correctness | `cargo test --workspace` | Spec Contract and test suite |
| Lean correctness | `(cd lean && lake build)` | Pinned Lean toolchain and proof sources |
| Core coverage | `cargo llvm-cov --package collatz-engine --lib --all-features --fail-under-lines 90` | Mathematical-core threshold |
| Reference mutation | `cargo mutants --file crates/collatz-engine/src/reference.rs` | Material-mutant review |
| Benchmark build | `cargo bench --workspace --no-run` | Criterion benchmark targets |
| Benchmarks | `cargo bench --workspace` | Criterion reports on Apple Silicon |

## Judgment boundaries

**ALWAYS**

- Work on a topic branch and finish repository-changing tasks with an
  intentional commit, push, and draft pull request.
- Define or refine mathematical semantics and Lean4 obligations before
  optimizing an engine.
- Use checked arithmetic in the `u128` engine and exact arithmetic in the
  `rug::Integer` engine.
- Run proof and correctness gates before interpreting benchmark results.
- Add example, property-based, and cross-engine regression coverage for every
  applicable behavior change.
- Update the relevant spec in the same change when observable behavior changes.
- Preserve number provenance, configuration identity, and complete-versus-prefix
  metric labels in every experiment result.

**ASK**

- Before changing the Collatz map, stopping convention, step-count convention,
  or public result/error semantics.
- Before adding dependencies beyond Rust, `rug`, the property-testing stack,
  the coverage/mutation/benchmarking stack, and Lean4 tooling.
- Before expanding the supported platform or execution model beyond scalar CPU
  execution on macOS Apple Silicon.

**NEVER**

- Commit or push directly to `main`, bypass repository hooks with
  `--no-verify`, force-push, or merge a pull request without explicit human
  direction.
- Treat finite computation as a proof of the Collatz conjecture.
- Hide arithmetic overflow, step-limit exhaustion, or invalid zero input.
- Accept a performance optimization that changes mathematical results or step
  accounting.

This file follows the local
[`AGENTS.md` practice](.asdlc/practices/agents-md-spec.md): it contains only
cross-cutting judgment and pointers; feature details remain in the Spec.
