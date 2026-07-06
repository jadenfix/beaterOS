## What does this PR do?

<!-- Describe the beaterOS feature slice and the section(s) of final.md it implements. -->

## Type of change

- [ ] Agent kernel contract
- [ ] Capability or policy enforcement
- [ ] Journal, receipt, or audit trail
- [ ] Sandbox, tool, browser, memory, eval, payment, or model service
- [ ] Docs / process only
- [ ] Refactor / internal
- [ ] CI / tooling

## Lane

- [ ] Compatibility/runtime lane
- [ ] Linux add-on lane
- [ ] Metal research lane
- [ ] Not applicable

If this PR claims Linux add-on, kernel-adjacent, accelerator, or metal work,
name the hosted-lane trace, benchmark, profile, or security proof that justifies
that lane choice.

## Contract checklist

- [ ] New or changed contracts are versioned, typed, and covered by tests.
- [ ] Side-effecting actions are represented by manifests and receipts.
- [ ] Capability checks happen outside model output.
- [ ] No ambient authority is introduced.
- [ ] Language, FFI, unsafe, accelerator, or platform-specific boundaries are
      justified by measurement or a required platform contract.
- [ ] `final.md` was not shortened or weakened.

## Optimization packet

Fill this out for performance, compiler/runtime, language-boundary,
accelerator, Linux add-on, or metal claims.

- Lane:
- Workload / replay command:
- Bottleneck class:
- Baseline:
- Target budget:
- Profile / trace artifact:
- Compiler/runtime/backend versions:
- Authority boundary preserved:
- Copy/allocation/syscall/queue/device budget:
- macOS path and fallback:
- Regression gate:

## Tests

- [ ] `cargo fmt --check`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets`

## Review routing

- [ ] Reviewed by an agent/person who did not author the PR.
- [ ] Merge performed by an agent/person who did not author the PR.

## Notes for reviewers

<!-- Risks, follow-ups, and any areas needing deeper review. -->
