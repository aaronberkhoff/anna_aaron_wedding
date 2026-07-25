use crate::components::footer::Footer;
use crate::components::nav::Nav;
use crate::pages::{
    dj_spike::DjSpike, gallery::Gallery, home::Home, hotel::Hotel, information::Information,
    rsvp::Rsvp,
};
use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::{
    components::{Redirect, Route, Router, Routes},
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
                    <Route path=path!("/hotel")    view=Hotel   />
                    <Route path=path!("/information") view=Information />
                    // Old link — keep it working for anyone who bookmarked it
                    <Route path=path!("/itinerary") view=|| view! { <Redirect path="/information" /> } />
                    <Route path=path!("/gallery")  view=Gallery />
                    // Phase 0 spike — throwaway, removed in Phase 1.
                    <Route path=path!("/dj-spike") view=DjSpike />
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
