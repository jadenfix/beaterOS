use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::contracts::{
    ActionKind, ActionManifest, ApprovalEvidence, ApprovalMode, CapabilityGrant, DataClass,
    DecisionResult, PolicyDecision, RiskClass, SideEffectClass, SimulationEvidence, TaintLabel,
    ToolManifest,
};
use crate::error::BeaterOsResult;
use crate::hash::HashValue;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionContext {
    pub now: DateTime<Utc>,
    pub actor_id: String,
    pub session_id: String,
    pub policy_version: String,
    pub grants: Vec<CapabilityGrant>,
    pub approvals: Vec<ApprovalEvidence>,
    pub simulations: Vec<SimulationEvidence>,
    /// Kernel-held ground truth about the tools an action may invoke, keyed by
    /// `tool_id`. Registered out of band at (signed) tool-install time, never by
    /// the agent per action. Admission grounds the agent's self-reported risk
    /// and side effects against these entries; see
    /// [`PolicyEngine::admit`] and issue #46.
    pub tool_registry: BTreeMap<String, ToolManifest>,
}

#[derive(Clone, Debug, Default)]
pub struct PolicyEngine;

impl PolicyEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn admit(
        &self,
        manifest: &ActionManifest,
        ctx: &AdmissionContext,
    ) -> BeaterOsResult<PolicyDecision> {
        let mut matched_rules = Vec::new();
        let manifest_hash = manifest.digest()?;

        // §7.4/§12.3/§26: risk can be raised by policy from kernel-derived fields,
        // never lowered by the agent. The agent-asserted `risk_class` may only
        // RAISE the effective risk above this kernel-derived floor.
        let derived_floor = derived_risk_floor(
            &manifest.action_kind,
            &manifest.expected_side_effects,
            &manifest.data_classes,
        );
        let effective_risk = manifest.risk_class.max(derived_floor);
        matched_rules.push(format!(
            "kernel_derived_risk_floor={derived_floor:?};declared_risk={:?};effective_risk={effective_risk:?}",
            manifest.risk_class
        ));

        if manifest.session_id != ctx.session_id {
            return Ok(decision(
                manifest,
                manifest_hash,
                ctx,
                DecisionResult::Denied,
                matched_rules,
                "action manifest session does not match admission context session",
                DecisionFollowup::none(),
            ));
        }
        matched_rules.push("manifest_bound_to_context_session".to_string());

        if manifest.required_grants.is_empty() {
            return Ok(decision(
                manifest,
                manifest_hash,
                ctx,
                DecisionResult::Denied,
                matched_rules,
                "action manifests must name at least one required grant",
                DecisionFollowup::none(),
            ));
        }
        matched_rules.push("required_grants_present".to_string());

        if manifest.has_external_side_effect() && manifest.idempotency_key.is_none() {
            return Ok(decision(
                manifest,
                manifest_hash,
                ctx,
                DecisionResult::Denied,
                matched_rules,
                "external side effects require an idempotency key before execution",
                DecisionFollowup::none(),
            ));
        }
        matched_rules.push("external_side_effect_idempotency".to_string());

        if manifest
            .expected_side_effects
            .contains(&SideEffectClass::Payment)
            && manifest.action_kind != ActionKind::Spend
        {
            return Ok(decision(
                manifest,
                manifest_hash,
                ctx,
                DecisionResult::Denied,
                matched_rules,
                "payment side effects must use the spend action kind",
                DecisionFollowup::none(),
            ));
        }

        if let Some(reason) = manifest_understates_registered_tool(manifest, ctx) {
            return Ok(decision(
                manifest,
                manifest_hash,
                ctx,
                DecisionResult::Denied,
                matched_rules,
                reason,
                DecisionFollowup::none(),
            ));
        }
        matched_rules.push("manifest_consistent_with_registered_tool".to_string());

        let matching_grants: Vec<&CapabilityGrant> = ctx
            .grants
            .iter()
            .filter(|grant| manifest.required_grants.contains(&grant.grant_id))
            .collect();
        if matching_grants.len() != manifest.required_grants.len() {
            return Ok(decision(
                manifest,
                manifest_hash,
                ctx,
                DecisionResult::Denied,
                matched_rules,
                "one or more required grants are missing from the admission context",
                DecisionFollowup::none(),
            ));
        }
        matched_rules.push("required_grants_available".to_string());

        let allowed = matching_grants
            .iter()
            .all(|grant| grant.allows_manifest(manifest, effective_risk, ctx.now, &ctx.actor_id));
        if !allowed {
            return Ok(decision(
                manifest,
                manifest_hash,
                ctx,
                DecisionResult::NeedsNarrowedGrant,
                matched_rules,
                "available grants do not allow this action, target, risk, data class, or time window",
                DecisionFollowup::none(),
            ));
        }
        matched_rules.push("all_required_capabilities_allow_action".to_string());

        if dangerous_untrusted_instruction(manifest)
            && !all_grants_have_explicit_action_approval(
                &matching_grants,
                manifest,
                &manifest_hash,
                ctx,
            )
        {
            return Ok(decision(
                manifest,
                manifest_hash,
                ctx,
                DecisionResult::NeedsApproval,
                matched_rules,
                "untrusted content cannot directly authorize spend, deploy, or delegation actions without action-bound approval",
                DecisionFollowup::review(format!(
                    "action:{}:untrusted-risk-review",
                    manifest.action_id
                )),
            ));
        }
        matched_rules.push("untrusted_instruction_policy_checked".to_string());

        if matching_grants
            .iter()
            .filter(|grant| {
                grant.approval.mode != ApprovalMode::None
                    && effective_risk >= grant.approval.threshold_risk
            })
            .any(|grant| !has_approval_for_grant(grant, manifest, &manifest_hash, ctx))
        {
            return Ok(decision(
                manifest,
                manifest_hash,
                ctx,
                DecisionResult::NeedsApproval,
                matched_rules,
                "grant policy requires human approval for this risk class",
                DecisionFollowup::review(format!(
                    "action:{}:grant-threshold-review",
                    manifest.action_id
                )),
            ));
        }
        matched_rules.push("grant_approval_policy_checked".to_string());

        if effective_risk >= RiskClass::High
            && manifest.has_external_side_effect()
            && !has_passed_simulation_for_action(manifest, &manifest_hash, ctx)
        {
            return Ok(decision(
                manifest,
                manifest_hash,
                ctx,
                DecisionResult::NeedsSimulation,
                matched_rules,
                "high-risk external side effects require a passed simulation before execution",
                DecisionFollowup::simulation(format!(
                    "action:{}:high-risk-side-effect-simulation",
                    manifest.action_id
                )),
            ));
        }

        matched_rules.push("admitted_by_capability_policy".to_string());
        Ok(decision(
            manifest,
            manifest_hash,
            ctx,
            DecisionResult::Allowed,
            matched_rules,
            "action admitted by explicit active capability grant",
            DecisionFollowup::none(),
        ))
    }
}

/// Kernel-derived risk floor (final.md §7.4/§12.3/§26).
///
/// Risk class can be raised by policy but never lowered by the agent, and no
/// policy predicate may condition on an agent-asserted field. This function is a
/// pure function of the kernel-derived fields only (`action_kind`,
/// `expected_side_effects`, `data_classes`); it must never read the
/// agent-asserted `risk_class`. The returned floor is the conservative maximum
/// across the action kind and every present side effect and data class.
pub fn derived_risk_floor(
    action_kind: &ActionKind,
    side_effects: &BTreeSet<SideEffectClass>,
    data_classes: &BTreeSet<DataClass>,
) -> RiskClass {
    let mut floor = RiskClass::Low;

    if matches!(
        action_kind,
        ActionKind::Spend | ActionKind::Deploy | ActionKind::Delegate
    ) {
        floor = floor.max(RiskClass::High);
    }

    for effect in side_effects {
        let contribution = match effect {
            SideEffectClass::Payment
            | SideEffectClass::CloudMutation
            | SideEffectClass::Deployment
            | SideEffectClass::Delegation => RiskClass::High,
            SideEffectClass::NetworkWrite
            | SideEffectClass::BrowserSubmit
            | SideEffectClass::HumanCommunication => RiskClass::Medium,
            // Benign side effects must not be over-gated.
            SideEffectClass::None | SideEffectClass::LocalWrite | SideEffectClass::MemoryWrite => {
                RiskClass::Low
            }
        };
        floor = floor.max(contribution);
    }

    for class in data_classes {
        let contribution = match class {
            DataClass::Secret | DataClass::Financial => RiskClass::High,
            DataClass::Customer | DataClass::Personal => RiskClass::Medium,
            _ => RiskClass::Low,
        };
        floor = floor.max(contribution);
    }

    floor
}

/// Ground the agent's self-reported manifest against kernel-held tool truth
/// (issue #46). The agent authors `risk_class` and `expected_side_effects`, so
/// an adversarial or injected agent can under-declare them to slip past the
/// idempotency, simulation, and approval gates keyed off those fields. The tool
/// registry records what the invoked tool is actually known to do, out of band
/// from any single action. Admission therefore consumes the registered value as
/// a floor: a manifest may over-declare risk or effects (stricter is always
/// safe), but never under-declare below the registered tool.
///
/// Returns `Some(reason)` when the manifest understates the registered tool, or
/// `None` when it is consistent — or when the tool has no registry entry, which
/// this slice cannot ground and leaves to the rest of policy. Failing closed on
/// an *unregistered* tool is deliberately deferred: it is coupled to the
/// tool-registration lifecycle (issue #44) and would change admission for every
/// caller that has not yet wired a registry.
fn manifest_understates_registered_tool(
    manifest: &ActionManifest,
    ctx: &AdmissionContext,
) -> Option<&'static str> {
    let tool = ctx.tool_registry.get(&manifest.tool_id)?;
    if manifest.risk_class < tool.risk_class {
        return Some("manifest under-rates risk below the registered tool floor");
    }
    let omits_registered_side_effect = tool.side_effects.iter().any(|effect| {
        *effect != SideEffectClass::None && !manifest.expected_side_effects.contains(effect)
    });
    if omits_registered_side_effect {
        return Some("manifest omits a side effect the registered tool is known to produce");
    }
    None
}

fn dangerous_untrusted_instruction(manifest: &ActionManifest) -> bool {
    manifest.taint.iter().any(|label| {
        matches!(
            label,
            TaintLabel::UntrustedWeb | TaintLabel::UntrustedEmail | TaintLabel::UntrustedDocument
        )
    }) && matches!(
        manifest.action_kind,
        ActionKind::Spend | ActionKind::Deploy | ActionKind::Delegate
    )
}

fn all_grants_have_explicit_action_approval(
    grants: &[&CapabilityGrant],
    manifest: &ActionManifest,
    manifest_hash: &HashValue,
    ctx: &AdmissionContext,
) -> bool {
    grants.iter().all(|grant| match grant.approval.mode {
        ApprovalMode::None => false,
        ApprovalMode::Human | ApprovalMode::MultiParty => {
            has_approval_for_grant(grant, manifest, manifest_hash, ctx)
        }
    })
}

fn has_approval_for_grant(
    grant: &CapabilityGrant,
    manifest: &ActionManifest,
    manifest_hash: &HashValue,
    ctx: &AdmissionContext,
) -> bool {
    match grant.approval.mode {
        ApprovalMode::None => true,
        ApprovalMode::Human => grant.approval.reviewer_ids.iter().any(|reviewer_id| {
            has_approval_from_reviewer(grant, manifest, manifest_hash, ctx, reviewer_id)
        }),
        ApprovalMode::MultiParty => {
            !grant.approval.reviewer_ids.is_empty()
                && grant.approval.reviewer_ids.iter().all(|reviewer_id| {
                    has_approval_from_reviewer(grant, manifest, manifest_hash, ctx, reviewer_id)
                })
        }
    }
}

fn has_approval_from_reviewer(
    grant: &CapabilityGrant,
    manifest: &ActionManifest,
    manifest_hash: &HashValue,
    ctx: &AdmissionContext,
    reviewer_id: &str,
) -> bool {
    ctx.approvals.iter().any(|approval| {
        approval.approved_at <= ctx.now
            && approval.action_id == manifest.action_id
            && approval.manifest_hash == *manifest_hash
            && approval.grant_id == grant.grant_id
            && approval.policy_version == ctx.policy_version
            && approval.reviewer_id == reviewer_id
    })
}

fn has_passed_simulation_for_action(
    manifest: &ActionManifest,
    manifest_hash: &HashValue,
    ctx: &AdmissionContext,
) -> bool {
    ctx.simulations.iter().any(|simulation| {
        simulation.passed_at <= ctx.now
            && simulation.action_id == manifest.action_id
            && simulation.manifest_hash == *manifest_hash
            && simulation.policy_version == ctx.policy_version
    })
}

fn decision(
    manifest: &ActionManifest,
    manifest_hash: HashValue,
    ctx: &AdmissionContext,
    result: DecisionResult,
    matched_rules: Vec<String>,
    explanation: &str,
    followup: DecisionFollowup,
) -> PolicyDecision {
    PolicyDecision {
        decision_id: Uuid::new_v4().to_string(),
        action_id: manifest.action_id.clone(),
        manifest_hash,
        policy_version: ctx.policy_version.clone(),
        result,
        matched_rules,
        explanation: explanation.to_string(),
        required_review: followup.required_review,
        required_simulation: followup.required_simulation,
        created_at: ctx.now,
    }
}

#[derive(Clone, Debug, Default)]
struct DecisionFollowup {
    required_review: Option<String>,
    required_simulation: Option<String>,
}

impl DecisionFollowup {
    fn none() -> Self {
        Self::default()
    }

    fn review(review: String) -> Self {
        Self {
            required_review: Some(review),
            required_simulation: None,
        }
    }

    fn simulation(simulation: String) -> Self {
        Self {
            required_review: None,
            required_simulation: Some(simulation),
        }
    }
}
