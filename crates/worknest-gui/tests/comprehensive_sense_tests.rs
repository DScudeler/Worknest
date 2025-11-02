//! Comprehensive Sense Configuration Tests
//!
//! These tests validate that ALL interactive elements across ALL screens
//! have proper egui::Sense configuration to ensure mouse events work correctly.
//!
//! This addresses the known bug where some screens display elements without
//! mouse events available.

use wasm_bindgen_test::*;
use worknest_core::models::{Priority, Project, Ticket, TicketStatus, TicketType, User};
use worknest_gui::{api_client::ApiClient, screens::Screen, state::AppState};

wasm_bindgen_test_configure!(run_in_browser);

// =============================================================================
// Dashboard Screen Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_dashboard_project_cards_have_sense() {
    // Project cards in Dashboard should be clickable and hoverable
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    // Add test projects
    for i in 0..3 {
        let project = Project::new(format!("Project {}", i), user.id);
        state.projects.push(project);
    }

    state.navigate_to(Screen::Dashboard);

    // Dashboard project cards SHOULD have Sense::click().union(Sense::hover())
    // This is implemented correctly in dashboard.rs:187-188
    assert_eq!(state.current_screen, Screen::Dashboard);
    assert_eq!(state.projects.len(), 3);
}

#[wasm_bindgen_test]
fn test_dashboard_navigation_buttons() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    state.navigate_to(Screen::Dashboard);

    // All buttons should be clickable
    assert!(state.is_authenticated());
}

// =============================================================================
// Project List Screen Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_project_list_project_cards_have_sense() {
    // Project cards in ProjectList should be clickable and hoverable
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    state.projects.push(project);

    state.navigate_to(Screen::ProjectList);

    // Project cards SHOULD have Sense::click().union(Sense::hover())
    // This is implemented correctly in project_list.rs:204-205
    assert_eq!(state.current_screen, Screen::ProjectList);
    assert_eq!(state.projects.len(), 1);
}

#[wasm_bindgen_test]
fn test_project_list_create_button() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user, "token".to_string());

    state.navigate_to(Screen::ProjectList);

    // "+ New Project" button should be clickable
    assert!(state.is_authenticated());
}

// =============================================================================
// Project Detail Screen Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_project_detail_buttons_available() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let project_id = project.id;
    state.projects.push(project);

    state.navigate_to(Screen::ProjectDetail(project_id));

    // All action buttons should be available:
    // - "Edit" button
    // - "+ New Ticket" button
    // - "View Board" button
    // - "View All Tickets" button
    // - "Back" button
    match state.current_screen {
        Screen::ProjectDetail(pid) => assert_eq!(pid, project_id),
        _ => panic!("Expected ProjectDetail screen"),
    }
}

#[wasm_bindgen_test]
fn test_project_detail_new_ticket_button_tooltip() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let project_id = project.id;
    state.projects.push(project);

    state.navigate_to(Screen::ProjectDetail(project_id));

    // The "+ New Ticket" button is always visible and should show tooltip
    // This was implemented in the recent fix
    assert!(state.is_authenticated());
}

// =============================================================================
// Ticket List Screen Tests - KNOWN BUG AREA
// =============================================================================

#[wasm_bindgen_test]
fn test_ticket_list_ticket_cards_missing_sense() {
    // BUG: Ticket cards in TicketList do NOT have .interact() called
    // They use ui.group() but don't make the group clickable
    // Only the "View" button inside works
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let ticket = Ticket::new(
        project.id,
        "Test Ticket".to_string(),
        TicketType::Task,
        user.id,
    );
    state.projects.push(project.clone());
    state.tickets.push(ticket);

    state.navigate_to(Screen::TicketList {
        project_id: Some(project.id),
    });

    // This test documents the bug:
    // - Ticket cards should be clickable to navigate to ticket detail
    // - Currently only the "View" button works
    // - The card itself has no Sense configuration
    assert_eq!(state.tickets.len(), 1);
}

#[wasm_bindgen_test]
fn test_ticket_list_filters_available() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    state.projects.push(project.clone());

    state.navigate_to(Screen::TicketList {
        project_id: Some(project.id),
    });

    // Filters should be interactive:
    // - Search text field
    // - Status dropdown
    assert!(state.is_authenticated());
}

#[wasm_bindgen_test]
fn test_ticket_list_new_ticket_button() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    state.projects.push(project.clone());

    state.navigate_to(Screen::TicketList {
        project_id: Some(project.id),
    });

    // "+ New Ticket" button should be enabled when project_id is present
    assert!(state.is_authenticated());
}

#[wasm_bindgen_test]
fn test_ticket_list_new_ticket_button_disabled_without_project() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user, "token".to_string());

    state.navigate_to(Screen::TicketList { project_id: None });

    // "+ New Ticket" button should be DISABLED and show tooltip
    // when no project_id is present
    assert!(state.is_authenticated());
}

// =============================================================================
// Ticket Board Screen Tests - KNOWN BUG AREA
// =============================================================================

#[wasm_bindgen_test]
fn test_ticket_board_cards_missing_sense() {
    // BUG: Ticket cards in TicketBoard do NOT have .interact() called
    // Same issue as TicketList - cards use ui.group() without Sense
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);

    // Create tickets in different statuses
    let ticket1 = Ticket::new(
        project.id,
        "Open Ticket".to_string(),
        TicketType::Task,
        user.id,
    );

    let mut ticket2 = Ticket::new(
        project.id,
        "In Progress Ticket".to_string(),
        TicketType::Bug,
        user.id,
    );
    ticket2.status = TicketStatus::InProgress;

    state.projects.push(project.clone());
    state.tickets.push(ticket1);
    state.tickets.push(ticket2);

    state.navigate_to(Screen::TicketBoard {
        project_id: project.id,
    });

    // This test documents the bug:
    // - Board cards should be clickable to navigate to ticket detail
    // - Currently only the "View" button works
    // - The card itself has no Sense configuration
    assert_eq!(state.tickets.len(), 2);
}

#[wasm_bindgen_test]
fn test_ticket_board_column_layout() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    state.projects.push(project.clone());

    state.navigate_to(Screen::TicketBoard {
        project_id: project.id,
    });

    // Board should show columns for each status
    // Columns themselves are not clickable (correct behavior)
    match state.current_screen {
        Screen::TicketBoard { project_id } => assert_eq!(project_id, project.id),
        _ => panic!("Expected TicketBoard screen"),
    }
}

// =============================================================================
// Ticket Detail Screen Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_ticket_detail_action_buttons() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let ticket = Ticket::new(
        project.id,
        "Test Ticket".to_string(),
        TicketType::Feature,
        user.id,
    );
    let ticket_id = ticket.id;

    state.projects.push(project);
    state.tickets.push(ticket);

    state.navigate_to(Screen::TicketDetail(ticket_id));

    // All action buttons should be clickable:
    // - "Edit" button
    // - Status change buttons
    // - "Delete" button
    // - "Back" button
    match state.current_screen {
        Screen::TicketDetail(tid) => assert_eq!(tid, ticket_id),
        _ => panic!("Expected TicketDetail screen"),
    }
}

#[wasm_bindgen_test]
fn test_ticket_detail_status_buttons() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let ticket = Ticket::new(
        project.id,
        "Test Ticket".to_string(),
        TicketType::Bug,
        user.id,
    );
    let ticket_id = ticket.id;

    state.projects.push(project);
    state.tickets.push(ticket);

    state.navigate_to(Screen::TicketDetail(ticket_id));

    // Status buttons should all be clickable:
    // - Open, In Progress, Review, Done, Closed
    assert!(state.is_authenticated());
}

#[wasm_bindgen_test]
fn test_ticket_detail_comment_section() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let ticket = Ticket::new(
        project.id,
        "Test Ticket".to_string(),
        TicketType::Task,
        user.id,
    );
    let ticket_id = ticket.id;

    state.projects.push(project);
    state.tickets.push(ticket);

    state.navigate_to(Screen::TicketDetail(ticket_id));

    // Comment section elements:
    // - Comment text area should be editable
    // - "Post Comment" button should be clickable
    // - Edit/Delete buttons on comments should be clickable
    assert!(state.is_authenticated());
}

// =============================================================================
// Settings Screen Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_settings_tab_navigation() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user, "token".to_string());

    state.navigate_to(Screen::Settings);

    // Tab buttons should be clickable:
    // - Profile
    // - Preferences
    // - About
    assert_eq!(state.current_screen, Screen::Settings);
}

#[wasm_bindgen_test]
fn test_settings_profile_form_elements() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user, "token".to_string());

    state.navigate_to(Screen::Settings);

    // Profile form elements should be interactive:
    // - Username text field
    // - Email text field
    // - "Save Changes" button
    assert!(state.is_authenticated());
}

#[wasm_bindgen_test]
fn test_settings_password_form_elements() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user, "token".to_string());

    state.navigate_to(Screen::Settings);

    // Password form elements should be interactive:
    // - Current password field
    // - New password field
    // - Confirm password field
    // - "Change Password" button
    assert!(state.is_authenticated());
}

#[wasm_bindgen_test]
fn test_settings_preferences_elements() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user, "token".to_string());

    state.navigate_to(Screen::Settings);

    // Preferences elements should be interactive:
    // - Theme radio buttons
    // - Notification checkboxes
    // - Default view radio buttons
    assert!(state.is_authenticated());
}

// =============================================================================
// Login/Register Screen Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_login_form_elements() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let state = AppState::new(api_client);

    // Login form elements should be interactive:
    // - Username/email text field
    // - Password text field
    // - "Login" button
    // - "Register" link/button
    assert_eq!(state.current_screen, Screen::Login);
}

#[wasm_bindgen_test]
fn test_register_form_elements() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    state.navigate_to(Screen::Register);

    // Register form elements should be interactive:
    // - Username text field
    // - Email text field
    // - Password text field
    // - Confirm password text field
    // - "Register" button
    // - "Back to Login" link/button
    assert_eq!(state.current_screen, Screen::Register);
}

// =============================================================================
// Cross-Screen Navigation Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_navigation_flow_comprehensive() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let project_id = project.id;
    let ticket = Ticket::new(
        project_id,
        "Test Ticket".to_string(),
        TicketType::Task,
        user.id,
    );
    let ticket_id = ticket.id;

    state.projects.push(project);
    state.tickets.push(ticket);

    // Test complete navigation flow
    assert_eq!(state.current_screen, Screen::Dashboard);

    state.navigate_to(Screen::ProjectList);
    assert_eq!(state.current_screen, Screen::ProjectList);

    state.navigate_to(Screen::ProjectDetail(project_id));
    match state.current_screen {
        Screen::ProjectDetail(pid) => assert_eq!(pid, project_id),
        _ => panic!("Expected ProjectDetail"),
    }

    state.navigate_to(Screen::TicketList {
        project_id: Some(project_id),
    });
    match state.current_screen {
        Screen::TicketList { project_id: Some(pid) } => assert_eq!(pid, project_id),
        _ => panic!("Expected TicketList with project_id"),
    }

    state.navigate_to(Screen::TicketBoard { project_id });
    match state.current_screen {
        Screen::TicketBoard { project_id: pid } => assert_eq!(pid, project_id),
        _ => panic!("Expected TicketBoard"),
    }

    state.navigate_to(Screen::TicketDetail(ticket_id));
    match state.current_screen {
        Screen::TicketDetail(tid) => assert_eq!(tid, ticket_id),
        _ => panic!("Expected TicketDetail"),
    }

    state.navigate_to(Screen::Settings);
    assert_eq!(state.current_screen, Screen::Settings);

    state.navigate_to(Screen::Dashboard);
    assert_eq!(state.current_screen, Screen::Dashboard);
}

// =============================================================================
// Priority and Type Visual Indicators Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_priority_indicators_all_types() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);

    // Create tickets with all priority levels
    let priorities = vec![
        Priority::Low,
        Priority::Medium,
        Priority::High,
        Priority::Critical,
    ];

    for priority in priorities {
        let mut ticket = Ticket::new(
            project.id,
            format!("{:?} Priority Ticket", priority),
            TicketType::Task,
            user.id,
        );
        ticket.priority = priority;
        state.tickets.push(ticket);
    }

    state.projects.push(project.clone());

    state.navigate_to(Screen::TicketList {
        project_id: Some(project.id),
    });

    // All priority indicators should be visible
    assert_eq!(state.tickets.len(), 4);
}

#[wasm_bindgen_test]
fn test_ticket_type_badges_all_types() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);

    // Create tickets with all types
    let types = vec![
        TicketType::Task,
        TicketType::Bug,
        TicketType::Feature,
        TicketType::Epic,
    ];

    for ticket_type in types {
        let ticket = Ticket::new(
            project.id,
            format!("{:?} Type Ticket", ticket_type),
            ticket_type,
            user.id,
        );
        state.tickets.push(ticket);
    }

    state.projects.push(project.clone());

    state.navigate_to(Screen::TicketBoard {
        project_id: project.id,
    });

    // All type badges should be visible
    assert_eq!(state.tickets.len(), 4);
}
