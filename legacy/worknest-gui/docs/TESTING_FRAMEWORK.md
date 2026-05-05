# Layer-Based Testing Framework for Worknest GUI

## Overview

This testing framework provides comprehensive validation for egui-based applications through a **layered testing approach** that tests each component of the application independently.

## Architecture

The framework is organized into three distinct testing layers:

```
┌─────────────────────────────────────────────┐
│         Layer 1: UI Interaction             │
│  Test egui widget availability & events     │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│      Layer 2: State → API Validation        │
│  Test that state changes trigger API calls  │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│    Layer 3: State → UI Transition           │
│  Test that state updates reflect in UI      │
└─────────────────────────────────────────────┘
```

## Layer 1: UI Interaction Testing

### Purpose
Validate that UI elements are properly configured for user interaction in egui's immediate mode rendering system.

### Challenge with egui
egui is an **immediate mode GUI** framework, which means:
- No persistent widget tree to query
- Widgets are created/destroyed each frame
- Can't use traditional DOM-based testing tools
- Must verify interaction capabilities through egui::Response analysis

### Solution: UiTestContext

```rust
use worknest_gui::tests::framework::ui_interaction::{UiTestContext, SenseCapability};

let mut ui_ctx = UiTestContext::new();

// During rendering, record widget responses
ui_ctx.record_widget("my_button", &button_response);

// Verify interaction capabilities
assert!(ui_ctx.is_clickable("my_button"));
assert!(ui_ctx.can_hover("my_button"));
```

### Interaction Matrix

The **Action-State-UI Matrix** maps user actions to expected outcomes:

```rust
use worknest_gui::tests::framework::ui_interaction::{
    InteractionMatrix, UserAction, ExpectedState, ExpectedUI
};

let matrix = InteractionMatrix::new(
    "project_list",
    UserAction::Click("new_project_button".to_string())
)
.expects_state(ExpectedState {
    screen_changed: false,
    new_screen: None,
    data_modified: false,
    api_called: false,
})
.expects_ui(ExpectedUI {
    elements_visible: vec!["create_project_dialog".to_string()],
    elements_hidden: vec![],
    elements_enabled: vec!["create_button".to_string()],
    elements_disabled: vec![],
});
```

### Common UI Testing Patterns

#### Test Button Availability
```rust
#[wasm_bindgen_test]
fn test_button_is_clickable() {
    let mut ctx = UiTestContext::new();

    // Record button response during render
    ctx.record_widget("submit_button", &response);

    // Verify it's clickable
    assert!(ctx.is_clickable("submit_button"));
}
```

#### Test Element Visibility
```rust
#[wasm_bindgen_test]
fn test_dialog_visibility() {
    let mut ctx = UiTestContext::new();

    // Open dialog
    show_dialog = true;

    // Render and record
    ctx.record_widget("confirmation_dialog", &dialog_response);

    assert!(ctx.is_visible("confirmation_dialog"));
}
```

#### Test Hover Capabilities
```rust
#[wasm_bindgen_test]
fn test_card_hover() {
    let mut ctx = UiTestContext::new();

    ctx.record_widget("project_card", &card_response);

    // Card should support both click and hover
    assert!(ctx.can_hover("project_card"));
    assert!(ctx.is_clickable("project_card"));
}
```

### Fixing Mouse Event Bugs

**Common Issue**: Elements visible but not interactive

**Cause**: Missing or incorrect `Sense` configuration

**Fix**: Ensure proper Sense configuration:

```rust
// ❌ WRONG: No hover feedback
ui.group(|ui| {
    ui.label("Project Card");
})

// ✅ CORRECT: Both click and hover
let group_response = ui.group(|ui| {
    ui.label("Project Card");
});

let card_response = group_response.response.interact(
    egui::Sense::click().union(egui::Sense::hover())
);

if card_response.hovered() {
    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
}
```

## Layer 2: State → API Call Validation

### Purpose
Verify that state changes trigger the correct API calls with proper authentication and payloads.

### Mock API Client

```rust
use worknest_gui::tests::framework::api_validation::{
    MockApiClient, ApiCallValidator, ExpectedCall, HttpMethod
};

let mock_client = MockApiClient::new("http://localhost:3000".to_string());

// API calls are automatically recorded
// ...perform actions...

// Validate the calls
let validator = ApiCallValidator::new()
    .expect_call(ExpectedCall {
        method: HttpMethod::POST,
        path: "/api/projects".to_string(),
        requires_auth: true,
        body: None,
    })
    .with_recorded_calls(mock_client.get_recorded_calls());

validator.validate().assert_passed();
```

### Common API Testing Patterns

#### Test API Call is Made
```rust
#[wasm_bindgen_test]
fn test_creates_project_via_api() {
    let mock = MockApiClient::new("http://localhost:3000".to_string());

    // Trigger project creation
    // ...

    assert!(mock.was_called(&ExpectedCall {
        method: HttpMethod::POST,
        path: "/api/projects".to_string(),
        requires_auth: true,
        body: None,
    }));
}
```

#### Test Authentication Header
```rust
#[wasm_bindgen_test]
fn test_api_includes_auth_token() {
    let mock = MockApiClient::new("http://localhost:3000".to_string());

    // Perform authenticated action
    // ...

    let last_call = mock.get_last_call().unwrap();
    assert!(last_call.has_auth_header());
    assert_eq!(last_call.get_auth_token(), Some("Bearer test_token".to_string()));
}
```

#### Test API Call Count
```rust
#[wasm_bindgen_test]
fn test_api_called_once() {
    let mock = MockApiClient::new("http://localhost:3000".to_string());

    // Should only call API once
    // ...

    assert_eq!(mock.count_calls(&HttpMethod::GET, "/api/projects"), 1);
}
```

## Layer 3: State → UI Transition Validation

### Purpose
Ensure that state changes produce the expected UI transitions and display updates.

### State Transition Validator

```rust
use worknest_gui::tests::framework::state_transition::{
    StateTransitionValidator, ScreenValidator
};

let validator = StateTransitionValidator::new(
    Screen::Login,
    Screen::Dashboard
).expects_notification();

// Perform login
state.login(user, token);

// Validate transition
validator.validate(&state).assert_passed();
```

### Common Transition Testing Patterns

#### Test Screen Navigation
```rust
#[wasm_bindgen_test]
fn test_navigates_to_projects() {
    let validator = StateTransitionValidator::new(
        Screen::Dashboard,
        Screen::ProjectList
    );

    state.navigate_to(Screen::ProjectList);

    validator.validate(&state).assert_passed();
}
```

#### Test Authentication Requirements
```rust
#[wasm_bindgen_test]
fn test_screen_requires_auth() {
    let validator = ScreenValidator::for_screen(Screen::ProjectList)
        .requires_authentication();

    // Without login, should fail
    let result = validator.validate(&state);
    assert!(!result.passed);
}
```

#### Test Data Requirements
```rust
#[wasm_bindgen_test]
fn test_screen_requires_data() {
    let validator = ScreenValidator::for_screen(Screen::ProjectDetail(id))
        .requires_projects(1);

    let result = validator.validate(&state);
    result.assert_passed();
}
```

## Complete Workflow Testing

Combine all three layers to test complete user workflows:

```rust
#[wasm_bindgen_test]
fn test_complete_project_creation_workflow() {
    // Layer 1: UI
    // - Verify "New Project" button is clickable
    // - Verify dialog appears with enabled form

    // Layer 2: State → API
    // - Verify POST /api/projects is called
    // - Verify authentication header included

    // Layer 3: State → UI
    // - Verify success notification shown
    // - Verify project appears in list
    // - Verify dialog closes

    // ... test implementation ...
}
```

## Running Tests

### All Tests
```bash
wasm-pack test --headless --firefox crates/worknest-gui
```

### Specific Test File
```bash
wasm-pack test --headless --firefox crates/worknest-gui --test layer_based_tests
```

### Single Test
```bash
wasm-pack test --headless --firefox crates/worknest-gui --test layer_based_tests -- test_project_card_interactions
```

## Best Practices

### 1. Test Each Layer Independently

```rust
// ✅ GOOD: Test each layer separately
#[wasm_bindgen_test]
fn test_ui_button_clickable() { /* Layer 1 only */ }

#[wasm_bindgen_test]
fn test_api_call_made() { /* Layer 2 only */ }

#[wasm_bindgen_test]
fn test_state_transition() { /* Layer 3 only */ }

// ❌ BAD: Mixing layers in one test
#[wasm_bindgen_test]
fn test_everything_at_once() { /* Too complex */ }
```

### 2. Use Descriptive Test Names

```rust
// ✅ GOOD: Clear purpose
#[wasm_bindgen_test]
fn test_project_card_has_hover_and_click_events() { }

// ❌ BAD: Unclear intent
#[wasm_bindgen_test]
fn test_project() { }
```

### 3. Assert Expected Failures

```rust
// ✅ GOOD: Test negative cases
#[wasm_bindgen_test]
fn test_unauthenticated_api_call_fails() {
    let result = validator.validate(&state);
    assert!(!result.passed); // Expect failure
}
```

### 4. Test Edge Cases

```rust
// Test with no data
#[wasm_bindgen_test]
fn test_empty_project_list() { }

// Test with maximum data
#[wasm_bindgen_test]
fn test_many_projects() { }

// Test error scenarios
#[wasm_bindgen_test]
fn test_api_failure_handling() { }
```

## Common Issues and Solutions

### Issue: "Element not clickable"

**Cause**: Missing `Sense::click()` configuration

**Solution**:
```rust
// Add proper sense to interactive elements
let response = ui.button("Click me"); // Has Sense::click by default

// For custom widgets:
let response = ui.allocate_response(
    size,
    egui::Sense::click().union(egui::Sense::hover())
);
```

### Issue: "Mouse events not working"

**Cause**: Element rendered but not configured for interaction

**Solution**: Verify `Sense` configuration
```rust
// Use UiTestContext to diagnose
let mut ctx = UiTestContext::new();
ctx.record_widget("my_element", &response);

if !ctx.is_clickable("my_element") {
    println!("Element not clickable - check Sense configuration");
}
```

### Issue: "API call not recorded"

**Cause**: Mock client not being used

**Solution**: Ensure mock client is properly integrated
```rust
// Replace real API client with mock
let mock_client = MockApiClient::new("http://localhost:3000".to_string());
let state = AppState::new(mock_client.clone());

// Now calls will be recorded
```

## Framework API Reference

### UiTestContext

```rust
pub struct UiTestContext {
    // Records and tracks UI element interactions
}

impl UiTestContext {
    pub fn new() -> Self;
    pub fn record_widget(&mut self, id: impl Into<String>, response: &Response);
    pub fn is_clickable(&self, id: &str) -> bool;
    pub fn is_visible(&self, id: &str) -> bool;
    pub fn can_hover(&self, id: &str) -> bool;
    pub fn get_clickable_elements(&self) -> Vec<String>;
}
```

### MockApiClient

```rust
pub struct MockApiClient {
    // Records API calls for validation
}

impl MockApiClient {
    pub fn new(base_url: String) -> Self;
    pub fn get_recorded_calls(&self) -> Vec<RecordedCall>;
    pub fn was_called(&self, expected: &ExpectedCall) -> bool;
    pub fn count_calls(&self, method: &HttpMethod, path: &str) -> usize;
    pub fn get_last_call(&self) -> Option<RecordedCall>;
}
```

### StateTransitionValidator

```rust
pub struct StateTransitionValidator {
    // Validates state transitions
}

impl StateTransitionValidator {
    pub fn new(initial: Screen, expected: Screen) -> Self;
    pub fn expects_data_update(self) -> Self;
    pub fn expects_notification(self) -> Self;
    pub fn validate(&self, state: &AppState) -> TransitionResult;
}
```

## Contributing

When adding new screens or features:

1. **Add UI interaction tests** - Verify all buttons and interactive elements
2. **Add API validation tests** - Ensure API calls are properly triggered
3. **Add transition tests** - Verify state changes reflect in UI
4. **Add integration tests** - Test complete workflows

## Resources

- [egui documentation](https://docs.rs/egui/)
- [wasm-bindgen-test guide](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/)
- [Worknest Testing Guide](../TESTING.md)
