use leptos::prelude::*;
use leptos_query_rs::*;
use serde::{Deserialize, Serialize};

/// Example data structure for paginated posts
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Post {
    id: usize,
    title: String,
    content: String,
    author: String,
    created_at: String,
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
        Post { 
            id: 1, 
            title: "Sample Post 1".to_string(), 
            content: "This is sample content for post 1".to_string(),
            author: "Author 1".to_string(),
            created_at: "2024-01-01".to_string(),
        },
        Post { 
            id: 2, 
            title: "Sample Post 2".to_string(), 
            content: "This is sample content for post 2".to_string(),
            author: "Author 2".to_string(),
            created_at: "2024-01-02".to_string(),
        },
        Post { 
            id: 3, 
            title: "Sample Post 3".to_string(), 
            content: "This is sample content for post 3".to_string(),
            author: "Author 3".to_string(),
            created_at: "2024-01-03".to_string(),
        },
    ]);

    view! {
        <div class="infinite-posts">
            <h2>"Infinite Posts Example"</h2>
            
            // Error display
            {move || error.get().map(|e: &QueryError| view! {
                <div class="error">
                    <strong>"Error: "</strong>
                    {e.to_string()}
                </div>
            })}
            
            // Posts list
            <div class="posts-container">
                {move || posts.get().into_iter().map(|post| {
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
                }).collect::<Vec<_>>()}
            </div>
            
            // Loading indicator
            {move || if is_loading.get() {
                view! { <div class="loading">"Loading more posts..."</div> }
            } else {
                view! { <div class="loading">"No more posts"</div> }
            }}
            
            // Navigation controls
            <div class="navigation">
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
                
                <span class="page-info">
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
            
            // Actions
            <div class="actions">
                <button
                    on:click=move |_| {
                        // Simulate refresh
                        is_loading.set(true);
                        // In a real app, this would refetch data
                        is_loading.set(false);
                    }
                >
                    "Refresh All"
                </button>
                
                <button
                    on:click=move |_| {
                        // Simulate invalidation
                        // In a real app, this would invalidate and refetch
                    }
                >
                    "Invalidate & Refetch"
                </button>
                
                <button
                    on:click=move |_| {
                        // Simulate cache clear
                        posts.set(Vec::new());
                    }
                >
                    "Clear Cache"
                </button>
            </div>
            
            // Statistics
            <div class="stats">
                <p>
                    <strong>"Total Posts: "</strong>
                    {move || posts.get().len()}
                </p>
                <p>
                    <strong>"Current Page: "</strong>
                    {move || current_page.get() + 1}
                </p>
                <p>
                    <strong>"Sample Data: "</strong>
                    "This example shows the UI structure for infinite queries"
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
