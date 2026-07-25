//! Phase 0 spike: proves a Leptos CSR component can decode two MP3s and play a
//! sample-accurate equal-power crossfade via `web-sys` Web Audio. Throwaway
//! quality per docs/ai-dj/01_PHASE_0_SPIKE.md — Phase 1 refactors this into
//! the real mixing engine. Temporary route, removed in Phase 1.

use std::f64::consts::FRAC_PI_2;
use std::time::Duration;

use js_sys::ArrayBuffer;
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, GainNode, Request, Response};

const TRACK_A_URL: &str = "/spike/a.m4a";
const TRACK_B_URL: &str = "/spike/b.m4a";
const CROSSFADE_START_S: f64 = 20.0;
const CROSSFADE_DURATION_S: f64 = 10.0;
const CURVE_STEPS: usize = 50;
/// Small lead-in before A starts, so `start_with_when` is never scheduled in the past.
const START_LEAD_S: f64 = 0.1;

/// Holds every Web Audio object that must stay alive for the whole playback.
/// Dropping any of these (letting them go out of scope) silences the graph.
#[allow(dead_code)]
struct Playback {
    ctx: AudioContext,
    source_a: AudioBufferSourceNode,
    source_b: AudioBufferSourceNode,
    gain_a: GainNode,
    gain_b: GainNode,
    buffer_a: AudioBuffer,
    buffer_b: AudioBuffer,
}

#[component]
pub fn DjSpike() -> impl IntoView {
    let status = RwSignal::new(String::from("Ready"));
    let playing = RwSignal::new(false);
    // web-sys types are !Send, so this needs local (single-threaded) signal storage.
    let playback = RwSignal::new_local(None::<Playback>);

    let on_click = move |_| {
        if playing.get_untracked() {
            return;
        }
        playing.set(true);
        status.set("loading…".to_string());

        leptos::task::spawn_local(async move {
            // A previous run's AudioContext is still open if Play was pressed
            // more than once — close it before starting a fresh one. Uses
            // try_update (not get) since Playback holds !Clone web-sys types.
            if let Some(prev) = playback.try_update(|p| p.take()).flatten() {
                let _ = prev.ctx.close();
            }

            match run_crossfade(status).await {
                Ok(built) => playback.set(Some(built)),
                Err(e) => status.set(format!("error: {e}")),
            }
            playing.set(false);
        });
    };

    view! {
        <div class="max-w-xl mx-auto px-6 py-12 flex flex-col items-center gap-6">
            <h1 class="font-serif text-3xl text-charcoal">"AI DJ — Crossfade Spike"</h1>
            <p class="text-sm text-charcoal/60 text-center">
                "Plays site/spike/a.m4a, then crossfades into site/spike/b.m4a "
                "starting 20s in, over a 10s equal-power fade."
            </p>
            <button
                on:click=on_click
                disabled=move || playing.get()
                class="bg-gold text-ivory font-semibold px-6 py-3 rounded disabled:opacity-50"
            >
                "Play"
            </button>
            <p class="text-charcoal/70">{move || status.get()}</p>
        </div>
    }
}

async fn run_crossfade(status: RwSignal<String>) -> Result<Playback, String> {
    let ctx = AudioContext::new().map_err(js_err)?;

    let raw_a = fetch_array_buffer(TRACK_A_URL).await?;
    let raw_b = fetch_array_buffer(TRACK_B_URL).await?;

    let buffer_a = decode(&ctx, raw_a).await?;
    let buffer_b = decode(&ctx, raw_b).await?;

    let gain_a = ctx.create_gain().map_err(js_err)?;
    let gain_b = ctx.create_gain().map_err(js_err)?;
    gain_a
        .connect_with_audio_node(&ctx.destination())
        .map_err(js_err)?;
    gain_b
        .connect_with_audio_node(&ctx.destination())
        .map_err(js_err)?;

    let source_a = ctx.create_buffer_source().map_err(js_err)?;
    source_a.set_buffer(Some(&buffer_a));
    source_a.connect_with_audio_node(&gain_a).map_err(js_err)?;

    let source_b = ctx.create_buffer_source().map_err(js_err)?;
    source_b.set_buffer(Some(&buffer_b));
    source_b.connect_with_audio_node(&gain_b).map_err(js_err)?;

    // Schedule everything up front against the AudioContext's own clock —
    // the crossfade must survive tab throttling, so nothing here may be
    // driven by JS timers.
    let now = ctx.current_time();
    let start_a = now + START_LEAD_S;
    let crossfade_start = start_a + CROSSFADE_START_S;
    let crossfade_end = crossfade_start + CROSSFADE_DURATION_S;

    source_a.start_with_when(start_a).map_err(js_err)?;
    source_b.start_with_when(crossfade_start).map_err(js_err)?;

    let mut curve_a = equal_power_curve(true);
    let mut curve_b = equal_power_curve(false);
    gain_a
        .gain()
        .set_value_curve_at_time(&mut curve_a, crossfade_start, CROSSFADE_DURATION_S)
        .map_err(js_err)?;
    gain_b
        .gain()
        .set_value_curve_at_time(&mut curve_b, crossfade_start, CROSSFADE_DURATION_S)
        .map_err(js_err)?;

    // Status text is cosmetic only — approximate the timings with one-shot
    // JS timers relative to wall time. Does not touch the audio graph.
    let delay_playing_a = Duration::from_secs_f64((start_a - now).max(0.0));
    let delay_crossfading = Duration::from_secs_f64((crossfade_start - now).max(0.0));
    let delay_playing_b = Duration::from_secs_f64((crossfade_end - now).max(0.0));
    set_timeout(move || status.set("playing A".to_string()), delay_playing_a);
    set_timeout(
        move || status.set("crossfading".to_string()),
        delay_crossfading,
    );
    set_timeout(move || status.set("playing B".to_string()), delay_playing_b);

    Ok(Playback {
        ctx,
        source_a,
        source_b,
        gain_a,
        gain_b,
        buffer_a,
        buffer_b,
    })
}

/// Equal-power crossfade curve: cos(x*pi/2) fading out, sin(x*pi/2) fading in.
fn equal_power_curve(fade_out: bool) -> Vec<f32> {
    (0..=CURVE_STEPS)
        .map(|i| {
            let x = i as f64 / CURVE_STEPS as f64;
            let y = if fade_out {
                (x * FRAC_PI_2).cos()
            } else {
                (x * FRAC_PI_2).sin()
            };
            y as f32
        })
        .collect()
}

async fn fetch_array_buffer(path: &str) -> Result<ArrayBuffer, String> {
    let request = Request::new_with_str(path).map_err(js_err)?;
    let window = web_sys::window().ok_or("no window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(js_err)?;

    let resp: Response = resp_value.dyn_into().map_err(js_err)?;
    if !resp.ok() {
        return Err(format!("HTTP {} fetching {path}", resp.status()));
    }

    let buf_value = JsFuture::from(resp.array_buffer().map_err(js_err)?)
        .await
        .map_err(js_err)?;
    buf_value.dyn_into::<ArrayBuffer>().map_err(js_err)
}

async fn decode(ctx: &AudioContext, raw: ArrayBuffer) -> Result<AudioBuffer, String> {
    let promise = ctx.decode_audio_data(&raw).map_err(js_err)?;
    let value = JsFuture::from(promise).await.map_err(js_err)?;
    value.dyn_into::<AudioBuffer>().map_err(js_err)
}

fn js_err(e: JsValue) -> String {
    format!("{e:?}")
}
