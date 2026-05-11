//! Domain models for Worknest

pub mod agent_deployment;
pub mod agent_event;
pub mod agent_tick;
pub mod attachment;
pub mod comment;
pub mod persona;
pub mod project;
pub mod role;
pub mod tag;
pub mod team;
pub mod ticket;
pub mod user;

pub use agent_deployment::{ActivationStep, AgentDeployment, AgentDeploymentId, AgentStatus};
pub use agent_event::{AgentEvent, AgentEventId, AgentEventKind};
pub use agent_tick::{AgentTick, AgentTickId, TickOutcome};
pub use attachment::{Attachment, AttachmentId};
pub use comment::{Comment, CommentId};
pub use persona::{AgentModel, Capability, Persona, PersonaId};
pub use project::{Project, ProjectId};
pub use role::{Permission, PermissionId, Role, RoleId};
pub use tag::{Tag, TagId};
pub use team::{Team, TeamId};
pub use ticket::{Priority, Ticket, TicketId, TicketStatus, TicketType};
pub use user::{User, UserId};
