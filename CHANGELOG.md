# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Project Infrastructure
- Initial project structure with Rust workspace
- Comprehensive project roadmap (ROADMAP.md) - 5 development phases
- Technical architecture documentation (ARCHITECTURE.md)
- Contribution guidelines (CONTRIBUTING.md)
- Code of Conduct
- Security Policy
- GitHub Actions CI/CD pipeline
  - Test suite automation
  - Code formatting checks (cargo fmt)
  - Clippy linting with warnings as errors
  - Security audit (cargo audit)
  - Code coverage reporting (cargo-tarpaulin)
  - Release automation
  - Dependency management
- Development tools
  - Makefile with 15+ common commands
  - rustfmt configuration (stable features)
  - clippy configuration
  - EditorConfig for consistent formatting
- GitHub templates
  - Pull request template with quality checklist
  - Bug report template
  - Feature request template
- Dependabot configuration for automatic dependency updates

#### Core Domain Models (worknest-core)
- User entity with validation
- Project entity with archive/unarchive
- Ticket entity with full workflow support
- Type-safe ID types (UserId, ProjectId, TicketId)
- Enums: TicketType, TicketStatus, Priority
- 14 unit tests covering all domain logic

#### Database Layer (worknest-db)
- SQLite connection pooling (r2d2)
- Foreign key constraints enabled by default
- Embedded migrations system (refinery)
- Initial schema migration (V1__initial_schema.sql)
  - 5 tables: users, projects, tickets, comments, sessions
  - 15 indexes for query optimization
  - Foreign key relationships with cascading deletes
- Repository pattern implementation
  - Generic Repository<T, ID> trait
  - UserRepository with password management
  - ProjectRepository with archive support
  - TicketRepository with rich queries
- 23 comprehensive integration tests
- In-memory database support for testing

#### Authentication System (worknest-auth)
- Password hashing with bcrypt (cost: 12)
- Password strength validation (8-72 characters)
- JWT token generation and verification
- Token refresh functionality
- Configurable token expiration (default: 24 hours)
- Authentication service with:
  - User registration with duplicate prevention
  - Login with username or email
  - Token verification
  - User extraction from token
  - Password change with verification
- 26 comprehensive tests covering all auth flows

#### GUI Application (worknest-gui)
- Basic egui application shell
- Welcome screen placeholder
- Application icon support
- Logging infrastructure

### Changed
- Nothing yet

### Deprecated
- Nothing yet

### Removed
- Nothing yet

### Fixed
- Nothing yet

### Security
- Bcrypt password hashing with cost factor 12
- JWT-based authentication with HS256 signing
- Token expiration enforced (24 hour default)
- SQL injection prevention via parameterized queries
- Foreign key constraints for data integrity
- Password strength requirements
- Username/email uniqueness enforced

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
