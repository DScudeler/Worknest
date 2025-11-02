//! Skeleton loader components for loading states

use egui::{Context, Ui, Vec2};
use crate::theme::Spacing;

/// Skeleton loader for list items
pub struct SkeletonLoader {
    /// Number of skeleton items to show
    count: usize,
    /// Animation progress (0.0 to 1.0)
    animation_progress: f32,
}

impl SkeletonLoader {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            animation_progress: 0.0,
        }
    }

    /// Update animation state
    pub fn update(&mut self, ctx: &Context) {
        self.animation_progress = (self.animation_progress + 0.02) % 1.0;
        ctx.request_repaint();
    }

    /// Render skeleton loaders
    pub fn render(&self, ui: &mut Ui) {
        for i in 0..self.count {
            self.render_item(ui, i);
            ui.add_space(Spacing::SMALL);
        }
    }

    /// Render a single skeleton item
    fn render_item(&self, ui: &mut Ui, index: usize) {
        // Calculate shimmer effect
        let phase = (self.animation_progress + (index as f32 * 0.1)) % 1.0;
        let shimmer_alpha = (phase * std::f32::consts::PI * 2.0).sin() * 0.3 + 0.7;

        let base_color = egui::Color32::from_gray(40);
        let shimmer_color = egui::Color32::from_gray((40.0 * shimmer_alpha) as u8);

        egui::Frame::new()
            .fill(base_color)
            .corner_radius(8.0)
            .inner_margin(egui::Margin::same(Spacing::MEDIUM as i8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Avatar/Icon placeholder
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(40.0, 40.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(
                        rect,
                        4.0,
                        shimmer_color,
                    );

                    ui.add_space(Spacing::MEDIUM);

                    ui.vertical(|ui| {
                        // Title line
                        let (rect, _) = ui.allocate_exact_size(
                            Vec2::new(200.0, 16.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(
                            rect,
                            4.0,
                            shimmer_color,
                        );

                        ui.add_space(Spacing::SMALL);

                        // Description line
                        let (rect, _) = ui.allocate_exact_size(
                            Vec2::new(150.0, 12.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(
                            rect,
                            4.0,
                            shimmer_color,
                        );
                    });
                });
            });
    }
}

impl Default for SkeletonLoader {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Skeleton loader for project cards
pub struct ProjectCardSkeleton {
    /// Number of cards to show
    count: usize,
    /// Animation progress
    animation_progress: f32,
}

impl ProjectCardSkeleton {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            animation_progress: 0.0,
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        self.animation_progress = (self.animation_progress + 0.02) % 1.0;
        ctx.request_repaint();
    }

    pub fn render(&self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for i in 0..self.count {
                self.render_card(ui, i);
            }
        });
    }

    fn render_card(&self, ui: &mut Ui, index: usize) {
        let phase = (self.animation_progress + (index as f32 * 0.1)) % 1.0;
        let shimmer_alpha = (phase * std::f32::consts::PI * 2.0).sin() * 0.3 + 0.7;

        let base_color = egui::Color32::from_gray(40);
        let shimmer_color = egui::Color32::from_gray((40.0 * shimmer_alpha) as u8);

        egui::Frame::new()
            .fill(base_color)
            .corner_radius(8.0)
            .inner_margin(egui::Margin::same(Spacing::MEDIUM as i8))
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(250.0, 120.0));

                ui.vertical(|ui| {
                    // Project name
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(180.0, 20.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 4.0, shimmer_color);

                    ui.add_space(Spacing::MEDIUM);

                    // Description
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(220.0, 14.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 4.0, shimmer_color);

                    ui.add_space(Spacing::SMALL);

                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(180.0, 14.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 4.0, shimmer_color);

                    ui.add_space(Spacing::MEDIUM);

                    // Metadata
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(
                            Vec2::new(60.0, 12.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 4.0, shimmer_color);

                        ui.add_space(Spacing::MEDIUM);

                        let (rect, _) = ui.allocate_exact_size(
                            Vec2::new(80.0, 12.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 4.0, shimmer_color);
                    });
                });
            });
    }
}

impl Default for ProjectCardSkeleton {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Skeleton loader for ticket list items
pub struct TicketSkeletonLoader {
    /// Number of skeleton items
    count: usize,
    /// Animation progress
    animation_progress: f32,
}

impl TicketSkeletonLoader {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            animation_progress: 0.0,
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        self.animation_progress = (self.animation_progress + 0.02) % 1.0;
        ctx.request_repaint();
    }

    pub fn render(&self, ui: &mut Ui) {
        for i in 0..self.count {
            self.render_item(ui, i);
            ui.add_space(Spacing::SMALL);
        }
    }

    fn render_item(&self, ui: &mut Ui, index: usize) {
        let phase = (self.animation_progress + (index as f32 * 0.1)) % 1.0;
        let shimmer_alpha = (phase * std::f32::consts::PI * 2.0).sin() * 0.3 + 0.7;

        let base_color = egui::Color32::from_gray(40);
        let shimmer_color = egui::Color32::from_gray((40.0 * shimmer_alpha) as u8);

        egui::Frame::new()
            .fill(base_color)
            .corner_radius(8.0)
            .inner_margin(egui::Margin::same(Spacing::MEDIUM as i8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Priority indicator
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(4.0, 40.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 2.0, shimmer_color);

                    ui.add_space(Spacing::MEDIUM);

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            // Ticket ID
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(80.0, 14.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 4.0, shimmer_color);

                            ui.add_space(Spacing::MEDIUM);

                            // Status badge
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(60.0, 14.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(rect, 4.0, shimmer_color);
                        });

                        ui.add_space(Spacing::SMALL);

                        // Title
                        let (rect, _) = ui.allocate_exact_size(
                            Vec2::new(250.0, 16.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 4.0, shimmer_color);

                        ui.add_space(Spacing::SMALL);

                        // Assignee/date
                        let (rect, _) = ui.allocate_exact_size(
                            Vec2::new(150.0, 12.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 4.0, shimmer_color);
                    });
                });
            });
    }
}

impl Default for TicketSkeletonLoader {
    fn default() -> Self {
        Self::new(5)
    }
}
