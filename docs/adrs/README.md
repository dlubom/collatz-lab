# Architecture Decision Records

This directory contains immutable records for significant Collatz Lab
architecture decisions. The foundational ADRs were accepted with the
documentation baseline on 2026-08-02.

## Index

| ADR | Status | Decision |
|---|---|---|
| [ADR-001](ADR-001-rust-as-implementation-language.md) | Accepted | Use Rust for the executable implementation |
| [ADR-002](ADR-002-lean4-verification-boundary.md) | Accepted | Limit Lean 4 claims to the mathematical model and local obligations |
| [ADR-003](ADR-003-arbitrary-precision-with-rug-and-gmp.md) | Accepted | Use `rug::Integer`/GMP and promote before overflow |
| [ADR-004](ADR-004-no-simd-or-gpu-in-mvp.md) | Accepted | Keep the MVP scalar; defer SIMD and GPU |

The sequence is intentional: implementation language precedes the formal
boundary, which precedes the arbitrary-precision implementation, which precedes
the execution-model exclusion.

## Creating an ADR

Follow the local ASDLC
[`ADR Authoring`](../../.asdlc/practices/adr-authoring.md) practice:

1. search this directory for a decision in the same domain;
2. select the next sequential number;
3. use `ADR-NNN-short-descriptive-title.md`;
4. record context, an unambiguous decision, positive/negative/neutral
   consequences, concrete alternatives, and reconsideration triggers;
5. start at `Proposed` unless a human has explicitly accepted it.

Search examples:

```bash
rg -n "^# ADR-|^\*\*Status:\*\*|^## Decision" docs/adrs
rg -n -i "rust|lean|gmp|simd|gpu" docs/adrs
```

## Lifecycle

Accepted ADR content is historical and is not rewritten. If a decision changes,
create a new ADR, update the old status to `Superseded by ADR-NNN`, and preserve
the original context. Routine definitions and implementation details belong in
the mathematical authority, living Spec, or active PBI rather than in an ADR.
