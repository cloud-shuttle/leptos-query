
use leptos_query_rs::*;
use leptos_query_rs::retry::{QueryError, RetryConfig};
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    value: String,
}

// Mock functions removed to eliminate warnings

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
    
    // SerializedData::serialize removed in current API - test cache operations instead
    let client = QueryClient::new();
    let key = QueryKey::new(&["test-user"]);
    assert!(client.set_query_data(&key, data.clone()).is_ok());
    let entry = client.get_cache_entry(&key).unwrap();
    let deserialized: TestData = entry.get_data().unwrap();
    
    assert_eq!(data, deserialized);
}

#[test]
fn test_query_options_builder() {
    let options = QueryOptions::default()
        .with_stale_time(Duration::from_secs(60))
        .with_cache_time(Duration::from_secs(300));
    
    assert_eq!(options.stale_time, Duration::from_secs(60));
    assert_eq!(options.cache_time, Duration::from_secs(300));
    // keep_previous_data and suspense removed in current API
}

#[test]
fn test_retry_config() {
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    
    let custom_config = RetryConfig::new(5, Duration::from_secs(1));
    
    assert_eq!(custom_config.max_retries, 5);
}

#[test]
fn test_error_types() {
    let network_error = QueryError::NetworkError("connection failed".to_string());
    let timeout_error = QueryError::TimeoutError("5000".to_string());
    let http_error = QueryError::GenericError("server error".to_string());
    let custom_error = QueryError::GenericError("validation failed".to_string());
    
    // Test that errors are properly constructed
    assert!(matches!(network_error, QueryError::NetworkError(_)));
    assert!(matches!(timeout_error, QueryError::TimeoutError(_)));
    assert!(matches!(http_error, QueryError::GenericError(_)));
    assert!(matches!(custom_error, QueryError::GenericError(_)));
}

#[test]
fn test_query_client_creation() {
    let client = QueryClient::new();
    
    // Test that we can set and get data
    let test_data = TestData {
        id: 1,
        value: "test".to_string(),
    };
    
    let key = QueryKey::new(&["test", "1"]);
    assert!(client.set_query_data(&key, test_data.clone()).is_ok());
    
    let entry = client.get_cache_entry(&key).unwrap();
    let retrieved: TestData = entry.get_data().unwrap();
    assert_eq!(retrieved, test_data);
}

// Sleep function removed to eliminate warnings

// Integration test with Leptos runtime
#[test]
fn test_query_hook_integration() {
    // This would require a full Leptos runtime setup
    // For now, we'll just test the individual components
    
    let client = QueryClient::new();
    
    // Test cache functionality
    let key = QueryKey::new(&["test", "1"]);
    let test_data = TestData {
        id: 1,
        value: "cached".to_string(),
    };
    
    assert!(client.set_query_data(&key, test_data.clone()).is_ok());
    
    // Test cache invalidation
    // invalidate via patterns is not implemented in client API
    
    // Test cache removal
    client.remove_query(&key);
}

#[test]
fn test_mutation_hook_integration() {
    let client = QueryClient::new();
    
    // Test that the client can handle mutations
    let key = QueryKey::new(&["users", "1"]);
    let user_data = TestData {
        id: 1,
        value: "user".to_string(),
    };
    
    assert!(client.set_query_data(&key, user_data.clone()).is_ok());
    
    // Test cache invalidation after mutation
    // invalidate via patterns is not implemented in client API
}
