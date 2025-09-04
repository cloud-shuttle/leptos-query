use leptos::*;
use leptos_query::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Example data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Post {
    id: u32,
    title: String,
    body: String,
    user_id: u32,
}

/// Mock API function
async fn fetch_posts() -> Result<Vec<Post>, String> {
    // Simulate network delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Simulate some posts
    Ok(vec![
        Post {
            id: 1,
            title: "First Post".to_string(),
            body: "This is the first post content.".to_string(),
            user_id: 1,
        },
        Post {
            id: 2,
            title: "Second Post".to_string(),
            body: "This is the second post content.".to_string(),
            user_id: 2,
        },
    ])
}

/// Mock mutation function
async fn create_post(title: String, body: String) -> Result<Post, String> {
    // Simulate network delay
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    Ok(Post {
        id: 3,
        title,
        body,
        user_id: 1,
    })
}

/// DevTools panel component
#[component]
fn DevToolsPanel() -> impl IntoView {
    let client = use_query_client();
    let devtools = use_context::<DevToolsManager>().expect("DevTools not found");
    
    let (show_metrics, set_show_metrics) = create_signal(false);
    let (show_network, set_show_network) = create_signal(false);
    let (show_cache, set_show_cache) = create_signal(false);
    let (show_events, set_show_events) = create_signal(false);
    
    let metrics = create_memo(move |_| {
        if show_metrics.get() {
            devtools.get_query_metrics()
        } else {
            Vec::new()
        }
    });
    
    let network_history = create_memo(move |_| {
        if show_network.get() {
            devtools.get_network_history()
        } else {
            Vec::new()
        }
    });
    
    let cache_history = create_memo(move |_| {
        if show_cache.get() {
            devtools.get_cache_history()
        } else {
            Vec::new()
        }
    });
    
    let event_history = create_memo(move |_| {
        if show_events.get() {
            devtools.get_event_history()
        } else {
            Vec::new()
        }
    });
    
    let active_queries = create_memo(move |_| devtools.get_active_queries());
    let cache_stats = create_memo(move |_| devtools.get_cache_stats(&client));
    
    let clear_history = move |_| {
        devtools.clear_history();
    };
    
    let export_data = move |_| {
        let data = devtools.export_data();
        // In a real app, you might save this to a file or send to an external tool
        log::info!("Exported DevTools data: {:?} events", data.event_history.len());
    };
    
    view! {
        <div class="devtools-panel">
            <h3>"DevTools Panel"</h3>
            
            <div class="devtools-controls">
                <button on:click=move |_| set_show_metrics.set(!show_metrics.get())>
                    {move || if show_metrics.get() { "Hide" } else { "Show" } } " Metrics"
                </button>
                <button on:click=move |_| set_show_network.set(!show_network.get())>
                    {move || if show_network.get() { "Hide" } else { "Show" } } " Network"
                </button>
                <button on:click=move |_| set_show_cache.set(!show_cache.get())>
                    {move || if show_metrics.get() { "Hide" } else { "Show" } } " Cache"
                </button>
                <button on:click=move |_| set_show_events.set(!show_events.get())>
                    {move || if show_events.get() { "Hide" } else { "Show" } } " Events"
                </button>
                <button on:click=clear_history>"Clear History"</button>
                <button on:click=export_data>"Export Data"</button>
            </div>
            
            <div class="devtools-content">
                // Active Queries
                <div class="section">
                    <h4>"Active Queries"</h4>
                    <p>"Count: " {move || active_queries.get().len()}</p>
                </div>
                
                // Cache Stats
                <div class="section">
                    <h4>"Cache Statistics"</h4>
                    <p>"Total Entries: " {move || cache_stats.get().total_entries}</p>
                    <p>"Stale Entries: " {move || cache_stats.get().stale_entries}</p>
                    <p>"Total Size: " {move || cache_stats.get().total_size} " bytes"</p>
                </div>
                
                // Query Metrics
                {move || if show_metrics.get() {
                    view! {
                        <div class="section">
                            <h4>"Query Metrics"</h4>
                            <div class="metrics-list">
                                {move || metrics.get().into_iter().map(|metric| {
                                    view! {
                                        <div class="metric-item">
                                            <strong>"Key: " {metric.key}</strong><br/>
                                            "Executions: " {metric.execution_count}<br/>
                                            "Success Rate: " {format!("{:.1}%", metric.success_count as f64 / metric.execution_count as f64 * 100.0)}<br/>
                                            "Avg Time: " {format!("{:.2}ms", metric.avg_time.as_millis() as f64)}<br/>
                                            "Cache Hit Rate: " {format!("{:.1}%", metric.cache_hit_rate * 100.0)}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }}
                
                // Network History
                {move || if show_network.get() {
                    view! {
                        <div class="section">
                            <h4>"Network History"</h4>
                            <div class="network-list">
                                {move || network_history.get().into_iter().map(|request| {
                                    view! {
                                        <div class="network-item">
                                            <strong>"URL: " {request.url}</strong><br/>
                                            "Method: " {request.method}<br/>
                                            "Status: " {request.status.map(|s| s.to_string()).unwrap_or_else(|| "Pending".to_string())}<br/>
                                            "Duration: " {request.duration.map(|d| format!("{:.2}ms", d.as_millis() as f64)).unwrap_or_else(|| "Pending".to_string())}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }}
                
                // Cache History
                {move || if show_cache.get() {
                    view! {
                        <div class="section">
                            <h4>"Cache History"</h4>
                            <div class="cache-list">
                                {move || cache_history.get().into_iter().map(|op| {
                                    match op {
                                        CacheOperation::Set { key, size, timestamp } => {
                                            view! {
                                                <div class="cache-item">
                                                    <strong>"SET"</strong> " - " {key} " (" {size} " bytes) at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                        CacheOperation::Get { key, hit, timestamp } => {
                                            view! {
                                                <div class="cache-item">
                                                    <strong>{if hit { "HIT" } else { "MISS" }}</strong> " - " {key} " at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                        CacheOperation::Remove { key, timestamp } => {
                                            view! {
                                                <div class="cache-item">
                                                    <strong>"REMOVE"</strong> " - " {key} " at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                        CacheOperation::Clear { timestamp } => {
                                            view! {
                                                <div class="cache-item">
                                                    <strong>"CLEAR"</strong> " at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                        CacheOperation::Expire { key, timestamp } => {
                                            view! {
                                                <div class="cache-item">
                                                    <strong>"EXPIRE"</strong> " - " {key} " at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }}
                
                // Event History
                {move || if show_events.get() {
                    view! {
                        <div class="section">
                            <h4>"Event History"</h4>
                            <div class="event-list">
                                {move || event_history.get().into_iter().map(|event| {
                                    match event {
                                        DevToolsEvent::QueryStart { key, timestamp } => {
                                            view! {
                                                <div class="event-item query-start">
                                                    <strong>"QUERY START"</strong> " - " {key} " at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                        DevToolsEvent::QueryComplete { key, success, duration, timestamp } => {
                                            view! {
                                                <div class="event-item query-complete">
                                                    <strong>"QUERY COMPLETE"</strong> " - " {key} " (" {if success { "SUCCESS" } else { "ERROR" } }) " 
                                                    "in " {format!("{:.2}ms ", duration.as_millis() as f64)} " at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                        DevToolsEvent::CacheOp { operation } => {
                                            view! {
                                                <div class="event-item cache-op ">
                                                    <strong>"CACHE OP "</strong> " - " {format!("{:?}", operation)}
                                                </div>
                                            }
                                        }
                                        DevToolsEvent::NetworkRequest { request } => {
                                            view! {
                                                <div class="event-item network-request">
                                                    <strong>"NETWORK"</strong> " - " {request.method} " " {request.url}
                                                </div>
                                            }
                                        }
                                        DevToolsEvent::OptimisticUpdate { key, update_id, timestamp } => {
                                            view! {
                                                <div class="event-item optimistic-update">
                                                    <strong>"OPTIMISTIC UPDATE"</strong> " - " {key} " (" {update_id} ") at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                        DevToolsEvent::OptimisticConfirm { key, update_id, timestamp } => {
                                            view! {
                                                <div class="event-item optimistic-confirm">
                                                    <strong>"OPTIMISTIC CONFIRM"</strong> " - " {key} " (" {update_id} ") at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                        DevToolsEvent::OptimisticRollback { key, update_id, timestamp } => {
                                            view! {
                                                <div class="event-item optimistic-rollback">
                                                    <strong>"OPTIMISTIC ROLLBACK"</strong> " - " {key} " (" {update_id} ") at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                        DevToolsEvent::PersistenceOp { operation, key, timestamp } => {
                                            view! {
                                                <div class="event-item persistence-op">
                                                    <strong>"PERSISTENCE"</strong> " - " {operation} 
                                                    {move || key.as_ref().map(|k| format!(" ({})", k)).unwrap_or_default()}
                                                    " at " {format!("{:?}", timestamp)}
                                                </div>
                                            }
                                        }
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }}
            </div>
        </div>
    }
}

/// Main app component
#[component]
fn App() -> impl IntoView {
    // Create DevTools manager
    let devtools = DevToolsManager::new(DevToolsConfig::default());
    
    // Provide DevTools context
    provide_context(devtools.clone());
    
    // Query for posts
    let posts_query = use_query(
        || "posts".to_string(),
        |_| fetch_posts(),
        QueryOptions::default()
            .stale_time(Duration::from_secs(30))
            .cache_time(Duration::from_secs(5 * 60)),
    );
    
    // Mutation for creating posts
    let create_post_mutation = use_mutation(
        |input: (String, String)| create_post(input.0, input.1),
        MutationOptions::default(),
    );
    
    let (title, set_title) = create_signal(String::new());
    let (body, set_body) = create_signal(String::new());
    
    let handle_submit = move |_| {
        let title_val = title.get();
        let body_val = body.get();
        if !title_val.is_empty() && !body_val.is_empty() {
            create_post_mutation.mutate((title_val, body_val));
            set_title.set(String::new());
            set_body.set(String::new());
        }
    };
    
    view! {
        <div class="app">
            <h1>"Leptos Query - DevTools Example"</h1>
            
            <div class="main-content">
                <div class="posts-section">
                    <h2>"Posts"</h2>
                    
                    {move || match posts_query.get() {
                        QueryResult::Loading => view! { <div>"Loading posts..."</div> },
                        QueryResult::Error(error) => view! { <div class="error">"Error: " {error}</div> },
                        QueryResult::Success(posts) => view! {
                            <div class="posts-list">
                                {posts.into_iter().map(|post| {
                                    view! {
                                        <div class="post-item">
                                            <h3>{post.title}</h3>
                                            <p>{post.body}</p>
                                            <small>"User ID: " {post.user_id}</small>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        },
                        _ => view! { <div></div> },
                    }}
                    
                    <button on:click=move |_| posts_query.refetch()>
                        "Refresh Posts"
                    </button>
                </div>
                
                <div class="form-section">
                    <h2>"Create New Post"</h2>
                    <div class="form">
                        <input
                            type="text"
                            placeholder="Title"
                            prop:value=title
                            on:input=move |ev| set_title.set(event_target_value(&ev))
                        />
                        <textarea
                            placeholder="Body"
                            prop:value=body
                            on:input=move |ev| set_body.set(event_target_value(&ev))
                        />
                        <button 
                            on:click=handle_submit
                            disabled=move || create_post_mutation.is_pending()
                        >
                            {move || if create_post_mutation.is_pending() { "Creating..." } else { "Create Post" }}
                        </button>
                    </div>
                    
                    {move || if let Some(result) = create_post_mutation.get() {
                        match result {
                            Ok(post) => view! { <div class="success">"Post created: " {post.title}</div> },
                            Err(error) => view! { <div class="error">"Error: " {error}</div> },
                        }
                    } else {
                        view! { <div></div> }
                    }}
                </div>
            </div>
            
            <DevToolsPanel />
        </div>
    }
}

/// Main function
fn main() {
    _ = console_log::init_with_level(log::Level::Info);
    console_error_panic_hook::set_once();
    
    mount_to_body(|| view! { <App /> })
}
