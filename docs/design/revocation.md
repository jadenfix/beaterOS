# Design Spec: Capability Revocation Semantics

Status: design spec closing issue #10. Audit/plan-hardening lane (PR #21).
Grounded in the shipped `CapabilityGrant` in
`crates/beater-os-core/src/contracts.rs` and admission in `policy.rs`. Specifies
what "revoke" means for **in-flight actions** and **delegated sub-grants** —
both undefined today. Revocation is a §26 never-compromise and the core of
incident response (§13.15 "freeze session, revoke grants").

## 1. What exists, and the two gaps

`CapabilityGrant` today:

```rust
pub struct CapabilityGrant {
    pub grant_id: String, pub issuer: String, pub holder: String,
    pub session_id: String, pub scope: CapabilityScope, /* ... */
    pub expires_at: DateTime<Utc>,
    pub delegation: DelegationMode,     // None | AttenuatedOnly | SameScope
    pub revocation_handle: String,      // present but UNUSED
    pub revoked: bool,                  // local mutable flag
}
impl CapabilityGrant {
    pub fn is_active(&self, now) -> bool { !self.revoked && self.expires_at > now }
}
```

Admission (`policy.rs`) checks `grants.all(|g| g.allows_manifest(manifest, now, actor))`
once, at `admit()` time.

**Gap 1 — delegated propagation is missing.** A delegated grant is a separate
`CapabilityGrant` with its own `revoked` flag and **no `parent_grant_id`**.
Revoking a parent therefore does nothing to its children. `DelegationMode`
constrains *scope at creation* (`AttenuatedOnly`/`SameScope`) but there is no
runtime link, so §8.2/§20.7 ("parent remains accountable", revocable delegation)
and §6.2 ("revoked through indirection") are not enforced.

**Gap 2 — in-flight actions are undefined.** `is_active`/`allows_manifest` run at
admission only. A grant revoked *after* admission but *before/while* the side
effect commits (a payment in flight, a form submitting) is never re-checked. For
irreversible actions this is the case that matters most, and there is no rule.

Also: `revocation_handle` is dead, and there is no `GrantRevoked` journal event
(the `JournalEvent` enum has no revocation variant), so revocation isn't even
recorded as causal history.

## 2. Model: revocation as monotonic indirection (not a per-grant bool)

Replace the local `revoked: bool` semantics with an indirection consulted at
every check — this is what makes revocation *cascade* and *fail closed*.

- A **RevocationSet** (monotonic; handles only ever added) holds revoked
  `revocation_handle`s. Revoking = insert a handle. It never shrinks, so a
  revoked grant can never silently reactivate.
- Grants form a **delegation chain**: add `parent_grant_id: Option<String>`. A
  grant carries (transitively) its own handle plus every ancestor's handle.
- Effective liveness becomes a chain predicate:

```
is_active(grant, now, revset) =
      grant.expires_at > now
   && no handle in ancestors(grant).revocation_handle ∈ revset
   && all ancestors satisfy is_active            // child cannot outlive/outscope parent
```

Revoking a parent's handle now invalidates the whole subtree in one step — the
"indirection" of §6.2 — without walking and mutating every child.

### 2.1 Delegation invariants (checked at grant creation and at use)

For a child `c` of parent `p`:

- `c.scope ⊆ p.scope` and `c.denied_actions ⊇ p.denied_actions` (attenuation;
  `DelegationMode::AttenuatedOnly`).
- `c.expires_at ≤ p.expires_at` (a child cannot outlive its parent).
- `c` active ⇒ every ancestor active (liveness flows down, revocation flows down).
- `p.delegation == None` ⇒ `c` cannot exist.

## 3. In-flight actions: two-phase admission

Split enforcement into two capability checks bound by the manifest's
`idempotency_key`:

1. **Admit** (unchanged, at `admit()`): full policy + capability check. Journaled
   as `PolicyDecided`.
2. **Pre-commit re-check** (new): immediately before the side effect is
   *committed* by the executing lane, re-evaluate `is_active(..., revset_now)` for
   every required grant against the **current** RevocationSet and clock.

Outcomes at pre-commit:

| Side effect state when revocation observed | Action |
| --- | --- |
| Not yet committed (idempotency key unused) | **Abort**, fail closed; journal `ActionAborted{reason: revoked}`; no receipt (no side effect happened). |
| Already committed (external txn done) | Cannot un-happen; emit the normal receipt **plus** run `compensation_plan` (§12.3) as a new, separately-admitted action; journal both. |
| Commit in progress / unknown | Treat as committed (fail safe toward recording), then reconcile via the receipt's `external_ids`. |

The idempotency key guarantees the pre-commit/commit pair is not double-executed
under a retry.

## 4. Consistency and latency guarantee

- The RevocationSet is **monotonic** and every capability check reads the current
  snapshot (a revocation epoch counter is sufficient: checks record the epoch they
  observed).
- Guarantee: *no action is admitted or committed under a grant whose handle — or
  any ancestor's handle — is in the RevocationSet as of that check.* Bounded
  staleness = the gap between admit and pre-commit, which the pre-commit check
  closes for irreversible effects.
- Incident mode (§13.15): "revoke grants / freeze session" = insert the session,
  agent, or tool handles into the RevocationSet. All future admits and all
  in-flight pre-commits then fail closed immediately.

## 5. Revocation is audited; receipts stay valid

- Add `JournalEvent::GrantRevoked { revocation_handle, revoked_by: CapabilityId,
  reason, scope: grant|session|agent|tool }`, appended and hash-linked like any
  event. Revocation becomes causal history, not a silent mutation.
- Revoking a grant does **not** invalidate receipts already produced under it —
  those are historical facts and `ReceiptLedger::verify_chain` is unaffected.
  Revocation bounds the *future*, not the past.

## 6. Follow-up kernel slice (contracts lane)

1. Add `parent_grant_id` to `CapabilityGrant`; enforce §2.1 invariants at
   creation.
2. Introduce a `RevocationSet`/epoch; thread it into `is_active` and
   `allows_manifest` (replace the bare `revoked` bool read with a chain+set
   check).
3. Add a pre-commit re-check hook the executing lanes must call before commit,
   keyed by `idempotency_key`.
4. Add `JournalEvent::GrantRevoked` and an `ActionAborted` outcome.

Contract only; implementation belongs to the kernel lane (codex/#1), consistent
with `risk-class.md` and `journal-redaction.md`.

## 7. Acceptance mapping (issue #10)

- [x] In-flight action behavior on revoke is defined — §3 (two-phase + table).
- [x] Parent→child (delegated) revocation propagation is defined — §2 indirection +
      §2.1 invariants.
- [x] Revocation interacts cleanly with receipts/compensation — §3 (compensation
      for committed effects) and §5 (receipts remain valid).
