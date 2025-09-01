use leptos::*;
use leptos_query_rs::QueryClientProvider;
use console_error_panic_hook;
use console_log;

mod app;

use app::App;

fn main() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Initialize logging
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
    
    // Mount the app
    mount_to_body(|| {
        view! {
            <QueryClientProvider>
                <App/>
            </QueryClientProvider>
        }
    });
}
