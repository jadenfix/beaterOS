# beaterOS Bare-Metal Readiness Program

This document defines the next implementation lane for moving into metal-facing work:
start from contracts, measure where host abstractions saturate, and only then add
new low-level ownership or scheduler boundaries.

## Current lane intent

- Keep the compatibility lane shipping on Linux/macOS from contracts and runtime
  today.
- Add a reusable capability model so every bare-metal-oriented PR maps to a
  machine class and accelerator class budget.
- Keep this lane measurable: each PR has an optimization packet and a host class
  plan with explicit queue/copy/fallback constraints.
- Keep author-review separation: the implementation PR author never merges their own
  PR, and PRs for this lane need a separate reviewer.

## What this file means for implementation

1. If a PR claims bare-metal value, it must include at least one path in the
   readiness manifest that it changes or requires.
2. If a PR changes `docs/sota-systems-engineering.md`, `docs/optimization-agent-playbook.md`
   or `final.md` clauses that affect metal-lane behavior, that PR also updates this
   manifest entry to avoid drift.
3. PRs changing bare-metal pathways must either preserve or tighten one of:
   - authority boundary (`policy`, `admission`, `receipt`, `audit`, `memory`)
   - data movement budget (`copy`, `resident`, `queue`, `serialization`)
   - fallback story (`cpu fallback`, `microVM fallback`, `software control path`)

## Progress policy

- Every PR in this lane:
  - updates `docs/engineering/bare-metal-readiness-manifest.json` if machine-class
    assumptions change,
  - adds or updates a test for manifest schema or host-class validation logic,
  - lands with a non-author reviewer and non-author merge.
- This repository ships this with local e2e via
  `scripts/check-bare-metal-readiness.py`, so changes cannot be merged without
  passing at least manifest validation and report synthesis.

## Manifest contract (for this slice)

- `schema_version`: integer contract version; new changes require migration review.
- `profiles`: list of hardware or platform profiles used for planning.
- Each profile defines:
  - `name`, `scope`, `stability_tier`, `target_os`, `target_arch`
  - `resource_contract` envelope with hard minimum resource assumptions
  - `accelerators` entries with required and fallback semantics
  - `optimization_targets` with concrete measurable limits for p95 and throughput.

## Language and optimization baseline

Use Rust for control plane/kernel-like code. Use C only for stable ABI,
platform/driver boundaries, or measured hot-path interop after profiling. Use Python
for this manifest and readiness checker slice because its purpose is to keep the
e2e program honest and lightweight.

## Next concrete slice in progress

- Add script-backed manifest validation (`scripts/check-bare-metal-readiness.py`).
- Add local-e2e gate to ensure the readiness contract is always checked.
- Extend tests for schema shape + host-compatibility checks.
- Keep the manifest authoritative for any machine-class claim in this lane.
