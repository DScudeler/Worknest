//! Left sidebar navigation component

use egui::{Context, Key, KeyboardShortcut, Modifiers, RichText, Ui};

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Sidebar navigation component with collapsible functionality
pub struct Sidebar {
    /// Whether the sidebar is currently expanded
    pub is_expanded: bool,
    /// Width of the expanded sidebar
    pub expanded_width: f32,
    /// Width of the collapsed sidebar (icons only)
    pub collapsed_width: f32,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            is_expanded: true,
            expanded_width: 220.0,
            collapsed_width: 50.0,
        }
    }

    /// Get the current width based on expanded state
    pub fn width(&self) -> f32 {
        if self.is_expanded {
            self.expanded_width
        } else {
            self.collapsed_width
        }
    }

    /// Toggle sidebar expanded/collapsed state
    pub fn toggle(&mut self) {
        self.is_expanded = !self.is_expanded;
    }

    /// Render the sidebar
    pub fn render(&mut self, ctx: &Context, state: &mut AppState) {
        // Check for keyboard shortcut (Ctrl/Cmd + B)
        if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::B)))
        {
            self.toggle();
        }

        let width = self.width();

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .min_width(width)
            .max_width(width)
            .show(ctx, |ui| {
                // Header with logo and toggle button
                self.render_header(ui);

                ui.add_space(Spacing::LARGE);

                // Main navigation items
                self.render_navigation(ui, state);

                ui.add_space(Spacing::LARGE);

                // Bottom section
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    self.render_footer(ui, state);
                });
            });
    }

    fn render_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if self.is_expanded {
                ui.heading(RichText::new("Worknest").size(20.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(RichText::new("≡").size(18.0))
                        .on_hover_text("Collapse sidebar (Ctrl/Cmd+B)")
                        .clicked()
                    {
                        self.toggle();
                    }
                });
            } else if ui
                .button(RichText::new("≡").size(18.0))
                .on_hover_text("Expand sidebar (Ctrl/Cmd+B)")
                .clicked()
            {
                self.toggle();
            }
        });

        ui.separator();
    }

    fn render_navigation(&self, ui: &mut Ui, state: &mut AppState) {
        let is_authenticated = state.is_authenticated();

        // Only show navigation if authenticated
        if !is_authenticated {
            if self.is_expanded {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Please login").color(egui::Color32::GRAY));
                });
            }
            return;
        }

        // Dashboard
        self.render_nav_item(
            ui,
            "🏠",
            "Dashboard",
            matches!(state.current_screen, Screen::Dashboard),
            || state.navigate_to(Screen::Dashboard),
        );

        // Projects
        self.render_nav_item(
            ui,
            "📁",
            "Projects",
            matches!(state.current_screen, Screen::ProjectList),
            || state.navigate_to(Screen::ProjectList),
        );

        // All Tickets
        self.render_nav_item(
            ui,
            "🎫",
            "All Tickets",
            matches!(state.current_screen, Screen::TicketList { .. }),
            || state.navigate_to(Screen::TicketList { project_id: None }),
        );

        ui.add_space(Spacing::MEDIUM);
        ui.separator();
        ui.add_space(Spacing::MEDIUM);

        // Recent Projects Section (if expanded)
        if self.is_expanded {
            ui.label(
                RichText::new("RECENT PROJECTS")
                    .small()
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(Spacing::SMALL);

            // Collect project data to avoid borrowing issues
            let recent_projects: Vec<_> = state
                .projects
                .iter()
                .take(5)
                .map(|p| (p.id, p.name.clone()))
                .collect();

            if recent_projects.is_empty() {
                ui.label(
                    RichText::new("No projects yet")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            } else {
                for (project_id, project_name) in recent_projects {
                    let is_active = matches!(&state.current_screen, Screen::ProjectDetail(id) if *id == project_id);

                    let response = ui.add(
                        egui::Button::new(RichText::new(&project_name).size(14.0))
                            .fill(if is_active {
                                Colors::PRIMARY.linear_multiply(0.3)
                            } else {
                                egui::Color32::TRANSPARENT
                            })
                            .frame(false),
                    );

                    if response.clicked() {
                        state.navigate_to(Screen::ProjectDetail(project_id));
                    }
                }
            }

            ui.add_space(Spacing::MEDIUM);
            ui.separator();
            ui.add_space(Spacing::MEDIUM);
        }
    }

    fn render_nav_item<F>(&self, ui: &mut Ui, icon: &str, label: &str, is_active: bool, on_click: F)
    where
        F: FnOnce(),
    {
        let button_text = if self.is_expanded {
            format!("{} {}", icon, label)
        } else {
            icon.to_string()
        };

        let mut response = ui.add_sized(
            [ui.available_width(), 36.0],
            egui::Button::new(RichText::new(button_text).size(15.0))
                .fill(if is_active {
                    Colors::PRIMARY.linear_multiply(0.3)
                } else {
                    egui::Color32::TRANSPARENT
                })
                .frame(false),
        );

        if !self.is_expanded {
            response = response.on_hover_text(label);
        }

        if response.clicked() {
            on_click();
        }
    }

    fn render_footer(&self, ui: &mut Ui, state: &mut AppState) {
        ui.add_space(Spacing::SMALL);

        // Settings
        self.render_nav_item(
            ui,
            "⚙️",
            "Settings",
            matches!(state.current_screen, Screen::Settings),
            || state.navigate_to(Screen::Settings),
        );

        // User profile (if expanded)
        if self.is_expanded {
            ui.add_space(Spacing::SMALL);
            ui.separator();
            ui.add_space(Spacing::SMALL);

            if let Some(user) = &state.current_user {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("👤").size(16.0));
                    ui.label(RichText::new(&user.username).size(14.0));
                });
            }
        }

        ui.add_space(Spacing::SMALL);
    }
}
