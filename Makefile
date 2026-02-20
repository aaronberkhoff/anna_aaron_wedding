# Wedding Website — Developer convenience targets
#
# Prerequisites:
#   cargo, trunk, sqlx-cli, cargo-watch
#
# Install tools:
#   cargo install trunk sqlx-cli cargo-watch
#   rustup target add wasm32-unknown-unknown

.PHONY: help dev-server dev-frontend migrate migrate-revert db-prepare \
        check check-wasm fmt clippy test build-release clean

FRONTEND_DIR := crates/frontend
SERVER_DIR   := crates/server
# Use an absolute path so DB_URL works regardless of the working directory.
# $(CURDIR) is the project root (where make is invoked).
# sqlite:// + /abs/path = sqlite:///abs/path (three slashes = absolute SQLite path).
DB_URL       := sqlite://$(CURDIR)/wedding.db

# ── Help ─────────────────────────────────────────────────────────────────────

help:
	@echo ""
	@echo "  Wedding Website — available make targets"
	@echo ""
	@echo "  Development"
	@echo "    dev-server       Start Axum server with auto-reload (cargo-watch)"
	@echo "    dev-frontend     Start Trunk dev server on :3000 (hot-reload)"
	@echo ""
	@echo "  Database"
	@echo "    migrate          Run pending SQLite migrations"
	@echo "    migrate-revert   Revert the last migration"
	@echo "    db-prepare       Regenerate .sqlx offline query cache (commit after)"
	@echo ""
	@echo "  Checks"
	@echo "    check            cargo check — native crates (excludes frontend)"
	@echo "    check-wasm       cargo check — frontend crate (wasm32 target)"
	@echo "    fmt              Run rustfmt on all crates"
	@echo "    clippy           Run clippy — native crates"
	@echo "    test             Run tests — native crates"
	@echo ""
	@echo "  Build"
	@echo "    build-release    Build server binary + Trunk frontend (release)"
	@echo "    clean            Remove build artifacts and site/dist"
	@echo ""

# ── Development ───────────────────────────────────────────────────────────────

# Run the Axum server with live reload on source changes.
# Requires cargo-watch: cargo install cargo-watch
dev-server:
	DATABASE_URL=$(DB_URL) \
	DIST_DIR=site/dist \
	BIND_ADDR=0.0.0.0:8080 \
	RUST_LOG=debug \
	cargo watch \
	  -w $(SERVER_DIR)/src \
	  -w crates/shared/src \
	  -x "run --package server"

# Run the Trunk dev server (hot-reload WASM frontend on :3000).
# Note: In dev, the frontend fetches the API at localhost:8080 (via API_BASE_URL).
dev-frontend:
	cd $(FRONTEND_DIR) && \
	API_BASE_URL=http://localhost:8080 \
	trunk serve --port 3000

# ── Database ──────────────────────────────────────────────────────────────────

migrate:
	DATABASE_URL=$(DB_URL) sqlx database create
	DATABASE_URL=$(DB_URL) sqlx migrate run --source $(SERVER_DIR)/migrations

migrate-revert:
	DATABASE_URL=$(DB_URL) sqlx migrate revert --source $(SERVER_DIR)/migrations

# Regenerate .sqlx offline query metadata.
# Run `make migrate` first to ensure the DB schema is current.
# Commit the generated .sqlx/ directory so CI can use SQLX_OFFLINE=true.
db-prepare:
	cd $(SERVER_DIR) && DATABASE_URL=$(DB_URL) cargo sqlx prepare

# ── Checks ────────────────────────────────────────────────────────────────────

check:
	cargo check --workspace --exclude frontend

check-wasm:
	cargo check -p frontend --target wasm32-unknown-unknown

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --exclude frontend --all-targets -- -D warnings

test:
	SQLX_OFFLINE=true DATABASE_URL=$(DB_URL) \
	cargo test --workspace --exclude frontend

# ── Build ─────────────────────────────────────────────────────────────────────

build-release:
	SQLX_OFFLINE=true cargo build --release --workspace --exclude frontend
	cd $(FRONTEND_DIR) && trunk build --release

clean:
	cargo clean
	rm -rf site/dist
