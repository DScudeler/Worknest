//! Ticket board (Kanban) screen

use egui::{RichText, ScrollArea};

use worknest_core::models::{Priority, ProjectId, Ticket, TicketStatus, TicketType};

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Ticket board screen
pub struct TicketBoardScreen {
    pub project_id: ProjectId,
    tickets: Vec<Ticket>,
    data_loaded: bool,
}

impl TicketBoardScreen {
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            tickets: Vec::new(),
            data_loaded: false,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !self.data_loaded {
            self.load_tickets(state);
            self.data_loaded = true;
        }

        // Sync tickets from state
        self.tickets = state
            .demo_tickets
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

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("List View").clicked() {
                        state.navigate_to(Screen::TicketList {
                            project_id: Some(self.project_id),
                        });
                    }
                });
            });

            ui.add_space(Spacing::LARGE);

            // Board columns
            ui.horizontal_top(|ui| {
                let columns = vec![
                    TicketStatus::Open,
                    TicketStatus::InProgress,
                    TicketStatus::Review,
                    TicketStatus::Done,
                ];

                for status in columns {
                    self.render_column(ui, status, state);
                }
            });
        });
    }

    fn render_column(&mut self, ui: &mut egui::Ui, status: TicketStatus, state: &mut AppState) {
        let column_tickets: Vec<_> = self.tickets.iter().filter(|t| t.status == status).collect();

        let column_title = match status {
            TicketStatus::Open => "Open",
            TicketStatus::InProgress => "In Progress",
            TicketStatus::Review => "Review",
            TicketStatus::Done => "Done",
            TicketStatus::Closed => "Closed",
        };

        ui.vertical(|ui| {
            ui.set_min_width(250.0);
            ui.set_max_width(300.0);

            // Column header
            ui.group(|ui| {
                ui.set_min_width(f32::INFINITY);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(column_title).strong().size(16.0));
                    ui.label(
                        RichText::new(format!("{} tickets", column_tickets.len()))
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                });
            });

            ui.add_space(Spacing::SMALL);

            // Column content
            ScrollArea::vertical().max_height(600.0).show(ui, |ui| {
                for ticket in column_tickets {
                    self.render_board_card(ui, ticket, state);
                    ui.add_space(Spacing::SMALL);
                }
            });
        });
    }

    fn render_board_card(&self, ui: &mut egui::Ui, ticket: &Ticket, state: &mut AppState) {
        ui.group(|ui| {
            ui.set_min_width(f32::INFINITY);
            ui.vertical(|ui| {
                // Priority indicator
                let priority_color = match ticket.priority {
                    Priority::Low => Colors::PRIORITY_LOW,
                    Priority::Medium => Colors::PRIORITY_MEDIUM,
                    Priority::High => Colors::PRIORITY_HIGH,
                    Priority::Critical => Colors::PRIORITY_CRITICAL,
                };

                ui.horizontal(|ui| {
                    ui.colored_label(priority_color, "●");
                    ui.label(RichText::new(&ticket.title).strong());
                });

                if let Some(desc) = &ticket.description {
                    ui.label(
                        RichText::new(desc.chars().take(80).collect::<String>())
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                }

                ui.add_space(Spacing::SMALL);

                ui.horizontal(|ui| {
                    // Type badge
                    let (type_text, type_color) = match ticket.ticket_type {
                        TicketType::Task => ("Task", Colors::TYPE_TASK),
                        TicketType::Bug => ("Bug", Colors::TYPE_BUG),
                        TicketType::Feature => ("Feature", Colors::TYPE_FEATURE),
                        TicketType::Epic => ("Epic", Colors::TYPE_EPIC),
                    };
                    ui.label(RichText::new(type_text).small().color(type_color));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("View").clicked() {
                            state.navigate_to(Screen::TicketDetail(ticket.id));
                        }
                    });
                });
            });
        });
    }

    fn load_tickets(&mut self, state: &AppState) {
        if state.is_demo_mode() {
            // Demo mode: Load from in-memory state
            self.tickets = state
                .demo_tickets
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
                    }
                    Err(e) => {
                        tracing::error!("Failed to load tickets for board: {:?}", e);
                        event_queue.push(AppEvent::TicketError {
                            message: e.to_string(),
                        });
                    }
                }
            });
        }
    }
}
