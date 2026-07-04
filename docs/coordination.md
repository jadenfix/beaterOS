# Work-Claiming Board

The **collision-avoidance board** for the agents building beaterOS in parallel.
Claim a disjoint write scope here *before* you build. This is the "journal
before side effects" of development (see `AGENTS.md` → Multi-Agent Contribution
& Review Contract).

**This is a claiming board, not a review ledger.** The canonical record of who
*authored / reviewed / merged* each PR — and the `scripts/check-governance.py`
linter over it — lives in
[`docs/governance/coordination-ledger.md`](governance/coordination-ledger.md).
Keep the two separate to avoid a second source of truth (`final.md` §22).

## Rules

- **Claim before you build.** Add a row with a *disjoint write scope*.
- **One active claim per branch.** Don't start if your scope overlaps an active
  claim; pick another slice, narrow scope, or coordinate on the other agent's PR.
- **Additive-first.** Prefer new files over editing shared ones (`AGENTS.md`,
  `README.md`, `final.md`, `Cargo.*`, shared workflows) to reduce cross-agent
  conflicts. If you must touch a shared file, keep the edit small and localized.
- **Release the claim.** Delete the branch and drop your row after merge.

Status: `claimed` → `in-progress` → `in-review` → `merged`/`dropped`.

## Active claims (open branches)

| Agent | Slice | Branch | Write scope | Status | PR |
| --- | --- | --- | --- | --- | --- |
| claude/iaxamo | Governance backbone (contract + CI + CODEOWNERS + claiming board) | `claude/multi-agent-pr-review-iaxamo` | `AGENTS.md` (governance section only), `CONTRIBUTING.md`, `docs/coordination.md`, `.github/CODEOWNERS`, `.github/workflows/pr-governance.yml` | in-review | #19 |
| claude/nvl2yq | E2E audit + plan-hardening docs | `claude/repo-e2e-audit-nvl2yq` | `docs/design/**`, `docs/audit/**`, `docs/glossary.md` | in-review (draft) | #21 |
| claude/qp5d8a | Phase-0 glossary + open questions | `claude/multi-agent-pr-review-qp5d8a` | `docs/glossary.md`, `docs/open-questions.md` | in-review | #24 |
| claude/4cfv9t | `beater-os-audit` verifier crate | `claude/multi-agent-pr-review-4cfv9t` | `crates/beater-os-audit/**` | in-review | #27 |
| claude/vzkjv1 | Grant-constraints fail-closed security fix | `claude/multi-agent-pr-review-vzkjv1` | `crates/beater-os-core/src/contracts.rs`, `crates/beater-os-core/tests/**` | in-review | #30 |
| codex | Repo entrypoint + LICENSE + source audit | `codex/repo-entrypoint-source-audit` | `LICENSE`, `README.md`, `docs/source-matrix.md`, `AGENTS.md`/`CLAUDE.md` (lang policy) | in-review | #36 |

> Overlap watch: #21 and #24 both touch `docs/glossary.md` — coordinate on those
> threads before either merges (later one rebases). #19 and #36 both touch
> `AGENTS.md`, but in disjoint regions (governance section vs language-policy
> line); small, should auto-merge.

## Coordination notes

- **Governance dedup (resolved).** The fleet converged: **#19** owns the
  contribution backbone (`AGENTS.md` governance section, `CONTRIBUTING`,
  `CODEOWNERS`, the CI workflow, this board); **#23** (merged) owns the review
  gate (checklist + linter); **#22/#25** own the conformance suite + contract
  spec. Duplicate governance PRs were dropped/closed (#20, and the governance
  parts of #22/#24). `spec/COORDINATION.md` is a slice-scoped companion.
- **Merge routing under a shared id.** `pr-governance.yml` keys its self-merge
  guard on the `Author-Agent` string, so a `claude`-merges-`claude` trips it even
  across distinct sessions (intentional). Route merges to a distinct id: `codex`
  or `human:@jadenfix` merges `claude`-authored PRs, and a `claude` merges
  `codex`-authored ones.
