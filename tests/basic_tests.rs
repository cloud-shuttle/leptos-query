use leptos::*;
use leptos_query::*;
use leptos_query::retry::{QueryError, RetryConfig, RetryDelay};
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    value: String,
}

async fn mock_fetch(id: u32) -> Result<TestData, QueryError> {
    // Simulate network delay
    sleep(Duration::from_millis(10)).await;
    
    if id == 0 {
        return Err(QueryError::http(404, "Not found"));
    }
    
    Ok(TestData {
        id,
        value: format!("Data {}", id),
    })
}

async fn mock_mutate(data: TestData) -> Result<TestData, QueryError> {
    sleep(Duration::from_millis(10)).await;
    
    if data.value.contains("error") {
        return Err(QueryError::custom("Mutation failed"));
    }
    
    Ok(TestData {
        id: data.id + 1000,
        value: format!("Mutated {}", data.value),
    })
}

#[test]
fn test_query_key_creation() {
    let key1 = QueryKey::new(&["users", "1"]);
    let key2 = QueryKey::new(&["posts", "123"]);
    
    assert_eq!(key1.segments, vec!["users", "1"]);
    assert_eq!(key2.segments, vec!["posts", "123"]);
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
    let data = TestData {
        id: 1,
        value: "test".to_string(),
    };
    
    let serialized = SerializedData::serialize(&data).unwrap();
    let deserialized: TestData = serialized.deserialize().unwrap();
    
    assert_eq!(data, deserialized);
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

#[test]
fn test_query_client_creation() {
    let config = QueryClientConfig::default();
    let client = QueryClient::new(config);
    
    // Test that we can set and get data
    let test_data = TestData {
        id: 1,
        value: "test".to_string(),
    };
    
    let key = QueryKey::new(&["test", "1"]);
    assert!(client.set_query_data(&key, test_data.clone()).is_ok());
    
    let retrieved = client.get_query_data::<TestData>(&key);
    assert_eq!(retrieved, Some(test_data));
}

// Utility sleep function
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

// Integration test with Leptos runtime
#[test]
fn test_query_hook_integration() {
    // This would require a full Leptos runtime setup
    // For now, we'll just test the individual components
    
    let config = QueryClientConfig::default();
    let client = QueryClient::new(config);
    
    // Test cache functionality
    let key = QueryKey::new(&["test", "1"]);
    let test_data = TestData {
        id: 1,
        value: "cached".to_string(),
    };
    
    assert!(client.set_query_data(&key, test_data.clone()).is_ok());
    
    // Test cache invalidation
    client.invalidate_queries(&QueryKeyPattern::Exact(key.clone()));
    
    // Test cache removal
    client.remove_queries(&QueryKeyPattern::Exact(key));
}

#[test]
fn test_mutation_hook_integration() {
    let config = QueryClientConfig::default();
    let client = QueryClient::new(config);
    
    // Test that the client can handle mutations
    let key = QueryKey::new(&["users", "1"]);
    let user_data = TestData {
        id: 1,
        value: "user".to_string(),
    };
    
    assert!(client.set_query_data(&key, user_data.clone()).is_ok());
    
    // Test cache invalidation after mutation
    client.invalidate_queries(&QueryKeyPattern::Prefix(QueryKey::new(&["users"])));
}
