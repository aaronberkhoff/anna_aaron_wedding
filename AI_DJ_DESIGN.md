# AI DJ — Design & Game Plan

> Implementation docs (feed these to Claude Code, in order): [`docs/ai-dj/00_OVERVIEW.md`](docs/ai-dj/00_OVERVIEW.md) + one doc per phase.

Goal: pre-build the wedding playlist, let an algorithm handle ordering and transitions, and have the "DJ" just log into the site and press play.

## Concept

The "AI" is not a model running live — it's an **offline analysis pass** (BPM, energy, cue points per track) plus a **deterministic mixing engine** that runs in the browser using the Web Audio API. This is the right call for a live event: everything is precomputed and auditioned in advance, nothing depends on an external API or inference at 10pm on the dance floor.

Three pieces:

1. **Analyzer** (new native crate, run ahead of time) — decodes each MP3, computes BPM, energy, and intro/outro cue points, writes results to SQLite.
2. **Server** (`crates/server`) — stores the track library on the Fly volume, serves audio with HTTP range support, exposes track metadata + mix plan, gates the DJ page behind a passcode.
3. **Player** (`crates/frontend`) — a `/dj` page that fetches the plan, decodes tracks with Web Audio, and schedules beat-aligned equal-power crossfades. After initial load it is fully client-side, so a venue wifi dropout mid-set doesn't stop the music.

## Decisions (and why)

| Decision | Choice | Rationale |
| --- | --- | --- |
| Audio source | Self-hosted MP3s on the `/data` volume | Full mixing control via Web Audio. Spotify's SDK can't overlap two tracks (no true crossfade) and adds a live auth dependency. |
| Mixing depth | Smart crossfade (BPM/energy-aware cue points, energy arc ordering) | Sounds professional, low live risk. True beatmatching (tempo-stretching) is deferred — small optional `playbackRate` nudge (±4%) only when BPMs are already close. |
| Where mixing runs | Browser (frontend crate) | Server stays a dumb file/metadata host; laptop plugs into the PA; resilient to network loss. |
| "AI" scope | Offline analysis + deterministic scheduler | Rehearsable. You can listen to every transition before the wedding. |

## Architecture

```
                    ┌────────────────────────────┐
 offline, pre-event │  crates/analyzer (native)  │  symphonia decode → BPM /
                    │  cargo run -p analyzer     │  energy / cue detection
                    └─────────────┬──────────────┘
                                  │ writes tracks table
                                  ▼
 ┌──────────────┐   GET /api/dj/plan     ┌─────────────────────────────┐
 │ crates/server│◄───────────────────────│ crates/frontend  /dj page   │
 │  Axum        │   GET /api/dj/audio/:id│  Web Audio mixing engine    │
 │  SQLite      │──── bytes (ranges) ───►│  (decode next-2 lookahead)  │
 │  /data/music │                        │  laptop → venue PA          │
 └──────────────┘                        └─────────────────────────────┘
```

### Shared crate (`crates/shared`)

New route constants in `api/routes.rs`:

```rust
pub const DJ_LOGIN: &str = "/api/dj/login";        // POST dj passcode -> dj token
pub const DJ_TRACKS: &str = "/api/dj/tracks";      // GET library + analysis
pub const DJ_PLAN: &str = "/api/dj/plan";          // GET ordered mix plan
pub const DJ_AUDIO: &str = "/api/dj/audio/:id";    // GET audio bytes (range)
pub const DJ_REQUESTS: &str = "/api/dj/requests";  // Phase 3 (optional)

// Host-only (Aaron's login — separate credential, superset of DJ perms)
pub const HOST_LOGIN: &str = "/api/dj/host/login";       // POST host passcode -> host token
pub const HOST_SETTINGS: &str = "/api/dj/host/settings"; // GET/POST: requests_open, default cutoff, arc config
pub const HOST_ORDER: &str = "/api/dj/host/order";       // POST: pin/reorder/override tracks
pub const HOST_TRACK: &str = "/api/dj/host/tracks/:id";  // PATCH: per-track cutoff, era, phase
```

New types (WASM-safe, serde only):

```rust
pub struct Track {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub duration_ms: u32,
    pub release_year: Option<u16>, // drives the era arc (from ID3 tag, editable)
    pub bpm: Option<f32>,
    pub energy: Option<f32>,       // 0.0–1.0, normalized RMS
    pub intro_end_ms: Option<u32>, // first strong-beat point
    pub outro_start_ms: Option<u32>,
    pub cutoff_ms: Option<u32>,    // host override; None = analyzer decides
    pub phase: Phase,              // Dinner | Cocktail | Dance | LastDance
    pub pinned_slot: Option<Slot>, // First | Last (per phase) — always honored
}

pub struct MixStep {
    pub track_id: i64,
    pub start_at_ms: u32,          // skip long intros if desired
    pub end_at_ms: u32,            // mix-out point — songs never play in full
    pub crossfade_ms: u32,         // per-transition, from analysis
}

pub struct MixPlan { pub steps: Vec<MixStep> }
```

### Server crate

- `handlers/dj.rs` — new module, registered in `handlers/mod.rs`.
- **Audio serving**: custom handler that opens `{MUSIC_DIR}/{filename}` and honors `Range` headers (or `tower-http` `ServeFile` wrapped in the auth layer). Range support matters so the browser can stream/seek without full downloads.
- **Auth — two roles**, still proportional to the threat model (wedding guests, not attackers):
  - **DJ** (`DJ_PASSCODE`): can load the plan, play, pause, skip. Given to whoever runs the laptop.
  - **Host** (`HOST_PASSCODE` — Aaron only): everything the DJ can do, plus the requests on/off toggle, order editing, and per-track cutoff/era overrides. Only a host token is accepted on `/api/dj/host/*` routes.
  - Both: `POST login` with passcode → random token held in server state, sent as `Bearer` header. No users table, no sessions table.
- **Settings**: a one-row `settings` table (`requests_open`, `default_cutoff_ms`, arc config). `requests_open` defaults to off; `POST /api/dj/requests` returns 403 while it's off, so requests are fielded only when you flip the switch.
- **Config**: add `MUSIC_DIR` (default `/data/music`), `DJ_PASSCODE`, and `HOST_PASSCODE` to `config.rs`.
- **Uploads**: skip building an upload UI. Get files onto the volume with `fly sftp shell` → `put`. One less handler, zero risk.

### Analyzer crate (`crates/analyzer`, new, native-only)

Never touched by the frontend, so tokio/sqlx are fine here. Add it to the workspace and to the `--exclude frontend` command set (no change needed — Makefile excludes only `frontend`).

- Decode: `symphonia` (pure Rust, handles MP3/FLAC/AAC).
- BPM: autocorrelation of an onset-energy envelope (spectral flux). ~150 lines; accurate enough for 80–180 BPM pop. Store a confidence score; low-confidence tracks fall back to plain timed crossfades.
- Energy: windowed RMS, normalized across the library.
- Cue points: `intro_end` = first sustained-energy onset; `outro_start` = where trailing energy drops below threshold. These are where crossfades anchor.
- Release year: read from the ID3 `TDRC`/`TYER` tag (the `id3` or `lofty` crate); hand-editable when tags are wrong.
- **Auto cutoff** (`auto_cutoff_ms`): songs should mix out early, radio-DJ style, not play to the end. The analyzer picks the energy-section boundary (a sustained energy drop — typically the end of a chorus) nearest the `default_cutoff_ms` target, snapped back to a downbeat. Playback uses `cutoff_ms` (your override) if set, else `auto_cutoff_ms`, else `min(default_cutoff_ms, outro_start)`.
- CLI: `cargo run -p analyzer -- --music-dir ./music --db sqlite://...` scans the dir, upserts `tracks`. Rerun any time you add songs.
- Every analysis value is editable after the fact (plain SQLite columns), so you can hand-correct a bad BPM or cue point for a specific song.

### Migration

```sql
-- NNNN_create_tracks.sql
CREATE TABLE tracks (
    id              INTEGER PRIMARY KEY,
    title           TEXT NOT NULL,
    artist          TEXT NOT NULL,
    filename        TEXT NOT NULL UNIQUE,
    duration_ms     INTEGER NOT NULL,
    release_year    INTEGER,           -- from ID3 tag; hand-editable
    bpm             REAL,
    bpm_confidence  REAL,
    energy          REAL,
    intro_end_ms    INTEGER,
    outro_start_ms  INTEGER,
    auto_cutoff_ms  INTEGER,           -- analyzer's suggested mix-out point
    cutoff_ms       INTEGER,           -- host override; wins over auto_cutoff_ms
    phase           TEXT NOT NULL DEFAULT 'dance',
    pinned_slot     TEXT,              -- 'first' | 'last' (per phase), NULL = unpinned
    play_order      INTEGER,           -- NULL = let the scheduler order it
    analyzed_at     TEXT
);

CREATE TABLE settings (
    id                INTEGER PRIMARY KEY CHECK (id = 1),  -- single row
    requests_open     INTEGER NOT NULL DEFAULT 0,
    default_cutoff_ms INTEGER NOT NULL DEFAULT 150000       -- ~2:30 target play length
);
INSERT INTO settings (id) VALUES (1);
```

After adding: `make migrate` → `make db-prepare` → commit `.sqlx/`.

### Frontend (`crates/frontend`)

- New route `/dj` in `app.rs` → `pages/dj.rs`. Passcode form → token in memory → load plan.
- `api/client.rs`: add `get_bytes(path, token) -> Result<Vec<u8>, String>` (current helpers are JSON-only) and token-header support.
- **Mixing engine** (`frontend/src/dj/engine.rs`), on `web-sys`:
  - One `AudioContext`; per track a `AudioBufferSourceNode → GainNode → destination`.
  - Schedule with `AudioContext.currentTime`-based automation (`linearRampToValueAtTime` / equal-power curves) — sample-accurate, immune to JS jank.
  - Transition: at current track's `end_at_ms` (the cutoff — never `duration_ms`), start next track at its `intro_end` minus lead-in, snapped to the nearest downbeat of the *outgoing* track's beat grid (`bpm` + first-beat offset). Fade curves: equal-power over `crossfade_ms` (fast BPM-matched songs get short fades; ballad→dance boundaries get longer ones).
  - Optional polish: if `|bpm_a − bpm_b| / bpm_a ≤ 4%`, set the incoming source's `playback_rate` to match, ramp to 1.0 after the fade. Skip entirely when confidence is low.
  - **Memory**: a decoded 4-min stereo track is ~85 MB of PCM. Keep only *current + next* decoded; fetch bytes for track n+2 in the background but decode lazily; drop buffers after playback. Do not decode the whole library.
- **Scheduler** (can live client-side or server-side; client is simpler). Ordering is layered — each layer only fills what the layer above left open:
  1. **Pins** (absolute): `pinned_slot = first/last` per phase. First dance and last dance are always exactly what you chose.
  2. **Manual order**: any track with `play_order` set sits at that position, untouched.
  3. **Era arc** (the default for everything else): remaining tracks are sorted so `release_year` trends upward across the evening — older songs early, 2000s in the middle, modern at the end. Implemented as a target-year curve over the set; each slot picks the candidate minimizing `|year − target(slot)|` blended with BPM/energy continuity, so the era progression holds without wrecking transitions.
  4. **Energy shaping** within each phase (dinner low/flat → dance ramps with breathers) as a tiebreaker.
- **Host order editor** (host login only): drag-to-reorder list of the computed plan. Reordering writes `play_order`; a pin toggle writes `pinned_slot`; per-track cutoff and year are editable inline. Everything persists via `/api/dj/host/*`, so the plan you audition is exactly the plan that plays.
- **Live controls**: big Pause / Skip / Volume buttons, "now playing + up next" display, and a **panic button** that cuts to a plain single-track player. Request a screen Wake Lock (`navigator.wakeLock`) so the laptop doesn't sleep mid-set. When logged in as host, a **Requests: Open/Closed** toggle sits in the header.

## Build Plan

| Phase | Scope | Est. |
| --- | --- | --- |
| **0 — Spike** | Hardcode two MP3s in `site/`, Leptos page that crossfades them via web-sys Web Audio. Proves the whole risky part. | 1 weekend |
| **1 — Playable** | Migration, `dj.rs` handlers (DJ + host login, tracks, audio w/ ranges, settings), `/dj` page, fixed-length crossfades honoring `cutoff_ms`, manual `play_order` + pins. *Usable at the wedding from this point.* | 1–2 weekends |
| **2 — The "AI"** | `analyzer` crate (BPM/energy/cues/auto-cutoff/year), beat-aligned adaptive crossfades, era-arc + energy scheduler, host order editor, rehearsal mode (audition just the transition windows). | 2–3 weekends |
| **3 — Optional: guest requests** | Guests browse the pre-approved library from their phones and queue requests; scheduler weaves them in when energy fits. Gated by your `requests_open` toggle — 403 while closed. Reuses guest lookup for identity. | 1 weekend |
| **4 — Polish** | Now-playing screen for a projector/TV, visualizer (`AnalyserNode`), wake lock, panic mode. | as time allows |

Phase 1 is the safety net: even if analysis work stalls, you have a working curated player.

## Risks & Mitigations

- **Browser autoplay policy** — `AudioContext` must start from a user gesture. The "Press Play" button *is* the gesture. Never auto-resume on page load.
- **Venue network** — after plan + first tracks load, playback is offline-capable by design (client-side buffers, background prefetch of upcoming bytes). Test by killing wifi mid-set.
- **wasm decode memory** — mitigated by the current+next buffer policy above. Test on the actual laptop.
- **Bad auto-analysis on a specific song** — every value is hand-editable in SQLite; rehearsal mode surfaces bad transitions early.
- **Laptop sleep / tab throttling** — Wake Lock + keep the tab foregrounded; Web Audio scheduling continues through minor throttling since it's clock-based, not `setTimeout`-based.
- **The absolute fallback** — a phone with the playlist in a normal music app, plugged into the PA. Free insurance.

## Constraint compliance

- `shared` additions are serde-only structs/consts — WASM-safe. ✔
- All audio/mixing code in `frontend` uses `web-sys` only. ✔
- `analyzer` is native-only and not a dependency of `shared`/`frontend`. ✔
- New env vars (`MUSIC_DIR`, `DJ_PASSCODE`) follow the existing optional-config pattern; DJ feature disables cleanly if `DJ_PASSCODE` unset. ✔
- Music lives on the existing Fly `/data` volume next to the SQLite DB. ✔

## A note on music licensing

Playing legally purchased files at a private event is normally covered by the venue's public-performance licenses (PRS/ASCAP/BMI equivalents) — worth a one-line confirmation with your venue. (Not legal advice.)
