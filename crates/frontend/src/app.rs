use crate::components::footer::Footer;
use crate::components::nav::Nav;
use crate::pages::{
    gallery::Gallery, home::Home, hotel::Hotel, itinerary::Itinerary, rsvp::Rsvp,
    seating::Seating,
};
use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <Nav />
            <main class="min-h-screen">
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=path!("/")         view=Home    />
                    <Route path=path!("/rsvp")     view=Rsvp    />
                    <Route path=path!("/seating")  view=Seating />
                    <Route path=path!("/hotel")    view=Hotel   />
                    <Route path=path!("/itinerary") view=Itinerary />
                    <Route path=path!("/gallery")  view=Gallery />
                </Routes>
            </main>
            <Footer />
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-screen">
            <h1 class="text-4xl font-serif text-charcoal">"Page not found"</h1>
            <a href="/" class="mt-4 text-sage hover:underline">"Return home"</a>
        </div>
    }
}
