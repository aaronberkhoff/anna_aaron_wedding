mod api;
mod app;
mod components;
mod pages;

use app::App;
use wasm_bindgen::prelude::*;

/// WASM entry point. Trunk calls this automatically on page load because of
/// the `#[wasm_bindgen(start)]` attribute — no explicit JS call needed.
#[wasm_bindgen(start)]
pub fn main() {
    // Redirect Rust panics to the browser console (dev-friendly error messages).
    console_error_panic_hook::set_once();

    // Pipe log::* macros to the browser console.
    console_log::init_with_level(log::Level::Debug).expect("console_log init failed");

    // Mount the Leptos SPA to <body>.
    leptos::mount::mount_to_body(App);
}
