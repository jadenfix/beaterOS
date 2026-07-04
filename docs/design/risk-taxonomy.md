# Design Spec: Risk Taxonomy & Deterministic Assignment

Status: design spec. Closes the gap tracked in **issue #8**. Grounded in the
merged `crates/beater-os-core` code (`RiskClass`, `ActionManifest`, policy
admission) so the assignment rule is executable, not aspirational. Does not edit
`final.md`.

`final.md` makes `risk_class` load-bearing — it appears in `ActionManifest`
(§12.3), drives policy admission (§7.5, §10.3), and gates human review and
simulation — and states the invariant that *risk can be raised by policy, never
lowered by the agent* (§12.3). But it never enumerates the classes or says how a
class is assigned. This spec fixes both, consistent with the ratings the merged
admission tests already assume.

## 1. The ladder

`RiskClass` is a total order (`Low < Medium < High < Critical`), already encoded
in `beater-os-core`:

| Class | Meaning | Reversibility |
| --- | --- | --- |
| `Low` | Observation — read, navigate, ask a human | Nothing to undo |
| `Medium` | Reversible workspace mutation — local/memory write | Undo from the workspace diff / receipt |
| `High` | External but recoverable — network write, submit, communicate, deploy, cloud mutation, delegation | Compensating action possible |
| `Critical` | Irreversible economic loss — payment | No technical undo; economic only |

The dividing line between `High` and `Critical` is **irreversibility of
economic loss**. Deploying to production is serious but typically roll-back-able,
so it is `High`; money leaving an account is not, so `Payment`/`Spend` is
`Critical`. This matches the existing admission tests (deploy rated `High` and
gated by simulation; spend rated `Critical`).

## 2. Deterministic assignment

Risk is the **maximum** over three intrinsic contributors of the action — never
a single lookup, so any one high-risk facet escalates the whole action. This is
`RiskClass::derive_floor(action_kind, side_effects, data_classes)` in
`contracts.rs`.

**Action verb** (`action_kind`):

| Verbs | Floor |
| --- | --- |
| `read`, `navigate`, `ask_human` | `Low` |
| `write`, `remember` | `Medium` |
| `execute`, `submit`, `communicate`, `deploy`, `delegate` | `High` |
| `spend` | `Critical` |

**Declared side effect** (`expected_side_effects`):

| Effects | Floor |
| --- | --- |
| `none` | `Low` |
| `local_write`, `memory_write` | `Medium` |
| `network_write`, `browser_submit`, `human_communication`, `delegation`, `deployment`, `cloud_mutation` | `High` |
| `payment` | `Critical` |

**Data sensitivity** (`data_classes`):

| Classes | Contribution |
| --- | --- |
| `secret`, `financial` | `High` |
| `personal`, `customer` | `Medium` |
| `public`, `internal`, `code`, `binary`, `tool_output`, `untrusted_*` | `Low` |

Untrusted-provenance classes (`untrusted_web/email/document`) do **not** raise
the risk floor. Untrusted provenance is an injection concern carried by taint
labels and handled by the untrusted-instruction admission rule, not an intrinsic
sensitivity of the data being touched.

## 3. Fail-safe default

The floor is a `max`, so the fail-safe is structural: an action that combines
several facets is rated at the **highest** one, and adding any facet can only
raise the floor, never lower it. There is no "unknown" hole — every enum variant
maps explicitly, and a future variant added without a mapping is a compile error
in the exhaustive `match`, not a silent `Low`.

## 4. Enforcement: policy raises, the agent cannot lower

`derive_floor` is the executable form of §12.3's invariant. Admission computes
the floor from the manifest's own declared verb, effects, and data classes, then:

- If `manifest.risk_class < derived_floor`, the action is **denied** with
  `manifest under-rates risk below the floor its action kind, side effects, and
  data classes require`. The agent must resubmit a correctly-rated manifest.
- Otherwise the rule `manifest_risk_meets_derived_floor` is recorded and
  admission proceeds.

### Why deny-and-resubmit rather than silently raise

§12.3 says policy *raises* risk. In this kernel, admission cannot silently
mutate `risk_class`, because `manifest_hash` is computed over the whole manifest
(including `risk_class`) and every downstream evidence object — approvals,
simulations, receipts — binds to that hash. Silently raising the class would
desync the executing manifest from the one that was approved. Denying and
forcing the agent to resubmit a correctly-rated manifest preserves the
hash-binding invariant while achieving the same end: the agent cannot execute
under an under-rated class. The raise still happens — it just happens by
rejection and correction, not by mutation.

This is the same shape as the tool-registry grounding of issue #46: admission
consumes a kernel-derived floor rather than the agent's bare claim. The two
floors compose — the effective floor is the max of the intrinsic derivation
(this spec) and the registered-tool floor (#46).

## 5. What this spec does not settle

- **Counterparty and target sensitivity.** §5.3 (semantic risk) hints that *who*
  you pay or *which* endpoint you hit should move risk. That needs target
  metadata the manifest does not yet carry and is left to a follow-up.
- **Policy-profile overrides.** A deployment profile might legitimately raise
  `deploy` to `Critical`. The taxonomy here is the floor; per-profile raises on
  top of it are compatible and out of scope for this slice.
