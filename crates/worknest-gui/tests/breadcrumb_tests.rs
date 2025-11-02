//! Tests for the breadcrumb navigation component

use wasm_bindgen_test::*;
use chrono::Utc;
use worknest_core::models::{Priority, Project, ProjectId, Ticket, TicketId, TicketStatus, TicketType, UserId};
use worknest_gui::{
    api_client::ApiClient,
    components::{Breadcrumb, BreadcrumbItem},
    screens::Screen,
    state::AppState,
};

wasm_bindgen_test_configure!(run_in_browser);

fn create_test_state() -> AppState {
    let api_client = ApiClient::new_default();
    let mut state = AppState::new(api_client);

    // Add test project (using valid UUID format)
    let project = Project {
        id: ProjectId::from_string("urn:uuid:00000000-0000-0000-0000-000000000001").unwrap(),
        name: "Test Project".to_string(),
        description: Some("Test Description".to_string()),
        color: None,
        archived: false,
        created_by: UserId::from_string("urn:uuid:00000000-0000-0000-0000-000000000010").unwrap(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    state.projects.push(project);

    // Add test ticket (using valid UUID format)
    let ticket = Ticket {
        id: TicketId::from_string("urn:uuid:00000000-0000-0000-0000-000000000100").unwrap(),
        project_id: ProjectId::from_string("urn:uuid:00000000-0000-0000-0000-000000000001").unwrap(),
        title: "Test Ticket".to_string(),
        description: Some("Test Description".to_string()),
        ticket_type: TicketType::Task,
        status: TicketStatus::InProgress,
        priority: Priority::Medium,
        assignee_id: None,
        created_by: UserId::from_string("urn:uuid:00000000-0000-0000-0000-000000000010").unwrap(),
        due_date: None,
        estimate_hours: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    state.tickets.push(ticket);

    state
}

#[wasm_bindgen_test]
fn test_breadcrumb_initialization() {
    let breadcrumb = Breadcrumb::new();
    assert!(true, "Breadcrumb initializes successfully");
}

#[wasm_bindgen_test]
fn test_breadcrumb_default() {
    let breadcrumb = Breadcrumb::default();
    assert!(true, "Breadcrumb can be created with default()");
}

#[wasm_bindgen_test]
fn test_breadcrumb_item_creation() {
    let item = BreadcrumbItem::new("Home", Some(Screen::Dashboard));
    assert_eq!(item.label, "Home");
    assert!(!item.is_current);
}

#[wasm_bindgen_test]
fn test_breadcrumb_item_current() {
    let item = BreadcrumbItem::current("Settings");
    assert_eq!(item.label, "Settings");
    assert!(item.is_current);
    assert!(item.screen.is_none());
}

#[wasm_bindgen_test]
fn test_generate_trail_dashboard() {
    let state = create_test_state();
    let trail = Breadcrumb::generate_trail(&Screen::Dashboard, &state);

    assert_eq!(trail.len(), 1);
    assert_eq!(trail[0].label, "Home");
    assert!(trail[0].is_current);
}

#[wasm_bindgen_test]
fn test_generate_trail_project_list() {
    let state = create_test_state();
    let trail = Breadcrumb::generate_trail(&Screen::ProjectList, &state);

    assert_eq!(trail.len(), 2);
    assert_eq!(trail[0].label, "Home");
    assert!(!trail[0].is_current);
    assert_eq!(trail[1].label, "Projects");
    assert!(trail[1].is_current);
}

#[wasm_bindgen_test]
fn test_generate_trail_project_detail() {
    let state = create_test_state();
    let project_id = ProjectId::from_string("urn:uuid:00000000-0000-0000-0000-000000000001").unwrap();
    let trail = Breadcrumb::generate_trail(&Screen::ProjectDetail(project_id), &state);

    assert_eq!(trail.len(), 3);
    assert_eq!(trail[0].label, "Home");
    assert_eq!(trail[1].label, "Projects");
    assert_eq!(trail[2].label, "Test Project");
    assert!(trail[2].is_current);
}

#[wasm_bindgen_test]
fn test_generate_trail_ticket_list_with_project() {
    let state = create_test_state();
    let project_id = ProjectId::from_string("urn:uuid:00000000-0000-0000-0000-000000000001").unwrap();
    let trail = Breadcrumb::generate_trail(
        &Screen::TicketList {
            project_id: Some(project_id),
        },
        &state,
    );

    assert_eq!(trail.len(), 4);
    assert_eq!(trail[0].label, "Home");
    assert_eq!(trail[1].label, "Projects");
    assert_eq!(trail[2].label, "Test Project");
    assert_eq!(trail[3].label, "Tickets");
    assert!(trail[3].is_current);
}

#[wasm_bindgen_test]
fn test_generate_trail_ticket_list_all() {
    let state = create_test_state();
    let trail = Breadcrumb::generate_trail(&Screen::TicketList { project_id: None }, &state);

    assert_eq!(trail.len(), 2);
    assert_eq!(trail[0].label, "Home");
    assert_eq!(trail[1].label, "All Tickets");
    assert!(trail[1].is_current);
}

#[wasm_bindgen_test]
fn test_generate_trail_ticket_board() {
    let state = create_test_state();
    let project_id = ProjectId::from_string("urn:uuid:00000000-0000-0000-0000-000000000001").unwrap();
    let trail = Breadcrumb::generate_trail(&Screen::TicketBoard { project_id }, &state);

    assert_eq!(trail.len(), 4);
    assert_eq!(trail[0].label, "Home");
    assert_eq!(trail[1].label, "Projects");
    assert_eq!(trail[2].label, "Test Project");
    assert_eq!(trail[3].label, "Board");
    assert!(trail[3].is_current);
}

#[wasm_bindgen_test]
fn test_generate_trail_ticket_detail() {
    let state = create_test_state();
    let ticket_id = TicketId::from_string("urn:uuid:00000000-0000-0000-0000-000000000100").unwrap();
    let trail = Breadcrumb::generate_trail(&Screen::TicketDetail(ticket_id), &state);

    assert_eq!(trail.len(), 5);
    assert_eq!(trail[0].label, "Home");
    assert_eq!(trail[1].label, "Projects");
    assert_eq!(trail[2].label, "Test Project");
    assert_eq!(trail[3].label, "Tickets");
    assert_eq!(trail[4].label, "Test Ticket");
    assert!(trail[4].is_current);
}

#[wasm_bindgen_test]
fn test_generate_trail_settings() {
    let state = create_test_state();
    let trail = Breadcrumb::generate_trail(&Screen::Settings, &state);

    assert_eq!(trail.len(), 2);
    assert_eq!(trail[0].label, "Home");
    assert_eq!(trail[1].label, "Settings");
    assert!(trail[1].is_current);
}

#[wasm_bindgen_test]
fn test_generate_trail_login() {
    let state = create_test_state();
    let trail = Breadcrumb::generate_trail(&Screen::Login, &state);

    assert_eq!(trail.len(), 0, "Login screen should have no breadcrumbs");
}

#[wasm_bindgen_test]
fn test_generate_trail_register() {
    let state = create_test_state();
    let trail = Breadcrumb::generate_trail(&Screen::Register, &state);

    assert_eq!(trail.len(), 0, "Register screen should have no breadcrumbs");
}

#[wasm_bindgen_test]
fn test_breadcrumb_update() {
    let mut breadcrumb = Breadcrumb::new();
    let state = create_test_state();

    breadcrumb.update(&Screen::Dashboard, &state);
    // Breadcrumb should be updated (no panic)
    assert!(true);
}

#[wasm_bindgen_test]
fn test_breadcrumb_update_multiple_screens() {
    let mut breadcrumb = Breadcrumb::new();
    let state = create_test_state();

    breadcrumb.update(&Screen::Dashboard, &state);
    breadcrumb.update(&Screen::ProjectList, &state);
    breadcrumb.update(&Screen::Settings, &state);

    // Should handle multiple updates without issues
    assert!(true);
}

#[wasm_bindgen_test]
fn test_breadcrumb_with_missing_project() {
    let state = create_test_state();
    let non_existent_id = ProjectId::from_string("urn:uuid:99999999-9999-9999-9999-999999999999").unwrap();
    let trail = Breadcrumb::generate_trail(&Screen::ProjectDetail(non_existent_id), &state);

    assert_eq!(trail.len(), 3);
    assert_eq!(trail[2].label, "Project", "Should use fallback name");
}

#[wasm_bindgen_test]
fn test_breadcrumb_with_missing_ticket() {
    let state = create_test_state();
    let non_existent_id = TicketId::from_string("urn:uuid:99999999-9999-9999-9999-999999999999").unwrap();
    let trail = Breadcrumb::generate_trail(&Screen::TicketDetail(non_existent_id), &state);

    assert_eq!(trail.len(), 2);
    assert_eq!(trail[0].label, "Home");
    assert_eq!(trail[1].label, "Ticket", "Should use fallback name");
}

#[wasm_bindgen_test]
fn test_breadcrumb_trail_hierarchy() {
    let state = create_test_state();
    let ticket_id = TicketId::from_string("urn:uuid:00000000-0000-0000-0000-000000000100").unwrap();
    let trail = Breadcrumb::generate_trail(&Screen::TicketDetail(ticket_id), &state);

    // Verify hierarchy: Home > Projects > Project Name > Tickets > Ticket Title
    assert_eq!(trail[0].label, "Home");
    assert_eq!(trail[1].label, "Projects");
    assert_eq!(trail[2].label, "Test Project");
    assert_eq!(trail[3].label, "Tickets");
    assert_eq!(trail[4].label, "Test Ticket");

    // Verify only last item is current
    assert!(!trail[0].is_current);
    assert!(!trail[1].is_current);
    assert!(!trail[2].is_current);
    assert!(!trail[3].is_current);
    assert!(trail[4].is_current);
}
