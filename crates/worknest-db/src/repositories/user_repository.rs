//! User repository implementation

use chrono::Utc;
use rusqlite::{params, OptionalExtension, Row};
use std::sync::Arc;

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
                "SELECT id, username, email, full_name, avatar_url, is_agent, created_at, updated_at FROM users WHERE username = ?1",
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
                "SELECT id, username, email, full_name, avatar_url, is_agent, created_at, updated_at FROM users WHERE email = ?1",
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
            "INSERT INTO users (id, username, email, full_name, avatar_url, is_agent, password_hash, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                user.id.0.to_string(),
                user.username,
                user.email,
                user.full_name,
                user.avatar_url,
                if user.is_agent { 1_i64 } else { 0_i64 },
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

    /// Look up an existing autonomous-agent user by its deterministic email.
    /// Returns `Ok(None)` when the email is unknown, `Err(ConstraintViolation)`
    /// when a *human* user happens to own that address — the activation
    /// pipeline turns that into a clear error instead of silently reusing a
    /// human identity.
    pub fn find_agent_by_email(&self, email: &str) -> Result<Option<User>> {
        match self.find_by_email(email)? {
            None => Ok(None),
            Some(u) if u.is_agent => Ok(Some(u)),
            Some(_) => Err(DbError::ConstraintViolation(format!(
                "Email {email} is already taken by a human user"
            ))),
        }
    }

    /// Update password hash for a user. Also bumps `password_changed_at` to
    /// the current Unix timestamp so any JWT minted before now (with an
    /// older `iat` claim) is rejected by the auth layer.
    pub fn update_password(&self, user_id: UserId, password_hash: &str) -> Result<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let now = Utc::now();
        let rows_affected = conn
            .execute(
                "UPDATE users SET password_hash = ?1, password_changed_at = ?2, updated_at = ?3 \
                 WHERE id = ?4",
                params![
                    password_hash,
                    now.timestamp(),
                    now.to_rfc3339(),
                    user_id.0.to_string(),
                ],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

        if rows_affected == 0 {
            return Err(DbError::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    /// Look up the Unix-epoch second when this user's password was last
    /// changed. Returns 0 for users that haven't rotated since the column
    /// was added.
    pub fn get_password_changed_at(&self, user_id: UserId) -> Result<i64> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT password_changed_at FROM users WHERE id = ?1")
            .map_err(|e| DbError::Query(e.to_string()))?;

        let ts: Option<i64> = stmt
            .query_row(params![user_id.0.to_string()], |row| row.get(0))
            .optional()
            .map_err(|e| DbError::Query(e.to_string()))?;

        ts.ok_or_else(|| DbError::NotFound("User not found".to_string()))
    }
}

impl Repository<User, UserId> for UserRepository {
    fn find_by_id(&self, id: UserId) -> Result<Option<User>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| DbError::Connection(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT id, username, email, full_name, avatar_url, is_agent, created_at, updated_at FROM users WHERE id = ?1")
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
                "SELECT id, username, email, full_name, avatar_url, is_agent, created_at, updated_at FROM users ORDER BY username",
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
                "UPDATE users SET username = ?1, email = ?2, full_name = ?3, \
                 avatar_url = ?4, updated_at = ?5 WHERE id = ?6",
                params![
                    entity.username,
                    entity.email,
                    entity.full_name,
                    entity.avatar_url,
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

use super::{parse_datetime, parse_uuid};

/// Convert a database row to a User. Column order:
/// (id, username, email, full_name, avatar_url, is_agent, created_at, updated_at)
fn row_to_user(row: &Row) -> rusqlite::Result<User> {
    let id_str: String = row.get(0)?;
    let id = UserId::from_uuid(parse_uuid(&id_str)?);

    let is_agent_int: i64 = row.get(5)?;

    let created_at_str: String = row.get(6)?;
    let created_at = parse_datetime(&created_at_str)?;

    let updated_at_str: String = row.get(7)?;
    let updated_at = parse_datetime(&updated_at_str)?;

    Ok(User {
        id,
        username: row.get(1)?,
        email: row.get(2)?,
        full_name: row.get(3)?,
        avatar_url: row.get(4)?,
        is_agent: is_agent_int != 0,
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
