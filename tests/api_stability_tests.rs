//! API Stability Tests
//! 
//! These tests verify that the public API is stable and works correctly
//! with the documented usage patterns.

use leptos_query_rs::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_import_path() {
        // This test verifies that the documented import path works
        // This should match the doctest in lib.rs
        let _client = QueryClient::new();
        let _key = QueryKey::new(&["test"]);
        let _options = QueryOptions::default();
        
        // If this compiles, the import path is correct
        assert!(true);
    }

    #[test]
    fn test_query_key_creation_patterns() {
        // Test the documented QueryKey creation patterns
        let key1 = QueryKey::new(&["user", "1"]);
        let key2 = QueryKey::from(["user", "1"]);
        let key3 = QueryKey::from(&["user", "1"][..]);
        
        assert_eq!(key1.segments, vec!["user", "1"]);
        assert_eq!(key2.segments, vec!["user", "1"]);
        assert_eq!(key3.segments, vec!["user", "1"]);
    }

    #[test]
    fn test_query_options_builder() {
        // Test the documented QueryOptions usage
        let options = QueryOptions::default()
            .with_stale_time(Duration::from_secs(60))
            .with_cache_time(Duration::from_secs(300));
        
        assert_eq!(options.stale_time, Duration::from_secs(60));
        assert_eq!(options.cache_time, Duration::from_secs(300));
    }

    #[test]
    fn test_mutation_options_builder() {
        // Test the documented MutationOptions usage
        let options = MutationOptions::default()
            .with_retry(RetryConfig::new(3, Duration::from_millis(100)));
        
        assert_eq!(options.retry.max_retries, 3);
    }

    #[test]
    fn test_error_types() {
        // Test the documented error types
        let network_error = QueryError::NetworkError("Connection failed".to_string());
        let timeout_error = QueryError::TimeoutError("5000".to_string());
        let generic_error = QueryError::GenericError("Something went wrong".to_string());
        
        assert!(matches!(network_error, QueryError::NetworkError(_)));
        assert!(matches!(timeout_error, QueryError::TimeoutError(_)));
        assert!(matches!(generic_error, QueryError::GenericError(_)));
    }

    #[test]
    fn test_retry_config_builder() {
        // Test the documented RetryConfig usage
        let config = RetryConfig::new(3, Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(1));
        
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(1));
    }

    #[test]
    fn test_query_key_pattern_matching() {
        // Test the documented QueryKeyPattern usage
        let key = QueryKey::new(&["users", "1", "posts"]);
        
        // Exact match
        let exact_pattern = QueryKeyPattern::Exact(QueryKey::new(&["users", "1", "posts"]));
        assert!(key.matches_pattern(&exact_pattern));
        
        // Prefix match
        let prefix_pattern = QueryKeyPattern::Prefix(QueryKey::new(&["users"]));
        assert!(key.matches_pattern(&prefix_pattern));
        
        // Contains match
        let contains_pattern = QueryKeyPattern::Contains("posts".to_string());
        assert!(key.matches_pattern(&contains_pattern));
    }

    #[test]
    fn test_serialized_data_roundtrip() {
        // Test the documented SerializedData usage
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let serialized = SerializedData {
            data: bincode::serialize(&user).unwrap(),
            timestamp: std::time::Instant::now(),
        };
        
        let deserialized: User = bincode::deserialize(&serialized.data).unwrap();
        assert_eq!(user, deserialized);
    }

    #[test]
    fn test_cache_operations() {
        // Test the documented cache operations
        let client = QueryClient::new();
        let key = QueryKey::new(&["test", "user"]);
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // Set data
        assert!(client.set_query_data(&key, user.clone()).is_ok());
        
        // Get data
        let entry = client.get_cache_entry(&key);
        assert!(entry.is_some());
        
        let retrieved: User = entry.unwrap().get_data().unwrap();
        assert_eq!(user, retrieved);
        
        // Remove data
        client.remove_query(&key);
        let entry_after_remove = client.get_cache_entry(&key);
        assert!(entry_after_remove.is_none());
    }

    #[test]
    fn test_cache_invalidation() {
        // Test the documented cache invalidation
        let client = QueryClient::new();
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // Add multiple entries
        let key1 = QueryKey::new(&["users", "1"]);
        let key2 = QueryKey::new(&["users", "2"]);
        let key3 = QueryKey::new(&["posts", "1"]);
        
        assert!(client.set_query_data(&key1, user.clone()).is_ok());
        assert!(client.set_query_data(&key2, user.clone()).is_ok());
        assert!(client.set_query_data(&key3, user.clone()).is_ok());
        
        // Test exact invalidation
        let exact_pattern = QueryKeyPattern::Exact(key1.clone());
        client.invalidate_queries(&exact_pattern);
        assert!(client.get_cache_entry(&key1).is_none());
        assert!(client.get_cache_entry(&key2).is_some());
        assert!(client.get_cache_entry(&key3).is_some());
        
        // Test prefix invalidation
        let prefix_pattern = QueryKeyPattern::Prefix(QueryKey::new(&["users"]));
        client.invalidate_queries(&prefix_pattern);
        assert!(client.get_cache_entry(&key2).is_none());
        assert!(client.get_cache_entry(&key3).is_some());
    }
}
