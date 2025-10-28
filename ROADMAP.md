# Worknest Roadmap

## Product Vision

Worknest is a modern, open-source project and task management system designed specifically for software development teams. Built with Rust for performance and reliability, it combines a powerful WASM-based GUI with a flexible backend architecture that supports local-first workflows and advanced extensibility through plugins.

**Core Principles:**
- **Performance First**: Leverage Rust's speed and safety
- **Local-First**: Work offline with local SQLite, sync when ready
- **Extensible**: Rich plugin system for custom workflows
- **Developer-Friendly**: Built by developers, for developers
- **Cross-Platform**: Desktop apps via egui, web via WASM

---

## Technology Stack

### Core Technologies
- **Language**: Rust (stable)
- **GUI Framework**: egui (immediate mode GUI)
- **WASM Runtime**: wasm-bindgen, wasm-pack
- **Database**: SQLite (local), with migration path to PostgreSQL
- **Authentication**: JWT tokens, bcrypt for password hashing
- **Serialization**: serde (JSON/bincode)
- **Plugin System**: wasmer/wasmtime for WASM plugins

### Project Structure
```
worknest/
├── crates/
│   ├── worknest-core/       # Core business logic, domain models
│   ├── worknest-db/         # Database layer, migrations, repositories
│   ├── worknest-auth/       # Authentication & authorization
│   ├── worknest-api/        # API layer (REST/GraphQL future)
│   ├── worknest-gui/        # egui application
│   ├── worknest-wasm/       # WASM bindings
│   └── worknest-plugins/    # Plugin SDK & runtime
├── plugins/                  # Official plugins
├── migrations/              # Database migrations
└── docs/                    # Documentation
```

---

## Phase 1: MVP (Milestone 1.0)

**Timeline**: 3-4 months
**Goal**: Functional desktop application for basic project and task management

### Features

#### 1.1 Foundation & Authentication
- **User Management**
  - Local user creation and storage
  - Password hashing with bcrypt (cost factor 12)
  - JWT-based session management
  - Simple login/logout flow

- **Database Setup**
  - SQLite database initialization
  - Migration system using refinery or diesel
  - Core schema: users, projects, tickets, comments
  - Indexes for performance

#### 1.2 Project Management
- **Project CRUD**
  - Create, read, update, delete projects
  - Project metadata: name, description, created/updated timestamps
  - Project color coding and icons
  - Archive/unarchive projects

- **Project Organization**
  - List view with search and filter
  - Sort by name, date, status
  - Quick stats: ticket counts, progress indicators

#### 1.3 Ticket Management
- **Ticket System**
  - Create tickets with title, description, status
  - Ticket types: Task, Bug, Feature, Epic
  - Priority levels: Low, Medium, High, Critical
  - Status workflow: Open, In Progress, Review, Done, Closed
  - Assignee support (single user MVP)
  - Due dates and estimates

- **Ticket Views**
  - List view with filtering and sorting
  - Kanban board (drag-and-drop)
  - Ticket detail view with full information
  - Quick edit capabilities

#### 1.4 Basic UI (egui)
- **Application Shell**
  - Main window with menu bar
  - Sidebar navigation (Projects, Tickets, Settings)
  - Top bar with user info and quick actions
  - Responsive layouts

- **Core Screens**
  - Login screen
  - Dashboard (overview of projects and tickets)
  - Project list and detail views
  - Ticket list, board, and detail views
  - Settings panel (theme, preferences)

- **UX Features**
  - Dark/light theme toggle
  - Keyboard shortcuts
  - Toast notifications for actions
  - Form validation and error messages

#### 1.5 Data Layer
- **Repository Pattern**
  - UserRepository
  - ProjectRepository
  - TicketRepository
  - Transaction support

- **Domain Models**
  - Clean separation between DB and domain models
  - Validation logic in domain layer
  - Type-safe IDs (UserId, ProjectId, TicketId)

### Technical Deliverables
- [x] Cargo workspace setup
- [x] SQLite schema and migrations (Complete with V1__initial_schema.sql)
- [x] Authentication module with JWT support (JWT + bcrypt password hashing)
- [x] Core domain models with validation (User, Project, Ticket, Comment, Team, Role, Attachment)
- [x] Repository implementations (Full CRUD with 26 passing tests)
- [x] REST API endpoints (Complete: auth, projects, tickets, comments, attachments)
- [x] egui/eframe application shell (WASM-first web app)
- [x] All MVP screens implemented (Login, Register, Dashboard, Projects, Tickets, Board, Settings)
- [x] Unit tests (wasm-bindgen-test: 19 GUI tests + 26 repository tests = 45 tests)
- [x] API client (Complete with full endpoint coverage)
- [x] Backend API server deployed locally (port 3000)
- [x] Dual-mode support (Demo/Integrated modes)
- [x] Token persistence with localStorage
- [x] Session auto-restore on app load
- [x] API client configured for localhost:3000
- [x] Complete async state management for API calls (Event queue pattern)
- [x] Full frontend-backend integration (auth, CRUD operations)
- [x] Response body decoding fix (UserDto timestamp fields)
- [ ] E2E integration tests
- [x] API documentation (see docs/API_INTEGRATION.md)
- [ ] Build and packaging scripts (Docker, releases)

**Phase 1 Progress: ~95% Complete**
- ✅ Foundation: Authentication, models, app shell (100%)
- ✅ UI: All 9 core screens implemented with theme system (100%)
- ✅ Web Platform: WASM build working with trunk (100%)
- ✅ Testing: 45 passing tests (19 GUI + 26 repository) (90%)
- ✅ Backend: Full REST API with auth middleware (100%)
- ✅ Data Layer: Repository pattern with SQLite (100%)
- ✅ API Client: Complete HTTP client implementation (100%)
- ✅ Backend Deployment: Local server running on port 3000 (100%)
- ✅ Token Persistence: localStorage integration (100%)
- ✅ Integration: Async state management with event queue (100%)
- ✅ API Contract: Request/response structures aligned (100%)
- ✅ Documentation: API integration guide complete (100%)
- 🚧 Packaging: Docker and release automation (0%)

---

## Phase 2: Enhanced Features (Milestone 2.0)

**Timeline**: 2-3 months
**Goal**: Professional-grade features and improved UX

### Features

#### 2.1 Advanced Ticket Features
- **Relations & Dependencies**
  - Parent/child ticket relationships (subtasks)
  - Blocking dependencies
  - Related tickets
  - Dependency graph visualization

- **Rich Content**
  - Markdown support in descriptions
  - File attachments (local storage)
  - Image preview
  - Syntax highlighting for code blocks

- **Activity & History**
  - Ticket change history
  - Comment threads
  - Activity feed per ticket
  - @mentions in comments

#### 2.2 Team Features
- **Multi-User Support**
  - Multiple assignees per ticket
  - User roles: Admin, Member, Viewer
  - Permission system
  - User profiles and avatars

- **Collaboration**
  - Real-time activity indicators
  - Notification system
  - Watching/subscribing to tickets
  - Team dashboards

#### 2.3 Advanced Views
- **Filtering & Search**
  - Advanced filter builder (AND/OR conditions)
  - Saved filters
  - Full-text search across tickets
  - Search indexing

- **Custom Views**
  - Timeline/Gantt chart view
  - Calendar view
  - Custom board configurations
  - View templates

- **Reporting**
  - Burndown charts
  - Velocity tracking
  - Time in status reports
  - Custom report builder

#### 2.4 Workflow Customization
- **Custom Workflows**
  - Define custom statuses
  - Workflow transitions and rules
  - Workflow templates
  - Per-project workflows

- **Custom Fields**
  - Add custom fields to tickets
  - Field types: text, number, date, select, multi-select
  - Required/optional field configuration
  - Default values

#### 2.5 Import/Export
- **Data Portability**
  - Export to JSON, CSV, Markdown
  - Import from CSV, JSON
  - Backup/restore functionality
  - Import from Jira, GitHub Issues, Linear

---

## Phase 3: Plugin System & Extensibility (Milestone 3.0)

**Timeline**: 3-4 months
**Goal**: Open plugin ecosystem for custom integrations

### Features

#### 3.1 Plugin Architecture
- **WASM Plugin Runtime**
  - Sandboxed plugin execution using wasmer
  - Plugin lifecycle: load, initialize, execute, unload
  - Resource limits (memory, CPU)
  - Permission system for plugin capabilities

- **Plugin SDK**
  - Rust SDK for plugin development
  - Plugin manifest schema
  - API documentation
  - Plugin testing framework
  - Example plugins

#### 3.2 Plugin Capabilities
- **Hook System**
  - Lifecycle hooks: ticket created, updated, status changed
  - UI hooks: custom views, panels, buttons
  - Data hooks: validation, transformation
  - Scheduled tasks

- **Plugin APIs**
  - Read/write access to tickets, projects
  - HTTP client for external integrations
  - Key-value storage for plugin data
  - Logging and metrics
  - UI rendering APIs

#### 3.3 Plugin Management
- **UI Integration**
  - Plugin marketplace/browser
  - Install/uninstall plugins
  - Enable/disable plugins
  - Plugin settings UI
  - Plugin update notifications

- **Security**
  - Plugin signing and verification
  - Permission grants by user
  - Audit logs for plugin actions
  - Sandboxing enforcement

#### 3.4 Official Plugins
- **Git Integration**
  - Link commits to tickets
  - Branch naming conventions
  - PR/MR tracking
  - Repository activity feed

- **Time Tracking**
  - Start/stop timers
  - Manual time entry
  - Time reports
  - Billable/non-billable hours

- **CI/CD Integration**
  - GitHub Actions, GitLab CI
  - Build status on tickets
  - Deploy tracking
  - Test result integration

- **Notification Channels**
  - Slack, Discord, MS Teams
  - Email notifications
  - Webhook support
  - Custom notification rules

---

## Phase 4: Scale & Advanced Features (Milestone 4.0)

**Timeline**: 4-5 months
**Goal**: Enterprise-ready with cloud sync and advanced capabilities

### Features

#### 4.1 Cloud Sync & Multi-Device
- **Sync Engine**
  - Conflict-free replicated data types (CRDTs)
  - Offline-first sync protocol
  - Incremental sync
  - Sync status indicators

- **Backend Service**
  - Sync server (optional)
  - PostgreSQL backend option
  - REST API
  - WebSocket for real-time updates

- **Web Application**
  - Full WASM web client
  - Same features as desktop
  - Progressive Web App (PWA)
  - Mobile-responsive design

#### 4.2 Advanced Project Management
- **Portfolio Management**
  - Multiple project hierarchies
  - Cross-project dependencies
  - Resource allocation
  - Portfolio dashboards

- **Roadmapping**
  - Visual roadmap builder
  - Milestone tracking
  - Release planning
  - Version management

- **Agile Workflows**
  - Sprint planning
  - Story points and estimation
  - Retrospective boards
  - Team capacity planning

#### 4.3 Analytics & Insights
- **Advanced Analytics**
  - Custom dashboards
  - Predictive analytics (ML-based ETA)
  - Trend analysis
  - Team performance metrics

- **Automation**
  - Rule-based automation
  - Scheduled actions
  - Auto-assignment rules
  - SLA tracking and escalation

#### 4.4 Enterprise Features
- **Security**
  - SSO/SAML integration
  - LDAP/Active Directory
  - Audit logs
  - Data encryption at rest
  - Role-based access control (RBAC)

- **Administration**
  - Multi-workspace support
  - Workspace templates
  - Bulk operations
  - Data retention policies
  - Compliance tools (GDPR, etc.)

---

## Phase 5: AI & Next-Gen Features (Future)

### Potential Features
- **AI Assistance**
  - Ticket auto-tagging and categorization
  - Smart assignment suggestions
  - Automatic estimation
  - Anomaly detection
  - Natural language ticket creation

- **Advanced Integrations**
  - IDE plugins (VSCode, IntelliJ)
  - Design tool integrations (Figma)
  - Documentation sync (Notion, Confluence)
  - Customer support tools

- **Mobile Apps**
  - Native iOS app
  - Native Android app
  - Mobile-specific workflows

- **Collaboration Features**
  - Video/voice calls
  - Screen sharing
  - Collaborative editing
  - Whiteboarding

---

## Development Principles

### Code Quality
- Comprehensive test coverage (unit, integration, E2E)
- Documentation for all public APIs
- Regular security audits
- Performance benchmarking
- Code review process

### Community
- Open roadmap and RFC process
- Community plugin showcase
- Regular releases (semver)
- Responsive to issues and PRs
- Clear contribution guidelines

### Performance Targets
- **Desktop App**: <50ms UI response time
- **Database**: <10ms query time for standard operations
- **Startup**: <2s cold start
- **Memory**: <200MB baseline usage
- **Plugin Load**: <100ms per plugin

---

## Success Metrics

### MVP Success Criteria
- [ ] 100+ GitHub stars in first month
- [ ] 10+ active users/testers
- [ ] 0 critical bugs
- [ ] Documentation complete
- [ ] Cross-platform builds (Windows, macOS, Linux)

### Phase 2+ Metrics
- Active user growth 20% MoM
- Plugin ecosystem: 10+ community plugins
- Average session time: 30+ minutes
- User retention: 60%+ after 30 days
- Performance: 95th percentile <100ms response time

---

## Release Schedule

### MVP (v1.0.0)
- **Month 1**: Foundation (auth, database, basic models)
- **Month 2**: Core features (projects, tickets)
- **Month 3**: UI polish, testing
- **Month 4**: Documentation, beta release

### Post-MVP
- **v2.0.0**: +3 months (Enhanced features)
- **v3.0.0**: +6 months (Plugin system)
- **v4.0.0**: +10 months (Cloud sync)
- Minor releases monthly with bug fixes and small features

---

## Getting Started with MVP Development

See [ARCHITECTURE.md](./ARCHITECTURE.md) for technical architecture details.

**First Steps:**
1. Set up Rust workspace and crate structure
2. Implement database schema and migrations
3. Build authentication system
4. Create core domain models
5. Implement repositories
6. Build egui application shell
7. Iterate on UI/UX

**Current Status**: Phase 1 - Foundation (~60% Complete)

**Recent Milestones (2024):**
- ✅ WASM-first web application architecture
- ✅ Complete UI implementation with egui/eframe
- ✅ Authentication flow (JWT-based)
- ✅ Core domain models and business logic
- ✅ Demo mode for frontend development
- ✅ Fixed WASM compatibility issues (tracing, time handling)
- ✅ Automated testing infrastructure with wasm-bindgen-test
- ✅ Interactive project cards with hover effects

**Next Steps:**
1. Complete backend API implementation (worknest-api)
2. Database schema and migrations
3. Connect frontend to backend API
4. Implement automated testing suite
5. Documentation and deployment setup
