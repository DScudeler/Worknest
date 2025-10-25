//! Database connection management

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;
use std::path::Path;

use crate::{DbError, Result};

/// Database connection pool type
pub type DbPool = Pool<SqliteConnectionManager>;

/// Database connection from pool
pub type DbConnection = r2d2::PooledConnection<SqliteConnectionManager>;

/// Initialize the database pool
///
/// # Arguments
/// * `database_path` - Path to the SQLite database file
///
/// # Returns
/// A connection pool for the database
pub fn init_pool<P: AsRef<Path>>(database_path: P) -> Result<DbPool> {
    let manager = SqliteConnectionManager::file(database_path)
        .with_flags(
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_init(|conn| {
            // Enable foreign keys
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            Ok(())
        });

    let pool = Pool::builder()
        .max_size(16)
        .build(manager)
        .map_err(|e| DbError::Connection(e.to_string()))?;

    Ok(pool)
}

/// Initialize an in-memory database pool (for testing)
pub fn init_memory_pool() -> Result<DbPool> {
    let manager = SqliteConnectionManager::memory().with_init(|conn| {
        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(())
    });

    let pool = Pool::builder()
        .max_size(4)
        .build(manager)
        .map_err(|e| DbError::Connection(e.to_string()))?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_init_pool() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let pool = init_pool(&db_path).unwrap();
        let conn = pool.get().unwrap();

        // Verify foreign keys are enabled
        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);

        drop(conn);
        drop(pool);
        fs::remove_file(db_path).unwrap();
    }

    #[test]
    fn test_init_memory_pool() {
        let pool = init_memory_pool().unwrap();
        let conn = pool.get().unwrap();

        // Verify foreign keys are enabled
        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);
    }

    #[test]
    fn test_pool_multiple_connections() {
        let pool = init_memory_pool().unwrap();

        let conn1 = pool.get().unwrap();
        let conn2 = pool.get().unwrap();

        assert_ne!(
            &*conn1 as *const _, &*conn2 as *const _,
            "Should get different connections"
        );
    }
}
