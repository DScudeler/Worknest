# Contributing to Worknest

Thank you for your interest in contributing to Worknest! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

Be respectful, inclusive, and professional in all interactions. We're building a welcoming community for developers of all backgrounds and experience levels.

## Getting Started

### Prerequisites

- Rust 1.70 or later (install from [rustup.rs](https://rustup.rs))
- Git
- A GitHub account
- SQLite 3.35+ (usually pre-installed)

### Setting Up the Development Environment

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/Worknest.git
   cd Worknest
   ```

3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/DScudeler/Worknest.git
   ```

4. Build the project:
   ```bash
   cargo build
   ```

5. Run tests:
   ```bash
   cargo test --workspace
   ```

## Development Workflow

### 1. Create a Branch

Always create a new branch for your work:

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Adding or updating tests

### 2. Make Your Changes

- Write clean, readable code
- Follow Rust naming conventions
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 3. Testing

Before submitting your changes, ensure:

```bash
# All tests pass
cargo test --workspace

# Code is properly formatted
cargo fmt --check

# No clippy warnings
cargo clippy -- -D warnings

# Build succeeds
cargo build --workspace
```

Fix any issues before submitting your PR.

### 4. Commit Your Changes

Write clear, descriptive commit messages:

```bash
git add .
git commit -m "Add feature: brief description

Detailed explanation of what changed and why.
Reference any related issues: Fixes #123"
```

Commit message guidelines:
- Use present tense ("Add feature" not "Added feature")
- First line is a brief summary (50 chars or less)
- Add detailed explanation in the body if needed
- Reference issues and PRs where appropriate

### 5. Push and Create a Pull Request

```bash
git push origin feature/your-feature-name
```

Then go to GitHub and create a Pull Request.

## Pull Request Guidelines

### PR Title

Use clear, descriptive titles:
- ✅ "Add user authentication with JWT"
- ✅ "Fix ticket status update bug"
- ❌ "Updates"
- ❌ "Fixed stuff"

### PR Description

Include:
- **What**: Brief description of changes
- **Why**: Reason for the changes
- **How**: Technical approach (if complex)
- **Testing**: How you tested the changes
- **Screenshots**: For UI changes
- **Related Issues**: Fixes #123, Closes #456

Example template:
```markdown
## Description
Brief overview of the changes.

## Motivation
Why are these changes needed?

## Changes Made
- Change 1
- Change 2
- Change 3

## Testing
How were these changes tested?

## Related Issues
Fixes #123
```

### Code Review Process

1. Maintainers will review your PR
2. Address feedback and update your PR
3. Once approved, maintainers will merge

## Coding Standards

### Rust Style Guide

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Prefer explicit types where clarity is important
- Document public APIs with doc comments
- Use `Result` for error handling
- Avoid unwrap() in production code (use proper error handling)

### Module Organization

```rust
// Standard library imports first
use std::collections::HashMap;

// External crates next
use serde::{Deserialize, Serialize};

// Internal crates last
use worknest_core::User;
```

### Documentation

Write documentation for:
- All public APIs (functions, structs, enums, traits)
- Complex algorithms or logic
- Non-obvious code

```rust
/// Brief description of what this does.
///
/// More detailed explanation if needed.
///
/// # Arguments
/// * `arg1` - Description of arg1
/// * `arg2` - Description of arg2
///
/// # Returns
/// Description of return value
///
/// # Errors
/// When this function will return errors
///
/// # Examples
/// ```
/// let result = function(arg1, arg2);
/// ```
pub fn function(arg1: Type1, arg2: Type2) -> Result<ReturnType> {
    // implementation
}
```

### Testing

Write tests for:
- All public APIs
- Edge cases
- Error conditions
- Business logic

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = create_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

Aim for >70% code coverage in core business logic.

## Project Structure

```
worknest/
├── crates/              # Rust crates
│   ├── worknest-core/   # Domain models and business logic
│   ├── worknest-db/     # Database layer
│   ├── worknest-auth/   # Authentication
│   ├── worknest-api/    # Application API
│   └── worknest-gui/    # Desktop UI
├── docs/                # Documentation
├── migrations/          # Database migrations
└── plugins/             # Official plugins (future)
```

## Areas to Contribute

### Good First Issues

Look for issues labeled `good-first-issue` - these are suitable for newcomers.

### Priority Areas (MVP)

- Database layer and repositories
- Authentication system
- Core UI components (egui)
- Project management features
- Ticket management features
- Test coverage
- Documentation

### Future Areas

- Plugin system
- Cloud sync
- Web application (WASM)
- Mobile apps
- Integrations

## Reporting Bugs

### Before Reporting

- Check if the bug is already reported
- Verify it's reproducible
- Gather relevant information

### Bug Report Template

```markdown
**Description**
Clear description of the bug.

**To Reproduce**
Steps to reproduce:
1. Go to '...'
2. Click on '...'
3. See error

**Expected Behavior**
What you expected to happen.

**Actual Behavior**
What actually happened.

**Environment**
- OS: [e.g., Windows 11, macOS 14, Ubuntu 22.04]
- Rust version: [e.g., 1.75.0]
- Worknest version: [e.g., 0.1.0]

**Logs**
```
Paste relevant logs here
```

**Screenshots**
If applicable, add screenshots.
```

## Feature Requests

We welcome feature requests! Please:

1. Check if it's already requested
2. Explain the use case
3. Describe the proposed solution
4. Consider alternatives

Use the feature request template in GitHub Issues.

## Documentation Contributions

Documentation improvements are always welcome:

- Fix typos and grammar
- Clarify confusing sections
- Add examples
- Update outdated information
- Write tutorials

## Questions?

- Open a [GitHub Discussion](https://github.com/DScudeler/Worknest/discussions)
- Check existing issues and discussions
- Join our Discord (coming soon)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Worknest! 🦀
