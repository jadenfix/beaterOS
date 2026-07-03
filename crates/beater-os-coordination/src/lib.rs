//! Multi-agent coordination kernel for beaterOS development.
//!
//! `final.md` is built by several agents working the same repository in
//! parallel. This crate applies beaterOS's own operating principles — bounded
//! authority, policy outside the model, tamper-evident journals, and
//! independent review — to that development process itself.
//!
//! It provides the "communication loop between agents" as inspectable,
//! deterministic contracts rather than conversation:
//!
//! - [`AgentPrincipal`]: who may author, review, and merge. Every principal has
//!   equal authority over work it did not author.
//! - [`SliceClaim`] + [`WriteScope`]: bounded, *disjoint* ownership of repo
//!   paths, so parallel agents cannot clobber each other.
//! - [`ReviewAttestation`]: tamper-evident, commit-bound evidence that someone
//!   *other than the author* reviewed a slice.
//! - [`MergePolicy`]: a deterministic gate enforcing the project's rule that no
//!   one merges their own PR (author ≠ reviewer ≠ merger), CI is green, and
//!   dependencies are merged first.
//! - [`CoordinationLedger`]: an append-only, hash-chained record of every
//!   claim, review, and merge decision.
//!
//! [`Coordinator`] composes these into a single, replayable source of truth.
//! It reuses `beater-os-core` for hashing so there is one cryptographic
//! implementation across the workspace.

mod claim;
mod coordinator;
mod error;
mod journal;
mod merge;
mod principal;
mod review;

pub use claim::{ClaimStatus, SliceClaim, WriteScope};
pub use coordinator::{ClaimInput, Coordinator};
pub use error::{CoordinationError, CoordinationResult};
pub use journal::{CoordinationEvent, CoordinationLedger, GENESIS_HASH, LedgerRecord};
pub use merge::{MergeEvaluation, MergeGateDecision, MergeGateResult, MergePolicy};
pub use principal::{AgentPrincipal, PrincipalKind};
pub use review::{ReviewAttestation, ReviewCheck, ReviewInput, ReviewVerdict};
