//! Ticket domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{project::ProjectId, user::UserId};

/// Ticket identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TicketId(pub Uuid);

impl TicketId {
    /// Create a new ticket ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for TicketId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TicketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Ticket type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketType {
    Task,
    Bug,
    Feature,
    Epic,
}

impl std::fmt::Display for TicketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketType::Task => write!(f, "Task"),
            TicketType::Bug => write!(f, "Bug"),
            TicketType::Feature => write!(f, "Feature"),
            TicketType::Epic => write!(f, "Epic"),
        }
    }
}

/// Ticket status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketStatus {
    Open,
    InProgress,
    Review,
    Done,
    Closed,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketStatus::Open => write!(f, "Open"),
            TicketStatus::InProgress => write!(f, "In Progress"),
            TicketStatus::Review => write!(f, "Review"),
            TicketStatus::Done => write!(f, "Done"),
            TicketStatus::Closed => write!(f, "Closed"),
        }
    }
}

/// Ticket priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "Low"),
            Priority::Medium => write!(f, "Medium"),
            Priority::High => write!(f, "High"),
            Priority::Critical => write!(f, "Critical"),
        }
    }
}

/// Ticket entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticket {
    pub id: TicketId,
    pub project_id: ProjectId,
    pub title: String,
    pub description: Option<String>,
    pub ticket_type: TicketType,
    pub status: TicketStatus,
    pub priority: Priority,
    pub assignee_id: Option<UserId>,
    pub created_by: UserId,
    pub due_date: Option<DateTime<Utc>>,
    pub estimate_hours: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Ticket {
    /// Create a new ticket
    pub fn new(
        project_id: ProjectId,
        title: String,
        ticket_type: TicketType,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: TicketId::new(),
            project_id,
            title,
            description: None,
            ticket_type,
            status: TicketStatus::Open,
            priority: Priority::Medium,
            assignee_id: None,
            created_by,
            due_date: None,
            estimate_hours: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate ticket data
    pub fn validate(&self) -> crate::Result<()> {
        if self.title.is_empty() {
            return Err(crate::CoreError::Validation(
                "Ticket title cannot be empty".to_string(),
            ));
        }

        if self.title.len() > 500 {
            return Err(crate::CoreError::Validation(
                "Ticket title must be 500 characters or less".to_string(),
            ));
        }

        if let Some(description) = &self.description {
            if description.len() > 10000 {
                return Err(crate::CoreError::Validation(
                    "Ticket description must be 10000 characters or less".to_string(),
                ));
            }
        }

        if let Some(hours) = self.estimate_hours {
            if hours < 0.0 {
                return Err(crate::CoreError::Validation(
                    "Estimate hours must be positive".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Update ticket status
    pub fn update_status(&mut self, status: TicketStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Assign ticket to a user
    pub fn assign_to(&mut self, user_id: UserId) {
        self.assignee_id = Some(user_id);
        self.updated_at = Utc::now();
    }

    /// Unassign ticket
    pub fn unassign(&mut self) {
        self.assignee_id = None;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_creation() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let ticket = Ticket::new(
            project_id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user_id,
        );
        assert_eq!(ticket.title, "Test Ticket");
        assert_eq!(ticket.status, TicketStatus::Open);
        assert_eq!(ticket.priority, Priority::Medium);
    }

    #[test]
    fn test_ticket_validation_success() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let ticket = Ticket::new(
            project_id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user_id,
        );
        assert!(ticket.validate().is_ok());
    }

    #[test]
    fn test_ticket_validation_empty_title() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let ticket = Ticket::new(project_id, "".to_string(), TicketType::Task, user_id);
        assert!(ticket.validate().is_err());
    }

    #[test]
    fn test_ticket_status_update() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let mut ticket = Ticket::new(
            project_id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user_id,
        );

        ticket.update_status(TicketStatus::InProgress);
        assert_eq!(ticket.status, TicketStatus::InProgress);
    }

    #[test]
    fn test_ticket_assignment() {
        let project_id = ProjectId::new();
        let user_id = UserId::new();
        let assignee_id = UserId::new();
        let mut ticket = Ticket::new(
            project_id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user_id,
        );

        assert!(ticket.assignee_id.is_none());

        ticket.assign_to(assignee_id);
        assert_eq!(ticket.assignee_id, Some(assignee_id));

        ticket.unassign();
        assert!(ticket.assignee_id.is_none());
    }
}
