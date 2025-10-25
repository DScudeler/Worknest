//! Project domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::user::UserId;

/// Project identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    /// Create a new project ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Parse from a string
    pub fn from_string(s: &str) -> crate::Result<Self> {
        Ok(Self(
            Uuid::parse_str(s).map_err(|e| crate::CoreError::InvalidId(e.to_string()))?,
        ))
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Project entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub archived: bool,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Create a new project
    pub fn new(name: String, created_by: UserId) -> Self {
        let now = Utc::now();
        Self {
            id: ProjectId::new(),
            name,
            description: None,
            color: None,
            archived: false,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate project data
    pub fn validate(&self) -> crate::Result<()> {
        if self.name.is_empty() {
            return Err(crate::CoreError::Validation(
                "Project name cannot be empty".to_string(),
            ));
        }

        if self.name.len() > 255 {
            return Err(crate::CoreError::Validation(
                "Project name must be 255 characters or less".to_string(),
            ));
        }

        if let Some(description) = &self.description {
            if description.len() > 5000 {
                return Err(crate::CoreError::Validation(
                    "Project description must be 5000 characters or less".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Archive this project
    pub fn archive(&mut self) {
        self.archived = true;
        self.updated_at = Utc::now();
    }

    /// Unarchive this project
    pub fn unarchive(&mut self) {
        self.archived = false;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let user_id = UserId::new();
        let project = Project::new("Test Project".to_string(), user_id);
        assert_eq!(project.name, "Test Project");
        assert!(!project.archived);
    }

    #[test]
    fn test_project_validation_success() {
        let user_id = UserId::new();
        let project = Project::new("Test Project".to_string(), user_id);
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_project_validation_empty_name() {
        let user_id = UserId::new();
        let project = Project::new("".to_string(), user_id);
        assert!(project.validate().is_err());
    }

    #[test]
    fn test_project_archive() {
        let user_id = UserId::new();
        let mut project = Project::new("Test Project".to_string(), user_id);
        assert!(!project.archived);

        project.archive();
        assert!(project.archived);

        project.unarchive();
        assert!(!project.archived);
    }
}
