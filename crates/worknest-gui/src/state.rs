//! Application state management

use crate::api_client::ApiClient;
use crate::screens::Screen;
use worknest_core::models::{Project, Ticket, User};

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
    /// Notification messages
    pub notifications: Vec<Notification>,
    /// Loading state
    pub is_loading: bool,
    /// Demo projects (for local development without backend)
    pub demo_projects: Vec<Project>,
    /// Demo tickets (for local development without backend)
    pub demo_tickets: Vec<Ticket>,
}

impl AppState {
    /// Create a new web application state with API client
    pub fn new(api_client: ApiClient) -> Self {
        Self {
            current_user: None,
            auth_token: None,
            current_screen: Screen::Login,
            api_client,
            notifications: Vec::new(),
            is_loading: false,
            demo_projects: Vec::new(),
            demo_tickets: Vec::new(),
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
        self.current_user = Some(user);
        self.auth_token = Some(token);
        self.navigate_to(Screen::Dashboard);
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
