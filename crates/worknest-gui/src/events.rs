//! Event system for async API callbacks

use std::sync::{Arc, Mutex};

use egui::Context;
use worknest_core::models::{Comment, Project, Ticket, User};

/// Event queue for handling async API responses.
///
/// Async work runs via `wasm_bindgen_futures::spawn_local` and pushes results
/// into this queue. egui's `App::update` only runs in response to input
/// (mouse, key, ...), so without explicitly waking the runtime the UI sits
/// frozen until the user wiggles the mouse. Storing an optional `egui::Context`
/// here lets `push` call `request_repaint`, which schedules an immediate frame.
#[derive(Clone)]
pub struct EventQueue {
    events: Arc<Mutex<Vec<AppEvent>>>,
    ctx: Arc<Mutex<Option<Context>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            ctx: Arc::new(Mutex::new(None)),
        }
    }

    /// Plumb the egui context so `push` can wake the UI from an async task.
    /// Cheap to call every frame — `Context` is `Arc`-backed.
    pub fn set_repaint_context(&self, ctx: Context) {
        if let Ok(mut c) = self.ctx.lock() {
            *c = Some(ctx);
        }
    }

    /// Push an event and request a repaint so the next frame fires immediately.
    pub fn push(&self, event: AppEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
        if let Ok(c) = self.ctx.lock() {
            if let Some(c) = c.as_ref() {
                c.request_repaint();
            }
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

    // Account management events — fired only after the API confirms success,
    // so screens can show success notifications and clear sensitive form
    // state without racing the network.
    ProfileUpdated { user: User },
    ProfileUpdateError { message: String },
    PasswordChanged,
    PasswordChangeError { message: String },

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

    // Comment events
    CommentsLoaded { comments: Vec<Comment> },
    CommentCreated { comment: Comment },
    CommentUpdated { comment: Comment },
    CommentDeleted { comment_id: String },
    CommentError { message: String },

    // Generic events
    ApiError { message: String },
    LoadingComplete,
}
