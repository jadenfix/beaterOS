# Work-Claiming Board

The **collision-avoidance protocol** for the agents building beaterOS in
parallel. Claim a disjoint write scope *before* you build. This is the "journal
before side effects" of development (see `AGENTS.md` → Multi-Agent Contribution
& Review Contract).

**This is a protocol, not a live snapshot.** A hand-maintained per-PR table goes
stale within hours on a fast-moving repo, so this file deliberately does **not**
list every open PR. For live state, read the sources that update themselves:

- **Who is building what right now** → the open pull requests
  (`gh pr list` / the PRs tab). Each PR names its lane in its title/body.
- **Who authored / reviewed / merged each PR** → the canonical audit ledger
  [`docs/governance/coordination-ledger.md`](governance/coordination-ledger.md),
  linted by `scripts/check-governance.py`.
- **How work maps to `final.md`** → [`docs/implementation-backlog.md`](implementation-backlog.md).

This board owns only the *rules* for claiming a scope; it is **not** a second
review ledger (`final.md` §22 — one source of truth per concern).

## How to claim a disjoint write scope

1. **Pick a slice** from `docs/implementation-backlog.md` (or a tracked issue).
2. **Scope your writes.** Decide the narrow set of files/paths you will touch.
   Prefer *new files* over editing shared ones (`AGENTS.md`, `README.md`,
   `final.md`, `Cargo.*`, shared workflows, `docs/governance/*`). If you must
   touch a shared file, keep the edit small and localized.
3. **Check for overlap** against the open PRs. If your write scope intersects an
   open PR's, do **not** start — pick another slice, narrow your scope, or
   coordinate on that PR's thread and let the earlier one merge, then rebase.
4. **Announce the claim** by opening a **draft PR early** (per the fleet's
   draft-first rule) with the agent-routing trailer filled in. The draft PR *is*
   your claim — visible to every other agent without a table that rots.
5. **Build small, review independently, merge by a non-author, then delete the
   branch** — freeing your scope for the next agent.

## Merge routing under a shared agent-id

`pr-governance.yml` keys its self-merge guard on the `Author-Agent` string, so a
`claude`-merges-`claude` trips it even across genuinely distinct sessions
(intentional). **Route merges to a distinct id:** `codex` or `human:@jadenfix`
merges a `claude`-authored PR, and a `claude` merges a `codex`-authored one.

## If two claims must touch the same shared file

The later agent waits for the earlier PR to merge, then rebases — rather than
both editing it in parallel. Announce the dependency on both PR threads.
