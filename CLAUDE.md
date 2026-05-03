# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Worknest is a project/ticket manager. Backend is a Rust REST API (Axum + SQLite); frontend is an egui app compiled to WASM and served as a static site. There is also a TypeScript VSCode extension under `worknest-vscode/` that talks to the same REST API.

`README.md` references a `Makefile` and `build-webapp.sh` — **neither exists**. The actual build tool for the frontend is **Trunk**. Use the commands in this file, not the README.

## Workspace layout

Cargo workspace (`resolver = "2"`) with five crates:

| Crate | Target | Role |
|---|---|---|
| `worknest-core` | native | Domain models (`User`, `Project`, `Ticket`, `Comment`, `Attachment`), validation, error types, newtype-wrapped UUID IDs |
| `worknest-db` | native | `rusqlite` + `r2d2` pool, `refinery` migrations in `src/migrations/`, `Repository<T, ID>` trait + per-entity repos |
| `worknest-auth` | native | bcrypt password hashing, JWT (`jsonwebtoken`), `AuthService` |
| `worknest-api` | native | Axum server (`main.rs` is a single-file router with all handlers, DTOs, middleware). Binary entry; `lib.rs` only re-exports errors |
| `worknest-gui` | **wasm32-unknown-unknown** | egui/eframe immediate-mode UI, screens + components + event-driven async state |

Plus `worknest-vscode/` — TypeScript VSCode extension (axios client, tree view, status bar, git integration). Not part of the Cargo workspace.

## Critical build rule: always exclude `worknest-gui` from workspace commands

`worknest-gui` only compiles for `wasm32-unknown-unknown` (it pulls `eframe` with `glow`, `web-sys`, `gloo-storage`, etc.). Running `cargo build`, `cargo test`, or `cargo clippy` against the whole workspace without excluding it will fail.

```bash
# Backend (default)
cargo build  --workspace --exclude worknest-gui
cargo test   --workspace --exclude worknest-gui
cargo clippy --workspace --exclude worknest-gui --all-targets -- -D warnings
cargo fmt --all -- --check

# Single crate / single test
cargo test -p worknest-core
cargo test -p worknest-core <name_substring> -- --nocapture

# Doc tests (CI runs these too)
cargo test --doc --workspace --exclude worknest-gui

# Frontend — Trunk reads Trunk.toml at the workspace root and builds crates/worknest-gui/index.html
trunk serve            # dev server on http://127.0.0.1:8080
trunk build --release  # output to dist/
cargo clippy -p worknest-gui --target wasm32-unknown-unknown -- -D warnings
```

Prerequisites for the frontend: `rustup target add wasm32-unknown-unknown` and `cargo install trunk`. If Trunk fails on a `wasm-bindgen` version mismatch, install the matching CLI: `cargo install wasm-bindgen-cli --version $(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "wasm-bindgen") | .version')`.

`./run-all-tests.sh` runs backend tests per crate, then WASM GUI tests via `wasm-pack test --headless --firefox` (needs `wasm-pack` + Firefox).

## Running the system locally

```bash
# Terminal 1 — API on :3000 (creates ./worknest-api.db on first run, applies migrations)
cargo run -p worknest-api

# Terminal 2 — WASM frontend on :8080
trunk serve

# VSCode extension (separate)
cd worknest-vscode && npm install && npm run compile  # then F5 in VSCode
```

API env vars (see `.env`): `PORT`, `WORKNEST_DB_PATH`, `WORKNEST_SECRET_KEY` (required ≥32 bytes when `WORKNEST_ENV=production`), `WORKNEST_ALLOWED_ORIGINS` (comma-separated, defaults to `http://localhost:8080`), `RUST_LOG`.

## Architecture in one page

**Layering (backend):** `worknest-api` (HTTP) → `worknest-auth` + repositories from `worknest-db` → `worknest-core` (domain models). Repos are `Arc`'d on a single `AppState` struct in `worknest-api/src/main.rs` and injected into Axum handlers via `State<AppState>`.

**Auth:** Login returns a JWT. `auth_middleware` extracts `Bearer <token>`, calls `AuthService::get_user_from_token`, and inserts the resolved `User` into request extensions. Protected handlers pull it out via the `AuthUser(User)` extractor. All non-`/health`, non-`/api/auth/*` routes go through it. Ownership checks (e.g., only project creator can update/delete) live inline in handlers — there's no centralized policy layer.

**Rate limiting:** `worknest-api/src/rate_limit.rs` is a per-IP token-bucket-style limiter (10 req/min) applied to `register` and `login`. A background task cleans it up every 5 minutes.

**Frontend (`worknest-gui`):** egui is immediate-mode — the entire UI re-runs every frame from `WorknestApp::update()` in `src/app.rs`. State is centralized in `AppState` (`src/state.rs`) and passed `&mut` to each screen's `render(ctx, &mut state)`. There is no retained widget tree.

The frontend is async without tokio. The pattern is:
1. A screen calls `state.api_client.foo(...)` inside `wasm_bindgen_futures::spawn_local`.
2. The async closure pushes an `AppEvent` (defined in `src/events.rs`) into the shared `EventQueue` (`Arc<Mutex<Vec<AppEvent>>>`).
3. On the next frame, `AppState::process_events()` drains the queue and updates cached `projects`/`tickets`/`comments` and notifications.

Always `clone()` `api_client` and `event_queue` before moving them into a `spawn_local` closure. Use `web_time::Instant`, never `std::time::Instant`. Auth token + current user persist via `gloo_storage::LocalStorage`; `try_restore_session()` reloads them on startup.

**Database:** `init_pool(path)` for production (file-backed, 16 conns), `init_memory_pool()` for tests (in-memory, 4 conns). Both enable `PRAGMA foreign_keys = ON`. Migrations live in `crates/worknest-db/src/migrations/V{N}__{snake_case}.sql` and are embedded via `embed_migrations!`. UUIDs are stored as `TEXT`, timestamps as RFC 3339 `TEXT`, booleans as `INTEGER`. Repos all implement `Repository<T, ID>` (`crates/worknest-db/src/repository.rs`); add domain-specific queries as inherent methods on the repo struct.

## Code conventions

- `rustfmt.toml`: `max_width = 100`, 4-space indent, Unix newlines, `use_field_init_shorthand`, `use_try_shorthand`, `reorder_imports`.
- `clippy.toml`: cognitive-complexity ≤ 30, ≤ 7 fn args. CI runs `-D warnings`.
- Domain IDs are newtypes wrapping `Uuid` (`UserId`, `ProjectId`, `TicketId`, `CommentId`, `AttachmentId`) — every new ID gets `new()`, `from_uuid`, `from_string`, `Default`, `Display`.
- Errors: `thiserror` per crate (`CoreError`, `DbError`, `AuthError`, `AppError` in api `main.rs`). No `unwrap()` / `expect()` outside tests and provably-infallible cases.
- All workspace dependency versions live in the root `Cargo.toml` under `[workspace.dependencies]`; crates use `dep.workspace = true`.
- `worknest-api/src/main.rs` strips SQL/path details from `BadRequest` messages and replaces `Internal` messages with `"An internal error occurred"` before responding — preserve this pattern when adding handlers; log the real error with `tracing::error!`.

## Specialized knowledge in `.claude/skills/`

Detailed how-tos already exist as Skill files — read them directly when working in the relevant area. They are kept in sync with the code and contain more depth than this file:

- `rust-build/` — every build/check/lint command and known gotchas
- `test-runner/` — test layout per crate, coverage with tarpaulin, debugging failing tests
- `code-style/` — full naming conventions, derive sets, validation patterns
- `db-migrations/` — adding a migration, building a new repository, query conventions
- `api-dev/` — handler patterns, DTO conventions, middleware stack, adding an endpoint
- `gui-wasm/` — adding a screen/component, the event-queue async pattern, WASM specifics
- `vscode-ext/` — TypeScript extension structure, adding commands and views
- `ci-pipeline/` — `.github/workflows/ci.yml` jobs and how to reproduce CI locally

`ARCHITECTURE.md` is older design-doc material — accurate on the broad shape but ahead of the code in places (mentions `worknest-plugins`, CQRS handlers, etc., that don't exist yet). Treat the skills and this file as the source of truth for what's actually in the tree.
