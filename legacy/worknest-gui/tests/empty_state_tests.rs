//! Tests for empty state components

use wasm_bindgen_test::*;
use worknest_gui::components::{EmptyState, EmptyStateAction, EmptyStates};
use worknest_gui::screens::Screen;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_empty_state_creation() {
    let empty_state = EmptyState::new("📁", "No Data", "There is no data to display");
    assert_eq!(empty_state.icon, "📁");
    assert_eq!(empty_state.heading, "No Data");
    assert_eq!(empty_state.message, "There is no data to display");
    assert!(empty_state.cta.is_none());
}

#[wasm_bindgen_test]
fn test_empty_state_with_cta() {
    let empty_state =
        EmptyState::new("📁", "No Data", "Message").with_cta("Click Me", EmptyStateAction::Refresh);

    assert!(empty_state.cta.is_some());
    let cta = empty_state.cta.unwrap();
    assert_eq!(cta.label, "Click Me");
}

#[wasm_bindgen_test]
fn test_empty_states_no_projects() {
    let state = EmptyStates::no_projects();
    assert_eq!(state.icon, "📁");
    assert_eq!(state.heading, "No Projects Yet");
    assert!(state.cta.is_some());
}

#[wasm_bindgen_test]
fn test_empty_states_no_tickets() {
    let state = EmptyStates::no_tickets();
    assert_eq!(state.icon, "🎫");
    assert_eq!(state.heading, "No Tickets");
    assert!(state.cta.is_some());
}

#[wasm_bindgen_test]
fn test_empty_states_no_tickets_in_project() {
    let state = EmptyStates::no_tickets_in_project("My Project");
    assert_eq!(state.icon, "🎫");
    assert_eq!(state.heading, "No Tickets in Project");
    assert!(state.message.contains("My Project"));
    assert!(state.cta.is_some());
}

#[wasm_bindgen_test]
fn test_empty_states_no_search_results() {
    let state = EmptyStates::no_search_results("test query");
    assert_eq!(state.icon, "🔍");
    assert_eq!(state.heading, "No Results Found");
    assert!(state.message.contains("test query"));
    assert!(state.cta.is_none());
}

#[wasm_bindgen_test]
fn test_empty_states_loading_failed() {
    let state = EmptyStates::loading_failed();
    assert_eq!(state.icon, "⚠️");
    assert_eq!(state.heading, "Failed to Load Data");
    assert!(state.cta.is_some());
}

#[wasm_bindgen_test]
fn test_empty_states_access_denied() {
    let state = EmptyStates::access_denied();
    assert_eq!(state.icon, "🔒");
    assert_eq!(state.heading, "Access Denied");
    assert!(state.cta.is_some());
}

#[wasm_bindgen_test]
fn test_empty_states_not_found() {
    let state = EmptyStates::not_found();
    assert_eq!(state.icon, "❓");
    assert_eq!(state.heading, "Page Not Found");
    assert!(state.cta.is_some());
}

#[wasm_bindgen_test]
fn test_empty_states_coming_soon() {
    let state = EmptyStates::coming_soon("Analytics");
    assert_eq!(state.icon, "🚧");
    assert_eq!(state.heading, "Coming Soon");
    assert!(state.message.contains("Analytics"));
    assert!(state.cta.is_none());
}

#[wasm_bindgen_test]
fn test_empty_states_archived() {
    let state = EmptyStates::archived();
    assert_eq!(state.icon, "📦");
    assert_eq!(state.heading, "No Archived Items");
    assert!(state.cta.is_none());
}

#[wasm_bindgen_test]
fn test_empty_state_action_navigate() {
    let action = EmptyStateAction::Navigate(Screen::Dashboard);
    // Action is created successfully
    assert!(true);
}

#[wasm_bindgen_test]
fn test_empty_state_action_create_project() {
    let action = EmptyStateAction::CreateProject;
    // Action is created successfully
    assert!(true);
}

#[wasm_bindgen_test]
fn test_empty_state_action_create_ticket() {
    let action = EmptyStateAction::CreateTicket;
    // Action is created successfully
    assert!(true);
}

#[wasm_bindgen_test]
fn test_empty_state_action_refresh() {
    let action = EmptyStateAction::Refresh;
    // Action is created successfully
    assert!(true);
}

#[wasm_bindgen_test]
fn test_multiple_empty_states() {
    let state1 = EmptyStates::no_projects();
    let state2 = EmptyStates::no_tickets();
    let state3 = EmptyStates::not_found();

    assert_eq!(state1.icon, "📁");
    assert_eq!(state2.icon, "🎫");
    assert_eq!(state3.icon, "❓");
}

#[wasm_bindgen_test]
fn test_empty_state_custom_with_multiple_ctas() {
    // Create base empty state
    let state1 =
        EmptyState::new("🎯", "Title", "Message").with_cta("Action 1", EmptyStateAction::Refresh);

    let state2 = EmptyState::new("🎯", "Title", "Message")
        .with_cta("Action 2", EmptyStateAction::CreateProject);

    assert!(state1.cta.is_some());
    assert!(state2.cta.is_some());
}

#[wasm_bindgen_test]
fn test_empty_state_chaining() {
    let state =
        EmptyState::new("Icon", "Heading", "Message").with_cta("Button", EmptyStateAction::Refresh);

    assert_eq!(state.icon, "Icon");
    assert_eq!(state.heading, "Heading");
    assert_eq!(state.message, "Message");
    assert!(state.cta.is_some());
}
