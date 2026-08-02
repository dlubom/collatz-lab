# PBI-004: Deliver the First Reproducible Experiment Catalog

- **Status:** Planned; blocked on PBI-003 and dependency approval
- **Type:** Small vertical experiment slice

## Goal

Implement the minimal declarative catalog, provenance validation, deterministic
bit-length-matched controls, line-oriented results, and local CLI needed to run
the first reproducible comparative experiment without building a database or
network ingestion system.

## Motivation

Correct engines are not sufficient for research. Inputs, controls, limits, and
results must retain enough provenance to reproduce an observation and to
distinguish a completed trajectory from a censored prefix or unverified
candidate.

## Dependencies

- Blocked by: PBI-003.
- Blocked by: explicit human approval of exact pinned serialization,
  deterministic-random, and SHA-256 dependencies. The proposed minimal set is
  `serde`, `serde_json`, `rand_chacha`, and `sha2`; no dependency is added merely
  because it appears here.
- Must merge before: any non-oracle research experiment or external data import.

## Context pointers

Read:

1. [`specs/collatz-engine/spec.md`](../specs/collatz-engine/spec.md), input,
   experiment, results, provenance, and experiment scenarios
2. [`docs/experimental-methodology.md`](../docs/experimental-methodology.md)
3. [`docs/mathematical-definitions.md`](../docs/mathematical-definitions.md),
   generators and metric semantics
4. [`research/planned-experiments.md`](../research/planned-experiments.md)
5. [`research/results/README.md`](../research/results/README.md)
6. [`docs/quality-strategy.md`](../docs/quality-strategy.md)

## Scope

- Define version-1 schemas for number definitions, experiment configurations,
  and result lines.
- Support positive literals, ordered lists, Mersenne, Fermat, repunit, and
  `a * 2^m - 1` definitions.
- Validate exact construction, parameter domains, bit length, decimal digits,
  source metadata, and imported SHA-256 when applicable.
- Maintain a small manually reviewed catalog with no internet download.
- Generate deterministic controls with exactly matched bit length from a pinned
  algorithm/version, seed, sample size, mapping, and rejection policy.
- Run one input/list/generator configuration through the existing public engine
  API and write one versioned JSONL record per observation.
- Materialize a canonical experiment plan so the same configuration reproduces
  identical ordered inputs and controls; reruns receive distinct run IDs.
- Add a minimal local CLI for catalog validation, plan materialization, and
  experiment execution.
- Deliver EXP-001 as the known-value validation run and EXP-002 as a small
  Mersenne-versus-matched-control comparative smoke experiment.

The version-1 catalog contains exactly ten reviewed records: literals `1`, `2`,
`3`, and `27`; Mersenne exponents `5`, `7`, and `13`; Fermat index `2`; decimal
repunit length `3`; and `N(3,4)`. EXP-001 uses the four literals with a classical
step limit of `200`. EXP-002 uses the three Mersenne values, eight controls per
value, a classical step limit of `10000`, primary metric first descent, and the
32-byte hexadecimal seed
`c011a72ab5e1d002c011a72ab5e1d002c011a72ab5e1d002c011a72ab5e1d002`
with the approved `ChaCha20` algorithm/version and the MVP rejection policy.

## Out of scope

- Network fetching, scraping, a server database, GUI, or web service.
- Huge values or full trajectories committed to Git.
- Statistical significance claims, automatic record publication, or automatic
  verified-bound updates.
- Additional generator families, distributed execution, concurrency, SIMD, or
  GPU.
- Elaborate manifests, dashboards, or scorecards.

## Concrete files

```text
Cargo.toml
Cargo.lock
crates/collatz-experiments/Cargo.toml
crates/collatz-experiments/src/lib.rs
crates/collatz-experiments/src/number.rs
crates/collatz-experiments/src/catalog.rs
crates/collatz-experiments/src/controls.rs
crates/collatz-experiments/src/config.rs
crates/collatz-experiments/src/result.rs
crates/collatz-experiments/src/runner.rs
crates/collatz-experiments/tests/catalog_contract.rs
crates/collatz-experiments/tests/reproducibility.rs
crates/collatz-cli/Cargo.toml
crates/collatz-cli/src/main.rs
schemas/number-definition-v1.schema.json
schemas/experiment-config-v1.schema.json
schemas/result-v1.schema.json
catalog/inputs-v1.jsonl
experiments/EXP-001.json
experiments/EXP-002.json
```

Small validated summaries may be added under `research/results/` only after the
PBI's test/smoke runs complete and must follow its README.

## Small tasks

1. Obtain approval and pin the minimum dependency versions.
2. Define schemas and Rust validation types with version fields and typed
   errors.
3. Implement pure reconstruction for the four generator forms and literals.
4. Add the reviewed catalog and validate all derived metadata.
5. Implement deterministic control generation and canonical plan
   materialization.
6. Implement versioned result serialization and explicit complete/prefix metric
   labels.
7. Add CLI commands `catalog validate`, `experiment plan`, and `experiment run`.
8. Add contract and reproducibility tests, including invalid definitions,
   repeated plan generation, resume semantics, provenance, and hashes.
9. Run EXP-001 and the deliberately small EXP-002 smoke comparison; label any
   unusual observation `needs-reproduction` without interpreting it as a
   record.
10. Record run IDs, commit, validation method, and concise results in the
    logbook/result summary.

## Acceptance criteria

- [ ] Single literal, ordered list, and each supported generator definition can
  be validated and executed.
- [ ] Invalid domains or inconsistent declared metadata yield `invalid_input`
  before engine execution.
- [ ] Every number/result carries the required provenance fields.
- [ ] Imported values require a matching SHA-256; locally generated values are
  reproducible from formula and parameters.
- [ ] Two plans from the same configuration produce byte-identical ordered
  inputs and controls.
- [ ] Every control has the matched special input's bit length and follows the
  declared duplicate/equality rejection policy.
- [ ] A seed is stored together with algorithm, version, mapping, and rejection
  rules.
- [ ] Rerunning one configuration keeps the configuration ID and creates a new
  run ID.
- [ ] Result lines distinguish complete metrics from prefix metrics and use the
  controlled termination statuses.
- [ ] EXP-001 reproduces the fixed known values.
- [ ] EXP-002 is small, deterministic, and stored/reported without a global or
  causal claim.
- [ ] Exceptional output is `needs-reproduction` until the documented
  independent procedure succeeds.
- [ ] No large value/trajectory, network fetcher, database, or automatic
  publication is introduced.
- [ ] Rust, Lean, coverage, and reference mutation gates still pass.

## Deterministic verification commands

After dependency approval, run from the repository root:

```bash
(cd lean && lake build)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --all-features
cargo llvm-cov --package collatz-engine --lib --all-features --fail-under-lines 90
cargo mutants --file crates/collatz-engine/src/reference.rs
cargo run -p collatz-cli -- catalog validate catalog/inputs-v1.jsonl
cargo run -p collatz-cli -- experiment plan experiments/EXP-002.json --output /tmp/collatz-exp002-a.json
cargo run -p collatz-cli -- experiment plan experiments/EXP-002.json --output /tmp/collatz-exp002-b.json
cmp /tmp/collatz-exp002-a.json /tmp/collatz-exp002-b.json
cargo run -p collatz-cli -- experiment run experiments/EXP-001.json --output /tmp/collatz-exp001-results.jsonl
cargo run -p collatz-cli -- experiment run experiments/EXP-002.json --output /tmp/collatz-exp002-results.jsonl
git diff --check
```

Expected results:

- Lean, Rustfmt, Clippy, tests, threshold coverage, CLI commands, `cmp`, and diff
  checks exit `0`; workspace coverage and mutation analysis report evidence.
- A nonzero mutation-tool exit is acceptable only for an independently accepted
  equivalent or tool-limited survivor, which the closure report names and
  explains.
- Catalog validation reports exactly `10` version-1 records and zero invalid
  records.
- Both materialized EXP-002 plans are byte-identical and `cmp` exits `0`.
- EXP-001 result lines reproduce the fixed known counts and peaks.
- EXP-002 writes exactly `27` valid result lines (three special inputs plus
  twenty-four controls) with the same configuration ID and distinct
  run/result IDs.
- The repository stays free of unreviewed large artifacts.

## Risks

- A PRNG seed may appear reproducible while algorithm or rejection order drifts.
- Canonical serialization may include nondeterministic map ordering or run time.
- Manual JSON handling could weaken validation; dependencies require explicit
  approval rather than an unsafe shortcut.
- Result fields may conflate prefix and complete metrics.
- The first comparison may invite post-hoc interpretation from a tiny sample.
- A catalog source may drift or be copied without reconstruction evidence.

## Completion conditions

Dependencies are explicitly approved and pinned, schemas and behavior match the
Spec, plans reproduce byte-for-byte, fixed examples and the small comparison run
complete, quality gates remain green, and the logbook records actual run IDs and
validation without overstated claims.

## Independent review

The reviewer reconstructs one value of every generator type, independently
checks one control's bit length and deterministic word mapping, validates a
SHA-256 example, compares two materialized plans, checks result
complete/prefix labels, and verifies that EXP-002 interpretation stays within
its declared sample.

## Logical commit boundaries

1. `build(experiments): add approved serialization and control dependencies`
2. `feat(experiments): add versioned input and provenance contract`
3. `feat(experiments): add deterministic controls and configuration IDs`
4. `feat(experiments): add result records and minimal CLI`
5. `test(experiments): add catalog and reproduction contracts`
6. `research: record EXP-001 and EXP-002 smoke runs`

## Refinement protocol

Internal parsing and serialization structure may be refined if canonical output
is preserved. Stop for human review before adding dependencies, changing schema
meaning, random-control population, status vocabulary, result interpretation,
network access, or storage architecture. Observable changes update the living
Spec and methodology in the same change; significant architectural changes
receive an ADR.
