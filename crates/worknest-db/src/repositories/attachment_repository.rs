//! Repository for Attachment operations

use rusqlite::{params, OptionalExtension, Row};
use std::sync::Arc;
use uuid::Uuid;

use crate::{DbError, DbPool, Repository, Result};
use worknest_core::models::{Attachment, AttachmentId, TicketId, UserId};

/// Repository for managing attachments (metadata only)
pub struct AttachmentRepository {
    pool: Arc<DbPool>,
}

impl AttachmentRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Find attachments by ticket ID
    pub fn find_by_ticket(&self, ticket_id: TicketId) -> Result<Vec<Attachment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, ticket_id, filename, file_size, mime_type, file_path, uploaded_by, created_at
             FROM attachments
             WHERE ticket_id = ?1
             ORDER BY created_at DESC"
        ).map_err(|e| DbError::Query(e.to_string()))?;

        let attachments = stmt
            .query_map(params![ticket_id.to_string()], row_to_attachment)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(attachments)
    }
}

impl Repository<Attachment, AttachmentId> for AttachmentRepository {
    fn find_by_id(&self, id: AttachmentId) -> Result<Option<Attachment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, ticket_id, filename, file_size, mime_type, file_path, uploaded_by, created_at
             FROM attachments
             WHERE id = ?1"
        ).map_err(|e| DbError::Query(e.to_string()))?;

        let attachment = stmt
            .query_row(params![id.to_string()], row_to_attachment)
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(attachment)
    }

    fn find_all(&self) -> Result<Vec<Attachment>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, ticket_id, filename, file_size, mime_type, file_path, uploaded_by, created_at
             FROM attachments
             ORDER BY created_at DESC"
        ).map_err(|e| DbError::Query(e.to_string()))?;

        let attachments = stmt
            .query_map([], row_to_attachment)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(attachments)
    }

    fn create(&self, attachment: &Attachment) -> Result<Attachment> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            "INSERT INTO attachments (id, ticket_id, filename, file_size, mime_type, file_path, uploaded_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                attachment.id.to_string(),
                attachment.ticket_id.to_string(),
                attachment.filename,
                attachment.file_size,
                attachment.mime_type,
                attachment.file_path,
                attachment.uploaded_by.to_string(),
                attachment.created_at.to_rfc3339(),
            ],
        ).map_err(|e| DbError::Query(e.to_string()))?;

        Ok(attachment.clone())
    }

    fn update(&self, _attachment: &Attachment) -> Result<Attachment> {
        // Attachments are immutable after creation for simplicity
        Err(DbError::Query("Attachments cannot be updated".to_string()))
    }

    fn delete(&self, id: AttachmentId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "DELETE FROM attachments WHERE id = ?1",
                params![id.to_string()],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound(format!(
                "Attachment with id {} not found",
                id
            )));
        }

        Ok(())
    }
}

fn row_to_attachment(row: &Row) -> rusqlite::Result<Attachment> {
    Ok(Attachment {
        id: AttachmentId::from_string(&row.get::<_, String>(0)?).unwrap(),
        ticket_id: TicketId::from_uuid(Uuid::parse_str(&row.get::<_, String>(1)?).unwrap()),
        filename: row.get(2)?,
        file_size: row.get(3)?,
        mime_type: row.get(4)?,
        file_path: row.get(5)?,
        uploaded_by: UserId::from_uuid(Uuid::parse_str(&row.get::<_, String>(6)?).unwrap()),
        created_at: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::init_memory_pool;
    use crate::migrations::run_migrations;
    use crate::repositories::{ProjectRepository, TicketRepository, UserRepository};
    use worknest_core::models::{Project, Ticket, TicketType, User};

    fn setup() -> (Arc<DbPool>, AttachmentRepository, UserId, TicketId) {
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

        let repo = AttachmentRepository::new(Arc::clone(&pool));
        (pool, repo, user.id, ticket.id)
    }

    #[test]
    fn test_create_attachment() {
        let (_pool, repo, user_id, ticket_id) = setup();
        let attachment = Attachment::new(
            ticket_id,
            "test.pdf".to_string(),
            1024,
            "application/pdf".to_string(),
            "/uploads/test.pdf".to_string(),
            user_id,
        );

        let result = repo.create(&attachment);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_by_id() {
        let (_pool, repo, user_id, ticket_id) = setup();
        let attachment = Attachment::new(
            ticket_id,
            "test.pdf".to_string(),
            1024,
            "application/pdf".to_string(),
            "/uploads/test.pdf".to_string(),
            user_id,
        );

        repo.create(&attachment).unwrap();
        let found = repo.find_by_id(attachment.id).unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().filename, "test.pdf");
    }

    #[test]
    fn test_delete_attachment() {
        let (_pool, repo, user_id, ticket_id) = setup();
        let attachment = Attachment::new(
            ticket_id,
            "test.pdf".to_string(),
            1024,
            "application/pdf".to_string(),
            "/uploads/test.pdf".to_string(),
            user_id,
        );

        repo.create(&attachment).unwrap();
        assert!(repo.delete(attachment.id).is_ok());
        assert!(repo.find_by_id(attachment.id).unwrap().is_none());
    }
}
