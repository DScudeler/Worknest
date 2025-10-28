//! Ticket detail screen

use egui::{RichText, ScrollArea};

use worknest_core::models::{Priority, Ticket, TicketId, TicketStatus, TicketType};

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Ticket detail screen
pub struct TicketDetailScreen {
    pub ticket_id: TicketId,
    ticket: Option<Ticket>,
    is_editing: bool,
    edit_title: String,
    edit_description: String,
    edit_type: TicketType,
    edit_status: TicketStatus,
    edit_priority: Priority,
    data_loaded: bool,
}

impl TicketDetailScreen {
    pub fn new(ticket_id: TicketId) -> Self {
        Self {
            ticket_id,
            ticket: None,
            is_editing: false,
            edit_title: String::new(),
            edit_description: String::new(),
            edit_type: TicketType::Task,
            edit_status: TicketStatus::Open,
            edit_priority: Priority::Medium,
            data_loaded: false,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !self.data_loaded {
            self.load_data(state);
            self.data_loaded = true;
        }

        // Sync ticket from state
        if self.ticket.is_none() || !state.is_demo_mode() {
            self.ticket = state
                .demo_tickets
                .iter()
                .find(|t| t.id == self.ticket_id)
                .cloned();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(Spacing::LARGE);

                if let Some(ticket) = self.ticket.clone() {
                    // Header
                    ui.horizontal(|ui| {
                        if ui.button("← Back").clicked() {
                            state.navigate_to(Screen::TicketList {
                                project_id: Some(ticket.project_id),
                            });
                        }

                        ui.add_space(Spacing::MEDIUM);

                        if !self.is_editing {
                            ui.heading(RichText::new(&ticket.title).size(24.0));

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Edit").clicked() {
                                        self.start_editing(&ticket);
                                    }

                                    if ui
                                        .add(egui::Button::new("Delete").fill(Colors::ERROR))
                                        .clicked()
                                    {
                                        self.delete_ticket(state, &ticket);
                                    }
                                },
                            );
                        }
                    });

                    ui.add_space(Spacing::LARGE);

                    if self.is_editing {
                        self.render_edit_form(ui, state);
                    } else {
                        // Ticket info
                        ui.group(|ui| {
                            ui.set_min_width(f32::INFINITY);
                            ui.vertical(|ui| {
                                ui.add_space(Spacing::MEDIUM);

                                // Type, Status, Priority
                                ui.horizontal(|ui| {
                                    let (type_text, type_color) = match ticket.ticket_type {
                                        TicketType::Task => ("Task", Colors::TYPE_TASK),
                                        TicketType::Bug => ("Bug", Colors::TYPE_BUG),
                                        TicketType::Feature => ("Feature", Colors::TYPE_FEATURE),
                                        TicketType::Epic => ("Epic", Colors::TYPE_EPIC),
                                    };
                                    ui.label(RichText::new(type_text).color(type_color));

                                    ui.separator();

                                    let status_text = match ticket.status {
                                        TicketStatus::Open => "Open",
                                        TicketStatus::InProgress => "In Progress",
                                        TicketStatus::Review => "Review",
                                        TicketStatus::Done => "Done",
                                        TicketStatus::Closed => "Closed",
                                    };
                                    ui.label(status_text);

                                    ui.separator();

                                    let (priority_text, priority_color) = match ticket.priority {
                                        Priority::Low => ("Low Priority", Colors::PRIORITY_LOW),
                                        Priority::Medium => {
                                            ("Medium Priority", Colors::PRIORITY_MEDIUM)
                                        },
                                        Priority::High => ("High Priority", Colors::PRIORITY_HIGH),
                                        Priority::Critical => {
                                            ("Critical Priority", Colors::PRIORITY_CRITICAL)
                                        },
                                    };
                                    ui.colored_label(priority_color, priority_text);
                                });

                                ui.add_space(Spacing::LARGE);

                                // Description
                                ui.label(RichText::new("Description").strong().size(16.0));
                                if let Some(desc) = &ticket.description {
                                    ui.label(desc);
                                } else {
                                    ui.label(
                                        RichText::new("No description")
                                            .color(egui::Color32::GRAY)
                                            .italics(),
                                    );
                                }

                                ui.add_space(Spacing::LARGE);

                                // Additional info
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Created:").strong());
                                    ui.label(
                                        ticket.created_at.format("%Y-%m-%d %H:%M").to_string(),
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Updated:").strong());
                                    ui.label(
                                        ticket.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                                    );
                                });

                                if let Some(due_date) = ticket.due_date {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Due Date:").strong());
                                        ui.label(due_date.format("%Y-%m-%d").to_string());
                                    });
                                }

                                if let Some(estimate) = ticket.estimate_hours {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Estimate:").strong());
                                        ui.label(format!("{} hours", estimate));
                                    });
                                }

                                ui.add_space(Spacing::MEDIUM);
                            });
                        });

                        ui.add_space(Spacing::XLARGE);

                        // Quick status update
                        ui.heading("Quick Actions");
                        ui.add_space(Spacing::MEDIUM);

                        ui.horizontal(|ui| {
                            if ticket.status != TicketStatus::Open
                                && ui.button("Mark as Open").clicked()
                            {
                                self.update_status(state, TicketStatus::Open);
                            }
                            if ticket.status != TicketStatus::InProgress
                                && ui.button("Start Progress").clicked()
                            {
                                self.update_status(state, TicketStatus::InProgress);
                            }
                            if ticket.status != TicketStatus::Review
                                && ui.button("Move to Review").clicked()
                            {
                                self.update_status(state, TicketStatus::Review);
                            }
                            if ticket.status != TicketStatus::Done
                                && ui.button("Mark as Done").clicked()
                            {
                                self.update_status(state, TicketStatus::Done);
                            }
                        });
                    }
                } else {
                    ui.label("Ticket not found");
                    if ui.button("← Back").clicked() {
                        state.navigate_to(Screen::Dashboard);
                    }
                }
            });
        });
    }

    fn render_edit_form(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.group(|ui| {
            ui.set_min_width(f32::INFINITY);
            ui.vertical(|ui| {
                ui.label("Title");
                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_title).desired_width(f32::INFINITY),
                );

                ui.add_space(Spacing::MEDIUM);

                ui.label("Description");
                ui.add(
                    egui::TextEdit::multiline(&mut self.edit_description)
                        .desired_width(f32::INFINITY)
                        .desired_rows(6),
                );

                ui.add_space(Spacing::MEDIUM);

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.radio_value(&mut self.edit_type, TicketType::Task, "Task");
                    ui.radio_value(&mut self.edit_type, TicketType::Bug, "Bug");
                    ui.radio_value(&mut self.edit_type, TicketType::Feature, "Feature");
                    ui.radio_value(&mut self.edit_type, TicketType::Epic, "Epic");
                });

                ui.add_space(Spacing::MEDIUM);

                ui.horizontal(|ui| {
                    ui.label("Status:");
                    ui.radio_value(&mut self.edit_status, TicketStatus::Open, "Open");
                    ui.radio_value(
                        &mut self.edit_status,
                        TicketStatus::InProgress,
                        "In Progress",
                    );
                    ui.radio_value(&mut self.edit_status, TicketStatus::Review, "Review");
                    ui.radio_value(&mut self.edit_status, TicketStatus::Done, "Done");
                    ui.radio_value(&mut self.edit_status, TicketStatus::Closed, "Closed");
                });

                ui.add_space(Spacing::MEDIUM);

                ui.horizontal(|ui| {
                    ui.label("Priority:");
                    ui.radio_value(&mut self.edit_priority, Priority::Low, "Low");
                    ui.radio_value(&mut self.edit_priority, Priority::Medium, "Medium");
                    ui.radio_value(&mut self.edit_priority, Priority::High, "High");
                    ui.radio_value(&mut self.edit_priority, Priority::Critical, "Critical");
                });

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

    fn start_editing(&mut self, ticket: &Ticket) {
        self.is_editing = true;
        self.edit_title = ticket.title.clone();
        self.edit_description = ticket.description.clone().unwrap_or_default();
        self.edit_type = ticket.ticket_type;
        self.edit_status = ticket.status;
        self.edit_priority = ticket.priority;
    }

    fn save_changes(&mut self, state: &mut AppState) {
        if let Some(mut ticket) = self.ticket.clone() {
            ticket.title = self.edit_title.clone();
            ticket.description = if self.edit_description.is_empty() {
                None
            } else {
                Some(self.edit_description.clone())
            };
            ticket.ticket_type = self.edit_type;
            ticket.status = self.edit_status;
            ticket.priority = self.edit_priority;

            if state.is_demo_mode() {
                // Demo mode: Update in-memory state
                if let Some(t) = state.demo_tickets.iter_mut().find(|t| t.id == ticket.id) {
                    *t = ticket;
                    state.notify_success("Ticket updated (Demo Mode)".to_string());
                    self.is_editing = false;
                    self.load_data(state);
                }
            } else {
                // Integrated mode: Call real API
                let api_client = state.api_client.clone();
                let event_queue = state.event_queue.clone();
                let token = match &state.auth_token {
                    Some(t) => t.clone(),
                    None => {
                        state.notify_error("Not authenticated".to_string());
                        return;
                    }
                };

                let ticket_id_uuid = ticket.id.0;
                let title = self.edit_title.clone();
                let description = if self.edit_description.is_empty() {
                    None
                } else {
                    Some(self.edit_description.clone())
                };
                let status = self.edit_status.to_string().to_lowercase();
                let priority = self.edit_priority.to_string().to_lowercase();
                let ticket_type = self.edit_type.to_string().to_lowercase();

                self.is_editing = false;
                state.is_loading = true;

                wasm_bindgen_futures::spawn_local(async move {
                    use crate::api_client::UpdateTicketRequest;
                    use crate::events::AppEvent;

                    let request = UpdateTicketRequest {
                        title: Some(title),
                        description,
                        status: Some(status),
                        priority: Some(priority),
                        ticket_type: Some(ticket_type),
                        assigned_to: None,
                    };

                    match api_client.update_ticket(&token, ticket_id_uuid, request).await {
                        Ok(updated_ticket) => {
                            tracing::info!("Ticket updated successfully: {}", updated_ticket.title);
                            event_queue.push(AppEvent::TicketUpdated {
                                ticket: updated_ticket,
                            });
                        }
                        Err(e) => {
                            tracing::error!("Failed to update ticket: {:?}", e);
                            event_queue.push(AppEvent::TicketError {
                                message: e.to_string(),
                            });
                        }
                    }
                });
            }
        }
    }

    fn update_status(&mut self, state: &mut AppState, new_status: TicketStatus) {
        if state.is_demo_mode() {
            // Demo mode: Update in-memory state
            if let Some(ticket) = state.demo_tickets.iter_mut().find(|t| t.id == self.ticket_id) {
                ticket.status = new_status;
                state.notify_success(format!("Ticket status updated to {:?} (Demo Mode)", new_status));
                self.load_data(state);
            }
        } else {
            // Integrated mode: Call real API
            let api_client = state.api_client.clone();
            let event_queue = state.event_queue.clone();
            let token = match &state.auth_token {
                Some(t) => t.clone(),
                None => {
                    state.notify_error("Not authenticated".to_string());
                    return;
                }
            };

            let ticket_id_uuid = self.ticket_id.0;
            let status_string = new_status.to_string().to_lowercase();

            state.is_loading = true;

            wasm_bindgen_futures::spawn_local(async move {
                use crate::api_client::UpdateTicketRequest;
                use crate::events::AppEvent;

                let request = UpdateTicketRequest {
                    title: None,
                    description: None,
                    status: Some(status_string),
                    priority: None,
                    ticket_type: None,
                    assigned_to: None,
                };

                match api_client.update_ticket(&token, ticket_id_uuid, request).await {
                    Ok(updated_ticket) => {
                        tracing::info!("Ticket status updated to: {:?}", updated_ticket.status);
                        event_queue.push(AppEvent::TicketUpdated {
                            ticket: updated_ticket,
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to update ticket status: {:?}", e);
                        event_queue.push(AppEvent::TicketError {
                            message: e.to_string(),
                        });
                    }
                }
            });
        }
    }

    fn delete_ticket(&mut self, state: &mut AppState, ticket: &Ticket) {
        if state.is_demo_mode() {
            // Demo mode: Remove from in-memory state
            state.demo_tickets.retain(|t| t.id != ticket.id);
            state.notify_success("Ticket deleted (Demo Mode)".to_string());
            state.navigate_to(Screen::TicketList {
                project_id: Some(ticket.project_id),
            });
        } else {
            // Integrated mode: Call real API
            let api_client = state.api_client.clone();
            let event_queue = state.event_queue.clone();
            let token = match &state.auth_token {
                Some(t) => t.clone(),
                None => {
                    state.notify_error("Not authenticated".to_string());
                    return;
                }
            };

            let ticket_id_uuid = ticket.id.0;
            let ticket_id_string = ticket.id.to_string();
            let project_id = ticket.project_id;

            state.is_loading = true;

            wasm_bindgen_futures::spawn_local(async move {
                use crate::events::AppEvent;

                match api_client.delete_ticket(&token, ticket_id_uuid).await {
                    Ok(_) => {
                        tracing::info!("Ticket deleted successfully");
                        event_queue.push(AppEvent::TicketDeleted {
                            ticket_id: ticket_id_string,
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete ticket: {:?}", e);
                        event_queue.push(AppEvent::TicketError {
                            message: e.to_string(),
                        });
                    }
                }
            });

            // Navigate back to list immediately
            state.navigate_to(Screen::TicketList {
                project_id: Some(project_id),
            });
        }
    }

    fn load_data(&mut self, state: &AppState) {
        if state.is_demo_mode() {
            // Demo mode: Load from in-memory state
            self.ticket = state
                .demo_tickets
                .iter()
                .find(|t| t.id == self.ticket_id)
                .cloned();
        } else {
            // Integrated mode: Load from API
            let api_client = state.api_client.clone();
            let event_queue = state.event_queue.clone();
            let token = match &state.auth_token {
                Some(t) => t.clone(),
                None => return,
            };
            let ticket_id_uuid = self.ticket_id.0;

            wasm_bindgen_futures::spawn_local(async move {
                use crate::events::AppEvent;

                match api_client.get_ticket(&token, ticket_id_uuid).await {
                    Ok(ticket) => {
                        tracing::info!("Loaded ticket: {}", ticket.title);
                        event_queue.push(AppEvent::TicketLoaded { ticket });
                    }
                    Err(e) => {
                        tracing::error!("Failed to load ticket: {:?}", e);
                        event_queue.push(AppEvent::TicketError {
                            message: e.to_string(),
                        });
                    }
                }
            });
        }
    }
}
