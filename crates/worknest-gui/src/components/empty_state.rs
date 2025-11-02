//! Empty state components with call-to-action

use crate::{
    screens::Screen,
    state::AppState,
    theme::{Colors, Spacing},
};
use egui::{RichText, Ui};

/// Empty state configuration
pub struct EmptyState {
    /// Icon or emoji to display
    pub icon: String,
    /// Main heading text
    pub heading: String,
    /// Descriptive message
    pub message: String,
    /// Optional call-to-action button
    pub cta: Option<CallToAction>,
}

/// Call-to-action button configuration
pub struct CallToAction {
    /// Button label
    pub label: String,
    /// Action to perform
    pub action: EmptyStateAction,
}

/// Actions that can be triggered from empty states
#[derive(Clone, Debug)]
pub enum EmptyStateAction {
    /// Navigate to a specific screen
    Navigate(Screen),
    /// Create a new project
    CreateProject,
    /// Create a new ticket
    CreateTicket,
    /// Refresh/reload data
    Refresh,
}

impl EmptyState {
    /// Create a new empty state
    pub fn new(
        icon: impl Into<String>,
        heading: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            icon: icon.into(),
            heading: heading.into(),
            message: message.into(),
            cta: None,
        }
    }

    /// Add a call-to-action button
    pub fn with_cta(mut self, label: impl Into<String>, action: EmptyStateAction) -> Self {
        self.cta = Some(CallToAction {
            label: label.into(),
            action,
        });
        self
    }

    /// Render the empty state
    pub fn render(&self, ui: &mut Ui, _state: &mut AppState) -> Option<EmptyStateAction> {
        let mut action = None;

        ui.vertical_centered(|ui| {
            ui.add_space(Spacing::LARGE * 2.0);

            // Icon
            ui.label(
                RichText::new(&self.icon)
                    .size(64.0)
                    .color(egui::Color32::from_gray(100)),
            );

            ui.add_space(Spacing::LARGE);

            // Heading
            ui.label(
                RichText::new(&self.heading)
                    .size(24.0)
                    .strong()
                    .color(Colors::TEXT_PRIMARY),
            );

            ui.add_space(Spacing::MEDIUM);

            // Message
            ui.label(
                RichText::new(&self.message)
                    .size(14.0)
                    .color(Colors::TEXT_SECONDARY),
            );

            ui.add_space(Spacing::LARGE);

            // CTA button
            if let Some(cta) = &self.cta {
                if ui
                    .add_sized(
                        [150.0, 40.0],
                        egui::Button::new(&cta.label).fill(Colors::PRIMARY),
                    )
                    .clicked()
                {
                    action = Some(cta.action.clone());
                }
            }

            ui.add_space(Spacing::LARGE * 2.0);
        });

        action
    }
}

/// Pre-configured empty states for common scenarios
pub struct EmptyStates;

impl EmptyStates {
    /// No projects empty state
    pub fn no_projects() -> EmptyState {
        EmptyState::new(
            "📁",
            "No Projects Yet",
            "Create your first project to get started with organizing your work",
        )
        .with_cta("Create Project", EmptyStateAction::CreateProject)
    }

    /// No tickets empty state
    pub fn no_tickets() -> EmptyState {
        EmptyState::new(
            "🎫",
            "No Tickets",
            "No tickets found. Create a new ticket to track your work",
        )
        .with_cta("Create Ticket", EmptyStateAction::CreateTicket)
    }

    /// No tickets in project
    pub fn no_tickets_in_project(project_name: &str) -> EmptyState {
        EmptyState::new(
            "🎫",
            "No Tickets in Project",
            &format!(
                "'{}' doesn't have any tickets yet. Create one to get started",
                project_name
            ),
        )
        .with_cta("Create Ticket", EmptyStateAction::CreateTicket)
    }

    /// No search results
    pub fn no_search_results(query: &str) -> EmptyState {
        EmptyState::new(
            "🔍",
            "No Results Found",
            &format!("No results found for '{}'", query),
        )
    }

    /// Loading failed
    pub fn loading_failed() -> EmptyState {
        EmptyState::new(
            "⚠️",
            "Failed to Load Data",
            "Something went wrong while loading. Please try again",
        )
        .with_cta("Retry", EmptyStateAction::Refresh)
    }

    /// Access denied
    pub fn access_denied() -> EmptyState {
        EmptyState::new(
            "🔒",
            "Access Denied",
            "You don't have permission to view this content",
        )
        .with_cta(
            "Go to Dashboard",
            EmptyStateAction::Navigate(Screen::Dashboard),
        )
    }

    /// Not found (404)
    pub fn not_found() -> EmptyState {
        EmptyState::new(
            "❓",
            "Page Not Found",
            "The page you're looking for doesn't exist",
        )
        .with_cta(
            "Go to Dashboard",
            EmptyStateAction::Navigate(Screen::Dashboard),
        )
    }

    /// Coming soon
    pub fn coming_soon(feature_name: &str) -> EmptyState {
        EmptyState::new(
            "🚧",
            "Coming Soon",
            &format!("{} is currently under development", feature_name),
        )
    }

    /// Archived items
    pub fn archived() -> EmptyState {
        EmptyState::new(
            "📦",
            "No Archived Items",
            "Items you archive will appear here",
        )
    }
}
