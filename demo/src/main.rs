use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_query::*;
use console_error_panic_hook;
use console_log;

mod app;
mod components;
mod api;
mod types;

use app::App;

fn main() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Initialize logging
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
    
    // Create the query client
    let query_client = QueryClient::new();
    
    // Mount the app
    mount_to_body(|| {
        view! {
            <QueryClientProvider client=query_client>
                <App/>
            </QueryClientProvider>
        }
    });
}
