use crate::api::client;
use leptos::prelude::*;
use shared::models::hotel::HotelRoom;

#[component]
pub fn Hotel() -> impl IntoView {
    // LocalResource: for CSR-only fetches using browser APIs (JsFuture is !Send).
    // Takes a single fetcher closure; reactive deps inside are tracked automatically.
    let hotels = LocalResource::new(|| async move {
        client::get::<Vec<HotelRoom>>(shared::api::routes::HOTELS_LIST).await
    });

    view! {
        <div class="max-w-3xl mx-auto px-6 py-12">
            <h1 class="font-serif text-4xl text-charcoal mb-8 text-center">"Hotel Information"</h1>
            <Suspense fallback=|| view! { <crate::components::spinner::Spinner /> }>
                {move || hotels.get().map(|data| {
                    // data is SendWrapper<Result<...>>; deref to access inner value.
                    match &*data {
                        Ok(rooms) if rooms.is_empty() => view! {
                            <p class="text-center text-charcoal italic">
                                "Hotel information will be posted soon. Check back later!"
                            </p>
                        }.into_any(),
                        Ok(rooms) => {
                            let rooms = rooms.clone();
                            view! {
                                <ul class="flex flex-col gap-4">
                                    {rooms.into_iter().map(|room| view! {
                                        <li class="border border-gold rounded p-4 bg-ivory">
                                            <h2 class="font-serif text-xl text-charcoal">{room.hotel_name}</h2>
                                            <p class="text-sm text-charcoal/70">{room.room_type}</p>
                                            {room.booking_url.map(|url| view! {
                                                <a
                                                    href=url
                                                    target="_blank"
                                                    class="text-sage underline text-sm"
                                                >
                                                    "Book now"
                                                </a>
                                            })}
                                        </li>
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <p class="text-red-500">"Failed to load hotel info: " {e.clone()}</p>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
