# Worknest

**An open-source project and task manager built for software development teams**

[![Deploy to GitHub Pages](https://github.com/DScudeler/Worknest/actions/workflows/deploy-pages.yml/badge.svg)](https://github.com/DScudeler/Worknest/actions/workflows/deploy-pages.yml)

Worknest is a modern, high-performance project management tool built entirely in Rust. It provides a responsive web application that works seamlessly on both desktop and mobile browsers, giving developers a fast, reliable, and extensible platform for managing projects and tasks.

## 🚀 Try the Demo

**[Launch Worknest Demo](https://dscudeler.github.io/Worknest/)** _(once GitHub Pages is enabled)_

The demo runs entirely in your browser with no backend required! All data is stored locally in your browser's localStorage.

## Features (Planned)

### MVP (v1.0)
- User authentication and session management
- Project management (create, organize, archive)
- Comprehensive ticket system (tasks, bugs, features)
- Multiple views: List, Kanban board
- Priority and status tracking
- Responsive web UI powered by egui
- Works on desktop and mobile browsers

### Future Releases
- **v2.0**: Advanced features (custom fields, workflows, reporting)
- **v3.0**: Plugin system with WASM-based extensibility
- **v4.0**: Cloud sync and collaboration features
- **v5.0+**: AI assistance, progressive web app, advanced integrations

See [ROADMAP.md](./ROADMAP.md) for the complete development plan.

## Why Worknest?

- **Performance**: Built in Rust for maximum speed and efficiency
- **Responsive**: Works seamlessly on desktop and mobile browsers
- **Developer-Focused**: Built by developers, for developers
- **Extensible**: Plugin system for custom integrations (coming in v3.0)
- **Open Source**: Free forever, community-driven development

## Architecture

Worknest follows a modular architecture with clear separation of concerns:

```
worknest/
├── crates/
│   ├── worknest-core/       # Core business logic
│   ├── worknest-db/         # Database layer (SQLite)
│   ├── worknest-auth/       # Authentication
│   ├── worknest-api/        # Backend API server
│   ├── worknest-gui/        # Web UI (egui/WASM)
│   └── worknest-plugins/    # Plugin system (future)
```

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed technical documentation.

## Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) for building the web application (`cargo install wasm-pack`)
- wasm32 target (`rustup target add wasm32-unknown-unknown`)
- Python 3 (for local development server)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/DScudeler/Worknest.git
cd Worknest

# Build the web application (simple one-command build)
make build-webapp

# Or use the build script directly
./build-webapp.sh release

# The built files will be in the dist/ directory
```

### Development

**Demo Mode**: The webapp runs in full demo mode with in-memory data - no backend required! Perfect for trying out features and UI development.

```bash
# Build and serve the web application locally
make serve-webapp

# Or manually:
./build-webapp.sh release
cd dist && python3 serve.py

# The application will be available at http://localhost:8080

# Run tests (backend only)
make test

# Or:
cargo test --workspace --exclude worknest-gui

# Check code formatting
make fmt

# Run linter
make clippy

# Quick check (format + lint + test)
make quick-check
```

**Available Make Commands:**
```bash
make help              # Show all available commands
make build-webapp      # Build webapp in production mode
make serve-webapp      # Build and serve webapp locally
make test              # Run all tests
make fmt               # Format code
make clippy            # Run linter
make clean             # Clean build artifacts
```

## Deploying to GitHub Pages

To enable the live demo on GitHub Pages:

1. **Enable GitHub Pages in your repository:**
   - Go to your repository Settings → Pages
   - Under "Source", select "GitHub Actions"
   - Save the settings

2. **The workflow will automatically deploy when you push to `main`:**
   - The `.github/workflows/deploy-pages.yml` workflow builds the webapp
   - Deploys to GitHub Pages automatically
   - Your demo will be available at: `https://dscudeler.github.io/Worknest/`

3. **Manual deployment (optional):**
   - Go to Actions → Deploy to GitHub Pages
   - Click "Run workflow" → "Run workflow"

**Note:** The first deployment may take a few minutes. Once deployed, your demo mode will be accessible to anyone with the URL!

## Project Status

**Current Phase**: MVP Development (~85% Complete)

The MVP is in advanced development with most core features implemented and tested.

### Roadmap Progress
- [x] Project planning and architecture design
- [x] Workspace and crate structure setup (5 crates)
- [x] Database schema and migrations (SQLite with refinery)
- [x] Authentication system (JWT + bcrypt)
- [x] Core domain models (User, Project, Ticket, Comment, Attachment)
- [x] Repository implementations (Full CRUD - 26 passing tests)
- [x] REST API endpoints (Complete with auth middleware)
- [x] GUI application shell (egui + WASM)
- [x] Project management features (Create, update, delete, archive)
- [x] Ticket management features (List, board, CRUD operations)
- [x] Testing infrastructure (45 passing tests: 19 GUI + 26 repository)
- [ ] Frontend-Backend integration (requires backend deployment)
- [ ] Complete documentation and deployment guides

## Contributing

We welcome contributions! Whether it's bug reports, feature requests, or code contributions, all help is appreciated.

### How to Contribute

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust standard naming conventions
- Write tests for new features
- Update documentation as needed
- Run `cargo fmt` and `cargo clippy` before committing
- Keep commits focused and atomic

See [CONTRIBUTING.md](./CONTRIBUTING.md) (coming soon) for detailed guidelines.

## Technology Stack

- **Language**: Rust 🦀
- **Frontend**: egui (immediate mode GUI) + WASM
- **Backend**: Axum web framework
- **Database**: SQLite with rusqlite
- **Authentication**: JWT + bcrypt
- **Serialization**: serde
- **Testing**: cargo test + proptest
- **Build Tool**: wasm-pack
- **Demo Mode**: Fully functional in-browser demo with localStorage
- **Future**: wasmer/wasmtime (plugins)

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Documentation

- [Roadmap](./ROADMAP.md) - Complete product roadmap
- [Architecture](./ARCHITECTURE.md) - Technical architecture and design
- [Contributing](./CONTRIBUTING.md) - Contribution guidelines (coming soon)
- [API Documentation](./docs/api.md) - API reference (coming soon)

## Community

- **Issues**: [GitHub Issues](https://github.com/DScudeler/Worknest/issues)
- **Discussions**: [GitHub Discussions](https://github.com/DScudeler/Worknest/discussions)
- **Discord**: Coming soon

## Acknowledgments

Special thanks to the Rust community and the egui project for making this possible.

## Star History

If you find Worknest useful, please consider giving it a star on GitHub!

---

**Built with ❤️ and Rust**
