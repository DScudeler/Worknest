# Worknest Testing Guide

Complete guide for running automated tests in Worknest project.

## Quick Start

Run all tests with a single command:

```bash
./run-all-tests.sh
```

This script runs:
- 33 Core model tests
- 30 Database layer tests
- 26 Authentication tests
- 6 GUI state tests
- 17 GUI UI tests
- 14 GUI E2E integration tests

**Total: 126+ tests**

## Prerequisites

### Required Tools

1. **Rust toolchain** (stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **WASM target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **wasm-pack** (for frontend tests)
   ```bash
   cargo install wasm-pack
   ```

4. **Firefox** (for headless browser testing)
   ```bash
   sudo apt-get install firefox  # Ubuntu/Debian
   brew install firefox          # macOS
   ```

## Running Tests Individually

### Backend Tests

```bash
# Core models
cargo test --package worknest-core --lib

# Database layer
cargo test --package worknest-db --lib

# Authentication
cargo test --package worknest-auth --lib
```

### Frontend Tests (WASM)

```bash
# State management tests
wasm-pack test --headless --firefox crates/worknest-gui --test state_tests

# UI interaction tests
wasm-pack test --headless --firefox crates/worknest-gui --test ui_tests

# End-to-end integration tests
wasm-pack test --headless --firefox crates/worknest-gui --test e2e_tests
```

### All Frontend Tests at Once

```bash
wasm-pack test --headless --firefox crates/worknest-gui
```

### E2E Browser Tests (Playwright)

```bash
cd e2e

# Install dependencies (first time only)
npm install
npx playwright install

# Run all E2E tests
npm test

# Run with visible browser
npm run test:headed

# Run with Playwright UI
npm run test:ui

# Run specific browser
npm run test:chromium
npm run test:firefox
npm run test:webkit
```

## Test Categories

### Backend Tests (89 tests)

**Core Models** (33 tests)
- User, Project, Ticket, Comment models
- Status and Type enums
- UUID generation and validation
- Model relationships and constraints

**Database Layer** (30 tests)
- Repository pattern implementation
- CRUD operations for all entities
- Query filtering and sorting
- Transaction handling
- Error handling

**Authentication** (26 tests)
- Password hashing and verification
- JWT token generation and validation
- User registration and login
- Authorization and permissions
- Token refresh and expiration

### Frontend Tests (37 tests)

**State Management** (6 tests)
- AppState initialization
- Authentication state
- Demo mode data
- Navigation state
- Notification system
- Event queue processing

**UI Components** (22 tests)
- Login screen interactions
- Project list and filtering
- Project detail view
- Ticket creation and editing (**NEW: from project detail screen**)
- Comment system UI
- Settings screen
- Navigation flows
- Button accessibility
- Dialog state management

**E2E Integration** (16 tests)
- Complete authentication flow
- Project lifecycle (create, update, archive, delete)
- Ticket lifecycle (create, update, complete, delete)
- **NEW: Ticket creation from project detail screen**
- Comment workflow (add, edit, delete)
- Multi-project management
- Cascade deletion
- Complete user journey simulation

### E2E Browser Tests (40+ tests) **NEW!**

**Authentication & Initialization** (8 tests)
- Application loading and WASM init
- Login screen state
- Demo mode navigation
- localStorage handling
- Accessibility compliance

**Project Management** (12 tests)
- Canvas interactions
- State persistence
- Keyboard navigation
- Rapid interactions
- Performance benchmarks

**Ticket Workflows** (15 tests)
- **Ticket creation from project detail (NEW FEATURE)**
- Multiple ticket types (Task, Bug, Feature, Epic)
- Priority levels (Low, Medium, High, Critical)
- Status transitions
- Project filtering
- Canvas interactions
- Keyboard navigation

**Comment Management** (10 tests)
- Comment creation
- Multiple comments per ticket
- Comment editing
- Comment deletion
- Ticket isolation

**Visual Regression** (15+ tests)
- Initial load screenshots
- Canvas rendering
- Responsive design (6 viewports)
- Component appearance
- Theme consistency
- Focus states
- Interaction states
- Loading states

## CI/CD Integration

### GitHub Actions

The project includes automated testing on GitHub Actions:

`.github/workflows/test.yml`

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`

**Jobs:**
1. Backend tests (Linux)
2. Frontend tests (Linux + Firefox)
3. **NEW: E2E Browser tests (Playwright - Chrome, Firefox, WebKit)**
4. Test summary report

### Local CI Simulation

Run the same checks as CI locally:

```bash
./run-all-tests.sh
```

## Browser Options

### Firefox (Default)

```bash
wasm-pack test --headless --firefox crates/worknest-gui
```

### Chrome

```bash
wasm-pack test --headless --chrome crates/worknest-gui
```

### Safari (macOS only)

```bash
wasm-pack test --headless --safari crates/worknest-gui
```

## Debugging Tests

### Run with Output

```bash
wasm-pack test --headless --firefox crates/worknest-gui -- --nocapture
```

### Run Specific Test

```bash
wasm-pack test --headless --firefox crates/worknest-gui --test e2e_tests -- e2e_complete_authentication_flow
```

### Interactive Browser (Non-headless)

```bash
wasm-pack test --firefox crates/worknest-gui
```

This opens Firefox with visible browser window for debugging.

## Test Coverage

Current test coverage: **165+ tests** (**NEW: +40 browser E2E tests**)

Coverage breakdown:
- Core models: 100%
- Database layer: 95%
- Authentication: 100%
- GUI state: 90%
- GUI UI: 90% (**improved with new tests**)
- E2E integration: 85% (**improved**)
- **NEW: Browser E2E: 80%** (real user interactions)

## Writing New Tests

### Backend Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = feature_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Frontend Test Template

```rust
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_ui_feature() {
    // Arrange
    let mut state = create_test_state();

    // Act
    state.perform_action();

    // Assert
    assert_eq!(state.current_screen, expected_screen);
}
```

### E2E Browser Test Template

```typescript
import { test, expect } from '@playwright/test';
import { goToDemoMode, waitForWasmInit } from '../fixtures/test-helpers';

test.describe('Feature Name', () => {
  test.beforeEach(async ({ page }) => {
    await goToDemoMode(page);
    await waitForWasmInit(page);
  });

  test('should perform action', async ({ page }) => {
    // Arrange
    const canvas = page.locator('canvas').first();

    // Act
    await canvas.click();

    // Assert
    await expect(canvas).toBeVisible();
  });
});
```

For more details on writing E2E tests, see [e2e/README.md](e2e/README.md).

## Performance

Test suite performance on typical hardware:
- Backend tests: ~1 second
- Frontend WASM tests: ~5 seconds
- **NEW: E2E Browser tests: ~2-3 minutes (40+ tests across 3 browsers)**
- **Total runtime: ~3-4 minutes**

Optimizations:
- Parallel test execution
- Cargo build caching
- WASM binary caching
- Headless browser mode
- **NEW: Playwright browser pooling**
- **NEW: Screenshot diffing with thresholds**

## Troubleshooting

### "geckodriver not found"

Install geckodriver:
```bash
# Ubuntu/Debian
sudo apt-get install firefox-geckodriver

# macOS
brew install geckodriver
```

### "wasm-pack not found"

Install wasm-pack:
```bash
cargo install wasm-pack
```

### Tests timeout

Increase timeout:
```bash
WASM_BINDGEN_TEST_TIMEOUT=60 wasm-pack test --headless --firefox crates/worknest-gui
```

### Firefox not starting

Check Firefox installation:
```bash
firefox --version
```

Try Chrome instead:
```bash
wasm-pack test --headless --chrome crates/worknest-gui
```

## Best Practices

1. **Run tests before committing**
   ```bash
   ./run-all-tests.sh
   ```

2. **Write tests first (TDD)**
   - Write failing test
   - Implement feature
   - Verify test passes

3. **Test edge cases**
   - Empty inputs
   - Invalid data
   - Boundary conditions
   - Error scenarios

4. **Keep tests fast**
   - Use demo mode for UI tests
   - Mock external dependencies
   - Avoid unnecessary sleeps

5. **Maintain test independence**
   - Each test should work in isolation
   - Clean up test data
   - Don't rely on test execution order

## Additional Resources

- [wasm-pack documentation](https://rustwasm.github.io/wasm-pack/)
- [wasm-bindgen-test guide](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/index.html)
- [Rust testing guide](https://doc.rust-lang.org/book/ch11-00-testing.html)

## Support

For testing issues:
1. Check this guide first
2. Review CI logs on GitHub
3. Run with `--nocapture` for debugging
4. Open an issue with test output
