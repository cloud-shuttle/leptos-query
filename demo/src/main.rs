use leptos::*;
use leptos::prelude::{Get, Set, ElementChild, ClassAttribute, OnAttribute, signal, event_target_value, use_context, Effect, mount_to_body};
use leptos_query_rs::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use wasm_bindgen::prelude::*;

// Enable console error panic hook for better debugging
#[wasm_bindgen]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
    avatar: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Post {
    id: u32,
    title: String,
    content: String,
    author_id: u32,
}

// Simulate API calls
async fn fetch_user(id: u32) -> Result<User, QueryError> {
    // Simulate network delay
    gloo_timers::future::sleep(Duration::from_millis(800)).await;
    
    // Simulate occasional errors
    if id == 999 {
        return Err(QueryError::NetworkError("User not found".to_string()));
    }
    
    Ok(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
        avatar: format!("https://i.pravatar.cc/150?u={}", id),
    })
}

async fn fetch_user_posts(user_id: u32) -> Result<Vec<Post>, QueryError> {
    gloo_timers::future::sleep(Duration::from_millis(600)).await;
    
    Ok(vec![
        Post {
            id: user_id * 10 + 1,
            title: format!("First Post by User {}", user_id),
            content: "This is the content of the first post...".to_string(),
            author_id: user_id,
        },
        Post {
            id: user_id * 10 + 2,
            title: format!("Second Post by User {}", user_id),
            content: "This is the content of the second post...".to_string(),
            author_id: user_id,
        },
    ])
}

async fn create_post(post: CreatePostRequest) -> Result<Post, QueryError> {
    gloo_timers::future::sleep(Duration::from_millis(500)).await;
    
    Ok(Post {
        id: post.author_id * 100 + rand::random::<u32>() % 100,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreatePostRequest {
    title: String,
    content: String,
    author_id: u32,
}

#[component]
fn UserProfile(user_id: u32) -> impl IntoView {
    let user_query = use_query(
        move || QueryKey::from(&["user", &user_id.to_string()][..]),
        move || fetch_user(user_id),
        QueryOptions::default()
            .with_stale_time(Duration::from_secs(30))
            .with_cache_time(Duration::from_secs(300)),
    );

    let posts_query = use_query(
        move || QueryKey::from(&["posts", &user_id.to_string()][..]),
        move || fetch_user_posts(user_id),
        QueryOptions::default()
            .with_stale_time(Duration::from_secs(60))
            .with_cache_time(Duration::from_secs(600)),
    );

    view! {
        <div class="user-profile">
            <h3>"User Profile"</h3>
            
            // User data section
            <div class="user-data">
                {move || match user_query.data.get() {
                    Some(user) => view! {
                        <div class="user-info">
                            <img src=user.avatar alt="Avatar" class="avatar"/>
                            <div class="user-details">
                                <h4>{user.name}</h4>
                                <p>"Email: " {user.email}</p>
                                <p>"ID: " {user.id}</p>
                            </div>
                        </div>
                    },
                    None if user_query.is_loading.get() => view! {
                        <div class="user-info">
                            <img src="" alt="" class="avatar"/>
                            <div class="user-details">
                                <p>"Loading user..."</p>
                                <p></p>
                                <p></p>
                            </div>
                        </div>
                    },
                    None => view! {
                        <div class="user-info">
                            <img src="" alt="" class="avatar"/>
                            <div class="user-details">
                                <p>"No user found"</p>
                                <p></p>
                                <p></p>
                            </div>
                        </div>
                    },
                }}
                
                // Error display
                {move || user_query.error.get().map(|error| view! {
                    <div class="error">
                        "Error loading user: " {format!("{:?}", error)}
                    </div>
                })}
            </div>
            
            // Posts section
            <div class="posts-section">
                <h4>"User Posts"</h4>
                {move || match posts_query.data.get() {
                    Some(posts) => view! {
                        <div class="posts-list">
                            {posts.into_iter().map(|post| view! {
                                <div class="post">
                                    <h5>{post.title}</h5>
                                    <p>{post.content}</p>
                                </div>
                            }).collect::<Vec<_>>()}
                        </div>
                    },
                    None if posts_query.is_loading.get() => view! {
                        <div class="posts-list">
                            <div class="loading">"Loading posts..."</div>
                        </div>
                    },
                    None => view! {
                        <div class="posts-list">
                            <div class="no-posts">"No posts found"</div>
                        </div>
                    },
                }}
            </div>
            
            // Actions
            <div class="actions">
                <button 
                    class="btn btn-primary"
                    on:click=move |_| user_query.refetch.emit(())
                    disabled=move || user_query.is_loading.get()
                >
                    {move || if user_query.is_loading.get() { "Refreshing..." } else { "Refresh User" }}
                </button>
                
                <button 
                    class="btn btn-secondary"
                    on:click=move |_| posts_query.refetch.emit(())
                    disabled=move || posts_query.is_loading.get()
                >
                    {move || if posts_query.is_loading.get() { "Refreshing..." } else { "Refresh Posts" }}
                </button>
            </div>
        </div>
    }
}

#[component]
fn CreatePostForm(user_id: u32) -> impl IntoView {
    let (title, set_title) = signal(String::new());
    let (content, set_content) = signal(String::new());
    
    let mutation = use_mutation(
        move |post: CreatePostRequest| create_post(post),
        MutationOptions::default()
            .invalidate_queries(vec![QueryKeyPattern::Exact(QueryKey::from(&["posts", &user_id.to_string()][..]))]),
    );
    
    let handle_submit = move |_| {
        if !title.get().is_empty() && !content.get().is_empty() {
            let post = CreatePostRequest {
                title: title.get(),
                content: content.get(),
                author_id: user_id,
            };
            mutation.mutate.emit(post);
            
            // Clear form
            set_title.set(String::new());
            set_content.set(String::new());
        }
    };
    
    view! {
        <div class="create-post-form">
            <h4>"Create New Post"</h4>
            <div class="form-group">
                <label>"Title:"</label>
                <input 
                    type="text" 
                    value=title.get()
                    on:input=move |ev| set_title.set(event_target_value(&ev))
                    placeholder="Enter post title"
                />
            </div>
            <div class="form-group">
                <label>"Content:"</label>
                <textarea 
                    on:input=move |ev| set_content.set(event_target_value(&ev))
                    placeholder="Enter post content"
                    rows="3"
                >{content.get()}</textarea>
            </div>
            <button 
                class="btn btn-success"
                on:click=handle_submit
                disabled=move || mutation.is_loading.get() || title.get().is_empty() || content.get().is_empty()
            >
                {move || if mutation.is_loading.get() { "Creating..." } else { "Create Post" }}
            </button>
            
            // Mutation status
            {move || mutation.data.get().map(|post| view! {
                <div class="success">
                    "Post created successfully: " {post.title}
                </div>
            })}
            
            {move || mutation.error.get().map(|error| view! {
                <div class="error">
                    "Error creating post: " {format!("{:?}", error)}
                </div>
            })}
        </div>
    }
}

#[component]
fn QueryStatus() -> impl IntoView {
    let client = use_context::<QueryClient>().expect("QueryClient not found");
    let (stats, set_stats) = signal(client.cache_stats());
    let client_for_button = client.clone();
    
    // Update stats periodically
    Effect::new(move |_| {
        let client = client.clone();
        let set_stats = set_stats.clone();
        
        gloo_timers::callback::Timeout::new(1000, move || {
            set_stats.set(client.cache_stats());
        }).forget();
    });
    
    view! {
        <div class="query-status">
            <h4>"Cache Status"</h4>
            <div class="stats">
                <p>"Total entries: " {stats.get().total_entries}</p>
                <p>"Stale entries: " {stats.get().stale_entries}</p>
            </div>
            <button 
                class="btn btn-warning"
                on:click=move |_| {
                    client_for_button.clear_cache();
                    set_stats.set(client_for_button.cache_stats());
                }
            >
                "Clear Cache"
            </button>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (selected_user_id, set_selected_user_id) = signal(1);
    
    view! {
        <QueryClientProvider>
            <div class="app">
                <header class="header">
                    <h1>"🚀 Leptos Query Demo"</h1>
                    <p>"A demonstration of the Leptos Query library for data fetching and caching"</p>
                </header>
                
                <main class="main">
                    <div class="sidebar">
                        <h3>"Select User"</h3>
                        <div class="user-selector">
                            {[1, 2, 3, 4, 5].into_iter().map(|id| {
                                let is_selected = move || selected_user_id.get() == id;
                                let set_id = set_selected_user_id.clone();
                                
                                view! {
                                    <button 
                                        class=move || if is_selected() { "user-btn selected" } else { "user-btn" }
                                        on:click=move |_| set_id.set(id)
                                    >
                                        "User " {id}
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        
                        <QueryStatus/>
                    </div>
                    
                    <div class="content">
                        <UserProfile user_id=selected_user_id.get()/>
                        <CreatePostForm user_id=selected_user_id.get()/>
                    </div>
                </main>
            </div>
        </QueryClientProvider>
    }
}
