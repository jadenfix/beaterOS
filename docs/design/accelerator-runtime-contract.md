# Accelerator Runtime Contract

Status: vendor-neutral design doctrine for GPU, TPU, LPU, NPU, Apple Silicon,
media-engine, enclave, and future ASIC work. This supplements `final.md` and
`docs/optimization-agent-playbook.md`.

## Principle

Accelerators are OS resources, not magic libraries. beaterOS treats accelerator
work like any other side-effecting scarce-resource action: admitted before use,
bounded while queued and running, cancelable where the backend allows it,
receipted afterward, and replayable enough for audit.

Vendor SDKs may execute kernels or graphs. They do not grant authority.

## Cost Model

Accelerator work must budget the whole path, not just device execution:

- host-to-device and device-to-host copy count, bytes, batching, and placement
- pinned-memory allocation, DMA/IOMMU mapping, page migration, and sync fences
- command-buffer, stream, graph, or kernel launch count
- model, embedding, graph, and memory-pool residency
- queue wait, maximum batch wait, rate-limit delay, and cold-start delay
- precision, quantization, atomic/reduction behavior, and tolerated numerical
  drift
- device partition, container, VM, IOMMU, MIG, SR-IOV-style, or single-tenant
  isolation boundary
- thermal, power, entitlement, and scheduler throttling
- fallback route and whether cancellation is preemptive, cooperative, or
  best-effort

## Portable Job Shape

Every accelerator backend should map to this shape before calling a vendor API:

- `DeviceClass`: `cpu`, `gpu`, `tpu`, `lpu`, `npu`, `apple_gpu`, `apple_ane`,
  `media_engine`, `secure_enclave`, or registered future class.
- `Backend`: vendor/runtime/compiler identity, version, driver/framework,
  target triple or device capability, feature flags, and observability limits.
- `AcceleratorJob`: manifest digest, principal, session, data class,
  model/artifact digest, precision, quantization, batch or streaming mode,
  expected p95/p99, timeout, cancellation token, and fallback route.
- `MemoryPolicy`: host bytes, device bytes, pinned bytes, unified-memory bytes,
  HBM/VRAM/SRAM residency, spill policy, cache key, eviction rule, and
  sensitivity/residency constraints.
- `QueuePolicy`: bounded depth, admission class, priority/fairness rule,
  maximum batch wait, tenant isolation, overload behavior, and retry limit.
- `Telemetry`: enqueue/dequeue/start/finish timestamps, queue delay, launch
  count, copy/map/sync bytes, execution time, occupancy where available,
  throttling, errors, cancellation, and fallback reason.
- `Receipt`: placement, backend version, partition or slice identity when
  available, model/artifact digest, input/output digests, timing, queueing,
  observed side effects, and replay evidence.

If a backend cannot report a field, record the limitation explicitly. Do not
silently treat unknown placement, queue delay, throttling, or copies as zero.

## Bottlenecks To Classify

Accelerator optimization begins by naming the bottleneck:

- launch overhead
- under-occupancy
- host-device copy or synchronization
- model or embedding residency miss
- HBM, VRAM, SRAM, cache, unified-memory, or pinned-memory pressure
- DMA/IOMMU mapping overhead
- precision or quantization conversion
- batch-size or streaming mismatch
- queue wait or provider cold start
- partition contention or noisy neighbor
- fallback route delay
- thermal, power, entitlement, or scheduler throttling
- telemetry opacity

Do not add a GPU/TPU/LPU/NPU path until the workload is actually accelerator
shaped and the transfer, queueing, and residency costs are in the budget.

## Backend Notes

- **CUDA / NVIDIA GPU:** account for streams, kernel launch count, occupancy,
  pinned memory, unified memory page migration, device/host copies, driver and
  toolkit version, and MIG or other partitioning when used. Small repeated
  kernels should consider graph/command reuse only after launch overhead is
  measured.
- **ROCm / AMD GPU:** account for HIP/ROCm version, kernel occupancy,
  host-device copies, memory pool behavior, partitioning/isolation limits, and
  fallback to CPU or another backend. Environment-variable device hiding is not
  an isolation boundary for untrusted work.
- **Metal / Apple GPU:** account for command buffers, resource storage mode,
  synchronization/fences, Xcode/Instruments/Metal telemetry, unified-memory
  pressure, and framework opacity. Unified memory removes some explicit copies;
  it does not remove bandwidth, cache, RSS, page-migration, or synchronization
  costs.
- **Core ML / ANE / Apple local acceleration:** record when placement is hidden
  by the framework. Provide fallback when ANE/GPU placement is unavailable,
  revoked by the platform, thermally throttled, or not observable enough for the
  risk class. Treat framework partitioning across CPU, GPU, and Neural Engine as
  backend-conditional unless telemetry proves placement.
- **OpenXLA / StableHLO / TPU:** record StableHLO/XLA/PJRT versions, graph
  compile/runtime split, sharding/partitioning, device topology, queue delay,
  roofline or provider metrics, input-pipeline limits, batch effects, pinned
  host-buffer use, and VM/pod isolation.
- **LPU or deterministic inference silicon:** treat low-jitter token generation
  as a distinct backend class, but require provider-independent measurements,
  model/artifact digests, token latency, rate-limit backpressure, queue
  telemetry, cancellation behavior, and fallback.
- **NPU / edge inference:** account for vendor compiler opacity, quantization,
  data residency, local privacy constraints, power envelope, and CPU/GPU
  fallback.
- **Secure enclave / TEE:** treat as a security boundary first and an
  accelerator second. Record attestation, key ownership, side-channel limits,
  rollback/replay behavior, and standard-crypto usage.

## Security Rules

- Admission happens before enqueue.
- Data-class policy controls whether prompts, embeddings, weights, traces, and
  outputs may reside on a device, provider, partition, cache, or unified-memory
  mapping.
- Cancellation and timeout behavior must release scarce work or mark the
  backend as uncertain in the receipt.
- Queue overload must fail closed for security-critical work and degrade
  explicitly for best-effort work.
- Accelerator caches are policy-managed state. Model residency, embedding
  caches, graph compilation caches, and memory pools must have eviction,
  sensitivity, tenant, and receipt rules.
- Side-channel and tenant-isolation limits are part of the review, especially
  for shared GPUs, cloud TPUs, provider LPUs, and local unified-memory systems.
- Device visibility environment variables are developer convenience, not
  security isolation. Untrusted accelerator work needs device ACLs, containers,
  VMs, IOMMU/VFIO-style isolation, hardware partitioning such as MIG where
  available, or conservative single-tenant scheduling.
- Determinism is backend-conditional. Receipts and tests must state the hardware,
  runtime/compiler version, precision mode, seed where meaningful, atomics or
  reduction risks, tolerated drift, and cross-device reproducibility limits.

## Review Checklist

For any accelerator PR, reviewers should be able to cite where the diff or PR
body answers:

- Which device class and backend are used?
- Why is this workload accelerator-shaped?
- What is the baseline CPU or existing backend performance?
- What are p95/p99, throughput, queue delay, launch count, copy bytes, memory
  residency, and throttling targets?
- What crosses host/device or process/provider boundaries?
- How are precision, quantization, seeds, and determinism handled?
- How is tenant isolation enforced beyond environment-variable device hiding?
- How does cancellation drain in-flight device work?
- What happens when the device is unavailable, overloaded, too expensive,
  thermally throttled, or insufficiently observable?
- Which receipt fields prove placement, version, queueing, timing, model digest,
  input/output digest, and fallback?
- What local macOS path exists, even if the target backend is Linux or cloud?
- What regression gate catches performance and authority regressions?
