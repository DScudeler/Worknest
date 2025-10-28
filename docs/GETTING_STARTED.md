# Getting Started with Worknest

## Quick Start

### Prerequisites

- Rust 1.75+ (install from [rustup.rs](https://rustup.rs/))
- trunk (install with `cargo install trunk`)
- wasm32 target (`rustup target add wasm32-unknown-unknown`)

### Running the Application

Worknest supports two modes:
- **Demo Mode**: Runs entirely in-browser with mock data
- **Integrated Mode**: Connects to backend API server

#### Option 1: Demo Mode (No Backend Required)

Perfect for trying out the UI or frontend development:

```bash
# Start the frontend
trunk serve --release

# Open browser
http://localhost:8080?mode=demo
```

You can create projects and tickets, and they'll persist in memory during your session.

#### Option 2: Integrated Mode (Full Stack)

Run both backend API and frontend for full functionality:

**Terminal 1 - Backend API:**
```bash
# Build the backend
cargo build --release -p worknest-api

# Run the backend server
./target/release/worknest-api

# Server starts on http://localhost:3000
```

**Terminal 2 - Frontend:**
```bash
# Start the frontend dev server
trunk serve --release

# Frontend available at http://localhost:8080
```

**Browser:**
```
http://localhost:8080  # Auto-connects to backend
```

The application will:
- Automatically connect to the backend API
- Persist your session in localStorage
- Auto-restore your session on page reload

## Development Workflow

### Backend Development

```bash
# Run backend with logging
RUST_LOG=debug ./target/release/worknest-api

# Test backend endpoints
curl http://localhost:3000/health
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"user","email":"user@example.com","password":"pass123"}'
```

### Frontend Development

```bash
# Hot reload dev server
trunk serve

# Release build for production
trunk build --release

# Run frontend tests
wasm-pack test --headless --chrome
```

### Full Stack Development

```bash
# Terminal 1: Backend with auto-restart
cargo watch -x 'run --release -p worknest-api'

# Terminal 2: Frontend with hot reload
trunk serve --release
```

## Project Structure

```
worknest/
├── crates/
│   ├── worknest-core/      # Domain models and business logic
│   ├── worknest-db/         # Database layer and repositories
│   ├── worknest-auth/       # Authentication and JWT
│   ├── worknest-api/        # REST API server (Axum)
│   └── worknest-gui/        # Web UI (egui/WASM)
├── docs/                    # Documentation
├── .env                     # Server configuration
└── Cargo.toml              # Workspace definition
```

## Configuration

### Backend (.env)

```bash
# Server port
PORT=3000

# Database file location
WORKNEST_DB_PATH=./worknest.db

# JWT secret (change in production!)
WORKNEST_SECRET_KEY=your-secret-key-here

# Logging
RUST_LOG=worknest_api=debug,tower_http=debug
```

### Frontend

Frontend configuration is handled through URL parameters:

- `?mode=demo` - Demo mode
- `?mode=integrated` - Integrated mode (default)

## Building for Production

### Backend

```bash
# Build optimized backend
cargo build --release -p worknest-api

# Binary at: target/release/worknest-api
# Deploy with .env file and migrations
```

### Frontend

```bash
# Build optimized WASM bundle
trunk build --release

# Output in: dist/
# Serve dist/ directory with any static file server
```

## Testing

### Backend Tests

```bash
# Run all backend tests
cargo test -p worknest-db -p worknest-auth

# Run specific test suite
cargo test -p worknest-db --lib repository
```

### Frontend Tests

```bash
# Run WASM tests
wasm-pack test --headless --chrome --release

# Run specific test
wasm-pack test --headless --chrome --test login_screen
```

## Troubleshooting

### "Cannot connect to backend"
- Verify backend is running on port 3000
- Check browser console for errors
- Try demo mode to test frontend: `?mode=demo`

### "Port already in use"
- Backend: Change PORT in .env file
- Frontend: Use `trunk serve --port 8081`

### "Build errors"
- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo build`
- Check wasm32 target: `rustup target list --installed`

### "WASM compilation fails"
- Install trunk: `cargo install trunk`
- Add wasm target: `rustup target add wasm32-unknown-unknown`
- Try: `trunk clean && trunk serve`

## Next Steps

- [API Integration Guide](./API_INTEGRATION.md) - Full API documentation
- [Architecture Overview](./ARCHITECTURE.md) - System design
- [Contributing Guide](../CONTRIBUTING.md) - How to contribute

## Features

### Current (Phase 1 MVP)
- ✅ User authentication (register/login/logout)
- ✅ Project management (CRUD operations)
- ✅ Ticket management (CRUD operations)
- ✅ Kanban board view
- ✅ Dashboard with statistics
- ✅ Dark/light theme
- ✅ Demo mode for offline usage
- ✅ Token persistence across sessions

### Coming Soon (Phase 2)
- Real-time updates
- Comments and attachments
- Advanced filtering
- Ticket dependencies
- Team collaboration
- Reporting and analytics

## Support

For issues or questions:
- GitHub Issues: [Worknest Issues](https://github.com/DScudeler/Worknest/issues)
- Documentation: [docs/](../docs/)
- Roadmap: [ROADMAP.md](../ROADMAP.md)
