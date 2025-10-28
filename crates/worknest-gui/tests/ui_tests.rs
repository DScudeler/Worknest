//! UI interaction and logic tests for Worknest GUI

use wasm_bindgen_test::*;
use worknest_core::models::{Project, User};
use worknest_gui::{api_client::ApiClient, screens::Screen, state::AppState};

wasm_bindgen_test_configure!(run_in_browser);

/// Test project creation in demo mode
#[wasm_bindgen_test]
fn test_project_creation() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    // Login to enable project creation
    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    // Initially no projects
    assert_eq!(state.demo_projects.len(), 0);

    // Create a project
    let project = Project::new("Test Project".to_string(), user.id);
    state.demo_projects.push(project);

    // Verify project was added
    assert_eq!(state.demo_projects.len(), 1);
    assert_eq!(state.demo_projects[0].name, "Test Project");
}

/// Test project archiving
#[wasm_bindgen_test]
fn test_project_archiving() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    // Create a project
    let mut project = Project::new("Test Project".to_string(), user.id);
    assert!(!project.archived);

    // Archive it
    project.archived = true;
    state.demo_projects.push(project.clone());

    assert!(state.demo_projects[0].archived);

    // Unarchive it
    state.demo_projects[0].archived = false;
    assert!(!state.demo_projects[0].archived);
}

/// Test screen transitions with project detail
#[wasm_bindgen_test]
fn test_project_detail_navigation() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let project_id = project.id;
    state.demo_projects.push(project);

    // Navigate to project detail
    state.navigate_to(Screen::ProjectDetail(project_id));
    assert_eq!(state.current_screen, Screen::ProjectDetail(project_id));
}

/// Test ticket list navigation with project filter
#[wasm_bindgen_test]
fn test_ticket_list_navigation() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let project_id = project.id;
    state.demo_projects.push(project);

    // Navigate to ticket list for this project
    state.navigate_to(Screen::TicketList {
        project_id: Some(project_id),
    });

    match state.current_screen {
        Screen::TicketList { project_id: Some(pid) } => {
            assert_eq!(pid, project_id);
        }
        _ => panic!("Expected TicketList screen with project_id"),
    }

    // Navigate to all tickets (no project filter)
    state.navigate_to(Screen::TicketList { project_id: None });

    match state.current_screen {
        Screen::TicketList { project_id: None } => {}
        _ => panic!("Expected TicketList screen without project_id"),
    }
}

/// Test ticket board navigation
#[wasm_bindgen_test]
fn test_ticket_board_navigation() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let project_id = project.id;
    state.demo_projects.push(project);

    // Navigate to ticket board
    state.navigate_to(Screen::TicketBoard { project_id });

    match state.current_screen {
        Screen::TicketBoard { project_id: pid } => {
            assert_eq!(pid, project_id);
        }
        _ => panic!("Expected TicketBoard screen"),
    }
}

/// Test that notifications are properly formatted
#[wasm_bindgen_test]
fn test_notification_messages() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    state.notify_success("Success message".to_string());
    state.notify_error("Error message".to_string());
    state.notify_info("Info message".to_string());

    assert_eq!(state.notifications.len(), 3);
    assert_eq!(state.notifications[0].message, "Success message");
    assert_eq!(state.notifications[1].message, "Error message");
    assert_eq!(state.notifications[2].message, "Info message");
}

/// Test project search/filter functionality
#[wasm_bindgen_test]
fn test_project_filtering() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    // Create multiple projects
    let mut project1 = Project::new("Alpha Project".to_string(), user.id);
    project1.description = Some("First project".to_string());

    let mut project2 = Project::new("Beta Project".to_string(), user.id);
    project2.description = Some("Second project".to_string());

    let mut project3 = Project::new("Gamma Project".to_string(), user.id);
    project3.archived = true;

    state.demo_projects.push(project1);
    state.demo_projects.push(project2);
    state.demo_projects.push(project3);

    // Test filtering by name (case-insensitive)
    let alpha_projects: Vec<&Project> = state
        .demo_projects
        .iter()
        .filter(|p| p.name.to_lowercase().contains("alpha"))
        .collect();
    assert_eq!(alpha_projects.len(), 1);
    assert_eq!(alpha_projects[0].name, "Alpha Project");

    // Test filtering by archived status
    let active_projects: Vec<&Project> = state
        .demo_projects
        .iter()
        .filter(|p| !p.archived)
        .collect();
    assert_eq!(active_projects.len(), 2);

    let archived_projects: Vec<&Project> = state
        .demo_projects
        .iter()
        .filter(|p| p.archived)
        .collect();
    assert_eq!(archived_projects.len(), 1);
}
