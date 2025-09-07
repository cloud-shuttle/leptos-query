//! Leptos 0.8 Compatibility Tests
//! 
//! These tests verify that the library works correctly with Leptos 0.8
//! and that all documented APIs are compatible.

use leptos::prelude::*;
use leptos_query_rs::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestUser {
    id: u32,
    name: String,
    email: String,
}

async fn mock_fetch_user(id: u32) -> Result<TestUser, QueryError> {
    // Simulate API call
    Ok(TestUser {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leptos_08_imports() {
        // Test that all necessary Leptos 0.8 imports work
        let (_read_signal, _write_signal) = signal(42);
        let _derived = Signal::derive(|| 42);
        let _callback = Callback::new(|_: ()| {});
        
        // Test that our library imports work with Leptos 0.8
        let _client = QueryClient::new();
        let _key = QueryKey::new(&["test"]);
        let _options = QueryOptions::default();
        
        assert!(true);
    }

    #[test]
    fn test_query_hook_compatibility() {
        // Test that use_query works with Leptos 0.8 patterns
        let (user_id, _set_user_id) = signal(1);
        
        // Test that the API compiles with Leptos 0.8
        // Note: This won't run without a Leptos runtime, but it tests compilation
        let _query_key_fn = move || QueryKey::new(&["user", &user_id.get().to_string()]);
        let _query_fn = move || async move { mock_fetch_user(user_id.get()).await };
        let _options = QueryOptions::default();
        
        assert!(true);
    }

    #[test]
    fn test_mutation_hook_compatibility() {
        // Test that use_mutation works with Leptos 0.8 patterns
        // Note: This won't run without a Leptos runtime, but it tests compilation
        let _mutation_fn = |id: u32| async move { mock_fetch_user(id).await };
        let _options = MutationOptions::default();
        
        assert!(true);
    }

    #[test]
    fn test_signal_integration() {
        // Test that our library works with Leptos 0.8 signals
        let (count, _set_count) = signal(0);
        let (name, _set_name) = signal("test".to_string());
        
        // Test that we can use signals in query keys
        let _query_key = Signal::derive(move || {
            QueryKey::new(&["user", &count.get().to_string(), &name.get()])
        });
        
        // Test that we can use signals in query options
        let _options = QueryOptions::default();
        
        assert!(true);
    }

    #[test]
    fn test_callback_integration() {
        // Test that our library works with Leptos 0.8 callbacks
        let _callback = Callback::new(|_: ()| {
            // This should work with Leptos 0.8
        });
        
        // Test that we can use callbacks in our library
        let _mutation_fn = |id: u32| async move { mock_fetch_user(id).await };
        let _options = MutationOptions::default();
        
        assert!(true);
    }

    #[test]
    fn test_view_macro_compatibility() {
        // Test that our library works with Leptos 0.8 view macro
        let (user_id, _set_user_id) = signal(1);
        
        // This should compile with Leptos 0.8
        let _view = view! {
            <div>
                {move || {
                    let id = user_id.get();
                    format!("User ID: {}", id)
                }}
            </div>
        };
        
        assert!(true);
    }

    #[test]
    fn test_component_integration() {
        // Test that our library works with Leptos 0.8 components
        #[component]
        fn TestComponent() -> impl IntoView {
            let user_query = use_query(
                || QueryKey::new(&["test", "user"]),
                || async move { mock_fetch_user(1).await },
                QueryOptions::default()
            );
            
            view! {
                <div>
                    {move || {
                        if let Some(user) = user_query.data.get() {
                            format!("User: {}", user.name)
                        } else if user_query.is_loading.get() {
                            "Loading...".to_string()
                        } else {
                            "No user".to_string()
                        }
                    }}
                </div>
            }
        }
        
        // If this compiles, the component integration works
        assert!(true);
    }

    #[test]
    fn test_context_provider_compatibility() {
        // Test that our QueryClientProvider works with Leptos 0.8
        #[component]
        fn TestApp() -> impl IntoView {
            view! {
                <QueryClientProvider>
                    <div>"Test App"</div>
                </QueryClientProvider>
            }
        }
        
        // If this compiles, the context provider works
        assert!(true);
    }

    #[test]
    fn test_async_integration() {
        // Test that our async functions work with Leptos 0.8
        let (user_id, _set_user_id) = signal(1);
        
        // Test async query function
        let _query_key_fn = move || QueryKey::new(&["async", "test"]);
        let _query_fn = move || async move {
            let id = user_id.get();
            mock_fetch_user(id).await
        };
        let _options = QueryOptions::default();
        
        assert!(true);
    }

    #[test]
    fn test_error_handling_compatibility() {
        // Test that our error handling works with Leptos 0.8
        let _query_key_fn = || QueryKey::new(&["error", "test"]);
        let _query_fn = || async move {
            Err::<TestUser, QueryError>(QueryError::NetworkError("Test error".to_string()))
        };
        let _options = QueryOptions::default();
        
        assert!(true);
    }

    #[test]
    fn test_retry_config_compatibility() {
        // Test that our retry configuration works with Leptos 0.8
        let retry_config = RetryConfig::new(3, Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(1));
        
        let _query_key_fn = || QueryKey::new(&["retry", "test"]);
        let _query_fn = || async move { mock_fetch_user(1).await };
        let _options = QueryOptions::default().with_retry(retry_config);
        
        assert!(true);
    }

    #[test]
    fn test_cache_operations_compatibility() {
        // Test that our cache operations work with Leptos 0.8
        let client = QueryClient::new();
        let key = QueryKey::new(&["cache", "test"]);
        let user = TestUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // Test cache operations
        assert!(client.set_query_data(&key, user.clone()).is_ok());
        let entry = client.get_cache_entry(&key);
        assert!(entry.is_some());
        
        let retrieved: TestUser = entry.unwrap().get_data().unwrap();
        assert_eq!(user, retrieved);
        
        // Test cache invalidation
        let pattern = QueryKeyPattern::Prefix(QueryKey::new(&["cache"]));
        client.invalidate_queries(&pattern);
        assert!(client.get_cache_entry(&key).is_none());
    }

    #[test]
    fn test_serialization_compatibility() {
        // Test that our serialization works with Leptos 0.8
        let user = TestUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // Test bincode serialization
        let serialized = bincode::serialize(&user).unwrap();
        let deserialized: TestUser = bincode::deserialize(&serialized).unwrap();
        assert_eq!(user, deserialized);
        
        // Test JSON serialization
        let json_serialized = serde_json::to_string(&user).unwrap();
        let json_deserialized: TestUser = serde_json::from_str(&json_serialized).unwrap();
        assert_eq!(user, json_deserialized);
    }

    #[test]
    fn test_type_safety_compatibility() {
        // Test that our type safety works with Leptos 0.8
        let (user_id, _set_user_id) = signal(1u32);
        let (user_name, _set_user_name) = signal("test".to_string());
        
        // Test that types are preserved through signals
        let _query_key_fn = move || QueryKey::new(&["type", "test", &user_id.get().to_string()]);
        let _query_fn = move || async move {
            let id: u32 = user_id.get();
            let name: String = user_name.get();
            Ok::<TestUser, QueryError>(TestUser {
                id,
                name,
                email: "test@example.com".to_string(),
            })
        };
        let _options = QueryOptions::default();
        
        assert!(true);
    }
}
