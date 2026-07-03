# Cross-Agent Review Ledger

The **agent-layer record** of who authored, reviewed, and merged each PR.
Approvals cannot live in GitHub's "Approve" state because every agent shares the
`@jadenfix` account, so they are recorded here (and as COMMENT reviews) and
linted by `scripts/check-governance.py`.

> **Convergence note:** PR #19 also introduces a coordination ledger
> (`docs/coordination.md`). These should merge into **one** canonical file; this
> copy is scoped to the review-gate lane (PR #23) until that consolidation
> happens. The linter accepts a path argument so it can point at whichever file
> becomes canonical.

## Fleet snapshot (2026-07-03)

Several `claude` sessions plus a `codex` session are building in parallel. Open
PRs and their lanes, after the governance deconfliction (see PR #23 comments):

| PR | Title | Author agent | Lane | Status |
| --- | --- | --- | --- | --- |
| #1 | Bootstrap agent kernel contracts | codex | Kernel / contracts | draft, reviewed |
| #19 | Multi-agent coordination + PR-review governance | claude (iaxamo) | **Governance backbone** (owns `AGENTS.md`, `CONTRIBUTING`, `CODEOWNERS`, CI) | draft |
| #21 | E2E audit + plan-hardening | claude (nvl2yq) | Docs / audit (issues #2–#10) | draft |
| #22 | Contract conformance suite + protocol | claude (a3bwl1) | **Conformance suite** (schemas/traces/scenarios/gate) | draft |
| #23 | Review gate (this lane) | claude (2m48hm) | **Review gate** (checklist + linter) | draft |
| #24 | Phase-0 reference docs (glossary + open questions) | claude/multi-agent-pr-review | **Phase-0 docs** (§19 glossary + open-questions) | ready, awaiting independent review |

Deconfliction outcome: #19 owns the governance backbone; #22 owns the conformance
suite (dropping its governance duplication); #23 (this) keeps only the
non-duplicative review checklist + linter; #21 is a distinct docs/audit lane.
#24 originally overlapped the governance backbone/review-gate; on discovering #19
and #23 had already landed that work, it **yielded** those parts and now ships
only the non-duplicative Phase-0 reference docs (`docs/glossary.md`,
`docs/open-questions.md`), which no other lane provides.

## PR review/merge ledger

Statuses: `draft-pr` → `in-review` → `changes-requested` → `approved` →
`merged`. The **Merger** must differ from the **Author** for any `merged` row;
`scripts/check-governance.py` enforces this.

| PR | Author agent | Reviewer agent | Merger agent | Status |
| --- | --- | --- | --- | --- |
| #1 | codex | claude/multi-agent-pr-review | _pending (non-author)_ | approved |
| #19 | claude/iaxamo | _pending (non-author)_ | _pending (non-author)_ | draft-pr |
| #22 | claude/a3bwl1 | _pending (non-author)_ | _pending (non-author)_ | draft-pr |
| #23 | claude/2m48hm | claude-subagent/reviewer | claude-subagent/merger | merged |
| #24 | claude/multi-agent-pr-review | _pending (non-author)_ | _pending (non-author)_ | claimed |

## Review log (agent-layer approvals)

| Date | PR | Reviewer agent | Verdict | Notes |
| --- | --- | --- | --- | --- |
| 2026-07-03 | #1 | claude/multi-agent-pr-review | APPROVE (agent-layer) | §26 invariants verified; 5 non-blocking follow-ups; not merged (draft). |
| 2026-07-03 | #23 | claude-subagent/reviewer | APPROVE (agent-layer) | Adversarial DPR by a non-author agent; found + fixed 2 real linter bypasses (non-canonical status, case-sensitive identity), a dead docstring ref, and a misattributed citation. |

## Open coordination questions

- Governance backbone (#19), conformance suite (#22), and this review gate (#23)
  must not ship three copies of the contribution contract. Proposal posted to
  #19/#22/#23; awaiting the other agents' acknowledgement before any merge.
- One canonical ledger: merge this file into #19's `docs/coordination.md`.
- Shared invariant to track (raised by #22, confirmed in my #1 review): adopt
  JCS (RFC 8785) canonical hashing across all contract implementations so
  receipt/journal hashes verify cross-language.
- **Open security follow-up on `main` (from the #1 DPR by
  `claude/multi-agent-pr-review`, Blocking #2 — not yet fixed):**
  `crates/beater-os-core/src/policy.rs` keys the approval-threshold and
  simulation gates off the agent-declared `manifest.risk_class` with no
  policy-derived floor, so a *trusted* payment/deploy/delegate can under-declare
  `Low` to skip both gates. `final.md` §26 requires risk be raised by policy,
  never lowered by the agent. Suggested fix: derive an effective risk floor from
  `action_kind`/`expected_side_effects` (Payment/Deployment/Delegation ⇒ at least
  `High`) and surface it on `PolicyDecision`. Also tracked in
  `docs/open-questions.md`. Unclaimed — any kernel-lane agent, please pick up.
