# beaterOS Governance

How multiple agents build beaterOS on one repository and one shared GitHub
account without stepping on each other, and without any agent reviewing or
merging its own work.

## Contents

- **[review-protocol.md](review-protocol.md)** — the lifecycle: claim (draft PR)
  → deconflict → build → review → merge, and the author≠reviewer≠merger rule at
  the agent-identity layer.
- **[review-checklist.md](review-checklist.md)** — the concrete gate a non-author
  reviewer runs, derived from `final.md` §26/§12/§13.
- **[coordination-ledger.md](coordination-ledger.md)** — the living record of who
  claimed/authored/reviewed/merged what. Every agent updates it.
- **[agent-roles.md](agent-roles.md)** — lanes that keep parallel write-scopes
  disjoint.
- **[../implementation-backlog.md](../implementation-backlog.md)** — the `codex`
  agent's 17-slice map of `final.md` into PRs (the *what*; this dir is the *how*).

## The one rule to remember

A PR is **authored by one agent and reviewed + merged by a different agent.**
Because all agents share the `jadenfix` account, GitHub can't enforce this — so
it is enforced by convention, by the coordination ledger, and by
`scripts/check-governance.py`.

## Run the governance check

```sh
python3 scripts/check-governance.py
```

Exits non-zero if any `merged` PR was merged by its author, or any in-review PR
lacks a distinct reviewer.
