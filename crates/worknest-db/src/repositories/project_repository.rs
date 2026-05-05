//! Project repository implementation

use chrono::Utc;
use rusqlite::{params, OptionalExtension, Row};
use std::sync::Arc;

use worknest_core::models::{Project, ProjectId, UserId};

use crate::{connection::DbPool, repository::Repository, DbError, Result};

/// Project repository for database operations
pub struct ProjectRepository {
    pool: Arc<DbPool>,
}

impl ProjectRepository {
    /// Create a new ProjectRepository
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Find all projects created by a specific user
    pub fn find_by_creator(&self, user_id: UserId) -> Result<Vec<Project>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, color, archived, created_by, created_at, updated_at
                 FROM projects WHERE created_by = ?1 ORDER BY name",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let projects = stmt
            .query_map(params![user_id.0.to_string()], row_to_project)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(projects)
    }

    /// Find all active (non-archived) projects
    pub fn find_active(&self) -> Result<Vec<Project>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, color, archived, created_by, created_at, updated_at
                 FROM projects WHERE archived = 0 ORDER BY name",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let projects = stmt
            .query_map([], row_to_project)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(projects)
    }

    /// Find archived projects
    pub fn find_archived(&self) -> Result<Vec<Project>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, color, archived, created_by, created_at, updated_at
                 FROM projects WHERE archived = 1 ORDER BY name",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let projects = stmt
            .query_map([], row_to_project)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(projects)
    }

    /// Archive a project and return the updated project
    pub fn archive(&self, project_id: ProjectId) -> Result<Project> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE projects SET archived = 1, updated_at = ?1 WHERE id = ?2",
                params![Utc::now().to_rfc3339(), project_id.0.to_string()],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("Project not found".to_string()));
        }

        // Fetch and return the updated project
        self.find_by_id(project_id)?
            .ok_or_else(|| DbError::NotFound("Project not found after archive".to_string()))
    }

    /// Add a user as a member of a project. Idempotent: re-inserting an
    /// existing (project, user) pair refreshes the role and `added_at`.
    pub fn add_member(&self, project_id: ProjectId, user_id: UserId, role: &str) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            "INSERT INTO project_members (project_id, user_id, role, added_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(project_id, user_id) DO UPDATE SET
                role = excluded.role,
                added_at = excluded.added_at",
            params![
                project_id.0.to_string(),
                user_id.0.to_string(),
                role,
                Utc::now().to_rfc3339(),
            ],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(())
    }

    /// Whether `user_id` is a member of `project_id` (any role, including
    /// implicit owner-as-member rows backfilled by V4).
    pub fn is_member(&self, project_id: ProjectId, user_id: UserId) -> Result<bool> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM project_members
                 WHERE project_id = ?1 AND user_id = ?2",
                params![project_id.0.to_string(), user_id.0.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(count > 0)
    }

    /// List all (user_id, role) pairs for a project, ordered by added_at.
    pub fn list_members(&self, project_id: ProjectId) -> Result<Vec<(UserId, String)>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT user_id, role FROM project_members
                 WHERE project_id = ?1 ORDER BY added_at",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let rows = stmt
            .query_map(params![project_id.0.to_string()], |row| {
                let uid_str: String = row.get(0)?;
                let role: String = row.get(1)?;
                Ok((uid_str, role))
            })
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut out = Vec::new();
        for r in rows {
            let (uid_str, role) = r.map_err(|e| DbError::Query(e.to_string()))?;
            let uid = UserId::from_uuid(parse_uuid(&uid_str).map_err(|e| {
                DbError::Query(format!("invalid user id in project_members: {e}"))
            })?);
            out.push((uid, role));
        }
        Ok(out)
    }

    /// Remove a member from a project. Returns true if a row was deleted.
    pub fn remove_member(&self, project_id: ProjectId, user_id: UserId) -> Result<bool> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows = conn
            .execute(
                "DELETE FROM project_members
                 WHERE project_id = ?1 AND user_id = ?2",
                params![project_id.0.to_string(), user_id.0.to_string()],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(rows > 0)
    }

    /// Find every project visible to `user_id`: those they own (`created_by`)
    /// OR those they are a member of. Replaces the previous in-memory filter
    /// in `list_projects`.
    pub fn find_visible_to(&self, user_id: UserId) -> Result<Vec<Project>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT p.id, p.name, p.description, p.color, p.archived,
                                 p.created_by, p.created_at, p.updated_at
                 FROM projects p
                 LEFT JOIN project_members pm ON pm.project_id = p.id
                 WHERE p.created_by = ?1 OR pm.user_id = ?1
                 ORDER BY p.name",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let projects = stmt
            .query_map(params![user_id.0.to_string()], row_to_project)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(projects)
    }

    /// Unarchive a project
    pub fn unarchive(&self, project_id: ProjectId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE projects SET archived = 0, updated_at = ?1 WHERE id = ?2",
                params![Utc::now().to_rfc3339(), project_id.0.to_string()],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("Project not found".to_string()));
        }

        Ok(())
    }
}

impl Repository<Project, ProjectId> for ProjectRepository {
    fn find_by_id(&self, id: ProjectId) -> Result<Option<Project>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, color, archived, created_by, created_at, updated_at
                 FROM projects WHERE id = ?1",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let project = stmt
            .query_row(params![id.0.to_string()], row_to_project)
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(project)
    }

    fn find_all(&self) -> Result<Vec<Project>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, color, archived, created_by, created_at, updated_at
                 FROM projects ORDER BY name",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let projects = stmt
            .query_map([], row_to_project)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(projects)
    }

    fn create(&self, entity: &Project) -> Result<Project> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            "INSERT INTO projects (id, name, description, color, archived, created_by, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entity.id.0.to_string(),
                entity.name,
                entity.description,
                entity.color,
                if entity.archived { 1 } else { 0 },
                entity.created_by.0.to_string(),
                entity.created_at.to_rfc3339(),
                entity.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(entity.clone())
    }

    fn update(&self, entity: &Project) -> Result<Project> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE projects SET name = ?1, description = ?2, color = ?3, archived = ?4, updated_at = ?5
                 WHERE id = ?6",
                params![
                    entity.name,
                    entity.description,
                    entity.color,
                    if entity.archived { 1 } else { 0 },
                    Utc::now().to_rfc3339(),
                    entity.id.0.to_string(),
                ],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("Project not found".to_string()));
        }

        Ok(entity.clone())
    }

    fn delete(&self, id: ProjectId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "DELETE FROM projects WHERE id = ?1",
                params![id.0.to_string()],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("Project not found".to_string()));
        }

        Ok(())
    }
}

use super::{parse_datetime, parse_uuid};

/// Convert a database row to a Project
fn row_to_project(row: &Row) -> rusqlite::Result<Project> {
    let id_str: String = row.get(0)?;
    let id = ProjectId::from_uuid(parse_uuid(&id_str)?);

    let created_by_str: String = row.get(5)?;
    let created_by = UserId::from_uuid(parse_uuid(&created_by_str)?);

    let created_at_str: String = row.get(6)?;
    let created_at = parse_datetime(&created_at_str)?;

    let updated_at_str: String = row.get(7)?;
    let updated_at = parse_datetime(&updated_at_str)?;

    let archived: i32 = row.get(4)?;

    Ok(Project {
        id,
        name: row.get(1)?,
        description: row.get(2)?,
        color: row.get(3)?,
        archived: archived == 1,
        created_by,
        created_at,
        updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        connection::init_memory_pool, migrations::run_migrations,
        repositories::user_repository::UserRepository,
    };
    use worknest_core::models::User;

    fn setup_test_repo() -> (ProjectRepository, UserId) {
        let pool = Arc::new(init_memory_pool().unwrap());
        let mut conn = pool.get().unwrap();
        run_migrations(&mut conn).unwrap();
        drop(conn);

        // Create a test user
        let user_repo = UserRepository::new(Arc::clone(&pool));
        let user = User::new("testuser".to_string(), "test@example.com".to_string());
        user_repo.create_with_password(&user, "hash").unwrap();

        (ProjectRepository::new(pool), user.id)
    }

    #[test]
    fn test_create_and_find_project() {
        let (repo, user_id) = setup_test_repo();
        let project = Project::new("Test Project".to_string(), user_id);

        let created = repo.create(&project).unwrap();
        assert_eq!(created.name, "Test Project");

        let found = repo.find_by_id(project.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Project");
    }

    #[test]
    fn test_update_project() {
        let (repo, user_id) = setup_test_repo();
        let mut project = Project::new("Test Project".to_string(), user_id);

        repo.create(&project).unwrap();

        project.name = "Updated Project".to_string();
        project.description = Some("New description".to_string());
        repo.update(&project).unwrap();

        let found = repo.find_by_id(project.id).unwrap().unwrap();
        assert_eq!(found.name, "Updated Project");
        assert_eq!(found.description, Some("New description".to_string()));
    }

    #[test]
    fn test_archive_project() {
        let (repo, user_id) = setup_test_repo();
        let project = Project::new("Test Project".to_string(), user_id);

        repo.create(&project).unwrap();
        repo.archive(project.id).unwrap();

        let found = repo.find_by_id(project.id).unwrap().unwrap();
        assert!(found.archived);

        repo.unarchive(project.id).unwrap();
        let found = repo.find_by_id(project.id).unwrap().unwrap();
        assert!(!found.archived);
    }

    #[test]
    fn test_find_active_and_archived() {
        let (repo, user_id) = setup_test_repo();
        let project1 = Project::new("Active Project".to_string(), user_id);
        let project2 = Project::new("Archived Project".to_string(), user_id);

        repo.create(&project1).unwrap();
        repo.create(&project2).unwrap();
        repo.archive(project2.id).unwrap();

        let active = repo.find_active().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "Active Project");

        let archived = repo.find_archived().unwrap();
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].name, "Archived Project");
    }

    #[test]
    fn test_find_by_creator() {
        let (repo, user_id) = setup_test_repo();
        let project1 = Project::new("Project 1".to_string(), user_id);
        let project2 = Project::new("Project 2".to_string(), user_id);

        repo.create(&project1).unwrap();
        repo.create(&project2).unwrap();

        let projects = repo.find_by_creator(user_id).unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_delete_project() {
        let (repo, user_id) = setup_test_repo();
        let project = Project::new("Test Project".to_string(), user_id);

        repo.create(&project).unwrap();
        repo.delete(project.id).unwrap();

        let found = repo.find_by_id(project.id).unwrap();
        assert!(found.is_none());
    }
}
