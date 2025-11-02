//! Project detail screen

use egui::{RichText, ScrollArea};

use worknest_core::models::{Priority, Project, ProjectId, Ticket, TicketType};

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
    // Ticket creation fields
    show_create_ticket_dialog: bool,
    new_ticket_title: String,
    new_ticket_description: String,
    new_ticket_type: TicketType,
    new_ticket_priority: Priority,
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
            // Initialize ticket creation fields
            show_create_ticket_dialog: false,
            new_ticket_title: String::new(),
            new_ticket_description: String::new(),
            new_ticket_type: TicketType::Task,
            new_ticket_priority: Priority::Medium,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !self.data_loaded {
            self.load_data(state);
            self.data_loaded = true;
        }

        // Sync project and tickets from state
        self.project = state
            .projects
            .iter()
            .find(|p| p.id == self.project_id)
            .cloned();

        tracing::debug!(
            "ProjectDetail render: project_id={}, projects_count={}, project_found={}",
            self.project_id.0,
            state.projects.len(),
            self.project.is_some()
        );

        self.tickets = state
            .tickets
            .iter()
            .filter(|t| t.project_id == self.project_id)
            .cloned()
            .collect();

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
                            ui.horizontal(|ui| {
                                ui.heading(RichText::new(&project.name).size(28.0));

                                ui.add_space(Spacing::MEDIUM);

                                if ui.button("Edit").clicked() {
                                    self.start_editing(&project);
                                }

                                // Archive/unarchive functionality is available in project list
                                // TODO: Add archive/unarchive buttons here when API is ready
                            });
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
                    }

                    ui.add_space(Spacing::XLARGE);

                    // Tickets section (always visible, regardless of edit mode)
                    ui.horizontal(|ui| {
                        ui.heading("Tickets");

                        ui.add_space(Spacing::MEDIUM);

                        // "+ New Ticket" button
                        if ui
                            .add_sized(
                                [120.0, 32.0],
                                egui::Button::new("+ New Ticket").fill(Colors::PRIMARY),
                            )
                            .clicked()
                        {
                            self.show_create_ticket_dialog = true;
                        }

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
                    });

                    ui.add_space(Spacing::MEDIUM);

                    // Ticket stats
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

                    ui.horizontal(|ui| {
                        ui.scope(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(format!("{}", open_count))
                                        .size(24.0)
                                        .color(Colors::INFO),
                                );
                                ui.label("Open");
                            });
                        });

                        ui.add_space(Spacing::LARGE);

                        ui.scope(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(format!("{}", in_progress_count))
                                        .size(24.0)
                                        .color(Colors::WARNING),
                                );
                                ui.label("In Progress");
                            });
                        });

                        ui.add_space(Spacing::LARGE);

                        ui.scope(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(format!("{}", review_count))
                                        .size(24.0)
                                        .color(Colors::PRIMARY),
                                );
                                ui.label("Review");
                            });
                        });

                        ui.add_space(Spacing::LARGE);

                        ui.scope(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(format!("{}", done_count))
                                        .size(24.0)
                                        .color(Colors::SUCCESS),
                                );
                                ui.label("Done");
                            });
                        });
                    });

                    ui.add_space(Spacing::LARGE);
                    ui.separator();
                    ui.add_space(Spacing::MEDIUM);

                    // Ticket list
                    if self.tickets.is_empty() {
                        ui.scope(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.add_space(Spacing::XLARGE);
                                ui.label(
                                    RichText::new("📋 No tickets yet")
                                        .size(18.0)
                                        .color(egui::Color32::GRAY),
                                );
                                ui.add_space(Spacing::SMALL);
                                ui.label(
                                    RichText::new("Click '+ New Ticket' above to create your first ticket")
                                        .small()
                                        .color(egui::Color32::GRAY)
                                        .italics(),
                                );
                                ui.add_space(Spacing::XLARGE);
                            });
                        });
                    } else {
                        for ticket in &self.tickets {
                            ui.group(|ui| {
                                ui.set_min_width(f32::INFINITY);
                                ui.horizontal(|ui| {
                                    // Priority indicator
                                    let priority_color = match ticket.priority {
                                        worknest_core::models::Priority::Critical => {
                                            egui::Color32::from_rgb(220, 38, 38)
                                        }
                                        worknest_core::models::Priority::High => {
                                            egui::Color32::from_rgb(234, 88, 12)
                                        }
                                        worknest_core::models::Priority::Medium => {
                                            egui::Color32::from_rgb(59, 130, 246)
                                        }
                                        worknest_core::models::Priority::Low => {
                                            egui::Color32::from_rgb(107, 114, 128)
                                        }
                                    };
                                    ui.colored_label(priority_color, "●");

                                    ui.vertical(|ui| {
                                        // Ticket title
                                        ui.label(RichText::new(&ticket.title).strong());

                                        // Description preview
                                        if let Some(desc) = &ticket.description {
                                            let preview = if desc.len() > 100 {
                                                format!("{}...", &desc[..100])
                                            } else {
                                                desc.clone()
                                            };
                                            ui.label(
                                                RichText::new(preview)
                                                    .small()
                                                    .color(egui::Color32::GRAY),
                                            );
                                        }
                                    });

                                    ui.add_space(Spacing::SMALL);

                                    // Type badge
                                    let type_icon = match ticket.ticket_type {
                                        worknest_core::models::TicketType::Bug => "🐛",
                                        worknest_core::models::TicketType::Feature => "✨",
                                        worknest_core::models::TicketType::Task => "📝",
                                        _ => "📋",
                                    };
                                    ui.label(
                                        RichText::new(type_icon)
                                            .small()
                                            .color(egui::Color32::GRAY),
                                    );

                                    // Status badge
                                    let (status_text, status_color) = match ticket.status {
                                        worknest_core::models::TicketStatus::Open => {
                                            ("Open", Colors::INFO)
                                        }
                                        worknest_core::models::TicketStatus::InProgress => {
                                            ("In Progress", Colors::WARNING)
                                        }
                                        worknest_core::models::TicketStatus::Review => {
                                            ("Review", Colors::PRIMARY)
                                        }
                                        worknest_core::models::TicketStatus::Done => {
                                            ("Done", Colors::SUCCESS)
                                        }
                                        worknest_core::models::TicketStatus::Closed => {
                                            ("Closed", egui::Color32::DARK_GRAY)
                                        }
                                    };
                                    ui.label(
                                        RichText::new(status_text)
                                            .small()
                                            .color(status_color),
                                    );

                                    ui.add_space(Spacing::SMALL);

                                    // View button
                                    if ui.small_button("View →").clicked() {
                                        state.navigate_to(Screen::TicketDetail(ticket.id));
                                    }
                                });
                            });
                            ui.add_space(Spacing::SMALL);
                        }
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(Spacing::XLARGE);
                        ui.label(RichText::new("Loading project...").size(18.0));
                        ui.add_space(Spacing::SMALL);
                        ui.label(
                            RichText::new(format!(
                                "Project ID: {} | Projects in state: {}",
                                self.project_id.0,
                                state.projects.len()
                            ))
                            .small()
                            .color(egui::Color32::GRAY),
                        );
                        ui.add_space(Spacing::MEDIUM);
                        if ui.button("← Back to Projects").clicked() {
                            state.navigate_to(Screen::ProjectList);
                        }
                        ui.add_space(Spacing::XLARGE);
                    });
                }
            });
        });

        // Render create ticket dialog after CentralPanel (so it appears on top)
        if self.show_create_ticket_dialog {
            self.render_create_ticket_dialog(ctx, state);
        }
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
        if let Some(project) = self.project.clone() {
            let name = self.edit_name.clone();
            let description = if self.edit_description.is_empty() {
                None
            } else {
                Some(self.edit_description.clone())
            };
            // Note: Color is not currently supported by the backend API
            // let _color = if self.edit_color.is_empty() {
            //     None
            // } else {
            //     Some(self.edit_color.clone())
            // };

            // Call API to update project
            let api_client = state.api_client.clone();
            let event_queue = state.event_queue.clone();

            if let Some(token) = &state.auth_token {
                let token = token.clone();
                let project_id = project.id.0;

                // Close edit mode immediately
                self.is_editing = false;
                state.is_loading = true;

                wasm_bindgen_futures::spawn_local(async move {
                    use crate::api_client::UpdateProjectRequest;
                    use crate::events::AppEvent;

                    let request = UpdateProjectRequest {
                        name: Some(name),
                        description,
                        is_archived: None,
                    };

                    match api_client.update_project(&token, project_id, request).await {
                        Ok(updated_project) => {
                            tracing::info!("Project updated: {}", updated_project.name);
                            event_queue.push(AppEvent::ProjectUpdated {
                                project: updated_project,
                            });
                        }
                        Err(e) => {
                            tracing::error!("Failed to update project: {:?}", e);
                            event_queue.push(AppEvent::ProjectError {
                                message: e.to_string(),
                            });
                        }
                    }
                });
            }
        }
    }

    fn load_data(&mut self, state: &AppState) {
        // Load from API
        let api_client = state.api_client.clone();
        let event_queue = state.event_queue.clone();
        let token = match &state.auth_token {
            Some(t) => t.clone(),
            None => return,
        };
        let project_id_uuid = self.project_id.0;

        // Load project
        wasm_bindgen_futures::spawn_local(async move {
            use crate::events::AppEvent;

            match api_client.get_project(&token, project_id_uuid).await {
                Ok(project) => {
                    tracing::info!("Loaded project: {}", project.name);
                    event_queue.push(AppEvent::ProjectLoaded { project });
                }
                Err(e) => {
                    tracing::error!("Failed to load project: {:?}", e);
                    event_queue.push(AppEvent::ProjectError {
                        message: e.to_string(),
                    });
                }
            }
        });

        // Load tickets for this project
        let api_client = state.api_client.clone();
        let event_queue = state.event_queue.clone();
        let token = match &state.auth_token {
            Some(t) => t.clone(),
            None => return,
        };

        wasm_bindgen_futures::spawn_local(async move {
            use crate::events::AppEvent;

            match api_client.get_tickets(&token, Some(project_id_uuid)).await {
                Ok(tickets) => {
                    tracing::info!("Loaded {} tickets for project", tickets.len());
                    event_queue.push(AppEvent::TicketsLoaded { tickets });
                }
                Err(e) => {
                    tracing::error!("Failed to load tickets: {:?}", e);
                    event_queue.push(AppEvent::TicketError {
                        message: e.to_string(),
                    });
                }
            }
        });
    }

    fn render_create_ticket_dialog(&mut self, ctx: &egui::Context, state: &mut AppState) {
        egui::Window::new("Create New Ticket")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(400.0);

                ui.label("Ticket Title");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_ticket_title)
                        .hint_text("Enter ticket title")
                        .desired_width(f32::INFINITY),
                );

                ui.add_space(Spacing::MEDIUM);

                ui.label("Description");
                ui.add(
                    egui::TextEdit::multiline(&mut self.new_ticket_description)
                        .hint_text("Enter ticket description")
                        .desired_width(f32::INFINITY)
                        .desired_rows(4),
                );

                ui.add_space(Spacing::MEDIUM);

                ui.label("Type");
                egui::ComboBox::from_id_salt("ticket_type")
                    .selected_text(format!("{:?}", self.new_ticket_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.new_ticket_type, TicketType::Task, "Task");
                        ui.selectable_value(&mut self.new_ticket_type, TicketType::Bug, "Bug");
                        ui.selectable_value(
                            &mut self.new_ticket_type,
                            TicketType::Feature,
                            "Feature",
                        );
                    });

                ui.add_space(Spacing::MEDIUM);

                ui.label("Priority");
                egui::ComboBox::from_id_salt("ticket_priority")
                    .selected_text(format!("{:?}", self.new_ticket_priority))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.new_ticket_priority, Priority::Low, "Low");
                        ui.selectable_value(
                            &mut self.new_ticket_priority,
                            Priority::Medium,
                            "Medium",
                        );
                        ui.selectable_value(&mut self.new_ticket_priority, Priority::High, "High");
                        ui.selectable_value(
                            &mut self.new_ticket_priority,
                            Priority::Critical,
                            "Critical",
                        );
                    });

                ui.add_space(Spacing::LARGE);

                ui.horizontal(|ui| {
                    if ui
                        .add_sized([100.0, 32.0], egui::Button::new("Cancel"))
                        .clicked()
                    {
                        self.show_create_ticket_dialog = false;
                        self.clear_create_ticket_form();
                    }

                    if ui
                        .add_sized(
                            [100.0, 32.0],
                            egui::Button::new("Create").fill(Colors::PRIMARY),
                        )
                        .clicked()
                    {
                        self.create_ticket(state);
                    }
                });
            });
    }

    fn create_ticket(&mut self, state: &mut AppState) {
        if self.new_ticket_title.is_empty() {
            state.notify_error("Ticket title is required".to_string());
            return;
        }

        if let Some(_user) = &state.current_user {
            // Call API to create ticket
            let api_client = state.api_client.clone();
            let event_queue = state.event_queue.clone();

            if let Some(token) = &state.auth_token {
                let token = token.clone();
                let title = self.new_ticket_title.clone();
                let description = if self.new_ticket_description.is_empty() {
                    None
                } else {
                    Some(self.new_ticket_description.clone())
                };
                let ticket_type = format!("{:?}", self.new_ticket_type);
                let priority = format!("{:?}", self.new_ticket_priority);
                let project_id = self.project_id.0;

                // Close dialog and clear form immediately
                self.show_create_ticket_dialog = false;
                self.clear_create_ticket_form();
                state.is_loading = true;

                wasm_bindgen_futures::spawn_local(async move {
                    use crate::api_client::CreateTicketRequest;
                    use crate::events::AppEvent;

                    let request = CreateTicketRequest {
                        project_id,
                        title,
                        description,
                        ticket_type,
                        priority,
                    };

                    match api_client.create_ticket(&token, request).await {
                        Ok(created_ticket) => {
                            tracing::info!("Ticket created: {}", created_ticket.title);
                            event_queue.push(AppEvent::TicketCreated {
                                ticket: created_ticket,
                            });
                        }
                        Err(e) => {
                            tracing::error!("Failed to create ticket: {:?}", e);
                            event_queue.push(AppEvent::TicketError {
                                message: e.to_string(),
                            });
                        }
                    }
                });
            }
        }
    }

    fn clear_create_ticket_form(&mut self) {
        self.new_ticket_title.clear();
        self.new_ticket_description.clear();
        self.new_ticket_type = TicketType::Task;
        self.new_ticket_priority = Priority::Medium;
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
