//! Login screen

use egui::RichText;

use crate::{
    state::AppState,
    theme::{Colors, Spacing},
};

/// Login screen state
#[derive(Default)]
pub struct LoginScreen {
    username: String,
    password: String,
    error_message: Option<String>,
}

impl LoginScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);

                // Logo/Title
                ui.heading(RichText::new("Worknest").size(48.0).strong());
                ui.label(
                    RichText::new("Project & Task Management")
                        .size(16.0)
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(50.0);

                // Login form
                ui.vertical(|ui| {
                    ui.set_max_width(400.0);

                    // Username field
                    ui.label("Username or Email");
                    let username_response = ui.add(
                        egui::TextEdit::singleline(&mut self.username)
                            .hint_text("Enter username or email")
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(Spacing::LARGE);

                    // Password field
                    ui.label("Password");
                    let password_response = ui.add(
                        egui::TextEdit::singleline(&mut self.password)
                            .password(true)
                            .hint_text("Enter password")
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(Spacing::XLARGE);

                    // Error message
                    if let Some(error) = &self.error_message {
                        ui.label(RichText::new(error).color(Colors::ERROR));
                        ui.add_space(Spacing::MEDIUM);
                    }

                    // Login button
                    let login_button = ui.add_sized(
                        [f32::INFINITY, 40.0],
                        egui::Button::new("Login").fill(Colors::PRIMARY),
                    );

                    // Handle enter key
                    if (username_response.lost_focus() || password_response.lost_focus())
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        self.attempt_login(state);
                    }

                    // Handle login button click
                    if login_button.clicked() {
                        self.attempt_login(state);
                    }

                    ui.add_space(Spacing::LARGE);

                    // Register link
                    ui.horizontal(|ui| {
                        ui.label("Don't have an account?");
                        if ui.link("Register").clicked() {
                            state.navigate_to(crate::screens::Screen::Register);
                        }
                    });
                });
            });
        });
    }

    #[allow(clippy::let_unit_value)]
    fn attempt_login(&mut self, state: &mut AppState) {
        self.error_message = None;

        if self.username.is_empty() {
            self.error_message = Some("Please enter username or email".to_string());
            return;
        }

        if self.password.is_empty() {
            self.error_message = Some("Please enter password".to_string());
            return;
        }

        // Call API to login
        let api_client = state.api_client.clone();
        let event_queue = state.event_queue.clone();
        let username = self.username.clone();
        let password = self.password.clone();

        // Clear form and show loading
        self.username.clear();
        self.password.clear();
        state.is_loading = true;

        wasm_bindgen_futures::spawn_local(async move {
            use crate::api_client::LoginRequest;
            use crate::events::AppEvent;

            let request = LoginRequest { username, password };

            match api_client.login(request).await {
                Ok(response) => {
                    tracing::info!("Login successful for user: {}", response.user.username);
                    event_queue.push(AppEvent::LoginSuccess {
                        user: response.user,
                        token: response.token,
                    });
                },
                Err(e) => {
                    tracing::error!("Login failed: {:?}", e);
                    event_queue.push(AppEvent::LoginError {
                        message: e.to_string(),
                    });
                },
            }
        });
    }
}
