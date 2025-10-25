//! Role and Permission domain models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{CoreError, Result};

/// Unique identifier for roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(Uuid);

impl RoleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self> {
        Ok(Self(
            Uuid::parse_str(s).map_err(|e| CoreError::InvalidId(e.to_string()))?,
        ))
    }

}

impl Default for RoleId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionId(Uuid);

impl PermissionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self> {
        Ok(Self(
            Uuid::parse_str(s).map_err(|e| CoreError::InvalidId(e.to_string()))?,
        ))
    }

}

impl Default for PermissionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PermissionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// User role in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: RoleId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Role {
    /// Create a new role
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: RoleId::new(),
            name,
            description,
            created_at: Utc::now(),
        }
    }

    /// Validate the role
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CoreError::Validation(
                "Role name cannot be empty".to_string(),
            ));
        }

        if self.name.len() < 2 {
            return Err(CoreError::Validation(
                "Role name must be at least 2 characters".to_string(),
            ));
        }

        if self.name.len() > 50 {
            return Err(CoreError::Validation(
                "Role name cannot exceed 50 characters".to_string(),
            ));
        }

        Ok(())
    }
}

/// Permission for a specific action on a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: PermissionId,
    pub name: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Permission {
    /// Create a new permission
    pub fn new(
        name: String,
        resource: String,
        action: String,
        description: Option<String>,
    ) -> Self {
        Self {
            id: PermissionId::new(),
            name,
            resource,
            action,
            description,
            created_at: Utc::now(),
        }
    }

    /// Validate the permission
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(CoreError::Validation(
                "Permission name cannot be empty".to_string(),
            ));
        }

        if self.resource.trim().is_empty() {
            return Err(CoreError::Validation(
                "Resource cannot be empty".to_string(),
            ));
        }

        if self.action.trim().is_empty() {
            return Err(CoreError::Validation("Action cannot be empty".to_string()));
        }

        // Permission name should follow format: resource:action
        if !self.name.contains(':') {
            return Err(CoreError::Validation(
                "Permission name should follow format 'resource:action'".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if permission matches a resource and action
    pub fn matches(&self, resource: &str, action: &str) -> bool {
        self.resource == resource && self.action == action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_role() {
        let role = Role::new("Admin".to_string(), Some("Administrator role".to_string()));
        assert_eq!(role.name, "Admin");
        assert!(role.validate().is_ok());
    }

    #[test]
    fn test_role_name_validation() {
        let empty_role = Role::new("".to_string(), None);
        assert!(empty_role.validate().is_err());

        let short_role = Role::new("A".to_string(), None);
        assert!(short_role.validate().is_err());

        let long_role = Role::new("a".repeat(51), None);
        assert!(long_role.validate().is_err());
    }

    #[test]
    fn test_create_permission() {
        let perm = Permission::new(
            "project:create".to_string(),
            "project".to_string(),
            "create".to_string(),
            Some("Create projects".to_string()),
        );
        assert_eq!(perm.name, "project:create");
        assert_eq!(perm.resource, "project");
        assert_eq!(perm.action, "create");
        assert!(perm.validate().is_ok());
    }

    #[test]
    fn test_permission_validation() {
        let invalid_perm = Permission::new(
            "invalid_format".to_string(),
            "project".to_string(),
            "create".to_string(),
            None,
        );
        assert!(invalid_perm.validate().is_err());

        let empty_resource = Permission::new(
            "project:create".to_string(),
            "".to_string(),
            "create".to_string(),
            None,
        );
        assert!(empty_resource.validate().is_err());
    }

    #[test]
    fn test_permission_matches() {
        let perm = Permission::new(
            "project:create".to_string(),
            "project".to_string(),
            "create".to_string(),
            None,
        );

        assert!(perm.matches("project", "create"));
        assert!(!perm.matches("project", "delete"));
        assert!(!perm.matches("ticket", "create"));
    }
}
