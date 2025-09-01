use leptos::*;
use leptos_query_rs::*;
use std::collections::HashMap;

// Advanced example showing complex patterns
#[derive(Clone, Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Clone, Debug)]
struct Post {
    id: u32,
    title: String,
    content: String,
    author_id: u32,
}

// Simulated API functions
async fn fetch_user(id: u32) -> Result<User, String> {
    // Simulate network delay
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    if id == 0 {
        return Err("User not found".to_string());
    }
    
    Ok(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

async fn fetch_user_posts(user_id: u32) -> Result<Vec<Post>, String> {
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

async fn create_post(user_id: u32, title: String, content: String) -> Result<Post, String> {
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
        move || &["users", &user_id.to_string()][..],
        move || || async move { fetch_user(user_id).await },
        QueryOptions::default()
            .stale_time(std::time::Duration::from_secs(300)) // 5 minutes
            .cache_time(std::time::Duration::from_secs(600)) // 10 minutes
    );

    // Query for user posts
    let posts_query = use_query(
        move || &["users", &user_id.to_string(), "posts"][..],
        move || || async move { fetch_user_posts(user_id).await },
        QueryOptions::default()
            .enabled(user_query.data().is_some()) // Only fetch posts if user exists
    );

    // Mutation for creating posts
    let create_post_mutation = use_mutation(
        move |new_post: &(String, String)| {
            let (title, content) = new_post.clone();
            async move { create_post(user_id, title, content).await }
        },
        MutationOptions::default()
            .on_success(move |_post| {
                // Invalidate and refetch posts
                let query_client = use_query_client();
                query_client.invalidate_queries(&["users", &user_id.to_string(), "posts"]);
            })
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
                {move || match user_query.data() {
                    Some(Ok(user)) => view! {
                        <div class="user-details">
                            <p><strong>"Name: "</strong>{user.name}</p>
                            <p><strong>"Email: "</strong>{user.email}</p>
                        </div>
                    }.into_view(),
                    Some(Err(e)) => view! {
                        <div class="error">
                            <p>"Error loading user: "{e}</p>
                        </div>
                    }.into_view(),
                    None if user_query.is_loading() => view! {
                        <div class="loading">
                            <p>"Loading user..."</p>
                        </div>
                    }.into_view(),
                    None => view! {
                        <div class="error">
                            <p>"No user data available"</p>
                        </div>
                    }.into_view(),
                }}
            </div>

            // Posts section
            <div class="posts-section">
                <h3>"User Posts"</h3>
                {move || if user_query.data().is_some() {
                    match posts_query.data() {
                        Some(Ok(posts)) => view! {
                            <div class="posts-list">
                                {posts.iter().map(|post| view! {
                                    <div class="post">
                                        <h4>{post.title.clone()}</h4>
                                        <p>{post.content.clone()}</p>
                                    </div>
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_view(),
                        Some(Err(e)) => view! {
                            <div class="error">
                                <p>"Error loading posts: "{e}</p>
                            </div>
                        }.into_view(),
                        None if posts_query.is_loading() => view! {
                            <div class="loading">
                                <p>"Loading posts..."</p>
                            </div>
                        }.into_view(),
                        None => view! {
                            <div class="error">
                                <p>"No posts data available"</p>
                            </div>
                        }.into_view(),
                    }
                } else {
                    view! {
                        <div class="loading">
                            <p>"Waiting for user data..."</p>
                        </div>
                    }.into_view()
                }}
            </div>

            // Create post form
            <div class="create-post">
                <h3>"Create New Post"</h3>
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    if !title.get().is_empty() && !content.get().is_empty() {
                        create_post_mutation.mutate((title.get(), content.get()));
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
                            value=content
                            on:input=move |ev| set_content.set(event_target_value(&ev))
                            required
                        ></textarea>
                    </div>
                    <button
                        type="submit"
                        disabled=move || create_post_mutation.is_pending()
                    >
                        {move || if create_post_mutation.is_pending() {
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
                <p>"User query loading: "{move || user_query.is_loading()}</p>
                <p>"Posts query loading: "{move || posts_query.is_loading()}</p>
                <p>"Mutation pending: "{move || create_post_mutation.is_pending()}</p>
                <p>"User query stale: "{move || user_query.is_stale()}</p>
                <p>"Posts query stale: "{move || posts_query.is_stale()}</p>
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
