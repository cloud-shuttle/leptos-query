use leptos::prelude::*;
use leptos::prelude::{ElementChild, OnAttribute, Get};
use leptos_query_rs::*;
use leptos_query_rs::retry::{QueryError, RetryConfig};
use leptos_query_rs::types::QueryStatus;
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
        return Err(QueryError::GenericError("User not found".to_string()));
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
        return Err(QueryError::GenericError("User not found".to_string()));
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
        return Err(QueryError::GenericError("Invalid email".to_string()));
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
        return Err(QueryError::GenericError("Title cannot be empty".to_string()));
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
            return Err(QueryError::NetworkError("Temporary network error".to_string()));
        }
    }
    
    Ok("Success after retries".to_string())
}

// Test component for query functionality
#[component]
fn TestQueryComponent() -> impl IntoView {
    let user_query = use_query(
        || QueryKey::new(&["users", "1"]),
        || async move { fetch_user(1).await },
        QueryOptions::default()
            .with_stale_time(Duration::from_secs(60))
            .with_cache_time(Duration::from_secs(300))
    );
    
    let posts_query = use_query(
        || QueryKey::new(&["posts", "1"]),
        || async move { fetch_user_posts(1).await },
        QueryOptions::default()
    );
    
    let error_query = use_query(
        || QueryKey::new(&["users", "0"]),
        || async move { fetch_user(0).await },
        QueryOptions::default()
    );
    
    view! {
        <div>
            <div>
                <h3>"User Query Test"</h3>
                {move || {
                    let content = match user_query.status.get() {
                        QueryStatus::Loading => "Loading user...".to_string(),
                        QueryStatus::Success => {
                            if let Some(user) = user_query.data.get() {
                                format!("User: {} Email: {}", user.name, user.email)
                            } else {
                                "No user data".to_string()
                            }
                        }
                        QueryStatus::Error => {
                            if let Some(error) = user_query.error.get() {
                                format!("Error: {:?}", error)
                            } else {
                                "Unknown error".to_string()
                            }
                        }
                        _ => "Idle".to_string(),
                    };
                    view! { <div><p>{content}</p></div> }.into_view()
                }}
            </div>
            
            <div>
                <h3>"Posts Query Test"</h3>
                {move || {
                    let content = match posts_query.status.get() {
                        QueryStatus::Loading => "Loading posts...".to_string(),
                        QueryStatus::Success => {
                            if let Some(posts) = posts_query.data.get() {
                                let posts_content = posts.iter().map(|post| {
                                    format!("Title: {} Content: {}", post.title, post.content)
                                }).collect::<Vec<_>>().join(" | ");
                                posts_content
                            } else {
                                "No posts data".to_string()
                            }
                        }
                        _ => "Posts not loaded".to_string(),
                    };
                    view! { <div><p>{content}</p></div> }.into_view()
                }}
            </div>
            
            <div>
                <h3>"Error Query Test"</h3>
                {move || {
                    let content = match error_query.status.get() {
                        QueryStatus::Loading => "Loading (should fail)...".to_string(),
                        QueryStatus::Error => {
                            if let Some(error) = error_query.error.get() {
                                format!("Expected error: {:?}", error)
                            } else {
                                "Unknown error".to_string()
                            }
                        }
                        _ => "Error query idle".to_string(),
                    };
                    view! { <div><p>{content}</p></div> }.into_view()
                }}
            </div>
        </div>
    }
}

// Test component for mutation functionality
#[component]
fn TestMutationComponent() -> impl IntoView {
    let create_user_mutation = use_mutation::<User, QueryError, CreateUserRequest, _, _>(
        |request: CreateUserRequest| async move { create_user(request).await },
        MutationOptions::default()
    );
    
    let create_post_mutation = use_mutation::<Post, QueryError, CreatePostRequest, _, _>(
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
            create_user_mutation.mutate.run(request);
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
            create_post_mutation.mutate.run(request);
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
                
                {move || {
                    let content = if create_user_mutation.is_success.get() {
                        if let Some(user) = create_user_mutation.data.get() {
                            format!("Created user: {}", user.name.clone())
                        } else {
                            "User created successfully".to_string()
                        }
                    } else if create_user_mutation.is_error.get() {
                        if let Some(error) = create_user_mutation.error.get() {
                            format!("Error: {:?}", error)
                        } else {
                            "Unknown error".to_string()
                        }
                    } else {
                        "Ready to create user".to_string()
                    };
                    view! { <div><p>{content}</p></div> }.into_view()
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
                
                {move || {
                    let content = if create_post_mutation.is_success.get() {
                        if let Some(post) = create_post_mutation.data.get() {
                            format!("Created post: {}", post.title.clone())
                        } else {
                            "Post created successfully".to_string()
                        }
                    } else if create_post_mutation.is_error.get() {
                        if let Some(error) = create_post_mutation.error.get() {
                            format!("Error: {:?}", error)
                        } else {
                            "Unknown error".to_string()
                        }
                    } else {
                        "Ready to create post".to_string()
                    };
                    view! { <div><p>{content}</p></div> }.into_view()
                }}
            </div>
        </div>
    }
}

// Test component for retry functionality
#[component]
fn TestRetryComponent() -> impl IntoView {
    let retry_query = use_query(
        || QueryKey::new(&["retry-test"]),
        || async move { fetch_with_retry().await },
        QueryOptions::default().with_retry(RetryConfig::new(3, Duration::from_millis(100)).with_max_delay(Duration::from_secs(1)))
    );
    
    view! {
        <div>
            <h3>"Retry Test"</h3>
            {move || {
                let content = match retry_query.status.get() {
                    QueryStatus::Loading => "Loading with retries...".to_string(),
                    QueryStatus::Success => {
                        if let Some(result) = retry_query.data.get() {
                            format!("Success: {}", result)
                        } else {
                            "No data".to_string()
                        }
                    }
                    QueryStatus::Error => {
                        if let Some(error) = retry_query.error.get() {
                            format!("Failed after retries: {:?}", error)
                        } else {
                            "Unknown error".to_string()
                        }
                    }
                    _ => "Idle".to_string(),
                };
                view! { <div><p>{content}</p></div> }.into_view()
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
            async move { fetch_user(id).await }
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
            user_query.refetch.run(());
        }
    };
    
    let handle_invalidate = {
        let _user_query = user_query.clone();
        move |_| {
            // invalidate removed in current API; handled via QueryClient in app code
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
            
            {move || {
                let content = match user_query.status.get() {
                    QueryStatus::Loading => "Loading user...".to_string(),
                    QueryStatus::Success => {
                        if let Some(user) = user_query.data.get() {
                            format!("User ID: {} User: {}", user_id.get(), user.name)
                        } else {
                            "No user data".to_string()
                        }
                    }
                    _ => "User not loaded".to_string(),
                };
                view! { <div><p>{content}</p></div> }.into_view()
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
        provide_context(QueryClient::new());
        view! {
            <TestApp />
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
        
        // SerializedData::serialize removed in current API - test cache operations instead
        let client = QueryClient::new();
        let key = QueryKey::new(&["test-user"]);
        assert!(client.set_query_data(&key, user.clone()).is_ok());
        let entry = client.get_cache_entry(&key).unwrap();
        let deserialized: User = entry.get_data().unwrap();
        
        assert_eq!(user, deserialized);
    }
    
    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        
        let custom_config = RetryConfig::new(5, Duration::from_secs(1));
        
        assert_eq!(custom_config.max_retries, 5);
    }
    
    #[test]
    fn test_query_options_builder() {
        let options = QueryOptions::default()
            .with_stale_time(Duration::from_secs(60))
            .with_cache_time(Duration::from_secs(300));
        
        assert_eq!(options.stale_time, Duration::from_secs(60));
        assert_eq!(options.cache_time, Duration::from_secs(300));
        // removed keep_previous_data and suspense in current API
    }
    
    #[test]
    fn test_mutation_options() {
        let options: MutationOptions = MutationOptions::default();
        
        assert!(options.invalidate_queries.is_none());
    }
    
    #[test]
    fn test_error_types() {
        let network_error = QueryError::NetworkError("connection failed".to_string());
        let http_error = QueryError::GenericError("server error".to_string());
        let timeout_error = QueryError::TimeoutError("5000".to_string());
        let custom_error = QueryError::GenericError("validation failed".to_string());
        
        assert!(retry::should_retry_error(&network_error, &RetryConfig::default()));
        assert!(retry::should_retry_error(&timeout_error, &RetryConfig::default()));
        assert!(retry::should_retry_error(&http_error, &RetryConfig::default()));
        assert!(retry::should_retry_error(&custom_error, &RetryConfig::default()));
    }
}
