# beaterOS PR Review Checklist

Used by the **reviewer** (a non-author) and the **merger** (a non-author) on
every PR. It exists so that any agent or person can perform a rigorous,
consistent review of code they did not write — the point of the fleet is shared
ownership, not author-locked components.

Reviewing is not rubber-stamping. Try to *break* the change: find the input that
makes it wrong, the invariant it quietly weakens, the file it collides with.

## A. Process gates (block merge if unchecked)

- [ ] The reviewer and the merger are **not** the author.
- [ ] Branch name follows `<agent>/<slice>` and matches the claimed slice.
- [ ] Write scope does not collide with another open PR's files (check open PRs).
- [ ] `final.md` was not shortened or weakened
      (`python3 tools/final_integrity.py` passes).

## B. beaterOS invariants (from final.md — block merge if violated)

- [ ] **No ambient authority.** Every new side effect flows through an explicit
      capability grant; nothing gains authority just by running.
- [ ] **Capability checks live outside model output.** Admission is
      deterministic code, never a model deciding it is allowed.
- [ ] **Manifests and receipts.** Side-effecting actions predeclare an
      `ActionManifest` and produce a `CapabilityReceipt`.
- [ ] **Policy before execution.** A `PolicyDecision` is journaled before any
      action runs; denied/needs-review actions cannot execute.
- [ ] **Fail closed.** Expired grants, revoked grants, missing budgets, unknown
      side effects → denial, not a default-allow.
- [ ] **Data ≠ instructions.** Web/email/doc/tool output is treated as tainted
      data, never as commands.
- [ ] **Contracts honored.** Any object crossing a component boundary validates
      against `contracts/schemas/`; new/changed contracts are versioned and
      typed, and divergences are logged in `contracts/README.md`.

## C. Correctness

- [ ] Try to find a failing input: boundary values, empty collections, missing
      optional fields, wildcard selectors, expiry exactly at `now`.
- [ ] Error paths fail closed and are explainable (denials must state a reason —
      final.md 22.9).
- [ ] Concurrency/ordering assumptions hold (journals are append-only; hashes
      chain).
- [ ] Tests include **negative** cases, not only happy paths.

## D. Quality & legibility

- [ ] Code reads like its neighbors; names and structure are clear to a
      non-author.
- [ ] Public items are documented; non-obvious decisions have a comment or a PR
      note.
- [ ] No dead code, no silent scope creep beyond the claimed slice.

## E. Checks are green

- [ ] Rust slices: `cargo fmt --check`, `cargo test --workspace`,
      `cargo clippy --workspace --all-targets`.
- [ ] Contract/tooling slices: `python3 -m unittest discover -s tests` and
      `python3 tools/contracts_validate.py`.
- [ ] CI on the PR is green (or its absence is understood and noted).

## Verdict

- **Approve** only when A, B, and E fully pass and C/D have no blocking issues.
- **Request changes** with specific, reproducible findings.
- **Escalate to the human owner** for anything ambiguous or architectural rather
  than guessing.

The merger re-confirms A and E immediately before merging, then merges. The
merge is recorded on the PR so the loop is auditable end to end.
