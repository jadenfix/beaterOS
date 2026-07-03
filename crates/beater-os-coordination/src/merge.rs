use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::claim::{ClaimStatus, SliceClaim};
use crate::review::{ReviewAttestation, ReviewVerdict};

/// Tunable thresholds for the merge gate.
///
/// The defaults encode the project's non-negotiables: at least one independent
/// approval, green CI, and merged dependencies. Policy lives *outside the
/// model* (`final.md` 8.12): this is a deterministic function, not a prompt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergePolicy {
    /// Minimum number of distinct non-author `Approve` reviews required.
    pub min_independent_approvals: usize,
    /// Require the reviewed commit to have passing CI.
    pub require_ci_green: bool,
    /// Require every declared dependency slice to be merged first.
    pub require_dependencies_merged: bool,
}

impl Default for MergePolicy {
    fn default() -> Self {
        Self {
            min_independent_approvals: 1,
            require_ci_green: true,
            require_dependencies_merged: true,
        }
    }
}

/// Dynamic facts the gate needs that are not part of the claim itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MergeEvaluation {
    pub now: DateTime<Utc>,
    /// Principal proposing to perform the merge.
    pub merger_id: String,
    /// Exact commit proposed for merge; approvals must target this commit.
    pub commit_sha: String,
    /// Whether CI is green on `commit_sha`.
    pub ci_green: bool,
    pub policy_version: String,
    /// `(dependency_slice_id, status)`; `None` means the dependency is unclaimed.
    pub dependency_statuses: Vec<(String, Option<ClaimStatus>)>,
}

/// Deterministic result of a merge-gate evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeGateResult {
    Allowed,
    Denied,
}

/// A first-class, journaled record of a merge-gate evaluation.
///
/// Modeled on `PolicyDecision` in `beater-os-core`: it names the matched rules
/// and, when denied, the exact blocking reasons — so a denial is always
/// explainable (`final.md` 22.9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeGateDecision {
    pub decision_id: String,
    pub slice_id: String,
    pub merger_id: String,
    pub commit_sha: String,
    pub policy_version: String,
    pub result: MergeGateResult,
    pub independent_approvals: usize,
    #[serde(default)]
    pub approving_reviewers: Vec<String>,
    #[serde(default)]
    pub matched_rules: Vec<String>,
    #[serde(default)]
    pub blocking_reasons: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl MergeGateDecision {
    /// Whether this decision authorizes the merge.
    pub fn is_allowed(&self) -> bool {
        self.result == MergeGateResult::Allowed
    }
}

impl MergePolicy {
    /// Evaluate whether `eval.merger_id` may merge `claim` at `eval.commit_sha`.
    ///
    /// The evaluation is total: it never panics and never short-circuits on the
    /// first failure, so the returned decision lists *every* blocking reason at
    /// once. Reviews are filtered to `eval.commit_sha`, and only the latest
    /// verdict per reviewer counts, so a stale `RequestChanges` on an older
    /// commit does not block a fixed one.
    pub fn evaluate(
        &self,
        claim: &SliceClaim,
        reviews: &[ReviewAttestation],
        eval: &MergeEvaluation,
    ) -> MergeGateDecision {
        let mut matched_rules = Vec::new();
        let mut blocking_reasons = Vec::new();

        // Rule 1: the claim must be open for merge.
        match claim.status {
            ClaimStatus::InReview | ClaimStatus::Approved => {
                matched_rules.push("claim_open_for_merge".to_string());
            }
            ClaimStatus::Claimed => blocking_reasons.push(
                "slice is not yet in review; open a PR and move the claim to in_review".to_string(),
            ),
            ClaimStatus::Merged => blocking_reasons.push("slice is already merged".to_string()),
            ClaimStatus::Released => blocking_reasons.push("claim has been released".to_string()),
        }

        // Rule 2: never merge your own PR.
        if eval.merger_id == claim.claimant {
            blocking_reasons.push(format!(
                "self-merge rejected: {} authored this slice and cannot merge it",
                eval.merger_id
            ));
        } else {
            matched_rules.push("merger_is_not_author".to_string());
        }

        // Rule 3: independent approvals at the exact reviewed commit.
        let latest = latest_verdict_per_reviewer(reviews, &claim.slice_id, &eval.commit_sha);
        let mut approving_reviewers: Vec<String> = Vec::new();
        for (reviewer_id, attestation) in &latest {
            match attestation.verdict {
                ReviewVerdict::Approve if *reviewer_id != claim.claimant => {
                    approving_reviewers.push(reviewer_id.clone());
                }
                ReviewVerdict::Approve => {
                    // An approval from the author is never counted (defensive:
                    // construction already forbids self-review).
                }
                ReviewVerdict::RequestChanges => blocking_reasons.push(format!(
                    "reviewer {reviewer_id} requested changes on {}",
                    eval.commit_sha
                )),
                ReviewVerdict::Reject => blocking_reasons.push(format!(
                    "reviewer {reviewer_id} rejected {}",
                    eval.commit_sha
                )),
            }
        }
        let independent_approvals = approving_reviewers.len();
        if independent_approvals >= self.min_independent_approvals {
            matched_rules.push("independent_approval_threshold_met".to_string());
        } else {
            blocking_reasons.push(format!(
                "needs {} independent approval(s) at {}, found {}",
                self.min_independent_approvals, eval.commit_sha, independent_approvals
            ));
        }

        // Rule 4: CI must be green on the reviewed commit.
        if self.require_ci_green {
            if eval.ci_green {
                matched_rules.push("ci_green".to_string());
            } else {
                blocking_reasons.push(format!("CI is not green on {}", eval.commit_sha));
            }
        }

        // Rule 5: declared dependencies must be merged.
        if self.require_dependencies_merged {
            for (dependency, status) in &eval.dependency_statuses {
                match status {
                    Some(ClaimStatus::Merged) => {}
                    Some(other) => blocking_reasons.push(format!(
                        "dependency {dependency} is not merged (status: {other})"
                    )),
                    None => blocking_reasons
                        .push(format!("dependency {dependency} has not been claimed")),
                }
            }
            if eval
                .dependency_statuses
                .iter()
                .all(|(_, status)| matches!(status, Some(ClaimStatus::Merged)))
            {
                matched_rules.push("dependencies_merged".to_string());
            }
        }

        let result = if blocking_reasons.is_empty() {
            MergeGateResult::Allowed
        } else {
            MergeGateResult::Denied
        };

        MergeGateDecision {
            decision_id: Uuid::new_v4().to_string(),
            slice_id: claim.slice_id.clone(),
            merger_id: eval.merger_id.clone(),
            commit_sha: eval.commit_sha.clone(),
            policy_version: eval.policy_version.clone(),
            result,
            independent_approvals,
            approving_reviewers,
            matched_rules,
            blocking_reasons,
            created_at: eval.now,
        }
    }
}

/// Return the latest review per reviewer for a given slice and commit.
///
/// Ordering is deterministic: newest `created_at` wins, ties broken by the
/// lexicographically greater `review_id`. Keyed and returned via `BTreeMap` so
/// callers iterate reviewers in a stable order.
fn latest_verdict_per_reviewer<'a>(
    reviews: &'a [ReviewAttestation],
    slice_id: &str,
    commit_sha: &str,
) -> BTreeMap<String, &'a ReviewAttestation> {
    let mut latest: BTreeMap<String, &'a ReviewAttestation> = BTreeMap::new();
    for review in reviews
        .iter()
        .filter(|r| r.slice_id == slice_id && r.commit_sha == commit_sha)
    {
        latest
            .entry(review.reviewer_id.clone())
            .and_modify(|current| {
                let newer = (review.created_at, &review.review_id)
                    > (current.created_at, &current.review_id);
                if newer {
                    *current = review;
                }
            })
            .or_insert(review);
    }
    latest
}
