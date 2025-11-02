//! Toast notification component with modern styling

use egui::{Context, RichText, Sense};
use web_time::{Duration, Instant};

use crate::{
    state::{Notification, NotificationLevel},
    theme::{Colors, Spacing},
};

/// Toast notification manager
pub struct ToastManager {
    /// Currently displayed toasts
    toasts: Vec<Toast>,
    /// Auto-dismiss duration
    auto_dismiss_duration: Duration,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            auto_dismiss_duration: Duration::from_secs(5),
        }
    }

    /// Add a notification as a toast
    pub fn add_toast(&mut self, notification: Notification) {
        self.toasts.push(Toast {
            notification,
            is_hovered: false,
            is_dismissed: false,
        });

        // Keep only last 5 toasts
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
    }

    /// Update toasts from notification list
    pub fn update_from_notifications(&mut self, notifications: &[Notification]) {
        // Find new notifications that aren't already in toasts
        for notification in notifications {
            let already_exists = self.toasts.iter().any(|toast| {
                toast.notification.message == notification.message
                    && toast.notification.timestamp == notification.timestamp
            });

            if !already_exists {
                self.add_toast(notification.clone());
            }
        }
    }

    /// Clear dismissed toasts and old ones
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.toasts.retain(|toast| {
            !toast.is_dismissed
                && (toast.is_hovered
                    || now.duration_since(toast.notification.timestamp)
                        < self.auto_dismiss_duration)
        });
    }

    /// Render all toasts
    pub fn render(&mut self, ctx: &Context) {
        self.cleanup();

        if self.toasts.is_empty() {
            return;
        }

        // Position toasts in top-right corner
        let screen_width =
            ctx.input(|i| i.viewport().inner_rect.map(|r| r.width()).unwrap_or(1024.0));

        egui::Area::new("toast_container".into())
            .fixed_pos(egui::pos2(
                screen_width - 320.0 - Spacing::LARGE,
                Spacing::LARGE,
            ))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.set_width(320.0);

                // Render each toast with spacing
                let mut dismissed_indices = Vec::new();

                for (index, toast) in self.toasts.iter_mut().enumerate() {
                    if toast.render(ui, &self.auto_dismiss_duration) {
                        dismissed_indices.push(index);
                    }
                    ui.add_space(Spacing::SMALL);
                }

                // Mark dismissed toasts
                for index in dismissed_indices {
                    if let Some(toast) = self.toasts.get_mut(index) {
                        toast.is_dismissed = true;
                    }
                }
            });
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual toast notification
struct Toast {
    notification: Notification,
    is_hovered: bool,
    is_dismissed: bool,
}

impl Toast {
    /// Render the toast and return true if dismissed
    fn render(&mut self, ui: &mut egui::Ui, auto_dismiss_duration: &Duration) -> bool {
        let mut dismissed = false;

        let (bg_color, icon, icon_color) = match self.notification.level {
            NotificationLevel::Success => (
                egui::Color32::from_rgb(22, 101, 52), // Dark green
                "✓",
                Colors::SUCCESS,
            ),
            NotificationLevel::Error => (
                egui::Color32::from_rgb(127, 29, 29), // Dark red
                "✕",
                Colors::ERROR,
            ),
            NotificationLevel::Info => (
                egui::Color32::from_rgb(30, 58, 138), // Dark blue
                "ℹ",
                Colors::INFO,
            ),
        };

        // Calculate progress (remaining time)
        let elapsed = Instant::now().duration_since(self.notification.timestamp);
        let progress = if self.is_hovered {
            1.0
        } else {
            1.0 - (elapsed.as_secs_f32() / auto_dismiss_duration.as_secs_f32()).min(1.0)
        };

        egui::Frame::new()
            .fill(bg_color)
            .corner_radius(8.0)
            .inner_margin(egui::Margin::same(Spacing::MEDIUM as i8))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)))
            .show(ui, |ui| {
                let response = ui
                    .horizontal(|ui| {
                        // Icon
                        ui.add_space(Spacing::SMALL);
                        ui.label(RichText::new(icon).size(20.0).color(icon_color));

                        ui.add_space(Spacing::MEDIUM);

                        // Message
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 40.0);

                            ui.label(
                                RichText::new(&self.notification.message)
                                    .color(egui::Color32::WHITE),
                            );

                            // Progress bar
                            ui.add_space(Spacing::SMALL);
                            let progress_bar_height = 2.0;
                            let progress_rect = egui::Rect::from_min_size(
                                ui.cursor().min,
                                egui::vec2(ui.available_width() * progress, progress_bar_height),
                            );

                            ui.painter().rect_filled(
                                progress_rect,
                                0.0,
                                egui::Color32::from_white_alpha(180),
                            );

                            ui.allocate_space(egui::vec2(
                                ui.available_width(),
                                progress_bar_height,
                            ));
                        });

                        // Close button
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_response = ui.add(
                                egui::Button::new(RichText::new("×").size(18.0))
                                    .fill(egui::Color32::TRANSPARENT)
                                    .frame(false),
                            );

                            if close_response.clicked() {
                                dismissed = true;
                            }
                        });
                    })
                    .response;

                // Track hover state to pause auto-dismiss
                self.is_hovered = response.hovered();

                // Make the whole toast clickable to dismiss
                if response.interact(Sense::click()).clicked() {
                    dismissed = true;
                }
            });

        dismissed
    }
}
