//! User repository implementation

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Row};
use std::sync::Arc;
use uuid::Uuid;

use worknest_core::models::{User, UserId};

use crate::{connection::DbPool, repository::Repository, DbError, Result};

/// User repository for database operations
pub struct UserRepository {
    pool: Arc<DbPool>,
}

impl UserRepository {
    /// Create a new UserRepository
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Find a user by username
    pub fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, username, email, created_at, updated_at FROM users WHERE username = ?1",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let user = stmt
            .query_row(params![username], row_to_user)
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(user)
    }

    /// Find a user by email
    pub fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, username, email, created_at, updated_at FROM users WHERE email = ?1",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let user = stmt
            .query_row(params![email], row_to_user)
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(user)
    }

    /// Get password hash for a user
    pub fn get_password_hash(&self, user_id: UserId) -> Result<Option<String>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT password_hash FROM users WHERE id = ?1")
            .map_err(|e| DbError::Query(e.to_string()))?;

        let hash = stmt
            .query_row(params![user_id.0.to_string()], |row| row.get(0))
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(hash)
    }

    /// Create a user with password hash
    pub fn create_with_password(&self, user: &User, password_hash: &str) -> Result<User> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        conn.execute(
            "INSERT INTO users (id, username, email, password_hash, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                user.id.0.to_string(),
                user.username,
                user.email,
                password_hash,
                user.created_at.to_rfc3339(),
                user.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                DbError::ConstraintViolation("User already exists".to_string())
            } else {
                DbError::Query(e.to_string())
            }
        })?;

        Ok(user.clone())
    }

    /// Update password hash for a user
    pub fn update_password(&self, user_id: UserId, password_hash: &str) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE users SET password_hash = ?1, updated_at = ?2 WHERE id = ?3",
                params![
                    password_hash,
                    Utc::now().to_rfc3339(),
                    user_id.0.to_string()
                ],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("User not found".to_string()));
        }

        Ok(())
    }
}

impl Repository<User, UserId> for UserRepository {
    fn find_by_id(&self, id: UserId) -> Result<Option<User>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT id, username, email, created_at, updated_at FROM users WHERE id = ?1")
            .map_err(|e| DbError::Query(e.to_string()))?;

        let user = stmt
            .query_row(params![id.0.to_string()], row_to_user)
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(user)
    }

    fn find_all(&self) -> Result<Vec<User>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, username, email, created_at, updated_at FROM users ORDER BY username",
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        let users = stmt
            .query_map([], row_to_user)
            .map_err(|e| DbError::Query(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(users)
    }

    fn create(&self, _entity: &User) -> Result<User> {
        Err(DbError::Query(
            "Use create_with_password instead".to_string(),
        ))
    }

    fn update(&self, entity: &User) -> Result<User> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute(
                "UPDATE users SET username = ?1, email = ?2, updated_at = ?3 WHERE id = ?4",
                params![
                    entity.username,
                    entity.email,
                    Utc::now().to_rfc3339(),
                    entity.id.0.to_string(),
                ],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    DbError::ConstraintViolation("Username or email already exists".to_string())
                } else {
                    DbError::Query(e.to_string())
                }
            })?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("User not found".to_string()));
        }

        Ok(entity.clone())
    }

    fn delete(&self, id: UserId) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let rows_affected = conn
            .execute("DELETE FROM users WHERE id = ?1", params![id.0.to_string()])
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("User not found".to_string()));
        }

        Ok(())
    }
}

/// Convert a database row to a User
fn row_to_user(row: &Row) -> rusqlite::Result<User> {
    let id_str: String = row.get(0)?;
    let id = UserId::from_uuid(Uuid::parse_str(&id_str).unwrap());

    let created_at_str: String = row.get(3)?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .unwrap()
        .with_timezone(&Utc);

    let updated_at_str: String = row.get(4)?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .unwrap()
        .with_timezone(&Utc);

    Ok(User {
        id,
        username: row.get(1)?,
        email: row.get(2)?,
        created_at,
        updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connection::init_memory_pool, migrations::run_migrations};

    fn setup_test_repo() -> UserRepository {
        let pool = Arc::new(init_memory_pool().unwrap());
        let mut conn = pool.get().unwrap();
        run_migrations(&mut conn).unwrap();
        drop(conn);
        UserRepository::new(pool)
    }

    #[test]
    fn test_create_and_find_user() {
        let repo = setup_test_repo();
        let user = User::new("testuser".to_string(), "test@example.com".to_string());

        // Create user with password
        let created = repo.create_with_password(&user, "hashed_password").unwrap();
        assert_eq!(created.username, "testuser");

        // Find by ID
        let found = repo.find_by_id(user.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "testuser");

        // Find by username
        let found = repo.find_by_username("testuser").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().email, "test@example.com");

        // Find by email
        let found = repo.find_by_email("test@example.com").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "testuser");
    }

    #[test]
    fn test_get_password_hash() {
        let repo = setup_test_repo();
        let user = User::new("testuser".to_string(), "test@example.com".to_string());

        repo.create_with_password(&user, "my_hash").unwrap();

        let hash = repo.get_password_hash(user.id).unwrap();
        assert_eq!(hash, Some("my_hash".to_string()));
    }

    #[test]
    fn test_update_user() {
        let repo = setup_test_repo();
        let mut user = User::new("testuser".to_string(), "test@example.com".to_string());

        repo.create_with_password(&user, "hash").unwrap();

        user.email = "newemail@example.com".to_string();
        repo.update(&user).unwrap();

        let found = repo.find_by_id(user.id).unwrap().unwrap();
        assert_eq!(found.email, "newemail@example.com");
    }

    #[test]
    fn test_delete_user() {
        let repo = setup_test_repo();
        let user = User::new("testuser".to_string(), "test@example.com".to_string());

        repo.create_with_password(&user, "hash").unwrap();
        repo.delete(user.id).unwrap();

        let found = repo.find_by_id(user.id).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_duplicate_username() {
        let repo = setup_test_repo();
        let user1 = User::new("testuser".to_string(), "test1@example.com".to_string());
        let user2 = User::new("testuser".to_string(), "test2@example.com".to_string());

        repo.create_with_password(&user1, "hash").unwrap();
        let result = repo.create_with_password(&user2, "hash");

        assert!(result.is_err());
    }

    #[test]
    fn test_find_all_users() {
        let repo = setup_test_repo();

        let user1 = User::new("alice".to_string(), "alice@example.com".to_string());
        let user2 = User::new("bob".to_string(), "bob@example.com".to_string());

        repo.create_with_password(&user1, "hash1").unwrap();
        repo.create_with_password(&user2, "hash2").unwrap();

        let users = repo.find_all().unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "alice"); // Ordered by username
        assert_eq!(users[1].username, "bob");
    }
}
