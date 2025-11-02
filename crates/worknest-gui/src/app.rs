//! Main Worknest application module

use eframe::egui;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::{
    api_client::ApiClient,
    components::{Breadcrumb, CommandAction, CommandPalette, ShortcutsHelp, Sidebar, ToastManager},
    screens::{
        DashboardScreen, LoginScreen, ProjectDetailScreen, ProjectListScreen, RegisterScreen,
        Screen, SettingsScreen, TicketBoardScreen, TicketDetailScreen, TicketListScreen,
    },
    state::AppState,
    theme::Theme,
};

/// Main application structure
pub struct WorknestApp {
    state: AppState,
    theme: Theme,
    sidebar: Sidebar,
    shortcuts_help: ShortcutsHelp,
    command_palette: CommandPalette,
    toast_manager: ToastManager,
    breadcrumb: Breadcrumb,
    // Screen instances
    login_screen: LoginScreen,
    register_screen: RegisterScreen,
    dashboard_screen: DashboardScreen,
    project_list_screen: ProjectListScreen,
    settings_screen: SettingsScreen,
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

        // Create API client with default localhost URL
        let api_client = ApiClient::new_default();
        tracing::info!("API client configured for: http://localhost:3000");

        // Create application state with API client
        let mut state = AppState::new(api_client);

        // Try to restore session from localStorage
        if state.try_restore_session() {
            tracing::info!("Session restored from localStorage");
        }

        // Parse hash from URL to determine initial screen (#/register, #/dashboard, etc.)
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let location = window.location();
                if let Ok(hash) = location.hash() {
                    if !hash.is_empty() {
                        let screen = match hash.as_str() {
                            "#/register" => Some(Screen::Register),
                            "#/dashboard" => Some(Screen::Dashboard),
                            "#/projects" => Some(Screen::ProjectList),
                            "#/settings" => Some(Screen::Settings),
                            "#/login" | "#/" => Some(Screen::Login),
                            _ => None,
                        };

                        if let Some(screen) = screen {
                            tracing::info!("Initial screen from hash '{}': {:?}", hash, screen);
                            state.current_screen = screen;
                        } else {
                            tracing::warn!("Unknown hash route: {}", hash);
                        }
                    }
                }
            }
        }

        Self {
            state,
            theme: Theme::Dark,
            sidebar: Sidebar::new(),
            shortcuts_help: ShortcutsHelp::new(),
            command_palette: CommandPalette::new(),
            toast_manager: ToastManager::new(),
            breadcrumb: Breadcrumb::new(),
            login_screen: LoginScreen::new(),
            register_screen: RegisterScreen::new(),
            dashboard_screen: DashboardScreen::new(),
            project_list_screen: ProjectListScreen::new(),
            settings_screen: SettingsScreen::new(),
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
                });
            });
        });
    }

    fn render_notifications(&mut self, ctx: &egui::Context) {
        // Update toast manager with current notifications
        self.toast_manager
            .update_from_notifications(&self.state.notifications);

        // Render toasts
        self.toast_manager.render(ctx);

        // Clear old notifications from state
        self.state.clear_old_notifications();
    }
}

impl eframe::App for WorknestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any pending events from async operations
        self.state.process_events();

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

        // Show sidebar and top bar if authenticated
        if self.state.is_authenticated() {
            self.sidebar.render(ctx, &mut self.state);
            self.render_top_bar(ctx);

            // Update and render breadcrumb navigation
            self.breadcrumb
                .update(&self.state.current_screen, &self.state);
            self.breadcrumb.render(ctx, &mut self.state);
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
                self.settings_screen.render(ctx, &mut self.state);
            },
        }

        // Check for keyboard shortcuts and render help modal (always, even on login screen)
        self.shortcuts_help.check_shortcut(ctx);
        self.shortcuts_help.render(ctx);

        // Check for command palette shortcut and render (only when authenticated)
        if self.state.is_authenticated() {
            self.command_palette.check_shortcut(ctx);
            if let Some(action) = self.command_palette.render(ctx, &mut self.state) {
                self.execute_command_action(action);
            }
        }
    }
}

impl WorknestApp {
    /// Execute a command action from the command palette
    fn execute_command_action(&mut self, action: CommandAction) {
        match action {
            CommandAction::Navigate(screen) => {
                self.state.navigate_to(screen);
            },
            CommandAction::ToggleSidebar => {
                self.sidebar.toggle();
            },
            CommandAction::ToggleTheme => {
                self.theme = self.theme.toggle();
            },
            CommandAction::ShowHelp => {
                self.shortcuts_help.open();
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
