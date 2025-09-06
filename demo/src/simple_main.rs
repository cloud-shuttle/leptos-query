use leptos::*;
use wasm_bindgen::prelude::*;

mod simple_app;
use simple_app::SimpleApp;

// Enable console error panic hook for better debugging
#[wasm_bindgen]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <QueryClientProvider><SimpleApp/></QueryClientProvider> })
}
