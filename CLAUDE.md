# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Stack

- **Backend**: Axum 0.8 + SQLite (sqlx 0.8) — `crates/server`
- **Frontend**: Leptos 0.7 CSR SPA compiled to `wasm32-unknown-unknown` via Trunk — `crates/frontend`
- **Shared types**: `crates/shared` — must stay WASM-compatible (no tokio/sqlx/axum)
- **Styling**: Tailwind CSS v3 via Trunk's standalone CLI (`@tailwind base/components/utilities` in CSS; config in `tailwind.config.js`)
- **Deployment**: Fly.io via multi-stage Dockerfile; SQLite on `/data` volume

## Critical Constraints

- The `frontend` crate targets **wasm32-unknown-unknown only**. Always use `--exclude frontend` for native cargo commands (`check`, `clippy`, `test`, `build`).
- Never add native-only deps (tokio, sqlx, axum) to `shared` or `frontend`.
- `cargo sqlx prepare` has no `--exclude` flag — run it from `crates/server/`: `cd crates/server && DATABASE_URL=... cargo sqlx prepare`
- `DB_URL` must be absolute: `sqlite:///abs/path/wedding.db`

## Common Commands

```bash
# Prerequisites: cargo install trunk sqlx-cli cargo-watch
# rustup target add wasm32-unknown-unknown

make dev-server        # Axum server with auto-reload on :8080
make dev-frontend      # Trunk hot-reload on :3000 (sets API_BASE_URL=http://localhost:8080)

make migrate           # Create DB + run all pending migrations
make migrate-revert    # Revert last migration
make db-prepare        # Regenerate .sqlx/ offline cache — commit after running

make check             # cargo check (native, excludes frontend)
make check-wasm        # cargo check frontend for wasm32
make fmt               # rustfmt all crates
make clippy            # clippy with -D warnings (native only)
make test              # cargo test (native only)

make build-release     # server binary + Trunk frontend (release)
make clean             # remove build artifacts and site/dist
```

Tests use `SQLX_OFFLINE=true` — run `make db-prepare` and commit `.sqlx/` before tests will pass.

## Architecture

### Request Flow

In dev: browser → Trunk dev server (:3000) for WASM assets; `fetch()` calls → Axum (:8080) for API.
In production: Axum (:8080) serves both the Leptos SPA from `site/dist/` (via `ServeDir`) and the API. All non-API paths fall back to `index.html` for client-side routing.

### Shared API Routes

`crates/shared/src/api/routes.rs` is the single source of truth for all API path strings. Both the Axum router (`crates/server/src/handlers/mod.rs`) and frontend fetch calls import from here to prevent string drift.

### Server Crate (`crates/server`)

- `main.rs` — boots Axum, applies middleware (CORS for dev, compression, tracing)
- `config.rs` — reads env vars: `DATABASE_URL` (required), `DIST_DIR`, `BIND_ADDR`, `SMTP_*` (optional)
- `state.rs` — `AppState { pool: SqlitePool, config: Arc<Config> }` injected into all handlers
- `db.rs` — creates pool, enables WAL mode + foreign keys, runs migrations on startup
- `handlers/` — one module per resource (guests, rsvp, tables, hotels, photos, health)
- `mail.rs` — sends RSVP notification emails via lettre/SMTP (only active when `SMTP_*` vars set)

### Frontend Crate (`crates/frontend`)

- Entry point: `src/lib.rs` with `#[wasm_bindgen(start)]` — mounts the Leptos `App` component to `<body>`
- `app.rs` — top-level `leptos_router` setup; routes: `/`, `/rsvp`, `/seating`, `/hotel`, `/itinerary`, `/gallery`
- `pages/` — one component per route
- `components/` — reusable UI (nav, footer, etc.)
- `api/client.rs` — thin `get<T>` / `post<B, T>` wrappers over browser `fetch()`; `API_BASE` baked in at compile time via `option_env!("API_BASE_URL")`

### Leptos 0.7 Patterns

- Use `LocalResource::new` (not `Resource::new`) for browser fetch calls — `JsFuture` is `!Send`
- `LocalResource` takes one fetcher closure (no source arg); value is `SendWrapper<T>`, deref with `&*data`
- `<A>` component has no `active_class` prop

### Database

Migrations live in `crates/server/migrations/`. The `.sqlx/` offline query cache lives at `crates/server/.sqlx/` — sqlx resolves it relative to the crate. After any schema change: run `make migrate` then `make db-prepare` and commit the result.

## CI/CD

- `ci.yml`: two jobs — `ci` (native, `--exclude frontend`) and `check-frontend` (wasm32 `cargo check` + clippy)
- `deploy.yml`: triggers on `workflow_run: CI completed` → `flyctl deploy --remote-only` (remote Docker build on Fly.io)
- Required GitHub secret: `FLY_API_TOKEN`

## Environment Variables

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `DATABASE_URL` | yes | — | `sqlite:///abs/path/wedding.db` |
| `DIST_DIR` | no | `site/dist` | Path to Trunk build output |
| `BIND_ADDR` | no | `0.0.0.0:8080` | Server listen address |
| `SMTP_FROM` | no | — | Sender address for RSVP emails |
| `SMTP_TO` | no | — | Comma-separated recipient(s) |
| `SMTP_USERNAME` | no | — | SMTP auth username |
| `SMTP_PASSWORD` | no | — | SMTP auth password |

All four `SMTP_*` vars must be set together; if any is missing, email notifications are silently disabled.
