//! Worknest Database Layer
//!
//! This crate provides database access and persistence for Worknest.
//! It implements the repository pattern for clean separation between
//! domain logic and data access.

pub mod error;

pub use error::{DbError, Result};

// Placeholder: Database modules will be implemented as part of MVP development
// pub mod connection;
// pub mod migrations;
// pub mod repositories;
