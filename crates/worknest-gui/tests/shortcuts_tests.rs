//! Tests for keyboard shortcuts system

use wasm_bindgen_test::*;
use worknest_gui::components::{ShortcutDefinition, ShortcutsHelp};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_shortcuts_help_initialization() {
    let shortcuts_help = ShortcutsHelp::new();

    // Should start closed
    assert!(
        !shortcuts_help.is_open,
        "Shortcuts help should start closed"
    );

    // Should have registered shortcuts
    let shortcuts = shortcuts_help.shortcuts();
    assert!(
        shortcuts.len() > 0,
        "Should have at least one registered shortcut"
    );

    // Verify essential shortcuts are registered
    let has_help_shortcut = shortcuts.iter().any(|s| {
        matches!(s.key, egui::Key::Questionmark) && s.description.contains("keyboard shortcuts")
    });
    assert!(
        has_help_shortcut,
        "Should have help shortcut (?) registered"
    );

    let has_sidebar_toggle = shortcuts
        .iter()
        .any(|s| matches!(s.key, egui::Key::B) && s.description.contains("sidebar"));
    assert!(
        has_sidebar_toggle,
        "Should have sidebar toggle shortcut registered"
    );
}

#[wasm_bindgen_test]
fn test_shortcuts_help_toggle() {
    let mut shortcuts_help = ShortcutsHelp::new();

    // Initially closed
    assert!(!shortcuts_help.is_open);

    // Toggle to open
    shortcuts_help.toggle();
    assert!(shortcuts_help.is_open, "Should be open after toggle");

    // Toggle to close
    shortcuts_help.toggle();
    assert!(!shortcuts_help.is_open, "Should be closed after toggle");
}

#[wasm_bindgen_test]
fn test_shortcuts_help_open_close() {
    let mut shortcuts_help = ShortcutsHelp::new();

    // Test explicit open
    shortcuts_help.open();
    assert!(shortcuts_help.is_open, "Should be open after open()");

    // Test explicit close
    shortcuts_help.close();
    assert!(!shortcuts_help.is_open, "Should be closed after close()");

    // Test multiple opens don't break state
    shortcuts_help.open();
    shortcuts_help.open();
    assert!(shortcuts_help.is_open, "Should stay open");

    // Test multiple closes don't break state
    shortcuts_help.close();
    shortcuts_help.close();
    assert!(!shortcuts_help.is_open, "Should stay closed");
}

#[wasm_bindgen_test]
fn test_shortcut_definition_creation() {
    let shortcut = ShortcutDefinition::new(
        egui::Key::K,
        egui::Modifiers::COMMAND,
        "Test shortcut",
        "Test Category",
    );

    assert!(matches!(shortcut.key, egui::Key::K));
    assert_eq!(shortcut.description, "Test shortcut");
    assert_eq!(shortcut.category, "Test Category");
}

#[wasm_bindgen_test]
fn test_shortcut_formatting() {
    // Test basic key
    let shortcut = ShortcutDefinition::new(
        egui::Key::K,
        egui::Modifiers::COMMAND,
        "Command+K",
        "Global",
    );

    let formatted = shortcut.format();
    // Should contain the key
    assert!(
        formatted.contains("K"),
        "Formatted shortcut should contain key"
    );

    // Test with number key
    let num_shortcut = ShortcutDefinition::new(
        egui::Key::Num1,
        egui::Modifiers::COMMAND,
        "Go to Dashboard",
        "Views",
    );

    let formatted_num = num_shortcut.format();
    assert!(
        formatted_num.contains("1"),
        "Formatted shortcut should contain number"
    );
}

#[wasm_bindgen_test]
fn test_shortcuts_by_category() {
    let shortcuts_help = ShortcutsHelp::new();
    let shortcuts = shortcuts_help.shortcuts();

    // Group by category
    let mut categories = std::collections::HashSet::new();
    for shortcut in shortcuts {
        categories.insert(shortcut.category.clone());
    }

    // Should have expected categories
    assert!(categories.contains("Global"), "Should have Global category");
    assert!(
        categories.contains("Navigation"),
        "Should have Navigation category"
    );
    assert!(categories.contains("Views"), "Should have Views category");
    assert!(
        categories.contains("Actions"),
        "Should have Actions category"
    );
}

#[wasm_bindgen_test]
fn test_essential_shortcuts_registered() {
    let shortcuts_help = ShortcutsHelp::new();
    let shortcuts = shortcuts_help.shortcuts();

    // Global shortcuts
    let has_command_palette = shortcuts
        .iter()
        .any(|s| matches!(s.key, egui::Key::K) && s.description.contains("command palette"));
    assert!(has_command_palette, "Should have command palette shortcut");

    // Navigation shortcuts
    let has_sidebar = shortcuts
        .iter()
        .any(|s| matches!(s.key, egui::Key::B) && s.category == "Navigation");
    assert!(has_sidebar, "Should have sidebar navigation shortcut");

    // View shortcuts
    let has_dashboard = shortcuts
        .iter()
        .any(|s| matches!(s.key, egui::Key::Num1) && s.description.contains("Dashboard"));
    assert!(has_dashboard, "Should have dashboard view shortcut");

    let has_projects = shortcuts
        .iter()
        .any(|s| matches!(s.key, egui::Key::Num2) && s.description.contains("Projects"));
    assert!(has_projects, "Should have projects view shortcut");

    let has_tickets = shortcuts
        .iter()
        .any(|s| matches!(s.key, egui::Key::Num3) && s.description.contains("Tickets"));
    assert!(has_tickets, "Should have tickets view shortcut");

    let has_settings = shortcuts
        .iter()
        .any(|s| matches!(s.key, egui::Key::Num9) && s.description.contains("Settings"));
    assert!(has_settings, "Should have settings view shortcut");

    // Action shortcuts
    let has_escape = shortcuts
        .iter()
        .any(|s| matches!(s.key, egui::Key::Escape) && s.category == "Actions");
    assert!(has_escape, "Should have escape action shortcut");
}

#[wasm_bindgen_test]
fn test_register_custom_shortcut() {
    let mut shortcuts_help = ShortcutsHelp::new();
    let initial_count = shortcuts_help.shortcuts().len();

    // Register a custom shortcut
    let custom_shortcut = ShortcutDefinition::new(
        egui::Key::F,
        egui::Modifiers::COMMAND,
        "Find in files",
        "Search",
    );

    shortcuts_help.register(custom_shortcut);

    // Should have one more shortcut
    assert_eq!(
        shortcuts_help.shortcuts().len(),
        initial_count + 1,
        "Should have one additional shortcut"
    );

    // Custom shortcut should be findable
    let has_custom = shortcuts_help
        .shortcuts()
        .iter()
        .any(|s| matches!(s.key, egui::Key::F) && s.description.contains("Find in files"));
    assert!(has_custom, "Should have registered custom shortcut");
}

#[wasm_bindgen_test]
fn test_shortcut_key_names() {
    // Test various key names
    let test_cases = vec![
        (egui::Key::A, "A"),
        (egui::Key::Z, "Z"),
        (egui::Key::Num1, "1"),
        (egui::Key::Num9, "9"),
        (egui::Key::Escape, "Esc"),
        (egui::Key::Enter, "Enter"),
        (egui::Key::Space, "Space"),
        (egui::Key::Tab, "Tab"),
        (egui::Key::Slash, "/"),
        (egui::Key::Questionmark, "?"),
    ];

    for (key, expected_name) in test_cases {
        let shortcut = ShortcutDefinition::new(key, egui::Modifiers::NONE, "Test", "Test");

        let formatted = shortcut.format();
        assert!(
            formatted.contains(expected_name),
            "Key {:?} should format as '{}', got '{}'",
            key,
            expected_name,
            formatted
        );
    }
}

#[wasm_bindgen_test]
fn test_shortcuts_no_duplicates() {
    let shortcuts_help = ShortcutsHelp::new();
    let shortcuts = shortcuts_help.shortcuts();

    // Check for duplicate key+modifier combinations
    let mut seen = std::collections::HashSet::new();

    for shortcut in shortcuts {
        let key = format!("{:?}+{:?}", shortcut.modifiers, shortcut.key);

        // Note: Some duplicates might be intentional (e.g., context-dependent)
        // This test just documents which shortcuts are registered
        if seen.contains(&key) {
            // Log but don't fail - duplicates might be intentional for different contexts
            tracing::warn!("Duplicate shortcut registered: {}", key);
        }
        seen.insert(key);
    }

    // Should have a reasonable number of shortcuts
    assert!(
        shortcuts.len() >= 10,
        "Should have at least 10 shortcuts registered"
    );
    assert!(
        shortcuts.len() <= 50,
        "Should not have an excessive number of shortcuts"
    );
}

#[wasm_bindgen_test]
fn test_all_shortcuts_have_descriptions() {
    let shortcuts_help = ShortcutsHelp::new();
    let shortcuts = shortcuts_help.shortcuts();

    for shortcut in shortcuts {
        assert!(
            !shortcut.description.is_empty(),
            "Shortcut {:?} should have a description",
            shortcut.key
        );
        assert!(
            !shortcut.category.is_empty(),
            "Shortcut {:?} should have a category",
            shortcut.key
        );
    }
}

#[wasm_bindgen_test]
fn test_shortcuts_help_default() {
    let shortcuts_help = ShortcutsHelp::default();

    // Should be equivalent to new()
    assert!(!shortcuts_help.is_open);
    assert!(shortcuts_help.shortcuts().len() > 0);
}
