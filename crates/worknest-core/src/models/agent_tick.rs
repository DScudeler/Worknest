//! Per-execution tick record. Append-only history that backs the
//! activity feed and the rolled-up runs_today/success_rate stats on
//! [`AgentDeployment`](super::agent_deployment::AgentDeployment).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::agent_deployment::AgentDeploymentId;
use super::ticket::TicketId;
use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentTickId(pub Uuid);

impl AgentTickId {
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

impl Default for AgentTickId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentTickId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TickOutcome {
    Success,
    Failure,
    Skipped,
}

impl std::fmt::Display for TickOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TickOutcome::Success => "Success",
            TickOutcome::Failure => "Failure",
            TickOutcome::Skipped => "Skipped",
        };
        f.write_str(s)
    }
}

impl TickOutcome {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "Success" => Self::Success,
            "Failure" => Self::Failure,
            "Skipped" => Self::Skipped,
            other => {
                return Err(CoreError::Validation(format!(
                    "Unknown TickOutcome: {other}"
                )))
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentTick {
    pub id: AgentTickId,
    pub deployment_id: AgentDeploymentId,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub outcome: Option<TickOutcome>,
    pub touched_ticket_id: Option<TicketId>,
    pub action_summary: Option<String>,
    pub error_message: Option<String>,
}
