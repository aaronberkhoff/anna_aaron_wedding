# =============================================================================
# Multi-stage Dockerfile for the Anna & Aaron wedding website.
#
# Stages:
#   1. chef        — base image with all build tools (Rust, Trunk, wasm-bindgen-cli)
#   2. planner     — extract dependency graph from manifests (cargo-chef prepare)
#   3. native-cook — pre-build native deps (server + shared) using recipe
#   4. wasm-cook   — pre-build WASM deps (frontend + shared) using recipe
#   5. builder     — compile full source on top of warmed caches
#   6. runtime     — minimal production image (binary + dist/ only)
#
# Why cargo-chef?
#   cargo-chef splits dependency compilation from source compilation so that
#   Docker layer caching avoids rebuilding all crates on every source change.
#   Only the last "builder" stage re-runs when you change application code.
# =============================================================================

# =============================================================================
# Stage 1: chef — build toolchain base image
# =============================================================================
FROM rust:1-slim-bookworm AS chef

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# cargo-chef handles dependency layer caching.
RUN cargo install cargo-chef --locked

# wasm32 target required for the frontend crate.
RUN rustup target add wasm32-unknown-unknown

# Trunk is the WASM bundler for the Leptos CSR frontend.
RUN cargo install trunk --locked

# wasm-bindgen-cli version MUST match the wasm-bindgen crate version in Cargo.toml.
# If you bump wasm-bindgen in Cargo.toml, update this version too.
RUN cargo install wasm-bindgen-cli --version 0.2.108 --locked

# wasm-opt optimizes WASM binary size (used by Trunk --release).
RUN curl -fsSL https://github.com/WebAssembly/binaryen/releases/download/version_119/binaryen-version_119-x86_64-linux.tar.gz \
    | tar xz -C /usr/local --strip-components=1

WORKDIR /app

# =============================================================================
# Stage 2: planner — extract dependency recipe
# Only invalidated when Cargo.toml files or Cargo.lock change.
# =============================================================================
FROM chef AS planner

# Copy only manifest files first (source changes won't bust this layer).
COPY Cargo.toml Cargo.lock ./
COPY .cargo/ ./.cargo/
COPY crates/shared/Cargo.toml ./crates/shared/Cargo.toml
COPY crates/server/Cargo.toml ./crates/server/Cargo.toml
COPY crates/frontend/Cargo.toml ./crates/frontend/Cargo.toml

# Create minimal stub src files so cargo-chef can parse the full workspace.
RUN mkdir -p crates/shared/src   && echo "pub fn _stub() {}"            > crates/shared/src/lib.rs
RUN mkdir -p crates/server/src   && echo "fn main() {}"                  > crates/server/src/main.rs
RUN mkdir -p crates/frontend/src && echo "pub fn _stub() {}"             > crates/frontend/src/lib.rs

RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 3: native-cook — warm native dependency cache (server + shared)
# =============================================================================
FROM chef AS native-cook

COPY --from=planner /app/recipe.json recipe.json
COPY --from=planner /app/.cargo ./.cargo

# Cook native deps for server + shared (cargo chef cook has no --exclude).
RUN cargo chef cook \
    --release \
    --recipe-path recipe.json \
    -p server \
    -p shared

# =============================================================================
# Stage 4: wasm-cook — warm WASM dependency cache (frontend + shared)
# =============================================================================
FROM chef AS wasm-cook

COPY --from=planner /app/recipe.json recipe.json
COPY --from=planner /app/.cargo ./.cargo

# Cook WASM deps for the frontend crate.
RUN cargo chef cook \
    --release \
    --recipe-path recipe.json \
    --target wasm32-unknown-unknown \
    -p frontend \
    -p shared

# =============================================================================
# Stage 5: builder — compile application source
# Deps are already compiled in stages 3 and 4; only app code compiles here.
# =============================================================================
FROM chef AS builder

# Bring in warmed native dependency artifacts.
COPY --from=native-cook /app/target ./target
COPY --from=native-cook /root/.cargo /root/.cargo

# Bring in warmed WASM dependency artifacts (merged into same target dir).
COPY --from=wasm-cook /app/target/wasm32-unknown-unknown ./target/wasm32-unknown-unknown

# Copy full workspace source.
COPY . .

# sqlx compile-time query checking requires either a live DB or .sqlx metadata.
# We use offline mode in Docker builds. Run `cargo sqlx prepare` locally and
# commit the .sqlx/ directory to enable this.
ENV SQLX_OFFLINE=true

# Build the native server binary.
RUN cargo build \
    --release \
    --workspace \
    --exclude frontend

# Build the Leptos frontend using Trunk.
# Trunk reads crates/frontend/Trunk.toml and outputs to site/dist/.
RUN cd crates/frontend && \
    trunk build \
    --release

# =============================================================================
# Stage 6: runtime — minimal production image
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Runtime deps only (no Rust toolchain).
RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Run as non-root for security.
RUN useradd -m -u 1001 -s /bin/sh wedding
WORKDIR /app

# Copy the compiled server binary.
COPY --from=builder /app/target/release/server ./server

# Copy the Leptos SPA dist output (HTML, WASM, CSS, assets).
COPY --from=builder /app/site/dist ./site/dist

# /data is a Fly.io persistent volume mount point for the SQLite database file.
RUN mkdir -p /data && chown wedding:wedding /data /app

USER wedding

EXPOSE 8080

# The server binary runs migrations at startup (via sqlx::migrate!).
CMD ["./server"]
