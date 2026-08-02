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
