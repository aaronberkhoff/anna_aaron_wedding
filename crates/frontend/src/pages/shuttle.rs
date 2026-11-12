use leptos::prelude::*;

/// A shuttle stop. Shared with the driver page (`drivers.rs`) so addresses stay in sync.
pub(crate) struct Stop {
    pub name: &'static str,
    pub address: &'static str,
    /// URL-encoded address for Google Maps queries.
    pub query: &'static str,
}

impl Stop {
    pub fn maps_url(&self) -> String {
        format!(
            "https://www.google.com/maps/search/?api=1&query={}",
            self.query
        )
    }
}

struct Leg {
    time: &'static str,
    title: &'static str,
    pickup: Stop,
    dropoff: Stop,
    note: Option<&'static str>,
}

pub(crate) const SPRINGHILL: Stop = Stop {
    name: "SpringHill Suites Downtown",
    address: "402 S Tejon St, Colorado Springs, CO 80903",
    query: "402+S+Tejon+St+Colorado+Springs+CO+80903",
};

pub(crate) const CORPUS_CHRISTI: Stop = Stop {
    name: "Corpus Christi Catholic Church",
    address: "2318 N Cascade Ave, Colorado Springs, CO 80907",
    query: "2318+N+Cascade+Ave+Colorado+Springs+CO+80907",
};

pub(crate) const OCC_BREWING: Stop = Stop {
    name: "Near OCC Brewing",
    address: "2316 W Colorado Ave, Colorado Springs, CO 80904",
    query: "2316+W+Colorado+Ave+Colorado+Springs+CO+80904",
};

pub(crate) const S_25TH_ST: Stop = Stop {
    name: "S 25th St (side street)",
    address: "110 S 25th St, Colorado Springs, CO 80904",
    query: "110+S+25th+St+Colorado+Springs+CO+80904",
};

pub(crate) const RED_ROCKS_BARN: Stop = Stop {
    name: "Red Rocks Barn",
    address: "2700 Robinson St, Colorado Springs, CO 80904",
    query: "2700+Robinson+St+Colorado+Springs+CO+80904",
};

const LEGS: &[Leg] = &[
    Leg {
        time: "12:35 PM \u{2014} departs 12:45 PM",
        title: "Hotel to the Mass",
        pickup: SPRINGHILL,
        dropoff: CORPUS_CHRISTI,
        note: Some("Buses will be waiting outside the hotel at 12:35 PM and leave promptly at 12:45 PM."),
    },
    Leg {
        time: "2:30 PM \u{2014} departs by 3:00 PM",
        title: "Church to the Bar Crawl",
        pickup: CORPUS_CHRISTI,
        dropoff: OCC_BREWING,
        note: Some("Board in the parking lot behind the church after the Mass."),
    },
    Leg {
        time: "4:30 PM",
        title: "Bar Crawl to the Reception",
        pickup: S_25TH_ST,
        dropoff: RED_ROCKS_BARN,
        note: Some("Buses will pull over along the street parking on S 25th St \u{2014} look for them on the side of the road."),
    },
    Leg {
        time: "Departs 9:15 PM",
        title: "Reception to the Hotel",
        pickup: RED_ROCKS_BARN,
        dropoff: SPRINGHILL,
        note: None,
    },
    Leg {
        time: "About 9:45 PM",
        title: "Final Run to the Hotel",
        pickup: RED_ROCKS_BARN,
        dropoff: SPRINGHILL,
        note: Some(
            "After the first drop-off, buses return to the Barn for the wedding party and \
             anyone who stayed to help pack up.",
        ),
    },
];

#[component]
fn StopRow(label: &'static str, stop: &'static Stop) -> impl IntoView {
    view! {
        <div>
            <p class="text-xs font-semibold uppercase tracking-wide text-charcoal/60 mb-0.5">{label}</p>
            <p class="text-sm text-charcoal">{stop.name}</p>
            <a
                href=stop.maps_url()
                target="_blank"
                rel="noopener noreferrer"
                class="text-sm text-sage underline hover:text-earth transition-colors"
            >
                {stop.address}
            </a>
        </div>
    }
}

#[component]
pub fn Shuttle() -> impl IntoView {
    view! {
        <div class="max-w-2xl mx-auto px-6 py-16">
            <h1 class="font-serif text-4xl text-charcoal mb-2 text-center">"Shuttle Schedule"</h1>
            <p class="text-center text-charcoal/50 text-sm tracking-widest font-sans mb-6">
                "Saturday · November 21, 2026"
            </p>
            <div class="flex items-center gap-4 justify-center mb-6 w-48 mx-auto">
                <div class="flex-1 h-px bg-gold/40"></div>
                <svg class="w-2 h-2 fill-gold opacity-60" viewBox="0 0 20 20">
                    <path d="M10 0 L12.5 7.5 L20 10 L12.5 12.5 L10 20 L7.5 12.5 L0 10 L7.5 7.5 Z" />
                </svg>
                <div class="flex-1 h-px bg-gold/40"></div>
            </div>
            <p class="text-center text-charcoal/70 font-sans text-sm leading-relaxed mb-14">
                "Shuttle buses will carry guests from the SpringHill Suites hotel block to "
                "the Mass, the bar crawl, the reception, and back to the hotel at the end of "
                "the night. Times listed are when the bus is ready to board."
            </p>

            <ol class="relative border-l border-gold/40 ml-4">
                {LEGS.iter().map(|leg| view! {
                    <li class="mb-10 ml-8">
                        <span class="absolute -left-[9px] w-[17px] h-[17px] rounded-full
                                     bg-champagne border-2 border-gold mt-1"></span>
                        <p class="text-xs text-gold tracking-[0.2em] uppercase font-sans mb-1">
                            {leg.time}
                        </p>
                        <h2 class="font-serif text-xl text-charcoal mb-3">{leg.title}</h2>
                        <div class="border border-gold/40 rounded-lg bg-ivory p-4 grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <StopRow label="Pick up" stop=&leg.pickup />
                            <StopRow label="Drop off" stop=&leg.dropoff />
                        </div>
                        {leg.note.map(|note| view! {
                            <p class="text-charcoal/60 font-sans text-sm leading-relaxed mt-3">{note}</p>
                        })}
                    </li>
                }).collect_view()}
            </ol>
        </div>
    }
}
