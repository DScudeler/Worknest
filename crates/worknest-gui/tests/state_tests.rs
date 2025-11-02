//! State management tests for Worknest GUI

use wasm_bindgen_test::*;
use worknest_gui::{api_client::ApiClient, screens::Screen, state::AppState};

wasm_bindgen_test_configure!(run_in_browser);

/// Test basic state initialization
#[wasm_bindgen_test]
fn test_state_initialization() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let state = AppState::new(api_client);

    assert!(!state.is_authenticated());
    assert_eq!(state.current_screen, Screen::Login);
    assert!(state.notifications.is_empty());
    assert!(!state.is_loading);
}

/// Test navigation between screens
#[wasm_bindgen_test]
fn test_navigation() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    // Initial screen
    assert_eq!(state.current_screen, Screen::Login);

    // Navigate to register
    state.navigate_to(Screen::Register);
    assert_eq!(state.current_screen, Screen::Register);

    // Navigate to dashboard
    state.navigate_to(Screen::Dashboard);
    assert_eq!(state.current_screen, Screen::Dashboard);

    // Navigate to project list
    state.navigate_to(Screen::ProjectList);
    assert_eq!(state.current_screen, Screen::ProjectList);
}

/// Test notification system
#[wasm_bindgen_test]
fn test_notifications() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    // Initially empty
    assert_eq!(state.notifications.len(), 0);

    // Add success notification
    state.notify_success("Test success".to_string());
    assert_eq!(state.notifications.len(), 1);

    // Add error notification
    state.notify_error("Test error".to_string());
    assert_eq!(state.notifications.len(), 2);

    // Add info notification
    state.notify_info("Test info".to_string());
    assert_eq!(state.notifications.len(), 3);

    // Test that notifications are capped at 10
    for i in 0..15 {
        state.notify_info(format!("Notification {}", i));
    }
    assert!(state.notifications.len() <= 10, "Notifications should be capped at 10");
}

/// Test authentication state
#[wasm_bindgen_test]
fn test_authentication() {
    use worknest_core::models::User;

    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    // Initially not authenticated
    assert!(!state.is_authenticated());

    // Create a test user
    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    let token = "test_token_123".to_string();

    // Login
    state.login(user.clone(), token.clone());

    // Now authenticated
    assert!(state.is_authenticated());
    assert_eq!(state.current_screen, Screen::Dashboard);
    assert_eq!(state.current_user.as_ref().unwrap().username, "testuser");
    assert_eq!(state.auth_token, Some(token));

    // Logout
    state.logout();

    // Back to not authenticated
    assert!(!state.is_authenticated());
    assert_eq!(state.current_screen, Screen::Login);
    assert!(state.current_user.is_none());
    assert!(state.auth_token.is_none());
}

/// Test loading state
#[wasm_bindgen_test]
fn test_loading_state() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let state = AppState::new(api_client);

    assert!(!state.is_loading);
}

/// Test demo projects initialization
#[wasm_bindgen_test]
fn test_demo_projects() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let state = AppState::new(api_client);

    // Initially empty (demo data added by screens)
    assert_eq!(state.projects.len(), 0);
    assert_eq!(state.tickets.len(), 0);
}
