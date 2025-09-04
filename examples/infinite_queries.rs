use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_query_rs::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Example data structure for paginated posts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Post {
    id: usize,
    title: String,
    content: String,
    author: String,
    created_at: String,
}

/// Mock API function to fetch posts with pagination
async fn fetch_posts(page: usize) -> Result<Page<Post>, QueryError> {
    // Simulate network delay
    std::thread::sleep(Duration::from_millis(100));
    
    let per_page = 5;
    let total_posts = 25; // Total posts available
    let start = page * per_page;
    
    // Simulate some posts
    let posts = (start..std::cmp::min(start + per_page, total_posts))
        .map(|i| Post {
            id: i,
            title: format!("Post Title {}", i + 1),
            content: format!("This is the content for post {}. It contains some sample text to demonstrate the infinite query functionality.", i + 1),
            author: format!("Author {}", (i % 3) + 1),
            created_at: format!("2024-{:02}-{:02}", (i % 12) + 1, (i % 28) + 1),
        })
        .collect();

    let page_info = PageInfo {
        page,
        per_page,
        total: total_posts,
        has_next: start + per_page < total_posts,
        has_prev: page > 0,
    };

    Ok(Page {
        data: posts,
        info: page_info,
    })
}

/// Component demonstrating infinite queries
#[component]
fn InfinitePosts() -> impl IntoView {
    let infinite_query = use_infinite_query(
        || ["posts", "infinite"],
        |page| async move { fetch_posts(page).await },
        InfiniteQueryOptions::builder()
            .max_pages(Some(5)) // Keep max 5 pages in memory
            .keep_previous_data(true)
            .build(),
    );

    let posts = infinite_query.pages;
    let current_page = infinite_query.current_page;
    let has_next = infinite_query.has_next;
    let has_prev = infinite_query.has_prev;
    let is_loading = infinite_query.is_loading;
    let error = infinite_query.error;

    view! {
        <div class="infinite-posts">
            <h2>"Infinite Posts Example"</h2>
            
            // Error display
            {move || error.get().map(|e| view! {
                <div class="error">
                    <strong>"Error: "</strong>
                    {e.to_string()}
                </div>
            })}
            
            // Posts list
            <div class="posts-container">
                {move || posts.get().into_iter().enumerate().flat_map(|(page_idx, page)| {
                    page.data.into_iter().enumerate().map(move |(item_idx, post)| {
                        let global_idx = page_idx * 5 + item_idx;
                        view! {
                            <div class="post-item">
                                <h3>{post.title}</h3>
                                <p class="post-meta">
                                    <span class="author">"By: " {post.author}</span>
                                    <span class="date">{post.created_at}</span>
                                </p>
                                <p class="post-content">{post.content}</p>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }).collect::<Vec<_>>()}
            </div>
            
            // Loading indicator
            {move || if is_loading.get() {
                view! { <div class="loading">"Loading more posts..."</div> }
            } else {
                view! { <div>"No more posts"</div> }
            }}
            
            // Navigation controls
            <div class="navigation">
                <button
                    disabled=move || !has_prev.get()
                    on:click=move |_| {
                        let query = infinite_query.clone();
                        spawn_local(async move {
                            let _ = query.fetch_previous_page().await;
                        });
                    }
                >
                    "← Previous Page"
                </button>
                
                <span class="page-info">
                    "Page " {move || current_page.get() + 1}
                </span>
                
                <button
                    disabled=move || !has_next.get()
                    on:click=move |_| {
                        let query = infinite_query.clone();
                        spawn_local(async move {
                            let _ = query.fetch_next_page().await;
                        });
                    }
                >
                    "Next Page →"
                </button>
            </div>
            
            // Actions
            <div class="actions">
                <button
                    on:click=move |_| {
                        let query = infinite_query.clone();
                        spawn_local(async move {
                            let _ = query.refetch().await;
                        });
                    }
                >
                    "Refresh All"
                </button>
                
                <button
                    on:click=move |_| {
                        let query = infinite_query.clone();
                        spawn_local(async move {
                            let _ = query.invalidate().await;
                        });
                    }
                >
                    "Invalidate & Refetch"
                </button>
                
                <button
                    on:click=move |_| {
                        let query = infinite_query.clone();
                        spawn_local(async move {
                            let _ = query.remove().await;
                        });
                    }
                >
                    "Clear Cache"
                </button>
            </div>
            
            // Statistics
            <div class="stats">
                <p>
                    <strong>"Total Posts: "</strong>
                    {move || infinite_query.get_total_count()}
                </p>
                <p>
                    <strong>"Pages Loaded: "</strong>
                    {move || posts.get().len()}
                </p>
                <p>
                    <strong>"Current Page: "</strong>
                    {move || current_page.get() + 1}
                </p>
            </div>
        </div>
    }
}

/// Main app component
#[component]
fn App() -> impl IntoView {
    view! {
        <QueryClientProvider>
            <div class="app">
                <h1>"Leptos Query - Infinite Queries Example"</h1>
                <InfinitePosts/>
            </div>
        </QueryClientProvider>
    }
}

/// Main function for the example
fn main() {
    mount_to_body(|| view! { <App/> });
}

/// CSS styles for the example
#[cfg(target_arch = "wasm32")]
mod styles {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(s: &str);
    }
    
    pub fn inject_styles() {
        let styles = r#"
            .infinite-posts {
                max-width: 800px;
                margin: 0 auto;
                padding: 20px;
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            }
            
            .post-item {
                border: 1px solid #e1e5e9;
                border-radius: 8px;
                padding: 20px;
                margin-bottom: 20px;
                background: white;
                box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            }
            
            .post-item h3 {
                margin: 0 0 10px 0;
                color: #2c3e50;
            }
            
            .post-meta {
                font-size: 14px;
                color: #7f8c8d;
                margin-bottom: 15px;
            }
            
            .post-meta .author {
                margin-right: 15px;
            }
            
            .post-content {
                line-height: 1.6;
                color: #34495e;
            }
            
            .navigation {
                display: flex;
                justify-content: center;
                align-items: center;
                gap: 20px;
                margin: 30px 0;
            }
            
            .navigation button {
                padding: 10px 20px;
                border: 1px solid #3498db;
                border-radius: 5px;
                background: #3498db;
                color: white;
                cursor: pointer;
                transition: all 0.3s ease;
            }
            
            .navigation button:hover:not(:disabled) {
                background: #2980b9;
                border-color: #2980b9;
            }
            
            .navigation button:disabled {
                opacity: 0.5;
                cursor: not-allowed;
            }
            
            .page-info {
                font-weight: bold;
                color: #2c3e50;
            }
            
            .actions {
                display: flex;
                justify-content: center;
                gap: 15px;
                margin: 20px 0;
            }
            
            .actions button {
                padding: 8px 16px;
                border: 1px solid #95a5a6;
                border-radius: 4px;
                background: #ecf0f1;
                color: #2c3e50;
                cursor: pointer;
                transition: all 0.3s ease;
            }
            
            .actions button:hover {
                background: #bdc3c7;
                border-color: #7f8c8d;
            }
            
            .stats {
                background: #f8f9fa;
                padding: 20px;
                border-radius: 8px;
                margin-top: 30px;
            }
            
            .stats p {
                margin: 5px 0;
                color: #2c3e50;
            }
            
            .loading {
                text-align: center;
                padding: 20px;
                color: #7f8c8d;
                font-style: italic;
            }
            
            .error {
                background: #fee;
                border: 1px solid #fcc;
                border-radius: 4px;
                padding: 15px;
                margin: 20px 0;
                color: #c33;
            }
        "#;
        
        // In a real app, you'd inject this into the DOM
        log("Styles loaded for infinite queries example");
    }
}
