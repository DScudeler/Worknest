//! Application state management

use crate::screens::Screen;
use worknest_core::models::User;

// Native-only imports
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use worknest_auth::{AuthService, AuthToken};
#[cfg(not(target_arch = "wasm32"))]
use worknest_db::{DbPool, ProjectRepository, TicketRepository, UserRepository};

// Web-only imports
#[cfg(target_arch = "wasm32")]
use crate::api_client::ApiClient;

/// Backend type - either native (DB) or web (API)
#[derive(Clone)]
pub enum Backend {
    #[cfg(not(target_arch = "wasm32"))]
    Native {
        pool: Arc<DbPool>,
        auth_service: Arc<AuthService>,
        user_repo: Arc<UserRepository>,
        project_repo: Arc<ProjectRepository>,
        ticket_repo: Arc<TicketRepository>,
    },
    #[cfg(target_arch = "wasm32")]
    Web { api_client: ApiClient },
}

/// Main application state
#[derive(Clone)]
pub struct AppState {
    /// Current authenticated user
    pub current_user: Option<User>,
    /// Authentication token (string for web, AuthToken for native)
    pub auth_token: Option<String>,
    /// Current screen
    pub current_screen: Screen,
    /// Backend (either native DB or web API)
    pub backend: Backend,
    /// Notification messages
    pub notifications: Vec<Notification>,
    /// Loading state
    pub is_loading: bool,
}

impl AppState {
    /// Create a new native application state (with local database)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_native(pool: Arc<DbPool>, secret_key: String) -> Self {
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
            backend: Backend::Native {
                pool,
                auth_service,
                user_repo,
                project_repo,
                ticket_repo,
            },
            notifications: Vec::new(),
            is_loading: false,
        }
    }

    /// Create a new web application state (with API client)
    #[cfg(target_arch = "wasm32")]
    pub fn new_web(api_client: ApiClient) -> Self {
        Self {
            current_user: None,
            auth_token: None,
            current_screen: Screen::Login,
            backend: Backend::Web { api_client },
            notifications: Vec::new(),
            is_loading: false,
        }
    }

    /// Get native backend (native only)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn native_backend(
        &self,
    ) -> Option<(
        &Arc<DbPool>,
        &Arc<AuthService>,
        &Arc<UserRepository>,
        &Arc<ProjectRepository>,
        &Arc<TicketRepository>,
    )> {
        match &self.backend {
            Backend::Native {
                pool,
                auth_service,
                user_repo,
                project_repo,
                ticket_repo,
            } => Some((pool, auth_service, user_repo, project_repo, ticket_repo)),
        }
    }

    /// Get web backend (web only)
    #[cfg(target_arch = "wasm32")]
    pub fn web_backend(&self) -> Option<&ApiClient> {
        match &self.backend {
            Backend::Web { api_client } => Some(api_client),
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

    /// Login user (with string token for both native and web)
    pub fn login(&mut self, user: User, token: String) {
        self.current_user = Some(user);
        self.auth_token = Some(token);
        self.navigate_to(Screen::Dashboard);
    }

    /// Logout user
    pub fn logout(&mut self) {
        self.current_user = None;
        self.auth_token = None;
        self.navigate_to(Screen::Login);

        // Clear local storage on web
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_storage::{LocalStorage, Storage};
            let _ = LocalStorage::delete("auth_token");
            let _ = LocalStorage::delete("current_user");
        }
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
