//! Worknest Database Layer
//!
//! This crate provides database access and persistence for Worknest.
//! It implements the repository pattern for clean separation between
//! domain logic and data access.

pub mod connection;
pub mod error;
pub mod migrations;
pub mod repositories;
pub mod repository;

pub use connection::{init_memory_pool, init_pool, DbConnection, DbPool};
pub use error::{DbError, Result};
pub use migrations::run_migrations;
pub use repositories::{ProjectRepository, TicketRepository, UserRepository};
pub use repository::Repository;
