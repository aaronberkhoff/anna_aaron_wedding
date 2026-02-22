use crate::api::client;
use leptos::prelude::*;
use shared::models::{
    guest::DietaryRestriction,
    rsvp::{RsvpRequest, RsvpResponse},
};

// ── RSVP open/closed toggle ───────────────────────────────────────────────────
// Set to `true` when you are ready to accept RSVPs.
const RSVP_OPEN: bool = false;

#[component]
pub fn Rsvp() -> impl IntoView {
    if !RSVP_OPEN {
        return view! {
            <div class="max-w-lg mx-auto px-6 py-20 text-center">
                <h1 class="font-serif text-4xl text-charcoal mb-6">"RSVP"</h1>
                <div class="flex items-center gap-4 justify-center mb-8 w-48 mx-auto">
                    <div class="flex-1 h-px bg-gold/40"></div>
                    <svg class="w-2 h-2 fill-gold opacity-60" viewBox="0 0 20 20">
                        <path d="M10 0 L12.5 7.5 L20 10 L12.5 12.5 L10 20 L7.5 12.5 L0 10 L7.5 7.5 Z" />
                    </svg>
                    <div class="flex-1 h-px bg-gold/40"></div>
                </div>
                <p class="font-serif italic text-xl text-charcoal/70 mb-3">
                    "RSVPs open soon!"
                </p>
            </div>
        }.into_any();
    }
    let (first_name, set_first_name) = signal(String::new());
    let (last_name, set_last_name) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (attending, set_attending) = signal(true);
    let (result, set_result) = signal(Option::<RsvpResponse>::None);
    let (error, set_error) = signal(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let payload = RsvpRequest {
            first_name: first_name.get(),
            last_name: last_name.get(),
            email: email.get(),
            attending: attending.get(),
            plus_one: false,
            plus_one_name: None,
            dietary_restriction: DietaryRestriction::None,
            song_request: None,
            message: None,
        };

        leptos::task::spawn_local(async move {
            match client::post::<_, RsvpResponse>(shared::api::routes::RSVP_SUBMIT, &payload).await
            {
                Ok(resp) => {
                    set_result.set(Some(resp));
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    };

    view! {
        <div class="max-w-lg mx-auto px-6 py-12">

            <h1 class="font-serif text-4xl text-charcoal mb-8 text-center">"RSVP"</h1>

            {move || result.get().map(|r| view! {
                <div class="bg-sage/20 border border-sage rounded p-4 mb-6 text-charcoal">
                    {r.message}
                </div>
            })}

            {move || error.get().map(|e| view! {
                <div class="bg-red-100 border border-red-400 rounded p-4 mb-6 text-red-700">
                    "Error: " {e}
                </div>
            })}

            <form on:submit=on_submit class="flex flex-col gap-4">
                <input
                    type="text"
                    placeholder="First name"
                    class="border border-gold rounded px-4 py-2 bg-ivory"
                    on:input=move |ev| set_first_name.set(event_target_value(&ev))
                />
                <input
                    type="text"
                    placeholder="Last name"
                    class="border border-gold rounded px-4 py-2 bg-ivory"
                    on:input=move |ev| set_last_name.set(event_target_value(&ev))
                />
                <input
                    type="email"
                    placeholder="Email address"
                    class="border border-gold rounded px-4 py-2 bg-ivory"
                    on:input=move |ev| set_email.set(event_target_value(&ev))
                />
                <label class="flex items-center gap-2 text-charcoal">
                    <input
                        type="checkbox"
                        checked=attending
                        on:change=move |ev| set_attending.set(event_target_checked(&ev))
                    />
                    "I will be attending"
                </label>
                <button
                    type="submit"
                    class="bg-gold text-ivory font-semibold px-6 py-3 rounded hover:bg-charcoal transition-colors"
                >
                    "Submit RSVP"
                </button>
            </form>
        </div>
    }.into_any()
}
