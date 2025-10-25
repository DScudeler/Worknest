//! Database migrations

use refinery::embed_migrations;
use rusqlite::Connection;

use crate::{DbError, Result};

// Embed migrations from the migrations directory
embed_migrations!("src/migrations");

/// Run all pending migrations
pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    migrations::runner()
        .run(conn)
        .map_err(|e| DbError::Migration(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        run_migrations(&mut conn).unwrap();

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"tickets".to_string()));
        assert!(tables.contains(&"comments".to_string()));
        assert!(tables.contains(&"sessions".to_string()));
    }
}
