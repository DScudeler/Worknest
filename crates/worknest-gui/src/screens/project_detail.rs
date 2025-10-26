//! Project detail screen

use egui::{RichText, ScrollArea};

use worknest_core::models::{Project, ProjectId, Ticket};

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Project detail screen
pub struct ProjectDetailScreen {
    pub project_id: ProjectId,
    project: Option<Project>,
    tickets: Vec<Ticket>,
    is_editing: bool,
    edit_name: String,
    edit_description: String,
    edit_color: String,
    data_loaded: bool,
}

impl ProjectDetailScreen {
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            project: None,
            tickets: Vec::new(),
            is_editing: false,
            edit_name: String::new(),
            edit_description: String::new(),
            edit_color: String::new(),
            data_loaded: false,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !self.data_loaded {
            self.load_data(state);
            self.data_loaded = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(Spacing::LARGE);

                if let Some(project) = self.project.clone() {
                    // Header
                    ui.horizontal(|ui| {
                        if ui.button("← Back").clicked() {
                            state.navigate_to(Screen::ProjectList);
                        }

                        ui.add_space(Spacing::MEDIUM);

                        if !self.is_editing {
                            ui.heading(RichText::new(&project.name).size(28.0));

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Edit").clicked() {
                                        self.start_editing(&project);
                                    }

                                    if project.archived {
                                        if ui.button("Unarchive").clicked()
                                            && false // TODO: API unarchive
                                        {
                                            state.notify_success("Project unarchived".to_string());
                                            self.load_data(state);
                                        }
                                    } else if ui.button("Archive").clicked()
                                        && false // TODO: API archive
                                    {
                                        state.notify_success("Project archived".to_string());
                                        self.load_data(state);
                                    }
                                },
                            );
                        }
                    });

                    ui.add_space(Spacing::LARGE);

                    if self.is_editing {
                        self.render_edit_form(ui, state);
                    } else {
                        // Project info
                        ui.group(|ui| {
                            ui.set_min_width(f32::INFINITY);
                            ui.vertical(|ui| {
                                ui.add_space(Spacing::MEDIUM);

                                if let Some(desc) = &project.description {
                                    ui.label(RichText::new("Description").strong());
                                    ui.label(desc);
                                } else {
                                    ui.label(
                                        RichText::new("No description")
                                            .color(egui::Color32::GRAY)
                                            .italics(),
                                    );
                                }

                                ui.add_space(Spacing::MEDIUM);

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Status:").strong());
                                    if project.archived {
                                        ui.label(
                                            RichText::new("Archived")
                                                .color(egui::Color32::DARK_GRAY),
                                        );
                                    } else {
                                        ui.label(RichText::new("Active").color(Colors::SUCCESS));
                                    }
                                });

                                if let Some(color) = &project.color {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Color:").strong());
                                        ui.label(color);
                                        if let Ok(color_val) = parse_hex_color(color) {
                                            ui.colored_label(color_val, "●");
                                        }
                                    });
                                }

                                ui.add_space(Spacing::MEDIUM);
                            });
                        });

                        ui.add_space(Spacing::XLARGE);

                        // Tickets section
                        ui.horizontal(|ui| {
                            ui.heading("Tickets");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("View Board").clicked() {
                                        state.navigate_to(Screen::TicketBoard {
                                            project_id: self.project_id,
                                        });
                                    }

                                    if ui.button("View All Tickets").clicked() {
                                        state.navigate_to(Screen::TicketList {
                                            project_id: Some(self.project_id),
                                        });
                                    }
                                },
                            );
                        });

                        ui.add_space(Spacing::MEDIUM);

                        // Ticket stats
                        ui.columns(4, |columns| {
                            let open_count = self
                                .tickets
                                .iter()
                                .filter(|t| t.status == worknest_core::models::TicketStatus::Open)
                                .count();
                            let in_progress_count = self
                                .tickets
                                .iter()
                                .filter(|t| {
                                    t.status == worknest_core::models::TicketStatus::InProgress
                                })
                                .count();
                            let review_count = self
                                .tickets
                                .iter()
                                .filter(|t| t.status == worknest_core::models::TicketStatus::Review)
                                .count();
                            let done_count = self
                                .tickets
                                .iter()
                                .filter(|t| {
                                    t.status == worknest_core::models::TicketStatus::Done
                                        || t.status == worknest_core::models::TicketStatus::Closed
                                })
                                .count();

                            columns[0].vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(format!("{}", open_count))
                                        .size(24.0)
                                        .color(Colors::INFO),
                                );
                                ui.label("Open");
                            });

                            columns[1].vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(format!("{}", in_progress_count))
                                        .size(24.0)
                                        .color(Colors::WARNING),
                                );
                                ui.label("In Progress");
                            });

                            columns[2].vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(format!("{}", review_count))
                                        .size(24.0)
                                        .color(Colors::PRIMARY),
                                );
                                ui.label("Review");
                            });

                            columns[3].vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(format!("{}", done_count))
                                        .size(24.0)
                                        .color(Colors::SUCCESS),
                                );
                                ui.label("Done");
                            });
                        });
                    }
                } else {
                    ui.label("Project not found");
                    if ui.button("← Back to Projects").clicked() {
                        state.navigate_to(Screen::ProjectList);
                    }
                }
            });
        });
    }

    fn render_edit_form(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.group(|ui| {
            ui.set_min_width(f32::INFINITY);
            ui.vertical(|ui| {
                ui.label("Project Name");
                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_name).desired_width(f32::INFINITY),
                );

                ui.add_space(Spacing::MEDIUM);

                ui.label("Description");
                ui.add(
                    egui::TextEdit::multiline(&mut self.edit_description)
                        .desired_width(f32::INFINITY)
                        .desired_rows(4),
                );

                ui.add_space(Spacing::MEDIUM);

                ui.label("Color");
                ui.add(egui::TextEdit::singleline(&mut self.edit_color).desired_width(200.0));

                ui.add_space(Spacing::LARGE);

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.is_editing = false;
                    }

                    if ui
                        .add(egui::Button::new("Save").fill(Colors::PRIMARY))
                        .clicked()
                    {
                        self.save_changes(state);
                    }
                });
            });
        });
    }

    fn start_editing(&mut self, project: &Project) {
        self.is_editing = true;
        self.edit_name = project.name.clone();
        self.edit_description = project.description.clone().unwrap_or_default();
        self.edit_color = project.color.clone().unwrap_or_default();
    }

    fn save_changes(&mut self, state: &mut AppState) {
        if let Some(mut project) = self.project.clone() {
            project.name = self.edit_name.clone();
            project.description = if self.edit_description.is_empty() {
                None
            } else {
                Some(self.edit_description.clone())
            };
            project.color = if self.edit_color.is_empty() {
                None
            } else {
                Some(self.edit_color.clone())
            };

            // TODO: API update project
            // match state.api_client.update_project(&project).await {
            //     Ok(_) => {
            //         state.notify_success("Project updated".to_string());
            //         self.is_editing = false;
            //         self.load_data(state);
            //     },
            //     Err(e) => {
            //         state.notify_error(format!("Failed to update project: {:?}", e));
            //     },
            // }
            state.notify_info("API integration in progress".to_string());
        }
    }

    fn load_data(&mut self, _state: &AppState) {
        // TODO: API find project
        self.project = None;

        // TODO: API find tickets
        self.tickets = Vec::new();
    }
}

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
