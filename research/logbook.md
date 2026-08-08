# Research Logbook

This chronological log records research decisions and observations. Entries do
not become factual result claims unless they cite a source or a stable run ID.
Do not rewrite old entries to make a failed experiment disappear; append a
correction and link the affected experiment.

When an entry summarizes a run, track only the MVP human-summary fields:
experiment ID; number name and provenance; exact formula or generator
parameters; input bit length; classical steps to one when known; peak value or
peak bit length; first descent when observed; termination status; elapsed time;
program version/commit; and validation method. Rich machine records remain in
the versioned result format rather than expanding the logbook.

## 2026-08-02

### Decisions

- Established the documentation-only project foundation.
- Adopted matched controls by bit length, stable experiment IDs, reproducible
  seeds, and independent reproduction as proposed MVP rules.
- Human accepted the documentation baseline and ADR-001 through ADR-004 for
  commit and publication; this did not start an implementation PBI.

### What we checked

- Reviewed the initial repository documentation and the local ASDLC knowledge
  base before implementation.
- Reconciled the project brief with the living specification, mathematical
  definitions, architecture, ADR proposals, and first four PBIs.

### Results

- No computational experiment was run and no Collatz result is claimed.

### Problems and errors

- The prior documentation used `proofs/` while the requested Lean layout uses
  `lean/`; the proposed documentation standardizes on `lean/`.
- The prior bootstrap PBI combined Rust and Lean scaffolding; the proposed
  backlog separates the formal model, reference engine, BigInt engine, and
  experiment catalog.

### Open questions

- No open decision blocks publication of the documentation baseline.
- Before PBI-004 execution, obtain explicit approval of the exact pinned
  serialization, random-generation, and hashing dependencies.

### Next steps

- Commit and publish the accepted documentation baseline. Start PBI-001 only on
  a separate implementation instruction.

## 2026-08-08

### Decisions

- Approved and pinned `serde 1.0.229`, `serde_json 1.0.151`,
  `rand_chacha 0.10.0`, and `sha2 0.11.0` for PBI-004.
- Fixed the version-1 control mapping and canonical configuration/plan rules in
  the experimental methodology; changing them requires a new version and
  configuration ID.
- Approved `engine_error` as the experiment status for a valid input that the
  selected engine cannot represent or complete because of bounded arithmetic.
- Kept full JSONL outputs outside Git and stored the small reviewable run
  summary in [`results/2026-08-08-exp-001-exp-002.md`](results/2026-08-08-exp-001-exp-002.md).

### What we checked

- Validated exactly ten catalog records and zero invalid records.
- Materialized EXP-002 twice; both 27-input plans were byte-identical with
  SHA-256 `ca3f63790035706d5f3b25771d2f7d0a279e5a4b493ce0927f25ff4293e6edd1`.
- Reconstructed every generator family, independently pinned the first control
  value, checked bit-length/equality/duplicate rules, compared reference,
  BigInt, and hybrid results, and verified complete-versus-prefix labels.

### Results

- EXP-001 configuration
  `86297883123deb36e3a295f79b8d895ce5338512d09ed9017a1622c0b2295a34`,
  run
  `run-4f28a2fb16e4bf4b7a0bed5481001cf5326c9904bac5e52bc3ad6acb41a2fc54`:
  four `reached_one` records reproduced the fixed counts and peaks.
- EXP-002 configuration
  `4bb88941efdbdb3e52acff0f09cae270fa7ab2c06c68cb519de7a44c9cbc0a33`,
  run
  `run-452b78372f0ff87fc4ae848adae432bece93c2a222e52a59c54f5723561336c7`:
  27 `reached_one` records were written for three Mersenne values and 24
  matched controls.
- Program commit for both configurations was
  `8c629f901b2aed607e68533e6eb9f327931ba6c5`.
- No candidate threshold was declared and no observation was classified as
  exceptional. The smoke comparison supports no record, global, causal, or
  significance claim.

### Problems and errors

- Initial workspace tests exposed that relative catalog paths depend on the
  CLI working directory; integration fixtures now use explicit catalog paths
  without changing the root-invocation CLI contract.
- Schema-reserved time/resource limits and verified-bound execution are not
  silently ignored; version 1 rejects non-null values until an owning PBI
  implements them.
- Independent review found that the first configuration-ID implementation
  hashed selected `input_id` values without their catalog definitions. The ID
  now also binds the selected construction, provenance, and declared metadata;
  a regression test changes provenance without changing the numeric value.
- Independent review also corrected the completed input `1` first-descent
  label to `unavailable`, separated bounded-engine failures into
  `engine_error`, and replaced a misleading Fermat-index-zero invalid-domain
  fixture with valid `F0` and exact reconstruction-boundary coverage.
- Follow-up PR review found three additional reproducibility/robustness gaps:
  configuration could self-declare an unchecked program SHA, control sample
  count was not operationally bounded, and plan-output I/O was mislabeled as
  `invalid_input`. Build provenance, the version-1 maximum, fallible
  reservations, and the `io_error` CLI category now close those gaps.

### Open questions

- None for PBI-004. Any interpretation beyond this declared tiny sample needs a
  separately preregistered experiment.

### Next steps

- Human review and merge remain separate. Future work may design a larger
  Mersenne comparison without reusing this smoke run as confirmatory evidence.

## Entry template

Copy this structure for a new dated entry:

```markdown
## YYYY-MM-DD

### Decisions

### What we checked

### Results

### Problems and errors

### Open questions

### Next steps
```
