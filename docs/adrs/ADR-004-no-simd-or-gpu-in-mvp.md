# ADR-004: No SIMD or GPU in the MVP

**Status:** Accepted

**Date:** 2026-08-02

## Context

One Collatz trajectory is sequential: each next value depends on the current
value. SIMD or GPU strategies therefore require batching independent starts,
managing divergence, or introducing transformed jumps. Those changes would
increase algorithmic and verification complexity before the simple engines,
metrics, and experiment methodology have been validated.

The MVP's research focus is selected values and modest matched controls, not
high-throughput interval scanning. No profiling evidence yet identifies a
hardware-parallel bottleneck because no engine has been implemented.

## Decision

We will implement the MVP as scalar CPU execution and will not add explicit
SIMD or GPU code. Correctness, exact step accounting, reproducibility, and
algorithmic clarity take precedence over throughput.

Independent numbers may be parallelized in a future phase, but that is not part
of this decision's MVP scope. SIMD or GPU execution will be reconsidered only
after profiling a correct baseline, identifying a workload that benefits, and
providing equivalence tests and formal obligations appropriate to the changed
algorithm.

## Consequences

**Positive:**

- Reference behavior remains simple to inspect and compare with Lean.
- The initial test and proof surface is smaller.
- Benchmark results establish an honest scalar baseline.
- Effort remains focused on the experimental question rather than throughput
  infrastructure.

**Negative:**

- Large batches will execute more slowly than a successful parallel design.
- The MVP cannot compete with specialized mass-verification projects on
  throughput.
- Later parallel work may require new data layout and scheduling boundaries.

**Neutral:**

- GMP may use internal implementation optimizations outside project control;
  this ADR excludes project-designed SIMD/GPU execution.
- Criterion still measures scalar engines on Apple Silicon.

## Alternatives Considered

### SIMD batching in the MVP

SIMD could process several trajectories together, but variable branch patterns,
step counts, overflow points, and termination statuses create lane divergence
and complicate accounting. There is no measurement showing this complexity is
needed for selected-input experiments.

### GPU batching in the MVP

GPU execution could offer high throughput for many starts, but requires a new
toolchain, host/device data flow, divergent control handling, and a different
arbitrary-precision strategy. It conflicts with the small local MVP and lacks a
validated workload.

### Parallel CPU execution across independent inputs

This is more natural than parallelizing a single trajectory and remains a
credible later step. It is deferred so deterministic result ordering, resource
limits, and the scalar correctness baseline are established first.

## Reconsideration triggers

A superseding ADR may be proposed after profiling identifies the bottleneck,
the target experiment size justifies the complexity, reference/differential
tests cover the new path, and Lean or equivalent evidence covers every changed
mathematical transformation.
