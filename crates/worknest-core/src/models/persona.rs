//! Persona domain model
//!
//! Personas are the workspace-shared catalogue of agent definitions. They are
//! visible to every authenticated user (no per-user ownership filter) and are
//! seeded with 9 entries by migration V7. A persona becomes runtime context for
//! a per-project [`AgentDeployment`](super::agent_deployment::AgentDeployment),
//! at which point its config is *snapshotted* into the deployment row so that
//! later edits to the persona do not disturb in-flight ticks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonaId(pub Uuid);

impl PersonaId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn from_string(s: &str) -> Result<Self> {
        Ok(Self(
            Uuid::parse_str(s).map_err(|e| CoreError::InvalidId(e.to_string()))?,
        ))
    }
}

impl Default for PersonaId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PersonaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Capability wire IDs used both in `Persona.capabilities` and in deployment
/// snapshots. The variants match the 8 actions the design exposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    Comment,
    Label,
    Assign,
    SetPriority,
    SetStatus,
    Attach,
    CreateTicket,
    Close,
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::Comment => write!(f, "Comment"),
            Capability::Label => write!(f, "Label"),
            Capability::Assign => write!(f, "Assign"),
            Capability::SetPriority => write!(f, "SetPriority"),
            Capability::SetStatus => write!(f, "SetStatus"),
            Capability::Attach => write!(f, "Attach"),
            Capability::CreateTicket => write!(f, "CreateTicket"),
            Capability::Close => write!(f, "Close"),
        }
    }
}

/// The three model tiers offered by the design. Each matches a Claude model
/// family; the actual model id is resolved at tick-spawn time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentModel {
    Haiku,
    Sonnet,
    Opus,
}

impl std::fmt::Display for AgentModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentModel::Haiku => write!(f, "Haiku"),
            AgentModel::Sonnet => write!(f, "Sonnet"),
            AgentModel::Opus => write!(f, "Opus"),
        }
    }
}

impl AgentModel {
    /// The wire model id passed to `claude --model <id>` for this tier.
    /// Resolves each family to its current latest minor release. Update
    /// these constants when a new minor lands.
    pub fn wire_id(self) -> &'static str {
        match self {
            AgentModel::Haiku => "claude-haiku-4-5",
            AgentModel::Sonnet => "claude-sonnet-4-6",
            AgentModel::Opus => "claude-opus-4-7",
        }
    }
}

/// Workspace-shared persona definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Persona {
    pub id: PersonaId,
    /// Stable, unique slug. Drives the deterministic agent identity email
    /// (`agent-<slug>@worknest.local`) and the persona overlay file picked
    /// at activation time.
    pub slug: String,
    pub name: String,
    pub emoji: String,
    pub color: String,
    pub description: String,
    pub role: String,
    pub tone: String,
    pub expertise: Vec<String>,
    pub instructions: String,
    pub capabilities: Vec<Capability>,
    pub model: AgentModel,
    /// 5-field cron in UTC, e.g. `*/30 9-18 * * 1-5`.
    pub default_cron: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Persona {
    pub fn validate(&self) -> Result<()> {
        if self.slug.is_empty() {
            return Err(CoreError::Validation("Persona slug cannot be empty".into()));
        }
        if self.name.is_empty() {
            return Err(CoreError::Validation("Persona name cannot be empty".into()));
        }
        if self.description.is_empty() {
            return Err(CoreError::Validation(
                "Persona description cannot be empty".into(),
            ));
        }
        if self.instructions.is_empty() {
            return Err(CoreError::Validation(
                "Persona instructions cannot be empty".into(),
            ));
        }
        if self.default_cron.split_whitespace().count() != 5 {
            return Err(CoreError::Validation(
                "default_cron must be a 5-field cron expression".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Persona {
        let now = Utc::now();
        Persona {
            id: PersonaId::new(),
            slug: "triage".into(),
            name: "Triage".into(),
            emoji: "🛎️".into(),
            color: "#bae6fd".into(),
            description: "Sorts incoming tickets.".into(),
            role: "Triage operator".into(),
            tone: "Concise".into(),
            expertise: vec!["classification".into()],
            instructions: "Read inbox; assign labels.".into(),
            capabilities: vec![Capability::Comment, Capability::Label],
            model: AgentModel::Haiku,
            default_cron: "*/30 9-18 * * 1-5".into(),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn validates_ok() {
        assert!(sample().validate().is_ok());
    }

    #[test]
    fn rejects_empty_slug() {
        let mut p = sample();
        p.slug = String::new();
        assert!(p.validate().is_err());
    }

    #[test]
    fn rejects_bad_cron() {
        let mut p = sample();
        p.default_cron = "not a cron".into();
        assert!(p.validate().is_err());
    }

    #[test]
    fn wire_ids_resolve_each_tier_to_a_versioned_model() {
        // Update these expectations when we move to a newer minor release.
        assert_eq!(AgentModel::Haiku.wire_id(), "claude-haiku-4-5");
        assert_eq!(AgentModel::Sonnet.wire_id(), "claude-sonnet-4-6");
        assert_eq!(AgentModel::Opus.wire_id(), "claude-opus-4-7");
    }
}
