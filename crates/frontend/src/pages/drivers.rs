//! Driver-facing shuttle page. Unlisted (not in the nav) — share the /drivers link
//! with the bus company. Stop addresses come from `shuttle.rs` so both pages agree.

use crate::pages::shuttle::{
    Stop, CORPUS_CHRISTI, OCC_BREWING, RED_ROCKS_BARN, SPRINGHILL, S_25TH_ST,
};
use leptos::prelude::*;
use leptos_meta::Meta;

enum Step {
    Drive {
        label: &'static str,
        title: &'static str,
        timing: &'static str,
        from: &'static Stop,
        to: &'static Stop,
        notes: &'static [&'static str],
        /// Show a satellite close-up of the pick-up spot.
        pickup_closeup: bool,
    },
    Stage {
        title: &'static str,
        timing: &'static str,
        at: &'static Stop,
        notes: &'static [&'static str],
    },
}

const STEPS: &[Step] = &[
    Step::Drive {
        label: "Run 1",
        title: "Hotel \u{2192} Church",
        timing: "Arrive 12:35 PM \u{00b7} Depart 12:45 PM",
        from: &SPRINGHILL,
        to: &CORPUS_CHRISTI,
        notes: &["Be outside the hotel by 12:35 PM. Leave at 12:45 PM."],
        pickup_closeup: false,
    },
    Step::Stage {
        title: "Wait during the Mass",
        timing: "Until 2:30 PM",
        at: &CORPUS_CHRISTI,
        notes: &[
            "Park in the parking lot behind the church, or go elsewhere.",
            "Be back in the lot behind the church by 2:30 PM.",
        ],
    },
    Step::Drive {
        label: "Run 2",
        title: "Church \u{2192} OCC Brewing",
        timing: "In lot by 2:30 PM \u{00b7} Depart by 3:00 PM",
        from: &CORPUS_CHRISTI,
        to: &OCC_BREWING,
        notes: &[
            "Pick up guests in the parking lot behind the church.",
            "Drop off near OCC Brewing, 2316 W Colorado Ave.",
        ],
        pickup_closeup: false,
    },
    Step::Drive {
        label: "Run 3",
        title: "S 25th St \u{2192} Red Rocks Barn",
        timing: "Pick up 4:30 PM",
        from: &S_25TH_ST,
        to: &RED_ROCKS_BARN,
        notes: &[
            "Pick up at 110 S 25th St. This is a side street \u{2014} pull off to the side \
             into one of the street parking spots so guests can board.",
        ],
        pickup_closeup: true,
    },
    Step::Stage {
        title: "Wait during the reception",
        timing: "Until 8:45 PM",
        at: &RED_ROCKS_BARN,
        notes: &[
            "Stay in the Red Rocks Barn parking lot, or go elsewhere.",
            "Be back at the Barn by 8:45 PM.",
        ],
    },
    Step::Drive {
        label: "Run 4",
        title: "Red Rocks Barn \u{2192} Hotel",
        timing: "At Barn by 8:45 PM \u{00b7} Depart no later than 9:10 PM",
        from: &RED_ROCKS_BARN,
        to: &SPRINGHILL,
        notes: &["Take guests back to the SpringHill Suites hotel block."],
        pickup_closeup: false,
    },
    Step::Drive {
        label: "Run 5",
        title: "Hotel \u{2192} Red Rocks Barn (empty)",
        timing: "Back at Barn no later than 9:45 PM",
        from: &SPRINGHILL,
        to: &RED_ROCKS_BARN,
        notes: &[
            "After dropping off, return to the Barn for the wedding party and the people \
             who stayed to help pack up.",
        ],
        pickup_closeup: false,
    },
    Step::Drive {
        label: "Run 6",
        title: "Red Rocks Barn \u{2192} Hotel (final)",
        timing: "After 9:45 PM",
        from: &RED_ROCKS_BARN,
        to: &SPRINGHILL,
        notes: &["Drop off the final guests at the SpringHill Suites hotel block."],
        pickup_closeup: false,
    },
];

fn directions_embed(from: &Stop, to: &Stop) -> String {
    format!(
        "https://maps.google.com/maps?saddr={}&daddr={}&output=embed",
        from.query, to.query
    )
}

fn navigate_url(from: &Stop, to: &Stop) -> String {
    format!(
        "https://www.google.com/maps/dir/?api=1&origin={}&destination={}&travelmode=driving",
        from.query, to.query
    )
}

fn satellite_embed(at: &Stop) -> String {
    format!(
        "https://maps.google.com/maps?q={}&t=k&z=18&output=embed",
        at.query
    )
}

fn overview_embed() -> String {
    let stops = [
        &CORPUS_CHRISTI,
        &OCC_BREWING,
        &S_25TH_ST,
        &RED_ROCKS_BARN,
        &SPRINGHILL,
    ];
    let daddr = stops
        .iter()
        .map(|s| s.query)
        .collect::<Vec<_>>()
        .join("+to:");
    format!(
        "https://maps.google.com/maps?saddr={}&daddr={}&output=embed",
        SPRINGHILL.query, daddr
    )
}

#[component]
fn MapFrame(src: String, title: String) -> impl IntoView {
    view! {
        <iframe
            src=src
            title=title
            class="w-full h-72 rounded border border-gold/40 bg-cream"
            referrerpolicy="no-referrer-when-downgrade"
        ></iframe>
    }
}

#[component]
fn Notes(notes: &'static [&'static str]) -> impl IntoView {
    view! {
        <ul class="list-disc list-inside text-sm text-charcoal/80 space-y-1">
            {notes.iter().map(|n| view! { <li>{*n}</li> }).collect_view()}
        </ul>
    }
}

#[component]
fn StepCard(step: &'static Step) -> impl IntoView {
    match step {
        Step::Drive {
            label,
            title,
            timing,
            from,
            to,
            notes,
            pickup_closeup,
        } => view! {
            <section class="border border-gold rounded-lg bg-ivory p-5 space-y-4">
                <div>
                    <p class="text-xs text-gold tracking-[0.2em] uppercase font-sans">{*label}</p>
                    <h2 class="font-serif text-2xl text-charcoal">{*title}</h2>
                    <p class="text-sm font-semibold text-charcoal mt-1">{*timing}</p>
                </div>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 text-sm">
                    <div>
                        <p class="text-xs font-semibold uppercase tracking-wide text-charcoal/60">"From"</p>
                        <p class="text-charcoal">{from.name}</p>
                        <p class="text-charcoal/70">{from.address}</p>
                    </div>
                    <div>
                        <p class="text-xs font-semibold uppercase tracking-wide text-charcoal/60">"To"</p>
                        <p class="text-charcoal">{to.name}</p>
                        <p class="text-charcoal/70">{to.address}</p>
                    </div>
                </div>
                <Notes notes=*notes />
                <MapFrame src=directions_embed(from, to) title=format!("Route: {title}") />
                {pickup_closeup.then(|| view! {
                    <div class="space-y-1">
                        <p class="text-xs font-semibold uppercase tracking-wide text-charcoal/60">
                            "Pick-up spot (satellite)"
                        </p>
                        <MapFrame src=satellite_embed(from) title=format!("Pick-up: {}", from.name) />
                    </div>
                })}
                <a
                    href=navigate_url(from, to)
                    target="_blank"
                    rel="noopener noreferrer"
                    class="block text-center bg-gold text-ivory font-semibold px-6 py-3 rounded hover:bg-charcoal transition-colors"
                >
                    "Navigate in Google Maps"
                </a>
            </section>
        }
        .into_any(),
        Step::Stage {
            title,
            timing,
            at,
            notes,
        } => view! {
            <section class="border border-dashed border-sage rounded-lg bg-cream p-5 space-y-4">
                <div>
                    <p class="text-xs text-sage tracking-[0.2em] uppercase font-sans">"Staging"</p>
                    <h2 class="font-serif text-2xl text-charcoal">{*title}</h2>
                    <p class="text-sm font-semibold text-charcoal mt-1">{*timing}</p>
                    <p class="text-sm text-charcoal/70 mt-1">{at.name}" \u{00b7} "{at.address}</p>
                </div>
                <Notes notes=*notes />
                <MapFrame src=satellite_embed(at) title=format!("Staging: {}", at.name) />
            </section>
        }
        .into_any(),
    }
}

#[component]
pub fn Drivers() -> impl IntoView {
    view! {
        <Meta name="robots" content="noindex, nofollow" />
        <div class="max-w-3xl mx-auto px-4 sm:px-6 py-12 space-y-8">
            <div class="text-center">
                <h1 class="font-serif text-4xl text-charcoal mb-2">"Shuttle Driver Guide"</h1>
                <p class="text-charcoal/60 text-sm tracking-widest font-sans">
                    "Saturday \u{00b7} November 21, 2026 \u{00b7} Anna & Aaron\u{2019}s Wedding"
                </p>
            </div>

            // ── At a glance ────────────────────────────────────────────────
            <section class="border border-gold rounded-lg bg-ivory p-5">
                <h2 class="font-serif text-2xl text-charcoal mb-3">"At a Glance"</h2>
                <table class="w-full text-sm">
                    <tbody class="divide-y divide-gold/30">
                        <GlanceRow time="12:35 PM" what="At SpringHill Suites (leave 12:45)" />
                        <GlanceRow time="2:30 PM" what="In lot behind Corpus Christi (leave by 3:00)" />
                        <GlanceRow time="4:30 PM" what="Pick up at 110 S 25th St \u{2192} Red Rocks Barn" />
                        <GlanceRow time="8:45 PM" what="At Red Rocks Barn (leave by 9:10 \u{2192} hotel)" />
                        <GlanceRow time="9:45 PM" what="Back at Red Rocks Barn \u{2192} final run to hotel" />
                    </tbody>
                </table>
            </section>

            // ── Whole-day loop ─────────────────────────────────────────────
            <section class="space-y-2">
                <h2 class="font-serif text-2xl text-charcoal">"Full Route"</h2>
                <p class="text-sm text-charcoal/70">
                    "SpringHill Suites \u{2192} Corpus Christi \u{2192} OCC Brewing \u{2192} "
                    "S 25th St \u{2192} Red Rocks Barn \u{2192} SpringHill Suites"
                </p>
                <MapFrame src=overview_embed() title="Full day route".to_string() />
            </section>

            {STEPS.iter().map(|step| view! { <StepCard step=step /> }).collect_view()}
        </div>
    }
}

#[component]
fn GlanceRow(time: &'static str, what: &'static str) -> impl IntoView {
    view! {
        <tr>
            <td class="py-2 pr-4 font-semibold text-charcoal whitespace-nowrap align-top">{time}</td>
            <td class="py-2 text-charcoal/80">{what}</td>
        </tr>
    }
}
