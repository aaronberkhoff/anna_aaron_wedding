use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="bg-charcoal text-ivory/50 text-center py-10">
            <p class="font-script text-2xl text-ivory/80 mb-2">"Anna & Aaron"</p>
            <p class="text-xs tracking-widest font-sans">
                "November 21, 2026  ·  Corpus Christi Catholic Church  ·  Colorado Springs, CO"
            </p>
        </footer>
    }
}
