//! Comment domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{TicketId, UserId};
use crate::error::{CoreError, Result};

/// Unique identifier for comments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommentId(Uuid);

impl CommentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self> {
        Ok(Self(
            Uuid::parse_str(s).map_err(|e| CoreError::InvalidId(e.to_string()))?,
        ))
    }

}

impl Default for CommentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CommentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Comment on a ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: CommentId,
    pub ticket_id: TicketId,
    pub user_id: UserId,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Comment {
    /// Create a new comment
    pub fn new(ticket_id: TicketId, user_id: UserId, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: CommentId::new(),
            ticket_id,
            user_id,
            content,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate the comment
    pub fn validate(&self) -> Result<()> {
        if self.content.trim().is_empty() {
            return Err(CoreError::Validation(
                "Comment content cannot be empty".to_string(),
            ));
        }

        if self.content.len() > 10000 {
            return Err(CoreError::Validation(
                "Comment content cannot exceed 10000 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Update comment content
    pub fn update_content(&mut self, content: String) -> Result<()> {
        self.content = content;
        self.updated_at = Utc::now();
        self.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_comment() {
        let ticket_id = TicketId::new();
        let user_id = UserId::new();
        let comment = Comment::new(ticket_id, user_id, "Test comment".to_string());

        assert_eq!(comment.ticket_id, ticket_id);
        assert_eq!(comment.user_id, user_id);
        assert_eq!(comment.content, "Test comment");
        assert!(comment.validate().is_ok());
    }

    #[test]
    fn test_empty_content_validation() {
        let comment = Comment::new(TicketId::new(), UserId::new(), "   ".to_string());
        assert!(comment.validate().is_err());
    }

    #[test]
    fn test_content_too_long() {
        let long_content = "a".repeat(10001);
        let comment = Comment::new(TicketId::new(), UserId::new(), long_content);
        assert!(comment.validate().is_err());
    }

    #[test]
    fn test_update_content() {
        let mut comment = Comment::new(TicketId::new(), UserId::new(), "Original".to_string());
        let original_created = comment.created_at;

        assert!(comment.update_content("Updated".to_string()).is_ok());
        assert_eq!(comment.content, "Updated");
        assert!(comment.updated_at > original_created);
    }
}
