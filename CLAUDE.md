# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Worknest is a project/ticket manager. Backend is a Rust REST API (Axum + SQLite); frontend is a **React + Vite + TypeScript SPA** under `web/` that talks to the API. There is also a TypeScript VSCode extension under `worknest-vscode/` that talks to the same REST API.

The retired egui frontend lives in `legacy/worknest-gui/`. It is excluded from the Cargo workspace and kept only for historical reference — do not modify it.

## Workspace layout

Cargo workspace (`resolver = "2"`) with four crates (all native; the GUI is no longer Rust):

| Crate | Role |
|---|---|
| `worknest-core` | Domain models (`User`, `Project`, `Ticket`, `Comment`, `Attachment`, `Tag`), validation, error types, newtype-wrapped UUID IDs |
| `worknest-db` | `rusqlite` + `r2d2` pool, `refinery` migrations in `src/migrations/`, `Repository<T, ID>` trait + per-entity repos including `TagRepository` |
| `worknest-auth` | bcrypt password hashing, JWT (`jsonwebtoken`), `AuthService` |
| `worknest-api` | Axum server (`main.rs` is a single-file router with all handlers, DTOs, middleware). Binary entry; `lib.rs` only re-exports errors |

Plus:

- `web/` — React + Vite + TypeScript SPA (the active frontend). **Not** a Cargo crate.
- `worknest-vscode/` — TypeScript VSCode extension (axios client, tree view, status bar, git integration). Not part of the Cargo workspace.
- `legacy/worknest-gui/` — retired egui frontend, excluded from the Cargo workspace.

## Build commands

Workspace builds cleanly with no `--exclude` flags now that the egui crate is in `legacy/`:

```bash
cargo build  --workspace
cargo test   --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

# Single crate / single test
cargo test -p worknest-core
cargo test -p worknest-core <name_substring> -- --nocapture

# Doc tests (CI runs these too)
cargo test --doc --workspace
```

Frontend (run from `web/`):

```bash
npm install        # one-time
npm run dev        # Vite dev server on :5173 with /api -> :3000 proxy
npm run build      # Production build to web/dist/
npm run typecheck  # tsc --noEmit
npm run lint       # eslint
```

## Running the system locally

```bash
# Terminal 1 — API on :3000 (creates ./worknest-api.db on first run, applies migrations)
cargo run -p worknest-api

# Terminal 2 — React frontend on :5173 (Vite proxies /api to :3000)
cd web && npm run dev

# VSCode extension (separate)
cd worknest-vscode && npm install && npm run compile  # then F5 in VSCode
```

API env vars (see `.env`): `PORT`, `WORKNEST_DB_PATH`, `WORKNEST_SECRET_KEY` (required ≥32 bytes when `WORKNEST_ENV=production`), `WORKNEST_ALLOWED_ORIGINS` (comma-separated, defaults to `http://localhost:8080`; add the prod web origin), `RUST_LOG`.

## Architecture in one page

**Layering (backend):** `worknest-api` (HTTP) → `worknest-auth` + repositories from `worknest-db` → `worknest-core` (domain models). Repos are `Arc`'d on a single `AppState` struct in `worknest-api/src/main.rs` and injected into Axum handlers via `State<AppState>`.

**Auth:** Login returns a JWT. `auth_middleware` extracts `Bearer <token>`, calls `AuthService::get_user_from_token`, and inserts the resolved `User` into request extensions. Protected handlers pull it out via the `AuthUser(User)` extractor. All non-`/health`, non-`/api/auth/*` routes go through it. Project ownership vs membership is split: `load_project_for_access` allows owners and members; `load_project_for_owner` is stricter and required for project mutations.

**Rate limiting:** `worknest-api/src/rate_limit.rs` is a per-IP token-bucket-style limiter (10 req/min) applied to `register` and `login`. A background task cleans it up every 5 minutes.

**Optimistic concurrency on tickets:** `PUT /api/tickets/{id}` honours `If-Match: <RFC3339 updated_at>` and replies with an `ETag` header. Use this for any client that wants to chain updates (Kanban drag-drop, etc.).

**Tags:** `Ticket` responses embed a `tags: Vec<Tag>` field via the `TicketResponse` DTO (flatten over the inner `Ticket`). `CreateTicketRequest` and `UpdateTicketRequest` accept an optional `tag_ids: Vec<TagId>`. `GET /api/tags` lists the catalogue; the seed data lives in V5 migration.

**Stats:** `GET /api/stats` returns `{ open_tickets, assigned_to_me, due_this_week, active_projects }` for the authenticated user, computed from the existing visibility filter.

**Frontend (`web/`):** Vite + React 18 + TypeScript. Routing via React Router. Server state via TanStack Query. Auth/theme contexts in `web/src/state/`. Design tokens + component CSS in `web/src/styles/`. Lucide for icons. Toasts via `react-hot-toast`. Auth token + current user persist via `localStorage`. The Vite dev server proxies `/api` to `127.0.0.1:3000`, so dev needs no CORS setup.

**Database:** `init_pool(path)` for production (file-backed, 16 conns), `init_memory_pool()` for tests (in-memory, 4 conns). Both enable `PRAGMA foreign_keys = ON`. Migrations live in `crates/worknest-db/src/migrations/V{N}__{snake_case}.sql` and are embedded via `embed_migrations!`. UUIDs are stored as `TEXT`, timestamps as RFC 3339 `TEXT` (use `strftime('%Y-%m-%dT%H:%M:%fZ', 'now')` if you need a default in SQL — `datetime('now')` produces a non-RFC3339 form that the row-mappers reject), booleans as `INTEGER`. Repos all implement `Repository<T, ID>` (`crates/worknest-db/src/repository.rs`); add domain-specific queries as inherent methods on the repo struct.

## Code conventions

- `rustfmt.toml`: `max_width = 100`, 4-space indent, Unix newlines, `use_field_init_shorthand`, `use_try_shorthand`, `reorder_imports`.
- `clippy.toml`: cognitive-complexity ≤ 30, ≤ 7 fn args. CI runs `-D warnings`.
- Domain IDs are newtypes wrapping `Uuid` (`UserId`, `ProjectId`, `TicketId`, `CommentId`, `AttachmentId`, `TagId`) — every new ID gets `new()`, `from_uuid`, `from_string`, `Default`, `Display`.
- Errors: `thiserror` per crate (`CoreError`, `DbError`, `AuthError`, `AppError` in api `main.rs`). No `unwrap()` / `expect()` outside tests and provably-infallible cases.
- Backend enums (`TicketType`, `TicketStatus`, `Priority`) serialize as PascalCase (`Task`, `Open`, `InProgress`, `High`). Input handlers accept lowercase via explicit `to_lowercase()` matches; outgoing JSON is always PascalCase. The TS types in `web/src/lib/types.ts` mirror that wire shape.
- All workspace dependency versions live in the root `Cargo.toml` under `[workspace.dependencies]`; crates use `dep.workspace = true`.
- `worknest-api/src/main.rs` strips SQL/path details from `BadRequest` messages and replaces `Internal` messages with `"An internal error occurred"` before responding — preserve this pattern when adding handlers; log the real error with `tracing::error!`.

## Specialized knowledge in `.claude/skills/`

Detailed how-tos already exist as Skill files — read them directly when working in the relevant area. The `gui-wasm/` skill is now obsolete; ignore it (or update to cover the React stack if you touch it). The others are still accurate:

- `rust-build/` — every build/check/lint command and known gotchas
- `test-runner/` — test layout per crate, coverage with tarpaulin, debugging failing tests
- `code-style/` — full naming conventions, derive sets, validation patterns
- `db-migrations/` — adding a migration, building a new repository, query conventions
- `api-dev/` — handler patterns, DTO conventions, middleware stack, adding an endpoint
- `vscode-ext/` — TypeScript extension structure, adding commands and views
- `ci-pipeline/` — `.github/workflows/ci.yml` jobs and how to reproduce CI locally

`ARCHITECTURE.md` is older design-doc material — accurate on the broad shape but ahead of the code in places (mentions `worknest-plugins`, CQRS handlers, etc., that don't exist yet). Treat the skills and this file as the source of truth for what's actually in the tree.
