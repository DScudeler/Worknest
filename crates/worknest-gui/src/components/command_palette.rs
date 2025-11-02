//! Command palette for quick navigation and actions

use egui::{Context, Key, KeyboardShortcut, Modifiers, RichText, TextEdit};
use crate::{screens::Screen, state::AppState, theme::{Colors, Spacing}};

/// A command that can be executed from the palette
#[derive(Clone, Debug)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub category: CommandCategory,
    pub action: CommandAction,
}

/// Category of commands for grouping
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Navigation,
    Actions,
    Settings,
}

impl CommandCategory {
    pub fn name(&self) -> &str {
        match self {
            CommandCategory::Navigation => "Navigation",
            CommandCategory::Actions => "Actions",
            CommandCategory::Settings => "Settings",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            CommandCategory::Navigation => "🧭",
            CommandCategory::Actions => "⚡",
            CommandCategory::Settings => "⚙️",
        }
    }
}

/// Actions that can be triggered by commands
#[derive(Clone, Debug)]
pub enum CommandAction {
    Navigate(Screen),
    ToggleSidebar,
    ToggleTheme,
    ShowHelp,
}

/// Command palette component
pub struct CommandPalette {
    /// Whether the palette is currently open
    pub is_open: bool,
    /// Current search query
    search_query: String,
    /// Selected command index
    selected_index: usize,
    /// Available commands
    commands: Vec<Command>,
    /// Filtered commands based on search
    filtered_commands: Vec<usize>, // Indices into commands vec
    /// Whether to focus the search input
    focus_input: bool,
}

impl CommandPalette {
    pub fn new() -> Self {
        let mut palette = Self {
            is_open: false,
            search_query: String::new(),
            selected_index: 0,
            commands: Vec::new(),
            filtered_commands: Vec::new(),
            focus_input: false,
        };

        palette.initialize_commands();
        palette.update_filtered_commands();
        palette
    }

    fn initialize_commands(&mut self) {
        self.commands = vec![
            // Navigation commands
            Command {
                name: "Go to Dashboard".to_string(),
                description: "View your dashboard overview".to_string(),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate(Screen::Dashboard),
            },
            Command {
                name: "Go to Projects".to_string(),
                description: "View all projects".to_string(),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate(Screen::ProjectList),
            },
            Command {
                name: "Go to All Tickets".to_string(),
                description: "View all tickets across projects".to_string(),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate(Screen::TicketList { project_id: None }),
            },
            Command {
                name: "Go to Settings".to_string(),
                description: "Manage your account settings".to_string(),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate(Screen::Settings),
            },
            // Action commands
            Command {
                name: "Toggle Sidebar".to_string(),
                description: "Show or hide the sidebar".to_string(),
                category: CommandCategory::Actions,
                action: CommandAction::ToggleSidebar,
            },
            Command {
                name: "Toggle Theme".to_string(),
                description: "Switch between light and dark mode".to_string(),
                category: CommandCategory::Settings,
                action: CommandAction::ToggleTheme,
            },
            Command {
                name: "Show Keyboard Shortcuts".to_string(),
                description: "View all available keyboard shortcuts".to_string(),
                category: CommandCategory::Actions,
                action: CommandAction::ShowHelp,
            },
        ];
    }

    /// Add dynamic commands based on current state (projects, tickets, etc.)
    pub fn add_dynamic_commands(&mut self, state: &AppState) {
        // Remove old dynamic commands (keep only the static ones)
        self.commands.retain(|cmd| {
            !matches!(cmd.action, CommandAction::Navigate(Screen::ProjectDetail(_)))
                && !matches!(cmd.action, CommandAction::Navigate(Screen::TicketDetail(_)))
        });

        // Add project commands
        for project in &state.projects {
            self.commands.push(Command {
                name: format!("Open Project: {}", project.name),
                description: project.description.clone().unwrap_or_default(),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate(Screen::ProjectDetail(project.id)),
            });
        }

        // Add recent ticket commands (up to 10)
        for ticket in state.tickets.iter().take(10) {
            let project_name = state
                .projects
                .iter()
                .find(|p| p.id == ticket.project_id)
                .map(|p| p.name.as_str())
                .unwrap_or("Unknown Project");

            self.commands.push(Command {
                name: format!("Open Ticket: {}", ticket.title),
                description: format!("{} - {}", project_name, ticket.ticket_type),
                category: CommandCategory::Navigation,
                action: CommandAction::Navigate(Screen::TicketDetail(ticket.id)),
            });
        }

        self.update_filtered_commands();
    }

    /// Toggle the command palette
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open {
            self.search_query.clear();
            self.selected_index = 0;
            self.focus_input = true;
            self.update_filtered_commands();
        }
    }

    /// Open the command palette
    pub fn open(&mut self) {
        self.is_open = true;
        self.search_query.clear();
        self.selected_index = 0;
        self.focus_input = true;
        self.update_filtered_commands();
    }

    /// Close the command palette
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Update filtered commands based on search query
    fn update_filtered_commands(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_commands = (0..self.commands.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_commands = self
                .commands
                .iter()
                .enumerate()
                .filter(|(_, cmd)| {
                    cmd.name.to_lowercase().contains(&query)
                        || cmd.description.to_lowercase().contains(&query)
                        || cmd.category.name().to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_commands.len() {
            self.selected_index = 0;
        }
    }

    /// Check for keyboard shortcut and handle opening
    pub fn check_shortcut(&mut self, ctx: &Context) {
        // Check for Ctrl/Cmd+K
        if ctx.input_mut(|i| {
            i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::K))
        }) {
            self.toggle();
        }

        // Check for Escape to close
        if self.is_open && ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.close();
        }
    }

    /// Execute the currently selected command
    fn execute_selected(&mut self, _state: &mut AppState) -> Option<CommandAction> {
        if let Some(&cmd_idx) = self.filtered_commands.get(self.selected_index) {
            if let Some(cmd) = self.commands.get(cmd_idx) {
                let action = cmd.action.clone();
                self.close();
                return Some(action);
            }
        }
        None
    }

    /// Render the command palette
    pub fn render(&mut self, ctx: &Context, state: &mut AppState) -> Option<CommandAction> {
        if !self.is_open {
            return None;
        }

        // Update dynamic commands when opening
        if self.focus_input {
            self.add_dynamic_commands(state);
        }

        let mut action = None;

        // Modal background overlay
        egui::Area::new("command_palette_overlay".into())
            .fixed_pos(egui::pos2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                if let Some(content_rect) = ui.ctx().input(|i| i.viewport().inner_rect) {
                    let painter = ui.painter();
                    painter.rect_filled(
                        content_rect,
                        0.0,
                        egui::Color32::from_black_alpha(128),
                    );
                }
            });

        // Command palette window
        egui::Window::new("Command Palette")
            .id("command_palette_window".into())
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .min_width(600.0)
            .max_width(600.0)
            .show(ctx, |ui| {
                ui.add_space(Spacing::SMALL);

                // Search input
                let search_response = ui.add_sized(
                    [ui.available_width(), 30.0],
                    TextEdit::singleline(&mut self.search_query)
                        .hint_text("Search commands...")
                        .font(egui::TextStyle::Body),
                );

                // Auto-focus input when opened
                if self.focus_input {
                    search_response.request_focus();
                    self.focus_input = false;
                }

                // Update filtered commands when search changes
                if search_response.changed() {
                    self.update_filtered_commands();
                }

                // Handle keyboard navigation
                ui.input(|i| {
                    if i.key_pressed(Key::ArrowDown) {
                        self.selected_index = (self.selected_index + 1)
                            .min(self.filtered_commands.len().saturating_sub(1));
                    }
                    if i.key_pressed(Key::ArrowUp) {
                        self.selected_index = self.selected_index.saturating_sub(1);
                    }
                    if i.key_pressed(Key::Enter) {
                        action = self.execute_selected(state);
                    }
                });

                ui.add_space(Spacing::MEDIUM);
                ui.separator();
                ui.add_space(Spacing::SMALL);

                // Results list
                if self.filtered_commands.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(Spacing::LARGE);
                        ui.label(
                            RichText::new("No commands found")
                                .color(egui::Color32::GRAY),
                        );
                        ui.add_space(Spacing::LARGE);
                    });
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            for (display_idx, &cmd_idx) in self.filtered_commands.iter().enumerate() {
                                if let Some(cmd) = self.commands.get(cmd_idx) {
                                    let is_selected = display_idx == self.selected_index;

                                    let response = ui.add_sized(
                                        [ui.available_width(), 50.0],
                                        egui::Button::new("")
                                            .fill(if is_selected {
                                                Colors::PRIMARY.linear_multiply(0.3)
                                            } else {
                                                egui::Color32::TRANSPARENT
                                            })
                                            .frame(false),
                                    );

                                    // Render command content on top of button
                                    let button_rect = response.rect;
                                    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(button_rect));
                                    child_ui.horizontal(|ui| {
                                        ui.add_space(Spacing::MEDIUM);

                                        // Category icon
                                        ui.label(
                                            RichText::new(cmd.category.icon())
                                                .size(20.0),
                                        );

                                        ui.add_space(Spacing::SMALL);

                                        ui.vertical(|ui| {
                                            // Command name
                                            ui.label(RichText::new(&cmd.name).strong());

                                            // Command description
                                            if !cmd.description.is_empty() {
                                                ui.label(
                                                    RichText::new(&cmd.description)
                                                        .small()
                                                        .color(egui::Color32::GRAY),
                                                );
                                            }
                                        });
                                    });

                                    if response.clicked() {
                                        action = Some(cmd.action.clone());
                                    }

                                    // Auto-scroll to selected
                                    if is_selected {
                                        response.scroll_to_me(Some(egui::Align::Center));
                                    }
                                }
                            }
                        });
                }

                ui.add_space(Spacing::SMALL);
                ui.separator();
                ui.add_space(Spacing::SMALL);

                // Footer with hint
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("↑↓ Navigate • Enter Select • Esc Close")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                });

                ui.add_space(Spacing::SMALL);
            });

        // Close palette if an action was selected
        if action.is_some() {
            self.close();
        }

        action
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}
