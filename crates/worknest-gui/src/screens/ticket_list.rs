//! Ticket list screen

use egui::{RichText, ScrollArea};

use worknest_core::models::{Priority, ProjectId, Ticket, TicketStatus, TicketType};

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Ticket list screen
pub struct TicketListScreen {
    pub project_id: Option<ProjectId>,
    tickets: Vec<Ticket>,
    filter_status: Option<TicketStatus>,
    search_query: String,
    show_create_dialog: bool,
    new_ticket_title: String,
    new_ticket_description: String,
    new_ticket_type: TicketType,
    new_ticket_priority: Priority,
    data_loaded: bool,
}

impl TicketListScreen {
    pub fn new(project_id: Option<ProjectId>) -> Self {
        Self {
            project_id,
            tickets: Vec::new(),
            filter_status: None,
            search_query: String::new(),
            show_create_dialog: false,
            new_ticket_title: String::new(),
            new_ticket_description: String::new(),
            new_ticket_type: TicketType::Task,
            new_ticket_priority: Priority::Medium,
            data_loaded: false,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut AppState) {
        if !self.data_loaded {
            self.load_tickets(state);
            self.data_loaded = true;
        }

        if self.show_create_dialog {
            self.render_create_dialog(ctx, state);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(Spacing::LARGE);

            // Header
            ui.horizontal(|ui| {
                if ui.button("← Back").clicked() {
                    if self.project_id.is_some() {
                        state.navigate_to(Screen::ProjectList);
                    } else {
                        state.navigate_to(Screen::Dashboard);
                    }
                }

                ui.add_space(Spacing::MEDIUM);
                ui.heading(RichText::new("Tickets").size(28.0));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.project_id.is_some()
                        && ui
                            .add_sized(
                                [120.0, 32.0],
                                egui::Button::new("+ New Ticket").fill(Colors::PRIMARY),
                            )
                            .clicked()
                    {
                        self.show_create_dialog = true;
                    }
                });
            });

            ui.add_space(Spacing::LARGE);

            // Filters
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search tickets...")
                        .desired_width(300.0),
                );

                ui.add_space(Spacing::MEDIUM);

                ui.label("Status:");
                egui::ComboBox::from_id_salt("status_filter")
                    .selected_text(match self.filter_status {
                        Some(s) => format!("{:?}", s),
                        None => "All".to_string(),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.filter_status, None, "All");
                        ui.selectable_value(
                            &mut self.filter_status,
                            Some(TicketStatus::Open),
                            "Open",
                        );
                        ui.selectable_value(
                            &mut self.filter_status,
                            Some(TicketStatus::InProgress),
                            "In Progress",
                        );
                        ui.selectable_value(
                            &mut self.filter_status,
                            Some(TicketStatus::Review),
                            "Review",
                        );
                        ui.selectable_value(
                            &mut self.filter_status,
                            Some(TicketStatus::Done),
                            "Done",
                        );
                        ui.selectable_value(
                            &mut self.filter_status,
                            Some(TicketStatus::Closed),
                            "Closed",
                        );
                    });
            });

            ui.add_space(Spacing::MEDIUM);
            ui.separator();
            ui.add_space(Spacing::MEDIUM);

            // Ticket list
            ScrollArea::vertical().show(ui, |ui| {
                let filtered_tickets: Vec<_> = self
                    .tickets
                    .iter()
                    .filter(|t| {
                        (self.filter_status.is_none() || Some(t.status) == self.filter_status)
                            && (self.search_query.is_empty()
                                || t.title
                                    .to_lowercase()
                                    .contains(&self.search_query.to_lowercase()))
                    })
                    .collect();

                if filtered_tickets.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(
                            RichText::new("No tickets found")
                                .size(16.0)
                                .color(egui::Color32::GRAY),
                        );
                    });
                } else {
                    for ticket in filtered_tickets {
                        self.render_ticket_card(ui, ticket, state);
                        ui.add_space(Spacing::SMALL);
                    }
                }
            });
        });
    }

    fn render_ticket_card(&self, ui: &mut egui::Ui, ticket: &Ticket, state: &mut AppState) {
        ui.group(|ui| {
            ui.set_min_size([f32::INFINITY, 60.0].into());
            ui.horizontal(|ui| {
                // Priority indicator
                let priority_color = match ticket.priority {
                    Priority::Low => Colors::PRIORITY_LOW,
                    Priority::Medium => Colors::PRIORITY_MEDIUM,
                    Priority::High => Colors::PRIORITY_HIGH,
                    Priority::Critical => Colors::PRIORITY_CRITICAL,
                };
                ui.colored_label(priority_color, "●");

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&ticket.title).strong());

                        // Type badge
                        let (type_text, type_color) = match ticket.ticket_type {
                            TicketType::Task => ("Task", Colors::TYPE_TASK),
                            TicketType::Bug => ("Bug", Colors::TYPE_BUG),
                            TicketType::Feature => ("Feature", Colors::TYPE_FEATURE),
                            TicketType::Epic => ("Epic", Colors::TYPE_EPIC),
                        };
                        ui.label(RichText::new(type_text).small().color(type_color));
                    });

                    if let Some(desc) = &ticket.description {
                        ui.label(
                            RichText::new(desc.chars().take(100).collect::<String>())
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                    }

                    ui.horizontal(|ui| {
                        // Status
                        let status_text = match ticket.status {
                            TicketStatus::Open => "Open",
                            TicketStatus::InProgress => "In Progress",
                            TicketStatus::Review => "Review",
                            TicketStatus::Done => "Done",
                            TicketStatus::Closed => "Closed",
                        };
                        ui.label(RichText::new(status_text).small());

                        // Priority
                        let priority_text = match ticket.priority {
                            Priority::Low => "Low",
                            Priority::Medium => "Medium",
                            Priority::High => "High",
                            Priority::Critical => "Critical",
                        };
                        ui.label(RichText::new(priority_text).small().color(priority_color));
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("View").clicked() {
                        state.navigate_to(Screen::TicketDetail(ticket.id));
                    }
                });
            });
        });
    }

    fn render_create_dialog(&mut self, ctx: &egui::Context, state: &mut AppState) {
        egui::Window::new("Create New Ticket")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(500.0);

                ui.label("Title");
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

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.radio_value(&mut self.new_ticket_type, TicketType::Task, "Task");
                    ui.radio_value(&mut self.new_ticket_type, TicketType::Bug, "Bug");
                    ui.radio_value(&mut self.new_ticket_type, TicketType::Feature, "Feature");
                    ui.radio_value(&mut self.new_ticket_type, TicketType::Epic, "Epic");
                });

                ui.add_space(Spacing::MEDIUM);

                ui.horizontal(|ui| {
                    ui.label("Priority:");
                    ui.radio_value(&mut self.new_ticket_priority, Priority::Low, "Low");
                    ui.radio_value(&mut self.new_ticket_priority, Priority::Medium, "Medium");
                    ui.radio_value(&mut self.new_ticket_priority, Priority::High, "High");
                    ui.radio_value(
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

        if let (Some(project_id), Some(user)) = (self.project_id, &state.current_user) {
            let mut ticket = Ticket::new(
                project_id,
                self.new_ticket_title.clone(),
                self.new_ticket_type,
                user.id,
            );

            if !self.new_ticket_description.is_empty() {
                ticket.description = Some(self.new_ticket_description.clone());
            }

            ticket.priority = self.new_ticket_priority;

            if state.is_demo_mode() {
                // Demo mode: Add to in-memory state
                state.demo_tickets.push(ticket);
                state.notify_success("Ticket created successfully (Demo Mode)".to_string());
                self.show_create_dialog = false;
                self.clear_create_form();
                self.load_tickets(state);
            } else {
                // Integrated mode: Call API
                // TODO: Implement API call when backend is ready
                // wasm_bindgen_futures::spawn_local(async move {
                //     match state.api_client.create_ticket(ticket).await {
                //         Ok(created_ticket) => { /* handle success */ },
                //         Err(e) => { /* handle error */ },
                //     }
                // });
                state.notify_error("Integrated mode: Backend API not yet connected".to_string());
            }
        }
    }

    fn clear_create_form(&mut self) {
        self.new_ticket_title.clear();
        self.new_ticket_description.clear();
        self.new_ticket_type = TicketType::Task;
        self.new_ticket_priority = Priority::Medium;
    }

    fn load_tickets(&mut self, state: &AppState) {
        if state.is_demo_mode() {
            // Demo mode: Load from in-memory state
            self.tickets = if let Some(project_id) = self.project_id {
                state
                    .demo_tickets
                    .iter()
                    .filter(|t| t.project_id == project_id)
                    .cloned()
                    .collect()
            } else {
                state.demo_tickets.clone()
            };
        } else {
            // Integrated mode: Load from API
            // TODO: Implement API call when backend is ready
            // wasm_bindgen_futures::spawn_local(async move {
            //     match state.api_client.get_tickets(self.project_id).await {
            //         Ok(tickets) => { /* update self.tickets */ },
            //         Err(e) => { /* handle error */ },
            //     }
            // });
            self.tickets = Vec::new();
        }
    }
}
