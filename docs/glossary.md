# beaterOS Glossary

Status: initial glossary (Phase 0 deliverable, see `final.md` §19 and issue #4).
Defines the load-bearing vocabulary of `final.md` once, so the rest of the docs
can reference instead of re-explain. Terms are grouped; within a group they build
on each other.

> Naming note: `final.md` currently names the side-effect record three ways
> (§7.6 "Side-Effect Receipt", §12.5 "CapabilityReceipt", §10/§26 "receipt").
> This glossary uses **Receipt** as the canonical term and flags the alias.
> Resolving the name in `final.md` is tracked in issue #6.

## Core contracts (the typed objects the kernel admits, records, and replays)

- **AgentSession** — The container for one goal-directed run: intent,
  constraints, owner, agent identity, policy profile, initial capabilities,
  budgets, model policy, memory scope, and a journal root. A session cannot
  execute actions without at least one capability grant. (`final.md` §7.2, §12.1)
- **CapabilityGrant** — The central authority object: explicit, unforgeable,
  scoped permission for a principal to perform specific actions on a specific
  resource, bounded by time, budget, data-sensitivity, delegation, and approval
  rules. Grants are never inferred from prompt text. (§7.3, §12.2)
- **ActionManifest** — A pre-declaration of a proposed observation or side
  effect: tool, target, input summary/digest, expected side effect, risk class,
  required grants, idempotency key, and compensation plan. Submitted *before*
  execution so policy can inspect it. (§7.4, §12.3)
- **PolicyDecision** — The deterministic admission result for a manifest:
  allow / deny / needs-approval / needs-simulation / needs-narrowed-grant, plus
  matched rules and an explanation. Journaled before execution. (§7.5, §12.4)
- **Receipt** *(alias: CapabilityReceipt, Side-Effect Receipt)* — The
  append-only, hash-linked record of what actually happened after an action ran:
  timing, input/output digests, side-effect summary, external IDs, status, and a
  hash link to the previous receipt. Enables replay and accountability.
  (§7.6, §12.5)
- **MemoryRecord** — A unit of knowledge with provenance: source event, writer,
  confidence, data class, sensitivity, expiry, and access policy. Memory is a
  projection of journaled events, not an ungoverned blob. (§7.7, §12.6)
- **PaymentMandate** — Bounded economic authority: who may spend, asset, max
  amount, counterparty policy, purpose, window, approval threshold, idempotency,
  receipt requirement. No payment without a mandate. (§12.7, §8.7)
- **ScenarioManifest** — The eval/simulation spec for a workflow: goal,
  environment fixtures, allowed tools, forbidden actions, oracle, success
  criteria, risk traps, budget, and expected trace properties. Production
  incidents become new scenarios. (§7.10, §12.8)

## Authority and protection

- **Capability** — Possession of a specific, unforgeable token of authority
  (object-capability style): grants exact power, can be attenuated, delegated,
  expired, revoked, and audited. (§6.2)
- **Attenuation** — Narrowing authority when delegating: a subagent receives a
  grant equal to or narrower than the parent's, never broader. (§8.2, §12.2)
- **Ambient authority** — Authority a process holds implicitly by context (global
  filesystem, shell, cookies, env-var secrets). beaterOS's core failure mode to
  eliminate: every dangerous resource requires an explicit grant. (§5.2, §13.2)
- **Principal** — Any party the system distinguishes for trust: human user, org,
  agent, subagent, model provider, tool provider, MCP server, browser origin,
  document source, memory source, payment counterparty, policy authority. (§4.2)
- **Revocation handle** — The indirection through which a grant is invalidated;
  revocation should cascade to delegated child grants. In-flight and delegated
  revocation semantics are being specified in issue #10. (§6.2, §12.2)
- **Fail-closed** — On expiry, revocation, missing grant, or unknown side effect,
  the default is denial. (§12.2, §12.3)
- **TCB (Trusted Computing Base)** — The minimal set of components that must be
  correct for security to hold: candidate members are the capability service,
  policy engine, journal verifier, secret broker, and sandbox launcher.
  Everything else is less trusted and the TCB is shrunk continuously. (§20.2, §22.10)

## Provenance, memory, and data

- **Journal** — The append-only, hash-linked event log that is the source of
  truth for causality. Memory and other views are projected from it; it must
  support redaction without destroying integrity (mechanism tracked in #9).
  (§8.3, §10.4)
- **Taint / provenance label** — A tag on every information source recording its
  trust class (`trusted_user_instruction`, `untrusted_web`, `tool_output`,
  `secret`, `customer_data`, `payment_instruction`, …). Drives information-flow
  policy. (§13.4)
- **Data class / Sensitivity** — Classification of the *kind* and *sensitivity*
  of data an action or memory touches; gates which model routes and tools may
  see it. (§10.8, §13.4)
- **Risk class** — The severity tier of a proposed action; drives allow/deny/
  review/simulation. The agent may propose it; policy may raise but never lower
  it. The taxonomy and assignment rule are defined in issue #8. (§12.3, §5.3)

## Execution and services

- **Agent kernel (`beater-osd`)** — The smallest trusted daemon: owns the root
  journal and policy versions, issues grants, validates manifests, accepts
  receipts, publishes trace events. It does not run tools directly. (§9.2, §10.1)
- **Tool gateway** — The mediated boundary that normalizes MCP / A2A / OpenAPI /
  CLI / browser / local tools into typed registry entries, enforces grants,
  redacts I/O, and blocks token passthrough. (§8.6, §10.5)
- **Sandbox lane** — An isolated execution environment for an action class: pure
  function, WASI, container, browser, VM, or remote-tool lane. Every lane emits
  receipts. (§10.6)
- **Model router** — Chooses a model by task, risk, data class, latency, cost,
  and policy; the router obeys policy, it does not make policy. (§10.9)
- **Human review** — A first-class system service (queue, diff/preview, decision,
  escalation, multi-party approval) — not an ad-hoc chat prompt. (§7.9, §10.10)
- **Eval gate** — A release gate that runs scenario manifests and blocks changes
  (model/tool/policy/prompt) that regress behavior. (§5.9, §14.6)
- **Workspace** — A scoped environment for work (files, repos, browser contexts,
  secret references, project memory, tools, scenarios, policies, audit logs).
  (§9.4)

## Evals and integrity

- **Oracle** — The mechanism that scores whether a scenario succeeded, from
  strongest to weakest: exact match, unit tests, static checks, diff/state
  checks, API/DOM assertions, screenshot comparison, human rubric, calibrated
  LLM-judge, multi-oracle consensus. Use the weakest oracle that is reliable
  enough. (§14.3)
- **Counterfactual replay** — Re-running a recorded trace under a changed
  variable (narrower grant, cheaper model, injected web content, timed-out
  approval) to turn production traces into design insight. (§14.7)
- **Idempotency key** — A token on an action that makes re-execution safe, so
  retries and recovery don't duplicate side effects. (§7.4, §12.3)
- **Compensation plan** — The rollback/undo path declared for an action whose
  side effect may need reversing during recovery or revocation. (§12.3, §4.6)
- **Merkle DAG / transparency log** — Hash-linked / Merkle-batched structures
  used to make journals, receipts, tool-registry changes, and release
  attestations tamper-evident (detect modification — not a substitute for good
  policy). (§13.11, §16.4)

## External protocols

- **MCP (Model Context Protocol)** — Standard for exposing tools/context to
  models; beaterOS supports it but routes it through the gateway rather than
  trusting servers directly. (§8.6, §13.6)
- **A2A (Agent-to-Agent)** — Cross-agent / cross-org interoperability; needs
  identity, authorization, and trace boundaries between agents (design tracked
  in issue #13). (§18.4)
- **x402** — HTTP-native payment flow; treated as a payment *adapter* behind a
  PaymentMandate, never a foundation. (§8.7, §16.1)
