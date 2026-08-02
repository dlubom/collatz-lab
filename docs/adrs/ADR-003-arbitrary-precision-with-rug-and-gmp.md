# ADR-003: Arbitrary Precision with rug and GMP

**Status:** Accepted

**Date:** 2026-08-02

## Context

The checked `u128` reference engine is intentionally bounded, but Collatz Lab
must study inputs and intermediate values beyond 128 bits. The arbitrary-
precision path must use exact integer arithmetic, agree with the bounded engine
on their common domain, and avoid turning custom limb arithmetic into a second
research project.

The hybrid execution policy must promote before an odd `3n+1` operation would
overflow. Promotion after overflow, wrapping, or recomputing an already-counted
transition would violate the mathematical contract.

## Decision

We will use `rug::Integer`, backed by GMP, for arbitrary-precision arithmetic.
The standalone BigInt engine will keep values as `rug::Integer` for the complete
run. A hybrid runner may begin in `u128`; before an unrepresentable odd
transition it will convert the current value to `rug::Integer`, execute that
transition exactly, and continue without changing step, peak, or stopping
semantics.

We will not implement a custom BigInt. Common-domain differential tests will
compare the checked `u128` and BigInt engines, and promotion tests will compare
the hybrid suffix with a BigInt-from-start run.

## Consequences

**Positive:**

- GMP supplies mature, exact, high-performance large-integer arithmetic.
- `rug` provides an idiomatic Rust ownership and safety boundary.
- The project can handle inputs and peaks beyond fixed-width arithmetic.
- Differential testing separates bounded checked behavior, exact behavior, and
  promotion behavior.

**Negative:**

- GMP increases build time and complicates dependency compilation and linking.
- Platform portability depends on `gmp-mpfr-sys`/GMP support and must be checked
  explicitly before adding platforms.
- A defect in `rug` or GMP remains outside Lean's proof boundary.
- Conversion and allocation costs affect benchmark interpretation.

**Neutral:**

- The bounded reference engine continues to report overflow instead of
  promoting; only the hybrid policy promotes.
- The initial supported environment remains macOS Apple Silicon.
- Exact dependency versions are pinned during the implementing PBI rather than
  in this decision record.

## Alternatives Considered

### A pure-Rust BigInt library

A pure-Rust implementation could simplify foreign-library builds and broaden
portability. It was not selected for the baseline because GMP is mature and
highly optimized for the very large arithmetic central to this project. The
choice may be revisited if build or deployment evidence outweighs that benefit.

### Direct GMP FFI

Calling GMP directly would expose lower-level lifetime and unsafe-code concerns
to project code, conflicting with the MVP's `#![forbid(unsafe_code)]` policy.
`rug` provides the needed abstraction.

### Custom BigInt

A custom representation would expand scope into arithmetic-library design,
carry handling, allocation, and its own extensive validation. It provides no
MVP research advantage and would weaken independent confidence.

### Fixed-width integers larger than 128 bits

A larger fixed width merely moves the overflow boundary. It cannot support
arbitrary manually supplied inputs or unbounded intermediate growth and would
still require an exact fallback.

## Reconsideration triggers

Reconsider if GMP cannot build reliably on the supported platform, if licensing
or distribution constraints change, or if measured workloads and validated
alternatives show a material advantage without weakening exactness or test
independence.
