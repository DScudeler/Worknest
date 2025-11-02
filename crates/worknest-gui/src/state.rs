//! Application state management

use crate::api_client::ApiClient;
use crate::events::{AppEvent, EventQueue};
use crate::screens::Screen;
use worknest_core::models::{Comment, Project, Ticket, User};

// Use web_time::Instant for WASM compatibility instead of std::time::Instant
use web_time::Instant;

/// Main application state
#[derive(Clone)]
pub struct AppState {
    /// Current authenticated user
    pub current_user: Option<User>,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Current screen
    pub current_screen: Screen,
    /// API client for backend communication
    pub api_client: ApiClient,
    /// Event queue for async API responses
    pub event_queue: EventQueue,
    /// Notification messages
    pub notifications: Vec<Notification>,
    /// Loading state
    pub is_loading: bool,
    /// Cached projects from API
    pub projects: Vec<Project>,
    /// Cached tickets from API
    pub tickets: Vec<Ticket>,
    /// Cached comments from API
    pub comments: Vec<Comment>,
}

impl AppState {
    /// Create a new web application state with API client
    pub fn new(api_client: ApiClient) -> Self {
        Self {
            current_user: None,
            auth_token: None,
            current_screen: Screen::Login,
            api_client,
            event_queue: EventQueue::new(),
            notifications: Vec::new(),
            is_loading: false,
            projects: Vec::new(),
            tickets: Vec::new(),
            comments: Vec::new(),
        }
    }

    /// Process pending events from async operations
    pub fn process_events(&mut self) {
        let events = self.event_queue.drain();

        for event in events {
            match event {
                AppEvent::LoginSuccess { user, token } => {
                    self.login(user, token);
                    self.notify_success("Login successful!".to_string());
                }
                AppEvent::LoginError { message } => {
                    self.notify_error(format!("Login failed: {}", message));
                }
                AppEvent::RegisterSuccess { user, token } => {
                    self.login(user, token);
                    self.notify_success("Registration successful!".to_string());
                }
                AppEvent::RegisterError { message } => {
                    self.notify_error(format!("Registration failed: {}", message));
                }
                AppEvent::ProjectsLoaded { projects } => {
                    self.projects = projects;
                    self.is_loading = false;
                }
                AppEvent::ProjectCreated { project } => {
                    self.projects.push(project);
                    self.notify_success("Project created successfully!".to_string());
                }
                AppEvent::ProjectUpdated { project } => {
                    if let Some(p) = self.projects.iter_mut().find(|p| p.id == project.id) {
                        *p = project;
                    }
                    self.notify_success("Project updated successfully!".to_string());
                }
                AppEvent::ProjectDeleted { project_id } => {
                    use worknest_core::models::ProjectId;
                    if let Ok(id) = ProjectId::from_string(&project_id) {
                        self.projects.retain(|p| p.id != id);
                        self.notify_success("Project deleted successfully!".to_string());
                    }
                }
                AppEvent::ProjectError { message } => {
                    self.notify_error(format!("Project error: {}", message));
                }
                AppEvent::TicketsLoaded { tickets } => {
                    self.tickets = tickets;
                    self.is_loading = false;
                }
                AppEvent::TicketCreated { ticket } => {
                    self.tickets.push(ticket);
                    self.notify_success("Ticket created successfully!".to_string());
                }
                AppEvent::TicketUpdated { ticket } => {
                    if let Some(t) = self.tickets.iter_mut().find(|t| t.id == ticket.id) {
                        *t = ticket;
                    }
                    self.notify_success("Ticket updated successfully!".to_string());
                }
                AppEvent::TicketDeleted { ticket_id } => {
                    use worknest_core::models::TicketId;
                    if let Ok(id) = TicketId::from_string(&ticket_id) {
                        self.tickets.retain(|t| t.id != id);
                        self.notify_success("Ticket deleted successfully!".to_string());
                    }
                }
                AppEvent::TicketError { message } => {
                    self.notify_error(format!("Ticket error: {}", message));
                }
                AppEvent::CommentsLoaded { comments } => {
                    self.comments = comments;
                    self.is_loading = false;
                }
                AppEvent::CommentCreated { comment } => {
                    self.comments.push(comment);
                    self.notify_success("Comment added successfully!".to_string());
                }
                AppEvent::CommentUpdated { comment } => {
                    if let Some(c) = self.comments.iter_mut().find(|c| c.id == comment.id) {
                        *c = comment;
                    }
                    self.notify_success("Comment updated successfully!".to_string());
                }
                AppEvent::CommentDeleted { comment_id } => {
                    use worknest_core::models::CommentId;
                    if let Ok(id) = CommentId::from_string(&comment_id) {
                        self.comments.retain(|c| c.id != id);
                    }
                    self.notify_success("Comment deleted successfully!".to_string());
                }
                AppEvent::CommentError { message } => {
                    self.notify_error(format!("Comment error: {}", message));
                }
                AppEvent::ApiError { message } => {
                    self.notify_error(format!("API error: {}", message));
                    self.is_loading = false;
                }
                AppEvent::LoadingComplete => {
                    self.is_loading = false;
                }
                AppEvent::ProjectLoaded { project } => {
                    // Update single project in list if it exists
                    if let Some(p) = self.projects.iter_mut().find(|p| p.id == project.id) {
                        *p = project;
                    }
                }
                AppEvent::TicketLoaded { ticket } => {
                    // Update single ticket in list if it exists
                    if let Some(t) = self.tickets.iter_mut().find(|t| t.id == ticket.id) {
                        *t = ticket;
                    }
                }
            }
        }
    }

    /// Navigate to a screen
    pub fn navigate_to(&mut self, screen: Screen) {
        self.current_screen = screen;
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.current_user.is_some() && self.auth_token.is_some()
    }

    /// Login user with authentication token
    pub fn login(&mut self, user: User, token: String) {
        self.current_user = Some(user.clone());
        self.auth_token = Some(token.clone());

        // Persist to localStorage
        use gloo_storage::{LocalStorage, Storage};
        let _ = LocalStorage::set("auth_token", token);
        let _ = LocalStorage::set("current_user", user);

        self.navigate_to(Screen::Dashboard);
    }

    /// Try to restore session from localStorage
    pub fn try_restore_session(&mut self) -> bool {
        use gloo_storage::{LocalStorage, Storage};

        let token: Result<String, _> = LocalStorage::get("auth_token");
        let user: Result<User, _> = LocalStorage::get("current_user");

        if let (Ok(token), Ok(user)) = (token, user) {
            self.current_user = Some(user);
            self.auth_token = Some(token);
            self.navigate_to(Screen::Dashboard);
            true
        } else {
            false
        }
    }

    /// Logout user
    #[allow(clippy::let_unit_value)]
    pub fn logout(&mut self) {
        self.current_user = None;
        self.auth_token = None;
        self.navigate_to(Screen::Login);

        // Clear local storage
        use gloo_storage::{LocalStorage, Storage};
        let _ = LocalStorage::delete("auth_token");
        let _ = LocalStorage::delete("current_user");
    }

    /// Add a notification
    pub fn add_notification(&mut self, message: String, level: NotificationLevel) {
        self.notifications.push(Notification {
            message,
            level,
            timestamp: Instant::now(),
        });

        // Keep only last 10 notifications
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    /// Clear old notifications (older than 5 seconds)
    pub fn clear_old_notifications(&mut self) {
        let now = Instant::now();
        self.notifications
            .retain(|n| now.duration_since(n.timestamp).as_secs() < 5);
    }

    /// Add success notification
    pub fn notify_success(&mut self, message: String) {
        self.add_notification(message, NotificationLevel::Success);
    }

    /// Add error notification
    pub fn notify_error(&mut self, message: String) {
        self.add_notification(message, NotificationLevel::Error);
    }

    /// Add info notification
    pub fn notify_info(&mut self, message: String) {
        self.add_notification(message, NotificationLevel::Info);
    }
}

/// Notification for user feedback
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub timestamp: Instant,
}

/// Notification level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Error,
}
