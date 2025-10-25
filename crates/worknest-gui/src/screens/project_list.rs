//! Project list screen

use egui::{RichText, ScrollArea};

use worknest_core::models::Project;
use worknest_db::Repository;

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Project list screen
pub struct ProjectListScreen {
    projects: Vec<Project>,
    show_archived: bool,
    search_query: String,
    show_create_dialog: bool,
    new_project_name: String,
    new_project_description: String,
    new_project_color: String,
    data_loaded: bool,
}

impl Default for ProjectListScreen {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
            show_archived: false,
            search_query: String::new(),
            show_create_dialog: false,
            new_project_name: String::new(),
            new_project_description: String::new(),
            new_project_color: String::from("#3B82F6"),
            data_loaded: false,
        }
    }
}

impl ProjectListScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        // Load data on first render
        if !self.data_loaded {
            self.load_projects(state);
            self.data_loaded = true;
        }

        // Show create dialog if open
        if self.show_create_dialog {
            self.render_create_dialog(ctx, state);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(Spacing::LARGE);

            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Projects").size(28.0));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_sized(
                            [120.0, 32.0],
                            egui::Button::new("+ New Project").fill(Colors::PRIMARY),
                        )
                        .clicked()
                    {
                        self.show_create_dialog = true;
                    }

                    // Toggle archived
                    ui.checkbox(&mut self.show_archived, "Show Archived");
                });
            });

            ui.add_space(Spacing::LARGE);

            // Search bar
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search projects...")
                        .desired_width(300.0),
                );
            });

            ui.add_space(Spacing::MEDIUM);
            ui.separator();
            ui.add_space(Spacing::MEDIUM);

            // Project list
            ScrollArea::vertical().show(ui, |ui| {
                let filtered_projects: Vec<Project> = self
                    .projects
                    .iter()
                    .filter(|p| {
                        (self.show_archived || !p.archived)
                            && (self.search_query.is_empty()
                                || p.name
                                    .to_lowercase()
                                    .contains(&self.search_query.to_lowercase())
                                || p.description
                                    .as_ref()
                                    .map(|d| {
                                        d.to_lowercase().contains(&self.search_query.to_lowercase())
                                    })
                                    .unwrap_or(false))
                    })
                    .cloned()
                    .collect();

                if filtered_projects.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(
                            RichText::new("No projects found")
                                .size(16.0)
                                .color(egui::Color32::GRAY),
                        );
                    });
                } else {
                    for project in &filtered_projects {
                        self.render_project_card(ui, project, state);
                        ui.add_space(Spacing::SMALL);
                    }
                }
            });
        });
    }

    fn render_project_card(&mut self, ui: &mut egui::Ui, project: &Project, state: &mut AppState) {
        ui.group(|ui| {
            ui.set_min_size([f32::INFINITY, 80.0].into());
            ui.horizontal(|ui| {
                // Color indicator
                if let Some(color) = &project.color {
                    if let Ok(color_val) = parse_hex_color(color) {
                        ui.colored_label(color_val, RichText::new("●").size(24.0));
                    }
                }

                ui.vertical(|ui| {
                    ui.label(RichText::new(&project.name).size(18.0).strong());
                    if let Some(desc) = &project.description {
                        ui.label(RichText::new(desc).small().color(egui::Color32::GRAY));
                    }

                    if project.archived {
                        ui.label(
                            RichText::new("Archived")
                                .small()
                                .color(egui::Color32::DARK_GRAY),
                        );
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("View").clicked() {
                        state.navigate_to(Screen::ProjectDetail(project.id));
                    }

                    if ui.button("Tickets").clicked() {
                        state.navigate_to(Screen::TicketList {
                            project_id: Some(project.id),
                        });
                    }

                    if ui.button("Board").clicked() {
                        state.navigate_to(Screen::TicketBoard {
                            project_id: project.id,
                        });
                    }

                    if project.archived {
                        if ui.button("Unarchive").clicked()
                            && state.project_repo.unarchive(project.id).is_ok()
                        {
                            state.notify_success("Project unarchived".to_string());
                            self.load_projects(state);
                        }
                    } else if ui.button("Archive").clicked()
                        && state.project_repo.archive(project.id).is_ok()
                    {
                        state.notify_success("Project archived".to_string());
                        self.load_projects(state);
                    }
                });
            });
        });
    }

    fn render_create_dialog(&mut self, ctx: &egui::Context, state: &mut AppState) {
        egui::Window::new("Create New Project")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(400.0);

                ui.label("Project Name");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_project_name)
                        .hint_text("Enter project name")
                        .desired_width(f32::INFINITY),
                );

                ui.add_space(Spacing::MEDIUM);

                ui.label("Description (optional)");
                ui.add(
                    egui::TextEdit::multiline(&mut self.new_project_description)
                        .hint_text("Enter project description")
                        .desired_width(f32::INFINITY)
                        .desired_rows(3),
                );

                ui.add_space(Spacing::MEDIUM);

                ui.label("Color");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_project_color)
                        .hint_text("#3B82F6")
                        .desired_width(f32::INFINITY),
                );

                ui.add_space(Spacing::LARGE);

                ui.horizontal(|ui| {
                    if ui
                        .add_sized([100.0, 32.0], egui::Button::new("Cancel"))
                        .clicked()
                    {
                        self.show_create_dialog = false;
                        self.clear_create_form();
                    }

                    if ui
                        .add_sized(
                            [100.0, 32.0],
                            egui::Button::new("Create").fill(Colors::PRIMARY),
                        )
                        .clicked()
                    {
                        self.create_project(state);
                    }
                });
            });
    }

    fn create_project(&mut self, state: &mut AppState) {
        if self.new_project_name.is_empty() {
            state.notify_error("Project name is required".to_string());
            return;
        }

        if let Some(user) = &state.current_user {
            let mut project = Project::new(self.new_project_name.clone(), user.id);

            if !self.new_project_description.is_empty() {
                project.description = Some(self.new_project_description.clone());
            }

            if !self.new_project_color.is_empty() {
                project.color = Some(self.new_project_color.clone());
            }

            match state.project_repo.create(&project) {
                Ok(_) => {
                    state.notify_success("Project created successfully".to_string());
                    self.show_create_dialog = false;
                    self.clear_create_form();
                    self.load_projects(state);
                },
                Err(e) => {
                    state.notify_error(format!("Failed to create project: {:?}", e));
                },
            }
        }
    }

    fn clear_create_form(&mut self) {
        self.new_project_name.clear();
        self.new_project_description.clear();
        self.new_project_color = String::from("#3B82F6");
    }

    fn load_projects(&mut self, state: &AppState) {
        if let Ok(projects) = state.project_repo.find_all() {
            self.projects = projects;
        }
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
