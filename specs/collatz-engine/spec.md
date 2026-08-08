# Feature: Collatz Lab MVP

- **Status:** Accepted baseline; Lean model, scalar engines, and experiment slice implemented
- **ASDLC mode:** Lightweight, spec-anchored
- **Scope:** Collatz execution, selected-number experiments, provenance, and
  reproducible MVP results
- **Mathematical authority:**
  [`docs/mathematical-definitions.md`](../../docs/mathematical-definitions.md)

This existing feature-domain path remains the project's single living MVP Spec;
no competing `specs/collatz-lab/spec.md` is created. Detailed rationale lives in
the vision and ADRs, while this document records current state and observable
acceptance contracts.

## Blueprint

### Context

Collatz Lab studies trajectories of selected known, structured, unusual, and
very large positive integers. It is intended for small reproducible experiments
and comparisons with bit-length-matched random controls, not systematic
verification of enormous contiguous ranges.

Potentially interesting outcomes include long steps-to-one, late first descent,
large peaks, within-family records, family/control differences, reproducible
external-data discrepancies, and candidates for independent investigation.
Such outcomes are finite observations. The project does not promise a
counterexample and does not treat computation as a proof of the Collatz
conjecture.

### Goals

- Accept one positive integer, an ordered list, or a supported generator
  definition.
- Execute the standard Collatz trajectory with a simple checked reference
  engine and arbitrary-precision arithmetic.
- Promote from `u128` before overflow when the hybrid policy is selected.
- Report mathematically unambiguous metrics and termination statuses.
- Record exact construction, source provenance, limits, configuration, and
  program version.
- Create deterministic random controls matched by bit length.
- Produce a versioned line-oriented result suitable for later analysis and
  exact reruns.
- Validate key mathematical transformations and small examples in Lean 4.
- Require independent reproduction before an exceptional result is treated as
  credible.

### Non-goals

- Proving or claiming the Collatz conjecture.
- Systematic scanning of huge contiguous intervals.
- SIMD, GPU, distributed computation, or parallel search in the MVP.
- A GUI, network service, or server-backed database.
- Automatic download of large external datasets or automatic record
  publication.
- Advanced multi-iteration polynomial jumps or a custom BigInt implementation.
- Formal verification of the compiled Rust program, GMP, `rug::Integer`, CLI,
  JSON/JSONL serialization, operating system, memory management, or
  performance.

### Research questions

1. Do known number families differ from random values of the same bit length?
2. Does `a * 2^m - 1` yield predictable initial accelerated growth?
3. Are record holders from other iterative processes unusual under Collatz?
4. Can records be found inside precisely defined families?
5. Are primality, factorization, or digit properties measurably associated with
   Collatz dynamics?
6. Do family differences remain after normalization by input bit length?
7. Can published database and OEIS observations be reproduced independently?

No positive answer is assumed. The detailed interpretation stance is in
[`docs/project-vision.md`](../../docs/project-vision.md).

### Architecture

The system follows [`ARCHITECTURE.md`](../../ARCHITECTURE.md):

- `lean/` owns the reference definitions and local proof obligations;
- `crates/collatz-engine/` owns pure MVP generators, classical execution,
  metrics, checked `u128`, BigInt, and hybrid promotion;
- `crates/collatz-experiments/` owns input definitions, provenance, controls,
  configuration, and result records;
- `crates/collatz-cli/` owns the minimal local interface;
- `research/` owns the human logbook, plans, and small result summaries.

Mathematical definitions constrain all executable layers. The experiment layer
uses the public engine contract and does not duplicate the Collatz map.

### Mathematical contract

The exact definitions of the classical step, iterates, trajectory, steps to
one, accelerated odd step, compressed iteration and weights, peak, first
descent, verified bound, and generators are solely authoritative in
[`docs/mathematical-definitions.md`](../../docs/mathematical-definitions.md).

Externally visible reports distinguish:

- one classical transition;
- one accelerated odd step;
- one compressed iteration with a classical weight;
- any future jump covering multiple compressed iterations.

These terms are never used interchangeably. A runner stops on observing `1`
before applying another transition. Every execution has finite declared limits.

### Engine contract

- The standalone reference engine uses checked `u128` arithmetic and reports a
  typed arithmetic-overflow outcome without changing counters.
- The arbitrary-precision engine uses `rug::Integer` and applies the same
  mathematical and accounting semantics exactly.
- The hybrid policy promotes the current value before a nonrepresentable odd
  transition and continues without repeating or dropping the transition.
- On every completely representable prefix, reference and BigInt results agree
  on values, classical counts, peak, first descent, and termination.
- Compression, when used, preserves classical checkpoint correspondence, total
  classical count, limits, and peak accounting.

Public result/error semantics are behavioral. Rust signatures may be refined
idiomatically without changing these outcomes.

The checked reference, exact BigInt, and hybrid slices are implemented in
[`reference.rs`](../../crates/collatz-engine/src/reference.rs),
[`bigint.rs`](../../crates/collatz-engine/src/bigint.rs), and
[`hybrid.rs`](../../crates/collatz-engine/src/hybrid.rs), with public
domain/result types in
[`domain.rs`](../../crates/collatz-engine/src/domain.rs). Compressed execution
remains pending under a later PBI.

### Input definitions

The MVP recognizes:

- a positive decimal integer literal;
- an ordered list of accepted definitions;
- Mersenne `2^p - 1`, with `p >= 1`;
- Fermat `2^(2^k) + 1`, with `k >= 0`;
- repunit `(b^d - 1)/(b - 1)`, with `b >= 2`, `d >= 1`;
- `a * 2^m - 1`, with `a >= 1`, `m >= 1`.

Each definition carries a stable ID, name, family, exact construction and
parameters, source metadata, external ID if relevant, retrieval date when
external, derived bit length and decimal digits, and an imported-value SHA-256
when applicable. Unsupported or internally inconsistent definitions are
rejected rather than guessed.

### Experiment configuration and controls

Every experiment has a stable experiment ID and an exact configuration ID.
Configuration includes ordered inputs, schema version, engine policy, limits,
metrics, optional verified-bound reference, deterministic control algorithm and
seed, output version, and program commit.

The default comparison control matches each special input's bit length and uses
a comparable declared sample size. Algorithm, version, seed, mapping, and
rejection rules are stored so controls can be regenerated exactly.

Version 1 pins `ChaCha20` from `rand_chacha 0.10.0`, the mapping name
`sha256-subseed-little-endian-mask-v1`, both-parity sampling, and
equality-before-duplicate rejection. The byte-level mapping is authoritative in
[`docs/experimental-methodology.md`](../../docs/experimental-methodology.md).
Configuration IDs hash canonical validated configuration JSON together with
the selected validated catalog definitions in configured order. This binds the
identity to construction, provenance, and declared metadata rather than only
their stable IDs. Canonical plans exclude run IDs and time so repeated
materialization is byte-identical.

### Results

The MVP writes one versioned JSONL-compatible record per observation. A record
contains identifiers, reconstructed input metadata, engine policy, limits,
termination status, completed counts, complete or prefix-labeled metrics,
timing, promotion count, program commit, and validation state.

Allowed experiment statuses are `reached_one`, `reached_verified_bound`,
`step_limit_reached`, `time_limit_reached`, `resource_limit_reached`,
`engine_error`, `invalid_input`, and `verification_failed`. `engine_error`
means the selected engine could not execute the valid input, while its precise
cause remains in `engine_outcome`; it is not a successful completion and does
not itself invalidate reconstructed input provenance.

Large integers and trajectories remain outside Git and are addressed by
SHA-256 plus metadata. Exceptional observations remain
`needs-reproduction` in research metadata until the independent confirmation
procedure succeeds.

The PBI-004 runner executes classical step limits through reference, BigInt,
or hybrid policy. Version-1 schema fields and result statuses reserve verified
bounds plus time and resource limits, but a non-null unsupported limit is
rejected before execution rather than ignored.

### Formal verification boundary

Lean 4 defines the reference model, proves mathematical properties and
optimization equivalences, checks or generates reviewed small vectors, and
records assumptions. The planned obligations include parity/positivity,
accelerated and compressed correspondence, classical counter preservation,
peak handling, generator definitions, the `a * 2^m - 1` identity, and its
Mersenne corollary.

Every future mathematical optimization needs a theorem connecting it to the
reference model. Lean does not prove universal termination or the compiled
software stack.

### Constraints

- The initial platform is macOS Apple Silicon with scalar CPU execution.
- Mathematical semantics and Lean obligations precede optimization.
- `u128` arithmetic is checked; BigInt arithmetic is exact.
- Production Rust crate roots forbid unsafe code.
- Observable behavior and this Spec change together.
- Benchmarks run only after proof and correctness gates and never serve as
  correctness oracles.
- Important results are verified through at least two independent paths.

## Contract

### Definition of Done

- [x] Single, list, and supported-generator inputs are accepted and validated.
- [x] The checked reference engine implements classical steps, finite limits,
  counts, first descent, and peak semantics without wrapping.
- [x] BigInt and hybrid execution preserve the same semantics, and promotion
  occurs before overflow.
- [x] Fixed examples `1`, `2`, `3`, and `27` use independent expected values.
- [x] Lean builds with no `sorry`, principal theorems do not depend on
  `sorryAx`, and small examples match the mathematical authority.
- [x] Common-domain and promotion differential tests pass.
- [x] MVP generators reconstruct their declared values and provenance.
- [x] Deterministic matched controls can be regenerated from configuration.
- [x] A stopped experiment reports its exact status and never presents a prefix
  metric as a complete trajectory metric.
- [x] Versioned result records contain enough configuration and provenance for
  reproduction.
- [x] Exceptional results remain `needs-reproduction` until independently
  confirmed.
- [x] The implemented engine portion of the Minimal Quality Gate passes with
  exact command evidence in
  [`PBI-002`](../../tasks/PBI-002-rust-reference-engine.md) and
  [`PBI-003`](../../tasks/PBI-003-arbitrary-precision-engine.md), with the
  experiment slice recorded in
  [`PBI-004`](../../tasks/PBI-004-experiment-catalog.md).

### Regression guardrails

- The classical map, stopping convention, and classical step-count convention
  do not change without human approval.
- Start `1` terminates in zero steps with peak `1`.
- The start and all successfully reached classical values participate in peak
  accounting.
- A transition counter increments only after its result is represented.
- Compression does not overshoot a classical limit or omit skipped peak
  candidates.
- The reference engine neither wraps nor silently promotes.
- The BigInt engine does not narrow intermediate values.
- Promotion does not duplicate or lose a transition.
- A finite checked set is never described as a proof of the conjecture.
- Number source and construction remain attached to every result.
- Control-generation seed without algorithm/version is insufficient for
  reproducibility.
- Benchmarks cannot override mathematical or behavioral evidence.

### Scenarios

```gherkin
Feature: Correct and reproducible Collatz experiments

  Scenario: Input one is already complete
    Given the positive input 1
    And a classical step limit of 0
    When the trajectory is evaluated
    Then the status is reached_one
    And the classical step count is 0
    And the last value and peak are 1

  Scenario: Input two has one classical step
    Given the positive input 2
    And a classical step limit of at least 1
    When the trajectory is evaluated
    Then the observed values are 2 and 1
    And the classical step count is 1
    And the peak is 2

  Scenario: Input three matches the independent oracle
    Given the positive input 3
    And a classical step limit of at least 7
    When the trajectory is evaluated
    Then the observed values are 3, 10, 5, 16, 8, 4, 2, and 1
    And the classical step count is 7
    And the peak is 16

  Scenario: Input twenty-seven matches the independent oracle
    Given the positive input 27
    And a classical step limit of at least 111
    When the trajectory is evaluated
    Then the status is reached_one
    And the classical step count is 111
    And the peak is 9232

  Scenario: A small Mersenne definition is reconstructed
    Given a Mersenne definition with exponent 5
    When the input is validated
    Then its value is 31
    And its construction is recorded as 2^5 - 1

  Scenario: A small Fermat definition is reconstructed
    Given a Fermat definition with index 2
    When the input is validated
    Then its value is 17
    And its construction is recorded as 2^(2^2) + 1

  Scenario: A small repunit definition is reconstructed
    Given a repunit definition with base 10 and length 3
    When the input is validated
    Then its value is 111
    And its base and length remain in provenance

  Scenario: An affine power-of-two definition is reconstructed
    Given a definition a * 2^m - 1 with a equal to 3 and m equal to 4
    When the input is validated
    Then its value is 47
    And both parameters remain in provenance

  Scenario: Rust agrees with a Lean-approved vector
    Given a small trajectory vector checked by Lean 4
    When the same start is evaluated by the Rust reference engine
    Then every expected checkpoint, classical count, and peak agrees

  Scenario: Bounded and arbitrary-precision engines agree
    Given a positive input and finite classical step limit
    And every value in the observed prefix is representable as u128
    When both engines evaluate the input
    Then values, counts, peak, first descent, and termination agree

  Scenario: The reference engine detects overflow safely
    Given an odd positive u128 value whose 3n + 1 result is not representable
    When one reference transition is requested
    Then arithmetic overflow is reported at the current value
    And no transition is counted
    And no wrapped or saturated value is produced

  Scenario: Hybrid execution promotes before overflow
    Given the same overflow-risk value under the hybrid policy
    When the next transition is requested
    Then the current value is promoted before the arithmetic operation
    And the exact 3n + 1 result is represented
    And exactly one classical transition and one promotion are counted

  Scenario: Compressed execution preserves classical accounting
    Given a positive input and a classical step budget
    When compressed iterations are used
    Then their weights sum to the reported classical transition count
    And the budget is not overshot
    And skipped intermediate peak candidates are included

  Scenario: A sourced number retains provenance
    Given a valid externally sourced number definition
    When its experiment result is written
    Then the source, external identifier, retrieval date, construction, and input ID are present
    And an imported value includes its SHA-256

  Scenario: Matched controls are deterministic
    Given a special input, control algorithm version, sample size, and seed
    When controls are generated twice from the same configuration
    Then both ordered control sets are identical
    And every matched control has the special input's bit length

  Scenario: An experiment resumes from the same configuration
    Given a stored experiment and configuration ID
    When the experiment is run again with that exact configuration
    Then inputs, controls, engine policy, limits, and metric definitions are identical
    And the new run receives a distinct run ID

  Scenario: An invalid number definition is rejected
    Given a generator definition outside its documented parameter domain
    When input validation runs
    Then the status is invalid_input
    And no Collatz transition is attempted

  Scenario: An exceptional result awaits independent confirmation
    Given a result that exceeds the declared candidate threshold
    When the initial run completes
    Then the result is labeled needs-reproduction
    And it is not labeled a record or counterexample
    And that label remains until the independent reproduction procedure succeeds

  Scenario: A verified bound stops only under its recorded contract
    Given an inclusive verified bound with source and version metadata
    And an executed checkpoint inside that bound
    When the configured early-stop policy is applied
    Then the status is reached_verified_bound
    And the reached value, bound, source, and prefix metrics are recorded
    And no total stopping time or full peak is inferred without verified suffix data
```

## Quality attributes

### Correctness

Checked arithmetic, exact BigInt arithmetic, independent fixed examples, Lean
local proofs, differential tests, and mutation analysis protect the core.
Overflow, invalid input, and limits are explicit outcomes.

### Reproducibility

Inputs are reconstructible, controls are deterministic, configurations are
identified, results name the program commit, and large artifacts carry hashes.

### Auditability

Definitions, ADR rationale, living state, task deltas, run metadata, and
logbook observations remain separate and linked. Failed or invalidated
experiments are retained.

### Performance

The MVP favors clarity and correctness. Performance is measured on declared
Apple Silicon configurations only after correctness gates. No unmeasured
throughput target is part of the MVP.

### Portability

The supported baseline is macOS Apple Silicon. Rust is portable in principle,
but GMP build/link behavior and other platforms are outside the acceptance
contract until explicitly added.

## Minimal Quality Gate

The first complete reference engine requires, in order:

```bash
(cd lean && lake build)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --all-features
cargo llvm-cov --package collatz-engine --lib --all-features --fail-under-lines 90
cargo mutants --file crates/collatz-engine/src/reference.rs
```

The mathematical core has at least 90% line coverage. Whole-workspace coverage
is reported without a global hard threshold. Mutation scope is only the
reference module, with every material survivor explained. Exact policy and
evidence rules live in
[`docs/quality-strategy.md`](../../docs/quality-strategy.md).

## Verification strategy

1. Build Lean and review theorem statements against the mathematical authority.
2. Run formatting and Clippy with warnings denied; production crate roots use
   `#![forbid(unsafe_code)]`.
3. Run independent known-value unit tests and branch/boundary tests.
4. Run property tests for positivity, parity, accounting, and limits.
5. Run reference/BigInt and hybrid/BigInt differential tests.
6. Enforce core coverage and report workspace coverage.
7. Run constrained mutation testing and classify every material survivor.
8. Build benchmarks only after correctness checks.
9. Apply the independent reproduction procedure to exceptional results.

PBI closure reports the commands actually executed and observed results; “tests
pass” is not sufficient.

## Experimental methodology

Experiments follow
[`docs/experimental-methodology.md`](../../docs/experimental-methodology.md):
stable IDs, preregistered primary metrics where appropriate, bit-length-matched
controls, deterministic seeds and algorithms, explicit censoring, typed
statuses, and independent confirmation. The initial experiment plan is
[`research/planned-experiments.md`](../../research/planned-experiments.md).

## Data provenance

Every number carries its stable ID, family, exact formula, parameters, source,
external identifier where relevant, retrieval date, bit length, decimal digit
count, imported-value SHA-256 when relevant, and reconstruction information.
Every result adds experiment/configuration/run IDs, program commit, engine
policy, limits, format version, and validation state.

Automatic internet ingestion is outside the MVP. External facts are entered as
reviewed source metadata and remain distinguishable from generated values.

## Risks

- Incorrect step or terminal semantics could make all results incomparable.
- Compression may preserve endpoints while corrupting classical counts or
  peaks if weights/intermediates are mishandled.
- Shared defects may make two engines agree incorrectly.
- GMP complicates portability and builds.
- Selection bias, multiple comparisons, control mismatch, dependence, and
  censoring may create misleading family conclusions.
- External sources or verified bounds may drift.
- Large artifacts may become irreproducible if hashes and configuration are
  omitted.
- Performance pressure may encourage premature optimization.

Mitigations are the mathematical authority, local proofs, independent oracles,
checked and exact engines, provenance contract, conservative interpretation,
quality gates, and independent reproduction.

## Deferred scope

### Product and compute

- SIMD, GPU, multithreading, distributed execution, and interval scanning;
- GUI, service API, server database, and automatic publication;
- advanced multi-iteration jumps and a custom BigInt;
- automatic external dataset downloads.

### Quality hardening

- fuzzing, Miri, sanitizers, and executable Gherkin;
- automated traceability matrices and whole-repository mutation testing;
- specialized agent teams, nightly/weekly pipelines, elaborate manifests,
  scorecards, and automatic benchmark publishing.

Deferred items are not implicit omissions and are not MVP completion criteria.

## Traceability

- Vision and interpretation: [`docs/project-vision.md`](../../docs/project-vision.md)
- Mathematics and proof obligations:
  [`docs/mathematical-definitions.md`](../../docs/mathematical-definitions.md)
- Architecture: [`ARCHITECTURE.md`](../../ARCHITECTURE.md)
- Methodology:
  [`docs/experimental-methodology.md`](../../docs/experimental-methodology.md)
- Quality gate: [`docs/quality-strategy.md`](../../docs/quality-strategy.md)
- Decisions: [`docs/adrs/README.md`](../../docs/adrs/README.md)
- Implementation deltas: [`tasks/`](../../tasks/)
