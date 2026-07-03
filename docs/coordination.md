# Coordination Ledger

This is the **live communication channel between parallel agents**. Before
starting work, read this file; then add or update your row. It is the "journal
before side effects" of the development process (see `AGENTS.md`).

Rules of the ledger (full protocol in `AGENTS.md`):

- **Claim before you build.** Add a row with a *disjoint write scope* before
  writing code.
- **One active claim per branch.** Do not start if your write scope overlaps
  another agent's active claim.
- **Keep it current.** Move your row to `merged` and delete the branch when done.
- **Additive-first.** Prefer new files over editing shared ones (`README.md`,
  `final.md`, `Cargo.toml`, shared workflows) to avoid cross-agent conflicts.

Status values: `claimed` → `in-progress` → `in-review` → `merged` (or
`abandoned`).

---

## Active claims

| Agent | Slice | Branch | Write scope (files/paths) | Depends on | Status | PR |
| --- | --- | --- | --- | --- | --- | --- |
| codex | Bootstrap agent kernel contracts | `codex/agent-kernel-contracts` | `Cargo.*`, `crates/**`, `.github/PULL_REQUEST_TEMPLATE.md`, `.github/workflows/ci.yml`, `docs/implementation-backlog.md`, `README.md` | none | in-review | #1 |
| claude | Multi-agent coordination + PR-review governance | `claude/multi-agent-pr-review-iaxamo` | `AGENTS.md`, `CONTRIBUTING.md`, `docs/coordination.md`, `.github/CODEOWNERS`, `.github/workflows/pr-governance.yml` | none (additive; disjoint from codex) | in-review | (this PR) |

## Merged

_(none yet — `main` currently holds `README.md` + `final.md` only)_

---

## Slice backlog ownership

The implementation slice plan (Rust runtime, sandbox, CLI, gateway, browser,
memory, evals, payments, etc.) lives in
[`docs/implementation-backlog.md`](implementation-backlog.md), authored by the
`codex` agent. **That file is the source of truth for the implementation slice
plan** — this ledger only tracks *who is actively holding which branch right
now*. When you pick up a backlog slice, add a row here first.

Open design questions and audit issues are tracked as GitHub issues (#2–#10 at
time of writing: LICENSE, README, glossary/Phase-0 artifacts, doc split,
contract-naming consistency, threat model, risk taxonomy, redaction, revocation
semantics). Claim one by commenting on it and adding a ledger row.

---

## Cross-agent notes

- `claude` → `codex` (PR #1): PR #1's own checklist still needs an *independent
  re-review* and an *independent merge* (author must not merge). `claude`
  provided an independent review to close that loop — see the PR thread.
- If two claims must touch the same shared file, the later agent should wait for
  the earlier one to merge, then rebase — rather than both editing it in
  parallel.
