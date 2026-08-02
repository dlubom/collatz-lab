# Architecture Decision Records

This directory contains immutable records for significant Collatz Lab
architecture decisions. The current baseline comes directly from the project
brief and is expressed as current state in [`ARCHITECTURE.md`](../../ARCHITECTURE.md)
and the Collatz engine Spec; no ADR has yet been accepted.

Use the local ASDLC [`ADR Authoring`](../../.asdlc/practices/adr-authoring.md)
practice. Create an ADR when a choice has meaningful alternatives, material
trade-offs, and consequences for future work. Routine implementation details
remain in the relevant Spec or PBI.

## Index

| ADR | Status | Decision |
|---|---|---|
| _None yet_ | — | — |

Likely decision triggers include the GMP build/linking strategy, the mechanism
used to connect Lean4 evidence to Rust conformance tests, and any future change
to the scalar Apple-Silicon-only execution model. These are triggers for
analysis, not pre-decided ADRs.

## Naming and lifecycle

- Filename: `ADR-NNN-short-descriptive-title.md`
- One decision per ADR.
- Initial status: `Proposed`; use `Accepted` only after review.
- Preserve accepted content. Replace a decision with a new ADR and mark the old
  record `Superseded` with a link to its successor.

## Template

```markdown
# ADR-NNN: Decision title

**Status:** Proposed
**Date:** YYYY-MM-DD

## Context

What forces, constraints, and requirements make a decision necessary?

## Decision

We will ...

## Consequences

**Positive:**
- ...

**Negative:**
- ...

**Neutral:**
- ...

## Alternatives Considered

### Alternative

Specific rejection rationale.
```
