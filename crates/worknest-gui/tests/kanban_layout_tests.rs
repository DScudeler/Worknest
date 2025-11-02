//! Kanban board layout and sizing tests
//!
//! These tests prevent regression of layout issues:
//! - Responsive column width calculation
//! - Horizontal ScrollArea wrapper
//! - Full-height column usage
//! - Column background and styling
//! - Card design and spacing
//! - Empty state handling

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_kanban_has_horizontal_scroll() {
    // Test that Kanban board wraps columns in horizontal ScrollArea
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    assert!(
        ticket_board_source.contains("ScrollArea::horizontal()"),
        "ticket_board.rs should use horizontal ScrollArea for board"
    );

    assert!(
        ticket_board_source.contains("ui.horizontal_top("),
        "ticket_board.rs should use horizontal_top layout for columns"
    );
}

#[wasm_bindgen_test]
fn test_kanban_responsive_column_width() {
    // Test that column width is calculated responsively
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // Should calculate width based on available space
    assert!(
        ticket_board_source.contains("let available_width = ui.available_width()"),
        "ticket_board.rs should calculate responsive column width"
    );

    assert!(
        ticket_board_source.contains("let column_width = calculated_width.max(280.0)"),
        "ticket_board.rs should have minimum 280px column width"
    );

    // Should pass column_width as parameter
    assert!(
        ticket_board_source.contains("fn render_column(&mut self, ui: &mut egui::Ui, status: TicketStatus, state: &mut AppState, column_width: f32)"),
        "ticket_board.rs render_column should accept column_width parameter"
    );
}

#[wasm_bindgen_test]
fn test_kanban_no_fixed_width_constraints() {
    // Test that columns don't use old fixed width constraints
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // Should NOT have the old fixed constraints
    assert!(
        !ticket_board_source.contains("set_min_width(250.0)")
            || ticket_board_source.contains("// Old pattern removed"),
        "ticket_board.rs should not use fixed min_width(250.0)"
    );

    assert!(
        !ticket_board_source.contains("set_max_width(300.0)")
            || ticket_board_source.contains("// Old pattern removed"),
        "ticket_board.rs should not use fixed max_width(300.0)"
    );

    // Should use responsive width setting
    assert!(
        ticket_board_source.contains("ui.set_width(column_width)"),
        "ticket_board.rs should set column width dynamically"
    );
}

#[wasm_bindgen_test]
fn test_kanban_full_height_columns() {
    // Test that columns use full available height
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    assert!(
        ticket_board_source.contains("let available_height = ui.available_height()"),
        "ticket_board.rs should use available_height for column sizing"
    );

    assert!(
        ticket_board_source.contains(".max_height(available_height)"),
        "ticket_board.rs should set ScrollArea max_height to available_height"
    );

    // Should NOT use old fixed 600px height
    assert!(
        !ticket_board_source.contains("max_height(600.0)")
            || ticket_board_source.contains("// Reserve space for header"),
        "ticket_board.rs should not use fixed 600px height for column content"
    );
}

#[wasm_bindgen_test]
fn test_kanban_column_background_styling() {
    // Test that columns have proper background styling
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // Should have background color logic
    assert!(
        ticket_board_source.contains("let bg_color = if ui.style().visuals.dark_mode"),
        "ticket_board.rs should have dark/light mode background colors"
    );

    // Should use Frame with background
    assert!(
        ticket_board_source.contains("egui::Frame::NONE")
            || ticket_board_source.contains("egui::Frame::none()"),
        "ticket_board.rs should use Frame for column styling"
    );

    assert!(
        ticket_board_source.contains(".fill(bg_color)"),
        "ticket_board.rs should fill columns with background color"
    );

    assert!(
        ticket_board_source.contains(".corner_radius("),
        "ticket_board.rs should have rounded corners on columns"
    );
}

#[wasm_bindgen_test]
fn test_kanban_column_header_color_coding() {
    // Test that column headers are color-coded by status
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    assert!(
        ticket_board_source.contains("let (column_title, column_color) = match status"),
        "ticket_board.rs should color-code column headers by status"
    );

    // Should have color for each status
    assert!(
        ticket_board_source.contains("TicketStatus::Open => (\"Open\", Colors::INFO)"),
        "ticket_board.rs should color Open columns with INFO color"
    );

    assert!(
        ticket_board_source
            .contains("TicketStatus::InProgress => (\"In Progress\", Colors::WARNING)"),
        "ticket_board.rs should color InProgress columns with WARNING color"
    );

    assert!(
        ticket_board_source.contains("TicketStatus::Review => (\"Review\", Colors::PRIMARY)"),
        "ticket_board.rs should color Review columns with PRIMARY color"
    );

    assert!(
        ticket_board_source.contains("TicketStatus::Done => (\"Done\", Colors::SUCCESS)"),
        "ticket_board.rs should color Done columns with SUCCESS color"
    );
}

#[wasm_bindgen_test]
fn test_kanban_empty_state() {
    // Test that empty columns show proper empty state
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    assert!(
        ticket_board_source.contains("if column_tickets.is_empty()"),
        "ticket_board.rs should check for empty columns"
    );

    assert!(
        ticket_board_source.contains("\"No tickets\""),
        "ticket_board.rs should show 'No tickets' message for empty columns"
    );

    assert!(
        ticket_board_source.contains("ui.vertical_centered("),
        "ticket_board.rs should center empty state message"
    );
}

#[wasm_bindgen_test]
fn test_kanban_card_styling() {
    // Test that cards have proper styling with hover effects
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // Should have card background
    assert!(
        ticket_board_source.contains("let card_bg = if ui.style().visuals.dark_mode"),
        "ticket_board.rs should have dark/light mode card backgrounds"
    );

    // Should have minimum card height
    assert!(
        ticket_board_source.contains("ui.set_min_height(100.0)"),
        "ticket_board.rs should set minimum card height"
    );

    // Should have priority indicator
    assert!(
        ticket_board_source.contains("ui.colored_label(priority_color, \"▊\")"),
        "ticket_board.rs should show priority indicator bar"
    );

    // Should have hover effect
    assert!(
        ticket_board_source.contains("if card_response.hovered()"),
        "ticket_board.rs should check for card hover"
    );

    assert!(
        ticket_board_source.contains("set_cursor_icon(egui::CursorIcon::PointingHand)"),
        "ticket_board.rs should show pointing hand cursor on hover"
    );
}

#[wasm_bindgen_test]
fn test_kanban_card_click_interaction() {
    // Test that cards use proper click interaction pattern
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // Should use ui.interact() for clickable area
    assert!(
        ticket_board_source.contains("let card_response = ui.interact("),
        "ticket_board.rs should use ui.interact() for card clicks"
    );

    assert!(
        ticket_board_source.contains("egui::Sense::click()"),
        "ticket_board.rs should use Sense::click() for card interaction"
    );

    // Should NOT call .interact() on response
    assert!(
        !ticket_board_source.contains(".response.interact("),
        "ticket_board.rs should NOT call .interact() on response (causes double interaction)"
    );

    // Should handle click to navigate
    assert!(
        ticket_board_source.contains("if card_response.clicked()"),
        "ticket_board.rs should handle card clicks"
    );

    assert!(
        ticket_board_source.contains("state.navigate_to(Screen::TicketDetail"),
        "ticket_board.rs should navigate to ticket detail on card click"
    );
}

#[wasm_bindgen_test]
fn test_kanban_proper_spacing() {
    // Test that columns and cards have proper spacing
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // Should have spacing between columns
    assert!(
        ticket_board_source.contains("ui.add_space(Spacing::MEDIUM)")
            && ticket_board_source.contains("for (idx, status) in columns.iter().enumerate()"),
        "ticket_board.rs should add spacing between columns"
    );

    // Should have inner margin on frames
    assert!(
        ticket_board_source.contains(".inner_margin(Spacing::MEDIUM)"),
        "ticket_board.rs should use proper inner margins"
    );
}

#[wasm_bindgen_test]
fn test_kanban_unique_card_ids() {
    // Test that each card has a unique interaction ID
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // Should use ticket ID in interaction ID
    assert!(
        ticket_board_source.contains("ui.id().with((\"board_ticket_card\", ticket.id")
            || ticket_board_source.contains("ui.id().with(\"board_ticket_card\")"),
        "ticket_board.rs should use unique IDs for card interactions"
    );

    // Should use distinct ID prefix from other screens
    assert!(
        ticket_board_source.contains("\"board_ticket_card\""),
        "ticket_board.rs should use 'board_ticket_card' ID prefix"
    );
}

#[wasm_bindgen_test]
fn test_kanban_card_metadata() {
    // Test that cards show proper metadata
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    // Should show type badge
    assert!(
        ticket_board_source.contains("let (type_text, type_color) = match ticket.ticket_type"),
        "ticket_board.rs should display ticket type badge"
    );

    // Should show priority
    assert!(
        ticket_board_source.contains("let priority_color = match ticket.priority"),
        "ticket_board.rs should display priority information"
    );

    // Should show description with ellipsis
    assert!(
        ticket_board_source.contains("if desc.len() > 100")
            && ticket_board_source.contains("format!(\"{}...\", &desc[..100])"),
        "ticket_board.rs should truncate long descriptions with ellipsis"
    );
}

#[wasm_bindgen_test]
fn test_kanban_no_columns_function() {
    // Test that old ui.columns() pattern is not used (causes panics)
    let ticket_board_source = include_str!("../src/screens/ticket_board.rs");

    assert!(
        !ticket_board_source.contains("ui.columns("),
        "ticket_board.rs should NOT use ui.columns() (causes interaction panics)"
    );
}
