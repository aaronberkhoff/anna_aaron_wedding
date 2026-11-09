use crate::pages::shuttle::{Stop, OCC_BREWING, S_25TH_ST};
use leptos::prelude::*;

const MOTHER_MUFFS: Stop = Stop {
    name: "Mother Muff\u{2019}s",
    address: "2432 W Colorado Ave, Colorado Springs, CO 80904",
    query: "2432+W+Colorado+Ave+Colorado+Springs+CO+80904",
};

struct CrawlStop {
    label: &'static str,
    title: &'static str,
    stop: &'static Stop,
    description: &'static str,
}

const CRAWL: &[CrawlStop] = &[
    CrawlStop {
        label: "Stop 1 \u{00b7} About 3:00 PM",
        title: "OCC Brewing",
        stop: &OCC_BREWING,
        description: "The shuttle from Corpus Christi leaves by 3:00 PM and drops guests off \
                      right here. Kick things off with a local craft beer in the heart of Old \
                      Colorado City.",
    },
    CrawlStop {
        label: "Stop 2",
        title: "Mother Muff\u{2019}s",
        stop: &MOTHER_MUFFS,
        description: "About a block west on Colorado Ave \u{2014} an easy walk from OCC Brewing.",
    },
    CrawlStop {
        label: "4:30 PM",
        title: "Shuttle to the reception",
        stop: &S_25TH_ST,
        description: "Walk west to 25th St and turn south \u{2014} the shuttle picks up on the \
                      side street at 4:30 PM and heads to Red Rocks Barn. Please don\u{2019}t \
                      miss it!",
    },
];

fn walking_route_embed() -> String {
    format!(
        "https://maps.google.com/maps?saddr={}&daddr={}+to:{}&dirflg=w&output=embed",
        OCC_BREWING.query, MOTHER_MUFFS.query, S_25TH_ST.query
    )
}

#[component]
pub fn BarCrawl() -> impl IntoView {
    view! {
        <div class="max-w-2xl mx-auto px-6 py-16">
            <h1 class="font-serif text-4xl text-charcoal mb-2 text-center">"Bar Crawl"</h1>
            <p class="text-center text-charcoal/50 text-sm tracking-widest font-sans mb-6">
                "Saturday · November 21, 2026  ·  Old Colorado City"
            </p>
            <div class="flex items-center gap-4 justify-center mb-6 w-48 mx-auto">
                <div class="flex-1 h-px bg-gold/40"></div>
                <svg class="w-2 h-2 fill-gold opacity-60" viewBox="0 0 20 20">
                    <path d="M10 0 L12.5 7.5 L20 10 L12.5 12.5 L10 20 L7.5 12.5 L0 10 L7.5 7.5 Z" />
                </svg>
                <div class="flex-1 h-px bg-gold/40"></div>
            </div>
            <p class="text-center text-charcoal/70 font-sans text-sm leading-relaxed mb-12">
                "Between the Mass and the reception, join us for a casual walk between two of "
                "our favorite spots in Old Colorado City. The shuttle drops you off and picks "
                "you up \u{2014} no driving needed."
            </p>

            <ol class="relative border-l border-gold/40 ml-4 mb-12">
                {CRAWL.iter().map(|c| view! {
                    <li class="mb-10 ml-8">
                        <span class="absolute -left-[9px] w-[17px] h-[17px] rounded-full
                                     bg-champagne border-2 border-gold mt-1"></span>
                        <p class="text-xs text-gold tracking-[0.2em] uppercase font-sans mb-1">
                            {c.label}
                        </p>
                        <h2 class="font-serif text-xl text-charcoal mb-1">{c.title}</h2>
                        <a
                            href=c.stop.maps_url()
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-sm text-sage underline hover:text-earth transition-colors"
                        >
                            {c.stop.address}
                        </a>
                        <p class="text-charcoal/60 font-sans text-sm leading-relaxed mt-1">
                            {c.description}
                        </p>
                    </li>
                }).collect_view()}
            </ol>

            <section class="space-y-3">
                <h2 class="font-serif text-2xl text-charcoal text-center">"Walking Route"</h2>
                <p class="text-center text-charcoal/60 font-sans text-sm">
                    "OCC Brewing \u{2192} Mother Muff\u{2019}s \u{2192} shuttle pick-up on S 25th St"
                </p>
                <iframe
                    src=walking_route_embed()
                    title="Bar crawl walking route"
                    class="w-full h-80 rounded border border-gold/40 bg-cream"
                    referrerpolicy="no-referrer-when-downgrade"
                ></iframe>
                <p class="text-center text-charcoal/60 font-sans text-sm">
                    "See the "
                    <a href="/shuttle" class="text-sage underline hover:text-earth transition-colors">
                        "shuttle schedule"
                    </a>
                    " for all pick-up times."
                </p>
            </section>
        </div>
    }
}
