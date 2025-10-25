//! Error types for the core domain

use thiserror::Error;

/// Core domain errors
#[derive(Debug, Error)]
pub enum CoreError {
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Entity not found
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// Unauthorized access
    #[error("Unauthorized")]
    Unauthorized,

    /// Conflict (e.g., duplicate entry)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Business rule violation
    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    /// Invalid ID format
    #[error("Invalid ID: {0}")]
    InvalidId(String),
}

/// Result type alias using CoreError
pub type Result<T> = std::result::Result<T, CoreError>;
