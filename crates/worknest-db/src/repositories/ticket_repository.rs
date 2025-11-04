//! Ticket repository implementation

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};
use std::sync::Arc;
use uuid::Uuid;

use worknest_core::models::{
    Priority, ProjectId, Ticket, TicketId, TicketStatus, TicketType, UserId,
};

use crate::{connection::DbPool, repository::Repository, DbError, Result};

/// Ticket repository for database operations
pub struct TicketRepository {
    pool: Arc<DbPool>,
}

impl TicketRepository {
    /// Create a new TicketRepository
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Find all tickets for a project
    pub fn find_by_project(&self, project_id: ProjectId) -> Result<Vec<Ticket>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, ticket_type, status, priority,
                        assignee_id, created_by, due_date, estimate_hours, created_at, updated_at
                 FROM tickets WHERE project_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let tickets = stmt
            .query_map(params![project_id.0.to_string()], row_to_ticket)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(tickets)
    }

    /// Find all tickets assigned to a user
    pub fn find_by_assignee(&self, assignee_id: UserId) -> Result<Vec<Ticket>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, ticket_type, status, priority,
                        assignee_id, created_by, due_date, estimate_hours, created_at, updated_at
                 FROM tickets WHERE assignee_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let tickets = stmt
            .query_map(params![assignee_id.0.to_string()], row_to_ticket)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(tickets)
    }

    /// Find tickets by status
    pub fn find_by_status(&self, status: TicketStatus) -> Result<Vec<Ticket>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, ticket_type, status, priority,
                        assignee_id, created_by, due_date, estimate_hours, created_at, updated_at
                 FROM tickets WHERE status = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let tickets = stmt
            .query_map(params![status_to_string(&status)], row_to_ticket)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(tickets)
    }

    /// Find tickets by project and status
    pub fn find_by_project_and_status(
        &self,
        project_id: ProjectId,
        status: TicketStatus,
    ) -> Result<Vec<Ticket>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, ticket_type, status, priority,
                        assignee_id, created_by, due_date, estimate_hours, created_at, updated_at
                 FROM tickets WHERE project_id = ?1 AND status = ?2 ORDER BY created_at DESC",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let tickets = stmt
            .query_map(
                params![project_id.0.to_string(), status_to_string(&status)],
                row_to_ticket,
            )
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(tickets)
    }

    /// Find tickets created by a user
    pub fn find_by_creator(&self, creator_id: UserId) -> Result<Vec<Ticket>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, ticket_type, status, priority,
                        assignee_id, created_by, due_date, estimate_hours, created_at, updated_at
                 FROM tickets WHERE created_by = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let tickets = stmt
            .query_map(params![creator_id.0.to_string()], row_to_ticket)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(tickets)
    }

    /// Update ticket status
    pub fn update_status(&self, ticket_id: TicketId, status: TicketStatus) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE tickets SET status = ?1, updated_at = ?2 WHERE id = ?3",
                params![
                    status_to_string(&status),
                    Utc::now().to_rfc3339(),
                    ticket_id.0.to_string()
                ],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("Ticket not found".to_string()));
        }

        Ok(())
    }

    /// Assign ticket to a user
    pub fn assign(&self, ticket_id: TicketId, assignee_id: UserId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE tickets SET assignee_id = ?1, updated_at = ?2 WHERE id = ?3",
                params![
                    assignee_id.0.to_string(),
                    Utc::now().to_rfc3339(),
                    ticket_id.0.to_string()
                ],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("Ticket not found".to_string()));
        }

        Ok(())
    }

    /// Unassign ticket
    pub fn unassign(&self, ticket_id: TicketId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE tickets SET assignee_id = NULL, updated_at = ?1 WHERE id = ?2",
                params![Utc::now().to_rfc3339(), ticket_id.0.to_string()],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("Ticket not found".to_string()));
        }

        Ok(())
    }

    /// Search tickets using full-text search
    pub fn search(&self, query: &str, project_id: Option<ProjectId>) -> Result<Vec<Ticket>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let sql = if project_id.is_some() {
            "SELECT t.id, t.project_id, t.title, t.description, t.ticket_type, t.status, t.priority,
                    t.assignee_id, t.created_by, t.due_date, t.estimate_hours, t.created_at, t.updated_at
             FROM tickets t
             JOIN tickets_fts fts ON t.id = fts.ticket_id
             WHERE fts.tickets_fts MATCH ?1 AND t.project_id = ?2
             ORDER BY t.created_at DESC"
        } else {
            "SELECT t.id, t.project_id, t.title, t.description, t.ticket_type, t.status, t.priority,
                    t.assignee_id, t.created_by, t.due_date, t.estimate_hours, t.created_at, t.updated_at
             FROM tickets t
             JOIN tickets_fts fts ON t.id = fts.ticket_id
             WHERE fts.tickets_fts MATCH ?1
             ORDER BY t.created_at DESC"
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| DbError::Query(e.to_string()))?;

        let tickets = if let Some(proj_id) = project_id {
            stmt.query_map(params![query, proj_id.0.to_string()], row_to_ticket)
                .map_err(|e| DbError::Query(e.to_string()))?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| DbError::Query(e.to_string()))?
        } else {
            stmt.query_map(params![query], row_to_ticket)
                .map_err(|e| DbError::Query(e.to_string()))?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| DbError::Query(e.to_string()))?
        };

        Ok(tickets)
    }
}

impl Repository<Ticket, TicketId> for TicketRepository {
    fn find_by_id(&self, id: TicketId) -> Result<Option<Ticket>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, ticket_type, status, priority,
                        assignee_id, created_by, due_date, estimate_hours, created_at, updated_at
                 FROM tickets WHERE id = ?1",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let ticket = stmt
            .query_row(params![id.0.to_string()], row_to_ticket)
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(ticket)
    }

    fn find_all(&self) -> Result<Vec<Ticket>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, ticket_type, status, priority,
                        assignee_id, created_by, due_date, estimate_hours, created_at, updated_at
                 FROM tickets ORDER BY created_at DESC",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let tickets = stmt
            .query_map([], row_to_ticket)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(tickets)
    }

    fn create(&self, entity: &Ticket) -> Result<Ticket> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            "INSERT INTO tickets (id, project_id, title, description, ticket_type, status, priority,
                                  assignee_id, created_by, due_date, estimate_hours, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                entity.id.0.to_string(),
                entity.project_id.0.to_string(),
                entity.title,
                entity.description,
                ticket_type_to_string(&entity.ticket_type),
                status_to_string(&entity.status),
                priority_to_string(&entity.priority),
                entity.assignee_id.map(|id| id.0.to_string()),
                entity.created_by.0.to_string(),
                entity.due_date.map(|d| d.to_rfc3339()),
                entity.estimate_hours,
                entity.created_at.to_rfc3339(),
                entity.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(entity.clone())
    }

    fn update(&self, entity: &Ticket) -> Result<Ticket> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE tickets SET title = ?1, description = ?2, ticket_type = ?3, status = ?4, priority = ?5,
                                    assignee_id = ?6, due_date = ?7, estimate_hours = ?8, updated_at = ?9
                 WHERE id = ?10",
                params![
                    entity.title,
                    entity.description,
                    ticket_type_to_string(&entity.ticket_type),
                    status_to_string(&entity.status),
                    priority_to_string(&entity.priority),
                    entity.assignee_id.map(|id| id.0.to_string()),
                    entity.due_date.map(|d| d.to_rfc3339()),
                    entity.estimate_hours,
                    Utc::now().to_rfc3339(),
                    entity.id.0.to_string(),
                ],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("Ticket not found".to_string()));
        }

        Ok(entity.clone())
    }

    fn delete(&self, id: TicketId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "DELETE FROM tickets WHERE id = ?1",
                params![id.0.to_string()],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("Ticket not found".to_string()));
        }

        Ok(())
    }
}

/// Convert a database row to a Ticket
fn row_to_ticket(row: &Row) -> rusqlite::Result<Ticket> {
    let id_str: String = row.get(0)?;
    let id = TicketId::from_uuid(Uuid::parse_str(&id_str).unwrap());

    let project_id_str: String = row.get(1)?;
    let project_id = ProjectId::from_uuid(Uuid::parse_str(&project_id_str).unwrap());

    let ticket_type_str: String = row.get(4)?;
    let ticket_type = string_to_ticket_type(&ticket_type_str);

    let status_str: String = row.get(5)?;
    let status = string_to_status(&status_str);

    let priority_str: String = row.get(6)?;
    let priority = string_to_priority(&priority_str);

    let assignee_id: Option<String> = row.get(7)?;
    let assignee_id = assignee_id.map(|s| UserId::from_uuid(Uuid::parse_str(&s).unwrap()));

    let created_by_str: String = row.get(8)?;
    let created_by = UserId::from_uuid(Uuid::parse_str(&created_by_str).unwrap());

    let due_date: Option<String> = row.get(9)?;
    let due_date = due_date.map(|s| {
        DateTime::parse_from_rfc3339(&s)
            .unwrap()
            .with_timezone(&Utc)
    });

    let created_at_str: String = row.get(11)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .unwrap()
        .with_timezone(&Utc);

    let updated_at_str: String = row.get(12)?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .unwrap()
        .with_timezone(&Utc);

    Ok(Ticket {
        id,
        project_id,
        title: row.get(2)?,
        description: row.get(3)?,
        ticket_type,
        status,
        priority,
        assignee_id,
        created_by,
        due_date,
        estimate_hours: row.get(10)?,
        created_at,
        updated_at,
    })
}

// Helper functions for enum conversions
fn ticket_type_to_string(ticket_type: &TicketType) -> String {
    match ticket_type {
        TicketType::Task => "Task".to_string(),
        TicketType::Bug => "Bug".to_string(),
        TicketType::Feature => "Feature".to_string(),
        TicketType::Epic => "Epic".to_string(),
    }
}

fn string_to_ticket_type(s: &str) -> TicketType {
    match s {
        "Task" => TicketType::Task,
        "Bug" => TicketType::Bug,
        "Feature" => TicketType::Feature,
        "Epic" => TicketType::Epic,
        _ => TicketType::Task,
    }
}

fn status_to_string(status: &TicketStatus) -> String {
    match status {
        TicketStatus::Open => "Open".to_string(),
        TicketStatus::InProgress => "InProgress".to_string(),
        TicketStatus::Review => "Review".to_string(),
        TicketStatus::Done => "Done".to_string(),
        TicketStatus::Closed => "Closed".to_string(),
    }
}

fn string_to_status(s: &str) -> TicketStatus {
    match s {
        "Open" => TicketStatus::Open,
        "InProgress" => TicketStatus::InProgress,
        "Review" => TicketStatus::Review,
        "Done" => TicketStatus::Done,
        "Closed" => TicketStatus::Closed,
        _ => TicketStatus::Open,
    }
}

fn priority_to_string(priority: &Priority) -> String {
    match priority {
        Priority::Low => "Low".to_string(),
        Priority::Medium => "Medium".to_string(),
        Priority::High => "High".to_string(),
        Priority::Critical => "Critical".to_string(),
    }
}

fn string_to_priority(s: &str) -> Priority {
    match s {
        "Low" => Priority::Low,
        "Medium" => Priority::Medium,
        "High" => Priority::High,
        "Critical" => Priority::Critical,
        _ => Priority::Medium,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        connection::init_memory_pool,
        migrations::run_migrations,
        repositories::{project_repository::ProjectRepository, user_repository::UserRepository},
    };
    use worknest_core::models::{Project, User};

    fn setup_test_repo() -> (TicketRepository, ProjectId, UserId) {
        let pool = Arc::new(init_memory_pool().unwrap());
        let mut conn = pool.get().unwrap();
        run_migrations(&mut conn).unwrap();
        drop(conn);

        // Create test user and project
        let user_repo = UserRepository::new(Arc::clone(&pool));
        let user = User::new("testuser".to_string(), "test@example.com".to_string());
        user_repo.create_with_password(&user, "hash").unwrap();

        let project_repo = ProjectRepository::new(Arc::clone(&pool));
        let project = Project::new("Test Project".to_string(), user.id);
        project_repo.create(&project).unwrap();

        (TicketRepository::new(pool), project.id, user.id)
    }

    #[test]
    fn test_create_and_find_ticket() {
        let (repo, project_id, user_id) = setup_test_repo();
        let ticket = Ticket::new(
            project_id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user_id,
        );

        let created = repo.create(&ticket).unwrap();
        assert_eq!(created.title, "Test Ticket");

        let found = repo.find_by_id(ticket.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Ticket");
    }

    #[test]
    fn test_update_ticket() {
        let (repo, project_id, user_id) = setup_test_repo();
        let mut ticket = Ticket::new(
            project_id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user_id,
        );

        repo.create(&ticket).unwrap();

        ticket.title = "Updated Ticket".to_string();
        ticket.status = TicketStatus::InProgress;
        repo.update(&ticket).unwrap();

        let found = repo.find_by_id(ticket.id).unwrap().unwrap();
        assert_eq!(found.title, "Updated Ticket");
        assert_eq!(found.status, TicketStatus::InProgress);
    }

    #[test]
    fn test_find_by_project() {
        let (repo, project_id, user_id) = setup_test_repo();
        let ticket1 = Ticket::new(
            project_id,
            "Ticket 1".to_string(),
            TicketType::Task,
            user_id,
        );
        let ticket2 = Ticket::new(project_id, "Ticket 2".to_string(), TicketType::Bug, user_id);

        repo.create(&ticket1).unwrap();
        repo.create(&ticket2).unwrap();

        let tickets = repo.find_by_project(project_id).unwrap();
        assert_eq!(tickets.len(), 2);
    }

    #[test]
    fn test_update_status() {
        let (repo, project_id, user_id) = setup_test_repo();
        let ticket = Ticket::new(
            project_id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user_id,
        );

        repo.create(&ticket).unwrap();
        repo.update_status(ticket.id, TicketStatus::Done).unwrap();

        let found = repo.find_by_id(ticket.id).unwrap().unwrap();
        assert_eq!(found.status, TicketStatus::Done);
    }

    #[test]
    fn test_assign_and_unassign() {
        let (repo, project_id, user_id) = setup_test_repo();
        let ticket = Ticket::new(
            project_id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user_id,
        );

        repo.create(&ticket).unwrap();

        // Assign
        repo.assign(ticket.id, user_id).unwrap();
        let found = repo.find_by_id(ticket.id).unwrap().unwrap();
        assert_eq!(found.assignee_id, Some(user_id));

        // Unassign
        repo.unassign(ticket.id).unwrap();
        let found = repo.find_by_id(ticket.id).unwrap().unwrap();
        assert!(found.assignee_id.is_none());
    }

    #[test]
    fn test_find_by_status() {
        let (repo, project_id, user_id) = setup_test_repo();
        let mut ticket = Ticket::new(
            project_id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user_id,
        );
        ticket.status = TicketStatus::InProgress;

        repo.create(&ticket).unwrap();

        let tickets = repo.find_by_status(TicketStatus::InProgress).unwrap();
        assert_eq!(tickets.len(), 1);
        assert_eq!(tickets[0].title, "Test Ticket");
    }

    #[test]
    fn test_delete_ticket() {
        let (repo, project_id, user_id) = setup_test_repo();
        let ticket = Ticket::new(
            project_id,
            "Test Ticket".to_string(),
            TicketType::Task,
            user_id,
        );

        repo.create(&ticket).unwrap();
        repo.delete(ticket.id).unwrap();

        let found = repo.find_by_id(ticket.id).unwrap();
        assert!(found.is_none());
    }
}
