# Phase 1 — Playable Player

**Goal:** a wedding-usable `/dj` page. DB-backed track library, DJ/Host auth, range-supporting audio streaming, and a player that walks an ordered playlist with fixed-length equal-power crossfades, honoring cutoffs and pins. No analyzer yet — metadata is entered by hand or from ID3 tags at import.

**Depends on:** Phase 0 merged (reuses its Web Audio code).

## 1. Migration — `crates/server/migrations/20250101000008_create_dj.sql`

```sql
CREATE TABLE tracks (
    id              INTEGER PRIMARY KEY,
    title           TEXT NOT NULL,
    artist          TEXT NOT NULL,
    filename        TEXT NOT NULL UNIQUE,
    duration_ms     INTEGER NOT NULL,
    release_year    INTEGER,
    bpm             REAL,
    bpm_confidence  REAL,
    energy          REAL,
    intro_end_ms    INTEGER,
    outro_start_ms  INTEGER,
    auto_cutoff_ms  INTEGER,
    cutoff_ms       INTEGER,
    phase           TEXT NOT NULL DEFAULT 'dance',
    pinned_slot     TEXT,
    play_order      INTEGER,
    analyzed_at     TEXT
);

CREATE TABLE settings (
    id                INTEGER PRIMARY KEY CHECK (id = 1),
    requests_open     INTEGER NOT NULL DEFAULT 0,
    default_cutoff_ms INTEGER NOT NULL DEFAULT 150000
);
INSERT INTO settings (id) VALUES (1);
```

Then: `make migrate` → `make db-prepare` → commit `.sqlx/`.

## 2. Shared crate

`crates/shared/src/api/routes.rs` — add:

```rust
pub const DJ_LOGIN: &str = "/api/dj/login";
pub const DJ_TRACKS: &str = "/api/dj/tracks";
pub const DJ_PLAN: &str = "/api/dj/plan";
pub const DJ_AUDIO: &str = "/api/dj/audio";        // frontend appends /{id}
pub const HOST_SETTINGS: &str = "/api/dj/host/settings";
pub const HOST_ORDER: &str = "/api/dj/host/order";
pub const HOST_TRACKS: &str = "/api/dj/host/tracks"; // frontend appends /{id}
```

`crates/shared/src/models/` — new module `dj.rs` (serde only, WASM-safe):

```rust
pub enum Phase { Dinner, Cocktail, Dance, LastDance }   // serde rename_all = "snake_case"
pub enum Slot { First, Last }
pub enum Role { Dj, Host }

pub struct Track { /* mirror the tracks table, Option for nullables */ }
pub struct LoginRequest { pub passcode: String }
pub struct LoginResponse { pub token: String, pub role: Role }
pub struct Settings { pub requests_open: bool, pub default_cutoff_ms: u32 }
pub struct MixStep { pub track_id: i64, pub start_at_ms: u32, pub end_at_ms: u32, pub crossfade_ms: u32 }
pub struct MixPlan { pub steps: Vec<MixStep> }
pub struct OrderUpdate { pub track_id: i64, pub play_order: Option<i64>, pub pinned_slot: Option<Slot> }
pub struct TrackPatch { pub cutoff_ms: Option<Option<u32>>, pub release_year: Option<Option<u16>>, pub phase: Option<Phase> }
```

(`Option<Option<T>>` pattern: outer None = leave unchanged, inner None = clear. Use `#[serde(default, skip_serializing_if = ...)]` as appropriate.)

## 3. Server crate

### `config.rs`
Add `music_dir: String` (env `MUSIC_DIR`, default `/data/music`), `dj_passcode: Option<String>`, `host_passcode: Option<String>`. If either passcode is unset, log a warning like the SMTP path does and treat the whole DJ feature as disabled (handlers return 404).

### `state.rs`
Add `dj_tokens: Arc<Mutex<HashMap<String, Role>>>` (std Mutex is fine; accesses are short).

### `handlers/dj.rs` (new; register in `handlers/mod.rs`)

- `POST /api/dj/login` — body `LoginRequest`. Compare against host passcode first, then DJ. On match: generate `uuid::Uuid::new_v4()` token, insert into `dj_tokens`, return `LoginResponse`. Wrong passcode → 401. Feature disabled → 404.
- **Auth extractor**: a small helper that reads the `Authorization: Bearer <token>` header, looks up role; `require_dj` (any role) and `require_host` (Host only, else 403).
- `GET /api/dj/tracks` (any role) — all tracks ordered by effective order.
- `GET /api/dj/plan` (any role) — computes `MixPlan` server-side in Phase 1 (simple version):
  - Order: `pinned_slot='first'` tracks first (by phase order dinner→cocktail→dance→last_dance), then tracks by `play_order`, then remaining by `release_year ASC` (nulls last) as a crude era arc placeholder, then `pinned_slot='last'` last.
  - Per step: `start_at_ms = intro_end_ms.unwrap_or(0)`, `end_at_ms` = cutoff precedence `cutoff_ms → auto_cutoff_ms → min(default_cutoff_ms, outro_start_ms?) → default_cutoff_ms` (clamped to `duration_ms`), `crossfade_ms = 8000` fixed.
- `GET /api/dj/audio/{id}` (any role) — look up filename, serve `{music_dir}/{filename}` **with Range support**: use `tower_http::services::ServeFile` invoked per-request (`ServeFile::new(path).oneshot(req)`), which handles Range/content-type. Reject filenames containing `/` or `..` (defense in depth; filenames come from our DB but be safe).
- `GET/POST /api/dj/host/settings` (host) — read/update the single settings row.
- `POST /api/dj/host/order` (host) — body `Vec<OrderUpdate>`; apply in one transaction.
- `PATCH /api/dj/host/tracks/{id}` (host) — body `TrackPatch`.

**Axum 0.8: register param routes as `/api/dj/audio/{id}`.**

### Track import (no upload UI)
Add a small CLI subcommand or standalone bin `crates/server/src/bin/import_tracks.rs`: scans `MUSIC_DIR`, reads ID3 tags (`lofty` crate: title, artist, year, duration), upserts into `tracks` by filename. Run manually after `fly sftp` uploads. Duration via lofty's properties (no decoding needed).

## 4. Frontend crate

### `api/client.rs`
- Add optional bearer-token support to `get`/`post` (e.g. new variants `get_auth`, `post_auth`, `patch_auth` taking `&str` token).
- Add `get_bytes(path: &str, token: &str) -> Result<js_sys::ArrayBuffer, String>`.

### `pages/dj.rs` (new; route `/dj` in `app.rs`; remove `/dj-spike`)
- **Login view**: passcode input → `POST DJ_LOGIN` → hold `(token, role)` in a signal (memory only; refresh = re-login, that's fine).
- **Player view** (after login):
  - Load `MixPlan` + `Track` list.
  - **Engine** (refactor spike code into `crates/frontend/src/dj/engine.rs`): plays plan steps sequentially. For each transition, schedule against `AudioContext.currentTime`: outgoing gain ramps down / incoming ramps up (equal-power) over `crossfade_ms`, incoming starts at its `start_at_ms` offset (`start_with_when_and_grain_offset`), outgoing stops at `end_at_ms`.
  - **Buffer policy**: keep only current + next decoded `AudioBuffer`s. Prefetch bytes for step n+2 in the background; decode lazily when it becomes "next"; drop buffers once a track finishes.
  - Controls: Play (the initial user gesture), Pause/Resume (`ctx.suspend()`/`ctx.resume()`), Skip (jump to next step: fast 2s crossfade), master volume GainNode.
  - Display: now playing, up next, progress within current step.
- **Host extras** (render only when `role == Host`): requests toggle (wired in Phase 3 but persist the setting now), and a plain table of tracks with editable cutoff/year/phase + pin buttons + move up/down (writes via `HOST_ORDER` / `HOST_TRACKS`). Drag-and-drop is Phase 2 polish — buttons are fine here.

## Acceptance criteria

- [ ] `make fmt && make clippy && make test && make check-wasm` green; `.sqlx/` committed.
- [ ] With passcodes unset, all `/api/dj/*` routes return 404 and the site otherwise works unchanged.
- [ ] DJ passcode: can log in, see plan, play through 3+ real tracks with 8s crossfades; each track starts at its intro cue and ends at its cutoff (never full length).
- [ ] Host passcode: can pin a track first/last, reorder, set a per-track cutoff — plan reflects it after reload.
- [ ] Audio endpoint: `curl -H "Range: bytes=0-1023" -H "Authorization: Bearer <t>" .../api/dj/audio/1` returns 206 with 1024 bytes; no token returns 401.
- [ ] Wrong-role access: DJ token on `/api/dj/host/settings` returns 403.
- [ ] Unit tests: cutoff precedence function; plan ordering (pins first/last, play_order respected, year fallback).

## Out of scope

BPM/beat alignment, analyzer, era-arc curve fitting, requests endpoint, visualizer.
