//! Unit tests for query lifecycle and state transitions

use leptos_query_rs::*;
use leptos_query_rs::retry::{QueryError, RetryConfig, should_retry_error};
use leptos_query_rs::types::{QueryStatus, QueryKey};
use leptos_query_rs::client::{SerializedData, CacheEntry};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestUser {
    id: u32,
    name: String,
    email: String,
}

// Mock API functions removed to eliminate warnings

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;
    // use leptos_test::*; // Not available, using basic tests instead

    #[test]
    fn test_query_status_transitions() {
        // Test that query status transitions work correctly
        let (status, set_status) = signal(QueryStatus::Idle);
        
        // Initial state
        assert_eq!(status.get(), QueryStatus::Idle);
        
        // Transition to loading
        set_status.set(QueryStatus::Loading);
        assert_eq!(status.get(), QueryStatus::Loading);
        
        // Transition to success
        set_status.set(QueryStatus::Success);
        assert_eq!(status.get(), QueryStatus::Success);
        
        // Transition to error
        set_status.set(QueryStatus::Error);
        assert_eq!(status.get(), QueryStatus::Error);
    }

    #[test]
    fn test_query_options_builder_pattern() {
        let options = QueryOptions::default()
            .with_stale_time(Duration::from_secs(60))
            .with_cache_time(Duration::from_secs(300))
            .with_refetch_interval(Duration::from_secs(30))
            .with_retry(RetryConfig::new(5, Duration::from_secs(1)))
            .disabled();
        
        assert_eq!(options.stale_time, Duration::from_secs(60));
        assert_eq!(options.cache_time, Duration::from_secs(300));
        assert_eq!(options.refetch_interval, Some(Duration::from_secs(30)));
        assert_eq!(options.retry.max_retries, 5);
        assert!(!options.enabled);
    }

    #[test]
    fn test_query_options_defaults() {
        let options = QueryOptions::default();
        
        assert!(options.enabled);
        assert_eq!(options.stale_time, Duration::from_secs(0));
        assert_eq!(options.cache_time, Duration::from_secs(5 * 60));
        assert!(options.refetch_interval.is_none());
        assert_eq!(options.retry.max_retries, 3);
    }

    #[test]
    fn test_query_key_creation_and_matching() {
        let key1 = QueryKey::new(&["users", "1"]);
        let key2 = QueryKey::new(&["users", "1", "posts"]);
        let key3 = QueryKey::from("simple");
        
        assert_eq!(key1.segments, vec!["users", "1"]);
        assert_eq!(key2.segments, vec!["users", "1", "posts"]);
        assert_eq!(key3.segments, vec!["simple"]);
        
        // Test pattern matching
        let pattern = QueryKeyPattern::Prefix(QueryKey::new(&["users"]));
        assert!(key1.matches_pattern(&pattern));
        assert!(key2.matches_pattern(&pattern));
        assert!(!key3.matches_pattern(&pattern));
    }

    #[test]
    fn test_query_key_pattern_matching() {
        let key = QueryKey::new(&["users", "123", "posts", "456"]);
        
        // Exact match
        let exact_pattern = QueryKeyPattern::Exact(QueryKey::new(&["users", "123", "posts", "456"]));
        assert!(key.matches_pattern(&exact_pattern));
        
        // Prefix match
        let prefix_pattern = QueryKeyPattern::Prefix(QueryKey::new(&["users"]));
        assert!(key.matches_pattern(&prefix_pattern));
        
        let prefix_pattern2 = QueryKeyPattern::Prefix(QueryKey::new(&["users", "123"]));
        assert!(key.matches_pattern(&prefix_pattern2));
        
        // Contains match
        let contains_pattern = QueryKeyPattern::Contains("posts".to_string());
        assert!(key.matches_pattern(&contains_pattern));
        
        // Non-matching patterns
        let non_matching_exact = QueryKeyPattern::Exact(QueryKey::new(&["users", "456"]));
        assert!(!key.matches_pattern(&non_matching_exact));
        
        let non_matching_prefix = QueryKeyPattern::Prefix(QueryKey::new(&["posts"]));
        assert!(!key.matches_pattern(&non_matching_prefix));
        
        let non_matching_contains = QueryKeyPattern::Contains("comments".to_string());
        assert!(!key.matches_pattern(&non_matching_contains));
    }

    #[test]
    fn test_query_meta_stale_and_expired() {
        let mut meta = QueryMeta::default();
        meta.stale_time = Duration::from_secs(60);
        meta.cache_time = Duration::from_secs(300);
        
        // Should not be stale or expired immediately
        assert!(!meta.is_stale());
        assert!(!meta.is_expired());
        
        // Should be stale but not expired after 120 seconds
        meta.updated_at = Instant::now() - Duration::from_secs(120);
        assert!(meta.is_stale());
        assert!(!meta.is_expired());
        
        // Should be both stale and expired after 400 seconds
        meta.updated_at = Instant::now() - Duration::from_secs(400);
        assert!(meta.is_stale());
        assert!(meta.is_expired());
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new(5, Duration::from_secs(2))
            .with_max_delay(Duration::from_secs(30))
            .with_fixed_delay()
            .no_network_retry();
        
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay, Duration::from_secs(2));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert!(!config.exponential_backoff);
        assert!(!config.retry_on_network_errors);
    }

    #[test]
    fn test_error_types_and_retryability() {
        let network_error = QueryError::NetworkError("Connection failed".to_string());
        let timeout_error = QueryError::TimeoutError("5000".to_string());
        let serialization_error = QueryError::SerializationError("Invalid JSON".to_string());
        let generic_error = QueryError::GenericError("Something went wrong".to_string());
        
        // Test error construction
        assert!(matches!(network_error, QueryError::NetworkError(_)));
        assert!(matches!(timeout_error, QueryError::TimeoutError(_)));
        assert!(matches!(serialization_error, QueryError::SerializationError(_)));
        assert!(matches!(generic_error, QueryError::GenericError(_)));
        
        // Test retryability based on actual implementation
        let config = RetryConfig::default();
        assert!(should_retry_error(&network_error, &config));
        assert!(should_retry_error(&timeout_error, &config));
        assert!(!should_retry_error(&serialization_error, &config)); // Serialization errors are not retryable
        assert!(should_retry_error(&generic_error, &config));
    }

    #[test]
    fn test_query_observer_id_uniqueness() {
        let id1 = QueryObserverId::new();
        let id2 = QueryObserverId::new();
        let id3 = QueryObserverId::new();
        
        // IDs should be unique
        assert_ne!(id1.id, id2.id);
        assert_ne!(id2.id, id3.id);
        assert_ne!(id1.id, id3.id);
        
        // IDs should be sequential
        assert!(id2.id > id1.id);
        assert!(id3.id > id2.id);
    }

    #[test]
    fn test_serialized_data_roundtrip() {
        let original_data = TestUser {
            id: 42,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };
        
        // Serialize
        let serialized = SerializedData {
            data: bincode::serialize(&original_data).unwrap(),
            timestamp: Instant::now(),
        };
        
        // Deserialize
        let deserialized: TestUser = bincode::deserialize(&serialized.data).unwrap();
        
        assert_eq!(original_data, deserialized);
    }

    #[test]
    fn test_cache_entry_lifecycle() {
        let data = TestUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let serialized = SerializedData {
            data: bincode::serialize(&data).unwrap(),
            timestamp: Instant::now(),
        };
        
        let mut meta = QueryMeta::default();
        meta.stale_time = Duration::from_secs(60);
        meta.cache_time = Duration::from_secs(300);
        
        let entry = CacheEntry {
            data: serialized.clone(),
            meta,
        };
        
        // Should not be stale immediately
        assert!(!entry.is_stale());
        
        // Should be able to get data
        let retrieved_data: TestUser = entry.get_data().unwrap();
        assert_eq!(data, retrieved_data);
        
        // Test stale entry
        let mut stale_meta = QueryMeta::default();
        stale_meta.stale_time = Duration::from_secs(60);
        stale_meta.updated_at = Instant::now() - Duration::from_secs(120);
        
        let stale_entry = CacheEntry {
            data: serialized,
            meta: stale_meta,
        };
        
        assert!(stale_entry.is_stale());
    }
}
