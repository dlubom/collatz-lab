# ADR-001: Rust as the Implementation Language

**Status:** Accepted

**Date:** 2026-08-02

## Context

Collatz Lab needs a small, auditable command-line research tool with a simple
bounded reference engine, arbitrary-precision integration, deterministic tests,
and credible performance measurement on macOS Apple Silicon. The standard odd
step can overflow fixed-width integers, so arithmetic failure must be checked
and represented rather than hidden by language defaults.

The implementation language must support efficient scalar execution without
making manual memory management part of the project's correctness burden. It
must also have mature testing, property-testing, benchmarking, and profiling
tools and a practical route to GMP.

## Decision

We will implement the executable Collatz Lab MVP in Rust. Production crate roots
will forbid unsafe code, the `u128` engine will use checked arithmetic, and
arbitrary precision will integrate through a safe Rust library described in
ADR-003.

Rust supplies memory safety without a garbage collector, explicit result/error
types, checked integer operations, a practical high-performance CLI ecosystem,
native Apple Silicon support, and mature Cargo-based test and benchmark
workflows.

## Consequences

**Positive:**

- Memory safety removes a large class of defects from the project's own code.
- Checked integer APIs make overflow policy explicit and testable.
- Native code and zero-cost abstractions support later scalar optimization.
- Cargo provides one ecosystem for builds, unit/property tests, Clippy,
  coverage integration, Criterion, mutation testing, and profiling builds.
- The Rust ecosystem provides maintained GMP integration.

**Negative:**

- Rust's ownership and type system increase initial implementation complexity.
- GMP-backed dependencies make builds less self-contained than a pure-Rust
  fixed-width engine.
- Rust/Lean conformance remains a testing and review obligation because code is
  not extracted from Lean.
- Developers need both Rust and Lean toolchains for the full quality gate.

**Neutral:**

- The supported MVP platform is still explicitly macOS Apple Silicon; Rust's
  broader portability does not expand the acceptance contract by itself.
- CLI and serialization choices remain separate decisions or PBI-level details.

## Alternatives Considered

### C++

C++ can match or exceed scalar performance and integrate with GMP directly.
It was not selected because memory safety and arithmetic/error discipline would
depend more heavily on review and local conventions, increasing the validation
surface of a correctness-first MVP.

### Python

Python offers built-in arbitrary-precision integers and fast prototyping. It was
not selected as the primary engine language because interpreter overhead would
confound the intended scalar engine benchmarks, and a separate compiled engine
would likely become necessary for the project's performance phase. Python may
later serve as an independent verification or analysis tool.

### Pure Lean 4

Lean can define and execute the mathematical model with strong proof
integration. It was not selected for the complete tool because the MVP also
needs a conventional CLI, GMP-backed large-integer engine, artifact processing,
and representative systems-performance measurements. Lean remains the formal
mathematical layer rather than the runtime implementation language.

## Reconsideration triggers

Reconsider only if Rust cannot satisfy a required correct behavior on the
supported platform, or if an independently reviewed implementation strategy
provides materially stronger end-to-end assurance without expanding the MVP
beyond its research goals.
