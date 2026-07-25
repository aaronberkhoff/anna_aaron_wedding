# Phase 0 — Web Audio Crossfade Spike

**Goal:** prove that a Leptos 0.7 CSR component can decode two MP3s and play a smooth, sample-accurate equal-power crossfade between them using `web-sys` Web Audio. Everything else in this project is routine; this is the one risky part.

**No server changes. No migrations. Throwaway-quality code is fine, but keep it — Phase 1 refactors it into the real engine.**

## Prerequisites

- Two MP3 files placed at `site/spike/a.mp3` and `site/spike/b.mp3` (any two songs; ask the user to supply them or use any CC-licensed tracks). Ensure Trunk copies them (e.g. `data-trunk rel="copy-dir"` for `site/spike` in `index.html`, or place them wherever the existing static assets pattern puts them).

## Changes

### 1. Root `Cargo.toml` — extend `web-sys` features

Add to the existing `web-sys` feature list:

```
"AudioContext", "AudioBuffer", "AudioBufferSourceNode", "AudioDestinationNode",
"AudioNode", "AudioParam", "GainNode", "BaseAudioContext",
```

### 2. New file `crates/frontend/src/pages/dj_spike.rs`

A component registered at route `/dj-spike` in `app.rs` (temporary route, removed in Phase 1). Behavior:

- Renders a single **Play** button. Nothing auto-plays — browsers require a user gesture to start an `AudioContext`.
- On click:
  1. Create one `AudioContext`.
  2. `fetch` both MP3s as `ArrayBuffer` (via `web_sys` Request/Response, same style as `api/client.rs`, but `resp.array_buffer()` instead of `.json()`).
  3. Decode with `ctx.decode_audio_data(&buf)` → `JsFuture` → `AudioBuffer`.
  4. Build two chains: `AudioBufferSourceNode -> GainNode -> ctx.destination()`.
  5. Play track A from t=0. At `t = crossfade_start` (hardcode: 20s into A), start track B and run a **10s equal-power crossfade**: gain_a = cos(x·π/2), gain_b = sin(x·π/2) for x∈[0,1]. Implement with `AudioParam::set_value_curve_at_time` (preferred) or a series of `linear_ramp_to_value_at_time` steps — schedule everything up front against `ctx.current_time()`; do NOT drive gain from JS timers.
- Show simple state text: "loading…" / "playing A" / "crossfading" / "playing B".

### 3. Notes for the implementer

- `decode_audio_data` returns a `Promise`; wrap with `wasm_bindgen_futures::JsFuture`. The whole click handler is an async task via `spawn_local`.
- `AudioBufferSourceNode` is one-shot: create it at schedule time, call `start_with_when(t)`.
- Keep both decoded buffers in scope until playback ends (drop = silence).
- Expect ~85 MB RAM per decoded 4-min track; irrelevant for the spike, critical later.

## Acceptance criteria

- [ ] `make check-wasm` passes; `make clippy` (native, unaffected) passes.
- [ ] `trunk serve` → `/dj-spike` → clicking Play produces: A plays alone, then a 10s smooth overlap where B fades in as A fades out (no volume dip — that's what equal-power means), then B alone.
- [ ] No audio glitches when the tab is briefly backgrounded during the crossfade (clock-based scheduling survives throttling).
- [ ] Refreshing mid-play and pressing Play again works (no stuck AudioContext).

## Out of scope

Cutoffs, auth, server routes, BPM alignment, more than two tracks.
