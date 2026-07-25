# Phase 3 (Optional) — Guest Song Requests

**Goal:** guests browse the pre-approved library from their phones and queue requests. The host (Aaron) alone controls whether requests are being fielded, via the `requests_open` toggle on his login. The scheduler weaves accepted requests in when energy/era fit.

**Depends on:** Phase 2 (uses scheduler + settings toggle already persisted in Phase 1).

## 1. Migration — `20250101000010_create_requests.sql`

```sql
CREATE TABLE requests (
    id          INTEGER PRIMARY KEY,
    track_id    INTEGER NOT NULL REFERENCES tracks(id),
    guest_name  TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    status      TEXT NOT NULL DEFAULT 'pending'   -- pending | queued | played | declined
);
```

`make migrate` → `make db-prepare` → commit `.sqlx/`.

## 2. Shared

Routes:

```rust
pub const DJ_REQUESTS: &str = "/api/dj/requests";              // POST (public), GET (auth)
pub const DJ_REQUESTABLE: &str = "/api/dj/requestable";        // GET (public): library for guests
pub const HOST_REQUEST_STATUS: &str = "/api/dj/host/requests"; // frontend appends /{id}
```

Models: `RequestSubmit { track_id, guest_name }`, `SongRequest { id, track_id, title, artist, guest_name, status, created_at }`.

## 3. Server (`handlers/dj.rs`)

- `GET /api/dj/requestable` — **public, no auth**: id/title/artist/release_year only (no filenames, no audio access). Returns 403 body-level flag or empty behavior consistent with below when closed — simplest: also gate it with `requests_open` so the guest page can show "requests are closed".
- `POST /api/dj/requests` — **public**: validates track exists; **403 unless `settings.requests_open = 1`** (read per-request, so the host toggle takes effect immediately). Basic abuse guard: reject duplicate (track_id) pending requests; cap 3 pending per guest_name.
- `GET /api/dj/requests` — any authed role: full list with status.
- `PATCH /api/dj/host/requests/{id}` — host only: set status (`queued`/`declined`).

## 4. Guest UI

New route `/requests` (public, linked from nav or QR code on tables): search/filter the requestable library, tap to request with your name. Shows "Requests are closed right now" when the toggle is off. Uses guest-facing styling consistent with existing pages.

## 5. Player integration (frontend)

- Host view: pending requests panel (poll `GET /api/dj/requests` every 15s — no websockets needed) with Accept/Decline. The **Requests: Open/Closed** header toggle from Phase 1 now actually gates submissions.
- On Accept (`status=queued`): the plan recomputes with the requested track inserted into the **upcoming** pool — scheduler places it at the slot minimizing its normal score (era/BPM/energy) among the next ~10 slots, never displacing pins or manual `play_order`. Mark `played` when its step completes.
- The DJ role sees the queue read-only; only host accepts/declines and toggles.

## Acceptance criteria

- [ ] Toggle off: `POST /api/dj/requests` → 403; guest page shows closed state. Toggle on (host UI): submissions succeed with no server restart.
- [ ] DJ token cannot change request status or the toggle (403); host can.
- [ ] Accepted request appears in "up next" within one plan recompute, at a sensible slot (test: scheduler unit test asserting insertion respects pins/manual order).
- [ ] Duplicate + per-guest caps enforced.
- [ ] `make fmt && make clippy && make test && make check-wasm` green; `.sqlx/` committed.
