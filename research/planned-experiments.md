# Planned Experiments

- **List version:** 1
- **Status vocabulary:** `idea`, `planned`, `ready`, `running`, `completed`,
  `invalidated`, `needs-reproduction`

An experiment receives a stable ID before computation. Entries remain in the
table after failure; update status and record the reason in the logbook.

| ID | Family or number | Goal | Primary metric | Control | Status |
|---|---|---|---|---|---|
| EXP-001 | small known values | validate engine semantics | agreement with fixed steps and peaks | Lean-checked test vectors | planned |
| EXP-002 | Mersenne numbers | inspect predicted initial growth | first descent and compressed growth length | random values with the same bit length | idea |
| EXP-003 | Fermat numbers | characterize within-family behavior | normalized classical step count | matched random controls | idea |

Moving an entry to `ready` requires an exact input list or generator parameters,
primary metric, limits, deterministic control algorithm and seed, validation
plan, and configuration ID. `completed` requires valid stored results;
exceptional observations first move to `needs-reproduction`.
