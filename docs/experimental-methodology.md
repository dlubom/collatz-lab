# Experimental Methodology

- **Status:** Accepted authority for Collatz Lab experiments
- **Mathematical terms:** [`mathematical-definitions.md`](mathematical-definitions.md)
- **Minimal storage rules:** [`../research/results/README.md`](../research/results/README.md)

## Unit of analysis

An experimental observation is the result of applying one versioned experiment
configuration to one reconstructible positive integer. A family experiment is
a declared collection of such observations plus a declared aggregation and
control-comparison rule.

Exploratory and confirmatory runs must be distinguishable. Changing the input
selection, primary metric, limit, control algorithm, or seed creates a new
configuration identifier; it does not silently amend an existing run.

## Input and provenance contract

Every input record must provide:

| Field | Requirement |
|---|---|
| Stable input ID and name | Unique inside the catalog and readable by humans |
| Family | A declared category such as `mersenne` or `manual` |
| Construction | Exact formula plus all generator parameters |
| Source kind | Explicitly `local` or `external` |
| Source | Citation or URL when external; `user-supplied` when manual |
| External ID | OEIS or database identifier when one exists |
| Retrieval date | Required for externally obtained or changeable data |
| Bit length | Recomputed from the constructed value |
| Decimal digits | Recomputed from the constructed value |
| SHA-256 | Required when a value is imported rather than generated locally |
| Reconstruction note | Enough information to rebuild the value without copying a huge decimal expansion |

Every external source requires a retrieval date, even when no external ID
exists. An imported value must be external and additionally requires a
nonempty external ID and matching SHA-256. The constructed value is validated
against declared bit length, decimal digit count, and imported hash before
execution. Malformed structure yields `invalid_input`; disagreement with
reconstructed metadata or hash yields `verification_failed`. Neither is
repaired silently.

## Experiment configuration

A configuration has a stable experiment ID, schema version, ordered input IDs,
construction parameters, selected engine policy, step/time/resource limits,
optional verified-bound reference, metric set, control specification, output
format version, and expected program-source SHA-256. The expected hash must
equal the stable content hash of the executable-source snapshot embedded during
the build; a mismatch stops before execution. The hash is independent of commit
identity, so squash or rebase does not invalidate an unchanged source snapshot.
Results use the embedded value and expose whether executable-source paths had
Git worktree changes when compiled. The canonical serialized form determines a
configuration ID by SHA-256.

Git commit identity is not part of the configuration or result contract. A
logbook may retain it as an auxiliary locator; uncommitted executable-source
changes are instead exposed by `program_source_dirty`.

An experiment ID identifies the research question, while a configuration ID
identifies an exact executable setup. Repeating a configuration creates a new
run ID and retains the same configuration ID.

## Matched random controls

Whenever a special input or family is compared statistically, the default
control group must have:

- exactly the same bit length for each matched input;
- a comparable and explicitly recorded number of observations;
- a declared deterministic pseudorandom algorithm and version;
- a recorded seed;
- a deterministic mapping from `(experiment ID, input ID, replicate index)` to
  generated control values;
- the same engine policy, limits, metric definitions, and hardware class used
  for the special inputs.

For bit length `b`, controls are sampled from `[2^(b-1), 2^b - 1]`. Rejection
rules such as excluding even numbers, duplicates, or members of the target
family must be declared before generation because each changes the control
population. The MVP default includes both parities and rejects only duplicates
within the control set and exact equality with its matched special value.
Version 1 permits from 1 through 4096 controls per matched input. A larger
request is rejected before allocation; fallible reservations convert allocation
failure into a typed control-generation error.
The same version permits at most 16384 total observations, computed with
checked arithmetic as `input_count * (1 + controls_per_input)`. Plan and result
aggregation use fallible reservations and return typed errors rather than
relying on an infallible allocation.

A recorded seed is insufficient by itself: the pseudorandom algorithm, output
mapping, rejection order, and implementation version are also part of the
configuration.

### Version-1 deterministic mapping

The implemented version-1 control contract uses `ChaCha20` from
`rand_chacha 0.10.0` and a 32-byte lowercase-hexadecimal master seed. For every
zero-based `(experiment ID, input ID, replicate index)` mapping, it derives the
ChaCha seed as SHA-256 over these bytes in order:

1. ASCII `collatz-lab-control-v1` followed by one zero byte;
2. the 32 decoded master-seed bytes followed by one zero byte;
3. UTF-8 experiment ID followed by one zero byte;
4. UTF-8 input ID followed by one zero byte;
5. the replicate index as an unsigned 32-bit big-endian integer.

Each replicate starts its own ChaCha stream. A candidate of bit length `b`
uses `ceil(b / 8)` stream bytes as a little-endian integer. Unused high bits in
the final byte are cleared and the highest retained bit is set, producing a
value in `[2^(b-1), 2^b - 1]` without modulo bias. The stream advances on a
rejection. Version 1 admits both parities, first rejects equality with the
matched special value, and then rejects duplicates already accepted for that
matched value. The stable mapping name is
`sha256-subseed-little-endian-mask-v1`; any change requires a new name and
configuration ID.

Version-1 configuration IDs are lowercase SHA-256 of compact UTF-8 JSON in
declared struct-field order. The hashed identity document contains the
validated configuration and the full selected validated catalog definitions
in `input_ids` order. Construction, provenance, and declared metadata therefore
remain identity-bearing even when an `input_id` and reconstructed value do not
change. The identity excludes run identifiers and clock data. Canonical plans
add reconstructed ordered controls but likewise exclude run-specific state.
Repeating a plan therefore produces identical bytes, while executing it creates
a distinct run ID.

## Metrics

The authoritative formulas are in
[`mathematical-definitions.md`](mathematical-definitions.md). Each result records
whether a metric is complete, prefix-only, unavailable, or derived.

1. **Classical steps to one:** total count of classical transitions, only for
   `reached_one` or when an independently verified suffix supplies an exact
   count.
2. **Compressed iterations:** count of compressed checkpoints actually
   executed.
3. **First descent:** first classical index whose value is below the start.
4. **Trajectory maximum:** maximum over the completed trajectory or explicitly
   labeled observed prefix.
5. **Peak ratio:** exact or reproducibly rounded maximum divided by start.
6. **Bit-length gain:** peak bit length minus input bit length.
7. **Initial growth-run length:** consecutive strictly increasing compressed
   checkpoints from the start.
8. **Execution time:** monotonic elapsed time for the declared timed region;
   never a cross-machine invariant.
9. **Engine transitions:** count of numeric-representation promotions; expected
   to be zero or one for the MVP hybrid policy.
10. **Termination status:** one status from the controlled vocabulary below.
11. **Reached verified bound:** endpoint and cited bound used for any certified
    early stop.
12. **Configuration ID:** identifier of the exact experimental setup.

The MVP result/logbook summary tracks only: experiment ID, input name and
provenance, exact construction, input bit length, classical step count when
known, peak value or bit length, first descent, termination status, elapsed
time, program-source SHA-256, and validation method. The remaining metrics may be
present in the line-oriented result schema but are not required in the first
human summary.

### Normalized views

For a completed or consistently censored comparison, report as applicable:

- classical steps divided by input bit length;
- a declared logarithm of the peak divided by input bit length;
- difference from the matched-control median and position relative to declared
  control quantiles;
- rank inside the declared family and sample.

Do not compare completed values with censored prefix metrics as though they
were equivalent. Quantiles, ties, missing observations, and small samples must
be reported explicitly. No statistical significance test is mandated in the
MVP.

## Termination statuses

| Status | Meaning |
|---|---|
| `reached_one` | The executed trajectory observed `1` |
| `reached_verified_bound` | Execution entered a cited independently verified interval under the safe-stop contract |
| `step_limit_reached` | The step limit was reached before another terminal condition |
| `time_limit_reached` | The declared time limit stopped execution |
| `resource_limit_reached` | A declared memory or other resource limit stopped execution |
| `engine_error` | The selected engine could not execute an otherwise valid input; `engine_outcome` records the precise cause |
| `invalid_input` | The number definition or value violates the input contract |
| `verification_failed` | Reconstruction, hash, cross-engine, or independent verification disagreed |

Arithmetic overflow or an out-of-range input in the bounded reference engine
uses `engine_error`; it is an engine outcome, not a successful experiment
status. A reference-only run records the cause explicitly; the hybrid policy
promotes before arithmetic overflow and records the promotion. A `validated`
research state confirms that record structure, input reconstruction, and
provenance checks succeeded; it does not turn an `engine_error` into a completed
trajectory.

## Safe use of a verified bound

Early termination at a verified bound is permitted only when the bound record
states an inclusive positive interval, identifies its external source and
version/date, and is itself stored with enough provenance to be audited. The
current exact value must fall inside that interval after a successfully checked
transition.

The result records the reached value, bound, source, and executed prefix. It
must not report a total stopping time or full-trajectory peak unless the cited
evidence also provides a trustworthy suffix with those exact metrics and the
composition has been independently checked. Entering a verified interval is
finite computational evidence, not a proof of the conjecture.

## Reproduction and exceptional-result procedure

A potentially record-setting, contradictory, or otherwise exceptional result
is marked `needs-reproduction` in research metadata and processed in order:

1. rerun the same configuration and compare the complete result record;
2. run the checked reference engine wherever every observed value is
   representable;
3. run the arbitrary-precision engine over the complete relevant prefix;
4. check with an independent script or external tool that does not reuse the
   tested algorithm;
5. preserve the complete configuration, program-source SHA-256, toolchain, platform,
   and relevant environment information;
6. record SHA-256 hashes for configuration and large artifacts;
7. perform a manual review of source provenance, step accounting, peak, and
   termination semantics;
8. only after agreement, change the label from candidate to reproduced within
   the precisely declared population.

Failure at any step yields `verification_failed` or remains
`needs-reproduction`; disagreements are retained and investigated rather than
overwritten.

## Reproducible reporting

Every result line points to an experiment ID, configuration ID, run ID, input
ID, build-embedded program-source SHA-256, executable-source dirty state, engine policy,
status, validation state, and format version.
Small text summaries may be versioned. Large values or trajectories remain
outside Git and are represented by metadata plus SHA-256 as described in
[`research/results/README.md`](../research/results/README.md).

CLI diagnostics are not experiment termination statuses. `invalid_input`
identifies malformed or unsupported input data, including invalid UTF-8;
`verification_failed` identifies reconstructed metadata or provenance/hash
disagreement; and `io_error` identifies a filesystem read or write failure,
including an I/O error surfaced by JSON serialization. Plan and result output
use the same mapping.
Contradictory provenance fields, such as a local source carrying external
retrieval metadata or an imported value declared as local, are consistency
disagreements and therefore use `verification_failed`.

Timing comparisons require the same machine description, power mode, build
profile, toolchain, workload, warm-up policy, and competing-load notes. See
[`benchmarking.md`](benchmarking.md).

## Methodology risks

- selection bias from choosing notable numbers after seeing their trajectories;
- multiple comparisons across many families and metrics;
- control mismatch caused by parity or generator rejection rules;
- dependence between members of a mathematical family;
- censoring caused by unequal limits;
- non-independent validation when both implementations share the same defect;
- timing noise and machine-specific performance;
- drift in external databases and verification bounds.

The logbook records deviations, invalidated experiments, and analysis choices
so these risks remain visible.
