use leptos::*;
use leptos_query::*;
use leptos_query::retry::{QueryError, RetryConfig, RetryDelay};
use leptos_query::types::{QueryStatus, MutationStatus};
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Post {
    id: u32,
    title: String,
    content: String,
    user_id: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct CreatePostRequest {
    title: String,
    content: String,
    user_id: u32,
}

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

// Mock API functions
async fn fetch_user(id: u32) -> Result<User, QueryError> {
    // Simulate network delay
    sleep(Duration::from_millis(50)).await;
    
    if id == 0 {
        return Err(QueryError::http(404, "User not found"));
    }
    
    Ok(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

async fn fetch_user_posts(user_id: u32) -> Result<Vec<Post>, QueryError> {
    sleep(Duration::from_millis(30)).await;
    
    if user_id == 0 {
        return Err(QueryError::http(404, "User not found"));
    }
    
    Ok(vec![
        Post {
            id: 1,
            title: "First Post".to_string(),
            content: "Content of first post".to_string(),
            user_id,
        },
        Post {
            id: 2,
            title: "Second Post".to_string(),
            content: "Content of second post".to_string(),
            user_id,
        },
    ])
}

async fn create_user(request: CreateUserRequest) -> Result<User, QueryError> {
    sleep(Duration::from_millis(100)).await;
    
    if request.email.contains("error") {
        return Err(QueryError::custom("Invalid email"));
    }
    
    Ok(User {
        id: 999, // Mock ID
        name: request.name,
        email: request.email,
    })
}

async fn create_post(request: CreatePostRequest) -> Result<Post, QueryError> {
    sleep(Duration::from_millis(80)).await;
    
    if request.title.is_empty() {
        return Err(QueryError::custom("Title cannot be empty"));
    }
    
    Ok(Post {
        id: 888, // Mock ID
        title: request.title,
        content: request.content,
        user_id: request.user_id,
    })
}

async fn fetch_with_retry() -> Result<String, QueryError> {
    static mut CALL_COUNT: u32 = 0;
    
    unsafe {
        CALL_COUNT += 1;
        if CALL_COUNT < 3 {
            return Err(QueryError::network("Temporary network error"));
        }
    }
    
    Ok("Success after retries".to_string())
}

// Test component for query functionality
#[component]
fn TestQueryComponent() -> impl IntoView {
    let user_query = use_query(
        || &["users", "1"][..],
        || || async move { fetch_user(1).await },
        QueryOptions::default()
            .with_stale_time(Duration::from_secs(60))
            .with_cache_time(Duration::from_secs(300))
    );
    
    let posts_query = use_query(
        || &["posts", "1"][..],
        || || async move { fetch_user_posts(1).await },
        QueryOptions::default()
    );
    
    let error_query = use_query(
        || &["users", "0"][..],
        || || async move { fetch_user(0).await },
        QueryOptions::default()
    );
    
    view! {
        <div>
            <div>
                <h3>"User Query Test"</h3>
                {move || match user_query.status.get() {
                    QueryStatus::Loading => view! { <div>"Loading user..."</div> }.into_view(),
                    QueryStatus::Success => {
                        if let Some(user) = user_query.data.get() {
                            view! { 
                                <div>
                                    <p>"User: " {user.name}</p>
                                    <p>"Email: " {user.email}</p>
                                </div>
                            }.into_view()
                        } else {
                            view! { <div>"No user data"</div> }.into_view()
                        }
                    }
                    QueryStatus::Error => {
                        if let Some(error) = user_query.error.get() {
                            view! { <div>"Error: " {format!("{:?}", error)}</div> }.into_view()
                        } else {
                            view! { <div>"Unknown error"</div> }.into_view()
                        }
                    }
                    _ => view! { <div>"Idle"</div> }.into_view(),
                }}
            </div>
            
            <div>
                <h3>"Posts Query Test"</h3>
                {move || match posts_query.status.get() {
                    QueryStatus::Loading => view! { <div>"Loading posts..."</div> }.into_view(),
                    QueryStatus::Success => {
                        if let Some(posts) = posts_query.data.get() {
                            view! { 
                                <div>
                                    {posts.iter().map(|post| view! {
                                        <div>
                                            <h4>{post.title.clone()}</h4>
                                            <p>{post.content.clone()}</p>
                                        </div>
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_view()
                        } else {
                            view! { <div>"No posts data"</div> }.into_view()
                        }
                    }
                    _ => view! { <div>"Posts not loaded"</div> }.into_view(),
                }}
            </div>
            
            <div>
                <h3>"Error Query Test"</h3>
                {move || match error_query.status.get() {
                    QueryStatus::Loading => view! { <div>"Loading (should fail)..."</div> }.into_view(),
                    QueryStatus::Error => {
                        if let Some(error) = error_query.error.get() {
                            view! { <div>"Expected error: " {format!("{:?}", error)}</div> }.into_view()
                        } else {
                            view! { <div>"Unknown error"</div> }.into_view()
                        }
                    }
                    _ => view! { <div>"Error query idle"</div> }.into_view(),
                }}
            </div>
        </div>
    }
}

// Test component for mutation functionality
#[component]
fn TestMutationComponent() -> impl IntoView {
    let create_user_mutation = use_mutation::<User, CreateUserRequest, (), _, _>(
        |request: CreateUserRequest| async move { create_user(request).await },
        MutationOptions::default()
    );
    
    let create_post_mutation = use_mutation::<Post, CreatePostRequest, (), _, _>(
        |request: CreatePostRequest| async move { create_post(request).await },
        MutationOptions::default()
    );
    
    let (user_name, set_user_name) = create_signal("".to_string());
    let (user_email, set_user_email) = create_signal("".to_string());
    let (post_title, set_post_title) = create_signal("".to_string());
    let (post_content, set_post_content) = create_signal("".to_string());
    
    let handle_create_user = {
        let create_user_mutation = create_user_mutation.clone();
        move |_| {
            let request = CreateUserRequest {
                name: user_name.get(),
                email: user_email.get(),
            };
            create_user_mutation.mutate.call(request);
        }
    };
    
    let handle_create_post = {
        let create_post_mutation = create_post_mutation.clone();
        move |_| {
            let request = CreatePostRequest {
                title: post_title.get(),
                content: post_content.get(),
                user_id: 1,
            };
            create_post_mutation.mutate.call(request);
        }
    };
    
    view! {
        <div>
            <h3>"Mutation Tests"</h3>
            
            <div>
                <h4>"Create User"</h4>
                <input 
                    placeholder="User name"
                    on:input=move |ev| set_user_name.set(event_target_value(&ev))
                />
                <input 
                    placeholder="User email"
                    on:input=move |ev| set_user_email.set(event_target_value(&ev))
                />
                <button on:click=handle_create_user disabled=move || create_user_mutation.is_loading.get()>
                    {move || if create_user_mutation.is_loading.get() { "Creating..." } else { "Create User" }}
                </button>
                
                {move || match create_user_mutation.status.get() {
                    MutationStatus::Success => {
                        if let Some(user) = create_user_mutation.data.get() {
                            view! { <div>"Created user: " {user.name.clone()}</div> }.into_view()
                        } else {
                            view! { <div>"User created successfully"</div> }.into_view()
                        }
                    }
                    MutationStatus::Error => {
                        if let Some(error) = create_user_mutation.error.get() {
                            view! { <div>"Error: " {format!("{:?}", error)}</div> }.into_view()
                        } else {
                            view! { <div>"Unknown error"</div> }.into_view()
                        }
                    }
                    _ => view! { <div></div> }.into_view(),
                }}
            </div>
            
            <div>
                <h4>"Create Post"</h4>
                <input 
                    placeholder="Post title"
                    on:input=move |ev| set_post_title.set(event_target_value(&ev))
                />
                <input 
                    placeholder="Post content"
                    on:input=move |ev| set_post_content.set(event_target_value(&ev))
                />
                <button on:click=handle_create_post disabled=move || create_post_mutation.is_loading.get()>
                    {move || if create_post_mutation.is_loading.get() { "Creating..." } else { "Create Post" }}
                </button>
                
                {move || match create_post_mutation.status.get() {
                    MutationStatus::Success => {
                        if let Some(post) = create_post_mutation.data.get() {
                            view! { <div>"Created post: " {post.title.clone()}</div> }.into_view()
                        } else {
                            view! { <div>"Post created successfully"</div> }.into_view()
                        }
                    }
                    MutationStatus::Error => {
                        if let Some(error) = create_post_mutation.error.get() {
                            view! { <div>"Error: " {format!("{:?}", error)}</div> }.into_view()
                        } else {
                            view! { <div>"Unknown error"</div> }.into_view()
                        }
                    }
                    _ => view! { <div></div> }.into_view(),
                }}
            </div>
        </div>
    }
}

// Test component for retry functionality
#[component]
fn TestRetryComponent() -> impl IntoView {
    let retry_query = use_query(
        || &["retry-test"][..],
        || || async move { fetch_with_retry().await },
        QueryOptions::default()
            .with_retry(RetryConfig {
                max_attempts: 3,
                delay: RetryDelay::Exponential {
                    initial: Duration::from_millis(100),
                    multiplier: 2.0,
                    max: Duration::from_secs(1),
                },
                jitter: false,
            })
    );
    
    view! {
        <div>
            <h3>"Retry Test"</h3>
            {move || match retry_query.status.get() {
                QueryStatus::Loading => view! { <div>"Loading with retries..."</div> }.into_view(),
                QueryStatus::Success => {
                    if let Some(result) = retry_query.data.get() {
                        view! { <div>"Success: " {result}</div> }.into_view()
                    } else {
                        view! { <div>"No data"</div> }.into_view()
                    }
                }
                QueryStatus::Error => {
                    if let Some(error) = retry_query.error.get() {
                        view! { <div>"Failed after retries: " {format!("{:?}", error)}</div> }.into_view()
                    } else {
                        view! { <div>"Unknown error"</div> }.into_view()
                    }
                }
                _ => view! { <div>"Idle"</div> }.into_view(),
            }}
        </div>
    }
}

// Test component for cache functionality
#[component]
fn TestCacheComponent() -> impl IntoView {
    let (user_id, set_user_id) = create_signal(1);
    
    let user_query = use_query(
        move || {
            let id = user_id.get();
            QueryKey::new(&["cache-test", &id.to_string()])
        },
        move || {
            let id = user_id.get();
            move || async move { fetch_user(id).await }
        },
        QueryOptions::default()
            .with_stale_time(Duration::from_secs(5))
            .with_cache_time(Duration::from_secs(10))
    );
    
    let (refetch_count, set_refetch_count) = create_signal(0);
    
    let handle_refetch = {
        let user_query = user_query.clone();
        move |_| {
            set_refetch_count.update(|count| *count += 1);
            user_query.refetch.call(());
        }
    };
    
    let handle_invalidate = {
        let user_query = user_query.clone();
        move |_| {
            user_query.invalidate.call(());
        }
    };
    
    view! {
        <div>
            <h3>"Cache Test"</h3>
            
            <div>
                <button on:click=move |_| set_user_id.update(|id| *id += 1)>"Change User ID"</button>
                <button on:click=handle_refetch>"Refetch"</button>
                <button on:click=handle_invalidate>"Invalidate"</button>
                <p>"Refetch count: " {refetch_count}</p>
            </div>
            
            {move || match user_query.status.get() {
                QueryStatus::Loading => view! { <div>"Loading user..."</div> }.into_view(),
                QueryStatus::Success => {
                    if let Some(user) = user_query.data.get() {
                        view! { 
                            <div>
                                <p>"User ID: " {user_id}</p>
                                <p>"User: " {user.name}</p>
                                <p>"Stale: " {if user_query.is_stale.get() { "Yes" } else { "No" }}</p>
                                <p>"Fetching: " {if user_query.is_fetching.get() { "Yes" } else { "No" }}</p>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div>"No user data"</div> }.into_view()
                    }
                }
                _ => view! { <div>"User not loaded"</div> }.into_view(),
            }}
        </div>
    }
}

// Main test app
#[component]
fn TestApp() -> impl IntoView {
    view! {
        <div>
            <h1>"Leptos Query Integration Tests"</h1>
            
            <TestQueryComponent />
            <TestMutationComponent />
            <TestRetryComponent />
            <TestCacheComponent />
        </div>
    }
}



// Test runner function
pub fn run_integration_tests() {
    mount_to_body(|| {
        view! {
            <QueryClientProvider>
                <TestApp />
            </QueryClientProvider>
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_key_creation() {
        let key1 = QueryKey::new(&["users", "1"]);
        let key2 = QueryKey::from_parts(&["users", "1"]).unwrap();
        let key3: QueryKey = ("1",).into();
        
        assert_eq!(key1.segments, vec!["users", "1"]);
        assert_eq!(key2.segments, vec!["\"users\"", "\"1\""]); // from_parts serializes to JSON
        assert_eq!(key3.segments, vec!["1"]);
    }
    
    #[test]
    fn test_query_key_pattern_matching() {
        let key = QueryKey::new(&["users", "1", "posts"]);
        
        assert!(key.matches_pattern(&QueryKeyPattern::Exact(key.clone())));
        assert!(key.matches_pattern(&QueryKeyPattern::Prefix(QueryKey::new(&["users"]))));
        assert!(key.matches_pattern(&QueryKeyPattern::Contains("posts".to_string())));
        assert!(!key.matches_pattern(&QueryKeyPattern::Contains("comments".to_string())));
    }
    
    #[test]
    fn test_serialized_data() {
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let serialized = SerializedData::serialize(&user).unwrap();
        let deserialized: User = serialized.deserialize().unwrap();
        
        assert_eq!(user, deserialized);
    }
    
    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        
        let custom_config = RetryConfig {
            max_attempts: 5,
            delay: RetryDelay::Fixed(Duration::from_secs(1)),
            jitter: false,
        };
        
        assert_eq!(custom_config.max_attempts, 5);
    }
    
    #[test]
    fn test_query_options_builder() {
        let options = QueryOptions::default()
            .with_stale_time(Duration::from_secs(60))
            .with_cache_time(Duration::from_secs(300))
            .keep_previous_data()
            .with_suspense();
        
        assert_eq!(options.stale_time, Duration::from_secs(60));
        assert_eq!(options.cache_time, Duration::from_secs(300));
        assert!(options.keep_previous_data);
        assert!(options.suspense);
    }
    
    #[test]
    fn test_mutation_options() {
        let options: MutationOptions<User, CreateUserRequest, ()> = MutationOptions::default();
        
        assert_eq!(options.invalidates.len(), 0);
        assert!(!options.throw_on_error);
    }
    
    #[test]
    fn test_error_types() {
        let network_error = QueryError::network("connection failed");
        let http_error = QueryError::http(500, "server error");
        let timeout_error = QueryError::timeout(5000);
        let custom_error = QueryError::custom("validation failed");
        
        assert!(network_error.is_retryable());
        assert!(http_error.is_retryable());
        assert!(timeout_error.is_retryable());
        assert!(!custom_error.is_retryable());
    }
}
