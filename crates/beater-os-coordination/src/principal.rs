use serde::{Deserialize, Serialize};

/// Whether a coordinating principal is an autonomous agent or a human.
///
/// beaterOS treats humans and agents as distinct principals (`final.md` 4.2),
/// but in this coordination kernel both have *equal authority* over any slice
/// they did not author. Authority here is not role-gated: every registered
/// principal may review and merge work, provided they are not its author. This
/// realizes the requirement that reviewers "have the power to do everything
/// with it, not just the one that wrote it."
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    Agent,
    Human,
}

/// A principal that can author, review, and merge slices.
///
/// Modeled on `AgentIdentity` in `beater-os-core`: a durable identity separate
/// from any single process, always attributable in the coordination journal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrincipal {
    /// Stable identifier, e.g. `"codex"`, `"claude"`, or a human handle.
    pub principal_id: String,
    pub kind: PrincipalKind,
    pub display_name: String,
    /// Free-form descriptor (e.g. `"kernel-contracts"`). Advisory only:
    /// it never widens or narrows what the principal is allowed to do.
    #[serde(default)]
    pub role: Option<String>,
}

impl AgentPrincipal {
    /// Construct an agent principal.
    pub fn agent(principal_id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            principal_id: principal_id.into(),
            kind: PrincipalKind::Agent,
            display_name: display_name.into(),
            role: None,
        }
    }

    /// Construct a human principal.
    pub fn human(principal_id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            principal_id: principal_id.into(),
            kind: PrincipalKind::Human,
            display_name: display_name.into(),
            role: None,
        }
    }

    /// Attach an advisory role label.
    #[must_use]
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }
}
