use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="bg-champagne border-b border-gold px-6 py-4 flex items-center justify-between">
            <a href="/" class="font-serif text-2xl text-charcoal tracking-wide">
                "Anna & Aaron"
            </a>
            <ul class="flex gap-6 text-sm font-sans text-charcoal">
                <li><A href="/">"Home"</A></li>
                <li><A href="/rsvp">"RSVP"</A></li>
                <li><A href="/seating">"Seating"</A></li>
                <li><A href="/hotel">"Hotel"</A></li>
                <li><A href="/gallery">"Gallery"</A></li>
            </ul>
        </nav>
    }
}
