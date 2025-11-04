//! Ticket board (Kanban) screen

use egui::{RichText, ScrollArea};
use std::sync::Arc;

use worknest_core::models::{Priority, ProjectId, Ticket, TicketId, TicketStatus, TicketType};

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Drag payload for ticket cards
#[derive(Clone, Debug)]
struct TicketDragPayload {
    ticket_id: TicketId,
    source_status: TicketStatus,
}

/// Ticket board screen
pub struct TicketBoardScreen {
    pub project_id: ProjectId,
    tickets: Vec<Ticket>,
    data_loaded: bool,
    // Drag and drop state
    dragging_ticket: Option<TicketId>,
    drag_hover_status: Option<TicketStatus>,
}

impl TicketBoardScreen {
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            tickets: Vec::new(),
            data_loaded: false,
            dragging_ticket: None,
            drag_hover_status: None,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !self.data_loaded {
            self.load_tickets(state);
            self.data_loaded = true;
        }

        // Sync tickets from state
        self.tickets = state
            .tickets
            .iter()
            .filter(|t| t.project_id == self.project_id)
            .cloned()
            .collect();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(Spacing::LARGE);

            // Header
            ui.horizontal(|ui| {
                if ui.button("← Back").clicked() {
                    state.navigate_to(Screen::ProjectDetail(self.project_id));
                }

                ui.add_space(Spacing::MEDIUM);

                ui.heading(RichText::new("Kanban Board").size(28.0));

                ui.add_space(Spacing::MEDIUM);

                if ui.button("List View").clicked() {
                    state.navigate_to(Screen::TicketList {
                        project_id: Some(self.project_id),
                    });
                }
            });

            ui.add_space(Spacing::LARGE);

            // Board columns with horizontal scroll
            ScrollArea::horizontal().show(ui, |ui| {
                // Calculate responsive column width
                let available_width = ui.available_width();
                let column_count = 4.0;
                let spacing = Spacing::MEDIUM * (column_count - 1.0);
                let calculated_width = (available_width - spacing) / column_count;
                let column_width = calculated_width.max(280.0); // Minimum 280px per column

                ui.horizontal_top(|ui| {
                    let columns = [
                        TicketStatus::Open,
                        TicketStatus::InProgress,
                        TicketStatus::Review,
                        TicketStatus::Done,
                    ];

                    for (idx, status) in columns.iter().enumerate() {
                        if idx > 0 {
                            ui.add_space(Spacing::MEDIUM);
                        }
                        self.render_column(ui, *status, state, column_width);
                    }
                });
            });
        });
    }

    fn render_column(
        &mut self,
        ui: &mut egui::Ui,
        status: TicketStatus,
        state: &mut AppState,
        column_width: f32,
    ) {
        let column_tickets: Vec<_> = self
            .tickets
            .iter()
            .filter(|t| t.status == status)
            .cloned()
            .collect();

        let (column_title, column_color) = match status {
            TicketStatus::Open => ("Open", Colors::INFO),
            TicketStatus::InProgress => ("In Progress", Colors::WARNING),
            TicketStatus::Review => ("Review", Colors::PRIMARY),
            TicketStatus::Done => ("Done", Colors::SUCCESS),
            TicketStatus::Closed => ("Closed", egui::Color32::GRAY),
        };

        // Calculate available height for column content
        let available_height = ui.available_height() - 120.0; // Reserve space for header and padding

        ui.vertical(|ui| {
            ui.set_width(column_width);

            // Column background frame
            let bg_color = if ui.style().visuals.dark_mode {
                egui::Color32::from_gray(30)
            } else {
                egui::Color32::from_gray(245)
            };

            // Wrap entire column in drop zone
            let frame = egui::Frame::NONE
                .fill(bg_color)
                .inner_margin(Spacing::MEDIUM)
                .corner_radius(8.0);

            let (drop_response, dropped_payload) =
                ui.dnd_drop_zone::<TicketDragPayload, ()>(frame, |ui| {
                    ui.set_width(column_width - Spacing::MEDIUM * 2.0);

                    // Column header
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(column_title)
                                .strong()
                                .size(16.0)
                                .color(column_color),
                        );
                        ui.label(
                            RichText::new(format!("{} tickets", column_tickets.len()))
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                    });

                    ui.add_space(Spacing::MEDIUM);
                    ui.separator();
                    ui.add_space(Spacing::MEDIUM);

                    // Column content with full height scroll
                    ScrollArea::vertical()
                        .max_height(available_height)
                        .show(ui, |ui| {
                            if column_tickets.is_empty() {
                                // Empty state
                                ui.vertical_centered(|ui| {
                                    ui.add_space(Spacing::XLARGE);
                                    ui.label(
                                        RichText::new("No tickets")
                                            .size(14.0)
                                            .color(egui::Color32::GRAY)
                                            .italics(),
                                    );
                                    ui.add_space(Spacing::XLARGE);
                                });
                            } else {
                                for ticket in &column_tickets {
                                    self.render_draggable_card(ui, ticket, state, status);
                                    ui.add_space(Spacing::MEDIUM);
                                }
                            }
                        });
                });

            // Handle dropped ticket
            if let Some(payload) = dropped_payload {
                self.handle_ticket_drop(payload, status, state);
            }

            // Visual feedback for drag hover
            if drop_response.response.hovered() && self.dragging_ticket.is_some() {
                self.drag_hover_status = Some(status);

                // Draw highlight border around drop zone
                ui.painter().rect_stroke(
                    drop_response.response.rect,
                    8.0,
                    egui::Stroke::new(2.0, column_color.linear_multiply(0.8)),
                    egui::StrokeKind::Outside,
                );
            }
        });
    }

    /// Render a draggable card with drag and drop support
    fn render_draggable_card(
        &mut self,
        ui: &mut egui::Ui,
        ticket: &Ticket,
        state: &mut AppState,
        current_status: TicketStatus,
    ) {
        let drag_id = ui.id().with(("drag_ticket", ticket.id.0));
        let payload = TicketDragPayload {
            ticket_id: ticket.id,
            source_status: current_status,
        };

        let is_being_dragged = self.dragging_ticket == Some(ticket.id);

        // Wrap card in drag source
        let response = ui.dnd_drag_source(drag_id, payload, |ui| {
            // Apply transparency if being dragged
            if is_being_dragged {
                ui.visuals_mut().override_text_color = Some(egui::Color32::from_gray(150));
            }

            self.render_card_ui(ui, ticket, state, is_being_dragged)
        });

        // Track drag state
        if response.response.drag_started() {
            self.dragging_ticket = Some(ticket.id);
        }
        if response.response.drag_stopped() {
            self.dragging_ticket = None;
            self.drag_hover_status = None;
        }

        // Cursor feedback
        if response.response.hovered() && !is_being_dragged {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
        if is_being_dragged {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        }
    }

    /// Render the card UI (extracted for reuse in drag source)
    fn render_card_ui(
        &self,
        ui: &mut egui::Ui,
        ticket: &Ticket,
        state: &mut AppState,
        is_being_dragged: bool,
    ) -> egui::Response {
        let priority_color = match ticket.priority {
            Priority::Low => Colors::PRIORITY_LOW,
            Priority::Medium => Colors::PRIORITY_MEDIUM,
            Priority::High => Colors::PRIORITY_HIGH,
            Priority::Critical => Colors::PRIORITY_CRITICAL,
        };

        let (type_text, type_color) = match ticket.ticket_type {
            TicketType::Task => ("Task", Colors::TYPE_TASK),
            TicketType::Bug => ("Bug", Colors::TYPE_BUG),
            TicketType::Feature => ("Feature", Colors::TYPE_FEATURE),
            TicketType::Epic => ("Epic", Colors::TYPE_EPIC),
        };

        // Card background - adjust for drag state
        let card_bg = if is_being_dragged {
            if ui.style().visuals.dark_mode {
                egui::Color32::from_gray(40).linear_multiply(0.7)
            } else {
                egui::Color32::WHITE.linear_multiply(0.7)
            }
        } else if ui.style().visuals.dark_mode {
            egui::Color32::from_gray(40)
        } else {
            egui::Color32::WHITE
        };

        let frame = egui::Frame::NONE
            .fill(card_bg)
            .inner_margin(Spacing::MEDIUM)
            .outer_margin(0.0)
            .corner_radius(6.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(200)));

        let frame_response = frame.show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(100.0);

            ui.vertical(|ui| {
                // Priority bar on left edge (visual indicator)
                ui.horizontal(|ui| {
                    // Priority indicator
                    ui.colored_label(priority_color, "▊");

                    ui.vertical(|ui| {
                        // Ticket title
                        ui.label(RichText::new(&ticket.title).strong().size(14.0));

                        // Description with proper ellipsis
                        if let Some(desc) = &ticket.description {
                            let preview = if desc.len() > 100 {
                                format!("{}...", &desc[..100])
                            } else {
                                desc.clone()
                            };
                            ui.label(RichText::new(preview).small().color(egui::Color32::GRAY));
                        }
                    });
                });

                ui.add_space(Spacing::MEDIUM);

                // Metadata row
                ui.horizontal(|ui| {
                    // Type badge
                    ui.label(RichText::new(type_text).small().color(type_color));

                    ui.add_space(Spacing::SMALL);

                    // Priority badge
                    let priority_text = match ticket.priority {
                        Priority::Low => "Low",
                        Priority::Medium => "Med",
                        Priority::High => "High",
                        Priority::Critical => "Critical",
                    };
                    ui.label(RichText::new(priority_text).small().color(priority_color));
                });

                ui.add_space(Spacing::SMALL);

                // View button
                if ui.small_button("View Details →").clicked() {
                    state.navigate_to(Screen::TicketDetail(ticket.id));
                }
            });
        });

        frame_response.response
    }

    /// Handle ticket drop - update status via API with optimistic update
    fn handle_ticket_drop(
        &mut self,
        payload: Arc<TicketDragPayload>,
        target_status: TicketStatus,
        state: &mut AppState,
    ) {
        // Don't process if dropped in same column
        if payload.source_status == target_status {
            return;
        }

        let ticket_id = payload.ticket_id;

        tracing::info!(
            "Dropping ticket {:?} from {:?} to {:?}",
            ticket_id,
            payload.source_status,
            target_status
        );

        // Optimistic update in state
        if let Some(ticket) = state.tickets.iter_mut().find(|t| t.id == ticket_id) {
            ticket.status = target_status;
            ticket.updated_at = chrono::Utc::now();
        }

        // Trigger API call
        let api_client = state.api_client.clone();
        let event_queue = state.event_queue.clone();
        let token = match &state.auth_token {
            Some(t) => t.clone(),
            None => {
                tracing::error!("No auth token available for ticket update");
                return;
            },
        };

        // Convert status to string format expected by API
        let status_str = match target_status {
            TicketStatus::Open => "open",
            TicketStatus::InProgress => "in progress", // Backend now accepts both "inprogress" and "in progress"
            TicketStatus::Review => "review",
            TicketStatus::Done => "done",
            TicketStatus::Closed => "closed",
        }
        .to_string();

        wasm_bindgen_futures::spawn_local(async move {
            use crate::api_client::UpdateTicketRequest;
            use crate::events::AppEvent;

            let request = UpdateTicketRequest {
                title: None,
                description: None,
                status: Some(status_str),
                priority: None,
                ticket_type: None,
                assigned_to: None,
            };

            match api_client.update_ticket(&token, ticket_id.0, request).await {
                Ok(updated_ticket) => {
                    tracing::info!(
                        "Ticket status updated successfully: {:?}",
                        updated_ticket.id
                    );
                    event_queue.push(AppEvent::TicketUpdated {
                        ticket: updated_ticket,
                    });
                },
                Err(e) => {
                    tracing::error!("Failed to update ticket status: {:?}", e);
                    // On error, reload the ticket to revert optimistic update
                    event_queue.push(AppEvent::TicketError {
                        message: format!("Failed to move ticket: {}", e),
                    });
                },
            }
        });
    }

    fn load_tickets(&mut self, state: &AppState) {
        if false {
            // Demo mode: Load from in-memory state
            self.tickets = state
                .tickets
                .iter()
                .filter(|t| t.project_id == self.project_id)
                .cloned()
                .collect();
        } else {
            // Integrated mode: Load from API
            let api_client = state.api_client.clone();
            let event_queue = state.event_queue.clone();
            let token = match &state.auth_token {
                Some(t) => t.clone(),
                None => return,
            };
            let project_id_uuid = self.project_id.0;

            wasm_bindgen_futures::spawn_local(async move {
                use crate::events::AppEvent;

                match api_client.get_tickets(&token, Some(project_id_uuid)).await {
                    Ok(tickets) => {
                        tracing::info!("Loaded {} tickets for kanban board", tickets.len());
                        event_queue.push(AppEvent::TicketsLoaded { tickets });
                    },
                    Err(e) => {
                        tracing::error!("Failed to load tickets for board: {:?}", e);
                        event_queue.push(AppEvent::TicketError {
                            message: e.to_string(),
                        });
                    },
                }
            });
        }
    }
}
