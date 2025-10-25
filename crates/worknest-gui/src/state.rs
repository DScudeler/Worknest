//! Application state management

use std::sync::Arc;

use worknest_auth::{AuthService, AuthToken};
use worknest_core::models::User;
use worknest_db::{DbPool, ProjectRepository, TicketRepository, UserRepository};

use crate::screens::Screen;

/// Main application state
#[derive(Clone)]
pub struct AppState {
    /// Current authenticated user
    pub current_user: Option<User>,
    /// Authentication token
    pub auth_token: Option<AuthToken>,
    /// Current screen
    pub current_screen: Screen,
    /// Database pool
    pub pool: Arc<DbPool>,
    /// Authentication service
    pub auth_service: Arc<AuthService>,
    /// User repository
    pub user_repo: Arc<UserRepository>,
    /// Project repository
    pub project_repo: Arc<ProjectRepository>,
    /// Ticket repository
    pub ticket_repo: Arc<TicketRepository>,
    /// Notification messages
    pub notifications: Vec<Notification>,
    /// Loading state
    pub is_loading: bool,
}

impl AppState {
    /// Create a new application state
    pub fn new(pool: Arc<DbPool>, secret_key: String) -> Self {
        let user_repo = Arc::new(UserRepository::new(Arc::clone(&pool)));
        let project_repo = Arc::new(ProjectRepository::new(Arc::clone(&pool)));
        let ticket_repo = Arc::new(TicketRepository::new(Arc::clone(&pool)));
        let auth_service = Arc::new(AuthService::new(
            Arc::clone(&user_repo),
            secret_key,
            Some(24),
        ));

        Self {
            current_user: None,
            auth_token: None,
            current_screen: Screen::Login,
            pool,
            auth_service,
            user_repo,
            project_repo,
            ticket_repo,
            notifications: Vec::new(),
            is_loading: false,
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

    /// Login user
    pub fn login(&mut self, user: User, token: AuthToken) {
        self.current_user = Some(user);
        self.auth_token = Some(token);
        self.navigate_to(Screen::Dashboard);
    }

    /// Logout user
    pub fn logout(&mut self) {
        self.current_user = None;
        self.auth_token = None;
        self.navigate_to(Screen::Login);
    }

    /// Add a notification
    pub fn add_notification(&mut self, message: String, level: NotificationLevel) {
        self.notifications.push(Notification {
            message,
            level,
            timestamp: std::time::Instant::now(),
        });

        // Keep only last 10 notifications
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    /// Clear old notifications (older than 5 seconds)
    pub fn clear_old_notifications(&mut self) {
        let now = std::time::Instant::now();
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
    pub timestamp: std::time::Instant,
}

/// Notification level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Error,
}
