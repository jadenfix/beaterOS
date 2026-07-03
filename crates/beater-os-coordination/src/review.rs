use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{CoordinationError, CoordinationResult};

/// Outcome of one independent review.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    /// The reviewer vouches for the slice at `commit_sha`.
    Approve,
    /// The reviewer found issues that must be fixed before merge.
    RequestChanges,
    /// The reviewer rejects the approach outright.
    Reject,
}

impl ReviewVerdict {
    /// Whether this verdict blocks a merge until resolved.
    pub fn is_blocking(self) -> bool {
        matches!(self, ReviewVerdict::RequestChanges | ReviewVerdict::Reject)
    }
}

/// A tamper-evident record that a principal *other than the author* reviewed a
/// slice.
///
/// This is the coordination analogue of `ApprovalEvidence` in
/// `beater-os-core`: approval is bound to a concrete subject (slice + commit),
/// a reviewer, an author, and a policy version, so it cannot be replayed for a
/// different change. The self-review invariant (`reviewer_id != author_id`) is
/// enforced at construction, matching `final.md` 13.14 ("bad approval prompts")
/// and the project rule that no one reviews their own work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewAttestation {
    pub review_id: String,
    pub slice_id: String,
    /// Reference to the PR or branch under review (e.g. `"pr:2"`).
    pub subject_ref: String,
    /// The exact commit the reviewer examined.
    pub commit_sha: String,
    pub reviewer_id: String,
    pub author_id: String,
    pub verdict: ReviewVerdict,
    pub summary: String,
    /// Named checklist items the reviewer verified (e.g. review gates).
    #[serde(default)]
    pub checklist: Vec<ReviewCheck>,
    pub policy_version: String,
    pub created_at: DateTime<Utc>,
}

/// One named check a reviewer explicitly confirmed or refuted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewCheck {
    pub name: String,
    pub passed: bool,
}

impl ReviewCheck {
    pub fn new(name: impl Into<String>, passed: bool) -> Self {
        Self {
            name: name.into(),
            passed,
        }
    }
}

/// Fields required to record a review. `review_id` is generated when omitted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewInput {
    #[serde(default)]
    pub review_id: Option<String>,
    pub slice_id: String,
    pub subject_ref: String,
    pub commit_sha: String,
    pub reviewer_id: String,
    pub author_id: String,
    pub verdict: ReviewVerdict,
    pub summary: String,
    #[serde(default)]
    pub checklist: Vec<ReviewCheck>,
    pub policy_version: String,
}

impl ReviewAttestation {
    /// Build an attestation, enforcing the self-review invariant.
    ///
    /// Returns [`CoordinationError::SelfReview`] when the reviewer is the
    /// author, so a self-approval can never enter the ledger.
    pub fn build(input: ReviewInput, created_at: DateTime<Utc>) -> CoordinationResult<Self> {
        if input.reviewer_id == input.author_id {
            return Err(CoordinationError::SelfReview {
                reviewer_id: input.reviewer_id,
                slice_id: input.slice_id,
            });
        }
        Ok(Self {
            review_id: input
                .review_id
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            slice_id: input.slice_id,
            subject_ref: input.subject_ref,
            commit_sha: input.commit_sha,
            reviewer_id: input.reviewer_id,
            author_id: input.author_id,
            verdict: input.verdict,
            summary: input.summary,
            checklist: input.checklist,
            policy_version: input.policy_version,
            created_at,
        })
    }
}
