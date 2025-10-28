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
                        // Create the card and track button interactions
                        let group_response = ui.group(|ui| {
                            ui.set_min_size([f32::INFINITY, 60.0].into());

                            let mut view_clicked = false;

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
                                            view_clicked = true;
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

                            view_clicked
                        });

                        // Extract the button click state
                        let view_clicked = group_response.inner;

                        // Make the entire card interactive with BOTH click and hover detection
                        let card_response = group_response.response.interact(
                            egui::Sense::click().union(egui::Sense::hover())
                        );

                        // Apply hover styling with visual feedback
                        if card_response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);

                            // Add subtle background color change on hover
                            let hover_fill = if ui.style().visuals.dark_mode {
                                egui::Color32::from_gray(50)
                            } else {
                                egui::Color32::from_gray(240)
                            };

                            ui.painter().rect_filled(
                                card_response.rect,
                                4.0,
                                hover_fill.linear_multiply(0.3),
                            );
                        }

                        // Handle button click or card click
                        if view_clicked || card_response.clicked() {
                            state.navigate_to(Screen::ProjectDetail(project.id));
                        }

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

    fn load_data(&mut self, state: &AppState) {
        // Demo mode: Load from in-memory state
        // TODO: Replace with API call when backend is available
        // wasm_bindgen_futures::spawn_local(async move {
        //     if let Some(token) = &state.auth_token {
        //         match state.api_client.get_projects(token).await {
        //             Ok(projects) => { /* update state */ },
        //             Err(e) => { /* handle error */ },
        //         }
        //     }
        // });

        self.recent_projects = state.demo_projects.clone();
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
