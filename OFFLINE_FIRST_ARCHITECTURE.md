# Offline-First Architecture for Worknest

## Overview

Worknest will support **offline-first** operation, allowing users to work seamlessly whether connected or disconnected. Data syncs automatically when connection is restored.

## Architecture Strategy

### Simple, Pragmatic Approach

We'll use a **simplified architecture** with minimal dependencies:

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  ┌──────────────┐              ┌──────────────┐            │
│  │  Desktop GUI │              │   Web/WASM   │            │
│  │   (Native)   │              │   (Browser)  │            │
│  └──────┬───────┘              └───────┬──────┘            │
│         │                              │                    │
│         │                              │                    │
│  ┌──────▼────────────────────────────

──▼──────┐            │
│  │      Local Storage / State Management      │            │
│  │                                             │            │
│  │  Desktop: SQLite directly                  │            │
│  │  Web: IndexedDB + API calls                │            │
│  └──────┬────────────────────────────┬────────┘            │
│         │                            │                      │
│         │ (when online)              │ (sync queue)        │
│         │                            │                      │
│  ┌──────▼────────┐          ┌────────▼──────┐             │
│  │  API Server   │◄─────────┤  Sync Service │             │
│  │  (optional)   │          │               │             │
│  └───────────────┘          └───────────────┘             │
└─────────────────────────────────────────────────────────────┘
```

### Key Principles

1. **Local-First**: All data operations work locally first
2. **Sync Later**: Changes queue and sync when online
3. **Conflict Resolution**: Last-write-wins with timestamps
4. **Progressive Enhancement**: Works better when online, but functional offline

## Implementation Plan

### Phase 1: Desktop App (Already Complete)
- ✅ Direct SQLite access
- ✅ No network required
- ✅ Full offline functionality

### Phase 2: Offline-Aware Web App (Next)

#### 2.1 IndexedDB Storage Layer
```rust
// worknest-web crate
pub struct OfflineStore {
    // Uses IndexedDB via web-sys
    // Stores: projects, tickets, comments, attachments (metadata)
}

impl OfflineStore {
    pub async fn save_project(&self, project: &Project) -> Result<()>;
    pub async fn get_project(&self, id: ProjectId) -> Result<Option<Project>>;
    pub async fn list_projects(&self) -> Result<Vec<Project>>;
    // ... CRUD for all entities
}
```

#### 2.2 Sync Queue
```rust
pub struct SyncQueue {
    pending_changes: Vec<Change>,
}

pub struct Change {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub operation: Operation, // Create, Update, Delete
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl SyncQueue {
    pub fn enqueue(&mut self, change: Change);
    pub async fn sync_all(&mut self) -> Result<SyncResult>;
}
```

#### 2.3 Offline Detection
```rust
pub struct ConnectionMonitor {
    is_online: Arc<RwLock<bool>>,
}

impl ConnectionMonitor {
    pub fn new() -> Self;
    pub fn is_online(&self) -> bool;
    pub fn on_online(&self, callback: Box<dyn Fn()>);
    pub fn on_offline(&self, callback: Box<dyn Fn()>);
}
```

### Phase 3: Simplified API Layer

#### 3.1 Minimal REST API (Optional for sync only)
```rust
// worknest-api crate - minimal implementation
use axum::{Router, routing::*};

pub fn create_router() -> Router {
    Router::new()
        .route("/api/sync", post(sync_changes))
        .route("/api/projects", get(list_projects))
        .route("/api/tickets", get(list_tickets))
        // ... basic CRUD endpoints
}
```

**Key Decision**: API server is **optional**. Desktop app doesn't need it.

#### 3.2 Sync Protocol (Simple)
```json
{
  "client_id": "uuid",
  "last_sync": "2025-10-25T10:00:00Z",
  "changes": [
    {
      "id": "change-uuid",
      "type": "project",
      "operation": "create",
      "data": { ... },
      "timestamp": "2025-10-25T10:01:00Z"
    }
  ]
}
```

Server responds with:
```json
{
  "server_changes": [ ... ],
  "conflicts": [ ... ],
  "last_sync": "2025-10-25T10:05:00Z"
}
```

### Phase 4: Service Worker & PWA

#### 4.1 Service Worker for Caching
```javascript
// service-worker.js
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open('worknest-v1').then((cache) => {
      return cache.addAll([
        '/',
        '/index.html',
        '/worknest.wasm',
        '/styles.css',
      ]);
    })
  );
});

self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.match(event.request).then((response) => {
      return response || fetch(event.request);
    })
  );
});
```

#### 4.2 PWA Manifest
```json
{
  "name": "Worknest",
  "short_name": "Worknest",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#1a202c",
  "theme_color": "#3b82f6",
  "icons": [
    {
      "src": "/icon-192.png",
      "sizes": "192x192",
      "type": "image/png"
    },
    {
      "src": "/icon-512.png",
      "sizes": "512x512",
      "type": "image/png"
    }
  ]
}
```

## Conflict Resolution Strategy

### Simple Last-Write-Wins
```rust
pub fn resolve_conflict(local: &Entity, remote: &Entity) -> Entity {
    if local.updated_at > remote.updated_at {
        local.clone()
    } else {
        remote.clone()
    }
}
```

### For Complex Cases (Future)
- Track vector clocks or logical timestamps
- User intervention for critical conflicts
- Merge strategies for specific fields

## Data Storage Limits

### IndexedDB (Web)
- Typical limit: ~50MB to unlimited (browser dependent)
- Chrome: Unlimited with user permission
- Firefox: ~50MB default, more with permission
- Safari: ~1GB

### Strategy
- Store recent data only (last 30 days)
- Archive older data to server
- User can configure retention period

## Implementation Timeline

### Week 1: Core Offline Infrastructure
- ✅ Domain models (Complete)
- ✅ Repositories (Complete)
- ✅ Database migration (Complete)
- ⬜ IndexedDB wrapper for WASM
- ⬜ Sync queue implementation
- ⬜ Connection monitor

### Week 2: WASM Frontend
- ⬜ Port egui GUI to WASM
- ⬜ Implement OfflineStore
- ⬜ Add offline indicators in UI
- ⬜ Test offline scenarios

### Week 3: Sync & API (Optional)
- ⬜ Simple REST API with Axum
- ⬜ Sync protocol implementation
- ⬜ Conflict resolution
- ⬜ Testing sync logic

### Week 4: PWA & Polish
- ⬜ Service worker
- ⬜ PWA manifest
- ⬜ Install prompts
- ⬜ Performance optimization
- ⬜ Documentation

## Technology Choices

### For Web/WASM
- **Storage**: `web-sys` IndexedDB bindings
- **HTTP Client**: `reqwest` with WASM support
- **State Management**: `Rc<RefCell<T>>` (no Arc in WASM)
- **Serialization**: `serde_json`
- **GUI**: `egui` with `eframe` WASM backend

### Simplified Dependencies
```toml
[dependencies]
# Core (already have these)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde", "wasmbind"] }
uuid = { version = "1.6", features = ["v4", "serde", "js"] }

# Web-specific
[target.'cfg(target_arch = "wasm32")'.dependencies]
web-sys = { version = "0.3", features = ["Window", "Storage", "IdbDatabase"] }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
reqwest = { version = "0.11", features = ["json"] }
```

## Testing Strategy

### Offline Scenarios
1. Create project while offline
2. Edit ticket while offline
3. Add comment while offline
4. Go online and verify sync
5. Conflict resolution (edit same ticket on two devices)
6. Network interruption during sync

### Tools
- Manual testing with browser dev tools (offline mode)
- Automated tests with mocked network
- Integration tests for sync protocol

## Security Considerations

### Offline Security
- Data encrypted at rest in IndexedDB
- JWT tokens stored securely (httpOnly cookies for web)
- Sync only with authenticated API

### Data Privacy
- User controls what syncs
- Option to work completely offline (desktop mode)
- Clear data button in settings

## User Experience

### Offline Indicators
```
┌─────────────────────────────────────┐
│ Worknest          🔴 Offline       │  <- Clear indicator
├─────────────────────────────────────┤
│                                     │
│  Projects (3 pending changes)       │  <- Show pending sync count
│                                     │
│  ⚠️  Changes will sync when online  │  <- Reassuring message
│                                     │
└─────────────────────────────────────┘
```

### Sync Progress
```
Syncing... [=====>    ] 50% (5/10 changes)
```

### Conflict Notification
```
⚠️ Conflict detected: Project "Website Redesign"
   Last modified: You (10 mins ago) vs. Alice (5 mins ago)
   [Keep Mine] [Use Theirs] [View Diff]
```

## Benefits

1. **Works Everywhere**: Train, plane, poor network - always functional
2. **Fast**: Local operations are instant
3. **Resilient**: Network issues don't block work
4. **Simple**: Easy to understand and implement
5. **Scalable**: Server is optional, reduces infrastructure costs

## Trade-offs

### Accepted Trade-offs
- **Conflicts Possible**: Last-write-wins may lose some data (rare)
- **Storage Limits**: Can't store unlimited data offline
- **Complexity**: More code than pure client-server

### Mitigated By
- Clear conflict UI for important changes
- Smart data retention policies
- Good documentation and user education

## Future Enhancements

1. **CRDTs**: For better conflict-free merging
2. **Peer-to-Peer**: Direct device sync without server
3. **Selective Sync**: Choose what to store offline
4. **Backup/Export**: Export data to file
5. **Multi-device**: Show which devices have pending changes

---

**Status**: Planning Complete
**Next Step**: Implement IndexedDB storage layer for WASM
**Priority**: High (user requested offline functionality)
