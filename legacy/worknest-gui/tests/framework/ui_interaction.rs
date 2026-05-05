//! UI Interaction Testing Framework
//!
//! Tests egui widget availability, clickability, and interaction capabilities.
//! Since egui is immediate mode, we can't query a persistent DOM. Instead, we:
//! 1. Render the UI to capture widget responses
//! 2. Analyze egui::Response to determine availability and interaction state
//! 3. Build an interaction matrix to verify correct button/element configuration

use egui::{Context, Response, Sense};
use std::collections::HashMap;

/// Tracks UI element availability and interaction capabilities
#[derive(Debug, Clone)]
pub struct ElementAvailability {
    pub id: String,
    pub visible: bool,
    pub enabled: bool,
    pub sense: SenseCapability,
    pub hovered: bool,
    pub clicked: bool,
}

/// Represents interaction capabilities of a UI element
#[derive(Debug, Clone, PartialEq)]
pub struct SenseCapability {
    pub can_click: bool,
    pub can_hover: bool,
    pub can_drag: bool,
}

impl From<Sense> for SenseCapability {
    fn from(sense: Sense) -> Self {
        Self {
            can_click: sense.contains(Sense::CLICK),
            can_hover: sense.contains(Sense::HOVER)
                || sense.contains(Sense::CLICK)
                || sense.contains(Sense::DRAG),
            can_drag: sense.contains(Sense::DRAG),
        }
    }
}

/// UI testing context that tracks rendered elements
pub struct UiTestContext {
    elements: HashMap<String, ElementAvailability>,
    frame_count: u32,
}

impl UiTestContext {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            frame_count: 0,
        }
    }

    /// Record a widget interaction for testing
    pub fn record_widget(&mut self, id: impl Into<String>, response: &Response) {
        let availability = ElementAvailability {
            id: id.into(),
            visible: true, // If we got a response, it was rendered
            enabled: response.enabled(),
            sense: response.sense.into(),
            hovered: response.hovered(),
            clicked: response.clicked(),
        };

        self.elements.insert(availability.id.clone(), availability);
    }

    /// Record an element that should exist but wasn't found
    pub fn record_missing(&mut self, id: impl Into<String>) {
        let availability = ElementAvailability {
            id: id.into(),
            visible: false,
            enabled: false,
            sense: SenseCapability {
                can_click: false,
                can_hover: false,
                can_drag: false,
            },
            hovered: false,
            clicked: false,
        };

        self.elements.insert(availability.id.clone(), availability);
    }

    /// Check if an element is available and clickable
    pub fn is_clickable(&self, id: &str) -> bool {
        self.elements
            .get(id)
            .map(|e| e.visible && e.enabled && e.sense.can_click)
            .unwrap_or(false)
    }

    /// Check if an element is visible
    pub fn is_visible(&self, id: &str) -> bool {
        self.elements.get(id).map(|e| e.visible).unwrap_or(false)
    }

    /// Check if an element can receive hover events
    pub fn can_hover(&self, id: &str) -> bool {
        self.elements
            .get(id)
            .map(|e| e.visible && e.sense.can_hover)
            .unwrap_or(false)
    }

    /// Get all clickable elements
    pub fn get_clickable_elements(&self) -> Vec<String> {
        self.elements
            .iter()
            .filter(|(_, e)| e.visible && e.enabled && e.sense.can_click)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all visible but non-clickable elements (potential bugs)
    pub fn get_non_clickable_visible(&self) -> Vec<String> {
        self.elements
            .iter()
            .filter(|(_, e)| e.visible && (!e.enabled || !e.sense.can_click))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Increment frame counter
    pub fn next_frame(&mut self) {
        self.frame_count += 1;
        // Don't clear elements - maintain history for assertions
    }

    /// Clear all tracked elements
    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

impl Default for UiTestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Action-State-UI Interaction Matrix
///
/// Maps user actions to expected state changes and UI updates
#[derive(Debug, Clone)]
pub struct InteractionMatrix {
    pub screen: String,
    pub action: UserAction,
    pub expected_state: ExpectedState,
    pub expected_ui: ExpectedUI,
}

#[derive(Debug, Clone)]
pub enum UserAction {
    Click(String),         // Click element with ID
    Input(String, String), // Input text into field with ID
    Navigate(String),      // Navigate to screen
    Hover(String),         // Hover over element
}

#[derive(Debug, Clone)]
pub struct ExpectedState {
    pub screen_changed: bool,
    pub new_screen: Option<String>,
    pub data_modified: bool,
    pub api_called: bool,
}

#[derive(Debug, Clone)]
pub struct ExpectedUI {
    pub elements_visible: Vec<String>,
    pub elements_hidden: Vec<String>,
    pub elements_enabled: Vec<String>,
    pub elements_disabled: Vec<String>,
}

impl InteractionMatrix {
    /// Create a new interaction matrix entry
    pub fn new(screen: impl Into<String>, action: UserAction) -> Self {
        Self {
            screen: screen.into(),
            action,
            expected_state: ExpectedState {
                screen_changed: false,
                new_screen: None,
                data_modified: false,
                api_called: false,
            },
            expected_ui: ExpectedUI {
                elements_visible: Vec::new(),
                elements_hidden: Vec::new(),
                elements_enabled: Vec::new(),
                elements_disabled: Vec::new(),
            },
        }
    }

    /// Set expected state changes
    pub fn expects_state(mut self, state: ExpectedState) -> Self {
        self.expected_state = state;
        self
    }

    /// Set expected UI changes
    pub fn expects_ui(mut self, ui: ExpectedUI) -> Self {
        self.expected_ui = ui;
        self
    }
}

/// Builder for creating interaction test matrices
pub struct InteractionMatrixBuilder {
    matrices: Vec<InteractionMatrix>,
}

impl InteractionMatrixBuilder {
    pub fn new() -> Self {
        Self {
            matrices: Vec::new(),
        }
    }

    /// Add an interaction to test
    pub fn add_interaction(mut self, matrix: InteractionMatrix) -> Self {
        self.matrices.push(matrix);
        self
    }

    /// Build the complete test matrix
    pub fn build(self) -> Vec<InteractionMatrix> {
        self.matrices
    }

    /// Create a comprehensive matrix for a screen
    pub fn for_screen(screen: impl Into<String>) -> Self {
        Self::new()
    }
}

impl Default for InteractionMatrixBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sense_capability_from_click() {
        let sense = Sense::click();
        let capability = SenseCapability::from(sense);

        assert!(capability.can_click);
        assert!(capability.can_hover); // Click implies hover in our model
        assert!(!capability.can_drag);
    }

    #[test]
    fn test_sense_capability_from_hover() {
        let sense = Sense::hover();
        let capability = SenseCapability::from(sense);

        assert!(!capability.can_click);
        assert!(capability.can_hover);
        assert!(!capability.can_drag);
    }

    #[test]
    fn test_sense_capability_from_drag() {
        let sense = Sense::drag();
        let capability = SenseCapability::from(sense);

        assert!(!capability.can_click);
        assert!(capability.can_hover); // Drag implies hover in our model
        assert!(capability.can_drag);
    }

    #[test]
    fn test_sense_capability_from_union() {
        let sense = Sense::click().union(Sense::hover());
        let capability = SenseCapability::from(sense);

        assert!(capability.can_click);
        assert!(capability.can_hover);
        assert!(!capability.can_drag);
    }

    #[test]
    fn test_interaction_matrix_builder() {
        let matrix = InteractionMatrix::new(
            "project_list",
            UserAction::Click("new_project_btn".to_string()),
        )
        .expects_state(ExpectedState {
            screen_changed: false,
            new_screen: None,
            data_modified: false,
            api_called: false,
        })
        .expects_ui(ExpectedUI {
            elements_visible: vec!["create_dialog".to_string()],
            elements_hidden: vec![],
            elements_enabled: vec!["create_button".to_string()],
            elements_disabled: vec![],
        });

        assert_eq!(matrix.screen, "project_list");
        assert_eq!(matrix.expected_ui.elements_visible.len(), 1);
    }
}
