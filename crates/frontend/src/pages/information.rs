use leptos::prelude::*;

struct Event {
    time: &'static str,
    title: &'static str,
    description: &'static str,
    location: &'static str,
}

const EVENTS: &[Event] = &[
    Event {
        time: "12:45 PM",
        title: "Shuttle to the Mass",
        location: "SpringHill Suites Downtown",
        description: "Shuttle buses leave the hotel block for the church (board from 12:35 PM)",
    },
    Event {
        time: "1:30 PM",
        title: "Nuptial Mass",
        location: "Corpus Christi Catholic Church",
        description: "Wedding Mass",
    },
    Event {
        time: "2:45 PM",
        title: "Break",
        location: "Old Colorado City",
        description:
            "Wedding party pictures. Shuttle buses take guests from the church to the bar \
             crawl in Old Colorado City (leaving by 3:00 PM).",
    },
    Event {
        time: "4:45 PM",
        title: "Reception",
        location: "Red Rocks Barn",
        description: "Anna and Aaron will welcome guests for cocktail hour",
    },
    Event {
        time: "6:00 PM",
        title: "Dinner",
        description: "Dinner at Red Rocks Barn.",
        location: "Red Rocks Barn",
    },
    Event {
        time: "7:15 PM",
        title: "First Dance",
        location: "Red Rocks Barn",
        description: "First dance and opening the dance floor",
    },
    Event {
        time: "9:00 PM",
        title: "Send-off",
        description: "Exit from the venue \u{2014} shuttle buses back to the hotel block \
                      (leaving by 9:10 PM)",
        location: "Red Rocks Barn",
    },
    Event {
        time: "9:30 PM",
        title: "After Party",
        location: "Cowboys \u{00b7} 25 N Tejon St",
        description:
            "Walk from the hotel to Cowboys downtown. If the weather is nice, we\u{2019}ll \
                      head across the street to Gasoline Alley later in the night.",
    },
];

#[component]
fn Section(title: &'static str, subtitle: &'static str, children: Children) -> impl IntoView {
    view! {
        <section class="mb-12">
            <h2 class="font-serif text-2xl text-charcoal mb-1">{title}</h2>
            <p class="text-xs text-gold tracking-[0.2em] uppercase font-sans mb-4">{subtitle}</p>
            <div class="text-charcoal/60 font-sans text-sm leading-relaxed space-y-3">
                {children()}
            </div>
        </section>
    }
}

#[component]
pub fn Information() -> impl IntoView {
    view! {
        <div class="max-w-2xl mx-auto px-6 py-16">
            <h1 class="font-serif text-4xl text-charcoal mb-2 text-center">"Wedding Information"</h1>
            <p class="text-center text-charcoal/50 text-sm tracking-widest font-sans mb-14">
                "November 20\u{2013}21, 2026  ·  Colorado Springs, Colorado"
            </p>

            <Section title="Rehearsal & Welcome" subtitle="Friday · November 20">
                <p>
                    "The wedding rehearsal begins at 6:30 PM at Corpus Christi Catholic Church."
                </p>
                <p>
                    "Dinner follows the rehearsal at "
                    <a
                        href="https://mackenzieschophouse.com"
                        target="_blank"
                        rel="noopener"
                        class="text-sage underline hover:text-earth transition-colors"
                    >
                        "MacKenzie\u{2019}s Chop House"
                    </a>
                    ". The dinner is reserved for members of the wedding party and family."
                </p>
                <p>
                    "Later that evening, an informal welcome party will be held at COATI in "
                    "downtown Colorado Springs. It\u{2019}s a great chance for guests to meet and "
                    "mingle."
                </p>
            </Section>

            <Section title="Nuptial Mass" subtitle="Saturday · November 21">
                <p>
                    "The Nuptial Mass begins at 1:30 PM at Corpus Christi Catholic Church."
                </p>
            </Section>

            <Section title="Post-Mass Bar Crawl" subtitle="Old Colorado City">
                <p>
                    "Between the Mass and the reception, join us for a casual bar crawl through "
                    "Old Colorado City:"
                </p>
                <ul class="list-disc list-inside space-y-1">
                    <li>"OCC Brewing"</li>
                    <li>"Mother Muff\u{2019}s"</li>
                </ul>
                <p>
                    "See the "
                    <a href="/bar-crawl" class="text-sage underline hover:text-earth transition-colors">
                        "bar crawl page"
                    </a>
                    " for addresses and the walking route."
                </p>
            </Section>

            <Section title="Reception" subtitle="Red Rocks Barn">
                <p>
                    "The reception at Red Rocks Barn starts at 4:45 PM and ends at 9:00 PM. "
                    "Shuttle buses pick up from the bar crawl at 4:30 PM and run guests back "
                    "to the hotel block at the end of the night."
                </p>
            </Section>

            <Section title="After Party" subtitle="Downtown Colorado Springs">
                <p>
                    "Keep the celebration going downtown! Once the shuttle drops you back at "
                    "the hotel, walk up Tejon Street to "
                    <a
                        href="https://www.google.com/maps/search/?api=1&query=25+N+Tejon+St+Colorado+Springs+CO+80903"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-sage underline hover:text-earth transition-colors"
                    >
                        "Cowboys (25 N Tejon St)"
                    </a>
                    " \u{2014} about a 10-minute walk from SpringHill Suites and even closer "
                    "from the Hyatt Place."
                </p>
                <p>
                    "After some time at Cowboys, if the weather is nice, we\u{2019}ll head across "
                    "the street to "
                    <a
                        href="https://www.google.com/maps/search/?api=1&query=28+N+Tejon+St+Colorado+Springs+CO+80903"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-sage underline hover:text-earth transition-colors"
                    >
                        "Gasoline Alley (28 N Tejon St)"
                    </a>
                    "."
                </p>
            </Section>

            <Section title="Getting Around" subtitle="Shuttle Service">
                <p>
                    "Shuttle buses will take guests to every stop of the day, starting and ending "
                    "at the SpringHill Suites hotel block. Our suggested plan for the day:"
                </p>
                <ol class="list-decimal list-inside space-y-1">
                    <li>"Take the 12:45 PM shuttle from SpringHill Suites to the Mass"</li>
                    <li>"Ride the shuttle from the church to the bar crawl after the Mass"</li>
                    <li>"Walk between the bars in Old Colorado City"</li>
                    <li>"Catch the 4:30 PM shuttle on S 25th St to the reception"</li>
                    <li>"Take the shuttle back to the hotel block after the reception"</li>
                    <li>"Walk up Tejon St to Cowboys for the after party"</li>
                </ol>
                <p>
                    "See the "
                    <a href="/shuttle" class="text-sage underline hover:text-earth transition-colors">
                        "shuttle schedule"
                    </a>
                    " for exact pick-up times and addresses."
                </p>
            </Section>

            // ── Day-of schedule ────────────────────────────────────────────────
            <section class="mt-16">
                <h2 class="font-serif text-2xl text-charcoal mb-1 text-center">
                    "Day-of Schedule"
                </h2>
                <p class="text-center text-charcoal/50 text-xs tracking-widest font-sans mb-10">
                    "November 21, 2026"
                </p>

                <ol class="relative border-l border-gold/40 ml-4">
                    {EVENTS.iter().map(|event| view! {
                        <li class="mb-10 ml-8">
                            // Dot
                            <span class="absolute -left-[9px] w-[17px] h-[17px] rounded-full
                                         bg-champagne border-2 border-gold mt-1"></span>

                            // Time
                            <p class="text-xs text-gold tracking-[0.2em] uppercase font-sans mb-1">
                                {event.time}
                            </p>

                            // Title
                            <h3 class="font-serif text-xl text-charcoal mb-1">{event.title}</h3>

                            // Location
                            <h3 class="text-charcoal/60 font-sans text-sm leading-relaxed">{event.location}</h3>

                            // Description
                            <p class="text-charcoal/60 font-sans text-sm leading-relaxed">
                                {event.description}
                            </p>
                        </li>
                    }).collect_view()}
                </ol>
            </section>
        </div>
    }
}
