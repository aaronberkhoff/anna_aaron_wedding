use crate::api::client;
use leptos::prelude::*;
use shared::{
    api::routes,
    models::{
        guest::{GuestLookup, GuestSearchResult},
        rsvp::{GuestRsvp, RsvpRequest, RsvpResponse},
    },
};

// (guest_id, attending_reception, attending_rehearsal)
type PartyStateVec = RwSignal<Vec<(String, bool, Option<bool>)>>;

const RSVP_OPEN: bool = true;

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
                <p class="font-serif italic text-xl text-charcoal/70">"RSVPs open soon!"</p>
            </div>
        }
        .into_any();
    }

    let (step, set_step) = signal(0u8);

    // ── Step 1: Lookup state ──────────────────────────────────────────────────
    let (code_input, set_code_input) = signal(String::new());
    let (name_input, set_name_input) = signal(String::new());
    let (search_results, set_search_results) = signal(Vec::<GuestSearchResult>::new());
    let (selected_id, set_selected_id) = signal(String::new());
    let (looking_up, set_looking_up) = signal(false);
    let (searching, set_searching) = signal(false);
    let (lookup_error, set_lookup_error) = signal(Option::<String>::None);
    let (rsvp_exists, set_rsvp_exists) = signal(false);

    // ── Shared guest data ─────────────────────────────────────────────────────
    let (guest_data, set_guest_data) = signal(Option::<GuestLookup>::None);

    // ── Step 2: Form state ────────────────────────────────────────────────────
    // One entry per party member: (guest_id, attending_reception, attending_rehearsal)
    let party_states: PartyStateVec = RwSignal::new(vec![]);
    let selected_known_guests: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let (kg_query, set_kg_query) = signal(String::new());
    let (kg_results, set_kg_results) = signal(Vec::<GuestSearchResult>::new());
    let (message, set_message) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal(Option::<String>::None);

    // ── Step 3: Confirmation ──────────────────────────────────────────────────
    let (confirm_msg, set_confirm_msg) = signal(String::new());

    // ── Lookup helper ─────────────────────────────────────────────────────────
    let do_lookup = move |query_param: String| {
        set_looking_up.set(true);
        set_lookup_error.set(None);
        leptos::task::spawn_local(async move {
            let url = format!("{}?{}", routes::GUEST_LOOKUP, query_param);
            match client::get::<GuestLookup>(&url).await {
                Ok(data) => {
                    // Initialise party state: default reception=true, rehearsal=Some(true) if invited.
                    let initial: Vec<(String, bool, Option<bool>)> = data
                        .members
                        .iter()
                        .map(|m| {
                            let reh = if m.rehearsal_invited {
                                Some(true)
                            } else {
                                None
                            };
                            (m.id.clone(), true, reh)
                        })
                        .collect();
                    party_states.set(initial);

                    let already_rsvped = data
                        .members
                        .first()
                        .map(|m| m.rsvp_status != "pending")
                        .unwrap_or(false);
                    set_guest_data.set(Some(data));
                    if already_rsvped {
                        set_rsvp_exists.set(true);
                    } else {
                        set_step.set(1);
                    }
                }
                Err(e) => {
                    let msg = if e.contains("404") || e.contains("HTTP 404") {
                        "Invite code not found. Please check your code or search by name."
                            .to_string()
                    } else {
                        format!("Lookup failed: {e}")
                    };
                    set_lookup_error.set(Some(msg));
                }
            }
            set_looking_up.set(false);
        });
    };

    let do_lookup_by_code = move |_| {
        let code = code_input.get().trim().to_string();
        if code.is_empty() {
            set_lookup_error.set(Some("Please enter your 4-digit invite code.".to_string()));
            return;
        }
        do_lookup(format!("code={code}"));
    };

    let do_lookup_by_id = move |_| {
        let id = selected_id.get().trim().to_string();
        if id.is_empty() {
            set_lookup_error.set(Some(
                "Please select a guest from the search results.".to_string(),
            ));
            return;
        }
        do_lookup(format!("id={id}"));
    };

    let do_search = move |_| {
        let q = name_input.get().trim().to_string();
        if q.len() < 2 {
            set_lookup_error.set(Some("Please enter at least 2 characters.".to_string()));
            return;
        }
        set_searching.set(true);
        set_lookup_error.set(None);
        leptos::task::spawn_local(async move {
            let url = format!("{}?q={}", routes::GUEST_SEARCH, q);
            match client::get::<Vec<GuestSearchResult>>(&url).await {
                Ok(results) => {
                    if results.is_empty() {
                        set_lookup_error.set(Some(
                            "No guests found. Please check the spelling or try your invite code."
                                .to_string(),
                        ));
                    } else {
                        set_selected_id.set(results[0].id.clone());
                    }
                    set_search_results.set(results);
                }
                Err(e) => set_lookup_error.set(Some(format!("Search failed: {e}"))),
            }
            set_searching.set(false);
        });
    };

    let do_kg_search = move |q: String| {
        if q.len() < 2 {
            set_kg_results.set(vec![]);
            return;
        }
        leptos::task::spawn_local(async move {
            let url = format!("{}?q={}", routes::GUEST_SEARCH, q);
            if let Ok(results) = client::get::<Vec<GuestSearchResult>>(&url).await {
                set_kg_results.set(results);
            }
        });
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Some(data) = guest_data.get() else { return };

        let party: Vec<GuestRsvp> = party_states
            .get()
            .into_iter()
            .zip(data.members.iter())
            .map(|((id, att_rec, att_reh), member)| GuestRsvp {
                guest_id: id,
                name: member.full_name(),
                attending_reception: att_rec,
                attending_rehearsal: att_reh,
            })
            .collect();

        let payload = RsvpRequest {
            party,
            known_guests: selected_known_guests.get(),
            song_request: None,
            message: {
                let m = message.get();
                if m.is_empty() {
                    None
                } else {
                    Some(m)
                }
            },
        };

        set_submitting.set(true);
        set_submit_error.set(None);
        leptos::task::spawn_local(async move {
            match client::post::<_, RsvpResponse>(routes::RSVP_SUBMIT, &payload).await {
                Ok(resp) => {
                    set_confirm_msg.set(resp.message);
                    set_step.set(2);
                }
                Err(e) => set_submit_error.set(Some(e)),
            }
            set_submitting.set(false);
        });
    };

    view! {
        <div class="max-w-2xl mx-auto px-6 py-12">
            <h1 class="font-serif text-4xl text-charcoal mb-4 text-center">"RSVP"</h1>
            <div class="flex items-center gap-4 justify-center mb-8 w-48 mx-auto">
                <div class="flex-1 h-px bg-gold/40"></div>
                <svg class="w-2 h-2 fill-gold opacity-60" viewBox="0 0 20 20">
                    <path d="M10 0 L12.5 7.5 L20 10 L12.5 12.5 L10 20 L7.5 12.5 L0 10 L7.5 7.5 Z" />
                </svg>
                <div class="flex-1 h-px bg-gold/40"></div>
            </div>

            // ── Step 1: Lookup ────────────────────────────────────────────────
            {move || (step.get() == 0).then(|| {
                if rsvp_exists.get() {
                    let guest_name = guest_data.get()
                        .and_then(|d| d.members.into_iter().next())
                        .map(|m| m.full_name())
                        .unwrap_or_default();
                    view! {
                        <div class="bg-amber-50 border border-amber-300 rounded-lg p-6 flex flex-col gap-4 text-center">
                            <p class="font-serif text-xl text-charcoal">
                                "Welcome back, " {guest_name} "!"
                            </p>
                            <p class="text-charcoal/70 text-sm">
                                "We already have an RSVP on file for you."
                                <br />
                                "Would you like to update it?"
                            </p>
                            <div class="flex gap-3 justify-center flex-wrap">
                                <button
                                    type="button"
                                    class="bg-gold text-ivory font-semibold px-5 py-2 rounded hover:bg-charcoal transition-colors text-sm"
                                    on:click=move |_| {
                                        set_rsvp_exists.set(false);
                                        set_step.set(1);
                                    }
                                >
                                    "Yes, update my RSVP"
                                </button>
                                <button
                                    type="button"
                                    class="border border-charcoal/30 text-charcoal/60 px-5 py-2 rounded hover:bg-charcoal/5 transition-colors text-sm"
                                    on:click=move |_| {
                                        set_rsvp_exists.set(false);
                                        set_guest_data.set(None);
                                        set_lookup_error.set(None);
                                        set_search_results.set(vec![]);
                                    }
                                >
                                    "No, keep my existing RSVP"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex flex-col gap-6">
                            <p class="text-center text-charcoal/70">
                                "Enter your 4-digit invite code to RSVP."
                            </p>

                            <div class="flex gap-2">
                                <input
                                    type="text"
                                    maxlength="4"
                                    placeholder="Invite code (e.g. 4821)"
                                    class="flex-1 border border-gold rounded px-4 py-2 bg-ivory text-charcoal tracking-widest text-center text-lg"
                                    on:input=move |ev| set_code_input.set(event_target_value(&ev))
                                />
                                <button
                                    class="bg-gold text-ivory font-semibold px-5 py-2 rounded hover:bg-charcoal transition-colors"
                                    on:click=do_lookup_by_code
                                    disabled=looking_up
                                >
                                    {move || if looking_up.get() { "Looking up…" } else { "Find Invite" }}
                                </button>
                            </div>

                            <div class="flex items-center gap-3 text-charcoal/40 text-sm">
                                <div class="flex-1 h-px bg-charcoal/20"></div>
                                "or search by name"
                                <div class="flex-1 h-px bg-charcoal/20"></div>
                            </div>

                            <div class="flex gap-2">
                                <input
                                    type="text"
                                    placeholder="Your full name"
                                    class="flex-1 border border-gold rounded px-4 py-2 bg-ivory text-charcoal"
                                    on:input=move |ev| set_name_input.set(event_target_value(&ev))
                                />
                                <button
                                    class="bg-gold text-ivory font-semibold px-5 py-2 rounded hover:bg-charcoal transition-colors"
                                    on:click=do_search
                                    disabled=searching
                                >
                                    {move || if searching.get() { "Searching…" } else { "Search" }}
                                </button>
                            </div>

                            {move || {
                                let results = search_results.get();
                                (!results.is_empty()).then(|| {
                                    let options = results.clone();
                                    view! {
                                        <div class="flex gap-2">
                                            <select
                                                class="flex-1 border border-gold rounded px-4 py-2 bg-ivory text-charcoal"
                                                on:change=move |ev| set_selected_id.set(event_target_value(&ev))
                                            >
                                                {options.into_iter().map(|r| {
                                                    let id = r.id.clone();
                                                    view! { <option value=id>{r.full_name}</option> }
                                                }).collect_view()}
                                            </select>
                                            <button
                                                class="bg-gold text-ivory font-semibold px-5 py-2 rounded hover:bg-charcoal transition-colors"
                                                on:click=do_lookup_by_id
                                                disabled=looking_up
                                            >
                                                "Select"
                                            </button>
                                        </div>
                                    }
                                })
                            }}

                            {move || lookup_error.get().map(|e| view! {
                                <p class="text-red-600 text-sm text-center">{e}</p>
                            })}
                        </div>
                    }.into_any()
                }
            })}

            // ── Step 2: RSVP Form ─────────────────────────────────────────────
            {move || (step.get() == 1).then(|| {
                let Some(data) = guest_data.get() else {
                    return view! { <p>"Loading…"</p> }.into_any();
                };
                let welcome_name = data.members.first().map(|m| m.full_name()).unwrap_or_default();
                let any_rehearsal = data.members.iter().any(|m| m.rehearsal_invited);
                let members_snap = data.members.clone();

                view! {
                    <form on:submit=on_submit class="flex flex-col gap-8">
                        <h2 class="font-serif text-2xl text-charcoal text-center">
                            "Welcome, " {welcome_name} "!"
                        </h2>

                        // ── Party attendance ──────────────────────────────────
                        <section class="flex flex-col gap-4">
                            <h3 class="font-semibold text-charcoal uppercase tracking-wide text-sm border-b border-gold/40 pb-1">
                                "Wedding Reception — November 21, 2026"
                            </h3>
                            {members_snap.iter().enumerate().map(|(i, member)| {
                                let name = member.full_name();
                                view! {
                                    <MemberRow
                                        name=name
                                        index=i
                                        party_states=party_states
                                        show_reception=true
                                        show_rehearsal=false
                                    />
                                }
                            }).collect_view()}
                        </section>

                        // ── Rehearsal dinner (if any member invited) ──────────
                        {any_rehearsal.then(|| {
                            let members_reh = data.members.clone();
                            view! {
                                <section class="flex flex-col gap-4">
                                    <h3 class="font-semibold text-charcoal uppercase tracking-wide text-sm border-b border-gold/40 pb-1">
                                        "Rehearsal Dinner — November 19, 2026"
                                    </h3>
                                    {members_reh.iter().enumerate().map(|(i, member)| {
                                        let name = member.full_name();
                                        view! {
                                            <MemberRow
                                                name=name
                                                index=i
                                                party_states=party_states
                                                show_reception=false
                                                show_rehearsal=true
                                            />
                                        }
                                    }).collect_view()}
                                </section>
                            }
                        })}

                        // ── Seating preference ────────────────────────────────
                        <section class="flex flex-col gap-2">
                            <label class="text-xs font-semibold uppercase tracking-wide text-charcoal/60">
                                "Who would you like to be seated near? (optional)"
                            </label>
                            <input
                                type="text"
                                placeholder="Search for a guest…"
                                class="border border-gold rounded px-4 py-2 bg-ivory text-charcoal text-sm"
                                prop:value=move || kg_query.get()
                                on:input=move |ev| {
                                    let q = event_target_value(&ev);
                                    set_kg_query.set(q.clone());
                                    do_kg_search(q);
                                }
                            />
                            {move || {
                                let results = kg_results.get();
                                let selected = selected_known_guests.get();
                                let unselected: Vec<_> = results.into_iter()
                                    .filter(|r| !selected.contains(&r.full_name))
                                    .collect();
                                (!unselected.is_empty()).then(|| view! {
                                    <div class="border border-gold/50 rounded bg-white shadow-sm divide-y divide-gold/20">
                                        {unselected.into_iter().map(|r| {
                                            let name = r.full_name.clone();
                                            view! {
                                                <button
                                                    type="button"
                                                    class="w-full text-left px-4 py-2 text-sm text-charcoal hover:bg-gold/10 transition-colors"
                                                    on:click=move |_| {
                                                        selected_known_guests.update(|v| {
                                                            if !v.contains(&name) { v.push(name.clone()); }
                                                        });
                                                        set_kg_results.set(vec![]);
                                                        set_kg_query.set(String::new());
                                                    }
                                                >
                                                    {r.full_name}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                })
                            }}
                            {move || {
                                let names = selected_known_guests.get();
                                (!names.is_empty()).then(|| view! {
                                    <div class="flex flex-wrap gap-2 mt-1">
                                        {names.into_iter().map(|name| {
                                            let name_remove = name.clone();
                                            view! {
                                                <span class="inline-flex items-center gap-1 bg-gold/20 text-charcoal text-xs px-3 py-1 rounded-full">
                                                    {name}
                                                    <button
                                                        type="button"
                                                        class="text-charcoal/50 hover:text-charcoal leading-none ml-1"
                                                        on:click=move |_| selected_known_guests.update(|v| v.retain(|n| n != &name_remove))
                                                    >
                                                        "×"
                                                    </button>
                                                </span>
                                            }
                                        }).collect_view()}
                                    </div>
                                })
                            }}
                        </section>

                        // ── Message ───────────────────────────────────────────
                        <section>
                            <textarea
                                placeholder="Message to the couple (optional)"
                                rows="3"
                                class="w-full border border-gold rounded px-4 py-2 bg-ivory text-charcoal text-sm resize-none"
                                on:input=move |ev| set_message.set(event_target_value(&ev))
                            ></textarea>
                        </section>

                        {move || submit_error.get().map(|e| view! {
                            <p class="text-red-600 text-sm text-center">{e}</p>
                        })}

                        <button
                            type="submit"
                            class="bg-gold text-ivory font-semibold px-6 py-3 rounded hover:bg-charcoal transition-colors"
                            disabled=submitting
                        >
                            {move || if submitting.get() { "Submitting…" } else { "Submit RSVP" }}
                        </button>
                    </form>
                }.into_any()
            })}

            // ── Step 3: Confirmation ──────────────────────────────────────────
            {move || (step.get() == 2).then(|| view! {
                <div class="text-center flex flex-col gap-4">
                    <div class="bg-sage/20 border border-sage rounded-lg p-8">
                        <p class="font-serif text-2xl text-charcoal mb-2">{confirm_msg.get()}</p>
                        <p class="text-charcoal/70 text-sm">
                            "We cannot wait to celebrate with you. See you in November!"
                        </p>
                    </div>
                    <button
                        class="text-sage underline text-sm"
                        on:click=move |_| {
                            set_step.set(0);
                            set_guest_data.set(None);
                            set_lookup_error.set(None);
                            set_search_results.set(vec![]);
                        }
                    >
                        "RSVP for another guest"
                    </button>
                </div>
            })}
        </div>
    }
    .into_any()
}

// ── MemberRow: one attendance card per guest ───────────────────────────────────

#[component]
fn MemberRow(
    name: String,
    index: usize,
    party_states: PartyStateVec,
    show_reception: bool,
    show_rehearsal: bool,
) -> impl IntoView {
    view! {
        <div class="border border-gold/30 rounded-lg bg-ivory p-4 flex flex-col gap-3">
            <p class="font-semibold text-charcoal">{name}</p>

            {show_reception.then(|| view! {
                <AttendanceToggle
                    label="Attending reception?"
                    value=Signal::derive(move || {
                        party_states.get().get(index).map(|s| s.1).unwrap_or(true)
                    })
                    on_change=move |v| party_states.update(|s| {
                        if let Some(row) = s.get_mut(index) { row.1 = v; }
                    })
                />
            })}

            {show_rehearsal.then(|| view! {
                <AttendanceToggle
                    label="Attending rehearsal dinner?"
                    value=Signal::derive(move || {
                        party_states.get().get(index).and_then(|s| s.2).unwrap_or(false)
                    })
                    on_change=move |v| party_states.update(|s| {
                        if let Some(row) = s.get_mut(index) { row.2 = Some(v); }
                    })
                />
            })}
        </div>
    }
}

// ── AttendanceToggle: Yes / No button pair ─────────────────────────────────────

#[component]
fn AttendanceToggle(
    label: &'static str,
    value: Signal<bool>,
    on_change: impl Fn(bool) + 'static,
) -> impl IntoView {
    let on_change = std::rc::Rc::new(on_change);
    let on_change_no = on_change.clone();
    view! {
        <div class="flex flex-col gap-1">
            <p class="text-xs font-semibold uppercase tracking-wide text-charcoal/60">{label}</p>
            <div class="flex gap-3">
                <button
                    type="button"
                    class=move || {
                        let active = "bg-sage text-ivory font-semibold";
                        let inactive = "border border-sage text-sage";
                        format!("px-4 py-1 rounded text-sm transition-colors {}", if value.get() { active } else { inactive })
                    }
                    on:click=move |_| on_change(true)
                >
                    "Yes"
                </button>
                <button
                    type="button"
                    class=move || {
                        let active = "bg-charcoal/60 text-ivory font-semibold";
                        let inactive = "border border-charcoal/40 text-charcoal/60";
                        format!("px-4 py-1 rounded text-sm transition-colors {}", if !value.get() { active } else { inactive })
                    }
                    on:click=move |_| on_change_no(false)
                >
                    "No"
                </button>
            </div>
        </div>
    }
}
