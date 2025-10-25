//! Team domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::UserId;
use crate::error::{Result, CoreError};

/// Unique identifier for teams
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeamId(Uuid);

impl TeamId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self> {
        Ok(Self(
            Uuid::parse_str(s).map_err(|e| CoreError::InvalidId(e.to_string()))?,
        ))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for TeamId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TeamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Team/Organization for collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub description: Option<String>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Team {
    /// Create a new team
    pub fn new(name: String, description: Option<String>, created_by: UserId) -> Self {
        let now = Utc::now();
        Self {
            id: TeamId::new(),
            name,
            description,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate the team
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CoreError::Validation(
                "Team name cannot be empty".to_string(),
            ));
        }

        if self.name.len() < 2 {
            return Err(CoreError::Validation(
                "Team name must be at least 2 characters".to_string(),
            ));
        }

        if self.name.len() > 100 {
            return Err(CoreError::Validation(
                "Team name cannot exceed 100 characters".to_string(),
            ));
        }

        if let Some(desc) = &self.description {
            if desc.len() > 1000 {
                return Err(CoreError::Validation(
                    "Team description cannot exceed 1000 characters".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Update team details
    pub fn update(&mut self, name: Option<String>, description: Option<String>) -> Result<()> {
        if let Some(n) = name {
            self.name = n;
        }
        if description.is_some() {
            self.description = description;
        }
        self.updated_at = Utc::now();
        self.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_team() {
        let user_id = UserId::new();
        let team = Team::new(
            "Engineering".to_string(),
            Some("Development team".to_string()),
            user_id,
        );

        assert_eq!(team.name, "Engineering");
        assert_eq!(team.created_by, user_id);
        assert!(team.validate().is_ok());
    }

    #[test]
    fn test_team_name_validation() {
        let user_id = UserId::new();

        let empty_team = Team::new("".to_string(), None, user_id);
        assert!(empty_team.validate().is_err());

        let short_team = Team::new("A".to_string(), None, user_id);
        assert!(short_team.validate().is_err());

        let long_team = Team::new("a".repeat(101), None, user_id);
        assert!(long_team.validate().is_err());
    }

    #[test]
    fn test_team_description_validation() {
        let user_id = UserId::new();
        let team = Team::new("Team".to_string(), Some("a".repeat(1001)), user_id);
        assert!(team.validate().is_err());
    }

    #[test]
    fn test_update_team() {
        let user_id = UserId::new();
        let mut team = Team::new("Old Name".to_string(), None, user_id);
        let original_created = team.created_at;

        assert!(team
            .update(
                Some("New Name".to_string()),
                Some("Updated description".to_string())
            )
            .is_ok());

        assert_eq!(team.name, "New Name");
        assert_eq!(team.description, Some("Updated description".to_string()));
        assert!(team.updated_at > original_created);
    }
}
