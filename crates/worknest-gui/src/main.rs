//! Worknest GUI Application
//!
//! Desktop application for Worknest built with egui.

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;
use worknest_db::{connection, migrations};

use worknest_gui::{
    screens::{
        DashboardScreen, LoginScreen, ProjectDetailScreen, ProjectListScreen, RegisterScreen,
        Screen, TicketBoardScreen, TicketDetailScreen, TicketListScreen,
    },
    state::AppState,
    theme::Theme,
};

fn main() -> Result<(), eframe::Error> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Worknest");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Worknest",
        options,
        Box::new(|cc| Box::new(WorknestApp::new(cc))),
    )
}

/// Load application icon
fn load_icon() -> egui::IconData {
    // Placeholder: will add actual icon later
    egui::IconData {
        rgba: vec![255; 32 * 32 * 4], // White 32x32 icon
        width: 32,
        height: 32,
    }
}

/// Main application structure
struct WorknestApp {
    state: AppState,
    theme: Theme,
    // Screen instances
    login_screen: LoginScreen,
    register_screen: RegisterScreen,
    dashboard_screen: DashboardScreen,
    project_list_screen: ProjectListScreen,
    // Detail screens are created on demand
    project_detail_screen: Option<ProjectDetailScreen>,
    ticket_list_screen: Option<TicketListScreen>,
    ticket_board_screen: Option<TicketBoardScreen>,
    ticket_detail_screen: Option<TicketDetailScreen>,
}

impl WorknestApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize database
        let db_path = get_database_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).expect("Failed to create database directory");
            }
        }

        // Initialize database pool
        let pool = connection::init_pool(&db_path).expect("Failed to initialize database");

        // Run migrations
        migrations::run_migrations(&mut pool.get().expect("Failed to get connection"))
            .expect("Failed to run database migrations");

        tracing::info!("Database initialized at {:?}", db_path);

        // Get secret key from environment or use default for development
        let secret_key = std::env::var("WORKNEST_SECRET_KEY")
            .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string());

        // Create application state
        let state = AppState::new(Arc::new(pool), secret_key);

        Self {
            state,
            theme: Theme::Dark,
            login_screen: LoginScreen::new(),
            register_screen: RegisterScreen::new(),
            dashboard_screen: DashboardScreen::new(),
            project_list_screen: ProjectListScreen::new(),
            project_detail_screen: None,
            ticket_list_screen: None,
            ticket_board_screen: None,
            ticket_detail_screen: None,
        }
    }

    fn render_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Worknest");

                ui.separator();

                // Navigation
                if ui.button("Dashboard").clicked() {
                    self.state.navigate_to(Screen::Dashboard);
                }

                if ui.button("Projects").clicked() {
                    self.state.navigate_to(Screen::ProjectList);
                }

                if ui.button("All Tickets").clicked() {
                    self.state
                        .navigate_to(Screen::TicketList { project_id: None });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Logout button
                    if ui.button("Logout").clicked() {
                        self.state.logout();
                    }

                    ui.separator();

                    // Theme toggle
                    let theme_text = match self.theme {
                        Theme::Light => "🌙 Dark",
                        Theme::Dark => "☀ Light",
                    };
                    if ui.button(theme_text).clicked() {
                        self.theme = self.theme.toggle();
                    }

                    ui.separator();

                    // User info
                    if let Some(user) = &self.state.current_user {
                        ui.label(format!("👤 {}", user.username));
                    }
                });
            });
        });
    }

    fn render_notifications(&mut self, ctx: &egui::Context) {
        // Clear old notifications
        self.state.clear_old_notifications();

        // Render notifications at the top of the screen
        if !self.state.notifications.is_empty() {
            egui::TopBottomPanel::top("notifications").show(ctx, |ui| {
                for notification in &self.state.notifications {
                    let color = match notification.level {
                        worknest_gui::state::NotificationLevel::Success => {
                            worknest_gui::theme::Colors::SUCCESS
                        },
                        worknest_gui::state::NotificationLevel::Error => {
                            worknest_gui::theme::Colors::ERROR
                        },
                        worknest_gui::state::NotificationLevel::Info => {
                            worknest_gui::theme::Colors::INFO
                        },
                    };

                    ui.colored_label(color, &notification.message);
                }
            });
        }
    }
}

impl eframe::App for WorknestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        self.theme.apply(ctx);

        // Show top bar if authenticated
        if self.state.is_authenticated() {
            self.render_top_bar(ctx);
        }

        // Show notifications
        self.render_notifications(ctx);

        // Render current screen
        match &self.state.current_screen {
            Screen::Login => {
                self.login_screen.render(ctx, &mut self.state);
            },
            Screen::Register => {
                self.register_screen.render(ctx, &mut self.state);
            },
            Screen::Dashboard => {
                self.dashboard_screen.render(ctx, &mut self.state);
            },
            Screen::ProjectList => {
                self.project_list_screen.render(ctx, &mut self.state);
            },
            Screen::ProjectDetail(project_id) => {
                // Create screen if it doesn't exist or if project_id changed
                if self.project_detail_screen.is_none()
                    || self
                        .project_detail_screen
                        .as_ref()
                        .map(|s| s.project_id != *project_id)
                        .unwrap_or(false)
                {
                    self.project_detail_screen = Some(ProjectDetailScreen::new(*project_id));
                }

                if let Some(screen) = &mut self.project_detail_screen {
                    screen.render(ctx, &mut self.state);
                }
            },
            Screen::TicketList { project_id } => {
                // Create screen if it doesn't exist or if project_id changed
                if self.ticket_list_screen.is_none()
                    || self
                        .ticket_list_screen
                        .as_ref()
                        .map(|s| s.project_id != *project_id)
                        .unwrap_or(false)
                {
                    self.ticket_list_screen = Some(TicketListScreen::new(*project_id));
                }

                if let Some(screen) = &mut self.ticket_list_screen {
                    screen.render(ctx, &mut self.state);
                }
            },
            Screen::TicketBoard { project_id } => {
                // Create screen if it doesn't exist or if project_id changed
                if self.ticket_board_screen.is_none()
                    || self
                        .ticket_board_screen
                        .as_ref()
                        .map(|s| s.project_id != *project_id)
                        .unwrap_or(false)
                {
                    self.ticket_board_screen = Some(TicketBoardScreen::new(*project_id));
                }

                if let Some(screen) = &mut self.ticket_board_screen {
                    screen.render(ctx, &mut self.state);
                }
            },
            Screen::TicketDetail(ticket_id) => {
                // Create screen if it doesn't exist or if ticket_id changed
                if self.ticket_detail_screen.is_none()
                    || self
                        .ticket_detail_screen
                        .as_ref()
                        .map(|s| s.ticket_id != *ticket_id)
                        .unwrap_or(false)
                {
                    self.ticket_detail_screen = Some(TicketDetailScreen::new(*ticket_id));
                }

                if let Some(screen) = &mut self.ticket_detail_screen {
                    screen.render(ctx, &mut self.state);
                }
            },
            Screen::Settings => {
                // Settings screen not implemented yet
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Settings");
                    ui.label("Settings screen coming soon!");
                });
            },
        }
    }
}

/// Get database path from environment or use default
fn get_database_path() -> PathBuf {
    if let Ok(path) = std::env::var("WORKNEST_DB_PATH") {
        PathBuf::from(path)
    } else {
        // Default to ~/.worknest/worknest.db
        let mut path = dirs::home_dir().expect("Failed to get home directory");
        path.push(".worknest");
        path.push("worknest.db");
        path
    }
}
