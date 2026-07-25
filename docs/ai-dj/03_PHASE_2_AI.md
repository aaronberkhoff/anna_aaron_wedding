# Phase 2 — The "AI": Analyzer + Smart Mixing + Era-Arc Scheduler

**Goal:** replace hand-entered metadata and fixed crossfades with computed analysis and musically-aware transitions. Add the host order editor and rehearsal mode.

**Depends on:** Phase 1 merged and usable.

## 1. New crate `crates/analyzer` (native-only)

Add to workspace `members`. May use tokio/sqlx freely — it is never a dependency of `shared` or `frontend`. It IS covered by `make clippy`/`make test` (they exclude only `frontend`), so keep it warning-free.

Deps: `symphonia` (mp3+aac+flac+isomp4 features), `lofty` (tags), `sqlx`, `clap`, `anyhow`.

### CLI

```
cargo run -p analyzer -- --music-dir /data/music --database-url sqlite:///abs/path/wedding.db [--force] [--file <name>]
```

Scans dir, skips tracks whose `analyzed_at` is set unless `--force`/`--file`. Upserts by filename (insert if the import bin hasn't seen it).

### Pipeline per file (implement in `src/analysis/`, one module per step, each unit-tested against synthesized signals)

1. **Decode** (symphonia) → mono f32 PCM downmixed, keep sample rate.
2. **Onset envelope**: STFT (2048 window, 512 hop) → spectral flux → half-wave rectified, lightly smoothed.
3. **BPM** (`bpm.rs`): autocorrelation of the onset envelope over 60–200 BPM lags; fold octave errors (prefer 80–160); `bpm_confidence` = peak prominence ratio. Also estimate `first_beat_offset_ms` (phase of the beat grid) by maximizing onset energy at grid points — store it (add nullable column `first_beat_ms` via a small migration `20250101000009_analyzer_columns.sql`).
4. **Energy** (`energy.rs`): windowed RMS (1s windows) → track `energy` = mean of the loudest 60% of windows, normalized 0–1 across the library in a second pass.
5. **Cues** (`cues.rs`): `intro_end_ms` = first time RMS sustains above 40% of track peak for ≥2s; `outro_start_ms` = last time it falls below that for the remainder.
6. **Auto-cutoff** (`cutoff.rs`): find energy-section boundaries (sustained RMS drops ≥ 25% lasting ≥ 1.5s — chorus/verse seams). `auto_cutoff_ms` = the boundary nearest `settings.default_cutoff_ms`, snapped back to the nearest downbeat from the beat grid. If no boundary within ±45s of target, leave NULL (fallback precedence handles it).
7. **Year**: from lofty tags if `release_year` is NULL.
8. Set `analyzed_at`.

Low `bpm_confidence` (< 0.5): still store bpm but the frontend must not beat-align or tempo-nudge that track (plain timed crossfade).

## 2. Scheduler upgrade (server, `GET /api/dj/plan`)

Replace the Phase 1 crude ordering for unpinned/unordered tracks with the **era arc**, per phase group:

1. Partition: pinned-first | manual `play_order` | pool | pinned-last.
2. For the pool within each phase, build a target-year curve across the open slots: linear from `min_year` to `max_year` of the pool (so pre-2000 lands early, 2000s middle, recent late).
3. Greedy fill each slot: score each candidate `w_year·|year − target(slot)| + w_bpm·|bpm − prev_bpm| + w_energy·|energy − energy_target(slot)|` with weights (1.0, 0.6, 0.4); missing values contribute a fixed neutral penalty. Pick the min.
4. Per-step `crossfade_ms`: both tracks beat-confident and |ΔBPM|/BPM ≤ 8% → 2 bars of the outgoing track (`4 · 60000/bpm · 2`); otherwise 8000ms flat. Clamp 4000–16000.

Keep the scheduler as pure functions in its own module with unit tests (deterministic: seedless greedy).

## 3. Frontend: smart transitions

In `dj/engine.rs`:

- **Beat alignment**: when both tracks are beat-confident, snap the transition start so the incoming track's first grid beat (from `first_beat_ms` + `start_at_ms`) lands on a downbeat of the outgoing track's grid at the cutoff point.
- **Tempo nudge**: if |ΔBPM|/BPM ≤ 4%, set incoming `playback_rate` to match outgoing BPM, then `linear_ramp_to_value_at_time(1.0)` over 8s after the fade completes. Skip when either confidence < 0.5.
- Respect `end_at_ms` exactly as before — cutoffs are unchanged, they're just downbeat-snapped now by the analyzer.

## 4. Host order editor upgrade (frontend)

Replace Phase 1's buttons table with a proper editor on the host view:

- Drag-to-reorder list of the computed plan (HTML5 drag events are fine; no external JS deps). Dropping writes `play_order` for the moved track (and renumbers as needed) via `HOST_ORDER`.
- Pin/unpin first & last per phase; inline edit of cutoff (mm:ss), year, phase.
- "Reset to auto" per track (clears `play_order`) and "Recompute plan" button (re-fetches `DJ_PLAN`).

## 5. Rehearsal mode (frontend, host or DJ)

A toggle on the player: instead of full playback, play only each **transition window** (last 15s before each cutoff through 15s after the next track locks in), with a "next transition" button. Purpose: audition every seam in a 100-song set in ~20 minutes. Implementation: same engine, but seek the outgoing track to `end_at_ms − 15000` when a step starts.

## Acceptance criteria

- [ ] `make fmt && make clippy && make test && make check-wasm` green; new migration's `.sqlx/` committed.
- [ ] Analyzer unit tests: BPM within ±2 BPM on synthesized click tracks (90/120/128/150 BPM, incl. one with an octave-error trap); cue/cutoff detection on synthesized envelope shapes; energy normalization.
- [ ] Running analyzer on a real folder populates bpm/energy/cues/auto_cutoff/year; rerun without `--force` skips analyzed files.
- [ ] Plan ordering test: given a pool spanning 1970–2024, output years are monotonically non-decreasing within each phase (allowing bounded local violations from the BPM/energy terms — assert Spearman rank correlation with slot index > 0.8).
- [ ] Two beat-confident tracks with close BPM audibly transition on-beat; a low-confidence track gets a plain timed fade (no rate change).
- [ ] Rehearsal mode plays only transition windows.
- [ ] Host can drag-reorder and the persisted plan survives reload.

## Out of scope

Requests (Phase 3), key detection/harmonic mixing, tempo-stretching beyond ±4% rate nudge.
