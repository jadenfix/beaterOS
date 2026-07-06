# Optimization Agent Playbook

Status: operating guidance for beaterOS agents working on performance,
language-boundary, compiler, scheduler, runtime, accelerator, and
close-to-metal changes.

This playbook supplements `final.md` and `docs/sota-systems-engineering.md`.
It does not replace measured implementation work. An optimization claim is not
accepted until a reviewer can replay the workload, identify the bottleneck, and
see that authority, receipts, and macOS compatibility were preserved.

## Current Toolchain Discipline

Toolchain facts change. At the start of any performance-sensitive PR, record the
date and the primary source used for the current compiler/runtime version. As of
2026-07-06, official sources show Rust 1.96.1, LLVM 22.1.8, Zig 0.16.0 with
0.17.0-dev snapshots, Swift 6.3.3, Go 1.26.4, and Python 3.14.6. Do not treat
this list as a pin; treat it as proof that agents must verify freshness before
making language or compiler claims.

Rules:

- Use the repo-pinned toolchain for builds unless the PR is explicitly about a
  toolchain change.
- If a newer compiler is claimed to be faster, safer, or required, include the
  release source, local benchmark delta, compatibility result, and rollback
  plan.
- Do not chase nightly/dev compilers for TCB code unless a specific bug,
  target, sanitizer, or backend requires it and the fallback is documented.
- Record CPU architecture, OS version, compiler version, target triple,
  feature flags, and relevant environment variables with every benchmark.
- Treat compiler optimizations as part of the evidence chain. A result without
  command line, profile mode, and input fixture is not evidence.

## Language Boundary Rules

Default to Rust for authority, hot control-plane services, scheduler-facing
paths, model/tool/payment lanes, journals, receipts, memory projection,
conformance tooling that ships with the product, and native IPC.

Use other languages only for a named reason:

- C: stable ABI, driver, hypervisor, kernel/platform API, sandbox primitive,
  existing high-quality C library, or measured hot-path interop. Wrap it in a
  safe Rust API with explicit ownership and failure invariants.
- C++: vendor SDK, browser/embedder integration, compiler/runtime extension, or
  existing library where replacing it would add more risk than isolating it.
  Keep templates, exceptions, RTTI, allocation ownership, and thread ownership
  out of the authority boundary unless reviewed explicitly.
- Assembly: hardware entry, register work, context-switch stub, atomics,
  syscall veneer, or vetted cryptographic/platform primitive only.
- Zig: freestanding build experiments, cross-compilation glue, or isolated
  low-level probes. Do not make it a TCB language before the toolchain stability
  and reviewer depth are proven.
- Swift: Apple-native UI or platform integration where Swift is the platform
  boundary. Keep policy and receipts in Rust.
- Go: non-TCB infrastructure daemons where fast iteration and static deploys
  matter more than lowest-latency ownership control. Do not use it for the
  kernel authority path.
- Python: bounded audit, validation, and research scripts only.
- TypeScript: UI, browser ergonomics, and agent authoring only. It is a client
  of beaterOS authority, not the authority boundary.
- CUDA, Metal, SYCL, XLA, MLIR, shader languages, and vendor graph compilers:
  accelerator kernels and compilation backends behind beaterOS admission,
  scheduling, memory, telemetry, and receipt contracts.

When proposing a boundary, answer: why this language, why not Rust, what crosses
the boundary, who owns memory, how errors propagate, how cancellation works, how
the boundary is fuzzed or property-tested, and how macOS remains supported.

## Bottleneck Taxonomy

Before editing code, classify the suspected bottleneck. Pick the first matching
class, because lower classes often vanish after the higher class is fixed.

1. Contract work: unnecessary action, receipt, model call, serialization pass,
   hash, scan, retry, or approval loop.
2. Algorithm: wrong complexity, repeated lookup, missing index, unbounded
   fan-out, missing batch, poor cache key, or avoidable recomputation.
3. Data layout: cold fields in hot structs, pointer chasing, allocation churn,
   poor locality, oversized records, or branch-heavy enums in tight loops.
4. Copy/encoding: JSON churn, clone storms, string formatting, buffer growth,
   host-device transfer, DOM/screenshot duplication, or needless compression.
5. Syscall and IO: fsync count, stat/read/write loops, process spawn,
   descriptor churn, network round trips, DNS, TLS setup, or storage flushes.
6. Concurrency: lock contention, wakeup storms, queue backlog, priority
   inversion, cancellation delay, async task leaks, or retry amplification.
7. Scheduler and platform: CPU affinity, context switches, NUMA, page faults,
   power state, thermal throttling, timer granularity, or kernel feature gap.
8. Accelerator: kernel launch overhead, under-occupancy, model residency miss,
   HBM/VRAM/SRAM pressure, DMA/pinned-memory cost, partition contention,
   precision conversion, batch-size mismatch, or fallback route delay.
9. Provider/runtime: model cold start, token streaming latency, rate limit,
   SDK retry, remote queue, browser engine behavior, or cloud control-plane
   delay.

Each PR should state the class, the measured baseline, the target, and the
artifact that will catch regression.

## Required Optimization Packet

A performance PR needs the smallest packet that proves the claim:

- workload: command, scenario, trace, fixture, or benchmark input
- baseline: current p50/p95/p99, throughput, memory, syscalls, copies, queue
  depth, model/tool calls, device occupancy, or other relevant metric
- budget: explicit success threshold and timeout behavior
- profile: Instruments, `sample`, `spindump`, Rust benchmark output, allocation
  count, syscall count, trace spans, `perf`, eBPF, Xcode GPU tools, Nsight,
  TPU/GPU provider metrics, or equivalent
- change: why the diff attacks the measured bottleneck
- safety: authority boundary, fail-closed behavior, receipt/audit replay, and
  rollback story
- portability: macOS path, Linux path if applicable, feature gate, and fallback
- regression: unit/property/scenario/benchmark/CI gate that would fail if the
  bottleneck or security bug returns

Do not accept "faster" without the baseline and the replay command. Do not
accept "more optimized language" without the boundary and safety packet.

## Agent Workflow

1. Read `AGENTS.md`, `docs/sota-systems-engineering.md`, and this playbook.
2. Name the invariant that must not break.
3. Name the hot path and cold path.
4. Gather a baseline before editing unless the task is only documentation.
5. Spawn subagents only for disjoint review, research, or implementation
   slices. Give each one a concrete file scope or question.
6. Make the smallest change that attacks the measured bottleneck.
7. Add or update a regression artifact.
8. Run the local gate and record what passed.
9. Ask for independent review of the authority and performance claims.

Useful subagent prompts:

- "Find every caller of this hot path and identify which ones are user-facing,
  authority-facing, or test-only."
- "Review this patch only for allocation/copy/syscall regressions and provide
  file:line findings."
- "Review this accelerator plan for host-device copies, queueing, residency,
  cancellation, telemetry, and fallback gaps."
- "Compare the language boundary against the playbook and identify any missing
  ownership, error, cancellation, or macOS story."

## Accelerator Review Packet

GPU, TPU, LPU, NPU, Apple Silicon, media-engine, enclave, and custom ASIC work
must include:

- accelerator class and vendor backend, with a vendor-neutral beaterOS contract
- model/artifact digest, runtime/compiler version, precision, quantization, and
  deterministic seed where meaningful
- memory budget split across host RAM, device memory, pinned memory, HBM/VRAM,
  SRAM, cache, and spill paths
- host-device copy count and bytes, launch count, queue delay, execution time,
  and observed throttling where available
- isolation story: MIG, VM/pod slice, process, sandbox, microVM, device ACL, or
  conservative single-tenant scheduling
- cancellation and timeout behavior that actually releases scarce device work
- data-class and residency policy for weights, embeddings, prompts, traces, and
  outputs
- fallback route when the accelerator is unavailable, overloaded, revoked, or
  too expensive
- receipt fields that prove placement, version, model digest, input/output
  digest, timing, queueing, and observed effects

Vendor SDKs can provide execution. They cannot provide authority.
