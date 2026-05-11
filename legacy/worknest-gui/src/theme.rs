//! Theme and styling for the application

use egui::{Color32, CornerRadius, Stroke, Visuals};

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    /// Toggle between light and dark theme
    pub fn toggle(&self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }

    /// Apply theme to egui context
    pub fn apply(&self, ctx: &egui::Context) {
        match self {
            Theme::Light => ctx.set_visuals(Visuals::light()),
            Theme::Dark => ctx.set_visuals(Visuals::dark()),
        }
    }
}

/// Color palette
pub struct Colors;

impl Colors {
    // Primary colors
    pub const PRIMARY: Color32 = Color32::from_rgb(59, 130, 246); // Blue
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(37, 99, 235);
    pub const PRIMARY_ACTIVE: Color32 = Color32::from_rgb(29, 78, 216);

    // Status colors
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94); // Green
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68); // Red
    pub const WARNING: Color32 = Color32::from_rgb(251, 191, 36); // Yellow
    pub const INFO: Color32 = Color32::from_rgb(59, 130, 246); // Blue

    // Priority colors
    pub const PRIORITY_LOW: Color32 = Color32::from_rgb(148, 163, 184); // Gray
    pub const PRIORITY_MEDIUM: Color32 = Color32::from_rgb(251, 191, 36); // Yellow
    pub const PRIORITY_HIGH: Color32 = Color32::from_rgb(249, 115, 22); // Orange
    pub const PRIORITY_CRITICAL: Color32 = Color32::from_rgb(239, 68, 68); // Red

    // Ticket type colors
    pub const TYPE_TASK: Color32 = Color32::from_rgb(59, 130, 246); // Blue
    pub const TYPE_BUG: Color32 = Color32::from_rgb(239, 68, 68); // Red
    pub const TYPE_FEATURE: Color32 = Color32::from_rgb(168, 85, 247); // Purple
    pub const TYPE_EPIC: Color32 = Color32::from_rgb(251, 191, 36); // Yellow

    // Text colors (will use theme defaults)
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(17, 24, 39);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(107, 114, 128);
}

/// Spacing constants
pub struct Spacing;

impl Spacing {
    pub const SMALL: f32 = 4.0;
    pub const MEDIUM: f32 = 8.0;
    pub const LARGE: f32 = 16.0;
    pub const XLARGE: f32 = 24.0;
}

/// Common UI components styling
pub struct Styles;

impl Styles {
    /// Create a primary button style
    pub fn primary_button() -> egui::Button<'static> {
        egui::Button::new("").fill(Colors::PRIMARY)
    }

    /// Create a danger button style
    pub fn danger_button() -> egui::Button<'static> {
        egui::Button::new("").fill(Colors::ERROR)
    }

    /// Create a card frame
    pub fn card() -> egui::Frame {
        egui::Frame::default()
            .inner_margin(Spacing::LARGE)
            .outer_margin(Spacing::MEDIUM)
            .corner_radius(CornerRadius::same(8))
            .stroke(Stroke::new(1.0, Color32::from_gray(200)))
            .fill(Color32::from_gray(250))
    }

    /// Create a panel frame
    pub fn panel() -> egui::Frame {
        egui::Frame::default()
            .inner_margin(Spacing::LARGE)
            .fill(Color32::from_gray(245))
    }
}
