# Worknest GUI Tests

Automated tests for the Worknest GUI using wasm-bindgen-test.

## Test Organization

### `state_tests.rs`
Tests for application state management:
- State initialization
- Navigation between screens
- Notification system
- Authentication state
- Loading states
- Demo data management

### `ui_tests.rs`
Tests for UI interactions and logic:
- Project creation and management
- Project archiving/unarchiving
- Screen transitions
- Navigation with parameters
- Filtering and search
- Notification formatting

## Running Tests

### Prerequisites
```bash
# Install wasm-pack if not already installed
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Or with cargo
cargo install wasm-pack
```

### Run All Tests

**In Firefox (headless):**
```bash
wasm-pack test --headless --firefox crates/worknest-gui
```

**In Chrome (headless):**
```bash
wasm-pack test --headless --chrome crates/worknest-gui
```

**In Browser (for debugging):**
```bash
wasm-pack test --firefox crates/worknest-gui
```

### Run Specific Test File

```bash
# Run only state tests
wasm-pack test --headless --firefox crates/worknest-gui -- --test state_tests

# Run only UI tests
wasm-pack test --headless --firefox crates/worknest-gui -- --test ui_tests
```

### Run Specific Test

```bash
wasm-pack test --headless --firefox crates/worknest-gui -- --test test_authentication
```

## Writing New Tests

### Test Structure

```rust
use wasm_bindgen_test::*;
use worknest_gui::state::AppState;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_my_feature() {
    // Arrange
    let state = AppState::new(...);

    // Act
    state.do_something();

    // Assert
    assert_eq!(state.some_value, expected_value);
}
```

### Best Practices

1. **Test Naming**: Use descriptive names starting with `test_`
2. **Arrange-Act-Assert**: Follow the AAA pattern
3. **Independence**: Each test should be independent and not rely on other tests
4. **Coverage**: Test both happy paths and edge cases
5. **Async**: Use `wasm_bindgen_test_configure!(run_in_browser)` for browser APIs

### Common Patterns

**Testing Navigation:**
```rust
state.navigate_to(Screen::Dashboard);
assert_eq!(state.current_screen, Screen::Dashboard);
```

**Testing Notifications:**
```rust
state.notify_success("Message".to_string());
assert_eq!(state.notifications.len(), 1);
```

**Testing Authentication:**
```rust
state.login(user, token);
assert!(state.is_authenticated());
```

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Run WASM Tests
  run: |
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    wasm-pack test --headless --firefox crates/worknest-gui
    wasm-pack test --headless --chrome crates/worknest-gui
```

## Test Coverage

Current coverage:
- State management: ✅ Basic tests implemented
- UI logic: ✅ Basic tests implemented
- Integration tests: 🚧 TODO
- E2E tests: 🚧 TODO

## Troubleshooting

### "wasm-pack: command not found"
Install wasm-pack: `cargo install wasm-pack`

### "geckodriver not found"
For Firefox tests, install geckodriver:
```bash
# Ubuntu/Debian
sudo apt-get install firefox-geckodriver

# macOS
brew install geckodriver

# Or download from: https://github.com/mozilla/geckodriver/releases
```

### "chromedriver not found"
For Chrome tests, install chromedriver:
```bash
# Ubuntu/Debian
sudo apt-get install chromium-chromedriver

# macOS
brew install chromedriver

# Or download from: https://chromedriver.chromium.org/
```

### Tests Timeout
Increase timeout with `WASM_BINDGEN_TEST_TIMEOUT`:
```bash
WASM_BINDGEN_TEST_TIMEOUT=300 wasm-pack test --headless --firefox crates/worknest-gui
```

## Future Improvements

- [ ] Add property-based tests with quickcheck
- [ ] Add visual regression tests
- [ ] Add performance benchmarks
- [ ] Increase test coverage to >70%
- [ ] Add E2E tests with Playwright
- [ ] Add mutation testing
- [ ] Integrate with CI/CD pipeline
