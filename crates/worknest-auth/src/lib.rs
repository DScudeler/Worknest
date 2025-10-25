//! Worknest Authentication
//!
//! This crate handles user authentication and authorization.
//! It provides user registration, login, JWT token management,
//! and session handling.

pub mod error;
pub mod password;
pub mod service;
pub mod token;

pub use error::{AuthError, Result};
pub use service::AuthService;
pub use token::{AuthToken, Claims};
