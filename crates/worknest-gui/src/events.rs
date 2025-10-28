//! Event system for async API callbacks

use std::sync::{Arc, Mutex};
use worknest_core::models::{Project, Ticket, User};

/// Event queue for handling async API responses
#[derive(Clone)]
pub struct EventQueue {
    events: Arc<Mutex<Vec<AppEvent>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Push an event to the queue
    pub fn push(&self, event: AppEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    /// Pop all pending events
    pub fn drain(&self) -> Vec<AppEvent> {
        if let Ok(mut events) = self.events.lock() {
            events.drain(..).collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Application events from async operations
#[derive(Debug, Clone)]
pub enum AppEvent {
    // Authentication events
    LoginSuccess { user: User, token: String },
    LoginError { message: String },
    RegisterSuccess { user: User, token: String },
    RegisterError { message: String },

    // Project events
    ProjectsLoaded { projects: Vec<Project> },
    ProjectLoaded { project: Project },
    ProjectCreated { project: Project },
    ProjectUpdated { project: Project },
    ProjectDeleted { project_id: String },
    ProjectError { message: String },

    // Ticket events
    TicketsLoaded { tickets: Vec<Ticket> },
    TicketLoaded { ticket: Ticket },
    TicketCreated { ticket: Ticket },
    TicketUpdated { ticket: Ticket },
    TicketDeleted { ticket_id: String },
    TicketError { message: String },

    // Generic events
    ApiError { message: String },
    LoadingComplete,
}
