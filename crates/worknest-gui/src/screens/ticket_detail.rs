//! Ticket detail screen

use egui::{RichText, ScrollArea};

use worknest_core::models::{Comment, CommentId, Priority, Ticket, TicketId, TicketStatus, TicketType};

use crate::{
    api_client::{CreateCommentRequest, UpdateCommentRequest},
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
    // Comment fields
    new_comment_content: String,
    editing_comment_id: Option<CommentId>,
    edit_comment_content: String,
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
            new_comment_content: String::new(),
            editing_comment_id: None,
            edit_comment_content: String::new(),
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !self.data_loaded {
            self.load_data(state);
            self.data_loaded = true;
        }

        // Sync ticket from state
        if self.ticket.is_none() || !false {
            self.ticket = state
                .tickets
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
                            ui.horizontal(|ui| {
                                ui.heading(RichText::new(&ticket.title).size(24.0));

                                ui.add_space(Spacing::MEDIUM);

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

                        // Comments section
                        ui.add_space(Spacing::XLARGE);
                        self.render_comments_section(ui, state, &ticket);
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

            if false {
                // Demo mode: Update in-memory state
                if let Some(t) = state.tickets.iter_mut().find(|t| t.id == ticket.id) {
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
        if false {
            // Demo mode: Update in-memory state
            if let Some(ticket) = state.tickets.iter_mut().find(|t| t.id == self.ticket_id) {
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
        if false {
            // Demo mode: Remove from in-memory state
            state.tickets.retain(|t| t.id != ticket.id);
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
        if false {
            // Demo mode: Load from in-memory state
            self.ticket = state
                .tickets
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

    fn render_comments_section(&mut self, ui: &mut egui::Ui, state: &mut AppState, ticket: &Ticket) {
        ui.heading("Comments");
        ui.add_space(Spacing::MEDIUM);

        // Get comments for this ticket
        let comments: Vec<Comment> = state
            .comments
            .iter()
            .filter(|c| c.ticket_id == ticket.id)
            .cloned()
            .collect();

        // Display existing comments
        if comments.is_empty() {
            ui.label(RichText::new("No comments yet").color(egui::Color32::GRAY).italics());
        } else {
            for comment in comments.iter() {
                self.render_comment(ui, state, comment);
                ui.add_space(Spacing::MEDIUM);
            }
        }

        ui.add_space(Spacing::LARGE);

        // New comment form
        ui.group(|ui| {
            ui.set_min_width(f32::INFINITY);
            ui.vertical(|ui| {
                ui.label(RichText::new("Add a comment").strong());
                ui.add_space(Spacing::SMALL);

                ui.add(
                    egui::TextEdit::multiline(&mut self.new_comment_content)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3)
                        .hint_text("Write your comment here..."),
                );

                ui.add_space(Spacing::SMALL);

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !self.new_comment_content.trim().is_empty(),
                            egui::Button::new("Post Comment"),
                        )
                        .clicked()
                    {
                        self.create_comment(state, ticket);
                    }

                    if ui.button("Clear").clicked() {
                        self.new_comment_content.clear();
                    }
                });
            });
        });
    }

    fn render_comment(&mut self, ui: &mut egui::Ui, state: &mut AppState, comment: &Comment) {
        ui.group(|ui| {
            ui.set_min_width(f32::INFINITY);
            ui.vertical(|ui| {
                // Comment header
                ui.horizontal(|ui| {
                    // Find username (in demo mode, use user_id as fallback)
                    let username = if let Some(user) = state.current_user.as_ref() {
                        if user.id == comment.user_id {
                            user.username.clone()
                        } else {
                            format!("User {}", comment.user_id)
                        }
                    } else {
                        format!("User {}", comment.user_id)
                    };

                    ui.label(RichText::new(username).strong().color(Colors::PRIMARY));
                    ui.separator();
                    ui.label(RichText::new(comment.created_at.format("%Y-%m-%d %H:%M").to_string())
                        .color(egui::Color32::GRAY));

                    if comment.created_at != comment.updated_at {
                        ui.label(RichText::new("(edited)").italics().color(egui::Color32::GRAY));
                    }

                    // Edit/Delete buttons (only if user owns the comment)
                    if let Some(user) = state.current_user.as_ref() {
                        if user.id == comment.user_id {
                            ui.add_space(Spacing::SMALL);

                            if self.editing_comment_id == Some(comment.id) {
                                if ui.small_button("Cancel").clicked() {
                                    self.editing_comment_id = None;
                                    self.edit_comment_content.clear();
                                }
                            } else if ui.small_button("Edit").clicked() {
                                self.editing_comment_id = Some(comment.id);
                                self.edit_comment_content = comment.content.clone();
                            }

                            if ui
                                .add(egui::Button::new("Delete").fill(Colors::ERROR).small())
                                .clicked()
                            {
                                self.delete_comment(state, comment.id);
                            }
                        }
                    }
                });

                ui.add_space(Spacing::SMALL);

                // Comment content (editable if in edit mode)
                if self.editing_comment_id == Some(comment.id) {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.edit_comment_content)
                            .desired_width(f32::INFINITY)
                            .desired_rows(3),
                    );

                    ui.add_space(Spacing::SMALL);

                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !self.edit_comment_content.trim().is_empty(),
                                egui::Button::new("Save"),
                            )
                            .clicked()
                        {
                            self.update_comment(state, comment.id);
                        }
                    });
                } else {
                    ui.label(&comment.content);
                }
            });
        });
    }

    fn create_comment(&mut self, state: &mut AppState, ticket: &Ticket) {
        let content = self.new_comment_content.trim().to_string();

        if content.is_empty() {
            return;
        }

        if false {
            // Demo mode: Create comment in memory
            if let Some(user) = &state.current_user {
                let comment = Comment::new(ticket.id, user.id, content);

                // Validate comment
                if let Err(e) = comment.validate() {
                    state.notify_error(format!("Invalid comment: {}", e));
                    return;
                }

                state.comments.push(comment);
                state.notify_success("Comment added (Demo Mode)".to_string());
                self.new_comment_content.clear();
            } else {
                state.notify_error("Not authenticated".to_string());
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
            let request = CreateCommentRequest { content };

            state.is_loading = true;

            wasm_bindgen_futures::spawn_local(async move {
                use crate::events::AppEvent;

                match api_client.create_comment(&token, ticket_id_uuid, request).await {
                    Ok(comment) => {
                        tracing::info!("Comment created successfully");
                        event_queue.push(AppEvent::CommentCreated { comment });
                    }
                    Err(e) => {
                        tracing::error!("Failed to create comment: {:?}", e);
                        event_queue.push(AppEvent::CommentError {
                            message: e.to_string(),
                        });
                    }
                }
            });

            self.new_comment_content.clear();
        }
    }

    fn update_comment(&mut self, state: &mut AppState, comment_id: CommentId) {
        let content = self.edit_comment_content.trim().to_string();

        if content.is_empty() {
            return;
        }

        if false {
            // Demo mode: Update comment in memory
            if let Some(comment) = state.comments.iter_mut().find(|c| c.id == comment_id) {
                if let Err(e) = comment.update_content(content) {
                    state.notify_error(format!("Invalid comment: {}", e));
                    return;
                }
                state.notify_success("Comment updated (Demo Mode)".to_string());
                self.editing_comment_id = None;
                self.edit_comment_content.clear();
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

            let comment_id_uuid = comment_id.0;
            let request = UpdateCommentRequest { content };

            state.is_loading = true;

            wasm_bindgen_futures::spawn_local(async move {
                use crate::events::AppEvent;

                match api_client.update_comment(&token, comment_id_uuid, request).await {
                    Ok(comment) => {
                        tracing::info!("Comment updated successfully");
                        event_queue.push(AppEvent::CommentUpdated { comment });
                    }
                    Err(e) => {
                        tracing::error!("Failed to update comment: {:?}", e);
                        event_queue.push(AppEvent::CommentError {
                            message: e.to_string(),
                        });
                    }
                }
            });

            self.editing_comment_id = None;
            self.edit_comment_content.clear();
        }
    }

    fn delete_comment(&mut self, state: &mut AppState, comment_id: CommentId) {
        if false {
            // Demo mode: Delete from memory
            state.comments.retain(|c| c.id != comment_id);
            state.notify_success("Comment deleted (Demo Mode)".to_string());
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

            let comment_id_uuid = comment_id.0;
            let comment_id_string = comment_id.to_string();

            state.is_loading = true;

            wasm_bindgen_futures::spawn_local(async move {
                use crate::events::AppEvent;

                match api_client.delete_comment(&token, comment_id_uuid).await {
                    Ok(_) => {
                        tracing::info!("Comment deleted successfully");
                        event_queue.push(AppEvent::CommentDeleted {
                            comment_id: comment_id_string,
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete comment: {:?}", e);
                        event_queue.push(AppEvent::CommentError {
                            message: e.to_string(),
                        });
                    }
                }
            });
        }
    }
}
