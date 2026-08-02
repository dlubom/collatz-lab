# Project Vision

- **Status:** Accepted project vision
- **Phase:** Documentation foundation; no implementation or experiment results
- **Durable contract:** [`specs/collatz-engine/spec.md`](../specs/collatz-engine/spec.md)

## Problem

Published Collatz work and verification projects understandably emphasize
systematic coverage, theoretical results, or global record searches. Collatz
Lab addresses a narrower experimental need: inspect trajectories of selected
integers that are already notable because of another mathematical structure,
data source, or iterative process, and compare them with reproducible controls.

The project must make it easy to ask a careful question about one number, a
small list, or a defined family without turning the observation into a claim
about universal termination. It must also retain enough provenance to let
another person reconstruct both the input and the experiment.

## Motivation

An integer can be interesting before its Collatz trajectory is computed. It may
come from primality research, a digit pattern, a factorization challenge, an
OEIS sequence, or a record in another dynamical system. Those selection
mechanisms may introduce structure that is invisible when inputs are sampled
uniformly by magnitude alone.

Collatz Lab tests whether such structure is associated with measurable Collatz
behavior. The valuable outcome is a reproducible observation, including a
well-supported negative result. Finding a counterexample is a remote long-term
possibility, not a planning assumption or success criterion.

## Research questions and null stance

The project asks:

1. Do known number families have Collatz trajectories statistically different
   from random integers of the same bit length?
2. Does the form `a * 2^m - 1` produce a predictable initial growth phase?
3. Are record holders from other iterative processes also unusual under the
   Collatz map?
4. Can reproducible records be identified inside a family even when they are
   not global records?
5. Are primality, factorization, or digit-representation properties measurably
   associated with Collatz dynamics?
6. Do between-family differences remain after normalization by input bit
   length?
7. Can results recorded in existing databases and OEIS sequences be reproduced
   independently?

For every question the initial position is the null stance: no effect is
assumed. Comparisons must report the observed sample, controls, uncertainty,
and selection limitations rather than interpreting a difference as causal.

## Candidate number families

Initial and prospective inputs include:

- Mersenne numbers and known Mersenne primes;
- Fermat numbers;
- repunits in declared bases;
- values `a * 2^m - 1`;
- `n! - 1`, `n! + 1`, `p# - 1`, and `p# + 1`;
- Carmichael numbers and pseudoprimes;
- known RSA Challenge numbers;
- multiplicative-persistence record holders;
- selected OEIS entries and record holders from other iterative processes;
- manually supplied large integers;
- random controls matched by bit length.

Inclusion in this list is a source category, not evidence of unusual Collatz
behavior. Each concrete experiment still needs a stable ID, a stated selection
rule, and a control design.

## Planned experiment types

1. **Oracle reproduction:** run small, independently established trajectories
   and published examples to validate semantics and measurement.
2. **Within-family ranking:** compare members of one family by predeclared
   metrics such as first-descent time or peak bit length.
3. **Matched-control comparison:** compare a special family with a deterministic
   random sample matched on bit length and, where practical, sample size.
4. **Structural sequence analysis:** test a preregistered consequence of a
   formula, such as the initial accelerated checkpoints of `a * 2^m - 1`.
5. **External-data reproduction:** reconstruct values and check results from a
   cited database or OEIS entry.
6. **Candidate investigation:** independently rerun an exceptional observation
   with multiple engines and a separate tool.

## What counts as interesting

An observation may be marked interesting when at least one of these conditions
is satisfied and the experiment definition existed before interpretation:

- it is extreme relative to the declared matched controls, with its rank and
  relevant quantiles reported;
- it is a new reproducible maximum or minimum inside a precisely defined
  family and sample;
- it contradicts a predeclared structural expectation;
- it reproduces or fails to reproduce a cited external result;
- it reveals a stable association that persists under bit-length normalization
  and reasonable sensitivity checks;
- it exposes a software, data, or methodology defect worth correcting.

An interesting observation is not automatically a record, discovery, or
counterexample. Those labels require the confirmation process in
[`experimental-methodology.md`](experimental-methodology.md).

## Interpretation principles

- State the tested population and never generalize beyond it without an
  explicit argument.
- Separate exploratory observations from preregistered comparisons.
- Report sample sizes and control-generation rules alongside summary metrics.
- Treat timing as machine- and configuration-specific.
- Distinguish completed trajectories from prefixes stopped by a bound or
  resource limit.
- Treat multiple comparisons and post-hoc family selection as sources of false
  positives.
- Preserve negative and invalidated results; they are part of the audit trail.
- Require independent reproduction before calling an unusual result credible.

## Project boundaries

The MVP is a scalar, local, command-line-oriented research tool for selected
inputs and modest catalogs. It excludes mass interval verification, SIMD, GPU,
distributed computation, a GUI, a network service, a server database,
automatic record publication, advanced multi-iteration polynomial jumps, and a
custom BigInt implementation.

Lean 4 establishes definitions and local mathematical obligations. It does not
verify the compiled Rust binary, GMP, `rug::Integer`, parsing, serialization,
the operating system, memory management, or performance. The project does not
attempt a proof of the Collatz conjecture.

## Potential later directions

After the MVP is correct and measurements justify more work, possible
directions include broader family catalogs, stronger statistical models,
parallel execution across independent inputs, authenticated external data
imports, richer artifact manifests, and profiled acceleration. SIMD, GPU, or
multi-iteration jumps are reconsidered only with a measured bottleneck and a
proof or equivalence argument tied to the reference model.
