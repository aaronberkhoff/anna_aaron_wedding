use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

// In production the frontend is served by the same origin as the API,
// so an empty base URL means same-origin requests. During dev with
// `trunk serve` running on :3000 and the server on :8080, set
// API_BASE_URL=http://localhost:8080 in .env before running `trunk serve`.
const API_BASE: &str = match option_env!("API_BASE_URL") {
    Some(url) => url,
    None => "",
};

/// GET request returning deserialized JSON.
pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, path);
    let request = Request::new_with_str(&url).map_err(|e| format!("{e:?}"))?;

    let window = web_sys::window().ok_or("no window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;

    let resp: Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?;

    serde_wasm_bindgen::from_value(json).map_err(|e| e.to_string())
}

/// POST request with a JSON body, returning deserialized JSON.
pub async fn post<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, String> {
    let body_str = serde_json::to_string(body).map_err(|e| e.to_string())?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    // Cors mode required: in dev, frontend (:3000) and API (:8080) are different origins.
    // In production they are same-origin (both served by Axum on :8080), so Cors is safe either way.
    opts.set_mode(RequestMode::Cors);
    opts.set_body(&JsValue::from_str(&body_str));

    let url = format!("{}{}", API_BASE, path);
    let request = Request::new_with_str_and_init(&url, &opts).map_err(|e| format!("{e:?}"))?;
    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("{e:?}"))?;

    let window = web_sys::window().ok_or("no window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("{e:?}"))?;

    let resp: Response = resp_value.dyn_into().map_err(|e| format!("{e:?}"))?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("{e:?}"))?)
        .await
        .map_err(|e| format!("{e:?}"))?;

    serde_wasm_bindgen::from_value(json).map_err(|e| e.to_string())
}
