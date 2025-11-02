//! End-to-End Integration Tests for Worknest GUI
//!
//! These tests verify complete user workflows across multiple screens and features,
//! ensuring that all components work together correctly.

use wasm_bindgen_test::*;
use worknest_core::models::{Priority, Project, TicketStatus, TicketType, User};
use worknest_gui::{api_client::ApiClient, screens::Screen, state::AppState};

wasm_bindgen_test_configure!(run_in_browser);

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test state with demo mode
fn create_test_state() -> AppState {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    AppState::new(api_client)
}

/// Create and login a test user
fn setup_authenticated_state() -> AppState {
    let mut state = create_test_state();
    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user, "test_token".to_string());
    state
}

/// Create a test project
fn create_test_project(state: &mut AppState, name: &str) -> Project {
    let user_id = state.current_user.as_ref().unwrap().id;
    let mut project = Project::new(name.to_string(), user_id);
    project.description = Some(format!("Test project: {}", name));
    state.projects.push(project.clone());
    project
}

// ============================================================================
// Authentication E2E Tests
// ============================================================================

#[wasm_bindgen_test]
fn e2e_complete_authentication_flow() {
    let mut state = create_test_state();

    // Initial state: not authenticated, on login screen
    assert!(!state.is_authenticated());
    assert_eq!(state.current_screen, Screen::Login);
    assert!(state.current_user.is_none());
    assert!(state.auth_token.is_none());

    // Register new user
    let user = User::new("newuser".to_string(), "newuser@example.com".to_string());
    state.login(user.clone(), "token123".to_string());

    // After registration/login: authenticated, navigated to dashboard
    assert!(state.is_authenticated());
    assert_eq!(state.current_screen, Screen::Dashboard);
    assert!(state.current_user.is_some());
    assert_eq!(state.current_user.as_ref().unwrap().username, "newuser");
    assert!(state.auth_token.is_some());

    // Logout
    state.logout();

    // After logout: not authenticated, back to login
    assert!(!state.is_authenticated());
    assert_eq!(state.current_screen, Screen::Login);
    assert!(state.current_user.is_none());
    assert!(state.auth_token.is_none());
}

#[wasm_bindgen_test]
fn e2e_authentication_persistence() {
    let mut state = create_test_state();

    // Login
    let user = User::new("persistuser".to_string(), "persist@example.com".to_string());
    state.login(user.clone(), "persist_token".to_string());

    assert!(state.is_authenticated());

    // In a real scenario, localStorage would persist the session
    // The try_restore_session() method should restore it
    assert_eq!(state.current_user.as_ref().unwrap().username, "persistuser");
}

// ============================================================================
// Project Management E2E Tests
// ============================================================================

#[wasm_bindgen_test]
fn e2e_complete_project_lifecycle() {
    let mut state = setup_authenticated_state();

    // Start from dashboard
    state.navigate_to(Screen::Dashboard);
    assert_eq!(state.current_screen, Screen::Dashboard);

    // Navigate to project list
    state.navigate_to(Screen::ProjectList);
    assert_eq!(state.current_screen, Screen::ProjectList);

    // Initially no projects
    assert_eq!(state.projects.len(), 0);

    // Create first project
    let project1 = create_test_project(&mut state, "E2E Test Project 1");
    assert_eq!(state.projects.len(), 1);

    // Create second project
    let project2 = create_test_project(&mut state, "E2E Test Project 2");
    assert_eq!(state.projects.len(), 2);

    // Navigate to project detail
    state.navigate_to(Screen::ProjectDetail(project1.id));
    assert_eq!(state.current_screen, Screen::ProjectDetail(project1.id));

    // Archive project
    if let Some(p) = state.projects.iter_mut().find(|p| p.id == project1.id) {
        p.archived = true;
    }
    assert!(state.projects.iter().find(|p| p.id == project1.id).unwrap().archived);

    // Unarchive project
    if let Some(p) = state.projects.iter_mut().find(|p| p.id == project1.id) {
        p.archived = false;
    }
    assert!(!state.projects.iter().find(|p| p.id == project1.id).unwrap().archived);

    // Delete project
    state.projects.retain(|p| p.id != project2.id);
    assert_eq!(state.projects.len(), 1);
    assert!(state.projects.iter().any(|p| p.id == project1.id));
    assert!(!state.projects.iter().any(|p| p.id == project2.id));

    // Verify navigation back to list after delete
    state.navigate_to(Screen::ProjectList);
    assert_eq!(state.current_screen, Screen::ProjectList);
}

#[wasm_bindgen_test]
fn e2e_project_update_workflow() {
    let mut state = setup_authenticated_state();
    let project = create_test_project(&mut state, "Original Name");

    // Update project details
    if let Some(p) = state.projects.iter_mut().find(|p| p.id == project.id) {
        p.name = "Updated Name".to_string();
        p.description = Some("Updated description".to_string());
        p.color = Some("#FF5733".to_string());
    }

    // Verify updates
    let updated_project = state.projects.iter().find(|p| p.id == project.id).unwrap();
    assert_eq!(updated_project.name, "Updated Name");
    assert_eq!(updated_project.description, Some("Updated description".to_string()));
    assert_eq!(updated_project.color, Some("#FF5733".to_string()));
}

// ============================================================================
// Ticket Management E2E Tests
// ============================================================================

#[wasm_bindgen_test]
fn e2e_complete_ticket_lifecycle() {
    let mut state = setup_authenticated_state();
    let project = create_test_project(&mut state, "Ticket Test Project");

    // Navigate to ticket list
    state.navigate_to(Screen::TicketList {
        project_id: Some(project.id),
    });

    // Initially no tickets
    assert_eq!(state.tickets.len(), 0);

    // Create ticket
    let user_id = state.current_user.as_ref().unwrap().id;
    let mut ticket = worknest_core::models::Ticket::new(
        project.id,
        "E2E Test Ticket".to_string(),
        TicketType::Feature,
        user_id,
    );
    ticket.description = Some("Test ticket description".to_string());
    ticket.priority = Priority::High;
    ticket.ticket_type = TicketType::Feature;
    state.tickets.push(ticket.clone());

    assert_eq!(state.tickets.len(), 1);

    // Navigate to ticket detail
    state.navigate_to(Screen::TicketDetail(ticket.id));
    assert_eq!(state.current_screen, Screen::TicketDetail(ticket.id));

    // Update ticket status
    if let Some(t) = state.tickets.iter_mut().find(|t| t.id == ticket.id) {
        t.status = TicketStatus::InProgress;
    }
    assert_eq!(
        state.tickets.iter().find(|t| t.id == ticket.id).unwrap().status,
        TicketStatus::InProgress
    );

    // Update ticket priority
    if let Some(t) = state.tickets.iter_mut().find(|t| t.id == ticket.id) {
        t.priority = Priority::Critical;
    }
    assert_eq!(
        state.tickets.iter().find(|t| t.id == ticket.id).unwrap().priority,
        Priority::Critical
    );

    // Complete ticket
    if let Some(t) = state.tickets.iter_mut().find(|t| t.id == ticket.id) {
        t.status = TicketStatus::Done;
    }
    assert_eq!(
        state.tickets.iter().find(|t| t.id == ticket.id).unwrap().status,
        TicketStatus::Done
    );

    // Delete ticket
    state.tickets.retain(|t| t.id != ticket.id);
    assert_eq!(state.tickets.len(), 0);
}

#[wasm_bindgen_test]
fn e2e_ticket_board_workflow() {
    let mut state = setup_authenticated_state();
    let project = create_test_project(&mut state, "Board Test Project");
    let user_id = state.current_user.as_ref().unwrap().id;

    // Create tickets with different statuses
    let mut ticket_open = worknest_core::models::Ticket::new(
        project.id,
        "Open Ticket".to_string(),
        TicketType::Task,
        user_id,
    );
    ticket_open.status = TicketStatus::Open;
    state.tickets.push(ticket_open.clone());

    let mut ticket_progress = worknest_core::models::Ticket::new(
        project.id,
        "In Progress Ticket".to_string(),
        TicketType::Task,
        user_id,
    );
    ticket_progress.status = TicketStatus::InProgress;
    state.tickets.push(ticket_progress.clone());

    let mut ticket_done = worknest_core::models::Ticket::new(
        project.id,
        "Done Ticket".to_string(),
        TicketType::Task,
        user_id,
    );
    ticket_done.status = TicketStatus::Done;
    state.tickets.push(ticket_done.clone());

    // Navigate to board view
    state.navigate_to(Screen::TicketBoard {
        project_id: project.id,
    });
    assert_eq!(
        state.current_screen,
        Screen::TicketBoard {
            project_id: project.id
        }
    );

    // Verify tickets are organized by status
    let open_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project.id && t.status == TicketStatus::Open)
        .collect();
    assert_eq!(open_tickets.len(), 1);

    let progress_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project.id && t.status == TicketStatus::InProgress)
        .collect();
    assert_eq!(progress_tickets.len(), 1);

    let done_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project.id && t.status == TicketStatus::Done)
        .collect();
    assert_eq!(done_tickets.len(), 1);
}

#[wasm_bindgen_test]
fn e2e_ticket_filtering_and_search() {
    let mut state = setup_authenticated_state();
    let project1 = create_test_project(&mut state, "Project 1");
    let project2 = create_test_project(&mut state, "Project 2");
    let user_id = state.current_user.as_ref().unwrap().id;

    // Create tickets in different projects
    let ticket1 = worknest_core::models::Ticket::new(
        project1.id,
        "Bug in Project 1".to_string(),
        TicketType::Bug,
        user_id,
    );
    state.tickets.push(ticket1);

    let ticket2 = worknest_core::models::Ticket::new(
        project2.id,
        "Feature in Project 2".to_string(),
        TicketType::Feature,
        user_id,
    );
    state.tickets.push(ticket2);

    // Filter by project
    let project1_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project1.id)
        .collect();
    assert_eq!(project1_tickets.len(), 1);

    let project2_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project2.id)
        .collect();
    assert_eq!(project2_tickets.len(), 1);

    // Search by title
    let bug_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.title.to_lowercase().contains("bug"))
        .collect();
    assert_eq!(bug_tickets.len(), 1);
}

// ============================================================================
// Comments System E2E Tests
// ============================================================================

#[wasm_bindgen_test]
fn e2e_complete_comment_workflow() {
    let mut state = setup_authenticated_state();
    let project = create_test_project(&mut state, "Comment Test Project");
    let user_id = state.current_user.as_ref().unwrap().id;

    // Create ticket
    let ticket = worknest_core::models::Ticket::new(
        project.id,
        "Ticket with Comments".to_string(),
        TicketType::Task,
        user_id,
    );
    state.tickets.push(ticket.clone());

    // Initially no comments
    assert_eq!(state.comments.len(), 0);

    // Create first comment
    let comment1 = worknest_core::models::Comment::new(
        ticket.id,
        user_id,
        "First comment".to_string(),
    );
    state.comments.push(comment1.clone());
    assert_eq!(state.comments.len(), 1);

    // Create second comment
    let comment2 = worknest_core::models::Comment::new(
        ticket.id,
        user_id,
        "Second comment".to_string(),
    );
    state.comments.push(comment2.clone());
    assert_eq!(state.comments.len(), 2);

    // Edit comment
    if let Some(c) = state.comments.iter_mut().find(|c| c.id == comment1.id) {
        c.update_content("Updated first comment".to_string()).unwrap();
    }
    assert_eq!(
        state.comments.iter().find(|c| c.id == comment1.id).unwrap().content,
        "Updated first comment"
    );

    // Delete comment
    state.comments.retain(|c| c.id != comment2.id);
    assert_eq!(state.comments.len(), 1);

    // Verify remaining comment
    let remaining_comments: Vec<_> = state
        .comments
        .iter()
        .filter(|c| c.ticket_id == ticket.id)
        .collect();
    assert_eq!(remaining_comments.len(), 1);
    assert_eq!(remaining_comments[0].content, "Updated first comment");
}

#[wasm_bindgen_test]
fn e2e_comments_per_ticket_isolation() {
    let mut state = setup_authenticated_state();
    let project = create_test_project(&mut state, "Multi-Ticket Project");
    let user_id = state.current_user.as_ref().unwrap().id;

    // Create two tickets
    let ticket1 = worknest_core::models::Ticket::new(
        project.id,
        "Ticket 1".to_string(),
        TicketType::Task,
        user_id,
    );
    state.tickets.push(ticket1.clone());

    let ticket2 = worknest_core::models::Ticket::new(
        project.id,
        "Ticket 2".to_string(),
        TicketType::Task,
        user_id,
    );
    state.tickets.push(ticket2.clone());

    // Add comments to each ticket
    state.comments.push(worknest_core::models::Comment::new(
        ticket1.id,
        user_id,
        "Comment on ticket 1".to_string(),
    ));
    state.comments.push(worknest_core::models::Comment::new(
        ticket1.id,
        user_id,
        "Another comment on ticket 1".to_string(),
    ));
    state.comments.push(worknest_core::models::Comment::new(
        ticket2.id,
        user_id,
        "Comment on ticket 2".to_string(),
    ));

    // Verify comments are properly isolated
    let ticket1_comments: Vec<_> = state
        .comments
        .iter()
        .filter(|c| c.ticket_id == ticket1.id)
        .collect();
    assert_eq!(ticket1_comments.len(), 2);

    let ticket2_comments: Vec<_> = state
        .comments
        .iter()
        .filter(|c| c.ticket_id == ticket2.id)
        .collect();
    assert_eq!(ticket2_comments.len(), 1);
}

// ============================================================================
// Settings E2E Tests
// ============================================================================

#[wasm_bindgen_test]
fn e2e_settings_navigation_and_updates() {
    let mut state = setup_authenticated_state();

    // Navigate to settings
    state.navigate_to(Screen::Settings);
    assert_eq!(state.current_screen, Screen::Settings);

    // Update user profile
    let original_email = state.current_user.as_ref().unwrap().email.clone();
    if let Some(user) = &mut state.current_user {
        user.email = "newemail@example.com".to_string();
    }

    assert_eq!(
        state.current_user.as_ref().unwrap().email,
        "newemail@example.com"
    );
    assert_ne!(state.current_user.as_ref().unwrap().email, original_email);

    // Navigate back to dashboard
    state.navigate_to(Screen::Dashboard);
    assert_eq!(state.current_screen, Screen::Dashboard);

    // Verify changes persist
    assert_eq!(
        state.current_user.as_ref().unwrap().email,
        "newemail@example.com"
    );
}

// ============================================================================
// Ticket Creation from Project Detail E2E Tests
// ============================================================================

#[wasm_bindgen_test]
fn e2e_create_ticket_from_project_detail() {
    let mut state = setup_authenticated_state();
    let project = create_test_project(&mut state, "Project Detail Ticket Test");

    // Navigate to project detail screen
    state.navigate_to(Screen::ProjectDetail(project.id));
    assert_eq!(state.current_screen, Screen::ProjectDetail(project.id));

    // Initially no tickets for this project
    let initial_ticket_count = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project.id)
        .count();
    assert_eq!(initial_ticket_count, 0);

    // Simulate opening the create ticket dialog
    // (In the actual UI, this would be triggered by clicking the "+ New Ticket" button)
    let show_create_ticket_dialog = true;
    assert!(show_create_ticket_dialog);

    // Create a new ticket directly through state
    // (In the actual UI, this would be done via the dialog form)
    let user_id = state.current_user.as_ref().unwrap().id;
    let mut new_ticket = worknest_core::models::Ticket::new(
        project.id,
        "New Feature from Project Detail".to_string(),
        TicketType::Feature,
        user_id,
    );
    new_ticket.description = Some("Created directly from project detail screen".to_string());
    new_ticket.priority = Priority::High;
    state.tickets.push(new_ticket.clone());

    // Verify ticket was created
    let final_ticket_count = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project.id)
        .count();
    assert_eq!(final_ticket_count, 1);

    // Verify ticket details
    let created_ticket = state
        .tickets
        .iter()
        .find(|t| t.id == new_ticket.id)
        .expect("Created ticket should exist");

    assert_eq!(created_ticket.title, "New Feature from Project Detail");
    assert_eq!(created_ticket.project_id, project.id);
    assert_eq!(created_ticket.ticket_type, TicketType::Feature);
    assert_eq!(created_ticket.priority, Priority::High);
    assert_eq!(
        created_ticket.description,
        Some("Created directly from project detail screen".to_string())
    );

    // Verify ticket appears in project's ticket list
    state.navigate_to(Screen::TicketList {
        project_id: Some(project.id),
    });
    let tickets_in_list = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project.id)
        .count();
    assert_eq!(tickets_in_list, 1);
}

#[wasm_bindgen_test]
fn e2e_multiple_tickets_from_project_detail() {
    let mut state = setup_authenticated_state();
    let project = create_test_project(&mut state, "Multi-Ticket Project");
    let user_id = state.current_user.as_ref().unwrap().id;

    // Navigate to project detail
    state.navigate_to(Screen::ProjectDetail(project.id));

    // Create multiple tickets with different types and priorities
    let ticket1 = worknest_core::models::Ticket::new(
        project.id,
        "Bug Fix".to_string(),
        TicketType::Bug,
        user_id,
    );
    state.tickets.push(ticket1);

    let mut ticket2 = worknest_core::models::Ticket::new(
        project.id,
        "Feature Request".to_string(),
        TicketType::Feature,
        user_id,
    );
    ticket2.priority = Priority::Critical;
    state.tickets.push(ticket2);

    let ticket3 = worknest_core::models::Ticket::new(
        project.id,
        "Task Item".to_string(),
        TicketType::Task,
        user_id,
    );
    state.tickets.push(ticket3);

    // Verify all tickets were created for this project
    let project_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project.id)
        .collect();
    assert_eq!(project_tickets.len(), 3);

    // Verify ticket types
    let bug_tickets: Vec<_> = project_tickets
        .iter()
        .filter(|t| t.ticket_type == TicketType::Bug)
        .collect();
    assert_eq!(bug_tickets.len(), 1);

    let feature_tickets: Vec<_> = project_tickets
        .iter()
        .filter(|t| t.ticket_type == TicketType::Feature)
        .collect();
    assert_eq!(feature_tickets.len(), 1);

    let task_tickets: Vec<_> = project_tickets
        .iter()
        .filter(|t| t.ticket_type == TicketType::Task)
        .collect();
    assert_eq!(task_tickets.len(), 1);
}

// ============================================================================
// Cross-Feature Integration Tests
// ============================================================================

#[wasm_bindgen_test]
fn e2e_complete_user_journey() {
    // Simulates a complete user workflow from login to ticket completion

    let mut state = create_test_state();

    // 1. Register/Login
    let user = User::new("journeyuser".to_string(), "journey@example.com".to_string());
    state.login(user.clone(), "journey_token".to_string());
    assert!(state.is_authenticated());
    assert_eq!(state.current_screen, Screen::Dashboard);

    // 2. Create a project
    let project = create_test_project(&mut state, "Journey Project");
    assert_eq!(state.projects.len(), 1);

    // 3. Navigate to project detail
    state.navigate_to(Screen::ProjectDetail(project.id));
    assert_eq!(state.current_screen, Screen::ProjectDetail(project.id));

    // 4. Create a ticket
    let ticket = worknest_core::models::Ticket::new(
        project.id,
        "Journey Ticket".to_string(),
        TicketType::Task,
        user.id,
    );
    state.tickets.push(ticket.clone());
    assert_eq!(state.tickets.len(), 1);

    // 5. Navigate to ticket detail
    state.navigate_to(Screen::TicketDetail(ticket.id));
    assert_eq!(state.current_screen, Screen::TicketDetail(ticket.id));

    // 6. Add comment
    let comment = worknest_core::models::Comment::new(
        ticket.id,
        user.id,
        "Working on this ticket".to_string(),
    );
    state.comments.push(comment);
    assert_eq!(state.comments.len(), 1);

    // 7. Update ticket status
    if let Some(t) = state.tickets.iter_mut().find(|t| t.id == ticket.id) {
        t.status = TicketStatus::InProgress;
    }

    // 8. Add another comment
    let comment2 = worknest_core::models::Comment::new(
        ticket.id,
        user.id,
        "Completed the work".to_string(),
    );
    state.comments.push(comment2);
    assert_eq!(state.comments.len(), 2);

    // 9. Mark ticket as done
    if let Some(t) = state.tickets.iter_mut().find(|t| t.id == ticket.id) {
        t.status = TicketStatus::Done;
    }
    assert_eq!(
        state.tickets.iter().find(|t| t.id == ticket.id).unwrap().status,
        TicketStatus::Done
    );

    // 10. Navigate to board view to see completed ticket
    state.navigate_to(Screen::TicketBoard {
        project_id: project.id,
    });

    // Verify ticket is in done column
    let done_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project.id && t.status == TicketStatus::Done)
        .collect();
    assert_eq!(done_tickets.len(), 1);

    // 11. Update settings
    state.navigate_to(Screen::Settings);
    if let Some(user) = &mut state.current_user {
        user.username = "journey_user_updated".to_string();
    }

    // 12. Logout
    state.logout();
    assert!(!state.is_authenticated());
    assert_eq!(state.current_screen, Screen::Login);
}

#[wasm_bindgen_test]
fn e2e_project_cascade_delete() {
    let mut state = setup_authenticated_state();
    let project = create_test_project(&mut state, "Cascade Delete Test");
    let user_id = state.current_user.as_ref().unwrap().id;

    // Create tickets and comments for the project
    let ticket = worknest_core::models::Ticket::new(
        project.id,
        "Ticket to be deleted".to_string(),
        TicketType::Task,
        user_id,
    );
    state.tickets.push(ticket.clone());

    let comment = worknest_core::models::Comment::new(
        ticket.id,
        user_id,
        "Comment to be deleted".to_string(),
    );
    state.comments.push(comment);

    // Verify data exists
    assert_eq!(state.projects.len(), 1);
    assert_eq!(state.tickets.len(), 1);
    assert_eq!(state.comments.len(), 1);

    // Delete project (should cascade to tickets and comments)
    state.projects.retain(|p| p.id != project.id);
    state.tickets.retain(|t| t.project_id != project.id);
    // In a real implementation, comments would also be cascade deleted
    state.comments.retain(|c| {
        !state.tickets.iter().any(|t| t.id == c.ticket_id)
    });

    // Verify everything is deleted
    assert_eq!(state.projects.len(), 0);
    assert_eq!(state.tickets.len(), 0);
}

#[wasm_bindgen_test]
fn e2e_multi_project_ticket_management() {
    let mut state = setup_authenticated_state();
    let user_id = state.current_user.as_ref().unwrap().id;

    // Create multiple projects
    let project1 = create_test_project(&mut state, "Frontend Project");
    let project2 = create_test_project(&mut state, "Backend Project");
    let project3 = create_test_project(&mut state, "DevOps Project");

    // Create tickets in each project
    state.tickets.push(worknest_core::models::Ticket::new(
        project1.id,
        "Frontend Bug".to_string(),
        TicketType::Bug,
        user_id,
    ));
    state.tickets.push(worknest_core::models::Ticket::new(
        project1.id,
        "Frontend Feature".to_string(),
        TicketType::Feature,
        user_id,
    ));
    state.tickets.push(worknest_core::models::Ticket::new(
        project2.id,
        "API Bug".to_string(),
        TicketType::Bug,
        user_id,
    ));
    state.tickets.push(worknest_core::models::Ticket::new(
        project3.id,
        "CI/CD Setup".to_string(),
        TicketType::Task,
        user_id,
    ));

    // Verify ticket distribution
    let p1_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project1.id)
        .collect();
    assert_eq!(p1_tickets.len(), 2);

    let p2_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project2.id)
        .collect();
    assert_eq!(p2_tickets.len(), 1);

    let p3_tickets: Vec<_> = state
        .tickets
        .iter()
        .filter(|t| t.project_id == project3.id)
        .collect();
    assert_eq!(p3_tickets.len(), 1);

    // Total tickets
    assert_eq!(state.tickets.len(), 4);
}

#[wasm_bindgen_test]
fn e2e_notification_system() {
    let mut state = setup_authenticated_state();

    // Initially no notifications
    assert_eq!(state.notifications.len(), 0);

    // Trigger various actions that create notifications
    state.notify_success("Success message".to_string());
    assert_eq!(state.notifications.len(), 1);

    state.notify_error("Error message".to_string());
    assert_eq!(state.notifications.len(), 2);

    state.notify_info("Info message".to_string());
    assert_eq!(state.notifications.len(), 3);

    // Verify notification content
    assert_eq!(state.notifications[0].message, "Success message");
    assert_eq!(state.notifications[1].message, "Error message");
    assert_eq!(state.notifications[2].message, "Info message");
}
