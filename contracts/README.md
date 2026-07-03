# beaterOS Contract Schemas (language-neutral wire format)

Canonical, language-neutral **JSON Schema** (draft 2020-12) for the beaterOS
agent-kernel contracts described in `final.md` §12. These schemas are the
*interop boundary*: any language (Python, TypeScript, Go, …) can validate the
exact same JSON that the Rust `beater-os-core` crate serializes, without
depending on the Rust source.

This directly serves the multi-agent build model
(`docs/multi-agent-coordination.md` / `AGENTS.md`): downstream slices — eval
scenarios, an MCP gateway, a trace viewer, or a non-Rust agent — can "build
against the contract, not the code" and proceed while the Rust core is still in
review.

## Why this exists (final.md alignment)

- §12 Core Data Contracts — these are those contracts, as data.
- §8.4 Typed Actions vs Raw Tool Calls — schemas make actions inspectable.
- §18.4 A2A / cross-vendor agents — a shared wire format is the integration point.
- §10.5 Tool Gateway normalization — external tools normalize into these types.

## Provenance and sync

Field names and enum values mirror the serde serialization of:

- `crates/beater-os-core/src/contracts.rs`
- `crates/beater-os-core/src/receipt.rs`
- `crates/beater-os-core/src/hash.rs` (`HashValue` = lowercase hex SHA-256)

Mirrored from codex's PR #1 at commit `ea81ff08`. codex's crate is the source
of truth for the Rust types; **these schemas track that wire format.** If codex
renames or adds a field, update `schemas/common.schema.json` to match (see the
coordination note on PR #19). The examples are illustrative — hashes are
well-formed but not chain-recomputed against the Rust hashing.

## Layout

```
contracts/
  schemas/
    common.schema.json         # single source of truth: all enums, shared
                               # structs, and every contract, under $defs
    <contract>.schema.json     # thin entry point: $ref into common.schema.json
  examples/
    <contract>.example.json    # valid instances (a coherent coding-workflow trace)
    invalid/
      <contract>.<reason>.json # fixtures that MUST be rejected
  validate.py                  # dependency-free validator (+ optional jsonschema cross-check)
```

Contracts covered: `AgentIdentity`, `AgentSession`, `CapabilityGrant`,
`ActionManifest`, `PolicyDecision`, `CapabilityReceipt`, `MemoryRecord`,
`PaymentMandate`, `ScenarioManifest`, `ToolManifest`, `HumanReviewRequest`,
`ApprovalEvidence`, `SimulationEvidence`, plus shared value types (`Budget`,
`ModelPolicy`, `CapabilitySelector`, `CapabilityScope`, `GrantConstraints`,
`ApprovalRequirement`) and the enums from `final.md` §13.4 / §20.1.

## Validate

```bash
python3 contracts/validate.py           # verbose
python3 contracts/validate.py --quiet   # summary + failures only
```

Exit code is `0` iff every valid example passes **and** every `invalid/`
fixture is correctly rejected. The validator is pure standard-library Python 3
(no network, no install) so any reviewer can reproduce it; if the optional
`jsonschema` package is present it runs as an independent cross-check and any
disagreement fails the run.

## Optionality rules (mirroring serde)

A field is **required** unless the Rust struct marks it `#[serde(default)]`.
Optional scalar fields are typed `["<type>", "null"]` — omitted, `null`, or the
value are all accepted, matching `Option<T>` + `#[serde(default)]`. Ordered
sets (`BTreeSet`) are arrays with `uniqueItems: true`.
