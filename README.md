# Worknest

**An open-source project and task manager built for software development teams**

Worknest is a modern, high-performance project management tool built entirely in Rust. It provides a responsive web application that works seamlessly on both desktop and mobile browsers, giving developers a fast, reliable, and extensible platform for managing projects and tasks.

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
- [Trunk](https://trunkrs.dev/) for building the web application (`cargo install trunk`)
- wasm32 target (`rustup target add wasm32-unknown-unknown`)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/DScudeler/Worknest.git
cd Worknest

# Build the web application
trunk build --release

# The built files will be in the dist/ directory
```

### Development

```bash
# Run the web application in development mode with hot reload
trunk serve

# The application will be available at http://127.0.0.1:8080

# Run tests
cargo test --workspace

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy --workspace --all-targets -- -D warnings
```

## Project Status

**Current Phase**: Foundation Setup (MVP Development)

We're currently in the early stages of development. The MVP (v1.0) is targeted for completion in 3-4 months.

### Roadmap Progress
- [x] Project planning and architecture design
- [ ] Workspace and crate structure setup
- [ ] Database schema and migrations
- [ ] Authentication system
- [ ] Core domain models
- [ ] Repository implementations
- [ ] GUI application shell
- [ ] Project management features
- [ ] Ticket management features
- [ ] Testing and documentation

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
- **Build Tool**: Trunk
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
