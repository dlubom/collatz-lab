# Contribution Workflow

Collatz Lab uses pull requests as the only path into `main`. This applies to
PBI implementation, fixes, documentation, research tooling, and repository
maintenance.

## One-time setup

Activate the versioned Git safeguards in every clone or worktree:

```bash
./scripts/install-git-hooks.sh
```

The installer copies the tracked hooks into this clone's Git metadata and sets
the repository-local `core.hooksPath` to that runtime copy. The guards therefore
remain active while branches are switched, including before this workflow is
present on `main`. It does not modify global Git configuration. Run the
installer again after pulling a change to the tracked hooks.

## Delivery sequence

1. Confirm that the local base matches the current `origin/main`.
2. Create a topic branch named `codex/<description>`. For an implementation
   PBI, use `codex/pbi-NNN-<description>`.
3. Read the sources of truth and active PBI named by [`AGENTS.md`](AGENTS.md).
4. Implement only the authorized delta. Keep living-spec changes with any
   observable behavior change.
5. Run `git diff --check`, the active PBI checks, and every applicable gate in
   [`docs/quality-strategy.md`](docs/quality-strategy.md). Record literal
   commands and observed outcomes when closing a PBI.
6. Review the complete diff and commit only intended files. Do not absorb
   unrelated worktree changes.
7. Push the topic branch and open a draft pull request using the repository
   template.
8. Review the PR against the Spec, mathematical authority, architecture,
   active PBI, tests, and ASDLC practices. Resolve actionable conversations
   and rerun checks on the resulting head.
9. Merge only after human review and an explicit merge decision. Delete the
   topic branch after the merge.

A repository-changing task is complete only when its applicable gates pass and
its intended changes have been committed, pushed, and exposed in a draft PR.
If a required gate fails, the task remains incomplete. An incomplete checkpoint
is pushed only when explicitly requested and must be labeled as such.

## Enforcement layers

| Layer | Enforced rule |
|---|---|
| `.githooks/pre-commit` | Rejects commits made while `main` or `master` is checked out. |
| `.githooks/pre-push` | Rejects any push whose destination is remote `main` or `master`, including `HEAD:main`. |
| `Repository policy` workflow | Tests the tracked hook behavior on every PR and every update to `main`. |
| GitHub `Protect main` ruleset | Requires a PR, an up-to-date passing `Repository policy` check, resolved conversations, and linear history; prevents force pushes and deletion; and has no bypass entries. |

Client-side hooks are an early safety net, not the security boundary: Git can
skip a hook with `--no-verify`. The active GitHub `Protect main` ruleset is the
authoritative boundary for the shared repository. It requires zero approving
reviews so that a sole owner is not deadlocked by GitHub's self-approval rules;
review remains mandatory workflow policy, conversations must be resolved, and
the merge remains a separate human decision. Bypassing either enforcement layer
is not part of the normal delivery workflow.

## Review expectations

A reviewer checks that:

- the change matches the active PBI or explicitly stated maintenance scope;
- mathematical semantics and public result/error behavior have not changed
  without prior approval;
- living specs, proof obligations, implementation, and regression tests remain
  synchronized where applicable;
- checked `u128` and exact BigInt invariants remain intact;
- provenance and complete-versus-prefix labels remain unambiguous;
- verification evidence belongs to the PR head being reviewed;
- no generated, secret, unrelated, or large research artifact entered the
  change unintentionally.

The merge is a separate human decision. Passing automation makes a change
reviewable; it does not authorize an automatic merge.
