# Contributing to beaterOS

beaterOS is developed by **multiple agents working in parallel** alongside human
contributors. To keep that safe and collision-free, everyone — human or agent —
follows one shared contract.

## Start here

1. **Read [`AGENTS.md`](AGENTS.md)** — the canonical contribution and review
   contract. It defines how work is claimed, reviewed, and merged.
2. **Read [`docs/coordination.md`](docs/coordination.md)** — the live ledger of
   who is working on what. Claim your slice there *before* you start.
3. **Read the relevant section of [`final.md`](final.md)** — the design your
   change implements. Do not shorten or weaken `final.md`.

## The non-negotiable rules

- **No self-merge.** The agent/person who authored a PR never merges it. A
  *different* party reviews, and a party who is *not the author* merges. See
  `AGENTS.md` §3.
- **Independent review is required.** Every PR gets a deep review (DPR) from
  someone who did not write it, with an explicit verdict.
- **Shared ownership.** Every reviewer has full authority over the whole tree —
  no file is owned only by its author. Write code any reviewer can understand
  and change.
- **Claim before you build.** Register a disjoint write scope in the ledger to
  avoid colliding with other agents.
- **Policy outside the actor.** Merge rules are also enforced by CI
  (`.github/workflows/pr-governance.yml`) and `.github/CODEOWNERS`, not by good
  intentions alone.

## Opening a PR

- Branch as `<agent-id>/<slice>` (e.g. `claude/multi-agent-pr-review`).
- Use the PR template and fill in the **Agent routing trailer** truthfully
  (`Author-Agent`, `Reviewer-Agent`, `Merged-By`).
- Keep the change small and contract-focused; link the `final.md` section(s).
- For Rust changes, run `cargo fmt --check`, `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets` locally.

## Filing issues and design questions

Open a GitHub issue for design gaps or audit findings. The `final.md` §20 open
questions are good candidates to turn into tracked issues.

## Trust model, honestly

All agents currently act as the same GitHub account, so GitHub cannot tell one
agent from another. Agent identity is **attested**, and the rules above are
enforced by convention + CI structural checks, not cryptographically. See
`AGENTS.md` §7 for the full honesty boundary and the upgrade path (per-agent
signing identities, `final.md` §7.1).
