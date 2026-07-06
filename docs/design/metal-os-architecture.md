# Metal OS Architecture

Status: design doctrine for separating the hosted runtime, Linux add-on path,
and future metal-touching beaterOS work. This supplements `final.md`; it does
not replace or weaken it.

## Thesis

An operating system built for agent workloads in 2026 should start from scarce
resources, authority, and evidence:

- CPU time, memory, IO, storage, network, accelerators, model calls, money,
  credentials, human review, and attention are scheduled resources.
- Every meaningful side effect is admitted by deterministic policy before it
  happens and replayable from receipts after it happens.
- Natural-language intent, tool descriptions, browser content, and model memory
  are untrusted inputs, not authority.
- The hot path is native, typed, bounded, and measured. Explanations,
  summarization, diagnostics, and product ergonomics stay off the critical path
  unless they are required for safety.

The repo should therefore grow in lanes, not by prematurely replacing Linux or
macOS.

## Lanes

| Lane | Purpose | Examples | Gate |
| --- | --- | --- | --- |
| Compatibility/runtime | Make agent work safe on existing systems now | Rust policy engine, sessions, grants, receipts, journals, sandbox lane, tool registry, model/payment/memory/browser adapters, local daemon, CLI | Contract tests, scenario tests, local e2e, macOS support |
| Linux add-on | Use Linux kernel features where they produce measured wins | cgroups, namespaces, seccomp, LSM/eBPF, XDP, `io_uring`, KVM, microVMs, packaging as a service or distro component | Linux benchmark or security proof, platform abstraction, macOS fallback or explicit future label |
| Metal research | Change OS, hypervisor, driver, firmware, scheduler, memory, IO, accelerator, or hardware boundaries | simulators, hypervisor-backed appliances, library OS, microkernel service, unikernel, narrow driver, hardware-backed isolation | Hosted trace or benchmark proving the host boundary is insufficient, safe fallback, reviewer depth, regression gate |

The compatibility lane is not a compromise. It is how beaterOS discovers which
low-level boundaries need to exist before it commits to a kernel, hypervisor, or
hardware target.

## First-Principles OS Shape

A beaterOS metal target should be decomposed by scarce resource and invariant:

- **Authority kernel:** capability grants, attenuation, revocation, policy
  decisions, approvals, payment mandates, and data-class constraints.
- **Execution scheduler:** sessions, task groups, cancellations, priorities,
  retry limits, queue bounds, model/tool/browser/sandbox lanes, and overload
  behavior.
- **Evidence plane:** append-only journals, receipts, canonical hashes, trace
  spans, redaction-safe bundles, replay, and independent verification.
- **Memory plane:** provenance, sensitivity, expiry, projection, rebuild,
  shared memory, zero-copy handoff, and cold/hot placement.
- **IO plane:** rings, batching, completion queues, backpressure, idempotency,
  filesystem/network origin policy, and crash recovery.
- **Accelerator plane:** CPU SIMD, GPU, TPU, LPU, NPU, Apple GPU/ANE, media
  engines, enclaves, and custom ASICs as schedulable devices behind the same
  admission and receipt contracts.
- **Compatibility plane:** Linux/macOS/container/browser/cloud adapters that
  preserve the same contracts while using host-specific primitives safely.
- **Recovery plane:** journal-before-side-effect, compensation, replay, audit
  repair, quarantine, and incident gates.

The metal lane should move one plane at a time. Broad driver-stack work is not
useful until agent-specific contracts prove what the driver, scheduler, or
hypervisor must enforce.

## Language And Boundary Defaults

Rust is the default for the trusted control plane and performance-sensitive
services. The workspace currently declares Rust `1.93` as its MSRV in
`Cargo.toml`; newer stable Rust releases are freshness checks, not build pins,
unless a PR explicitly updates the MSRV.

Other languages need a named reason:

- C: ABI, kernel/driver/hypervisor/platform API, sandbox primitive, existing C
  library, or measured hot-path interop.
- C++: vendor SDK, browser/embedder integration, compiler/runtime extension, or
  existing library whose replacement is riskier than isolation.
- Assembly: unavoidable hardware entry, context switch, register, atomics,
  syscall veneer, CPU feature probe, or vetted platform/crypto primitive.
- Zig: freestanding or cross-compilation experiments, not trusted authority
  paths until toolchain stability and reviewer depth are proven.
- Ada/SPARK: narrow proof experiments only, with a toolchain, reviewer, FFI,
  macOS, and assurance plan.
- Swift: Apple-native UI or platform integration; policy and receipts stay in
  Rust.
- Go: non-TCB infrastructure services.
- Python: bounded audit, validation, fixture, and research scripts.
- TypeScript: UI, browser ergonomics, and agent SDKs as clients of native
  beaterOS contracts.
- CUDA, Metal, XLA, MLIR, shader languages, and vendor graph compilers:
  accelerator backends behind portable beaterOS admission, scheduling,
  telemetry, and receipt contracts.

Any non-Rust boundary must document ownership, lifetime, error propagation,
cancellation, allocation, threading, ABI, macOS behavior, fuzz/property tests,
and rollback.

## Metal Promotion Packet

Before a subsystem moves closer to Linux-only APIs, kernel code, hypervisors,
drivers, firmware, or bare metal, the PR or design note must include:

- current lane and proposed lane
- hosted workload and replay command
- bottleneck class and measured baseline
- proof that the hosted boundary is insufficient
- authority boundary and evidence preserved
- platform target, simulator or hardware access, and macOS development path
- language/unsafe/FFI boundary review
- copy/allocation/syscall/queue/device budget
- security failure mode and fail-closed behavior
- fallback and rollback
- benchmark, trace, property test, scenario, or CI gate
- non-author reviewer for performance and authority claims

Without this packet, the work stays in the compatibility lane.
