# AI DJ — Implementation Overview

Read this first. Full design rationale lives in `/AI_DJ_DESIGN.md`; this folder breaks it into executable phases. Implement phases **in order** — each assumes the previous is merged and green.

## What we're building

A `/dj` page on the wedding site. The couple pre-builds the playlist; an algorithm orders and mixes it; the DJ logs in and presses play.

- **No live AI.** An offline analyzer (native Rust CLI) precomputes BPM, energy, cue points, release year, and a mix-out point per track into SQLite.
- **Mixing runs in the browser** (Web Audio API via `web-sys` in the Leptos frontend): beat-aligned equal-power crossfades. After load, playback is client-side — wifi loss doesn't stop music.
- **Server** serves audio bytes (HTTP range) from `MUSIC_DIR` on the Fly volume + track metadata + a computed mix plan.
- **Songs never play in full** — each has a cutoff (host override > analyzer suggestion > default cap) and mixes out there.
- **Two roles**: DJ passcode (play/pause/skip) and Host passcode (Aaron: everything + requests toggle + order/cutoff editing).
- **Ordering** is layered: pinned first/last songs → explicit `play_order` → era arc (older songs early, 2000s middle, modern late) blended with BPM/energy continuity.

## Phase index

| Doc | Phase | Outcome |
| --- | --- | --- |
| `01_PHASE_0_SPIKE.md` | Spike | Two hardcoded MP3s crossfade in the browser. Proves the risky part. |
| `02_PHASE_1_PLAYABLE.md` | Playable | Migration, auth, audio streaming, `/dj` player with fixed crossfades + cutoffs. **Wedding-usable from here.** |
| `03_PHASE_2_AI.md` | The "AI" | `analyzer` crate, adaptive beat-aligned transitions, era-arc scheduler, host order editor. |
| `04_PHASE_3_REQUESTS.md` | Requests (optional) | Guest song requests, gated by host toggle. |
| `05_PHASE_4_POLISH.md` | Polish | Now-playing screen, visualizer, wake lock, panic mode. |

## Repo conventions that apply to every phase

These come from `CLAUDE.md` — restated because violating them breaks CI:

1. `frontend` compiles **only** for `wasm32-unknown-unknown`. Native cargo commands must use `--exclude frontend` (the Makefile targets already do). Verify frontend with `make check-wasm`.
2. `shared` must stay WASM-safe: serde/serde_json only — never tokio, sqlx, axum.
3. All new API paths are declared as consts in `crates/shared/src/api/routes.rs` and imported by both the Axum router and frontend fetch calls. Never inline path strings.
4. **Axum 0.8 path params use `{id}` syntax** when registering routes (`/api/dj/audio/{id}`), not `:id`. Shared consts can keep whatever form; the frontend substitutes the id when building URLs anyway.
5. After any migration: `make migrate` → `make db-prepare` → **commit `.sqlx/`**. Tests run with `SQLX_OFFLINE=true` and fail without the refreshed cache.
6. Migrations go in `crates/server/migrations/` following the existing naming: `20250101000008_create_dj.sql` (increment the sequence).
7. Leptos 0.7: use `LocalResource::new` for fetches (`JsFuture` is `!Send`); deref values with `&*data`.
8. New `web-sys` features are added to the `[workspace.dependencies]` `web-sys` feature list in the root `Cargo.toml`.
9. Before finishing any phase: `make fmt && make clippy && make test && make check-wasm` must all pass.

## New environment variables (introduced in Phase 1)

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `MUSIC_DIR` | no | `/data/music` | Directory of audio files |
| `DJ_PASSCODE` | no | — | DJ login; feature 404s/disabled if unset |
| `HOST_PASSCODE` | no | — | Host (Aaron) login; superset of DJ perms |

Follow the existing `SMTP_*` optional-config pattern in `config.rs`: if passcodes are unset, DJ endpoints return 404 and the feature is off.

## Data model (final shape — built incrementally)

`tracks`: id, title, artist, filename (unique), duration_ms, release_year, bpm, bpm_confidence, energy, intro_end_ms, outro_start_ms, auto_cutoff_ms, cutoff_ms, phase (`dinner|cocktail|dance|last_dance`), pinned_slot (`first|last`|NULL), play_order (NULL = scheduler decides), analyzed_at.

`settings` (single row, id=1): requests_open (default 0), default_cutoff_ms (default 150000).

`requests` (Phase 3): id, track_id FK, guest_name, created_at, status.

**Cutoff precedence** (used everywhere a track's end is needed): `cutoff_ms` → `auto_cutoff_ms` → `min(default_cutoff_ms, outro_start_ms)` → `default_cutoff_ms`.

## API surface (final shape)

```
POST /api/dj/login              passcode -> { token, role }   (accepts DJ or Host passcode)
GET  /api/dj/tracks             Bearer (any role) -> Vec<Track>
GET  /api/dj/plan               Bearer (any role) -> MixPlan
GET  /api/dj/audio/{id}         Bearer (any role) -> audio bytes, Range supported
GET/POST /api/dj/host/settings  Bearer (host)     -> Settings
POST /api/dj/host/order         Bearer (host)     -> reorder / pin
PATCH /api/dj/host/tracks/{id}  Bearer (host)     -> cutoff_ms, release_year, phase overrides
POST /api/dj/requests           public, 403 unless settings.requests_open
```

Tokens are random UUIDs held in `AppState` memory (a `Mutex<HashMap<String, Role>>`); no sessions table. Restarting the server logs everyone out — acceptable.

## Definition of done per phase

Each phase doc ends with **Acceptance criteria**. A phase is done when all criteria pass, `make fmt && make clippy && make test && make check-wasm` are green, and `.sqlx/` is committed if the schema changed.
