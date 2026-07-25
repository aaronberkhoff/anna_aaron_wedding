use leptos::prelude::*;

struct AttireExample {
    src: &'static str,
    caption: &'static str,
}

const WEAR_EXAMPLES: &[AttireExample] = &[
    AttireExample {
        src: "/attire/wear-1.jpg",
        caption: "Boat neck \u{2014} chest fully covered",
    },
    AttireExample {
        src: "/attire/wear-2.jpg",
        caption: "High neckline, shoulders covered",
    },
    AttireExample {
        src: "/attire/wear-3.jpg",
        caption: "Halter with a secure, modest bodice",
    },
];

const AVOID_EXAMPLES: &[AttireExample] = &[
    AttireExample {
        src: "/attire/avoid-1.jpg",
        caption: "Strapless-style, neckline sits too low",
    },
    AttireExample {
        src: "/attire/avoid-2.jpg",
        caption: "Plunging neckline, too much exposed",
    },
    AttireExample {
        src: "/attire/avoid-3.jpg",
        caption: "Tube-top cut \u{2014} bust too showy",
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
fn AttireGrid(examples: &'static [AttireExample]) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-6">
            {examples.iter().map(|example| view! {
                <figure class="flex flex-col gap-2">
                    <img
                        src=example.src
                        alt=example.caption
                        class="w-full aspect-[3/4] object-cover rounded-lg border border-gold/30 bg-ivory"
                    />
                    <figcaption class="text-sm text-charcoal/60 text-center">
                        {example.caption}
                    </figcaption>
                </figure>
            }).collect_view()}
        </div>
    }
}

#[component]
fn AttireToggle() -> impl IntoView {
    let (showing_wear, set_showing_wear) = signal(true);

    let tab_class = move |is_wear: bool| {
        if showing_wear.get() == is_wear {
            "px-6 py-2 rounded-full font-sans text-xs tracking-[0.15em] uppercase \
             bg-gold text-ivory transition-colors"
        } else {
            "px-6 py-2 rounded-full font-sans text-xs tracking-[0.15em] uppercase \
             text-charcoal/50 border border-gold/30 hover:text-charcoal transition-colors"
        }
    };

    view! {
        <div class="flex justify-center gap-3 mb-8">
            <button
                type="button"
                class=move || tab_class(true)
                on:click=move |_| set_showing_wear.set(true)
            >
                "Wear This"
            </button>
            <button
                type="button"
                class=move || tab_class(false)
                on:click=move |_| set_showing_wear.set(false)
            >
                "Avoid This"
            </button>
        </div>
        {move || if showing_wear.get() {
            view! { <AttireGrid examples=WEAR_EXAMPLES /> }
        } else {
            view! { <AttireGrid examples=AVOID_EXAMPLES /> }
        }}
    }
}

#[component]
pub fn Faq() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto px-6 py-16">
            <h1 class="font-serif text-4xl text-charcoal mb-2 text-center">"Frequently Asked Questions"</h1>
            <p class="text-center text-charcoal/50 text-sm tracking-widest font-sans mb-14">
                "Attire & Getting Around"
            </p>

            <Section title="What Should I Wear?" subtitle="Cocktail Attire">
                <p>
                    "We're asking guests to dress in attire appropriate for a Catholic wedding. "
                    "The Mass is a sacred space for us \u{2014} we believe Christ is truly present "
                    "in the Eucharist \u{2014} and we want the atmosphere to feel reverent."
                </p>

                <h3 class="font-serif text-xl text-charcoal mt-8 mb-2">"Women"</h3>
                <p>
                    "Long dresses or skirts are encouraged, with a modest neckline \u{2014} "
                    "nothing strapless, low-cut, or bust-baring. Color and style are entirely up "
                    "to you \u{2014} the examples below are just about how much is covered up top."
                </p>
                <div class="mt-6">
                    <AttireToggle />
                </div>

                <h3 class="font-serif text-xl text-charcoal mt-10 mb-2">"Men"</h3>
                <p>
                    "A long-sleeve dress shirt with a collar (no polos) and slacks with a belt "
                    "(no jeans). A jacket and/or tie is optional, but always welcome."
                </p>
            </Section>

            <Section title="Parking" subtitle="At the Church & Venue">
                <p>
                    "Corpus Christi Catholic Church has an on-site parking lot with additional "
                    "street parking nearby. Red Rocks Barn has a dedicated gravel lot for guests "
                    "\u{2014} no permit or pass required."
                </p>
            </Section>

            <Section title="Getting Around" subtitle="Our Recommendation">
                <p>
                    "Our top recommendation is to grab a rental car and split it with a group of "
                    "friends for the weekend \u{2014} Uber and Lyft can get surprisingly expensive "
                    "in Colorado Springs, especially during busy hours, so a shared rental usually "
                    "ends up being the easiest and most cost-effective way to get around."
                </p>
                <p>
                    "Rideshare is still a completely fine option though. Uber and Lyft both "
                    "operate reliably throughout downtown and Old Colorado City, and you can get "
                    "through the whole weekend without a rental car if you'd rather not drive."
                </p>
            </Section>

            <Section title="Getting to Colorado Springs" subtitle="Airport & Rental Car">
                <p>
                    "The Colorado Springs Airport (COS) is about 20 minutes from downtown and "
                    "the venue. Denver International Airport (DEN) is roughly 1.5\u{2013}2 hours "
                    "away and typically has more flight options."
                </p>
                <p>
                    "If you're flying in, we'd suggest coordinating with other guests and renting "
                    "a car together for the weekend \u{2014} it's the best option for getting "
                    "between the hotels, the Mass, the bar crawl, and the reception."
                </p>
            </Section>
        </div>
    }
}
