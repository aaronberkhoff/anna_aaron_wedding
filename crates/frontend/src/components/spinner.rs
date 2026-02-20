use leptos::prelude::*;

/// Simple loading spinner shown while async resources are fetching.
#[component]
pub fn Spinner() -> impl IntoView {
    view! {
        <div class="flex justify-center items-center py-12">
            <div class="w-10 h-10 border-4 border-sage border-t-transparent rounded-full animate-spin" />
        </div>
    }
}
