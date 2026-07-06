## What does this PR do?

<!-- Describe the beaterOS feature slice and the section(s) of final.md it implements. -->

## Type of change

- [ ] Agent kernel contract
- [ ] Capability or policy enforcement
- [ ] Journal, receipt, or audit trail
- [ ] Sandbox, tool, browser, memory, eval, payment, or model service
- [ ] Bare-metal / native OS direction
- [ ] Performance, compiler, accelerator, or language-boundary work
- [ ] Docs / process only
- [ ] Refactor / internal
- [ ] CI / tooling

## Contract checklist

- [ ] New or changed contracts are versioned, typed, and covered by tests.
- [ ] Side-effecting actions are represented by manifests and receipts.
- [ ] Capability checks happen outside model output.
- [ ] No ambient authority is introduced.
- [ ] `final.md` was not shortened or weakened.
- [ ] Hosted compatibility work preserves the path to native beaterOS services.

## Bare-metal / optimization packet

<!-- Required for bare-metal, close-to-metal, performance-sensitive, compiler/runtime, accelerator, or language-boundary work. Use "N/A" only when the PR is clearly outside this scope. -->

- Workload / scenario:
- Replay command:
- Bottleneck class:
- Baseline:
- Target budget:
- Profile / trace artifact:
- Compiler/runtime/backend versions:
- Authority boundary preserved:
- Copy/allocation/syscall/queue/device budget:
- macOS path and fallback:
- Native OS migration criterion:
- Regression gate:
- Independent reviewer for performance + authority:

## Tests

- [ ] `cargo fmt --check`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets`
- [ ] `TMPDIR=/private/tmp python3 scripts/local-e2e.py`

## Review routing

- [ ] Reviewed by an agent/person who did not author the PR.
- [ ] Merge performed by an agent/person who did not author the PR.

## Notes for reviewers

<!-- Risks, follow-ups, and any areas needing deeper review. -->
