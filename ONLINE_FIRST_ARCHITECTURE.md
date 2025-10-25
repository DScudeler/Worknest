# Online-First Architecture with Offline Complement

## Overview

Worknest is **online-first** - it works best when connected to the server, with **graceful offline fallback** when connection is temporarily lost.

## Architecture

### Simple Client-Server Model

```
┌─────────────────────────────────────────────────────────┐
│                    Clients                              │
│                                                          │
│  ┌──────────────┐              ┌──────────────┐        │
│  │  Desktop GUI │              │   Web/WASM   │        │
│  │              │              │              │        │
│  │   SQLite     │              │  IndexedDB   │        │
│  │  (Primary)   │              │   (Cache)    │        │
│  └──────┬───────┘              └───────┬──────┘        │
│         │                              │                │
│         │ Optional sync                │ HTTP/WS        │
│         ▼                              ▼                │
│  ┌─────────────────────────────────────────────┐       │
│  │           REST API Server (Axum)            │       │
│  │                                             │       │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐ │       │
│  │  │   Auth   │  │  Routes  │  │WebSocket │ │       │
│  │  │Middleware│  │  /api/*  │  │  /ws     │ │       │
│  │  └──────────┘  └──────────┘  └──────────┘ │       │
│  │                                             │       │
│  │  ┌─────────────────────────────────────┐   │       │
│  │  │      Repositories (Business Logic)  │   │       │
│  │  └─────────────────────────────────────┘   │       │
│  │                                             │       │
│  │  ┌─────────────────────────────────────┐   │       │
│  │  │         SQLite Database             │   │       │
│  │  └─────────────────────────────────────┘   │       │
│  └─────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────┘
```

## Modes of Operation

### 1. Desktop App (Primary Mode)
- **Direct SQLite access** - no server needed
- **Fully offline** - works completely standalone
- **Optional sync** - can sync with server if available
- **Best performance** - no network latency

### 2. Web App (Requires Server)
- **Online-first** - connects to API server
- **HTTP/REST** - standard CRUD operations via API
- **WebSocket** - real-time updates (optional)
- **Cached data** - IndexedDB caches for brief disconnections
- **Reconnection** - automatic retry when connection restored

## Implementation Plan (Simplified)

### Phase 1: REST API Server
```rust
// worknest-api/src/main.rs
use axum::{Router, routing::*};

#[tokio::main]
async fn main() {
    let app = Router::new()
        // Auth
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))

        // Projects
        .route("/api/projects", get(projects::list).post(projects::create))
        .route("/api/projects/:id", get(projects::get).put(projects::update).delete(projects::delete))

        // Tickets
        .route("/api/tickets", get(tickets::list).post(tickets::create))
        .route("/api/tickets/:id", get(tickets::get).put(tickets::update).delete(tickets::delete))

        // Comments
        .route("/api/tickets/:id/comments", get(comments::list).post(comments::create))
        .route("/api/comments/:id", put(comments::update).delete(comments::delete))

        // Attachments
        .route("/api/tickets/:id/attachments", get(attachments::list).post(attachments::upload))
        .route("/api/attachments/:id", get(attachments::download).delete(attachments::delete))

        // WebSocket (optional)
        .route("/ws", get(websocket::handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Phase 2: WASM Web Client
```rust
// worknest-web/src/lib.rs
use reqwest::Client;

pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    pub async fn login(&mut self, username: &str, password: &str) -> Result<User> {
        let resp = self.client
            .post(&format!("{}/api/auth/login", self.base_url))
            .json(&json!({ "username": username, "password": password }))
            .send()
            .await?;

        let auth: AuthResponse = resp.json().await?;
        self.token = Some(auth.token);
        Ok(auth.user)
    }

    pub async fn get_projects(&self) -> Result<Vec<Project>> {
        let resp = self.client
            .get(&format!("{}/api/projects", self.base_url))
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await?;

        resp.json().await
    }

    // ... other methods
}
```

### Phase 3: Offline Cache (Simple)
```rust
// worknest-web/src/cache.rs
pub struct OfflineCache {
    // Simple in-memory cache with IndexedDB persistence
    projects: Vec<Project>,
    tickets: HashMap<ProjectId, Vec<Ticket>>,
    last_updated: DateTime<Utc>,
}

impl OfflineCache {
    pub fn update_projects(&mut self, projects: Vec<Project>) {
        self.projects = projects;
        self.save_to_indexeddb(); // Async persist
    }

    pub fn get_projects(&self) -> &[Project] {
        &self.projects
    }

    // When offline, serve from cache with warning
    pub fn is_stale(&self) -> bool {
        Utc::now() - self.last_updated > Duration::minutes(5)
    }
}
```

### Phase 4: Connection Monitoring
```rust
pub struct ConnectionStatus {
    is_online: bool,
    last_check: DateTime<Utc>,
}

impl ConnectionStatus {
    pub fn check_online(&mut self) {
        // Browser API: navigator.onLine
        // Or: try a ping request to server
        self.is_online = window().navigator().on_line();
    }

    pub fn on_reconnect(&self, callback: impl Fn()) {
        // Retry queued requests
        // Refresh cached data
        callback();
    }
}
```

## Offline Behavior (Graceful Degradation)

### When Connection Lost
1. **Show offline indicator**: "⚠️ Connection lost - showing cached data"
2. **Disable write operations**: Show message "Reconnect to make changes"
3. **Serve from cache**: Display last-known data
4. **Auto-retry**: Poll for connection every 10 seconds

### When Reconnected
1. **Show online indicator**: "✓ Connected"
2. **Refresh data**: Fetch latest from server
3. **Re-enable writes**: User can make changes again
4. **Show notification**: "Data refreshed"

## Technology Stack

### API Server (worknest-api)
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"

# Reuse existing crates
worknest-core = { path = "../worknest-core" }
worknest-db = { path = "../worknest-db" }
worknest-auth = { path = "../worknest-auth" }
```

### Web Client (worknest-web)
```toml
[dependencies]
egui = "0.24"
eframe = { version = "0.24", features = ["default_fonts", "wasm_gl"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# WASM-specific
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = ["Window", "Navigator", "Storage"] }
```

## API Endpoints

### Authentication
- `POST /api/auth/register` - Create account
- `POST /api/auth/login` - Get JWT token
- `POST /api/auth/refresh` - Refresh token
- `POST /api/auth/logout` - Invalidate token

### Projects
- `GET /api/projects` - List all projects
- `GET /api/projects/:id` - Get project details
- `POST /api/projects` - Create project
- `PUT /api/projects/:id` - Update project
- `DELETE /api/projects/:id` - Delete project
- `POST /api/projects/:id/archive` - Archive project

### Tickets
- `GET /api/tickets` - List all tickets (with filters)
- `GET /api/tickets/:id` - Get ticket details
- `POST /api/tickets` - Create ticket
- `PUT /api/tickets/:id` - Update ticket
- `DELETE /api/tickets/:id` - Delete ticket
- `PUT /api/tickets/:id/status` - Update status

### Comments
- `GET /api/tickets/:id/comments` - List comments
- `POST /api/tickets/:id/comments` - Add comment
- `PUT /api/comments/:id` - Update comment
- `DELETE /api/comments/:id` - Delete comment

### Attachments
- `GET /api/tickets/:id/attachments` - List attachments
- `POST /api/tickets/:id/attachments` - Upload file
- `GET /api/attachments/:id/download` - Download file
- `DELETE /api/attachments/:id` - Delete attachment

### WebSocket (Optional - Real-time)
- `WS /ws` - Real-time updates
  - `ticket.created`
  - `ticket.updated`
  - `comment.added`
  - etc.

## Security

### Authentication
- JWT tokens in Authorization header: `Bearer <token>`
- Token expiration: 24 hours
- Refresh tokens: 7 days
- HTTPS only in production

### CORS
```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(Any) // Configure per environment
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);
```

### File Upload
- Max size: 10MB per file
- Allowed types: images, PDFs, docs
- Virus scanning (future)
- Stored in: `./uploads/` directory

## Deployment

### Development
```bash
# Terminal 1: API Server
cd worknest
cargo run --bin worknest-api

# Terminal 2: Web Client (dev server)
cd worknest-web
trunk serve

# Desktop
cargo run -p worknest-gui
```

### Production

**API Server**:
```bash
cargo build --release --bin worknest-api
./target/release/worknest-api
```

**Web Client**:
```bash
cd worknest-web
trunk build --release
# Deploy dist/ to static hosting (Netlify, Vercel, etc.)
```

**Desktop**:
```bash
cargo build --release -p worknest-gui
# Distribute binary
```

## Benefits of This Approach

1. **Simple**: Standard client-server, well-understood
2. **Fast Development**: Less complexity than offline-first
3. **Reliable**: Server is source of truth
4. **Scalable**: Easy to add more features
5. **Desktop Works Standalone**: No server needed for single-user

## Trade-offs

### Accepted
- **Requires connection**: Web app needs internet (acceptable)
- **Server dependency**: Need to run API server (standard)
- **No offline writes**: Can't edit when disconnected (rare)

### Mitigated
- **Desktop mode**: Full offline for single-user
- **Connection lost**: Clear UI feedback
- **Quick reconnect**: Auto-retry every 10s

## Implementation Timeline

### Week 1: API Server
- ✅ Domain models (Complete)
- ✅ Repositories (Complete)
- ⬜ Axum routes for all CRUD
- ⬜ JWT authentication middleware
- ⬜ Error handling & logging
- ⬜ Test with Postman/curl

### Week 2: WASM Client
- ⬜ Setup trunk & WASM build
- ⬜ API client library
- ⬜ Port GUI screens to WASM
- ⬜ Connection status indicator
- ⬜ Basic caching

### Week 3: Features & Polish
- ⬜ File upload/download
- ⬜ Real-time updates (WebSocket)
- ⬜ Offline cache improvements
- ⬜ Desktop GUI Phase 2 features

### Week 4: Testing & Deployment
- ⬜ Integration tests
- ⬜ Load testing
- ⬜ Deployment scripts
- ⬜ Documentation

## Next Steps

1. **Start with API Server**: Implement basic Axum routes
2. **Test with Desktop**: Desktop can optionally connect to API
3. **Build WASM Client**: Port GUI to web with API calls
4. **Add offline cache**: Simple read-only cache for disconnections

---

**Architecture**: Online-first with offline fallback
**Priority**: API server → WASM client → Offline complement
**Status**: Ready to implement
