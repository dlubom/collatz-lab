# ADR-002: Lean 4 Verification Boundary

**Status:** Accepted

**Date:** 2026-08-02

## Context

Collatz Lab needs definitions and optimization obligations that remain precise
across implementation and performance work. It also needs honest claims about
what has and has not been verified. A Lean model can prove arithmetic and
iteration properties, but the MVP does not have a formal semantics bridge for
Rust, GMP, parsers, serializers, or the operating system.

Overstating the proof boundary would be more harmful than having no formal layer:
users could mistake local mathematical theorems or finite computation for a
proof of the Collatz conjecture or of the compiled application.

## Decision

We will use Lean 4 to:

- define the mathematical reference model;
- prove local mathematical properties;
- prove correspondence and step-accounting properties for accelerated,
  compressed, and future optimized transformations;
- validate or generate small reviewed test vectors;
- record all assumptions needed by theorems explicitly.

In the MVP Lean 4 will not formally verify:

- the compiled Rust program or Rust compiler;
- GMP or `rug::Integer`;
- CLI parsing;
- JSON or JSONL serialization;
- the operating system;
- memory management;
- performance.

Lean will not attempt to prove that every positive integer reaches one.

## Consequences

**Positive:**

- Step semantics and optimization claims have an executable, reviewable formal
  authority.
- Local proof obligations can block mathematically invalid optimizations before
  benchmarking.
- Small Lean-checked examples provide evidence independent of the Rust engine.
- Explicit assumptions constrain the wording of project claims.

**Negative:**

- The project maintains two implementations of related definitions and must
  review them for alignment.
- Passing `lake build` does not establish executable conformance; Rust tests and
  differential checks remain necessary.
- Lean expertise and a pinned toolchain add maintenance cost.
- End-to-end formal verification remains outside the assurance case.

**Neutral:**

- Lean is a design-time component under `lean/` and is not linked into the Rust
  runtime.
- The formal boundary may expand later only through a new explicit decision.

## Alternatives Considered

### No formal model

Documentation and tests alone would simplify the toolchain, but would leave
compression and special-form identities without machine-checked proofs and
make later optimization reviews depend entirely on examples.

### Verify the full Rust/GMP application in the MVP

Full executable verification would require formal Rust semantics, foreign
library modeling, extraction or a verified compilation path, and proofs for
I/O layers. That is a separate research program and would prevent delivery of
the intended experimental MVP.

### Implement the complete application in Lean

This would reduce the model/runtime language gap but would not eliminate the
need to trust Lean's compiler and runtime, and it would weaken the fit with the
planned GMP-backed CLI and systems benchmarking. ADR-001 selects Rust for those
responsibilities.

## Reconsideration triggers

Create a superseding ADR if the project adopts verified extraction, formal Rust
semantics, a proof-producing result format, or any claim that extends beyond
the local mathematical boundary above.
