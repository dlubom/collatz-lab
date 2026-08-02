# Feature: Collatz Engine

- **Status:** Proposed
- **ASDLC mode:** Spec-anchored, lightweight
- **Mathematical authority:**
  [`docs/mathematical-definitions.md`](../../docs/mathematical-definitions.md)

This document uses the ASDLC Blueprint and Contract structure from
[`Living Specs`](../../.asdlc/practices/living-specs.md).

## Blueprint

### Context

Collatz Lab needs a small execution core whose behavior can be reasoned about
mathematically before it is optimized. The engine supports bounded experiments
with the standard Collatz map while making nontermination, numeric overflow,
and counting conventions explicit.

Two scalar Rust implementations provide complementary evidence. A checked
`u128` engine is the bounded reference implementation. A
`rug::Integer`/GMP engine evaluates arbitrary-precision values and is checked
against the reference wherever their representable domains overlap.

### Architecture

#### Mathematical operation

For every positive integer `n`, one engine step applies the unaccelerated map:

```text
n / 2      when n is even
3n + 1     when n is odd
```

Each branch application counts as exactly one transition. A run stops when it
first observes `1`; it does not apply the odd rule to that terminal value.

#### Conceptual API

- `step(value)` returns one next value or a typed engine error.
- `run(start, step_limit)` returns a finite run summary or an invalid-input
  error.
- `RunSummary` contains `start`, `last`, `steps`, `peak`, and `termination`.
- `termination` is one of `ReachedOne`, `StepLimitReached`, or, for the `u128`
  engine only, `ArithmeticOverflow`.
- `start = 0` returns `InvalidInput`; zero is outside the mathematical domain.

This is a behavioral contract, not a frozen Rust signature. PBI implementation
may choose idiomatic Rust names and ownership while preserving these observable
semantics.

#### Implementations

- **Reference engine:** Rust with `u128` values and checked odd-branch
  arithmetic.
- **Arbitrary-precision engine:** Rust with `rug::Integer`, backed by GMP, with
  exact arithmetic.
- **Formal model:** Lean4 definitions and local proofs corresponding to the
  mathematical authority.

Both engines follow the run-state ordering in
[`ARCHITECTURE.md`](../../ARCHITECTURE.md#run-state-ordering). Neither engine
uses the other in its runtime implementation.

#### Verification dependencies

- Lean4 for formal definitions and local theorems.
- Rust's test harness plus a property-based testing library for generated
  invariants.
- Cross-engine differential tests over bounded, representable prefixes.
- Criterion for benchmarks after correctness gates pass.

### Constraints

- The supported execution target is macOS Apple Silicon
  (`aarch64-apple-darwin`).
- Engine execution is scalar CPU code.
- The standard, unaccelerated Collatz map and one-branch-per-transition counting
  define all results.
- Every public run is finite because the caller supplies a transition limit.
- `u128` arithmetic is checked; overflow is an observable termination reason.
- `rug::Integer` values remain arbitrary precision throughout engine
  computation.
- Mathematical correctness work, Lean4 proof obligations, and executable tests
  precede benchmark-driven optimization.
- Formal claims are limited to proved local properties. Universal termination
  is not an assumption or project claim.

## Contract

### Definition of Done

- [ ] A Rust engine crate exposes the shared single-step and bounded-run
  semantics described in the Blueprint.
- [ ] The checked `u128` reference engine returns correct results within its
  domain and a typed overflow outcome outside it.
- [ ] The `rug::Integer`/GMP engine applies the same semantics without numeric
  narrowing.
- [ ] Zero input, zero-transition limits, reaching one, exhausting a limit, and
  arithmetic overflow have distinct, tested outcomes.
- [ ] Lean4 formalizes the map and discharges the local proof obligations listed
  in the mathematical definitions.
- [ ] Fixed examples, property-based tests, and cross-engine differential tests
  cover the behavioral contract.
- [ ] Criterion benchmarks build and run on macOS Apple Silicon with documented
  cases and without becoming correctness gates.
- [ ] Correctness gates pass before any performance result is accepted or used
  to justify a change.

### Regression Guardrails

- A transition always applies exactly one documented branch; odd transitions
  are not implicitly accelerated by dividing out powers of two.
- The initial value is trajectory index zero and is included in the peak.
- A run that starts at `1` completes with zero transitions and peak `1`.
- A transition count increases only after a new value has been represented
  successfully.
- The `u128` engine never wraps arithmetic and never substitutes a saturated
  value.
- The arbitrary-precision engine never narrows intermediate values to `u128`.
- On their common representable domain, both engines return the same values,
  counts, peaks, and terminal classification.
- Hitting a step limit makes no claim about whether the trajectory eventually
  reaches one.
- Benchmark-specific changes preserve every mathematical and behavioral
  contract above.

### Scenarios

```gherkin
Feature: Correct bounded evaluation of the standard Collatz map

  Scenario: A run starting at one is already complete
    Given the positive start value 1
    And a step limit of 0
    When either engine runs the trajectory
    Then the termination reason is ReachedOne
    And the completed transition count is 0
    And the last value and peak are both 1

  Scenario: The reference engine evaluates a known trajectory
    Given the positive u128 start value 6
    And a step limit of at least 8
    When the reference engine runs the trajectory
    Then the represented values are 6, 3, 10, 5, 16, 8, 4, 2, 1
    And the termination reason is ReachedOne
    And the completed transition count is 8
    And the peak is 16

  Scenario: Both engines agree on a longer known trajectory
    Given the positive start value 27
    And a step limit of at least 111
    When both engines run the trajectory
    Then each termination reason is ReachedOne
    And each completed transition count is 111
    And each peak is 9232

  Scenario: Zero is rejected at the domain boundary
    Given the start value 0
    When either engine is asked to run the trajectory
    Then it returns InvalidInput
    And no transition is attempted

  Scenario: A zero-length prefix remains observable
    Given the positive start value 6
    And a step limit of 0
    When either engine runs the trajectory
    Then the termination reason is StepLimitReached
    And the completed transition count is 0
    And the last value and peak are both 6

  Scenario: Checked arithmetic reports an unrepresentable odd step
    Given the reference engine value u128::MAX
    When one Collatz step is requested
    Then it returns ArithmeticOverflow
    And it does not return a wrapped or saturated value

  Scenario: Arbitrary precision evaluates beyond the reference range
    Given the arbitrary-precision value equal to u128::MAX
    When one Collatz step is requested
    Then the result is exactly 3 times u128::MAX plus 1
    And the result is even

  Scenario: Generated inputs agree on the shared numeric domain
    Given a generated positive u128 start and finite step limit
    And every value in the observed prefix is representable as u128
    When both engines evaluate that prefix
    Then their values, transition count, peak, and termination reason agree

  Scenario: Correctness gates precede benchmark acceptance
    Given a proposed engine optimization
    When its performance is evaluated
    Then the relevant Lean4 proof obligations build successfully
    And fixed, property-based, and differential tests pass
    And only then may its Criterion measurements be compared
```

### Traceability

- Mathematical terms and proof limits:
  [`docs/mathematical-definitions.md`](../../docs/mathematical-definitions.md)
- System boundaries and gate order:
  [`ARCHITECTURE.md`](../../ARCHITECTURE.md)
- Initial executable delta:
  [`tasks/PBI-001-bootstrap.md`](../../tasks/PBI-001-bootstrap.md)
