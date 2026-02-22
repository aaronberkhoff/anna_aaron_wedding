use leptos::prelude::*;

struct Event {
    time: &'static str,
    title: &'static str,
    description: &'static str,
    location: &'static str,
}

// ── Edit these placeholders with your actual schedule ─────────────────────────
const EVENTS: &[Event] = &[
    Event {
        time: "1:30 PM",
        title: "Ceremony",
        location: "Corpus Christi Catholic Church",
        description: "Wedding Mass",
    },
    Event {
        time: "2:45 PM",
        title: "Break",
        location: "TBD",
        description: "Break for wedding party pictures. Guest activities TBD",
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
        description: "Exit from the venue",
        location: "Red Rocks Barn",
    },
    Event {
        time: "9:30 PM",
        title: "After Party",
        location: "Red Rocks Barn",
        description: "Join us for an after party downtown",
    },
];

#[component]
pub fn Itinerary() -> impl IntoView {
    view! {
        <div class="max-w-2xl mx-auto px-6 py-16">
            <h1 class="font-serif text-4xl text-charcoal mb-2 text-center">"Day-Of Schedule (Subject to Change)"</h1>
            <p class="text-center text-charcoal/50 text-sm tracking-widest font-sans mb-14">
                "November 21, 2026  ·  Red Rocks Barn"
            </p>

            // Timeline
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
        </div>
    }
}
