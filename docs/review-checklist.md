# beaterOS reviewer checklist

A reusable rubric any agent or person can apply to **any** PR they did not
author. It is derived from `final.md` Section 26 ("What Not To Compromise"),
Section 13 ("Security Model"), and the core data contracts in Section 12.

Use it top to bottom. A single unchecked **blocker** means `REQUEST_CHANGES`.

## A. Scope and clarity (all PRs)

- [ ] The PR maps to a named section of `final.md` and does one coherent thing.
- [ ] Code is understandable by a reviewer seeing it for the first time; public
      contracts are typed, named clearly, and documented where non-obvious.
- [ ] New/changed contracts are versioned and covered by tests.
- [ ] `final.md` was not shortened or weakened.
- [ ] The change claims a disjoint write scope in
      [`agent-coordination-log.md`](agent-coordination-log.md), or coordinates
      with the overlapping PR.

## B. Authority and capabilities (blocker if the PR touches authority)

- [ ] No ambient authority: every dangerous action requires an explicit
      capability grant; nothing inherits broad power by default.
- [ ] Grants are bound to session **and** acting identity.
- [ ] Grants cannot be broadened by the holder; delegated grants are
      equal-or-narrower (attenuation only).
- [ ] Expired and revoked grants **fail closed**.
- [ ] Grants are never inferred from prompt/model text.
- [ ] Resource matching is exact and non-bypassable (e.g. path-prefix checks
      cannot let `/a/b` match `/a/bc`; require normalized absolute paths).
- [ ] Budget/quota ceilings fail closed when the requested amount is omitted.

## C. Policy admission (blocker if the PR touches policy)

- [ ] Policy is evaluated **outside** the model, deterministically.
- [ ] Denied and review-required actions cannot execute.
- [ ] Risk class can be raised by policy, never lowered by the agent.
- [ ] Policy decisions are recorded before execution, with explanations and
      matched rules.
- [ ] Approval evidence is bound to action, grant, reviewer, policy version, and
      a non-future timestamp; multi-party approval requires **every** configured
      reviewer.
- [ ] Simulation evidence is bound to action, policy version, and a non-future
      pass time.

## D. Journal and receipts (blocker if the PR touches the audit trail)

- [ ] Intent is journaled **before** side effects; receipts are produced
      **after** side effects.
- [ ] Receipts are append-only and hash-chained (each links the previous hash).
- [ ] A receipt binds to a prior *allowed* policy decision and to the proposed
      manifest's tool, input digest, target, and declared side-effect classes.
- [ ] Redaction is possible without breaking chain integrity.
- [ ] Journal verification rejects receipts lacking a valid prior decision.

## E. Memory, tools, and I/O boundaries

- [ ] Memory records carry provenance (source, time, confidence, sensitivity,
      access policy) and are rebuildable/redactable.
- [ ] Untrusted content (web/email/doc/tool-output) cannot become privileged
      instruction or create new permissions.
- [ ] Tools are identified, pinned, and risk-tagged; no token passthrough.
- [ ] Secrets use handles, not values; secrets are redacted in traces and never
      sent to unauthorized model routes.

## F. Crypto and supply chain

- [ ] Standard, maintained crypto libraries only — no invented primitives.
- [ ] Signing separate from encryption; algorithm agility preserved.
- [ ] Dependencies are pinned; the direct dependency set stays small and justified.

## G. Tests, CI, and reproducibility

- [ ] Tests cover the invariants the change touches, including at least one
      adversarial/negative case (fail-closed, injection, or bypass attempt).
- [ ] CI gates the change (Rust: fmt/test/clippy with `-D warnings`; governance
      job green).
- [ ] Behavior is deterministic enough to replay where `final.md` requires it.

## H. Review routing (process, verified by the merger)

- [ ] The reviewer is not the author.
- [ ] For high-risk PRs, two independent approvals are present.
- [ ] The merger is not the author.

---

### How to record the review

Post a GitHub review with:

- **Verdict**: `APPROVE` / `APPROVE_WITH_NITS` / `REQUEST_CHANGES`.
- **Blocking**: `file:line` + defect + concrete failure scenario.
- **Non-blocking**: nits.
- **Test gaps**: invariants touched but untested.
- **Strengths**: what is done well (keep it honest and brief).
