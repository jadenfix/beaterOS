# beaterOS Agent Context

Use this file as startup context for Codex, Claude Code, Cursor, Copilot, and
other coding agents working in this repository.

## What beaterOS Is

beaterOS is an agent-first operating-system research and implementation repo.
The source-of-truth product plan is [final.md](final.md). Implementation must
turn that plan into reviewed, measurable, macOS-compatible slices without
shortening or weakening the plan.

The first implementation layer is a Rust workspace with kernel-facing contracts:
agent sessions, capability grants, action manifests, policy decisions, receipts,
and append-only journals. Future runtime work must preserve those contracts as
authority and audit boundaries, not merely as serialization types.

## Repo Shape

- `Cargo.toml` is the Rust workspace.
- `crates/beater-os-core` contains core contracts, policy admission, hashing,
  journal verification, and receipt-chain logic.
- `docs/implementation-backlog.md` maps `final.md` into PR-sized slices and
  review rules.
- `docs/sota-systems-engineering.md` is the performance, language, security, and
  macOS engineering doctrine for this project.
- `.codex/skills/beateros-systems-engineering/SKILL.md` packages that doctrine
  as a reusable Codex skill.
- `CLAUDE.md` and `.cursor/rules/beateros.mdc` keep equivalent guidance close to
  Claude Code and Cursor.

## Non-Negotiables

- Keep PRs scoped and reviewed. Every feature lands through a PR, and no author
  merges their own PR.
- Do not weaken `final.md`. Add clarifying docs or implementation artifacts
  around it unless the user explicitly asks to edit it.
- Treat performance as an architectural property. Identify the hot path, syscall
  budget, allocation budget, copy budget, queue bounds, and p95/p99 target before
  optimizing syntax.
- Treat security as a systems invariant. Capabilities, receipts, policy
  decisions, memory, payments, tools, and model calls must fail closed and be
  replayable from evidence.
- Make macOS work. The repo must build and test on macOS, including Apple
  Silicon. Do not introduce Linux-only assumptions without an abstraction and a
  macOS path.
- Prefer Rust for most implementation. Use C only for stable ABI, boot/platform,
  driver, hypervisor, or measured hot-path interop needs. Use assembly only at
  the hardware boundary. Isolate and review all unsafe code.

## SOTA Systems Engineering Checklist

Before designing or reviewing a substantial change, read
[docs/sota-systems-engineering.md](docs/sota-systems-engineering.md). At minimum,
be able to answer:

- What is the critical path, and what is explicitly off the critical path?
- What are the data ownership, lifetime, and copy rules?
- Which queue, cache, journal, network, filesystem, or model call can back up?
- What is bounded by construction: memory, CPU, IO, tool calls, model spend,
  payment spend, retries, and wall-clock time?
- Which authority boundary does this touch, and what evidence proves it?
- What benchmark, trace, property test, or scenario would catch a regression?
- Why is the chosen language boundary Rust, C, or assembly?

## Common Commands

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
git diff --check
```

## Multi-Agent Contribution & Review Contract

Several agents (Codex, Claude, sub-agents) build this repo **in parallel**.
Read this section before you touch anything. The human owner is `@jadenfix`.

> **One rule above all:** no agent merges its own work. One agent authors,
> a *different* agent reviews, and a *third party who is not the author*
> merges. This is enforced at the agent-identity layer because all agents
> share one GitHub account (see the honesty boundary below).

The process mirrors beaterOS's own safety model from `final.md`: no ambient
authority (§13.2 → no self-merge), policy outside the actor (§8.12 →
enforcement in CI + the linter, not goodwill), journal before side effects
(§4.5 → claim work before building), receipts after (§7.6 → recorded reviews
and merge receipts).

**Where the process lives (one source per concern — do not duplicate):**

- **This section** is the canonical contract summary. `CONTRIBUTING.md` is the
  human-facing entry point that expands it.
- **Review gate** — `docs/governance/` (review checklist + `scripts/check-governance.py`
  linter). A non-author reviewer fills the checklist; the linter fails a
  `merged` row whose merger equals its author.
- **Review/merge audit ledger** (canonical) — `docs/governance/coordination-ledger.md`,
  linted by `scripts/check-governance.py`.
- **Work-claiming board** — `docs/coordination.md`: claim a disjoint write scope
  *before* building so parallel agents don't collide. It is a claiming board,
  **not** a second review ledger.
- **CI enforcement** — `.github/workflows/pr-governance.yml` checks the PR
  routing trailer + no-self-merge and runs the ledger linter; `.github/CODEOWNERS`
  routes review authority to every reviewer, so no file is owned only by its author.

**Lifecycle:** claim (board) → branch `<agent-id>/<slice>` → build (disjoint
scope, small, `final.md` never weakened) → PR (fill the routing trailer) →
independent review (DPR verdict) → independent merge (non-author) → mark the
slice done and delete the branch.

**Agent routing trailer** — every PR body carries this; the CI check reads it:

```
Author-Agent: <agent-id>
Reviewer-Agent: <agent-id or "pending">
Merged-By: <agent-id / "human:@jadenfix" / "pending">
```

`Merged-By` must differ from `Author-Agent` at merge time.

**Honesty boundary (what is and isn't enforced):** GitHub authenticates the
*human account*, not the agent — all agents act as `@jadenfix`, so GitHub's own
"author can't approve their own PR" cannot separate one agent from another.
Agent identity is therefore **attested** (declared in the trailer/ledger),
enforced by convention + the CI structural checks + the linter, **not**
cryptographically. Concretely: the pre-merge CI check is bypassable (an author
can leave `Merged-By: pending` and merge anyway), and the self-merge guard keys
on the `Author-Agent` *string*, so two distinct sessions sharing an id (e.g. two
`claude` sessions) trip it — intentionally. **Route merges to a distinct id**
(`codex` or `human:@jadenfix`). The real gate is **branch protection** on `main`
requiring this check + a CODEOWNERS review; without it the green check is
advisory. Per-agent signing identities (`final.md` §7.1) would upgrade this from
attested to verifiable.
