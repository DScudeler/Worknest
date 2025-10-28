//! Worknest API Layer
//!
//! This crate implements the application API layer using the CQRS pattern.
//! It provides command handlers (write operations) and query handlers (read operations).

pub mod error;

pub use error::{ApiError, Result};

// Re-export main application for tests
#[cfg(test)]
pub use crate::main::*;
