//! Layer-Based Testing Framework for Worknest GUI
//!
//! This framework provides comprehensive testing capabilities for egui applications
//! by testing each layer independently:
//!
//! 1. **UI Layer**: egui widget interactions, availability, and event handling
//! 2. **State Layer**: State transitions and API call triggers
//! 3. **Integration Layer**: Complete workflows across all layers

pub mod api_validation;
pub mod state_transition;
pub mod ui_interaction;

// Re-export main testing utilities
pub use api_validation::{ApiCallValidator, ExpectedCall, MockApiClient};
pub use state_transition::{ScreenValidator, StateTransitionValidator};
pub use ui_interaction::{ElementAvailability, InteractionMatrix, UiTestContext};
