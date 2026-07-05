# Design Spec: Capability Revocation Semantics

Status: documents the **shipped** revocation model and specifies the two narrow
gaps that remain. Audit/plan-hardening lane (PR #21). Grounded in
`crates/beater-os-core/src/{contracts.rs,policy.rs}`.

> Reconciliation note (2026-07-05): an earlier draft claimed "delegated
> propagation is missing", "no `parent_grant_id`", and "`revocation_handle` is
> unused." Those are now **stale** — the delegation chain, a revocation registry,
> and transitive liveness all ship. This revision credits the shipped design and
> narrows the spec to the two things that are genuinely still missing:
> **in-flight pre-commit re-check** and a **journaled revocation event**.

## 1. What ships today

`CapabilityGrant` (`contracts.rs`) carries:

- `parent_grant_id: Option<String>` — the grant this one was attenuated from; a
  delegated grant is authority *indirected* through its parent.
- `revocation_handle: String` — the registry key used to revoke this grant.
- `revoked: bool` and `is_active_at(now)` — local liveness (expiry + revoked).

Admission (`policy.rs`) provides a `revoked_handles: BTreeSet<String>` in the
`AdmissionContext` and enforces liveness over the **whole ancestor chain** via
`grant_chain_effectively_active`:

```
walk grant → parent → … :
  fail closed if a cycle is seen (visited set)
  fail closed if !current.is_active_at(now)  OR  revoked_handles.contains(current.revocation_handle)
  fail closed if a named parent_grant_id is missing from the grant set
  succeed when a root grant (no parent) is reached still-live
```

So, already correct and enforced:

- **Revocation as indirection** — revoking a handle (adding it to
  `revoked_handles`) invalidates the grant without mutating it (§6.2).
- **Transitive/cascade revocation** — revoking a parent's handle fails the whole
  subtree, because every descendant re-checks its ancestors' handles.
- **Delegation-chain liveness** — a child cannot be exercised unless every
  ancestor is live; cycles and dangling parents fail closed.

The §12.2 invariants ("revoked grants fail closed", "delegated grants revocable
through indirection") therefore hold in code today.

## 2. Gap 1 — in-flight actions (pre-commit re-check)

`grant_chain_effectively_active` runs at `admit()` only. A grant revoked *after*
admission but *before/while* an irreversible side effect commits is not
re-checked. For payments/deploys this is the case that matters most.

**Spec:** add a second capability check bound to the manifest's `idempotency_key`,
run by the executing lane immediately before commit:

| Side effect state when revocation observed | Action |
| --- | --- |
| Not yet committed (idempotency key unused) | **Abort**, fail closed; journal an `ActionAborted{reason: revoked}`; no receipt. |
| Already committed (external txn done) | Emit the receipt **plus** run `compensation_plan` (§12.3) as a new, separately-admitted action; journal both. |
| Commit in progress / unknown | Treat as committed; reconcile via the receipt's `external_ids`. |

The pre-commit check reuses `grant_chain_effectively_active` against the current
`revoked_handles`/clock, so it inherits the shipped cascade semantics for free.
Guarantee: *no action is admitted or committed under a grant whose handle — or any
ancestor's — is in `revoked_handles` as of that check.*

## 3. Gap 2 — revocation is not journaled

`JournalEvent` (`journal.rs`) has variants for session/grant/action/policy/
receipt/memory/scenario/incident, but **no revocation variant** — so a revocation
is not causal history and cannot be replayed or audited.

**Spec:** add `JournalEvent::GrantRevoked { revocation_handle, revoked_by:
CapabilityId, reason, scope: grant|session|agent|tool }`, appended and hash-linked
like any event. Revoking a grant does **not** invalidate receipts already produced
under it (historical facts; `ReceiptLedger::verify_chain` unaffected) — revocation
bounds the future, not the past. Incident mode (§13.15) = insert session/agent/
tool handles into `revoked_handles` **and** append the `GrantRevoked` record.

## 4. Delegation invariants (verify these are enforced or add tests)

For a child `c` of parent `p`, in addition to the shipped chain-liveness:

- `c.scope ⊆ p.scope`, `c.denied_actions ⊇ p.denied_actions` (attenuation;
  `DelegationMode::AttenuatedOnly`).
- `c.expires_at ≤ p.expires_at` (a child cannot outlive its parent).
- `p.delegation == None` ⇒ `c` cannot be created.

These are creation-time checks; if not already enforced at grant issuance they
should be added alongside the chain check.

## 5. Follow-up kernel slice (contracts lane)

1. Add a pre-commit re-check hook the executing lanes call before commit, keyed by
   `idempotency_key`, reusing `grant_chain_effectively_active`.
2. Add `JournalEvent::GrantRevoked` and an `ActionAborted` outcome.
3. Confirm/add the §4 creation-time attenuation + lifetime invariants.

Everything else in this lane is already shipped; this is the narrowed remainder.

## 6. Acceptance mapping (issue #10)

- [x] Delegated (parent→child) revocation propagation — **shipped** (§1) and
      documented.
- [x] In-flight action behavior on revoke — specified (§2); pre-commit re-check is
      the remaining kernel work.
- [x] Revocation interacts cleanly with receipts/compensation — §2 (compensation)
      and §3 (receipts stay valid).
