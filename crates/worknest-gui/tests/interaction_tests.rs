//! Interaction tests for card click handling and dialog rendering
//!
//! These tests prevent regression of egui interaction bugs:
//! - Card click detection using ui.interact()
//! - Dialog rendering order (after CentralPanel)
//! - Button click handling with proper event propagation

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_project_card_click_interaction() {
    // Test that project cards use ui.interact() for click detection
    // This prevents the egui hit_test.rs panic from calling .interact() twice

    // Verify the pattern exists in project_list.rs
    let project_list_source = include_str!("../src/screens/project_list.rs");

    // Should have ui.interact() call with proper parameters
    assert!(
        project_list_source.contains("ui.interact(card_rect"),
        "project_list.rs should use ui.interact() for card clicks"
    );
    assert!(
        project_list_source.contains("egui::Sense::click()"),
        "project_list.rs should use Sense::click() for card interaction"
    );

    // Should NOT have the broken pattern
    assert!(
        !project_list_source.contains("group_response.response.interact("),
        "project_list.rs should NOT call .interact() on group response"
    );
}

#[wasm_bindgen_test]
fn test_dashboard_card_click_interaction() {
    // Test that dashboard cards use ui.interact() for click detection
    let dashboard_source = include_str!("../src/screens/dashboard.rs");

    // Should have ui.interact() call
    assert!(
        dashboard_source.contains("ui.interact(card_rect"),
        "dashboard.rs should use ui.interact() for card clicks"
    );
    assert!(
        dashboard_source.contains("egui::Sense::click()"),
        "dashboard.rs should use Sense::click() for card interaction"
    );

    // Should NOT have the broken pattern
    assert!(
        !dashboard_source.contains("group_response.response.interact("),
        "dashboard.rs should NOT call .interact() on group response"
    );
}

#[wasm_bindgen_test]
fn test_ticket_list_card_click_interaction() {
    // Test that ticket list cards use ui.interact() for click detection
    let ticket_list_source = include_str!("../src/screens/ticket_list.rs");

    // Should have ui.interact() call
    assert!(
        ticket_list_source.contains("ui.interact(card_rect"),
        "ticket_list.rs should use ui.interact() for card clicks"
    );
    assert!(
        ticket_list_source.contains("egui::Sense::click()"),
        "ticket_list.rs should use Sense::click() for card interaction"
    );

    // Should NOT have the broken pattern
    assert!(
        !ticket_list_source.contains("group_response.response.interact("),
        "ticket_list.rs should NOT call .interact() on group response"
    );
}

#[wasm_bindgen_test]
fn test_ticket_board_card_click_interaction() {
    // Test that ticket board cards use ui.interact() for click detection
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // Should have ui.interact() call (may be on multiple lines)
    assert!(
        ticket_board_source.contains("ui.interact(") && ticket_board_source.contains("card_rect"),
        "ticket_board.rs should use ui.interact() for card clicks"
    );
    assert!(
        ticket_board_source.contains("egui::Sense::click()"),
        "ticket_board.rs should use Sense::click() for card interaction"
    );

    // Should NOT have the broken pattern
    assert!(
        !ticket_board_source.contains("group_response.response.interact("),
        "ticket_board.rs should NOT call .interact() on group response"
    );
}

#[wasm_bindgen_test]
fn test_project_detail_dialog_rendering_order() {
    // Test that dialogs are rendered AFTER CentralPanel to ensure proper z-order
    let project_detail_source = include_str!("../src/screens/project_detail.rs");

    // Find CentralPanel position
    let central_panel_pos = project_detail_source
        .find("egui::CentralPanel::default().show(ctx")
        .expect("CentralPanel should exist");

    // Find dialog render position
    let dialog_render_pos = project_detail_source
        .find("self.render_create_ticket_dialog(ctx, state)")
        .expect("Dialog render call should exist");

    // Dialog should come AFTER CentralPanel
    assert!(
        dialog_render_pos > central_panel_pos,
        "Dialog should be rendered after CentralPanel to appear on top (z-order)"
    );
}

#[wasm_bindgen_test]
fn test_dialog_window_configuration() {
    // Test that dialog window has proper configuration
    let project_detail_source = include_str!("../src/screens/project_detail.rs");

    // Should have proper window configuration
    assert!(
        project_detail_source.contains("egui::Window::new(\"Create New Ticket\")"),
        "Dialog should use egui::Window"
    );
    assert!(
        project_detail_source.contains(".collapsible(false)"),
        "Dialog should not be collapsible"
    );
    assert!(
        project_detail_source.contains(".resizable(false)"),
        "Dialog should not be resizable"
    );
    assert!(
        project_detail_source.contains(".anchor(egui::Align2::CENTER_CENTER"),
        "Dialog should be centered"
    );
}

#[wasm_bindgen_test]
fn test_unique_interaction_ids() {
    // Test that each screen uses unique IDs for ui.interact()
    // This prevents ID collisions that can cause interaction bugs

    let project_list_source = include_str!("../src/screens/project_list.rs");
    let dashboard_source = include_str!("../src/screens/dashboard.rs");
    let ticket_list_source = include_str!("../src/screens/ticket_list.rs");
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // Each should have unique ID
    assert!(
        project_list_source.contains("ui.id().with(\"project_card\")"),
        "project_list.rs should use 'project_card' ID"
    );
    assert!(
        dashboard_source.contains("ui.id().with(\"dashboard_project_card\")"),
        "dashboard.rs should use 'dashboard_project_card' ID"
    );
    assert!(
        ticket_list_source.contains("ui.id().with(\"ticket_card\")"),
        "ticket_list.rs should use 'ticket_card' ID"
    );
    assert!(
        ticket_board_source.contains("\"board_ticket_card\""),
        "ticket_board.rs should use 'board_ticket_card' ID"
    );
}

#[wasm_bindgen_test]
fn test_hover_cursor_feedback() {
    // Test that all interactive cards show proper cursor feedback
    let screens = [
        (
            "project_list.rs",
            include_str!("../src/screens/project_list.rs"),
        ),
        ("dashboard.rs", include_str!("../src/screens/dashboard.rs")),
        (
            "ticket_list.rs",
            include_str!("../src/screens/ticket_list.rs"),
        ),
        (
            "ticket_board.rs",
            include_str!("../src/screens/ticket_board.rs"),
        ),
    ];

    for (name, source) in screens.iter() {
        assert!(
            source.contains("if card_response.hovered()"),
            "{} should check for hover state",
            name
        );
        assert!(
            source.contains("set_cursor_icon(egui::CursorIcon::PointingHand)"),
            "{} should set pointing hand cursor on hover",
            name
        );
    }
}

#[wasm_bindgen_test]
fn test_card_click_navigation() {
    // Test that cards properly handle click navigation
    let project_list_source = include_str!("../src/screens/project_list.rs");
    let dashboard_source = include_str!("../src/screens/dashboard.rs");
    let ticket_list_source = include_str!("../src/screens/ticket_list.rs");
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // All should check for card_response.clicked()
    assert!(
        project_list_source.contains("card_response.clicked()"),
        "project_list.rs should handle card clicks"
    );
    assert!(
        dashboard_source.contains("card_response.clicked()"),
        "dashboard.rs should handle card clicks"
    );
    assert!(
        ticket_list_source.contains("card_response.clicked()"),
        "ticket_list.rs should handle card clicks"
    );
    assert!(
        ticket_board_source.contains("card_response.clicked()"),
        "ticket_board.rs should handle card clicks"
    );
}

#[wasm_bindgen_test]
fn test_button_click_priority() {
    // Test that button clicks take priority over card clicks
    let project_list_source = include_str!("../src/screens/project_list.rs");

    // Should extract button clicks from group inner response
    assert!(
        project_list_source.contains("let (view_clicked, tickets_clicked, board_clicked, archive_clicked) = group_response.inner"),
        "Should extract button click states from group inner response"
    );

    // Should handle button clicks before card clicks (using if-else chain)
    assert!(
        project_list_source.contains("if view_clicked {"),
        "Should check view button click first"
    );
    assert!(
        project_list_source.contains("} else if")
            && project_list_source.contains("card_response.clicked()"),
        "Should check card click only if no button was clicked"
    );
}

#[wasm_bindgen_test]
fn test_no_double_interaction_calls() {
    // Critical test: Ensure we never call .interact() on an already-interacted Response
    // This was the root cause of the original egui hit_test.rs panic

    let screens = [
        (
            "project_list.rs",
            include_str!("../src/screens/project_list.rs"),
        ),
        ("dashboard.rs", include_str!("../src/screens/dashboard.rs")),
        (
            "ticket_list.rs",
            include_str!("../src/screens/ticket_list.rs"),
        ),
        (
            "ticket_board.rs",
            include_str!("../src/screens/ticket_board.rs"),
        ),
    ];

    for (name, source) in screens.iter() {
        // Should NOT have the broken pattern that caused the panic
        assert!(
            !source.contains(".response.interact("),
            "{} should NOT call .interact() on a response (causes double interaction)",
            name
        );

        // Should use the correct pattern instead
        assert!(
            source.contains("ui.interact(") || source.contains("ui.interact(card_rect"),
            "{} should use ui.interact() for creating interactive areas",
            name
        );
    }
}

#[wasm_bindgen_test]
fn test_dialog_visibility_flag() {
    // Test that dialog uses proper visibility flag
    let project_detail_source = include_str!("../src/screens/project_detail.rs");

    // Should check flag before rendering
    assert!(
        project_detail_source.contains("if self.show_create_ticket_dialog {"),
        "Should check visibility flag before rendering dialog"
    );

    // Button should set flag to true
    assert!(
        project_detail_source.contains("self.show_create_ticket_dialog = true"),
        "Button should set dialog visibility flag to true"
    );

    // Cancel button should set flag to false
    assert!(
        project_detail_source.contains("self.show_create_ticket_dialog = false"),
        "Cancel should set dialog visibility flag to false"
    );
}
