# Language And Toolchain Matrix

Status: current engineering policy plus a 2026-07-06 source snapshot. Version
facts are temporal; verify primary sources again before making a current-version
claim in a PR.

## Local Repo Facts

- The Rust workspace uses edition `2024`.
- `Cargo.toml` declares `rust-version = "1.93"`.
- Workspace lints set `unsafe_code = "forbid"` and deny `unwrap_used` and
  `expect_used`.
- Rust `1.93` is the current MSRV floor for this repo, not a promise that latest
  stable Rust is used in CI.

## Toolchain Policy

Use the repo-declared toolchain floor for builds unless the PR is explicitly
about a toolchain update. A newer compiler, language, or backend is not evidence
of optimization without a replayable local delta.

Every performance, language-boundary, compiler, accelerator, or close-to-metal
claim must record:

- date checked and primary source URL
- local compiler/runtime/backend versions
- CPU architecture, OS version, target triple, feature flags, and relevant env
  vars
- benchmark command, fixture, warmup, sample count, and variance/noise note
- baseline and target p50/p95/p99, throughput, memory, copy, syscall, queue, or
  device metrics
- profile or trace artifact
- bottleneck class
- authority boundary preserved
- macOS path, fallback, rollback, and regression gate

## Language Matrix

| Language/tool | beaterOS use | Gate |
| --- | --- | --- |
| Rust | Default for TCB, policy, receipts, journals, sandbox orchestration, model/tool/payment lanes, daemons, CLIs, and hot native services | Stable Rust, repo MSRV respected, `unsafe_code = forbid` by default |
| C | Stable ABI, platform/kernel/driver/hypervisor/sandbox APIs, existing C library, measured hot-path interop | Small safe Rust wrapper, ownership/failure/cancellation invariants, fuzz/property tests where practical |
| C++ | Vendor SDKs, browser/embedder integration, compiler/runtime extension, existing libraries where isolation is lower risk than replacement | No authority path by default; review exceptions, RTTI/templates, allocation, ABI, threading, failure behavior |
| Assembly | Boot/context-switch stubs, register access, atomics, syscall veneers, CPU feature probes, vetted crypto/platform primitives | Hardware necessity, ABI notes, feature detection, scalar fallback where applicable, property tests |
| Zig | Freestanding experiments, cross-compilation probes, build-system investigations | Not TCB until stability, reviewer depth, macOS behavior, and Rust boundary are proven |
| Ada/SPARK | Narrow formal proof experiments | Toolchain plan, reviewer plan, FFI story, macOS path, evidence Rust cannot satisfy the assurance target |
| Swift | Apple-native UI or platform integration | Policy, receipts, journals, and admission stay in Rust |
| Go | Non-TCB infrastructure daemons or developer tooling | No policy, receipt, journal, or scheduler authority path |
| Python | Bounded audit, validation, local e2e, fixture generation, research scripts | No admission, receipts, journals, sandbox authority, or shipped hot-path services; document dependencies and runtime |
| TypeScript | UI, browser ergonomics, dashboard, agent SDKs | Client of native beaterOS authority, generated contracts, no authority substitution |
| CUDA / Metal / XLA / MLIR / shader languages | Accelerator kernels and graph/compiler backends | Behind beaterOS admission, scheduling, memory, telemetry, receipt, fallback, and data-class contracts |

## 2026-07-06 Source Snapshot

This snapshot is useful for agent awareness, not as a build pin:

- Rust latest stable observed from the Rust release blog: `1.96.1`, released
  2026-06-30. Cargo `rust-version` remains the repo's supported floor signal,
  not a latest-stable pin.
- LLVM latest release observed from LLVM sources: `22.1.8`, released
  2026-06-16. Upstream LLVM facts do not automatically describe Apple Clang or
  vendor compilers.
- Zig official download page lists `0.16.0`.
- Swift official sources list the Swift `6.3` release line in 2026.
- Go official release history lists the Go `1.26` release line in 2026.
- Python downloads list Python `3.14.6`, released 2026-06-10.
- CUDA, Metal, OpenXLA, StableHLO, TPU, and accelerator vendor sources move
  independently from language compilers and must be verified for backend claims.

The maintained citation table for these inputs lives in
`docs/source-matrix.md`.
