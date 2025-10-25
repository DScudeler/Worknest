//! Error types for authentication

use thiserror::Error;

/// Authentication errors
#[derive(Debug, Error)]
pub enum AuthError {
    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Token expired
    #[error("Token expired")]
    TokenExpired,

    /// Invalid token
    #[error("Invalid token")]
    TokenInvalid,

    /// User already exists
    #[error("User already exists")]
    UserExists,

    /// User not found
    #[error("User not found")]
    UserNotFound,

    /// Password validation error
    #[error("Password validation error: {0}")]
    PasswordValidation(String),

    /// Internal authentication error
    #[error("Internal authentication error: {0}")]
    Internal(String),
}

/// Result type alias using AuthError
pub type Result<T> = std::result::Result<T, AuthError>;
