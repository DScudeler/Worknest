//! Repository implementations

pub mod agent_deployment_repository;
pub mod agent_event_repository;
pub mod agent_tick_repository;
pub mod attachment_repository;
pub mod comment_repository;
pub mod persona_repository;
pub mod project_repository;
pub mod tag_repository;
pub mod ticket_repository;
pub mod user_repository;

pub use agent_deployment_repository::{AgentDeploymentRepository, PersonaSnapshot};
pub use agent_event_repository::AgentEventRepository;
pub use agent_tick_repository::{AgentTickRepository, TickStats};
pub use attachment_repository::AttachmentRepository;
pub use comment_repository::CommentRepository;
pub use persona_repository::PersonaRepository;
pub use project_repository::ProjectRepository;
pub use tag_repository::TagRepository;
pub use ticket_repository::{TicketFilters, TicketRepository, TicketSort};
pub use user_repository::UserRepository;

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Parse a UUID string from a SQLite TEXT column.
pub(crate) fn parse_uuid(s: &str) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

/// Parse an RFC 3339 datetime from a SQLite TEXT column, returning UTC.
pub(crate) fn parse_datetime(s: &str) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
}
