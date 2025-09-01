use leptos::prelude::*;
use leptos::prelude::{ElementChild, OnAttribute, Get};
use leptos_query_rs::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

async fn fetch_user(id: u32) -> Result<User, QueryError> {
    // Simulate network delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Simulate successful response
    Ok(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

#[component]
fn UserProfile(user_id: u32) -> impl IntoView {
    let user_query = use_query(
        move || QueryKey::new(&["user", &user_id.to_string()]),
        move || async move { fetch_user(user_id).await },
        QueryOptions::default(),
    );

    view! {
        <div>
            <h2>"User Profile"</h2>
            {move || {
                let content = match user_query.data.get() {
                    Some(user) => format!("Name: {}, Email: {}", user.name, user.email),
                    None if user_query.is_loading.get() => "Loading...".to_string(),
                    None => "No user found".to_string(),
                };
                view! { <div><p>{content}</p></div> }.into_view()
            }}
            <button on:click=move |_| user_query.refetch.run(())>
                "Refresh"
            </button>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    view! {
        <div>
            <h1>"Leptos Query Example"</h1>
            <UserProfile user_id=1/>
        </div>
    }
}

fn main() {
    mount_to_body(|| view! { <App/> })
}
