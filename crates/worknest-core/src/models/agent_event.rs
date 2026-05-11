//! Lifecycle audit log per deployment. Append-only; records FSM transitions
//! and notable failures (separate from tick history so the UI can render
//! lifecycle events alongside individual ticks).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::agent_deployment::AgentDeploymentId;
use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentEventId(pub Uuid);

impl AgentEventId {
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

impl Default for AgentEventId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentEventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentEventKind {
    DeploymentCreated,
    IdentityRegistered,
    MembershipGranted,
    PersonaSnapshotted,
    WorkspaceProvisioned,
    WorktreeBootstrapped,
    TickScheduled,
    MarkedRunning,
    ActivationFailed,
    Suspended,
    Resumed,
    Retried,
    Stopped,
    TickFailedThreshold,
}

impl std::fmt::Display for AgentEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AgentEventKind::DeploymentCreated => "DeploymentCreated",
            AgentEventKind::IdentityRegistered => "IdentityRegistered",
            AgentEventKind::MembershipGranted => "MembershipGranted",
            AgentEventKind::PersonaSnapshotted => "PersonaSnapshotted",
            AgentEventKind::WorkspaceProvisioned => "WorkspaceProvisioned",
            AgentEventKind::WorktreeBootstrapped => "WorktreeBootstrapped",
            AgentEventKind::TickScheduled => "TickScheduled",
            AgentEventKind::MarkedRunning => "MarkedRunning",
            AgentEventKind::ActivationFailed => "ActivationFailed",
            AgentEventKind::Suspended => "Suspended",
            AgentEventKind::Resumed => "Resumed",
            AgentEventKind::Retried => "Retried",
            AgentEventKind::Stopped => "Stopped",
            AgentEventKind::TickFailedThreshold => "TickFailedThreshold",
        };
        f.write_str(s)
    }
}

impl AgentEventKind {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "DeploymentCreated" => Self::DeploymentCreated,
            "IdentityRegistered" => Self::IdentityRegistered,
            "MembershipGranted" => Self::MembershipGranted,
            "PersonaSnapshotted" => Self::PersonaSnapshotted,
            "WorkspaceProvisioned" => Self::WorkspaceProvisioned,
            "WorktreeBootstrapped" => Self::WorktreeBootstrapped,
            "TickScheduled" => Self::TickScheduled,
            "MarkedRunning" => Self::MarkedRunning,
            "ActivationFailed" => Self::ActivationFailed,
            "Suspended" => Self::Suspended,
            "Resumed" => Self::Resumed,
            "Retried" => Self::Retried,
            "Stopped" => Self::Stopped,
            "TickFailedThreshold" => Self::TickFailedThreshold,
            other => {
                return Err(CoreError::Validation(format!(
                    "Unknown AgentEventKind: {other}"
                )))
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentEvent {
    pub id: AgentEventId,
    pub deployment_id: AgentDeploymentId,
    pub kind: AgentEventKind,
    pub payload: serde_json::Value,
    pub message: String,
    pub at: DateTime<Utc>,
}
