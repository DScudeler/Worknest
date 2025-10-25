# Worknest Architecture

## Overview

Worknest follows a layered architecture with clear separation of concerns. The application is built as a Rust workspace with multiple crates, each responsible for a specific domain.

```
┌─────────────────────────────────────────────────────────────┐
│                     egui Desktop App                        │
│                   (worknest-gui)                            │
├─────────────────────────────────────────────────────────────┤
│                    Application Layer                        │
│              (State Management, UI Logic)                   │
├─────────────────────────────────────────────────────────────┤
│                      API Layer                              │
│                  (worknest-api)                             │
│         (Commands, Queries, DTOs)                           │
├─────────────────────────────────────────────────────────────┤
│                   Business Logic Layer                      │
│                   (worknest-core)                           │
│        (Domain Models, Services, Validators)                │
├─────────────────────────────────────────────────────────────┤
│     Authentication          │      Plugin System            │
│    (worknest-auth)          │   (worknest-plugins)          │
├─────────────────────────────┴───────────────────────────────┤
│                   Data Access Layer                         │
│                    (worknest-db)                            │
│          (Repositories, Migrations, Schema)                 │
├─────────────────────────────────────────────────────────────┤
│                      SQLite Database                        │
└─────────────────────────────────────────────────────────────┘
```

---

## Crate Structure

### worknest-core
**Purpose**: Core business logic and domain models

**Responsibilities**:
- Define domain entities (User, Project, Ticket, Comment)
- Business rules and validation
- Domain services
- Type-safe IDs
- Error types

**Key Types**:
```rust
// Domain IDs
pub struct UserId(Uuid);
pub struct ProjectId(Uuid);
pub struct TicketId(Uuid);

// Domain Models
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub archived: bool,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Ticket {
    pub id: TicketId,
    pub project_id: ProjectId,
    pub title: String,
    pub description: Option<String>,
    pub ticket_type: TicketType,
    pub status: TicketStatus,
    pub priority: Priority,
    pub assignee_id: Option<UserId>,
    pub created_by: UserId,
    pub due_date: Option<DateTime<Utc>>,
    pub estimate_hours: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Enums
pub enum TicketType { Task, Bug, Feature, Epic }
pub enum TicketStatus { Open, InProgress, Review, Done, Closed }
pub enum Priority { Low, Medium, High, Critical }
```

**Dependencies**:
- `serde` - Serialization
- `uuid` - Unique identifiers
- `chrono` - Date/time handling
- `thiserror` - Error handling
- `validator` - Input validation

---

### worknest-db
**Purpose**: Database access and persistence

**Responsibilities**:
- Database connection management
- Schema definitions
- Migrations
- Repository implementations
- Query builders

**Technology**:
- SQLite via `rusqlite` or `sqlx`
- Migrations via `refinery` or embedded SQL

**Repository Pattern**:
```rust
pub trait Repository<T, ID> {
    fn find_by_id(&self, id: ID) -> Result<Option<T>>;
    fn find_all(&self) -> Result<Vec<T>>;
    fn create(&self, entity: &T) -> Result<T>;
    fn update(&self, entity: &T) -> Result<T>;
    fn delete(&self, id: ID) -> Result<()>;
}

pub struct UserRepository {
    pool: Arc<Pool>,
}

impl Repository<User, UserId> for UserRepository {
    // Implementation
}

// Similar for ProjectRepository, TicketRepository, CommentRepository
```

**Key Features**:
- Connection pooling
- Transaction support
- Prepared statements
- Indexed queries
- Full-text search support

**Dependencies**:
- `rusqlite` or `sqlx` - Database driver
- `r2d2` - Connection pooling
- `refinery` - Migrations
- `worknest-core` - Domain models

---

### worknest-auth
**Purpose**: Authentication and authorization

**Responsibilities**:
- User registration and login
- Password hashing and verification
- JWT token generation and validation
- Session management
- Permission checking

**Authentication Flow**:
```rust
pub struct AuthService {
    user_repo: Arc<UserRepository>,
    secret_key: String,
}

impl AuthService {
    pub fn register(&self, username: &str, email: &str, password: &str)
        -> Result<User>;

    pub fn login(&self, username: &str, password: &str)
        -> Result<AuthToken>;

    pub fn verify_token(&self, token: &str)
        -> Result<Claims>;

    pub fn logout(&self, token: &str)
        -> Result<()>;
}

pub struct AuthToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

pub struct Claims {
    pub user_id: UserId,
    pub username: String,
    pub exp: i64,
}
```

**Security**:
- Bcrypt password hashing (cost: 12)
- JWT with HS256 signing
- Token expiration (default: 24 hours)
- Refresh token support (future)
- Rate limiting on login attempts

**Dependencies**:
- `bcrypt` - Password hashing
- `jsonwebtoken` - JWT handling
- `rand` - Token generation
- `worknest-core` - Domain models
- `worknest-db` - User repository

---

### worknest-api
**Purpose**: Application API layer (CQRS pattern)

**Responsibilities**:
- Command handlers (write operations)
- Query handlers (read operations)
- DTOs (Data Transfer Objects)
- Request validation
- Response formatting

**Command Pattern**:
```rust
// Commands (write operations)
pub struct CreateProjectCommand {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub user_id: UserId,
}

pub struct CreateTicketCommand {
    pub project_id: ProjectId,
    pub title: String,
    pub description: Option<String>,
    pub ticket_type: TicketType,
    pub priority: Priority,
    pub assignee_id: Option<UserId>,
    pub user_id: UserId,
}

// Queries (read operations)
pub struct GetProjectQuery {
    pub project_id: ProjectId,
}

pub struct ListTicketsQuery {
    pub project_id: Option<ProjectId>,
    pub status: Option<TicketStatus>,
    pub assignee_id: Option<UserId>,
    pub limit: usize,
    pub offset: usize,
}

// Handlers
pub struct ProjectCommandHandler {
    project_repo: Arc<ProjectRepository>,
}

impl ProjectCommandHandler {
    pub fn handle_create(&self, cmd: CreateProjectCommand)
        -> Result<ProjectDto>;
}
```

**Benefits**:
- Clear separation of reads and writes
- Easy to test
- Prepare for CQRS/Event Sourcing future
- Type-safe API

**Dependencies**:
- `worknest-core` - Domain models
- `worknest-db` - Repositories
- `worknest-auth` - Authentication
- `serde` - Serialization

---

### worknest-gui
**Purpose**: Desktop application UI (egui)

**Responsibilities**:
- Application state management
- Screen/view implementations
- User interactions
- Theme management
- Keyboard shortcuts

**Application Structure**:
```rust
pub struct WorknestApp {
    state: AppState,
    screens: ScreenManager,
    theme: Theme,
    auth_service: Arc<AuthService>,
    api: Api,
}

pub struct AppState {
    pub current_user: Option<User>,
    pub auth_token: Option<AuthToken>,
    pub current_screen: Screen,
    pub notifications: Vec<Notification>,
}

pub enum Screen {
    Login,
    Dashboard,
    ProjectList,
    ProjectDetail(ProjectId),
    TicketList { project_id: Option<ProjectId> },
    TicketBoard { project_id: ProjectId },
    TicketDetail(TicketId),
    Settings,
}

impl eframe::App for WorknestApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Main UI loop
        self.render_top_bar(ctx);
        self.render_sidebar(ctx);
        self.render_main_content(ctx);
        self.render_notifications(ctx);
    }
}
```

**Screen Implementations**:
Each screen is a separate module with consistent interface:
```rust
pub trait ScreenView {
    fn render(&mut self, ctx: &egui::Context, state: &mut AppState, api: &Api);
}

pub struct LoginScreen { /* fields */ }
pub struct DashboardScreen { /* fields */ }
pub struct ProjectListScreen { /* fields */ }
pub struct TicketBoardScreen { /* fields */ }
// etc.
```

**Features**:
- Dark/light theme switching
- Responsive layouts
- Drag-and-drop support (Kanban)
- Modal dialogs
- Toast notifications
- Loading states
- Error handling

**Dependencies**:
- `eframe` - egui framework
- `egui` - Immediate mode GUI
- `egui_extras` - Additional widgets
- `rfd` - File dialogs
- `worknest-api` - API layer
- `worknest-auth` - Authentication
- `worknest-core` - Domain models

---

### worknest-wasm (Future)
**Purpose**: WASM bindings for web application

**Responsibilities**:
- Web-compatible API bindings
- Browser storage integration
- WASM-specific optimizations

**Dependencies**:
- `wasm-bindgen`
- `web-sys`
- `wasm-pack`

---

### worknest-plugins (Phase 3)
**Purpose**: Plugin system runtime and SDK

**Responsibilities**:
- Plugin loading and execution
- Sandbox management
- Plugin API implementation
- Permission system

**Dependencies**:
- `wasmer` or `wasmtime`
- `wasmtime-wasi`

---

## Database Schema

### Users Table
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
```

### Projects Table
```sql
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,
    archived INTEGER NOT NULL DEFAULT 0,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX idx_projects_created_by ON projects(created_by);
CREATE INDEX idx_projects_archived ON projects(archived);
```

### Tickets Table
```sql
CREATE TABLE tickets (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    ticket_type TEXT NOT NULL,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    assignee_id TEXT,
    created_by TEXT NOT NULL,
    due_date TEXT,
    estimate_hours REAL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (assignee_id) REFERENCES users(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE INDEX idx_tickets_project_id ON tickets(project_id);
CREATE INDEX idx_tickets_assignee_id ON tickets(assignee_id);
CREATE INDEX idx_tickets_status ON tickets(status);
CREATE INDEX idx_tickets_created_by ON tickets(created_by);
CREATE INDEX idx_tickets_due_date ON tickets(due_date);
```

### Comments Table
```sql
CREATE TABLE comments (
    id TEXT PRIMARY KEY,
    ticket_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (ticket_id) REFERENCES tickets(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_comments_ticket_id ON comments(ticket_id);
CREATE INDEX idx_comments_user_id ON comments(user_id);
```

### Sessions Table (for auth tokens)
```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_sessions_token ON sessions(token);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
```

---

## Data Flow

### Creating a Ticket (Write Operation)

```
User Input (GUI)
    ↓
CreateTicketCommand
    ↓
TicketCommandHandler
    ↓
Validation (worknest-core)
    ↓
TicketRepository (worknest-db)
    ↓
SQLite Database
    ↓
Return TicketDto
    ↓
Update UI State
```

### Loading Tickets (Read Operation)

```
User Navigation (GUI)
    ↓
ListTicketsQuery
    ↓
TicketQueryHandler
    ↓
TicketRepository (worknest-db)
    ↓
SQLite Query with Filters
    ↓
Map to TicketDto
    ↓
Return List<TicketDto>
    ↓
Render in UI
```

---

## Error Handling

### Error Type Hierarchy
```rust
// Core errors
pub enum CoreError {
    Validation(ValidationError),
    NotFound(String),
    Unauthorized,
    Conflict(String),
}

// Database errors
pub enum DbError {
    Connection(String),
    Query(String),
    Migration(String),
    Transaction(String),
}

// Auth errors
pub enum AuthError {
    InvalidCredentials,
    TokenExpired,
    TokenInvalid,
    UserExists,
}

// Application error (combines all)
pub enum AppError {
    Core(CoreError),
    Database(DbError),
    Auth(AuthError),
    Unknown(String),
}
```

### Error Propagation
- Use `Result<T, E>` throughout
- Convert errors at boundary layers
- Log errors with context
- Display user-friendly messages in GUI

---

## State Management

### Application State
```rust
pub struct AppState {
    // Authentication
    pub current_user: Option<User>,
    pub auth_token: Option<AuthToken>,

    // Navigation
    pub current_screen: Screen,
    pub navigation_stack: Vec<Screen>,

    // Data Cache (for performance)
    pub projects_cache: HashMap<ProjectId, Project>,
    pub tickets_cache: HashMap<TicketId, Ticket>,
    pub users_cache: HashMap<UserId, User>,

    // UI State
    pub theme: Theme,
    pub notifications: VecDeque<Notification>,
    pub loading_states: HashMap<String, bool>,

    // Modal/Dialog State
    pub active_modal: Option<Modal>,
    pub form_states: HashMap<String, FormState>,
}
```

### State Updates
- Single source of truth (AppState)
- Immutable updates where possible
- State changes trigger UI re-render
- Cache invalidation on data changes

---

## Testing Strategy

### Unit Tests
- **Coverage**: >70% for core, db, auth crates
- **Focus**: Business logic, validators, utilities
- **Tools**: `cargo test`, `proptest` for property tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_validation() {
        let ticket = Ticket {
            title: "".to_string(), // Invalid: empty
            // ...
        };
        assert!(ticket.validate().is_err());
    }
}
```

### Integration Tests
- **Coverage**: Repository operations, auth flow
- **Focus**: Database interactions, multi-layer flows
- **Setup**: In-memory SQLite for tests

```rust
#[test]
fn test_create_and_retrieve_ticket() {
    let db = setup_test_db();
    let repo = TicketRepository::new(db);

    let ticket = create_test_ticket();
    let saved = repo.create(&ticket).unwrap();
    let retrieved = repo.find_by_id(saved.id).unwrap();

    assert_eq!(saved, retrieved);
}
```

### UI Tests
- **Coverage**: Screen rendering, user interactions
- **Focus**: UI state changes, navigation
- **Tools**: Manual testing + screenshot tests (future)

### End-to-End Tests
- **Coverage**: Complete user workflows
- **Focus**: Login → Create Project → Create Ticket → Update Status
- **Tools**: Automated UI testing (future)

---

## Build and Deployment

### Development Build
```bash
# Run desktop app
cargo run -p worknest-gui

# Run tests
cargo test --workspace

# Check formatting
cargo fmt --check

# Linting
cargo clippy -- -D warnings
```

### Release Build
```bash
# Optimized build
cargo build --release -p worknest-gui

# Platform-specific
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu
```

### Packaging
- **Windows**: Create installer with NSIS or WiX
- **macOS**: Bundle as .app with icon
- **Linux**: AppImage, .deb, and .rpm packages

### CI/CD Pipeline
```yaml
# GitHub Actions
- name: Test
  run: cargo test --workspace

- name: Build
  run: cargo build --release

- name: Package
  run: ./scripts/package.sh

- name: Release
  uses: actions/upload-artifact@v2
```

---

## Performance Considerations

### Database
- Use indexes on foreign keys and commonly queried fields
- Prepare and cache common queries
- Use connection pooling
- Batch operations where possible
- VACUUM database periodically

### GUI
- Lazy loading for large lists
- Virtual scrolling for >100 items
- Debounce search inputs
- Cache rendered layouts
- Minimize re-renders with memoization

### Memory
- Use Arc<T> for shared references
- Clear caches periodically
- Limit cache sizes
- Profile with `cargo-profiler`

### Startup Time
- Lazy initialization
- Background loading
- Splash screen during init

---

## Configuration

### Application Config
```toml
# worknest.toml
[database]
path = "~/.worknest/worknest.db"
auto_vacuum = true

[auth]
token_expiration_hours = 24
bcrypt_cost = 12

[ui]
theme = "dark"  # or "light"
enable_animations = true
default_view = "board"  # or "list"

[performance]
cache_size_mb = 50
enable_profiling = false
```

### Environment Variables
```bash
WORKNEST_DB_PATH=/custom/path/worknest.db
WORKNEST_LOG_LEVEL=info  # error, warn, info, debug, trace
WORKNEST_SECRET_KEY=your-secret-key-for-jwt
```

---

## Security Considerations

### Data at Rest
- Database file permissions (user-only read/write)
- Consider encryption for sensitive data (future)

### Data in Transit
- Not applicable for MVP (local only)
- HTTPS for cloud sync (Phase 4)

### Authentication
- No password storage (only hash)
- Salt included in bcrypt
- Token expiration enforced
- Session invalidation on logout

### Input Validation
- Validate at API boundary
- Sanitize user inputs
- Parameterized queries (prevent SQL injection)
- Length limits on all text fields

### Dependency Security
- Regular `cargo audit` runs
- Keep dependencies updated
- Minimal dependency tree
- Audit critical dependencies

---

## Logging and Monitoring

### Logging Strategy
```rust
use tracing::{info, warn, error};

// Structured logging
info!(user_id = %user.id, "User logged in");
error!(error = %e, "Failed to create ticket");

// Different log levels
// - ERROR: Errors requiring attention
// - WARN: Potential issues
// - INFO: Important events
// - DEBUG: Detailed debugging info
// - TRACE: Very verbose debugging
```

### Log Configuration
- File rotation (max 10MB per file, keep 5 files)
- Separate error log file
- Configurable log level
- Performance impact minimal

---

## Future Architecture Considerations

### Plugin System (Phase 3)
- WASM sandbox for plugins
- IPC for plugin communication
- Versioned plugin API
- Plugin discovery service

### Cloud Sync (Phase 4)
- Event sourcing architecture
- CQRS pattern
- Conflict resolution with CRDTs
- WebSocket for real-time updates
- Separate sync service

### Microservices (Far Future)
- Split into services (auth, projects, tickets, sync)
- gRPC for inter-service communication
- Message queue for events
- Distributed tracing

---

## Appendix: Key Dependencies

### Core Crates
- `serde` (1.0+) - Serialization
- `uuid` (1.0+) - Unique IDs
- `chrono` (0.4+) - Date/time
- `thiserror` (1.0+) - Error handling
- `anyhow` (1.0+) - Error context

### Database
- `rusqlite` (0.29+) or `sqlx` (0.7+)
- `r2d2` (0.8+) - Connection pooling
- `refinery` (0.8+) - Migrations

### Auth
- `bcrypt` (0.15+)
- `jsonwebtoken` (8.0+)
- `rand` (0.8+)

### GUI
- `eframe` (0.23+)
- `egui` (0.23+)
- `egui_extras` (0.23+)

### Logging
- `tracing` (0.1+)
- `tracing-subscriber` (0.3+)

### Testing
- `mockall` (0.11+) - Mocking
- `proptest` (1.0+) - Property testing
- `rstest` (0.18+) - Parameterized tests
