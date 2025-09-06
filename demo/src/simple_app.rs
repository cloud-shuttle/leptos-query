use leptos::*;
use leptos_query_rs::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Simple mock API function
async fn fetch_user(id: u32) -> Result<User, QueryError> {
    // Simulate network delay
    gloo_timers::future::TimeoutFuture::new(1000).await;
    
    if id == 0 {
        return Err(QueryError::NetworkError("User not found".to_string()));
    }
    
    Ok(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

#[component]
pub fn SimpleApp() -> impl IntoView {
    let (user_id, set_user_id) = signal(1u32);
    
    // Query for user data
    let user_query = use_query(
        move || {
            let id = user_id.get();
            QueryKey::new(&["user", &id.to_string()])
        },
        move || {
            let id = user_id.get();
            async move { fetch_user(id).await }
        },
        QueryOptions::default()
            .with_stale_time(Duration::from_secs(30))
            .with_cache_time(Duration::from_secs(60))
    );

    view! {
        <div class="app">
            <header class="app-header">
                <h1>"🚀 Leptos Query Demo"</h1>
                <p>"A simple demonstration of leptos-query features"</p>
            </header>

            <main class="app-main">
                <div class="demo-section">
                    <h2>"User Query Demo"</h2>
                    
                    <div class="controls">
                        <label>"User ID: "</label>
                        <input
                            type="number"
                            value=user_id
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<u32>().unwrap_or(1);
                                set_user_id.set(value);
                            }
                        />
                        <button 
                            on:click=move |_| user_query.refetch.run(())
                            disabled=move || user_query.is_loading.get()
                        >
                            {move || if user_query.is_loading.get() { "Loading..." } else { "Fetch User" }}
                        </button>
                    </div>

                    <div class="query-status">
                        <h3>"Query Status"</h3>
                        <p>"Loading: "{move || user_query.is_loading.get()}</p>
                        <p>"Stale: "{move || user_query.is_stale.get()}</p>
                        <p>"Error: "{move || user_query.error.get().is_some()}</p>
                    </div>

                    <div class="user-data">
                        <h3>"User Data"</h3>
                        {move || match user_query.data.get() {
                            Some(user) => view! {
                                <div class="user-card">
                                    <h4>{user.name}</h4>
                                    <p>"Email: "{user.email}</p>
                                    <p>"ID: "{user.id}</p>
                                </div>
                            }.into_view(),
                            None => {
                                if user_query.error.get().is_some() {
                                    view! {
                                        <div class="error">
                                            <p>"Error: "{user_query.error.get().unwrap().to_string()}</p>
                                        </div>
                                    }.into_view()
                                } else if user_query.is_loading.get() {
                                    view! {
                                        <div class="loading">
                                            <p>"Loading user data..."</p>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <div class="loading">
                                            <p>"Click 'Fetch User' to load data"</p>
                                        </div>
                                    }.into_view()
                                }
                            }
                        }}
                    </div>

                    <div class="features">
                        <h3>"Features Demonstrated"</h3>
                        <ul>
                            <li>"🔄 Automatic caching with configurable stale times"</li>
                            <li>"⚡ Background updates when data becomes stale"</li>
                            <li>"🎯 Query key-based cache invalidation"</li>
                            <li>"🛡️ Built-in error handling"</li>
                            <li>"📊 Loading and stale state management"</li>
                        </ul>
                    </div>
                </div>
            </main>

            <footer class="app-footer">
                <p>"Built with " <a href="https://github.com/cloud-shuttle/leptos-query" target="_blank">"leptos-query"</a></p>
            </footer>
        </div>
    }
}
