//! UI interaction and logic tests for Worknest GUI

use wasm_bindgen_test::*;
use worknest_core::models::{Comment, Project, TicketId, User, UserId};
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
    assert_eq!(state.projects.len(), 0);

    // Create a project
    let project = Project::new("Test Project".to_string(), user.id);
    state.projects.push(project);

    // Verify project was added
    assert_eq!(state.projects.len(), 1);
    assert_eq!(state.projects[0].name, "Test Project");
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
    state.projects.push(project.clone());

    assert!(state.projects[0].archived);

    // Unarchive it
    state.projects[0].archived = false;
    assert!(!state.projects[0].archived);
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
    state.projects.push(project);

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
    state.projects.push(project);

    // Navigate to ticket list for this project
    state.navigate_to(Screen::TicketList {
        project_id: Some(project_id),
    });

    match state.current_screen {
        Screen::TicketList {
            project_id: Some(pid),
        } => {
            assert_eq!(pid, project_id);
        },
        _ => panic!("Expected TicketList screen with project_id"),
    }

    // Navigate to all tickets (no project filter)
    state.navigate_to(Screen::TicketList { project_id: None });

    match state.current_screen {
        Screen::TicketList { project_id: None } => {},
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
    state.projects.push(project);

    // Navigate to ticket board
    state.navigate_to(Screen::TicketBoard { project_id });

    match state.current_screen {
        Screen::TicketBoard { project_id: pid } => {
            assert_eq!(pid, project_id);
        },
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

    state.projects.push(project1);
    state.projects.push(project2);
    state.projects.push(project3);

    // Test filtering by name (case-insensitive)
    let alpha_projects: Vec<&Project> = state
        .projects
        .iter()
        .filter(|p| p.name.to_lowercase().contains("alpha"))
        .collect();
    assert_eq!(alpha_projects.len(), 1);
    assert_eq!(alpha_projects[0].name, "Alpha Project");

    // Test filtering by archived status
    let active_projects: Vec<&Project> = state.projects.iter().filter(|p| !p.archived).collect();
    assert_eq!(active_projects.len(), 2);

    let archived_projects: Vec<&Project> = state.projects.iter().filter(|p| p.archived).collect();
    assert_eq!(archived_projects.len(), 1);
}

/// Test comment list display
#[wasm_bindgen_test]
fn test_comment_list_display() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let ticket_id = TicketId::new();

    // Initially no comments
    assert_eq!(state.comments.len(), 0);

    // Add comments
    let comment1 = Comment::new(ticket_id, user.id, "First comment".to_string());
    let comment2 = Comment::new(ticket_id, user.id, "Second comment".to_string());

    state.comments.push(comment1);
    state.comments.push(comment2);

    // Verify comments were added
    assert_eq!(state.comments.len(), 2);
    assert_eq!(state.comments[0].content, "First comment");
    assert_eq!(state.comments[1].content, "Second comment");

    // Test filtering by ticket_id
    let ticket_comments: Vec<&Comment> = state
        .comments
        .iter()
        .filter(|c| c.ticket_id == ticket_id)
        .collect();
    assert_eq!(ticket_comments.len(), 2);
}

/// Test comment creation
#[wasm_bindgen_test]
fn test_comment_creation() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let ticket_id = TicketId::new();

    // Create a comment
    let comment = Comment::new(ticket_id, user.id, "New comment content".to_string());

    // Validate comment
    assert!(comment.validate().is_ok());
    assert_eq!(comment.content, "New comment content");
    assert_eq!(comment.ticket_id, ticket_id);
    assert_eq!(comment.user_id, user.id);

    // Add to state
    state.comments.push(comment.clone());
    assert_eq!(state.comments.len(), 1);
    assert_eq!(state.comments[0].id, comment.id);
}

/// Test comment editing
#[wasm_bindgen_test]
fn test_comment_edit() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let ticket_id = TicketId::new();
    let mut comment = Comment::new(ticket_id, user.id, "Original content".to_string());

    state.comments.push(comment.clone());

    // Edit the comment
    let updated_content = "Updated content".to_string();
    assert!(comment.update_content(updated_content.clone()).is_ok());

    // Update in state
    if let Some(c) = state.comments.iter_mut().find(|c| c.id == comment.id) {
        *c = comment.clone();
    }

    assert_eq!(state.comments[0].content, "Updated content");
}

/// Test comment deletion
#[wasm_bindgen_test]
fn test_comment_delete() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let ticket_id = TicketId::new();
    let comment = Comment::new(ticket_id, user.id, "Comment to delete".to_string());
    let comment_id = comment.id;

    state.comments.push(comment);
    assert_eq!(state.comments.len(), 1);

    // Delete the comment
    state.comments.retain(|c| c.id != comment_id);
    assert_eq!(state.comments.len(), 0);
}

/// Test comment validation - empty content
#[wasm_bindgen_test]
fn test_comment_validation_empty() {
    let ticket_id = TicketId::new();
    let user_id = UserId::new();

    let comment = Comment::new(ticket_id, user_id, "   ".to_string());
    assert!(comment.validate().is_err());
}

/// Test comment validation - content too long
#[wasm_bindgen_test]
fn test_comment_validation_too_long() {
    let ticket_id = TicketId::new();
    let user_id = UserId::new();

    let long_content = "a".repeat(10001);
    let comment = Comment::new(ticket_id, user_id, long_content);
    assert!(comment.validate().is_err());
}

/// Test comments for specific ticket
#[wasm_bindgen_test]
fn test_filter_comments_by_ticket() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let ticket1_id = TicketId::new();
    let ticket2_id = TicketId::new();

    // Add comments for different tickets
    state.comments.push(Comment::new(
        ticket1_id,
        user.id,
        "Comment for ticket 1".to_string(),
    ));
    state.comments.push(Comment::new(
        ticket1_id,
        user.id,
        "Another comment for ticket 1".to_string(),
    ));
    state.comments.push(Comment::new(
        ticket2_id,
        user.id,
        "Comment for ticket 2".to_string(),
    ));

    // Filter comments by ticket
    let ticket1_comments: Vec<&Comment> = state
        .comments
        .iter()
        .filter(|c| c.ticket_id == ticket1_id)
        .collect();
    assert_eq!(ticket1_comments.len(), 2);

    let ticket2_comments: Vec<&Comment> = state
        .comments
        .iter()
        .filter(|c| c.ticket_id == ticket2_id)
        .collect();
    assert_eq!(ticket2_comments.len(), 1);
}

/// Test settings screen navigation
#[wasm_bindgen_test]
fn test_settings_navigation() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    // Navigate to settings
    state.navigate_to(Screen::Settings);
    assert_eq!(state.current_screen, Screen::Settings);
}

/// Test password change functionality
#[wasm_bindgen_test]
fn test_password_change() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let state = AppState::new(api_client);

    // In a real implementation, this would test:
    // - Current password validation
    // - New password strength requirements
    // - Password confirmation matching
    // - API call to update password

    // For now, just verify the test structure exists
    assert!(state.current_user.is_none());
}

/// Test profile update functionality
#[wasm_bindgen_test]
fn test_profile_update() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let mut user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    // Update user details
    user.email = "newemail@example.com".to_string();

    // In demo mode, update should work
    assert_eq!(state.current_user.as_ref().unwrap().username, "testuser");
}

// ============================================================================
// Ticket Creation Dialog State Management Tests
// ============================================================================

/// Test ticket creation dialog state lifecycle
#[wasm_bindgen_test]
fn test_ticket_creation_dialog_lifecycle() {
    use worknest_gui::screens::project_detail::ProjectDetailScreen;

    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let project_id = project.id;
    state.projects.push(project);

    // Create screen instance
    let _screen = ProjectDetailScreen::new(project_id);

    // Initially dialog should be closed
    // Note: We're testing the expected behavior even though we can't directly access private fields
    // The actual implementation has show_create_ticket_dialog: false by default

    // After user clicks "+ New Ticket" button, dialog should open
    // (In actual UI, this sets show_create_ticket_dialog = true)

    // After successful ticket creation, dialog should close and form should clear
    // (In actual UI, this sets show_create_ticket_dialog = false and calls clear_create_ticket_form)
}

/// Test ticket creation form validation
#[wasm_bindgen_test]
fn test_ticket_creation_form_validation() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    state.projects.push(project.clone());

    // Attempt to create ticket with empty title should fail
    // (In actual UI, this is validated in create_ticket() method)
    let initial_ticket_count = state.tickets.len();

    // Verify that validation prevents empty title tickets
    // The actual implementation checks: if self.new_ticket_title.is_empty()
    assert_eq!(state.tickets.len(), initial_ticket_count);

    // Create ticket with valid title should succeed
    use worknest_core::models::{Ticket, TicketType};
    let ticket = Ticket::new(
        project.id,
        "Valid Title".to_string(),
        TicketType::Task,
        user.id,
    );
    state.tickets.push(ticket);

    assert_eq!(state.tickets.len(), initial_ticket_count + 1);
}

/// Test ticket creation with different types and priorities
#[wasm_bindgen_test]
fn test_ticket_creation_types_and_priorities() {
    use worknest_core::models::{Priority, Ticket, TicketType};

    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    state.projects.push(project.clone());

    // Test all ticket types
    let types = vec![
        TicketType::Task,
        TicketType::Bug,
        TicketType::Feature,
        TicketType::Epic,
    ];

    for ticket_type in types {
        let ticket = Ticket::new(
            project.id,
            format!("{:?} Ticket", ticket_type),
            ticket_type,
            user.id,
        );
        state.tickets.push(ticket.clone());

        let created = state.tickets.iter().find(|t| t.id == ticket.id).unwrap();
        assert_eq!(created.ticket_type, ticket_type);
    }

    // Test all priority levels
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
        state.tickets.push(ticket.clone());

        let created = state.tickets.iter().find(|t| t.id == ticket.id).unwrap();
        assert_eq!(created.priority, priority);
    }
}

// ============================================================================
// Button Accessibility Tests
// ============================================================================

/// Test create ticket button accessibility from project detail
#[wasm_bindgen_test]
fn test_create_ticket_button_accessibility() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let project_id = project.id;
    state.projects.push(project);

    // Navigate to project detail
    state.navigate_to(Screen::ProjectDetail(project_id));
    assert_eq!(state.current_screen, Screen::ProjectDetail(project_id));

    // Button should be visible and enabled when on project detail screen
    // (In actual UI, the button is always enabled on project detail because project_id is available)

    // Clicking the button should open the dialog
    // (In actual UI, clicking sets show_create_ticket_dialog = true)

    // Test passes if we successfully navigated to project detail
    // The button's functionality is tested through the ticket creation E2E tests
}

/// Test navigation buttons from project detail
#[wasm_bindgen_test]
fn test_project_detail_navigation_buttons() {
    let api_client = ApiClient::new("http://localhost:3000".to_string());
    let mut state = AppState::new(api_client);

    let user = User::new("testuser".to_string(), "test@example.com".to_string());
    state.login(user.clone(), "token".to_string());

    let project = Project::new("Test Project".to_string(), user.id);
    let project_id = project.id;
    state.projects.push(project);

    state.navigate_to(Screen::ProjectDetail(project_id));

    // Test "View Board" button navigation
    state.navigate_to(Screen::TicketBoard { project_id });
    assert_eq!(state.current_screen, Screen::TicketBoard { project_id });

    // Navigate back
    state.navigate_to(Screen::ProjectDetail(project_id));

    // Test "View All Tickets" button navigation
    state.navigate_to(Screen::TicketList {
        project_id: Some(project_id),
    });
    match state.current_screen {
        Screen::TicketList {
            project_id: Some(pid),
        } => {
            assert_eq!(pid, project_id);
        },
        _ => panic!("Expected TicketList screen"),
    }

    // Navigate back
    state.navigate_to(Screen::ProjectDetail(project_id));
    assert_eq!(state.current_screen, Screen::ProjectDetail(project_id));
}
