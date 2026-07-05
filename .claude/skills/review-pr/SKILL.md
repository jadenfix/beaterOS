---
name: review-pr
description: High-recall, high-precision independent review of a beaterOS PR. Use when asked to review a PR in jadenfix/beaterOS (e.g. "/review-pr 106"). Reviews must be done by an agent that did NOT author the PR.
---

# beaterOS PR review

You are an independent, non-author reviewer for `jadenfix/beaterOS`. The argument is a PR number: `$ARGUMENTS`. Several agents work this repo concurrently — assume nothing about freshness, and never rubber-stamp. This rubric teaches you *how* to find bugs on any PR; it is deliberately not a list of past bugs to grep for.

## Ground rules

- **Non-author only.** Check `gh pr view <N> -R jadenfix/beaterOS --json commits -q '.commits[].messageHeadline'` — if you recognize any commit as your own work from this session, stop and hand the review to another agent. No self-merge is a standing repo rule.
- Read-only: do not modify the main clone, do not run `cargo` in a directory another agent may be building in. CI already builds per-PR; review by reading.
- Precision: every **blocker** carries a concrete traced failure scenario (specific input/state → specific wrong behavior, with `file:line`). If you cannot trace one, it is a nit.
- Recall: read the ENTIRE diff, the referenced issues, and the surrounding code of every touched file at current `main`. Bugs live at the seams the diff doesn't show.

## Procedure

1. `gh pr view <N> -R jadenfix/beaterOS --json title,body,author,files,mergeStateStatus,statusCheckRollup`
2. `gh pr diff <N> -R jadenfix/beaterOS` — all of it.
3. `gh issue view <issue> -R jadenfix/beaterOS` for every referenced issue; the issue defines the intended scope, and `docs/implementation-backlog.md` defines the slice boundaries.
4. **Supersession check:** `git log origin/main --oneline -30` plus targeted `git log -p` on touched files → REJECT (superseded) if main already contains an equivalent fix.
5. **Freshness check:** after any wait, force-push, PR body edit, or CI rerun, re-read PR state, head SHA, base SHA, check rollup, and linked issue state.
6. **Overlap check:** `gh pr list -R jadenfix/beaterOS --state open` — flag open PRs touching the same paths and whether merge order matters.
7. Hunt for bugs using the method below.
8. Post the review (format at the bottom) and return a structured verdict.

## How to find bugs (do this — don't just tick boxes)

- **Trace one path end to end.** Follow one agent request from session and grant lookup through policy decision to receipt and journal — into the deny, expiry, revocation, and malformed-manifest branches, not just the granted path.
- **Review from three seats.** beaterOS serves a **governed agent** (can it do anything its grants don't authorize, or be wrongly blocked by a decision that should be deterministic?), an **auditor** (do receipts and journals prove what actually happened, tamper-evidently?), and a **policy author / integrating runtime** (same inputs ⇒ same decision, contracts stable across versions?). For the code in the diff, ask how it hurts each of the three.
- **Enumerate failure modes** for every new input, call, or state transition: empty · malformed · oversized · slow/hung · repeated/retried · concurrent · out-of-order · partial failure · adversarial/untrusted.
- **Follow the seams the diff hides:** callers of changed signatures, callees now leaned on, invariants elsewhere that assumed the old behavior.
- **Reverted-fix test:** would any test in the PR still pass if the fix were reverted? If yes, it proves nothing — a blocker for a bugfix PR.
- **Adversarially verify** each candidate blocker: try to refute it against the code. Survives → blocker. No concrete trace → nit.
- **Preserve durable lessons** under `Durable guidance`; a follow-up author lands accepted guidance in this file from a separate PR.

## What to look for (general bug classes)

Correctness & honesty of the contract:
- [ ] Return values and decision outcomes tell the caller the truth — a denial, partial grant, or no-op is never reported as full success.
- [ ] Docs match code — no present-tense claims for unimplemented kernel features; the README's "early implementation" honesty is preserved.

Resource, lifecycle & availability:
- [ ] Everything that can grow is bounded: journals have a rotation/compaction story, session and grant tables are bounded, retries are capped.
- [ ] Every external round-trip has a timeout and a recovery path; cleanup runs on all exit paths.

Tests:
- [ ] Tests exercise the actual failure mode (survive the reverted-fix question); boundaries tested at, below, above.

Fit & simplicity:
- [ ] The change is one PR-sized slice doing exactly what its backlog item needs — no speculative abstraction or unused knob.
- [ ] It fits `final.md` (the design source of truth) and respects the non-goals (§21) and threat model; `docs/sota-systems-engineering.md` rules apply to hot-path/systems code.

## beaterOS-specific bug classes (check every one the diff touches)

Deterministic policy (the kernel promise):
- [ ] A policy decision is a pure function of (request, session, grants, policy version): no wall-clock reads, environment lookups, map-iteration-order dependence, or randomness inside decision evaluation. Time-dependent rules take time as an explicit, journaled input.
- [ ] Same inputs replayed ⇒ byte-identical decision and receipt content; anything that breaks decision replay is a blocker.

Authority & grants (narrow, explicit, decaying):
- [ ] No ambient authority: every side effect traces to an explicit grant; composition of grants cannot authorize what no single grant allows.
- [ ] Expiry and revocation are enforced at USE time, not only at issue time; a revoked grant honored anywhere is a critical blocker.
- [ ] Manifests and requests from agents are untrusted input: validated, size-bounded, and unable to smuggle authority through unvalidated fields.

Receipts & journals (the audit spine):
- [ ] Receipts/journals are append-only and tamper-evident; no code path rewrites or deletes committed entries.
- [ ] Every side-effect path emits its receipt — including error and partial-failure paths; a side effect without a receipt is a critical blocker.
- [ ] Journal writes are crash-safe: kill mid-write recovers on restart with no torn or silently lost committed entries.

Contracts & conformance (language-neutral by design):
- [ ] Rust types in `crates/beater-os-core` and the JSON Schemas in `spec/` change in the SAME slice, with conformance vectors updated; drift between them is a blocker.
- [ ] Wire compatibility holds in both directions: old serialized sessions/grants/receipts still deserialize, and defaults-filled values re-serialize compatibly; contract versioning is explicit, not implied.
- [ ] `serde(default)` does not excuse missing updates to existing struct literals and downstream constructors — scan callers.

## Verdict & posting

Post exactly one review:

```
gh pr review <N> -R jadenfix/beaterOS --comment --body "<body>"
```

Body format — first line is the verdict, nothing above it:

```
VERDICT: APPROVE | REQUEST-CHANGES | REJECT (superseded | wrong-approach)

<one-paragraph summary: what the PR does, whether it fixes the traced failure>

Blockers:
- <file:line — traced failure scenario>   (or "none")

Nits:
- <file:line — suggestion>                (or "none")

Durable guidance: <candidate reusable invariant for follow-up docs, or "none">

Overlap: <open PRs touching same paths + merge-order note, or "none">

— independent review agent (non-author)
```

APPROVE only with zero blockers. REQUEST-CHANGES when fixable blockers exist. REJECT when superseded or the approach conflicts with `final.md`. Do not merge — merging is the coordinator's job after CI + mergeability recheck.

## Deep mode (optional)

If asked for a "deep" review, fan out three parallel non-author subagents with distinct lenses — (a) determinism/authority correctness, (b) audit-spine integrity and crash safety, (c) contract-drift/scope — then adversarially verify each candidate blocker yourself before posting.
