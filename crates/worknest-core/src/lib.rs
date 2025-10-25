//! Worknest Core
//!
//! This crate contains the core business logic and domain models for Worknest.
//! It defines the fundamental entities, types, and validation rules.

pub mod error;
pub mod models;

pub use error::{CoreError, Result};
