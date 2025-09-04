use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_query_rs::*;
use serde::{Deserialize, Serialize};

/// Example data structure for paginated posts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Post {
    id: usize,
    title: String,
    content: String,
}

/// Mock API function to fetch posts with pagination
async fn fetch_posts(page: usize) -> Result<Page<Post>, QueryError> {
    // Simulate network delay
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    let per_page = 3;
    let total_posts = 10; // Total posts available
    let start = page * per_page;
    
    // Simulate some posts
    let posts = (start..std::cmp::min(start + per_page, total_posts))
        .map(|i| Post {
            id: i,
            title: format!("Post {}", i + 1),
            content: format!("Content for post {}", i + 1),
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
            .max_pages(Some(5))
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
        <div>
            <h2>"Infinite Posts Example"</h2>
            
            // Error display
            {move || error.get().map(|e| view! {
                <div style="color: red;">
                    <strong>"Error: "</strong>
                    {e.to_string()}
                </div>
            })}
            
            // Posts list
            <div>
                {move || posts.get().into_iter().enumerate().flat_map(|(page_idx, page)| {
                    page.data.into_iter().enumerate().map(move |(item_idx, post)| {
                        let global_idx = page_idx * 3 + item_idx;
                        view! {
                            <div style="border: 1px solid #ccc; margin: 10px; padding: 10px;">
                                <h3>{post.title}</h3>
                                <p>{post.content}</p>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }).collect::<Vec<_>>()}
            </div>
            
            // Loading indicator
            {move || if is_loading.get() {
                view! { <div>"Loading more posts..."</div> }
            } else {
                view! { <div>"No more posts"</div> }
            }}
            
            // Navigation controls
            <div style="margin: 20px;">
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
                
                <span style="margin: 0 20px;">
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
            
            // Statistics
            <div style="background: #f0f0f0; padding: 20px; margin-top: 20px;">
                <p><strong>"Total Posts: "</strong>{move || infinite_query.get_total_count()}</p>
                <p><strong>"Pages Loaded: "</strong>{move || posts.get().len()}</p>
                <p><strong>"Current Page: "</strong>{move || current_page.get() + 1}</p>
            </div>
        </div>
    }
}

/// Main app component
#[component]
fn App() -> impl IntoView {
    view! {
        <QueryClientProvider>
            <div>
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
