//! State Transition Validation Framework
//!
//! Validates that state changes produce expected UI transitions and displays.

use worknest_gui::screens::Screen;
use worknest_gui::state::AppState;

/// Validates state transitions
pub struct StateTransitionValidator {
    pub initial_screen: Screen,
    pub expected_screen: Screen,
    pub should_update_data: bool,
    pub should_show_notification: bool,
}

impl StateTransitionValidator {
    pub fn new(initial: Screen, expected: Screen) -> Self {
        Self {
            initial_screen: initial,
            expected_screen: expected,
            should_update_data: false,
            should_show_notification: false,
        }
    }

    /// Expect data to be modified
    pub fn expects_data_update(mut self) -> Self {
        self.should_update_data = true;
        self
    }

    /// Expect notification to be shown
    pub fn expects_notification(mut self) -> Self {
        self.should_show_notification = true;
        self
    }

    /// Validate the transition
    pub fn validate(&self, state: &AppState) -> TransitionResult {
        let screen_correct = state.current_screen == self.expected_screen;
        let notification_correct = !self.should_show_notification
            || !state.notifications.is_empty();

        TransitionResult {
            passed: screen_correct && notification_correct,
            actual_screen: state.current_screen.clone(),
            expected_screen: self.expected_screen.clone(),
            has_notification: !state.notifications.is_empty(),
        }
    }
}

#[derive(Debug)]
pub struct TransitionResult {
    pub passed: bool,
    pub actual_screen: Screen,
    pub expected_screen: Screen,
    pub has_notification: bool,
}

impl TransitionResult {
    pub fn assert_passed(&self) {
        if !self.passed {
            panic!(
                "State transition failed:\n  Expected: {:?}\n  Actual: {:?}",
                self.expected_screen, self.actual_screen
            );
        }
    }
}

/// Validates screen-specific state
pub struct ScreenValidator {
    screen: Screen,
    validations: Vec<Box<dyn Fn(&AppState) -> bool>>,
}

impl ScreenValidator {
    pub fn for_screen(screen: Screen) -> Self {
        Self {
            screen,
            validations: Vec::new(),
        }
    }

    /// Add a custom validation function
    pub fn add_validation<F>(mut self, validation: F) -> Self
    where
        F: Fn(&AppState) -> bool + 'static,
    {
        self.validations.push(Box::new(validation));
        self
    }

    /// Validate that user is authenticated
    pub fn requires_authentication(self) -> Self {
        self.add_validation(|state| state.is_authenticated())
    }

    /// Validate that specific data exists
    pub fn requires_projects(self, min_count: usize) -> Self {
        self.add_validation(move |state| state.projects.len() >= min_count)
    }

    /// Validate that specific data exists
    pub fn requires_tickets(self, min_count: usize) -> Self {
        self.add_validation(move |state| state.tickets.len() >= min_count)
    }

    /// Run all validations
    pub fn validate(&self, state: &AppState) -> ScreenValidationResult {
        let screen_correct = state.current_screen == self.screen;
        let all_validations_passed = self.validations.iter().all(|v| v(state));

        ScreenValidationResult {
            passed: screen_correct && all_validations_passed,
            screen_matches: screen_correct,
            validations_passed: all_validations_passed,
        }
    }
}

#[derive(Debug)]
pub struct ScreenValidationResult {
    pub passed: bool,
    pub screen_matches: bool,
    pub validations_passed: bool,
}

impl ScreenValidationResult {
    pub fn assert_passed(&self) {
        if !self.passed {
            if !self.screen_matches {
                panic!("Screen does not match expected");
            }
            if !self.validations_passed {
                panic!("Screen validations failed");
            }
        }
    }
}

/// Action-driven state transition builder
pub struct StateTransitionBuilder {
    transitions: Vec<(String, StateTransitionValidator)>,
}

impl StateTransitionBuilder {
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
        }
    }

    /// Add a transition to validate
    pub fn add_transition(
        mut self,
        action: impl Into<String>,
        validator: StateTransitionValidator,
    ) -> Self {
        self.transitions.push((action.into(), validator));
        self
    }

    /// Build the transition map
    pub fn build(self) -> Vec<(String, StateTransitionValidator)> {
        self.transitions
    }
}

impl Default for StateTransitionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worknest_gui::api_client::ApiClient;

    #[test]
    fn test_state_transition_validation() {
        let validator = StateTransitionValidator::new(
            Screen::Login,
            Screen::Dashboard,
        );

        let api_client = ApiClient::new("http://localhost:3000".to_string());
        let mut state = AppState::new(api_client);

        // Initially on login screen
        assert_eq!(state.current_screen, Screen::Login);

        // Navigate to dashboard
        state.navigate_to(Screen::Dashboard);

        // Validate transition
        let result = validator.validate(&state);
        assert!(result.passed);
    }

    #[test]
    fn test_screen_validator_authentication() {
        let validator = ScreenValidator::for_screen(Screen::Dashboard)
            .requires_authentication();

        let api_client = ApiClient::new("http://localhost:3000".to_string());
        let state = AppState::new(api_client);

        // Should fail - not authenticated
        let result = validator.validate(&state);
        assert!(!result.passed);
    }
}
