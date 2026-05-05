# Worknest

**An open-source project and ticket manager built for software development teams.**

Worknest is a project management tool with a Rust REST API backend (Axum +
SQLite) and a React + TypeScript frontend served as a static SPA. It also
ships a TypeScript VSCode extension that talks to the same REST API.

## Features

- User auth with JWT (registration, login, password change with token
  invalidation)
- Projects with multi-member access and per-project roles
- Tickets with type / status / priority / assignee / due date / estimate,
  plus optimistic concurrency on update via `If-Match`
- List + Kanban views, search, and filterable chips
- Comments (CRUD on own) and attachments (multipart upload, MIME
  allowlist, 10 MB limit)
- Tag/label system with paired light/dark colors
- Dashboard stats and per-user profile fields (`full_name`, `avatar_url`)
- VSCode extension for in-IDE ticket browsing, creation, and git
  integration

## Layout

```
worknest/
├── crates/
│   ├── worknest-core/    # Domain models + validation
│   ├── worknest-db/      # SQLite repositories + refinery migrations
│   ├── worknest-auth/    # bcrypt + JWT
│   └── worknest-api/     # Axum REST API
├── web/                  # React + Vite SPA (replaces the old egui crate)
├── worknest-vscode/      # VSCode extension
└── legacy/
    └── worknest-gui/     # Retired egui frontend (kept for reference;
                          # excluded from the workspace)
```

See `ARCHITECTURE.md` for technical detail and `CLAUDE.md` for the
contributor cheat-sheet (which build commands actually exist, etc.).

## Getting started

### Prerequisites

- Rust 1.70+ (https://rustup.rs)
- Node 20+ and npm 9+

### Run the system locally

```bash
# Terminal 1 — API on :3000 (creates ./worknest-api.db on first run)
cargo run -p worknest-api

# Terminal 2 — React frontend on :5173 with /api proxy to :3000
cd web
npm install
npm run dev
```

Open http://127.0.0.1:5173.

API environment variables (see `.env`):

- `PORT` — default 3000
- `WORKNEST_DB_PATH` — SQLite file path
- `WORKNEST_SECRET_KEY` — required (≥32 bytes) when
  `WORKNEST_ENV=production`
- `WORKNEST_ALLOWED_ORIGINS` — comma-separated CORS origins, defaults to
  `http://localhost:8080`. Add your prod web origin here.
- `RUST_LOG` — log filter

### Backend commands

```bash
cargo build  --workspace
cargo test   --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

### Frontend commands (run from `web/`)

```bash
npm run dev        # Vite dev server with /api proxy
npm run build      # Production build to web/dist/
npm run typecheck  # tsc --noEmit
npm run lint       # eslint
```

### VSCode extension

```bash
cargo run -p worknest-api  # API on :3000
cd worknest-vscode && npm install && npm run compile
# Press F5 in VSCode to launch the Extension Development Host
```

## Deployment

The API is REST-only. Deploy it behind a reverse proxy (nginx, Caddy,
etc.) and serve `web/dist/` as a separate static site (any static host:
nginx, S3+CloudFront, GitHub Pages, etc.). Add the static frontend's
origin to `WORKNEST_ALLOWED_ORIGINS` so the browser can reach the API.

## Technology stack

- **Backend**: Rust, Axum, SQLite (rusqlite + r2d2), refinery migrations,
  JWT (jsonwebtoken), bcrypt, tower-http (CORS, tracing).
- **Frontend**: React 18, TypeScript, Vite, React Router, TanStack Query,
  Lucide icons, react-hot-toast, vanilla CSS with design tokens.
- **VSCode extension**: TypeScript, axios.

## License

MIT — see `LICENSE`.
