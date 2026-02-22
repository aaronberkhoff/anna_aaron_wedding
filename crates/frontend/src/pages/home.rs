use leptos::prelude::*;
use leptos_router::components::A;
use wasm_bindgen_futures::JsFuture;

fn calc_countdown() -> (u64, u64, u64, u64) {
    let now_ms = js_sys::Date::now();
    // November 21, 2026 — noon UTC (5am MST, safely before the ceremony)
    let wedding_ms = js_sys::Date::parse("2026-11-21T18:00:00Z");
    let diff_ms = (wedding_ms - now_ms).max(0.0);
    let diff_s = (diff_ms / 1000.0) as u64;
    (
        diff_s / 86400,
        (diff_s % 86400) / 3600,
        (diff_s % 3600) / 60,
        diff_s % 60,
    )
}

async fn sleep_ms(ms: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .expect("window")
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
            .expect("setTimeout");
    });
    let _ = JsFuture::from(promise).await;
}

#[component]
pub fn Home() -> impl IntoView {
    let initial = calc_countdown();
    let (days, set_days) = signal(initial.0);
    let (hours, set_hours) = signal(initial.1);
    let (minutes, set_minutes) = signal(initial.2);
    let (seconds, set_seconds) = signal(initial.3);

    leptos::task::spawn_local(async move {
        loop {
            sleep_ms(1000).await;
            let (d, h, m, s) = calc_countdown();
            set_days.set(d);
            set_hours.set(h);
            set_minutes.set(m);
            set_seconds.set(s);
        }
    });

    view! {
        // ── Hero ──────────────────────────────────────────────────────────────
        <section class="min-h-[90vh] flex flex-col items-center justify-center text-center px-6 py-20
                        bg-gradient-to-b from-cream via-champagne to-champagne">

            // Date & location tag
            <p class="text-gold text-xs tracking-[0.35em] uppercase font-sans mb-10">
                "November 21, 2026  ·  Colorado Springs, CO"
            </p>

            // Couple's name in Dancing Script
            <h1 class="font-script text-8xl md:text-9xl text-charcoal leading-none mb-2 drop-shadow-sm">
                "Anna & Aaron"
            </h1>

            // Gold ornament divider
            <div class="flex items-center gap-4 my-8 w-72">
                <div class="flex-1 h-px bg-gold/50"></div>
                <svg class="w-3 h-3 fill-gold opacity-70" viewBox="0 0 20 20">
                    <path d="M10 0 L12.5 7.5 L20 10 L12.5 12.5 L10 20 L7.5 12.5 L0 10 L7.5 7.5 Z" />
                </svg>
                <div class="flex-1 h-px bg-gold/50"></div>
            </div>

            // Venue
            <p class="font-serif italic text-xl text-charcoal/75 mb-1">
                "Corpus Christi Catholic Church"
            </p>
            <p class="text-charcoal/50 text-sm tracking-widest mb-16 font-sans">
                "Colorado Springs, Colorado"
            </p>

            // Countdown timer
            <div class="flex items-start gap-6 md:gap-10 mb-14">
                <CountdownUnit value=days label="Days" />
                <span class="text-gold/60 text-3xl font-serif pt-2">"·"</span>
                <CountdownUnit value=hours label="Hours" />
                <span class="text-gold/60 text-3xl font-serif pt-2">"·"</span>
                <CountdownUnit value=minutes label="Minutes" />
                <span class="text-gold/60 text-3xl font-serif pt-2">"·"</span>
                <CountdownUnit value=seconds label="Seconds" />
            </div>

            // Call-to-action buttons
            <div class="flex gap-4">
                <A href="/rsvp">
                    <span class="inline-block border border-gold text-charcoal font-sans text-xs
                                  tracking-[0.2em] uppercase px-10 py-3 cursor-pointer
                                  hover:bg-gold hover:text-ivory transition-all duration-300">
                        "RSVP"
                    </span>
                </A>
                <A href="/itinerary">
                    <span class="inline-block border border-gold text-charcoal font-sans text-xs
                                  tracking-[0.2em] uppercase px-10 py-3 cursor-pointer
                                  hover:bg-gold hover:text-ivory transition-all duration-300">
                        "Itinerary"
                    </span>
                </A>
            </div>
        </section>

        // ── Welcome section ───────────────────────────────────────────────────
        <section class="max-w-xl mx-auto px-6 py-20 text-center">
            <h2 class="font-serif text-3xl text-charcoal mb-6">
                "We\u{2019}re getting married!"
            </h2>
            <p class="text-charcoal/60 leading-relaxed font-sans">
                "We are so excited to celebrate this special day surrounded by the people we love. "
                "Please use the navigation above to RSVP, find your seat, and explore hotel options."
            </p>
        </section>
    }
}

#[component]
fn CountdownUnit(value: ReadSignal<u64>, label: &'static str) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center gap-2">
            <span class="font-serif text-5xl text-charcoal tabular-nums">
                {move || format!("{:02}", value.get())}
            </span>
            <span class="text-[10px] text-charcoal/40 uppercase tracking-[0.25em] font-sans">
                {label}
            </span>
        </div>
    }
}
