use leptos::prelude::*;
use leptos::prelude::{ElementChild, OnAttribute, Get};
use leptos_query_rs::*;
use std::collections::HashMap;

// Advanced example showing complex patterns
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Post {
    id: u32,
    title: String,
    content: String,
    author_id: u32,
}

// Simulated API functions
use leptos_query_rs::retry::QueryError;

async fn fetch_user(id: u32) -> Result<User, QueryError> {
    // Simulate network delay
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    if id == 0 {
        return Err(QueryError::GenericError("User not found".to_string()));
    }
    
    Ok(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

async fn fetch_user_posts(user_id: u32) -> Result<Vec<Post>, QueryError> {
    std::thread::sleep(std::time::Duration::from_millis(150));
    
    Ok(vec![
        Post {
            id: 1,
            title: "First Post".to_string(),
            content: "This is the first post".to_string(),
            author_id: user_id,
        },
        Post {
            id: 2,
            title: "Second Post".to_string(),
            content: "This is the second post".to_string(),
            author_id: user_id,
        },
    ])
}

async fn create_post(user_id: u32, title: String, content: String) -> Result<Post, QueryError> {
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    Ok(Post {
        id: rand::random::<u32>(),
        title,
        content,
        author_id: user_id,
    })
}

#[component]
fn AdvancedUserProfile(user_id: u32) -> impl IntoView {
    // Query for user data
    let user_query = use_query(
        move || QueryKey::new(&["users", &user_id.to_string()]),
        move || async move { fetch_user(user_id).await },
        QueryOptions::default()
            .with_stale_time(std::time::Duration::from_secs(300)) // 5 minutes
            .with_cache_time(std::time::Duration::from_secs(600)) // 10 minutes
    );

    // Query for user posts
    let posts_query = use_query(
        move || QueryKey::new(&["users", &user_id.to_string(), "posts"]),
        move || async move { fetch_user_posts(user_id).await },
        QueryOptions::default()
    );

    // Mutation for creating posts
    let create_post_mutation = use_mutation::<Post, QueryError, (String, String), _, _>(
        move |new_post: (String, String)| {
            let (title, content) = new_post;
            async move { create_post(user_id, title, content).await }
        },
        MutationOptions::default()
    );

    // Form state
    let (title, set_title) = create_signal(String::new());
    let (content, set_content) = create_signal(String::new());

    view! {
        <div class="user-profile">
            <h2>"Advanced User Profile Example"</h2>
            
            // User information
            <div class="user-info">
                <h3>"User Information"</h3>
                                {move || {
                    let (content, class) = match user_query.data.get() {
                        Some(user) => (format!("Name: {}, Email: {}", user.name, user.email), "user-details"),
                        None if user_query.is_loading.get() => ("Loading user...".to_string(), "loading"),
                        None => ("No user data available".to_string(), "error"),
                    };
                    view! { <div class={class}><p>{content}</p></div> }.into_view()
                }}
            </div>

            // Posts section
            <div class="posts-section">
                <h3>"User Posts"</h3>
                {move || {
                    let (content, class) = if user_query.data.get().is_some() {
                        match posts_query.data.get() {
                            Some(posts) => {
                                let posts_content = posts.iter().map(|post| {
                                    format!("Title: {}, Content: {}", post.title, post.content)
                                }).collect::<Vec<_>>().join(" | ");
                                (posts_content, "posts-list")
                            },
                            None if posts_query.is_loading.get() => ("Loading posts...".to_string(), "loading"),
                            None => ("No posts data available".to_string(), "error"),
                        }
                    } else {
                        ("Waiting for user data...".to_string(), "loading")
                    };
                    view! { <div class={class}><p>{content}</p></div> }.into_view()
                }}
            </div>

            // Create post form
            <div class="create-post">
                <h3>"Create New Post"</h3>
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    if !title.get().is_empty() && !content.get().is_empty() {
                        create_post_mutation.mutate.run((title.get(), content.get()));
                        set_title.set(String::new());
                        set_content.set(String::new());
                    }
                }>
                    <div>
                        <label for="title">"Title:"</label>
                        <input
                            id="title"
                            type="text"
                            value=title
                            on:input=move |ev| set_title.set(event_target_value(&ev))
                            required
                        />
                    </div>
                    <div>
                        <label for="content">"Content:"</label>
                        <textarea
                            id="content"
                            prop:value=content
                            on:input=move |ev| set_content.set(event_target_value(&ev))
                            required
                        ></textarea>
                    </div>
                    <button
                        type="submit"
                        disabled=move || create_post_mutation.is_loading.get()
                    >
                        {move || if create_post_mutation.is_loading.get() {
                            "Creating..."
                        } else {
                            "Create Post"
                        }}
                    </button>
                </form>
            </div>

            // Query status information
            <div class="query-status">
                <h3>"Query Status"</h3>
                <p>"User query loading: "{move || user_query.is_loading.get()}</p>
                <p>"Posts query loading: "{move || posts_query.is_loading.get()}</p>
                <p>"Mutation pending: "{move || create_post_mutation.is_loading.get()}</p>
                
            </div>
        </div>
    }
}

fn main() {
    mount_to_body(|| view! {
        <div>
            <h1>"Leptos Query - Advanced Usage Example"</h1>
            <AdvancedUserProfile user_id=1/>
        </div>
    })
}
