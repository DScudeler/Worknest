//! Dashboard screen

use egui::{RichText, ScrollArea};

use worknest_core::models::Project;

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Dashboard screen
#[derive(Default)]
pub struct DashboardScreen {
    recent_projects: Vec<Project>,
    stats_loaded: bool,
}

impl DashboardScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        // Load data on first render
        if !self.stats_loaded {
            self.load_data(state);
            self.stats_loaded = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(Spacing::LARGE);

                // Welcome header
                if let Some(user) = &state.current_user {
                    ui.heading(RichText::new(format!("Welcome, {}!", user.username)).size(32.0));
                } else {
                    ui.heading("Dashboard");
                }

                ui.add_space(Spacing::XLARGE);

                // Quick actions
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [150.0, 40.0],
                            egui::Button::new("New Project").fill(Colors::PRIMARY),
                        )
                        .clicked()
                    {
                        state.navigate_to(Screen::ProjectList);
                    }

                    if ui
                        .add_sized(
                            [150.0, 40.0],
                            egui::Button::new("View Tickets").fill(Colors::PRIMARY),
                        )
                        .clicked()
                    {
                        state.navigate_to(Screen::TicketList { project_id: None });
                    }
                });

                ui.add_space(Spacing::XLARGE);

                // Stats cards
                ui.columns(3, |columns| {
                    // Total projects
                    columns[0].group(|ui| {
                        ui.set_min_size([200.0, 100.0].into());
                        ui.vertical_centered(|ui| {
                            ui.add_space(Spacing::LARGE);
                            ui.label(
                                RichText::new(format!("{}", self.recent_projects.len()))
                                    .size(36.0)
                                    .strong()
                                    .color(Colors::PRIMARY),
                            );
                            ui.label("Total Projects");
                            ui.add_space(Spacing::LARGE);
                        });
                    });

                    // Active projects
                    let active_count = self.recent_projects.iter().filter(|p| !p.archived).count();
                    columns[1].group(|ui| {
                        ui.set_min_size([200.0, 100.0].into());
                        ui.vertical_centered(|ui| {
                            ui.add_space(Spacing::LARGE);
                            ui.label(
                                RichText::new(format!("{}", active_count))
                                    .size(36.0)
                                    .strong()
                                    .color(Colors::SUCCESS),
                            );
                            ui.label("Active Projects");
                            ui.add_space(Spacing::LARGE);
                        });
                    });

                    // Archived projects
                    let archived_count = self.recent_projects.len() - active_count;
                    columns[2].group(|ui| {
                        ui.set_min_size([200.0, 100.0].into());
                        ui.vertical_centered(|ui| {
                            ui.add_space(Spacing::LARGE);
                            ui.label(
                                RichText::new(format!("{}", archived_count))
                                    .size(36.0)
                                    .strong()
                                    .color(egui::Color32::GRAY),
                            );
                            ui.label("Archived Projects");
                            ui.add_space(Spacing::LARGE);
                        });
                    });
                });

                ui.add_space(Spacing::XLARGE);

                // Recent projects
                ui.heading("Recent Projects");
                ui.add_space(Spacing::MEDIUM);

                if self.recent_projects.is_empty() {
                    ui.label(
                        RichText::new("No projects yet. Create your first project to get started!")
                            .color(egui::Color32::GRAY),
                    );
                } else {
                    for project in self.recent_projects.iter().take(5) {
                        ui.group(|ui| {
                            ui.set_min_size([f32::INFINITY, 60.0].into());
                            ui.horizontal(|ui| {
                                // Project color indicator
                                if let Some(color) = &project.color {
                                    if let Ok(color_val) = parse_hex_color(color) {
                                        ui.colored_label(color_val, "●");
                                    }
                                }

                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&project.name).strong());
                                    if let Some(desc) = &project.description {
                                        ui.label(
                                            RichText::new(desc).small().color(egui::Color32::GRAY),
                                        );
                                    }
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("View").clicked() {
                                            state.navigate_to(Screen::ProjectDetail(project.id));
                                        }

                                        if project.archived {
                                            ui.label(
                                                RichText::new("Archived")
                                                    .small()
                                                    .color(egui::Color32::GRAY),
                                            );
                                        }
                                    },
                                );
                            });
                        });
                        ui.add_space(Spacing::SMALL);
                    }
                }

                ui.add_space(Spacing::LARGE);

                // View all projects link
                if ui.link("View All Projects →").clicked() {
                    state.navigate_to(Screen::ProjectList);
                }
            });
        });
    }

    fn load_data(&mut self, _state: &AppState) {
        // TODO: Load recent projects from API
        // This should call state.api_client.get_projects()
        self.recent_projects = Vec::new();
    }
}

/// Parse hex color string
fn parse_hex_color(hex: &str) -> Result<egui::Color32, ()> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(());
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;

    Ok(egui::Color32::from_rgb(r, g, b))
}
