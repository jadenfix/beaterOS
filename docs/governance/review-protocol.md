# Multi-Agent Review & Merge Protocol

beaterOS is built by several agents working in parallel on one repository and
**one shared GitHub account**. This document defines how work is claimed,
reviewed, and merged so that no agent reviews-and-merges its own work, no two
agents silently build the same thing, and any reviewer can act on any code.

It complements `docs/implementation-backlog.md` ("Review And Merge Rules") from
the `codex` agent — that file owns the *what* (the slice list); this file owns
the *how* (claiming, deconfliction, review, merge under a shared account).

## The core rule

> A PR is authored by one agent and **reviewed and merged by a different agent.**

The rule is defined at the **agent-identity layer**, not the GitHub-account
layer, because all agents share `jadenfix`. Consequences:

- GitHub **"Approve"** is unavailable ("cannot approve your own pull request").
  Approvals are posted as **COMMENT** reviews that state the reviewer agent id
  and an explicit `Agent-layer verdict: APPROVE`, and are logged in
  `coordination-ledger.md`.
- GitHub **does** permit merging your own account's PR. That is the lever a
  non-author agent uses to enact the merge. The author agent must **not** press
  it. The ledger + `scripts/check-governance.py` make violations visible.
- A spawned **sub-agent** counts as a different agent from its author for review
  and merge, as long as it independently re-derives the verdict (it must not
  rubber-stamp).

## Lifecycle

1. **Claim (draft PR).** Before building, open a **draft** PR whose description
   states *only what you intend to do* and the `final.md` sections it maps to.
   Add a row to `coordination-ledger.md` with status `draft-pr`. This is how
   other agents see your intent.
2. **Deconflict.** Read the other open PRs and the ledger. If your intent
   overlaps another agent's:
   - Comment on both PR threads naming the overlap.
   - Decide who keeps it by *which approach is better* (stricter invariants,
     smaller trusted base, better tests, closer to `final.md`), not by seniority.
   - The agent that drops it updates the ledger to `dropped` and picks
     unclaimed work. Record the decision in both threads.
3. **Build.** Implement the slice. Keep it contract-focused and mapped to
   `final.md`. Run local gates.
4. **Ready for review.** Mark the PR "Ready for review"; set ledger to
   `in-review`. Request a non-author reviewer (ping in the thread / ledger).
5. **Review.** A different agent runs `review-checklist.md`, posts a COMMENT
   review with the sign-off block, and logs it in the ledger. Blocking findings →
   `changes-requested`; else `approved`.
6. **Merge.** A different agent (not the author) merges once `approved` and gates
   are green. Update the ledger row: set the **Merger agent** and status
   `merged`. Delete the branch.

## Deconfliction heuristic ("who takes it")

When two drafts overlap, prefer the version that:

1. Preserves more `final.md` §26 invariants with less code.
2. Has the smaller trusted computing base / attack surface.
3. Has deny-path and adversarial tests, not just happy-path.
4. Is more legible to a reviewer who did not write it.
5. Is further along *only* as a tie-breaker — do not reward a head start over a
   safer design.

The losing agent is not "wrong"; it frees itself to take unclaimed work, which
is the point of the parallel fleet.

## What every agent may do

Per the user directive, **all reviewers have full authority over all code** —
review, request changes, and merge any PR they did not author. Ownership is not
siloed to the author. `CODEOWNERS` reflects this (repo-owner scope), and code
must be written to be understood by a reviewer who did not write it (see
`review-checklist.md` §C).

## Escalation

If two agents cannot agree on deconfliction or a blocking finding, leave the PRs
open, state the disagreement plainly in the threads and the ledger's "Open
coordination questions", and let the human (repo owner) decide. Do not merge past
an unresolved blocking finding.
