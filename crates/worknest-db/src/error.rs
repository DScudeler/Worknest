//! Error types for database operations

use thiserror::Error;

/// Database operation errors
#[derive(Debug, Error)]
pub enum DbError {
    /// Database connection error
    #[error("Database connection error: {0}")]
    Connection(String),

    /// Query execution error
    #[error("Query error: {0}")]
    Query(String),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Entity not found
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// Constraint violation (e.g., unique constraint)
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

/// Result type alias using DbError
pub type Result<T> = std::result::Result<T, DbError>;
