# Agent Roles & Lanes

Parallel agents share this repo. To keep their write-scopes disjoint (per
`final.md` §19 "Parallelism" and the backlog's parallelism note), each agent
works a **lane**. Lanes are advisory but claiming one in the
`coordination-ledger.md` prevents two agents editing the same files.

## Lanes

| Lane | Owns (write scope) | Notes |
| --- | --- | --- |
| **Kernel / contracts** | `crates/beater-os-core/**`, `Cargo.*` | Currently `codex` (PR #1) and the backlog slices 1–17. |
| **Governance / review** | `docs/governance/**`, `scripts/check-governance.py`, `CONTRIBUTING.md`, `CODEOWNERS` | This lane. Owns the review/merge process + coordination ledger. |
| **Docs / spec** | `README.md`, `final.md`, `docs/*.md` (non-governance), `LICENSE` | Issues #2–#9 live here (README, glossary, threat model, split, license). |
| **CI / tooling** | `.github/workflows/**`, `.github/*` | Shared; coordinate before editing (PR #1 seeds `ci.yml` + PR template). |

## Rules for staying disjoint

1. Before editing a file outside your lane, check the ledger for an open PR that
   touches it. If one exists, comment there instead of racing it.
2. If your work needs a change in another lane, prefer a **new file** in your
   lane over editing a file another PR already modifies (avoids merge conflicts
   across parallel branches).
3. Cross-lane dependencies go through the merged `main`, not through another
   agent's unmerged branch — do not build on top of an open PR unless you own it.

## Current assignment (keep in sync with the ledger)

- `codex` → Kernel / contracts (PR #1 + backlog).
- `claude/multi-agent-pr-review` → Governance / review (this PR) + independent
  review/merge of others' PRs.
- Docs / spec and CI lanes are **open** — an agent picking up issues #2–#9 should
  claim the Docs lane in the ledger first.
