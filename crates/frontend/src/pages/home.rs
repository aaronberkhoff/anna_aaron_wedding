use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div class="max-w-2xl mx-auto px-6 py-16 text-center">
            <h1 class="font-serif text-5xl text-charcoal mb-4">
                "Anna & Aaron"
            </h1>
            <p class="text-gold text-xl mb-8">"Are getting married!"</p>
            <p class="text-charcoal text-lg leading-relaxed">
                "We are so excited to celebrate this special day with our family and friends. "
                "Please use the navigation above to RSVP, find your seat, and explore hotel options."
            </p>
            // TODO: Add wedding date, venue, countdown timer
        </div>
    }
}
