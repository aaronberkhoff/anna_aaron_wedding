use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="bg-champagne/90 backdrop-blur-sm border-b border-gold/25
                    px-8 py-4 flex items-center justify-between sticky top-0 z-50">
            <a href="/" class="font-script text-3xl text-charcoal hover:text-earth transition-colors">
                "Anna & Aaron"
            </a>
            <ul class="flex gap-8 text-xs font-sans tracking-[0.15em] uppercase text-charcoal/60">
                <li><A href="/">"Home"</A></li>
                <li><A href="/rsvp">"RSVP"</A></li>
                <li><A href="/seating">"Seating"</A></li>
                <li><A href="/hotel">"Hotel"</A></li>
                <li><A href="/itinerary">"Itinerary"</A></li>
                <li><A href="/gallery">"Gallery"</A></li>
            </ul>
        </nav>
    }
}
