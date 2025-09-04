use leptos::prelude::*;
use leptos_query_rs::*;
use serde::{Deserialize, Serialize};

/// Example data structure for paginated posts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Post {
    id: usize,
    title: String,
    content: String,
}

// Mock API function removed for simplified example

/// Component demonstrating infinite queries
#[component]
fn InfinitePosts() -> impl IntoView {
    // For now, let's use a simpler approach that demonstrates the concept
    // without the complex type inference issues
    let posts = RwSignal::new(Vec::new());
    let current_page = RwSignal::new(0);
    let has_next = RwSignal::new(true);
    let has_prev = RwSignal::new(false);
    let is_loading = RwSignal::new(false);
    let error = RwSignal::new(None);

    // Initialize with some sample data
    posts.set(vec![
        Post { id: 1, title: "Sample Post 1".to_string(), content: "This is sample content 1".to_string() },
        Post { id: 2, title: "Sample Post 2".to_string(), content: "This is sample content 2".to_string() },
        Post { id: 3, title: "Sample Post 3".to_string(), content: "This is sample content 3".to_string() },
    ]);

    view! {
        <div>
            <h2>"Infinite Posts Example"</h2>
            
            // Error display
            {move || error.get().map(|e: &QueryError| view! {
                <div style="color: red;">
                    <strong>"Error: "</strong>
                    {e.to_string()}
                </div>
            })}
            
            // Posts list
            <div>
                {move || posts.get().into_iter().map(|post| {
                    view! {
                        <div style="border: 1px solid #ccc; margin: 10px; padding: 10px;">
                            <h3>{post.title}</h3>
                            <p>{post.content}</p>
                        </div>
                    }
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
                        if current_page.get() > 0 {
                            current_page.set(current_page.get() - 1);
                        }
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
                        current_page.set(current_page.get() + 1);
                    }
                >
                    "Next Page →"
                </button>
            </div>
            
            // Statistics
            <div style="background: #f0f0f0; padding: 20px; margin-top: 20px;">
                <p><strong>"Total Posts: "</strong>{move || posts.get().len()}</p>
                <p><strong>"Current Page: "</strong>{move || current_page.get() + 1}</p>
                <p><strong>"Sample Data: "</strong>"This example shows the UI structure for infinite queries"</p>
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
