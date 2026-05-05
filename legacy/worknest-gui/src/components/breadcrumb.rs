//! Breadcrumb navigation component

use egui::{Context, RichText, Sense};

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};

/// Breadcrumb item in the navigation trail
#[derive(Clone, Debug)]
pub struct BreadcrumbItem {
    pub label: String,
    pub screen: Option<Screen>,
    pub is_current: bool,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<String>, screen: Option<Screen>) -> Self {
        Self {
            label: label.into(),
            screen,
            is_current: false,
        }
    }

    pub fn current(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            screen: None,
            is_current: true,
        }
    }
}

/// Breadcrumb navigation component
pub struct Breadcrumb {
    /// Breadcrumb items
    items: Vec<BreadcrumbItem>,
}

impl Breadcrumb {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Generate breadcrumb trail from current screen and state
    pub fn generate_trail(screen: &Screen, state: &AppState) -> Vec<BreadcrumbItem> {
        let mut items = vec![BreadcrumbItem::new("Home", Some(Screen::Dashboard))];

        match screen {
            Screen::Login | Screen::Register => {
                // No breadcrumbs for auth screens
                return vec![];
            },
            Screen::Dashboard => {
                items[0].is_current = true;
            },
            Screen::ProjectList => {
                items.push(BreadcrumbItem::current("Projects"));
            },
            Screen::ProjectDetail(project_id) => {
                items.push(BreadcrumbItem::new("Projects", Some(Screen::ProjectList)));

                // Find project name
                if let Some(project) = state.projects.iter().find(|p| p.id == *project_id) {
                    items.push(BreadcrumbItem::current(&project.name));
                } else {
                    items.push(BreadcrumbItem::current("Project"));
                }
            },
            Screen::TicketList { project_id } => {
                if let Some(pid) = project_id {
                    items.push(BreadcrumbItem::new("Projects", Some(Screen::ProjectList)));

                    // Find project name
                    if let Some(project) = state.projects.iter().find(|p| p.id == *pid) {
                        items.push(BreadcrumbItem::new(
                            &project.name,
                            Some(Screen::ProjectDetail(*pid)),
                        ));
                    }

                    items.push(BreadcrumbItem::current("Tickets"));
                } else {
                    items.push(BreadcrumbItem::current("All Tickets"));
                }
            },
            Screen::TicketBoard { project_id } => {
                items.push(BreadcrumbItem::new("Projects", Some(Screen::ProjectList)));

                // Find project name
                if let Some(project) = state.projects.iter().find(|p| p.id == *project_id) {
                    items.push(BreadcrumbItem::new(
                        &project.name,
                        Some(Screen::ProjectDetail(*project_id)),
                    ));
                }

                items.push(BreadcrumbItem::current("Board"));
            },
            Screen::TicketDetail(ticket_id) => {
                // Find ticket and its project
                if let Some(ticket) = state.tickets.iter().find(|t| t.id == *ticket_id) {
                    items.push(BreadcrumbItem::new("Projects", Some(Screen::ProjectList)));

                    if let Some(project) = state.projects.iter().find(|p| p.id == ticket.project_id)
                    {
                        items.push(BreadcrumbItem::new(
                            &project.name,
                            Some(Screen::ProjectDetail(project.id)),
                        ));
                    }

                    items.push(BreadcrumbItem::new(
                        "Tickets",
                        Some(Screen::TicketList {
                            project_id: Some(ticket.project_id),
                        }),
                    ));

                    items.push(BreadcrumbItem::current(&ticket.title));
                } else {
                    items.push(BreadcrumbItem::current("Ticket"));
                }
            },
            Screen::Settings => {
                items.push(BreadcrumbItem::current("Settings"));
            },
        }

        items
    }

    /// Update breadcrumb trail based on current screen
    pub fn update(&mut self, screen: &Screen, state: &AppState) {
        self.items = Self::generate_trail(screen, state);
    }

    /// Render the breadcrumb navigation
    pub fn render(&self, ctx: &Context, state: &mut AppState) {
        if self.items.is_empty() {
            return;
        }

        egui::TopBottomPanel::top("breadcrumb")
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(30, 30, 35))
                    .inner_margin(egui::Margin {
                        left: Spacing::MEDIUM as i8,
                        right: Spacing::MEDIUM as i8,
                        top: Spacing::SMALL as i8,
                        bottom: Spacing::SMALL as i8,
                    }),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (index, item) in self.items.iter().enumerate() {
                        // Render separator between items
                        if index > 0 {
                            ui.label(
                                RichText::new("/")
                                    .color(egui::Color32::from_gray(100))
                                    .size(12.0),
                            );
                            ui.add_space(Spacing::SMALL);
                        }

                        // Render breadcrumb item
                        if item.is_current {
                            // Current page (not clickable)
                            ui.label(
                                RichText::new(&item.label)
                                    .color(Colors::TEXT_PRIMARY)
                                    .strong(),
                            );
                        } else if let Some(ref screen) = item.screen {
                            // Clickable breadcrumb
                            let response = ui.add(
                                egui::Label::new(
                                    RichText::new(&item.label).color(Colors::TEXT_SECONDARY),
                                )
                                .sense(Sense::click()),
                            );

                            if response.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }

                            if response.clicked() {
                                state.navigate_to(screen.clone());
                            }

                            // Underline on hover
                            if response.hovered() {
                                let rect = response.rect;
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(rect.left(), rect.bottom()),
                                        egui::pos2(rect.right(), rect.bottom()),
                                    ],
                                    egui::Stroke::new(1.0, Colors::PRIMARY),
                                );
                            }
                        }

                        ui.add_space(Spacing::SMALL);
                    }
                });
            });
    }
}

impl Default for Breadcrumb {
    fn default() -> Self {
        Self::new()
    }
}
