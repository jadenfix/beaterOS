use thiserror::Error;

/// Result alias for coordination-layer operations.
pub type CoordinationResult<T> = Result<T, CoordinationError>;

/// Errors raised by the multi-agent coordination kernel.
///
/// These map to the invariants beaterOS applies to its *own* parallel-agent
/// development (see `final.md` sections on capability grants, independent
/// review, and tamper-evident journals). Every failure is deterministic and
/// carries a human-legible reason, mirroring `PolicyDecision` explanations in
/// `beater-os-core`.
#[derive(Debug, Error)]
pub enum CoordinationError {
    #[error("json serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("core hashing failed: {0}")]
    Core(#[from] beater_os_core::BeaterOsError),

    #[error("unknown principal {principal_id}: register it before it can act")]
    UnknownPrincipal { principal_id: String },

    #[error("principal {principal_id} is already registered")]
    DuplicatePrincipal { principal_id: String },

    #[error("slice {slice_id} is already claimed by {claimant}")]
    SliceAlreadyClaimed { slice_id: String, claimant: String },

    #[error("branch {branch} is already bound to slice {slice_id}")]
    BranchAlreadyClaimed { branch: String, slice_id: String },

    #[error(
        "write-scope conflict: slice {slice_id} prefix {prefix} overlaps slice {other_slice_id} prefix {other_prefix}"
    )]
    WriteScopeConflict {
        slice_id: String,
        prefix: String,
        other_slice_id: String,
        other_prefix: String,
    },

    #[error("write scope for slice {slice_id} is empty: a claim must own at least one path prefix")]
    EmptyWriteScope { slice_id: String },

    #[error("write-scope prefix {prefix} for slice {slice_id} must be a relative repo path")]
    InvalidWriteScopePrefix { slice_id: String, prefix: String },

    #[error("unknown slice {slice_id}: claim it before acting on it")]
    UnknownSlice { slice_id: String },

    #[error("dependency {dependency} of slice {slice_id} is not merged (current status: {status})")]
    UnmetDependency {
        slice_id: String,
        dependency: String,
        status: String,
    },

    #[error("dependency {dependency} of slice {slice_id} has not been claimed")]
    UnknownDependency {
        slice_id: String,
        dependency: String,
    },

    #[error("self-review rejected: {reviewer_id} authored slice {slice_id} and cannot review it")]
    SelfReview {
        reviewer_id: String,
        slice_id: String,
    },

    #[error("self-merge rejected: {merger_id} authored slice {slice_id} and cannot merge it")]
    SelfMerge { merger_id: String, slice_id: String },

    #[error(
        "merge blocked for slice {slice_id}: gate did not authorize {merger_id} to merge (run evaluate_merge first)"
    )]
    MergeNotAuthorized { slice_id: String, merger_id: String },

    #[error("invalid claim transition for slice {slice_id}: {from} -> {to}")]
    InvalidClaimTransition {
        slice_id: String,
        from: String,
        to: String,
    },

    #[error("coordination ledger chain violation at seq {seq}: {reason}")]
    LedgerChain { seq: u64, reason: String },
}
