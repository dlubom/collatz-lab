# PBI-001: Establish the Lean 4 Reference Model

- **Status:** Ready; execution has not started
- **Type:** Formal model and local proofs

## Goal

Create the minimal pinned Lean 4 project that formalizes the mathematical MVP
model, proves its first local equivalences, and checks small examples without
implementing any Rust production code.

## Motivation

The reference definitions and proof obligations must exist before executable
optimization or compression is accepted. This PBI turns the reviewed
mathematical authority into machine-checked statements while preserving the
explicit boundary that Lean does not prove the compiled application or the
Collatz conjecture.

## Dependencies

- Blocked by: None; ADR-002 and the mathematical authority are accepted.
- Must merge before: PBI-002, PBI-003, and any performance work.

## Context pointers

Read in order:

1. [`AGENTS.md`](../AGENTS.md)
2. [`docs/mathematical-definitions.md`](../docs/mathematical-definitions.md),
   especially sections 1–7 and 11–14
3. [`specs/collatz-engine/spec.md`](../specs/collatz-engine/spec.md), Blueprint
   and Contract scenarios
4. [`ARCHITECTURE.md`](../ARCHITECTURE.md), Mathematical and formal layer
5. [`docs/adrs/ADR-002-lean4-verification-boundary.md`](../docs/adrs/ADR-002-lean4-verification-boundary.md)

This PBI is a delta pointer. Mathematical definitions stay in the authority
above and are not copied into comments as a competing source.

## Scope

- Pin a Lean 4 toolchain and create a Lake project under `lean/`.
- Define positive-domain classical step, iteration, reachability, accelerated
  odd step, compressed iteration, and classical weights.
- Define the bounded terminal-before-limit observation needed to prove that
  start `1` completes in zero transitions.
- Prove positivity/parity lemmas, accelerated correspondence, compressed
  checkpoint correspondence, and accumulated classical-step accounting.
- Prove exact peak-candidate handling for one compressed macro.
- Define the four MVP generators.
- Prove the `a * 2^m - 1` accelerated identity for its valid range and derive
  the Mersenne corollary.
- Check small trajectory and generator examples in Lean.
- Keep assumptions explicit and use no `sorry` or equivalent escape hatch.

## Out of scope

- Rust, GMP, CLI, serialization, or experiment catalogs.
- A proof that every positive integer reaches one.
- Proof of compiled-code conformance or performance.
- Multi-iteration polynomial jumps, SIMD, or GPU.
- Large computations or exhaustive interval verification.

## Concrete files

Create only:

```text
lean/lakefile.toml
lean/lean-toolchain
lean/Collatz.lean
lean/Collatz/Basic.lean
lean/Collatz/Iteration.lean
lean/Collatz/Accelerated.lean
lean/Collatz/SpecialForms.lean
lean/Collatz/Generators.lean
lean/Collatz/TestVectors.lean
```

Update documentation in the same change only if implementation discoveries
alter a theorem statement, module boundary, path, or verification command.

## Small tasks

1. Pin Lean and Lake and make an empty namespace build.
2. Add `Basic.lean` with the positive-domain classical map and parity/positivity
   lemmas.
3. Add `Iteration.lean` with iterates, reachability, index/count lemmas, and the
   bounded terminal-before-limit convention.
4. Add `Accelerated.lean` with `nu_2`-based acceleration, compression weights,
   checkpoint equivalence, counter sum, and peak-candidate lemma.
5. Add `Generators.lean` and prove outputs match their formulas and domains.
6. Add `SpecialForms.lean` with the affine power-of-two identity and Mersenne
   corollary.
7. Add `TestVectors.lean` for starts `1`, `2`, `3`, and `27`, including their
   reviewed classical counts and peaks, plus generator examples `31`, `17`,
   `111`, and `47`.
8. Review theorem names and statements line by line against the mathematical
   authority.

## Acceptance criteria

- [ ] `(cd lean && lake build)` exits successfully.
- [ ] Lean code contains no `sorry` and no theorem depends on `sorryAx`.
- [ ] The classical step and runner-terminal convention remain distinct.
- [ ] The bounded observation proves start `1` completes in zero transitions
  even when the step limit is zero.
- [ ] For odd positive `n`, positivity and evenness of `3n+1` are proved.
- [ ] The accelerated step is proved equal to one odd classical step plus the
  exact number of following halvings.
- [ ] A compressed checkpoint is proved equal to the corresponding classical
  iterate and its weight contributes the correct classical count.
- [ ] The one-macro peak lemma accounts for the skipped `3n+1` value.
- [ ] MVP generator definitions match section 11 of the mathematical authority.
- [ ] The `a * 2^m - 1` theorem has the documented hypotheses and range, and
  the Mersenne theorem is a corollary rather than a duplicate proof.
- [ ] Examples `1`, `2`, `3`, and `27` and the four small generator values are
  checked by Lean 4 against literal expected results.
- [ ] A reviewer confirms theorem content, not merely theorem names, matches
  [`docs/mathematical-definitions.md`](../docs/mathematical-definitions.md).
- [ ] No Rust or experiment implementation is introduced.

## Deterministic verification commands

Run from the repository root:

```bash
(cd lean && lake build)
rg -n '\bsorry\b|sorryAx' lean --glob '*.lean'
git diff --check
```

Expected results:

- `lake build`: exit `0`; all declared modules compile.
- `rg`: no output and exit `1`, meaning no prohibited proof escape is present.
- `git diff --check`: exit `0`; no whitespace errors.

The closure report includes the actual toolchain versions and observed command
results.

## Risks

- A theorem may be technically true but weaker than the documented obligation.
- Natural-number division or valuation definitions may hide missing positivity
  premises.
- A computed example may pass without proving the general correspondence.
- Overloaded modules may make future proof review difficult.

Mitigate through narrow theorem statements, explicit premises, module ownership,
and independent statement review against the human-readable authority.

## Completion conditions

All acceptance criteria and commands are evidenced, documentation matches the
implemented formal statements, independent review finds no weakened obligation,
and the PBI closure records its commit(s). Completion authorizes review of
PBI-002; it does not authorize Rust implementation automatically.

## Independent review

The reviewer inspects imports, axioms, theorem bodies, and generated environment
for `sorryAx`; reconstructs at least the accelerated correspondence and special-
form index range on paper; and compares the small examples with the fixed
authority. A green build alone is insufficient.

## Logical commit boundaries

1. `build(lean): pin minimal Collatz project`
2. `proof(lean): add classical iteration and compression model`
3. `proof(lean): add generators special forms and test vectors`
4. `docs: align formal statements and verification evidence` only if document
   refinement is required

## Refinement protocol

Implementation may refine internal Lean names without review. If a proof
requires changing the Collatz map, domains, stopping/counting semantics, public
metrics, or stated obligation, stop and request human approval; update the
mathematical authority and living Spec in the same reviewed change.
