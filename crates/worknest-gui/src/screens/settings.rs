//! Settings screen

use egui::{RichText, ScrollArea};

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Settings screen tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    Profile,
    Application,
    About,
}

/// Settings screen
pub struct SettingsScreen {
    active_tab: SettingsTab,
    // Profile tab fields
    edit_username: String,
    edit_email: String,
    current_password: String,
    new_password: String,
    confirm_password: String,
    // Application tab fields
    selected_theme: ThemeOption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeOption {
    Dark,
    Light,
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self {
            active_tab: SettingsTab::Profile,
            edit_username: String::new(),
            edit_email: String::new(),
            current_password: String::new(),
            new_password: String::new(),
            confirm_password: String::new(),
            selected_theme: ThemeOption::Dark,
        }
    }
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        // Initialize fields from current user
        if self.edit_username.is_empty() {
            if let Some(user) = &state.current_user {
                self.edit_username = user.username.clone();
                self.edit_email = user.email.clone();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(Spacing::LARGE);

                // Header
                ui.horizontal(|ui| {
                    ui.heading(RichText::new("Settings").size(24.0));

                    ui.add_space(Spacing::MEDIUM);

                    if ui.button("← Back").clicked() {
                        state.navigate_to(Screen::Dashboard);
                    }
                });

                ui.add_space(Spacing::LARGE);

                // Tab bar
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(
                            self.active_tab == SettingsTab::Profile,
                            RichText::new("Profile").size(16.0),
                        )
                        .clicked()
                    {
                        self.active_tab = SettingsTab::Profile;
                    }

                    ui.separator();

                    if ui
                        .selectable_label(
                            self.active_tab == SettingsTab::Application,
                            RichText::new("Application").size(16.0),
                        )
                        .clicked()
                    {
                        self.active_tab = SettingsTab::Application;
                    }

                    ui.separator();

                    if ui
                        .selectable_label(
                            self.active_tab == SettingsTab::About,
                            RichText::new("About").size(16.0),
                        )
                        .clicked()
                    {
                        self.active_tab = SettingsTab::About;
                    }
                });

                ui.add_space(Spacing::XLARGE);

                // Tab content
                match self.active_tab {
                    SettingsTab::Profile => self.render_profile_tab(ui, state),
                    SettingsTab::Application => self.render_application_tab(ui, state),
                    SettingsTab::About => self.render_about_tab(ui, state),
                }
            });
        });
    }

    fn render_profile_tab(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.heading("User Profile");
        ui.add_space(Spacing::MEDIUM);

        // Profile information
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new("Profile Information").strong());
                ui.add_space(Spacing::SMALL);

                ui.label("Username");
                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_username)
                        .desired_width(ui.available_width()),
                );

                ui.add_space(Spacing::SMALL);

                ui.label("Email");
                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_email)
                        .desired_width(ui.available_width()),
                );

                ui.add_space(Spacing::MEDIUM);

                if ui.button("Save Profile").clicked() {
                    self.save_profile(state);
                }
            });
        });

        ui.add_space(Spacing::LARGE);

        // Password change
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new("Change Password").strong());
                ui.add_space(Spacing::SMALL);

                ui.label("Current Password");
                ui.add(
                    egui::TextEdit::singleline(&mut self.current_password)
                        .password(true)
                        .desired_width(ui.available_width()),
                );

                ui.add_space(Spacing::SMALL);

                ui.label("New Password");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_password)
                        .password(true)
                        .desired_width(ui.available_width()),
                );

                ui.add_space(Spacing::SMALL);

                ui.label("Confirm New Password");
                ui.add(
                    egui::TextEdit::singleline(&mut self.confirm_password)
                        .password(true)
                        .desired_width(ui.available_width()),
                );

                ui.add_space(Spacing::MEDIUM);

                let can_change_password = !self.current_password.is_empty()
                    && !self.new_password.is_empty()
                    && !self.confirm_password.is_empty()
                    && self.new_password == self.confirm_password;

                if ui
                    .add_enabled(can_change_password, egui::Button::new("Change Password"))
                    .clicked()
                {
                    self.change_password(state);
                }

                if !self.new_password.is_empty()
                    && !self.confirm_password.is_empty()
                    && self.new_password != self.confirm_password
                {
                    ui.label(
                        RichText::new("Passwords do not match")
                            .color(Colors::ERROR)
                            .italics(),
                    );
                }
            });
        });
    }

    fn render_application_tab(&mut self, ui: &mut egui::Ui, _state: &mut AppState) {
        ui.heading("Application Settings");
        ui.add_space(Spacing::MEDIUM);

        // Theme selection
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new("Appearance").strong());
                ui.add_space(Spacing::SMALL);

                ui.label("Theme");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.selected_theme, ThemeOption::Dark, "Dark");
                    ui.radio_value(&mut self.selected_theme, ThemeOption::Light, "Light");
                });

                ui.add_space(Spacing::SMALL);
                ui.label(
                    RichText::new("Note: Theme changes will be applied in a future update")
                        .color(egui::Color32::GRAY)
                        .italics(),
                );
            });
        });

        ui.add_space(Spacing::LARGE);

        // Notifications
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new("Notifications").strong());
                ui.add_space(Spacing::SMALL);

                ui.checkbox(&mut true, "Enable notifications");
                ui.checkbox(&mut true, "Show toast messages");
                ui.checkbox(&mut false, "Play sound on notifications");

                ui.add_space(Spacing::SMALL);
                ui.label(
                    RichText::new(
                        "Note: Notification preferences will be saved in a future update",
                    )
                    .color(egui::Color32::GRAY)
                    .italics(),
                );
            });
        });

        ui.add_space(Spacing::LARGE);

        // Default views
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new("Defaults").strong());
                ui.add_space(Spacing::SMALL);

                ui.label("Default view after login:");
                ui.radio_value(&mut 0, 0, "Dashboard");
                ui.radio_value(&mut 0, 1, "Project List");
                ui.radio_value(&mut 0, 2, "Ticket List");

                ui.add_space(Spacing::SMALL);
                ui.label(
                    RichText::new(
                        "Note: Default view preferences will be saved in a future update",
                    )
                    .color(egui::Color32::GRAY)
                    .italics(),
                );
            });
        });
    }

    fn render_about_tab(&mut self, ui: &mut egui::Ui, _state: &mut AppState) {
        ui.heading("About Worknest");
        ui.add_space(Spacing::MEDIUM);

        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new("Worknest").strong().size(18.0));
                ui.label("Version: 0.1.0 (MVP)");
                ui.add_space(Spacing::SMALL);

                ui.label(
                    "An open-source project and task manager built for software development teams.",
                );
                ui.add_space(Spacing::MEDIUM);

                ui.label(RichText::new("Built with").strong());
                ui.label("• Rust 🦀");
                ui.label("• egui (immediate mode GUI)");
                ui.label("• WASM (WebAssembly)");
                ui.label("• Axum (backend)");
                ui.label("• SQLite (database)");
                ui.add_space(Spacing::MEDIUM);

                ui.label(RichText::new("License").strong());
                ui.label("MIT License");
                ui.add_space(Spacing::MEDIUM);

                ui.label(RichText::new("Links").strong());
                if ui.link("GitHub Repository").clicked() {
                    // In a real app, this would open the link
                    tracing::info!("GitHub link clicked");
                }
                if ui.link("Documentation").clicked() {
                    tracing::info!("Documentation link clicked");
                }
                if ui.link("Report an Issue").clicked() {
                    tracing::info!("Report issue link clicked");
                }
            });
        });
    }

    fn save_profile(&mut self, state: &mut AppState) {
        // API integration coming soon
        state.notify_info("Profile update API integration coming soon".to_string());
    }

    fn change_password(&mut self, state: &mut AppState) {
        if self.new_password != self.confirm_password {
            state.notify_error("Passwords do not match".to_string());
            return;
        }

        if self.new_password.len() < 8 {
            state.notify_error("Password must be at least 8 characters".to_string());
            return;
        }

        // API integration coming soon
        state.notify_info("Password change API integration coming soon".to_string());
    }
}
