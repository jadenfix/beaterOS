# Cross-Agent Coordination Ledger

This is the **shared, living record** of who is doing what in this repository.
Multiple agents work `jadenfix/beaterOS` in parallel, all authenticating as the
same GitHub account. This ledger is how they avoid collisions and how the
"reviewed by a different agent, never self-merged" rule stays auditable.

**Every agent updates this file** when it: claims a slice, opens a PR, reviews a
PR, or merges a PR. When two agents want the same slice, they resolve it here
and in the PR thread (see `review-protocol.md` → "Deconfliction").

> **Why a file and not just GitHub state?** GitHub's review "Approve" and
> "cannot merge your own PR" checks operate on the *account*. Because all agents
> share one account, GitHub cannot distinguish them. Agent identity, approvals,
> and the author≠merger rule therefore live **here**, at the agent layer, and
> are enforced by `scripts/check-governance.py`.

## Active agents

| Agent id | Lane / mandate |
| --- | --- |
| `codex` | Implementation: the `beater-os-core` kernel + the 17-slice backlog in `docs/implementation-backlog.md`. |
| `claude/multi-agent-pr-review` | Independent review + merge, and the governance/coordination layer (this directory). |
| _(other parallel agents)_ | Append yourself here when you start. |

## Slice / PR ledger

Statuses: `claimed` → `draft-pr` → `in-review` → `changes-requested` →
`approved` → `merged` (or `dropped`). The **Merger** must differ from the
**Author** for any `merged` row; `scripts/check-governance.py` enforces this.

| PR | Title | Author agent | Reviewer agent | Merger agent | Status | final.md refs |
| --- | --- | --- | --- | --- | --- | --- |
| #1 | Bootstrap agent kernel contracts | codex | claude/multi-agent-pr-review | _pending (non-author)_ | approved | §12, §26 |
| _(this PR)_ | Multi-agent governance + coordination + review tooling | claude/multi-agent-pr-review | _pending (non-author)_ | _pending (non-author)_ | draft-pr | §19 Phase 0, §26 process |

## Review log (agent-layer approvals)

Because GitHub "Approve" is unavailable on same-account PRs, approvals are
recorded as COMMENT reviews **and** logged here.

| Date | PR | Reviewer agent | Verdict | Notes |
| --- | --- | --- | --- | --- |
| 2026-07-03 | #1 | claude/multi-agent-pr-review | APPROVE (agent-layer) | §26 invariants verified; 5 non-blocking follow-ups filed in review; not merged (still draft). |

## Open coordination questions

- PR #1 is a draft with unticked re-review/merge boxes. A non-author agent will
  merge once `codex` marks it "Ready for review". Until then it stays open.
- Slices 2 and 3 in the backlog can proceed in parallel once #1 merges; whichever
  agent picks them up should claim them here first.
