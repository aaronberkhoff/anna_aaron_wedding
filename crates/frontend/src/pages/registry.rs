use crate::api::client;
use leptos::prelude::*;
use shared::{
    api::routes,
    models::registry::{ContributionRequest, ContributionResponse, RegistryFund},
};

const MYREGISTRY_URL: &str = "https://www.myregistry.com/wedding-registry/anna-hagen-and-aaron-berkhoff-colorado-springs-co/5369696/giftlist";
const MYREGISTRY_PASSWORD: &str = "anna&aaron";

#[component]
pub fn Registry() -> impl IntoView {
    let funds = LocalResource::new(|| async move {
        client::get::<Vec<RegistryFund>>(routes::REGISTRY_FUNDS_LIST).await
    });

    view! {
        <div class="max-w-5xl mx-auto px-6 py-12">
            <h1 class="font-serif text-4xl text-charcoal mb-4 text-center">"Registry"</h1>
            <div class="flex items-center gap-4 justify-center mb-6 w-48 mx-auto">
                <div class="flex-1 h-px bg-gold/40"></div>
                <svg class="w-2 h-2 fill-gold opacity-60" viewBox="0 0 20 20">
                    <path d="M10 0 L12.5 7.5 L20 10 L12.5 12.5 L10 20 L7.5 12.5 L0 10 L7.5 7.5 Z" />
                </svg>
                <div class="flex-1 h-px bg-gold/40"></div>
            </div>

            <div class="text-center mb-12 flex flex-col items-center gap-3">
                <p class="text-charcoal/70 max-w-2xl">
                    "Your presence is the only gift we need, but for those who have asked, "
                    "we've put together a registry."
                </p>
                <a
                    href=MYREGISTRY_URL
                    target="_blank"
                    rel="noopener noreferrer"
                    class="inline-block bg-gold text-ivory font-semibold px-6 py-3 rounded hover:bg-charcoal transition-colors"
                >
                    "View Our MyRegistry List"
                </a>
                <p class="text-xs text-charcoal/50">
                    "Registry password: " {MYREGISTRY_PASSWORD}
                </p>
            </div>

            <h2 class="font-serif text-2xl text-charcoal mb-2 text-center">"Or Contribute to a Fund"</h2>
            <p class="text-center text-charcoal/60 text-sm max-w-2xl mx-auto mb-8">
                "These funds are paid directly via Venmo. Totals shown are self-reported by "
                "guests when they log a contribution — we have no way to verify a Venmo payment "
                "was actually completed, so please only log an amount you intend to send."
            </p>

            <Suspense fallback=|| view! { <crate::components::spinner::Spinner /> }>
                {move || funds.get().map(|data| {
                    match &*data {
                        Ok(funds) => {
                            let funds = funds.clone();
                            view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                                    {funds.into_iter().map(|fund| view! { <FundCard fund=fund /> }).collect_view()}
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <p class="text-red-500 text-center">"Failed to load funds: " {e.clone()}</p>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn FundCard(fund: RegistryFund) -> impl IntoView {
    let fund_id = fund.id.clone();
    let (total, set_total) = signal(fund.total_usd);
    let (name_input, set_name_input) = signal(String::new());
    let (amount_input, set_amount_input) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (venmo_result, set_venmo_result) = signal(Option::<ContributionResponse>::None);

    view! {
        <div class="border border-gold rounded-lg bg-ivory p-6 flex flex-col gap-4">
            <div>
                <h3 class="font-serif text-xl text-charcoal">{fund.name.clone()}</h3>
                <p class="text-sm text-charcoal/70 mt-1">{fund.description.clone()}</p>
            </div>
            <p class="font-serif text-2xl text-charcoal">
                {move || format!("${:.2}", total.get())} " raised so far"
            </p>

            {move || match venmo_result.get() {
                None => {
                    let fund_id = fund_id.clone();
                    let on_submit = move |_| {
                        let name = name_input.get().trim().to_string();
                        if name.is_empty() {
                            set_error.set(Some("Please enter your name.".to_string()));
                            return;
                        }
                        let amount: f64 = match amount_input.get().trim().parse() {
                            Ok(v) if v > 0.0 => v,
                            _ => {
                                set_error.set(Some("Please enter a valid amount.".to_string()));
                                return;
                            }
                        };

                        let payload = ContributionRequest {
                            fund_id: fund_id.clone(),
                            contributor_name: name,
                            amount_usd: amount,
                        };

                        set_error.set(None);
                        set_submitting.set(true);
                        leptos::task::spawn_local(async move {
                            match client::post::<_, ContributionResponse>(routes::REGISTRY_CONTRIBUTE, &payload).await {
                                Ok(resp) => {
                                    set_total.set(resp.new_total_usd);
                                    set_venmo_result.set(Some(resp));
                                }
                                Err(e) => set_error.set(Some(e)),
                            }
                            set_submitting.set(false);
                        });
                    };
                    view! {
                    <div class="flex flex-col gap-3">
                        <input
                            type="text"
                            placeholder="Your name"
                            class="border border-gold rounded px-4 py-2 bg-white text-charcoal text-sm"
                            on:input=move |ev| set_name_input.set(event_target_value(&ev))
                        />
                        <input
                            type="number"
                            step="0.01"
                            min="0.01"
                            placeholder="Amount ($)"
                            class="border border-gold rounded px-4 py-2 bg-white text-charcoal text-sm"
                            on:input=move |ev| set_amount_input.set(event_target_value(&ev))
                        />
                        {move || error.get().map(|e| view! {
                            <p class="text-red-600 text-xs">{e}</p>
                        })}
                        <button
                            type="button"
                            class="bg-gold text-ivory font-semibold px-5 py-2 rounded hover:bg-charcoal transition-colors text-sm"
                            on:click=on_submit
                            disabled=submitting
                        >
                            {move || if submitting.get() { "Logging…" } else { "Log Contribution & Continue to Venmo" }}
                        </button>
                    </div>
                    }.into_any()
                }
                Some(resp) => {
                    let username_no_at = resp.venmo_username.trim_start_matches('@').to_string();
                    let amount = amount_input.get();
                    let note = format!("Wedding gift — {}", fund.name);
                    let encoded_note = js_sys::encode_uri_component(&note).as_string().unwrap_or_default();
                    // Venmo's app-scheme deep link is undocumented/unofficial and can change
                    // without notice — test on a real phone before relying on it.
                    let venmo_app_link = format!(
                        "venmo://paycharge?txn=pay&recipients={username_no_at}&amount={amount}&note={encoded_note}"
                    );
                    view! {
                        <div class="flex flex-col gap-2 text-center">
                            <p class="text-sage text-sm">"Thank you! Your contribution has been logged."</p>
                            <a
                                href=venmo_app_link
                                class="block text-center bg-gold text-ivory font-semibold px-6 py-3 rounded hover:bg-charcoal transition-colors"
                            >
                                {format!("Pay ${amount} on Venmo")}
                            </a>
                            <p class="text-xs text-charcoal/50">
                                "If that doesn't open the app, open Venmo yourself and pay "
                                {resp.venmo_username.clone()}
                                " manually."
                            </p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
