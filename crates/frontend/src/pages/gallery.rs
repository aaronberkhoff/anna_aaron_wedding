use crate::api::client;
use leptos::prelude::*;
use shared::models::photo::Photo;

#[component]
pub fn Gallery() -> impl IntoView {
    // LocalResource: for CSR-only fetches using browser APIs (JsFuture is !Send).
    let photos = LocalResource::new(|| async move {
        client::get::<Vec<Photo>>(shared::api::routes::PHOTOS_LIST).await
    });

    view! {
        <div class="px-6 py-12">
            <h1 class="font-serif text-4xl text-charcoal mb-8 text-center">"Gallery"</h1>
            <Suspense fallback=|| view! { <crate::components::spinner::Spinner /> }>
                {move || photos.get().map(|data| {
                    // data is SendWrapper<Result<...>>; deref to access inner value.
                    match &*data {
                        Ok(photos) if photos.is_empty() => view! {
                            <p class="text-center text-charcoal italic">
                                "Photos will be added after the wedding. Stay tuned!"
                            </p>
                        }.into_any(),
                        Ok(photos) => {
                            let photos = photos.clone();
                            view! {
                                <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                                    {photos.into_iter().map(|photo| view! {
                                        <figure class="rounded overflow-hidden border border-gold bg-ivory">
                                            <img
                                                src=format!("/photos/{}", photo.filename)
                                                alt=photo.caption.clone().unwrap_or_default()
                                                class="w-full h-48 object-cover"
                                            />
                                            {photo.caption.map(|cap| view! {
                                                <figcaption class="px-2 py-1 text-xs text-charcoal/70">
                                                    {cap}
                                                </figcaption>
                                            })}
                                        </figure>
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <p class="text-red-500">"Failed to load gallery: " {e.clone()}</p>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
