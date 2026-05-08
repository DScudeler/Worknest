//! Per-project agent deployment model
//!
//! An `AgentDeployment` is the runtime instance of a [`Persona`] inside a
//! project. It owns the snapshotted persona config, the cron schedule, the
//! activation FSM state, and the materialised tick statistics displayed in
//! the deployments tab.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::persona::{AgentModel, Capability, PersonaId};
use super::project::ProjectId;
use super::ticket::TicketId;
use super::user::UserId;
use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentDeploymentId(pub Uuid);

impl AgentDeploymentId {
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

impl Default for AgentDeploymentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentDeploymentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status enum spanning the full deployment lifecycle.
///
/// The activation pipeline drives a deployment through `Pending` →
/// `Registering` → `Granting` → `Snapshotting` → `Provisioning` →
/// `Scheduling` → `Running`. User controls move it between `Running`,
/// `Paused`, and `Stopped`. `Idle` is reserved for future "no work to do"
/// reporting (the scheduler does not assign it today). `Error` is the
/// terminal-until-retry failure state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Pending,
    Registering,
    Granting,
    Snapshotting,
    Provisioning,
    Bootstrapping,
    Scheduling,
    Running,
    Paused,
    Idle,
    Stopped,
    Error,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AgentStatus::Pending => "Pending",
            AgentStatus::Registering => "Registering",
            AgentStatus::Granting => "Granting",
            AgentStatus::Snapshotting => "Snapshotting",
            AgentStatus::Provisioning => "Provisioning",
            AgentStatus::Bootstrapping => "Bootstrapping",
            AgentStatus::Scheduling => "Scheduling",
            AgentStatus::Running => "Running",
            AgentStatus::Paused => "Paused",
            AgentStatus::Idle => "Idle",
            AgentStatus::Stopped => "Stopped",
            AgentStatus::Error => "Error",
        };
        f.write_str(s)
    }
}

impl AgentStatus {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "Pending" => Self::Pending,
            "Registering" => Self::Registering,
            "Granting" => Self::Granting,
            "Snapshotting" => Self::Snapshotting,
            "Provisioning" => Self::Provisioning,
            "Bootstrapping" => Self::Bootstrapping,
            "Scheduling" => Self::Scheduling,
            "Running" => Self::Running,
            "Paused" => Self::Paused,
            "Idle" => Self::Idle,
            "Stopped" => Self::Stopped,
            "Error" => Self::Error,
            other => {
                return Err(CoreError::Validation(format!(
                    "Unknown AgentStatus: {other}"
                )))
            },
        })
    }
}

/// Activation pipeline step. Used both for the FSM dispatch and for the
/// `last_error_step` recovery anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivationStep {
    RegisterIdentity,
    GrantMembership,
    SnapshotPersona,
    ProvisionWorkspace,
    BootstrapWorktree,
    ScheduleNextTick,
    MarkRunning,
}

impl std::fmt::Display for ActivationStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ActivationStep::RegisterIdentity => "RegisterIdentity",
            ActivationStep::GrantMembership => "GrantMembership",
            ActivationStep::SnapshotPersona => "SnapshotPersona",
            ActivationStep::ProvisionWorkspace => "ProvisionWorkspace",
            ActivationStep::BootstrapWorktree => "BootstrapWorktree",
            ActivationStep::ScheduleNextTick => "ScheduleNextTick",
            ActivationStep::MarkRunning => "MarkRunning",
        };
        f.write_str(s)
    }
}

impl ActivationStep {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "RegisterIdentity" => Self::RegisterIdentity,
            "GrantMembership" => Self::GrantMembership,
            "SnapshotPersona" => Self::SnapshotPersona,
            "ProvisionWorkspace" => Self::ProvisionWorkspace,
            "BootstrapWorktree" => Self::BootstrapWorktree,
            "ScheduleNextTick" => Self::ScheduleNextTick,
            "MarkRunning" => Self::MarkRunning,
            other => {
                return Err(CoreError::Validation(format!(
                    "Unknown ActivationStep: {other}"
                )))
            },
        })
    }

    /// The status the deployment must be in when *entering* this step.
    /// Drives the guarded `transition_status(from, to)` on retry/recovery.
    pub fn entry_status(&self) -> AgentStatus {
        match self {
            ActivationStep::RegisterIdentity => AgentStatus::Pending,
            ActivationStep::GrantMembership => AgentStatus::Granting,
            ActivationStep::SnapshotPersona => AgentStatus::Snapshotting,
            ActivationStep::BootstrapWorktree => AgentStatus::Bootstrapping,
            ActivationStep::ProvisionWorkspace => AgentStatus::Provisioning,
            ActivationStep::ScheduleNextTick => AgentStatus::Scheduling,
            ActivationStep::MarkRunning => AgentStatus::Pending, // unused; MarkRunning is internal
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDeployment {
    pub id: AgentDeploymentId,
    pub project_id: ProjectId,
    pub persona_id: PersonaId,
    pub agent_user_id: Option<UserId>,

    pub snapshot_name: Option<String>,
    pub snapshot_role: Option<String>,
    pub snapshot_tone: Option<String>,
    pub snapshot_expertise: Vec<String>,
    pub snapshot_instructions: Option<String>,
    pub snapshot_capabilities: Vec<Capability>,
    pub snapshot_model: Option<AgentModel>,
    pub snapshot_taken_at: Option<DateTime<Utc>>,

    pub workspace_path: Option<String>,
    pub cron_expression: String,
    pub next_tick_at: Option<DateTime<Utc>>,
    pub tick_locked_at: Option<DateTime<Utc>>,
    pub tick_lock_token: Option<String>,

    pub status: AgentStatus,
    pub last_error_step: Option<ActivationStep>,
    pub error_message: Option<String>,
    pub error_count: i32,
    pub current_ticket_id: Option<TicketId>,

    pub runs_today: i32,
    pub touched_this_week: i32,
    pub success_rate: f32,
    pub last_activity_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_statuses_round_trip() {
        for s in [
            AgentStatus::Pending,
            AgentStatus::Registering,
            AgentStatus::Granting,
            AgentStatus::Snapshotting,
            AgentStatus::Provisioning,
            AgentStatus::Scheduling,
            AgentStatus::Running,
            AgentStatus::Paused,
            AgentStatus::Idle,
            AgentStatus::Stopped,
            AgentStatus::Error,
        ] {
            assert_eq!(AgentStatus::parse(&s.to_string()).unwrap(), s);
        }
    }

    #[test]
    fn rejects_unknown_status() {
        assert!(AgentStatus::parse("Bogus").is_err());
    }

    #[test]
    fn parses_all_steps_round_trip() {
        for s in [
            ActivationStep::RegisterIdentity,
            ActivationStep::GrantMembership,
            ActivationStep::SnapshotPersona,
            ActivationStep::ProvisionWorkspace,
            ActivationStep::ScheduleNextTick,
            ActivationStep::MarkRunning,
        ] {
            assert_eq!(ActivationStep::parse(&s.to_string()).unwrap(), s);
        }
    }
}
