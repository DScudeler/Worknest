# Layer-Based Testing Quick Start Guide

## What is Layer-Based Testing?

A systematic approach to test egui applications by validating each component independently:

1. **UI Layer**: Verify widgets are clickable and interactive
2. **API Layer**: Validate state changes trigger correct API calls
3. **Transition Layer**: Ensure state updates reflect in the UI

## Quick Examples

### Layer 1: Test Button Clickability

```rust
use wasm_bindgen_test::*;
use worknest_gui::tests::framework::ui_interaction::*;

#[wasm_bindgen_test]
fn test_button_is_clickable() {
    let mut ui_ctx = UiTestContext::new();

    // During render, record the button response
    ui_ctx.record_widget("my_button", &button_response);

    // Verify it's clickable
    assert!(ui_ctx.is_clickable("my_button"));
    assert!(ui_ctx.can_hover("my_button"));
}
```

### Layer 2: Test API Calls

```rust
use worknest_gui::tests::framework::api_validation::*;

#[wasm_bindgen_test]
fn test_api_call_made() {
    let mock = MockApiClient::new("http://localhost:3000".to_string());

    // Perform action that should trigger API call
    // ...

    // Verify correct API call was made
    assert!(mock.was_called(&ExpectedCall {
        method: HttpMethod::POST,
        path: "/api/projects".to_string(),
        requires_auth: true,
        body: None,
    }));
}
```

### Layer 3: Test State Transitions

```rust
use worknest_gui::tests::framework::state_transition::*;

#[wasm_bindgen_test]
fn test_screen_transition() {
    let validator = StateTransitionValidator::new(
        Screen::Login,
        Screen::Dashboard
    );

    // Perform login
    state.login(user, token);

    // Validate transition happened
    validator.validate(&state).assert_passed();
}
```

## Action-State-UI Matrix

Map user actions to expected outcomes:

```rust
let matrix = InteractionMatrix::new(
    "project_list",
    UserAction::Click("new_project_btn".to_string())
)
.expects_state(ExpectedState {
    screen_changed: false,
    new_screen: None,
    data_modified: false,
    api_called: false,
})
.expects_ui(ExpectedUI {
    elements_visible: vec!["create_dialog".to_string()],
    elements_hidden: vec![],
    elements_enabled: vec!["create_button".to_string()],
    elements_disabled: vec![],
});
```

## Testing Coverage Matrix

For each screen/feature, create tests for all three layers:

| Screen | Layer 1: UI | Layer 2: API | Layer 3: Transition |
|--------|-------------|--------------|---------------------|
| **Project List** | ✓ Buttons clickable<br>✓ Cards hoverable<br>✓ Dialog opens | ✓ GET /api/projects<br>✓ POST /api/projects<br>✓ Auth headers | ✓ Login → List<br>✓ Create → Display |
| **Project Detail** | ✓ Edit button<br>✓ New ticket button<br>✓ Navigation buttons | ✓ PATCH /api/projects/:id<br>✓ POST /api/tickets<br>✓ DELETE /api/projects/:id | ✓ Detail → Edit<br>✓ Detail → Board<br>✓ Delete → List |
| **Ticket Detail** | ✓ Status buttons<br>✓ Comment form<br>✓ Edit controls | ✓ PATCH /api/tickets/:id<br>✓ POST /api/comments<br>✓ DELETE /api/tickets/:id | ✓ Status changes<br>✓ Comment added<br>✓ Back navigation |

## Common Testing Patterns

### Pattern 1: Test New Feature End-to-End

```rust
#[wasm_bindgen_test]
fn test_create_project_workflow() {
    // 1. UI: Verify button is clickable
    assert!(ui_ctx.is_clickable("new_project_button"));

    // 2. UI: Verify dialog opens
    assert!(ui_ctx.is_visible("create_dialog"));

    // 3. API: Verify API call made
    assert!(mock.was_called(&ExpectedCall {
        method: HttpMethod::POST,
        path: "/api/projects".to_string(),
        requires_auth: true,
        body: None,
    }));

    // 4. State: Verify notification shown
    assert!(!state.notifications.is_empty());

    // 5. UI: Verify project appears in list
    assert_eq!(state.projects.len(), 1);
}
```

### Pattern 2: Test Error Handling

```rust
#[wasm_bindgen_test]
fn test_api_error_handling() {
    // Configure mock to fail
    mock.set_should_fail(true);

    // Trigger action
    // ...

    // Verify error notification shown
    assert!(state.notifications.iter().any(|n| n.is_error));
}
```

### Pattern 3: Test Authentication

```rust
#[wasm_bindgen_test]
fn test_requires_authentication() {
    let validator = ScreenValidator::for_screen(Screen::ProjectList)
        .requires_authentication();

    // Without login, should fail
    assert!(!validator.validate(&state).passed);

    // After login, should pass
    state.login(user, token);
    assert!(validator.validate(&state).passed);
}
```

## Fixing Mouse Event Bugs

### Issue: Element visible but not clickable

**Diagnosis:**
```rust
let mut ui_ctx = UiTestContext::new();
ui_ctx.record_widget("my_element", &response);

if !ui_ctx.is_clickable("my_element") {
    println!("❌ Element not clickable - check Sense configuration");
}
```

**Fix:**
```rust
// ❌ WRONG: No interaction capability
let group = ui.group(|ui| {
    ui.label("Content");
});

// ✅ CORRECT: Add click and hover sense
let group = ui.group(|ui| {
    ui.label("Content");
});

let response = group.response.interact(
    egui::Sense::click().union(egui::Sense::hover())
);

if response.hovered() {
    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
}

if response.clicked() {
    // Handle click
}
```

## Running Tests

```bash
# All layer-based tests
wasm-pack test --headless --firefox crates/worknest-gui --test layer_based_tests

# Specific layer
wasm-pack test --headless --firefox crates/worknest-gui --test layer_based_tests -- test_ui

# With output
wasm-pack test --headless --firefox crates/worknest-gui --test layer_based_tests -- --nocapture
```

## Test File Organization

```
tests/
├── framework/
│   ├── mod.rs                    # Framework exports
│   ├── ui_interaction.rs         # Layer 1 utilities
│   ├── api_validation.rs         # Layer 2 utilities
│   └── state_transition.rs       # Layer 3 utilities
│
├── layer_based_tests.rs          # Main test file
├── ui_tests.rs                   # Legacy UI tests
├── state_tests.rs                # Legacy state tests
└── e2e_tests.rs                  # Legacy E2E tests
```

## When to Use Each Layer

### Use Layer 1 (UI) When:
- Adding new buttons or interactive elements
- Fixing mouse event bugs
- Verifying hover feedback
- Testing dialog visibility

### Use Layer 2 (API) When:
- Implementing new API endpoints
- Testing authentication
- Validating request payloads
- Testing error handling

### Use Layer 3 (Transition) When:
- Testing navigation flows
- Verifying screen requirements (auth, data)
- Testing state-driven UI updates
- Validating notification display

## Next Steps

1. **Review** [TESTING_FRAMEWORK.md](./TESTING_FRAMEWORK.md) for complete documentation
2. **Run** existing layer-based tests to see examples
3. **Add** tests for your new features using this approach
4. **Fix** mouse event bugs using the diagnosis pattern

## Key Benefits

✅ **Systematic** - Test every aspect of each feature
✅ **Maintainable** - Clear separation of concerns
✅ **Debuggable** - Easily identify which layer is failing
✅ **Comprehensive** - Catch bugs at each layer independently
✅ **Documented** - Each test documents expected behavior
