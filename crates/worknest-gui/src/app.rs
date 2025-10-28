//! Main Worknest application module

use eframe::egui;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::{
    api_client::ApiClient,
    screens::{
        DashboardScreen, LoginScreen, ProjectDetailScreen, ProjectListScreen, RegisterScreen,
        Screen, TicketBoardScreen, TicketDetailScreen, TicketListScreen,
    },
    state::AppState,
    theme::Theme,
};

/// Main application structure
pub struct WorknestApp {
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
    // Track if this is the first frame to hide loading screen
    first_frame: bool,
}

impl WorknestApp {
    /// Create web app with API client
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        tracing::info!("WorknestApp::new() called - initializing application");

        // Parse app mode from URL parameter (?mode=demo or ?mode=integrated)
        // Default to Integrated mode
        let app_mode = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .and_then(|search| {
                if search.is_empty() {
                    None
                } else {
                    // Parse URLSearchParams
                    web_sys::UrlSearchParams::new_with_str(&search)
                        .ok()
                        .and_then(|params| params.get("mode"))
                }
            })
            .map(|mode_str| {
                let mode = crate::app_mode::AppMode::from_str(&mode_str);
                tracing::info!("App mode from URL: {:?}", mode);
                mode
            })
            .unwrap_or_else(|| {
                tracing::info!("App mode: Integrated (default)");
                crate::app_mode::AppMode::Integrated
            });

        // Get API URL from window.location or environment
        let api_url = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "http://localhost:3000".to_string());

        tracing::info!("API URL: {}", api_url);

        // Create API client
        let api_client = ApiClient::new(api_url);

        // Create application state with API client and mode
        let state = AppState::new(api_client, app_mode);

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
            first_frame: true,
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
                        crate::state::NotificationLevel::Success => crate::theme::Colors::SUCCESS,
                        crate::state::NotificationLevel::Error => crate::theme::Colors::ERROR,
                        crate::state::NotificationLevel::Info => crate::theme::Colors::INFO,
                    };

                    ui.colored_label(color, &notification.message);
                }
            });
        }
    }
}

impl eframe::App for WorknestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Hide loading screen on first frame (only on WASM)
        #[cfg(target_arch = "wasm32")]
        if self.first_frame {
            self.first_frame = false;
            tracing::info!(
                "First frame! Current screen: {:?}",
                self.state.current_screen
            );
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(loading) = document.get_element_by_id("loading") {
                        let _ = loading.set_attribute("style", "display: none;");
                        tracing::info!(
                            "Loading screen hidden - login screen should now be visible"
                        );
                    } else {
                        tracing::warn!("Could not find loading element!");
                    }
                } else {
                    tracing::warn!("Could not get document!");
                }
            } else {
                tracing::warn!("Could not get window!");
            }
        }

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

/// WASM entry point - initializes and runs the application
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    // Initialize panic hook for better error messages (safe to call multiple times)
    console_error_panic_hook::set_once();

    // Setup logging - ignore error if already initialized (e.g., during hot reload)
    let _ = tracing_wasm::try_set_as_global_default();

    tracing::info!("Starting Worknest (Web)");

    // Get the canvas element from the DOM
    let document = web_sys::window()
        .expect("No window found")
        .document()
        .expect("No document found");

    // Get the existing canvas element
    let canvas = document
        .get_element_by_id("worknest_canvas")
        .expect("Canvas element 'worknest_canvas' not found in HTML")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("Failed to cast to canvas");

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async move {
        tracing::info!("Creating eframe WebRunner...");

        let result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    tracing::info!("Creating WorknestApp...");
                    Ok(Box::new(WorknestApp::new(cc)) as Box<dyn eframe::App>)
                }),
            )
            .await;

        match result {
            Ok(_) => {
                tracing::info!("eframe started successfully! Waiting for first frame to hide loading screen...");
                // Loading screen will be hidden in App::update() on first frame
            },
            Err(e) => {
                tracing::error!("Failed to start eframe: {:?}", e);
                // Show error in the error div
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(error_div) = document.get_element_by_id("error") {
                            let _ = error_div.set_attribute("style", "display: block;");
                            error_div.set_inner_html(&format!(
                                "<h2>Failed to load application</h2><p>{:?}</p>",
                                e
                            ));
                        }
                        if let Some(loading) = document.get_element_by_id("loading") {
                            let _ = loading.set_attribute("style", "display: none;");
                        }
                    }
                }
            },
        }
    });
}
