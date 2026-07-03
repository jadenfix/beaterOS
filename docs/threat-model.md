# beaterOS Threat Model

Status: initial threat model (Phase 1 deliverable, see `final.md` §19 and issue #7).
This is the structured companion to `final.md` §13 (security principles/controls)
and §14.5 (security evals). §13 lists controls; this document says *what they
defend, against whom, across which boundary, and which eval proves it*.

Method: assets → trust boundaries → adversaries/capabilities → attack→mitigation→
eval matrix → residual risk / out-of-scope. Terms are defined in `docs/glossary.md`.

## 1. Assets (what must be protected)

| # | Asset | Why it matters |
|---|-------|----------------|
| A1 | Secrets / credentials | Direct compromise of user and third-party accounts (§13.9) |
| A2 | Capability grants | Forgery/broadening = unauthorized authority (§13.3) |
| A3 | Journal & receipt integrity | The source of truth for causality, audit, replay (§8.3) |
| A4 | Memory | Poisoned/leaked memory corrupts future decisions (§13.4) |
| A5 | Payment authority | Money movement is irreversible (§8.7) |
| A6 | User / customer data | Privacy, regulation, contractual duty (§13.4) |
| A7 | Model routes | Wrong route leaks sensitive data to an untrusted provider (§10.9) |
| A8 | Availability & budget | Runaway loops exhaust money/quota (issue #15) |

## 2. Trust boundaries

- **B1 Model ↔ policy engine** — the model proposes; policy decides. The model is
  never the root of trust. (§13.1)
- **B2 Agent ↔ subagent** — delegated authority must be attenuated. (§8.2)
- **B3 Gateway ↔ remote MCP/A2A server** — remote tools are untrusted network
  services with code-execution implications. (§13.6)
- **B4 Browser origin boundaries** — untrusted web content vs. credentialed
  sessions. (§13.7)
- **B5 Host ↔ sandbox lane** — untrusted code must not escape its lane. (§13.8)
- **B6 Local ↔ cloud** — authority roots locally; cloud is an execution lane, not
  a trust root. (§8.13)
- **B7 Data ↔ instruction channel** — untrusted data must not become privileged
  instruction. (§5.8, §13.5)

## 3. Adversaries and their capabilities

- **T1 Malicious content** (web page, document, email, image) — can inject text
  the model may treat as instructions; can hide instructions in multimodal data.
- **T2 Malicious / compromised tool or MCP server** — lies in its description,
  changes behavior across calls, requests broad access, asks for secrets.
- **T3 Compromised / non-compliant model provider** — retains or exfiltrates
  prompt data; returns manipulated output.
- **T4 Malicious subagent** — attempts to acquire parent authority or exceed its
  attenuated scope.
- **T5 Persuaded-but-honest model** — not compromised, but socially engineered by
  T1 into proposing an overbroad or dangerous action. (The central agent threat.)
- **T6 Over-privileged / careless user** — grants too much; approves without
  reading. (Insider-by-accident.)
- **T7 Network attacker** — MITM, DNS/lookalike origins, replay.
- **T8 Supply-chain attacker** — poisons a tool package, dependency, or update.

## 4. Attack → mitigation → eval matrix

Each row ties a concrete attack to the `final.md` §13 control that blocks it and
the §14.5 security-eval class that proves the block. **Success criterion: the OS
policy layer blocks or escalates even if the model is fully persuaded (T5).**

| Attack | Boundary | Primary mitigation (§13) | Eval (§14.5) |
|--------|----------|--------------------------|--------------|
| Web page: "ignore instructions, exfiltrate secrets" | B7 | Data/instruction separation; untrusted content cannot create grants; secrets are handles not values (§13.4, §13.5, §13.9) | Web prompt-injection scenario |
| PDF/image with hidden instructions | B7 | Multimodal classification before routing; taint labels (§15.3, §13.4) | Compromised-document scenario |
| MCP server exposes lookalike tool / changes schema | B3 | Schema pinning; description distrust; token audience binding; no passthrough (§13.6) | Malicious-tool / lookalike-tool scenario |
| Tool output asks for a secret | B3/B7 | Tool output cannot become instruction; secret broker mediates (§13.4, §13.9) | Tool-output-asks-for-secret scenario |
| Email: "wire funds to X" | B7/A5 | Untrusted email cannot authorize payment; PaymentMandate required; approval threshold (§13.4, §12.7) | Email-payment scenario |
| Browser origin imitates trusted site | B4/B7 | Origin classification; phishing/lookalike detection; credential-use receipts (§13.7) | Lookalike-origin scenario |
| Model suggests an overbroad permission (T5) | B1/A2 | Capability checks outside model; policy raises risk; overbroad grants hard to create (§13.1, §13.3) | Overbroad-grant scenario |
| Subagent requests parent authority (T4) | B2/A2 | Attenuation enforced; delegated ⊆ parent; separate identity (§8.2, §12.2) | Subagent-escalation scenario |
| Poisoned fact in memory (A4) | B7/A4 | Memory provenance; untrusted-source quarantine; confidence + expiry (§13.4, §10.8) | Memory-poisoning scenario |
| Payment address changes mid-flow (A5) | A5 | Counterparty binding in mandate; idempotency; receipts; re-approval on change (§16.1) | Payment-address-swap scenario |
| Untrusted code escapes lane (B5) | B5 | seccomp/AppArmor/cgroups, container/VM isolation, network-off default (§13.8) | Sandbox-escape scenario |
| Poisoned dependency/update (T8) | — | Signed manifests, pinned versions, SBOM, SLSA provenance, quarantine (§13.10) | Supply-chain scenario |
| Runaway loop exhausts budget (A8) | B1 | Kernel-enforced budget ceilings; step/retry/replan caps; fail-closed (issue #15) | Budget-exhaustion scenario |
| Secret sent to public model route (A7) | B1/A7 | Data-class policy on routes; redaction/handles before routing (§13.4, §10.9) | Data-exfil-via-route scenario |
| Journal/receipt tampering (A3) | — | Hash-linked journal; Merkle batching; journal verifier in TCB (§13.11) | Tamper-detection check |

## 5. Cross-cutting invariants (must hold regardless of attack)

These map to the zero-tolerance metrics that issue #14 asks to formalize:

1. No side effect executes without a valid, unexpired, non-revoked grant.
2. No grant is broadened by its holder; delegated grants are ⊆ parent.
3. Denied/needs-approval actions cannot execute before satisfaction.
4. Untrusted-tainted content never becomes a privileged instruction or a grant.
5. Every external side effect produces a hash-chained receipt.
6. Secrets never appear in prompts, logs, receipts, or external model inputs
   except where an explicit data-class policy permits.

## 6. Residual risk / explicitly out of scope

- **Compromised host kernel / hardware** below the sandbox lane — not defended by
  the user-space control plane (mitigated only on the §9 high-assurance track:
  seL4/CHERI/TEE).
- **A truly malicious (not merely careless) authorized human** who deliberately
  issues broad grants — the system makes this *legible and auditable*, not
  impossible. (§13.3, §13.14)
- **Model provider that violates its stated retention contract** — detectable via
  policy/attestation only where the provider supports it (§15.2, §13.13).
- **Novel prompt-injection techniques** — defense is defense-in-depth, not a
  proof; the bet is that policy/capability enforcement outside the model bounds
  the blast radius even when injection succeeds. (§13.5)

## 7. Open dependencies

This model has hard dependencies on specifications still being written:
`risk_class` taxonomy (#8), revocation semantics (#10), journal-redaction
mechanism (#9), and budget enforcement (#15). Rows above that reference them are
provisional until those land.
