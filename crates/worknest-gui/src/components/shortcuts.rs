//! Keyboard shortcuts system and help modal

use crate::theme::{Colors, Spacing};
use egui::{Context, Key, Modifiers, RichText, Ui};

/// Keyboard shortcut definition
#[derive(Clone, Debug)]
pub struct ShortcutDefinition {
    pub key: Key,
    pub modifiers: Modifiers,
    pub description: String,
    pub category: String,
}

impl ShortcutDefinition {
    pub fn new(key: Key, modifiers: Modifiers, description: &str, category: &str) -> Self {
        Self {
            key,
            modifiers,
            description: description.to_string(),
            category: category.to_string(),
        }
    }

    /// Format the shortcut for display (e.g., "Ctrl+K", "Cmd+B")
    pub fn format(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.mac_cmd {
            parts.push("Cmd");
        }
        if self.modifiers.command {
            // Use Cmd on macOS, Ctrl on other platforms
            #[cfg(target_os = "macos")]
            parts.push("Cmd");
            #[cfg(not(target_os = "macos"))]
            parts.push("Ctrl");
        }

        parts.push(self.key_name());
        parts.join("+")
    }

    fn key_name(&self) -> &str {
        match self.key {
            Key::A => "A",
            Key::B => "B",
            Key::C => "C",
            Key::D => "D",
            Key::E => "E",
            Key::F => "F",
            Key::G => "G",
            Key::H => "H",
            Key::I => "I",
            Key::J => "J",
            Key::K => "K",
            Key::L => "L",
            Key::M => "M",
            Key::N => "N",
            Key::O => "O",
            Key::P => "P",
            Key::Q => "Q",
            Key::R => "R",
            Key::S => "S",
            Key::T => "T",
            Key::U => "U",
            Key::V => "V",
            Key::W => "W",
            Key::X => "X",
            Key::Y => "Y",
            Key::Z => "Z",
            Key::Num1 => "1",
            Key::Num2 => "2",
            Key::Num3 => "3",
            Key::Num4 => "4",
            Key::Num5 => "5",
            Key::Num6 => "6",
            Key::Num7 => "7",
            Key::Num8 => "8",
            Key::Num9 => "9",
            Key::Num0 => "0",
            Key::Escape => "Esc",
            Key::Enter => "Enter",
            Key::Space => "Space",
            Key::Tab => "Tab",
            Key::Slash => "/",
            Key::Questionmark => "?",
            _ => "?",
        }
    }
}

/// Keyboard shortcuts help modal component
pub struct ShortcutsHelp {
    /// Whether the help modal is currently open
    pub is_open: bool,
    /// All registered shortcuts
    shortcuts: Vec<ShortcutDefinition>,
}

impl ShortcutsHelp {
    pub fn new() -> Self {
        let mut shortcuts = Vec::new();

        // Navigation shortcuts
        shortcuts.push(ShortcutDefinition::new(
            Key::B,
            Modifiers::COMMAND,
            "Toggle sidebar",
            "Navigation",
        ));

        // Global shortcuts
        shortcuts.push(ShortcutDefinition::new(
            Key::K,
            Modifiers::COMMAND,
            "Open command palette",
            "Global",
        ));

        shortcuts.push(ShortcutDefinition::new(
            Key::Slash,
            Modifiers::COMMAND,
            "Quick search",
            "Global",
        ));

        shortcuts.push(ShortcutDefinition::new(
            Key::Questionmark,
            Modifiers::NONE,
            "Show keyboard shortcuts",
            "Global",
        ));

        // View shortcuts
        shortcuts.push(ShortcutDefinition::new(
            Key::Num1,
            Modifiers::COMMAND,
            "Go to Dashboard",
            "Views",
        ));

        shortcuts.push(ShortcutDefinition::new(
            Key::Num2,
            Modifiers::COMMAND,
            "Go to Projects",
            "Views",
        ));

        shortcuts.push(ShortcutDefinition::new(
            Key::Num3,
            Modifiers::COMMAND,
            "Go to All Tickets",
            "Views",
        ));

        shortcuts.push(ShortcutDefinition::new(
            Key::Num9,
            Modifiers::COMMAND,
            "Go to Settings",
            "Views",
        ));

        // Action shortcuts
        shortcuts.push(ShortcutDefinition::new(
            Key::N,
            Modifiers::COMMAND,
            "Create new (context-aware)",
            "Actions",
        ));

        shortcuts.push(ShortcutDefinition::new(
            Key::Escape,
            Modifiers::NONE,
            "Close dialog/modal",
            "Actions",
        ));

        Self {
            is_open: false,
            shortcuts,
        }
    }

    /// Toggle the help modal
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Open the help modal
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Close the help modal
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Check for the help shortcut (?) and handle opening the modal
    pub fn check_shortcut(&mut self, ctx: &Context) {
        // Check for ? key (without modifiers)
        if ctx.input(|i| i.key_pressed(Key::Questionmark) && !i.modifiers.any()) {
            self.toggle();
        }

        // Also check for Escape to close
        if self.is_open && ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.close();
        }
    }

    /// Render the help modal
    pub fn render(&mut self, ctx: &Context) {
        if !self.is_open {
            return;
        }

        // Modal background overlay
        egui::Area::new("shortcuts_help_overlay".into())
            .fixed_pos(egui::pos2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                if let Some(content_rect) = ui.ctx().input(|i| i.viewport().inner_rect) {
                    let painter = ui.painter();
                    painter.rect_filled(content_rect, 0.0, egui::Color32::from_black_alpha(128));
                }
            });

        // Modal window
        egui::Window::new("Keyboard Shortcuts")
            .id("shortcuts_help_modal".into())
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .collapsible(false)
            .resizable(false)
            .min_width(600.0)
            .show(ctx, |ui| {
                ui.add_space(Spacing::SMALL);

                // Group shortcuts by category
                let mut categories: std::collections::HashMap<String, Vec<&ShortcutDefinition>> =
                    std::collections::HashMap::new();

                for shortcut in &self.shortcuts {
                    categories
                        .entry(shortcut.category.clone())
                        .or_default()
                        .push(shortcut);
                }

                // Render each category
                let category_order = vec!["Global", "Navigation", "Views", "Actions"];
                for category in category_order {
                    if let Some(shortcuts) = categories.get(category) {
                        self.render_category(ui, category, shortcuts);
                        ui.add_space(Spacing::MEDIUM);
                    }
                }

                ui.add_space(Spacing::MEDIUM);
                ui.separator();
                ui.add_space(Spacing::SMALL);

                // Footer with close button
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Press Esc or ? to close")
                            .small()
                            .color(egui::Color32::GRAY),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            self.close();
                        }
                    });
                });
            });
    }

    fn render_category(&self, ui: &mut Ui, category: &str, shortcuts: &[&ShortcutDefinition]) {
        ui.label(
            RichText::new(category)
                .strong()
                .size(16.0)
                .color(Colors::PRIMARY),
        );
        ui.add_space(Spacing::SMALL);

        // Render shortcuts in this category
        for shortcut in shortcuts {
            ui.horizontal(|ui| {
                // Shortcut key combination (fixed width for alignment)
                let key_text = RichText::new(&shortcut.format())
                    .monospace()
                    .color(Colors::TEXT_SECONDARY);

                ui.add_sized([150.0, 20.0], egui::Label::new(key_text));

                // Description
                ui.label(&shortcut.description);
            });
            ui.add_space(2.0);
        }
    }

    /// Get all registered shortcuts
    pub fn shortcuts(&self) -> &[ShortcutDefinition] {
        &self.shortcuts
    }

    /// Register a new shortcut
    pub fn register(&mut self, shortcut: ShortcutDefinition) {
        self.shortcuts.push(shortcut);
    }
}

impl Default for ShortcutsHelp {
    fn default() -> Self {
        Self::new()
    }
}
