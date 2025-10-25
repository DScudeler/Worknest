# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with Rust workspace
- Core domain models (User, Project, Ticket)
- Comprehensive project roadmap (ROADMAP.md)
- Technical architecture documentation (ARCHITECTURE.md)
- Contribution guidelines (CONTRIBUTING.md)
- GitHub Actions CI/CD pipeline
  - Test suite automation
  - Code formatting checks
  - Clippy linting
  - Security audit
  - Code coverage reporting
- Development tools
  - Makefile for common tasks
  - rustfmt configuration
  - clippy configuration
  - EditorConfig for consistent formatting
- GitHub templates
  - Pull request template
  - Bug report template
  - Feature request template
- Community files
  - Code of Conduct
  - Security Policy
- Dependabot configuration for dependency updates
- Basic egui GUI application shell

### Changed
- Nothing yet

### Deprecated
- Nothing yet

### Removed
- Nothing yet

### Fixed
- Nothing yet

### Security
- Implemented bcrypt password hashing in architecture
- Planned JWT-based authentication
- SQL injection prevention via parameterized queries

## [0.1.0] - TBD (MVP Release)

### Planned Features
- User authentication and session management
- Local SQLite database
- Project management (create, edit, delete, archive)
- Ticket management (Task, Bug, Feature, Epic)
- Multiple ticket views (List, Kanban board)
- Priority and status tracking
- Cross-platform desktop UI with egui

---

**Note**: This is a living document and will be updated as the project progresses.
