//! Repository for Comment operations

use rusqlite::{params, OptionalExtension, Row};
use std::sync::Arc;
use uuid::Uuid;

use crate::{DbError, DbPool, Repository, Result};
use worknest_core::models::{Comment, CommentId, TicketId, UserId};

/// Repository for managing comments
pub struct CommentRepository {
    pool: Arc<DbPool>,
}

impl CommentRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Find comments by ticket ID
    pub fn find_by_ticket(&self, ticket_id: TicketId) -> Result<Vec<Comment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, ticket_id, user_id, content, created_at, updated_at
             FROM comments
             WHERE ticket_id = ?1
             ORDER BY created_at ASC",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let comments = stmt
            .query_map(params![ticket_id.to_string()], row_to_comment)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(comments)
    }

    /// Find comments by user ID
    pub fn find_by_user(&self, user_id: UserId) -> Result<Vec<Comment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, ticket_id, user_id, content, created_at, updated_at
             FROM comments
             WHERE user_id = ?1
             ORDER BY created_at DESC",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let comments = stmt
            .query_map(params![user_id.to_string()], row_to_comment)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(comments)
    }
}

impl Repository<Comment, CommentId> for CommentRepository {
    fn find_by_id(&self, id: CommentId) -> Result<Option<Comment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, ticket_id, user_id, content, created_at, updated_at
             FROM comments
             WHERE id = ?1",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let comment = stmt
            .query_row(params![id.to_string()], row_to_comment)
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(comment)
    }

    fn find_all(&self) -> Result<Vec<Comment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, ticket_id, user_id, content, created_at, updated_at
             FROM comments
             ORDER BY created_at DESC",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let comments = stmt
            .query_map([], row_to_comment)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(comments)
    }

    fn create(&self, comment: &Comment) -> Result<Comment> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            "INSERT INTO comments (id, ticket_id, user_id, content, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                comment.id.to_string(),
                comment.ticket_id.to_string(),
                comment.user_id.to_string(),
                comment.content,
                comment.created_at.to_rfc3339(),
                comment.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(comment.clone())
    }

    fn update(&self, comment: &Comment) -> Result<Comment> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE comments
             SET content = ?1, updated_at = ?2
             WHERE id = ?3",
                params![
                    comment.content,
                    comment.updated_at.to_rfc3339(),
                    comment.id.to_string(),
                ],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound(format!(
                "Comment with id {} not found",
                comment.id
            )));
        }

        Ok(comment.clone())
    }

    fn delete(&self, id: CommentId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "DELETE FROM comments WHERE id = ?1",
                params![id.to_string()],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound(format!(
                "Comment with id {} not found",
                id
            )));
        }

        Ok(())
    }
}

fn row_to_comment(row: &Row) -> rusqlite::Result<Comment> {
    Ok(Comment {
        id: CommentId::from_string(&row.get::<_, String>(0)?).unwrap(),
        ticket_id: TicketId::from_uuid(Uuid::parse_str(&row.get::<_, String>(1)?).unwrap()),
        user_id: UserId::from_uuid(Uuid::parse_str(&row.get::<_, String>(2)?).unwrap()),
        content: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::init_memory_pool;
    use crate::migrations::run_migrations;
    use crate::repositories::{ProjectRepository, TicketRepository, UserRepository};
    use worknest_core::models::{Project, Ticket, TicketType, User};

    fn setup() -> (Arc<DbPool>, CommentRepository, UserId, TicketId) {
        let pool = Arc::new(init_memory_pool().unwrap());
        run_migrations(&mut pool.get().unwrap()).unwrap();

        // Create user, project, and ticket for foreign key constraints
        let user_repo = UserRepository::new(Arc::clone(&pool));
        let project_repo = ProjectRepository::new(Arc::clone(&pool));
        let ticket_repo = TicketRepository::new(Arc::clone(&pool));

        let user = User::new("testuser".to_string(), "test@example.com".to_string());
        user_repo
            .create_with_password(&user, "password123")
            .unwrap();

        let project = Project::new("Test Project".to_string(), user.id);
        project_repo.create(&project).unwrap();

        let ticket = Ticket::new(
            project.id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user.id,
        );
        ticket_repo.create(&ticket).unwrap();

        let repo = CommentRepository::new(Arc::clone(&pool));
        (pool, repo, user.id, ticket.id)
    }

    #[test]
    fn test_create_comment() {
        let (_pool, repo, user_id, ticket_id) = setup();
        let comment = Comment::new(ticket_id, user_id, "Test comment".to_string());

        let result = repo.create(&comment);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_by_id() {
        let (_pool, repo, user_id, ticket_id) = setup();
        let comment = Comment::new(ticket_id, user_id, "Test comment".to_string());

        repo.create(&comment).unwrap();
        let found = repo.find_by_id(comment.id).unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().content, "Test comment");
    }

    #[test]
    fn test_update_comment() {
        let (_pool, repo, user_id, ticket_id) = setup();
        let mut comment = Comment::new(ticket_id, user_id, "Original".to_string());

        repo.create(&comment).unwrap();
        comment.update_content("Updated".to_string()).unwrap();
        repo.update(&comment).unwrap();

        let updated = repo.find_by_id(comment.id).unwrap().unwrap();
        assert_eq!(updated.content, "Updated");
    }

    #[test]
    fn test_delete_comment() {
        let (_pool, repo, user_id, ticket_id) = setup();
        let comment = Comment::new(ticket_id, user_id, "Test comment".to_string());

        repo.create(&comment).unwrap();
        assert!(repo.delete(comment.id).is_ok());
        assert!(repo.find_by_id(comment.id).unwrap().is_none());
    }
}
