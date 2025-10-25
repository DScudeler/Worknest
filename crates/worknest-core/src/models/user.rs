//! User domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    /// Create a new user ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// User entity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user
    pub fn new(username: String, email: String) -> Self {
        let now = Utc::now();
        Self {
            id: UserId::new(),
            username,
            email,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate user data
    pub fn validate(&self) -> crate::Result<()> {
        if self.username.is_empty() {
            return Err(crate::CoreError::Validation(
                "Username cannot be empty".to_string(),
            ));
        }

        if self.username.len() < 3 {
            return Err(crate::CoreError::Validation(
                "Username must be at least 3 characters".to_string(),
            ));
        }

        if !self.email.contains('@') {
            return Err(crate::CoreError::Validation(
                "Invalid email format".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("testuser".to_string(), "test@example.com".to_string());
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_user_validation_success() {
        let user = User::new("testuser".to_string(), "test@example.com".to_string());
        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_user_validation_empty_username() {
        let user = User::new("".to_string(), "test@example.com".to_string());
        assert!(user.validate().is_err());
    }

    #[test]
    fn test_user_validation_short_username() {
        let user = User::new("ab".to_string(), "test@example.com".to_string());
        assert!(user.validate().is_err());
    }

    #[test]
    fn test_user_validation_invalid_email() {
        let user = User::new("testuser".to_string(), "invalid-email".to_string());
        assert!(user.validate().is_err());
    }
}
