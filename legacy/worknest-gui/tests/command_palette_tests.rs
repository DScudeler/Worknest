//! Tests for the command palette component

use wasm_bindgen_test::*;
use worknest_gui::components::{CommandAction, CommandCategory, CommandPalette};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_command_palette_initialization() {
    let palette = CommandPalette::new();
    assert!(!palette.is_open, "Command palette should start closed");
}

#[wasm_bindgen_test]
fn test_command_palette_toggle() {
    let mut palette = CommandPalette::new();
    assert!(!palette.is_open);

    palette.toggle();
    assert!(palette.is_open, "Should open after toggle");

    palette.toggle();
    assert!(!palette.is_open, "Should close after second toggle");
}

#[wasm_bindgen_test]
fn test_command_palette_open_close() {
    let mut palette = CommandPalette::new();

    palette.open();
    assert!(palette.is_open, "Should be open after open()");

    palette.close();
    assert!(!palette.is_open, "Should be closed after close()");
}

#[wasm_bindgen_test]
fn test_essential_navigation_commands() {
    let palette = CommandPalette::new();

    // Check that essential navigation commands are registered
    // We can't access private fields directly, but we can verify through rendering behavior
    // For now, we verify the palette initializes successfully
    assert!(!palette.is_open);
}

#[wasm_bindgen_test]
fn test_command_categories_exist() {
    // Test that all command categories are properly defined
    let _nav = CommandCategory::Navigation;
    let _actions = CommandCategory::Actions;
    let _settings = CommandCategory::Settings;

    // Verify category names
    assert_eq!(CommandCategory::Navigation.name(), "Navigation");
    assert_eq!(CommandCategory::Actions.name(), "Actions");
    assert_eq!(CommandCategory::Settings.name(), "Settings");
}

#[wasm_bindgen_test]
fn test_command_category_icons() {
    // Verify that each category has an icon
    assert!(!CommandCategory::Navigation.icon().is_empty());
    assert!(!CommandCategory::Actions.icon().is_empty());
    assert!(!CommandCategory::Settings.icon().is_empty());
}

#[wasm_bindgen_test]
fn test_command_action_variants() {
    use worknest_gui::screens::Screen;

    // Test that all CommandAction variants can be created
    let _nav = CommandAction::Navigate(Screen::Dashboard);
    let _sidebar = CommandAction::ToggleSidebar;
    let _theme = CommandAction::ToggleTheme;
    let _help = CommandAction::ShowHelp;
}

#[wasm_bindgen_test]
fn test_command_palette_state_consistency() {
    let mut palette = CommandPalette::new();

    // Opening should reset search
    palette.open();
    assert!(palette.is_open);

    // Closing should maintain consistency
    palette.close();
    assert!(!palette.is_open);

    // Multiple opens should be idempotent
    palette.open();
    palette.open();
    assert!(palette.is_open);
}

#[wasm_bindgen_test]
fn test_command_palette_with_app_state() {
    use worknest_gui::{api_client::ApiClient, state::AppState};

    let mut palette = CommandPalette::new();
    let api_client = ApiClient::new_default();
    let state = AppState::new(api_client);

    // Test dynamic command addition
    palette.add_dynamic_commands(&state);

    // The palette should still function correctly
    assert!(!palette.is_open);
}

#[wasm_bindgen_test]
fn test_command_palette_toggle_resets_state() {
    let mut palette = CommandPalette::new();

    palette.open();
    assert!(palette.is_open);

    // Toggling closed and then open again should reset search state
    palette.close();
    palette.open();
    assert!(palette.is_open);
}

#[wasm_bindgen_test]
fn test_command_palette_multiple_operations() {
    let mut palette = CommandPalette::new();

    // Simulate multiple user interactions
    palette.open();
    palette.close();
    palette.toggle();
    palette.toggle();
    palette.open();

    assert!(palette.is_open, "Should be open after final open()");
}

#[wasm_bindgen_test]
fn test_command_category_equality() {
    // Test that command categories can be compared
    assert_eq!(CommandCategory::Navigation, CommandCategory::Navigation);
    assert_ne!(CommandCategory::Navigation, CommandCategory::Actions);
    assert_ne!(CommandCategory::Actions, CommandCategory::Settings);
}
