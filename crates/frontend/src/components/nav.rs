use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="bg-champagne/90 backdrop-blur-sm border-b border-gold/25
                    px-8 py-4 flex items-center justify-between gap-6 sticky top-0 z-50">
            <a href="/" class="font-script text-3xl whitespace-nowrap text-charcoal hover:text-earth transition-colors">
                "Anna & Aaron"
            </a>
            <ul class="flex flex-wrap justify-end gap-x-8 gap-y-2 text-xs font-sans tracking-[0.15em] uppercase text-charcoal/60">
                <li><A href="/">"Home"</A></li>
                <li><A href="/rsvp">"RSVP"</A></li>
                <li><A href="/hotel">"Hotel"</A></li>
                <li><A href="/shuttle">"Shuttle"</A></li>
                <li><A href="/bar-crawl">"Bar Crawl"</A></li>
                <li><A href="/information">"Information"</A></li>
                <li><A href="/gallery">"Gallery"</A></li>
                <li><A href="/registry">"Registry"</A></li>
                <li><A href="/faq">"FAQ"</A></li>
            </ul>
        </nav>
    }
}
