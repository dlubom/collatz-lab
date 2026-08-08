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
- Kept full JSONL outputs outside Git and stored the small reviewable run
  summary in [`results/2026-08-08-exp-001-exp-002.md`](results/2026-08-08-exp-001-exp-002.md).

### What we checked

- Validated exactly ten catalog records and zero invalid records.
- Materialized EXP-002 twice; both 27-input plans were byte-identical with
  SHA-256 `bdfa0b95b327c7960cc53c3aa79ec25fe98d0c21afff26d455bb3e7d8621522b`.
- Reconstructed every generator family, independently pinned the first control
  value, checked bit-length/equality/duplicate rules, compared reference,
  BigInt, and hybrid results, and verified complete-versus-prefix labels.

### Results

- EXP-001 configuration
  `60f4e334f1435e58aca4e7ffb36cfe5baefaa5bff8c1e3c58583d84b81dd2afc`,
  run
  `run-db2c2fb14ffa3c8c2cc38fc3656991be2362c9c5ea89559e4037ae2f41d6bb7a`:
  four `reached_one` records reproduced the fixed counts and peaks.
- EXP-002 configuration
  `34dc3a0415bad44c76c1598c70a9a18d4a46e6333511daa4c9acfebebe8d4b6e`,
  run
  `run-54a72cf1bb87e7fcc64841d7a624686b8f157039327fe1c3df3ed4b0953f4f73`:
  27 `reached_one` records were written for three Mersenne values and 24
  matched controls.
- Program commit for both configurations was
  `eaf06d2b43fc7cc1ab2d86c6d0063619a2a304b1`.
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
