//! Registration screen

use egui::RichText;

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Registration screen state
#[derive(Default)]
pub struct RegisterScreen {
    username: String,
    email: String,
    password: String,
    confirm_password: String,
    error_message: Option<String>,
}

impl RegisterScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);

                // Title
                ui.heading(RichText::new("Create Account").size(36.0).strong());
                ui.label(
                    RichText::new("Join Worknest to manage your projects")
                        .size(14.0)
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(40.0);

                // Registration form
                ui.vertical(|ui| {
                    ui.set_max_width(400.0);

                    // Username field
                    ui.label("Username");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.username)
                            .hint_text("Choose a username")
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(Spacing::LARGE);

                    // Email field
                    ui.label("Email");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.email)
                            .hint_text("your@email.com")
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(Spacing::LARGE);

                    // Password field
                    ui.label("Password");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.password)
                            .password(true)
                            .hint_text("At least 8 characters")
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(Spacing::LARGE);

                    // Confirm password field
                    ui.label("Confirm Password");
                    let confirm_response = ui.add(
                        egui::TextEdit::singleline(&mut self.confirm_password)
                            .password(true)
                            .hint_text("Re-enter password")
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(Spacing::XLARGE);

                    // Error message
                    if let Some(error) = &self.error_message {
                        ui.label(RichText::new(error).color(Colors::ERROR));
                        ui.add_space(Spacing::MEDIUM);
                    }

                    // Register button
                    let register_button = ui.add_sized(
                        [f32::INFINITY, 40.0],
                        egui::Button::new("Register").fill(Colors::PRIMARY),
                    );

                    // Handle enter key
                    if confirm_response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        self.attempt_register(state);
                    }

                    // Handle register button click
                    if register_button.clicked() {
                        self.attempt_register(state);
                    }

                    ui.add_space(Spacing::LARGE);

                    // Login link
                    ui.horizontal(|ui| {
                        ui.label("Already have an account?");
                        if ui.link("Login").clicked() {
                            state.navigate_to(Screen::Login);
                        }
                    });
                });
            });
        });
    }

    fn attempt_register(&mut self, state: &mut AppState) {
        self.error_message = None;

        // Validate inputs
        if self.username.is_empty() {
            self.error_message = Some("Please enter a username".to_string());
            return;
        }

        if self.username.len() < 3 {
            self.error_message = Some("Username must be at least 3 characters".to_string());
            return;
        }

        if self.email.is_empty() {
            self.error_message = Some("Please enter an email".to_string());
            return;
        }

        if !self.email.contains('@') {
            self.error_message = Some("Please enter a valid email".to_string());
            return;
        }

        if self.password.is_empty() {
            self.error_message = Some("Please enter a password".to_string());
            return;
        }

        if self.password.len() < 8 {
            self.error_message = Some("Password must be at least 8 characters".to_string());
            return;
        }

        if self.password != self.confirm_password {
            self.error_message = Some("Passwords do not match".to_string());
            return;
        }

        state.is_loading = true;

        // Attempt registration
        match state
            .auth_service
            .register(&self.username, &self.email, &self.password)
        {
            Ok(user) => {
                // Auto-login after registration
                match state.auth_service.login(&self.username, &self.password) {
                    Ok(token) => {
                        state.login(user, token);
                        state.notify_success("Account created successfully!".to_string());

                        // Clear form
                        self.username.clear();
                        self.email.clear();
                        self.password.clear();
                        self.confirm_password.clear();
                        self.error_message = None;
                    },
                    Err(_) => {
                        // Registration succeeded but login failed, navigate to login
                        state.notify_success("Account created! Please login.".to_string());
                        state.navigate_to(Screen::Login);
                    },
                }
            },
            Err(e) => {
                self.error_message = Some(match e {
                    worknest_auth::AuthError::UserExists => {
                        "Username or email already exists".to_string()
                    },
                    worknest_auth::AuthError::PasswordValidation(msg) => msg,
                    _ => "Registration failed. Please try again.".to_string(),
                });
            },
        }

        state.is_loading = false;
    }
}
