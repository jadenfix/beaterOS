# Design Spec: Risk Class Taxonomy and Assignment

Status: design spec closing issue #8. In the audit/plan-hardening lane (PR #21).
Grounds the `final.md` §12.3 `risk_class` field in the enum already shipped in
`crates/beater-os-core/src/contracts.rs`, and fills the missing half: **how a
risk class is derived**, not just what the tiers are.

## 1. Problem

`risk_class` is load-bearing: policy admission uses it to require simulation and
human approval (`crates/beater-os-core/src/policy.rs`), and `final.md` §12.3
states the invariant *"risk class can be raised by policy, never lowered by the
agent."* But two things are undefined today:

1. **No documented taxonomy** — the tiers exist in code but not their meaning.
2. **No derivation rule** — the kernel currently *trusts* `manifest.risk_class`
   as supplied by the agent. Since the agent is model-driven and can be
   manipulated (§13.1), a value proposed by the model is not a trustworthy floor.
   There must be a deterministic function that (re)computes risk **outside the
   model** and takes the max with whatever the agent proposed.

## 2. Canonical tiers (already in code — do not fork)

`RiskClass` is a 4-tier ordered enum (`contracts.rs`); this spec only assigns
meaning and consequences. Order matters: `Low < Medium < High < Critical`.

| Tier | Meaning | Typical trigger | Policy consequence (as coded today) |
| --- | --- | --- | --- |
| `Low` | Pure observation or trivially reversible local effect | read granted paths; local file write inside a grant | Auto-allow within grant; default grant ceiling is `max_risk = Medium` |
| `Medium` | Reversible effect with limited blast radius | create branch; write memory; internal-data read | Allowed if inside grant; above default ceiling needs a wider grant |
| `High` | External side effect / hard-to-reverse / sensitive data egress | network write, cloud mutation, external email, customer/secret data to a model route | **Requires a passed simulation** before execution (`policy.rs`: `risk_class >= High`) |
| `Critical` | Irreversible or high-consequence | payment, deploy to prod, credential use, delegation of authority | Requires approval when a grant's `threshold_risk` is met (default `threshold_risk = Critical`) — plus the `High` simulation gate |

These consequences already exist in `policy.rs`; this spec makes the *inputs*
that land an action in each tier deterministic.

## 3. Derivation rule (the missing piece)

Risk is a **pure function** of structured, non-model-authored inputs:

```
risk(action) = max(
    base_by_action_kind[action.kind],
    base_by_side_effect[max(action.side_effect_classes)],
    data_sensitivity_bump(action.data_classes, action.taint_labels),
    reversibility_bump(action.compensation_plan),
    counterparty_bump(action.target),
    action.proposed_risk_class          // agent proposal is a FLOOR, never a cap
)
```

Every term is computed from typed fields (`ActionKind`, `SideEffectClass`,
`DataClass`, `TaintLabel`, `target`, `compensation_plan`) — none from free text.
`max` guarantees the §12.3 invariant structurally: policy can only raise.

### 3.1 Base by `ActionKind`

| `ActionKind` | Base |
| --- | --- |
| `Read`, `AskHuman` | `Low` |
| `Write`, `Remember`, `Navigate` | `Low` → `Medium` (see side-effect/data bumps) |
| `Execute`, `Submit`, `Communicate` | `Medium` |
| `Deploy`, `Delegate` | `High` |
| `Spend` | `Critical` |

### 3.2 Base by `SideEffectClass`

| `SideEffectClass` | Base |
| --- | --- |
| `None` | `Low` |
| `LocalWrite`, `MemoryWrite` | `Low`/`Medium` |
| `NetworkWrite`, `BrowserSubmit`, `HumanCommunication`, `CloudMutation` | `High` |
| `Deployment`, `Payment` | `Critical` |
| `Delegation` | `High` (→ `Critical` if the delegated scope includes `Spend`/`Deploy`) |

### 3.3 Data-sensitivity bump

If any `DataClass` in `{Secret, Financial, Customer, Personal}` **leaves a trust
boundary** (egress to a model route, network, or external recipient), bump to at
least `High`. `Secret` egress to a non-local model route is at least `Critical`.
Untrusted inputs (`UntrustedWeb/Email/Document`, `ToolOutput`) never *lower*
risk and cannot authorize the taint-gated actions in §3.5.

### 3.4 Reversibility bump

If `compensation_plan` is absent/`null` **and** the action has any non-`None`
`SideEffectClass`, bump one tier (an unreversible side effect is riskier than a
reversible one). An action that declares itself irreversible is at least `High`.

### 3.5 Counterparty / taint interaction (hard floors)

Independent of the numeric max, these are non-negotiable floors that also route
to the existing `policy.rs` taint rule ("untrusted content cannot authorize
spend/deploy/delegation without action-bound approval"):

- `Spend` / `Payment` → `Critical`, always, and requires a `PaymentMandate`.
- Any action whose **authorizing instruction** carries `UntrustedWeb`,
  `UntrustedEmail`, `UntrustedDocument`, or `ToolOutput` taint → risk is raised
  and the action needs explicit user-promoted approval; it can never be
  auto-allowed.
- Credential/secret use (`TaintLabel::Secret`, `PaymentInstruction`) → `High` min.

## 4. Invariants (for tests and evals)

1. **Monotone raise-only:** `derived_risk >= manifest.proposed_risk_class`. A run
   where an executed action's effective risk is below its derived risk is a bug.
2. **Fail-safe default:** any input the classifier cannot map (unknown target
   kind, unknown side effect, missing fields) yields `Critical` — consistent with
   §12.3 "unknown side effects require denial or review."
3. **Model-independence:** the classifier reads only typed manifest fields; it
   must produce the same class for the same fields regardless of prompt text or
   model. This is the property that makes risk trustworthy.
4. **Consequence coupling:** `High ⇒ simulation-gated`, `≥ grant.threshold_risk ⇒
   approval-gated` — already enforced in `policy.rs`; the classifier only decides
   the tier, never the gate.

## 5. Gap this closes and follow-up for the kernel

Today `admit()` in `policy.rs` consumes `manifest.risk_class` as-is. Closing #8
in code means adding a deterministic `classify_risk(&ActionManifest) -> RiskClass`
in `beater-os-core`, calling it during admission, and using
`max(proposed, classified)` — so a model that under-reports risk cannot lower a
gate. That is an implementation slice for the kernel lane (codex/#1); this spec is
the contract it should satisfy.

## 6. Acceptance mapping (issue #8)

- [x] Enumerated risk classes with definitions — §2 (grounded in `RiskClass`).
- [x] Deterministic assignment rule (inputs → class) — §3.
- [x] Fail-safe default documented — §4.2 (`Critical` on unknown).
