# Phase 4 — Polish & Event-Night Hardening

**Goal:** make the live experience bulletproof and pretty. Grab-bag phase — items are independent; implement in the order listed (reliability first, eye-candy last).

**Depends on:** Phase 2 (Phase 3 optional).

## 1. Panic mode (reliability — do first)

A prominent but mis-click-safe button (hold 1s or confirm) on the player that:
- Cancels all scheduled Web Audio automation, fades the current track to full volume solo over 1s (kills any in-progress transition).
- Switches to a dumb mode: play current track to its cutoff, hard 3s fade, next track, repeat. No beat alignment, no rate nudging.
- A "resume smart mixing" button returns to normal at the next track boundary.

## 2. Wake lock + audio continuity

- Request `navigator.wakeLock.request('screen')` on Play; re-acquire on `visibilitychange` (locks are released when the tab hides). `web-sys` features: `WakeLock`, `WakeLockSentinel`, `WakeLockType`.
- On `AudioContext` state change to `interrupted`/`suspended` (some OSes do this), surface a full-screen "TAP TO RESUME AUDIO" overlay — a human will be standing at the laptop.
- Preload aggressively once playing: fetch (not decode) bytes for the next 5 steps so a total network loss ≥ 4 songs out doesn't matter.

## 3. Now-playing display

New route `/dj/screen` (any authed role): a full-screen page for a projector/TV — current song title/artist, "up next", and during Phase-3 requests-open periods, a "Request a song → <QR/urL>" banner. Poll-free: open it on the same laptop in a second window and drive it via `BroadcastChannel` (`web-sys` feature) from the player tab; fallback to 10s polling of a tiny `GET /api/dj/now` endpoint (player POSTs its state) if a second device is used.

## 4. Visualizer

On both `/dj` and `/dj/screen`: an `AnalyserNode` tapped off the master gain → canvas bars or waveform (requestAnimationFrame; `web-sys`: `AnalyserNode`, `CanvasRenderingContext2d`, `HtmlCanvasElement`). Purely cosmetic; must not run in panic mode (keep CPU headroom).

## 5. Pre-flight check screen

On the player, before Play: a checklist the DJ can see — N tracks total, N analyzed, N missing files (server verifies existence in a `GET /api/dj/preflight` host/dj endpoint), first & last song names, total planned runtime vs. reception length. Red/green rows. Catches "forgot to upload the files" at 4pm instead of 8pm.

## 6. Play log

Append each completed/skipped step to a `play_log` table (migration `20250101000011_create_play_log.sql`: id, track_id, started_at, ended_at, skipped INTEGER). POST from the player on each boundary; fire-and-forget (failures must never affect playback). Nice keepsake + debugging aid.

## Acceptance criteria

- [ ] Panic button: audible chaos (mid-crossfade) → clean solo track within ~1s; dumb mode chains tracks; resume works.
- [ ] Screen stays awake through a 30-min playback test; pulling wifi after 2 songs → next 5 still play.
- [ ] `/dj/screen` mirrors now-playing within 1s (BroadcastChannel path).
- [ ] Pre-flight correctly flags a deliberately deleted audio file.
- [ ] Visualizer renders ≥ 30fps and stops in panic mode.
- [ ] `make fmt && make clippy && make test && make check-wasm` green; `.sqlx/` committed for the play_log migration.
