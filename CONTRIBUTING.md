# Contributing to beaterOS

beaterOS is developed by multiple agents (and humans) working in parallel on one
repository. This file is the entry point for how to contribute safely. The full
process lives in [`docs/governance/`](docs/governance/README.md).

## Ground rules

1. **Read `final.md` first.** It is the design source of truth. Implementation
   PRs map to named sections of it. `final.md` is **never shortened or weakened**
   to make a PR pass.
2. **Claim before you build.** Open a **draft** PR that states only what you
   intend to do, add a row to
   [`docs/governance/coordination-ledger.md`](docs/governance/coordination-ledger.md),
   then check the other open PRs for overlap and deconflict in the threads.
3. **Author ≠ reviewer ≠ merger.** Every PR is reviewed and merged by a
   *different agent* than the author. See
   [review-protocol.md](docs/governance/review-protocol.md) for how this works
   under a shared GitHub account (GitHub "Approve" is unavailable; approvals are
   COMMENT reviews logged in the ledger).
4. **Code is owned by all reviewers, not just its author.** Write it so a
   reviewer who didn't write it can understand and change it. Any reviewer may
   review, request changes on, or merge any PR they did not author.
5. **No ambient authority, journal before side effects, policy outside the
   model.** The [review-checklist.md](docs/governance/review-checklist.md)
   encodes the `final.md` §26 never-compromise invariants; a reviewer will hold
   your PR to them.

## Local checks

For Rust PRs:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

For governance/process PRs:

```sh
python3 scripts/check-governance.py
```

## Branch naming

Use `<agent-or-user>/<slice>` (e.g. `codex/session-runtime`,
`claude/multi-agent-pr-review`). Delete branches after merge.

## Filing concerns

Open a GitHub issue (see #2–#10 for the current audit backlog) or add an entry to
the ledger's "Open coordination questions". Don't merge past an unresolved
blocking review finding — escalate to the repo owner instead.
