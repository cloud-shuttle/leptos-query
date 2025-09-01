use leptos::*;
use leptos_query::*;
use leptos_query::retry::QueryError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Utility sleep function for WASM compatibility
async fn sleep(duration: Duration) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve, 
                duration.as_millis() as i32
            )
            .unwrap();
    });
    
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}

// Example data structures
#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

// Mock API functions
async fn fetch_user(id: u32) -> Result<User, QueryError> {
    // Simulate network delay
    sleep(Duration::from_millis(100)).await;
    
    // Simulate API call
    if id == 1 {
        Ok(User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        })
    } else {
        Err(QueryError::http(404, "User not found"))
    }
}

async fn create_user(request: CreateUserRequest) -> Result<User, QueryError> {
    // Simulate network delay
    sleep(Duration::from_millis(200)).await;
    
    // Simulate API call
    Ok(User {
        id: 2,
        name: request.name,
        email: request.email,
    })
}

// Example component using queries
#[component]
fn UserProfile(user_id: u32) -> impl IntoView {
    let user_query = use_query(
        move || {
            let id_str = user_id.to_string();
            QueryKey::new(&["users", &id_str])
        },
        move || move || async move { fetch_user(user_id).await },
        QueryOptions::default()
            .with_stale_time(Duration::from_secs(60))
            .with_cache_time(Duration::from_secs(300))
    );

    view! {
        <div>
            <h2>"User Profile"</h2>
            {move || {
                if user_query.is_loading.get() {
                    view! { <div>"Loading..."</div> }
                } else if let Some(error) = user_query.error.get() {
                    view! { <div>"Error: " {error.to_string()}</div> }
                } else if let Some(user) = user_query.data.get() {
                    view! {
                        <div>
                            <h3>{user.name}</h3>
                            <p>"Email: " {user.email}</p>
                            <p>"ID: " {user.id}</p>
                        </div>
                    }
                } else {
                    view! { <div>"No data"</div> }
                }
            }}
            <button on:click=move |_| user_query.refetch.call(())>
                "Refresh"
            </button>
        </div>
    }
}

// Example component using mutations
#[component]
fn CreateUserForm() -> impl IntoView {
    let create_user_mutation = use_mutation::<User, CreateUserRequest, (), _, _>(
        |request: CreateUserRequest| async move { create_user(request).await },
        MutationOptions::default()
    );

    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());

    let handle_submit = move |_| {
        let request = CreateUserRequest {
            name: name.get(),
            email: email.get(),
        };
        create_user_mutation.mutate.call(request);
    };

    view! {
        <div>
            <h2>"Create User"</h2>
            <form on:submit=handle_submit>
                <div>
                    <label>"Name: "</label>
                    <input
                        type="text"
                        value=name
                        on:input=move |ev| set_name.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label>"Email: "</label>
                    <input
                        type="email"
                        value=email
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                    />
                </div>
                <button type="submit" disabled=move || create_user_mutation.is_loading.get()>
                    {move || if create_user_mutation.is_loading.get() { "Creating..." } else { "Create User" }}
                </button>
            </form>
            
            {move || {
                if let Some(error) = create_user_mutation.error.get() {
                    view! { <div style="color: red">"Error: " {error.to_string()}</div> }
                } else if let Some(user) = create_user_mutation.data.get() {
                    view! { <div style="color: green">"Created user: " {user.name}</div> }
                } else {
                    view! { <div></div> }
                }
            }}
        </div>
    }
}

// Main app component
#[component]
fn App() -> impl IntoView {
    // Provide QueryClient to the app
    provide_context(QueryClient::new(QueryClientConfig::default()));

    view! {
        <div>
            <h1>"Leptos Query Example"</h1>
            <UserProfile user_id=1/>
            <hr/>
            <CreateUserForm/>
        </div>
    }
}

fn main() {
    mount_to_body(|| view! { <App/> })
}
