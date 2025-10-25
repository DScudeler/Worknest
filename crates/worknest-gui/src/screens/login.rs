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

        state.is_loading = true;

        // Attempt login
        match state.auth_service.login(&self.username, &self.password) {
            Ok(token) => {
                // Get user from token
                match state.auth_service.get_user_from_token(&token.token) {
                    Ok(user) => {
                        state.login(user, token);
                        state.notify_success("Login successful!".to_string());

                        // Clear form
                        self.username.clear();
                        self.password.clear();
                        self.error_message = None;
                    },
                    Err(e) => {
                        self.error_message = Some(format!("Failed to get user: {:?}", e));
                    },
                }
            },
            Err(_e) => {
                self.error_message = Some("Invalid username or password".to_string());
            },
        }

        state.is_loading = false;
    }
}
