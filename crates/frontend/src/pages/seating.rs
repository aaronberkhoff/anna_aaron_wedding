use leptos::prelude::*;

// ── D3 / Sigma interop hook ───────────────────────────────────────────────
// When you're ready to add the visualization:
// 1. Uncomment the extern block below.
// 2. Add `<script src="https://d3js.org/d3.v7.min.js"></script>` to index.html.
// 3. Implement `window.initSeatingChart(containerEl, data)` in a JS <script> block
//    or a separate .js file copied via Trunk `data-trunk rel="copy-file"`.
//
// use wasm_bindgen::prelude::*;
// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = window, js_name = initSeatingChart)]
//     fn init_seating_chart(container: &web_sys::HtmlElement, data: &JsValue);
// }

#[component]
pub fn Seating() -> impl IntoView {
    // NodeRef captures the container div that D3/Sigma will render into.
    // Leptos does NOT manage DOM inside this div — JS owns it after init.
    let chart_ref = NodeRef::<leptos::html::Div>::new();

    // LocalResource: for CSR-only fetches using browser APIs (JsFuture is !Send).
    let seating_data = LocalResource::new(|| async move {
        crate::api::client::get::<shared::models::table::SeatingChart>(
            shared::api::routes::SEATING_CHART,
        )
        .await
    });

    // Once both the DOM element and data are ready, initialize the chart.
    // Uncomment when the JS function is implemented:
    //
    // Effect::new(move |_| {
    //     if let Some(data) = seating_data.get() {
    //         if let (Some(el), Ok(chart)) = (chart_ref.get(), &*data) {
    //             let js_data = serde_wasm_bindgen::to_value(chart).unwrap_or(JsValue::NULL);
    //             init_seating_chart(&el, &js_data);
    //         }
    //     }
    // });

    view! {
        <div class="px-6 py-12">
            <h1 class="font-serif text-4xl text-charcoal mb-8 text-center">"Seating Chart"</h1>
            <Suspense fallback=|| view! { <crate::components::spinner::Spinner /> }>
                {move || seating_data.get().map(|data| {
                    // data is SendWrapper<Result<...>>; deref to access inner value.
                    match &*data {
                        Ok(_chart) => view! {
                            // D3/Sigma will render inside this div.
                            <div
                                node_ref=chart_ref
                                id="seating-chart-container"
                                class="w-full h-[600px] border border-gold rounded bg-ivory"
                            >
                                <p class="text-center text-charcoal pt-8 text-sm italic">
                                    "Seating visualization coming soon."
                                </p>
                            </div>
                        }.into_any(),
                        Err(e) => view! {
                            <p class="text-red-500">"Failed to load seating data: " {e.clone()}</p>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
