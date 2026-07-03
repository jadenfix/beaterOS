# beaterOS multi-agent Deep PR Review (DPR) protocol

This document defines *how* beaterOS PRs are reviewed and merged when multiple
autonomous agents and humans collaborate on one repository. It expands the
non-negotiable rule in [`AGENTS.md`](../AGENTS.md): **no one merges their own
PR; every change is reviewed and merged by an independent party.**

It exists because `final.md` argues that agent work is only trustworthy when
authority and review live *outside* the actor. We apply that to our own
development process.

## 1. Roles

| Role | Who | Responsibility |
| --- | --- | --- |
| Author | The agent/person who wrote the change | Opens the PR, keeps it scoped, addresses findings. |
| Reviewer | A **different** agent/person | Runs the DPR, records a GitHub review verdict. |
| Merger | A **different** agent/person than the author | Merges only after an independent `APPROVE`. |

The reviewer and merger may be the same independent party, or two different
independent parties. The only hard constraint is that **neither is the author**.

An agent satisfies "a different party" by spawning a sub-agent that receives the
diff fresh and reasons independently. The point is independence from authorship,
not a specific tool.

## 2. Lifecycle of a PR

```
author: branch -> implement -> local checks -> open PR (template)
              -> claim slice in docs/agent-coordination-log.md
reviewer: DPR against docs/review-checklist.md -> GitHub review
              (APPROVE | REQUEST_CHANGES | COMMENT)
author: address blocking findings -> push -> request re-review
reviewer: re-review (same independence rule)
merger (not author): confirm APPROVE + green CI -> merge -> update log
```

A PR is **mergeable** only when all of the following hold:

1. At least one independent `APPROVE` (two for high-risk PRs, see §4).
2. No open `REQUEST_CHANGES` from any reviewer.
3. CI is green: both the Rust job and the governance job.
4. The `Review routing` checklist in the PR body reflects reality.

## 3. What a Deep PR Review must cover

Reviewers work through [`docs/review-checklist.md`](review-checklist.md). At
minimum a DPR produces:

- A **verdict**: `APPROVE`, `APPROVE_WITH_NITS`, or `REQUEST_CHANGES`.
- **Blocking findings**: each with `file:line`, the defect, and a concrete
  failure scenario (inputs → wrong outcome). No hand-waving.
- **Non-blocking findings**: nits and suggestions.
- **Test-gap notes**: invariants the change touches but does not test.

Reviewers should verify claimed bugs by re-reading the exact code path, and
prefer a few high-confidence findings over many speculative ones.

## 4. Risk tiers

| Tier | Examples | Approvals required |
| --- | --- | --- |
| Standard | docs, tooling, refactors, additive schema fields | 1 independent approval |
| High | capability issuance/attenuation, policy admission, journal/receipt chain, secrets handling, payment mandates, sandbox escape surface | 2 independent approvals |

Risk tier is set by the author in the PR body and **may be raised by any
reviewer** — never lowered by the author (mirrors `final.md`: risk class can be
raised by policy, never lowered by the agent).

## 5. Handling disagreement and cross-PR conflicts

- If two open PRs write the same files, the authors coordinate in
  [`docs/agent-coordination-log.md`](agent-coordination-log.md) and in PR
  comments; the earlier-opened, closer-to-merge PR usually lands first and the
  other rebases.
- If a reviewer and author cannot agree on a blocking finding, escalate to a
  human maintainer via a PR comment rather than merging past the disagreement.
- Never merge past an unresolved `REQUEST_CHANGES`.

## 6. Automation support

- [`.github/PULL_REQUEST_TEMPLATE.md`](../.github/PULL_REQUEST_TEMPLATE.md)
  carries the routing checklist every PR must fill in.
- `tools/pr_governance_check.py` checks the repo invariants and (on demand) the
  author≠merger routing rule. Run it locally and it also runs in CI via
  [`.github/workflows/governance.yml`](../.github/workflows/governance.yml).
- CI cannot *prove* an independent human/agent reviewed the code (that is a
  social invariant), so the routing checklist and the coordination log are the
  auditable record. Falsifying them defeats the purpose.
