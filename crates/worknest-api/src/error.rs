//! Error types for API operations

use thiserror::Error;

/// API errors
#[derive(Debug, Error)]
pub enum ApiError {
    /// Core domain error
    #[error("Core error: {0}")]
    Core(#[from] worknest_core::CoreError),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] worknest_db::DbError),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(#[from] worknest_auth::AuthError),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Forbidden
    #[error("Forbidden")]
    Forbidden,

    /// Internal server error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias using ApiError
pub type Result<T> = std::result::Result<T, ApiError>;
