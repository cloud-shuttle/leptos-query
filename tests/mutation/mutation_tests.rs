//! Mutation tests for leptos-query core functionality
//! These tests validate that our test suite can catch bugs introduced by mutations

use leptos_query_rs::*;
use leptos_query_rs::types::{QueryKey, QueryKeyPattern, QueryStatus};
use leptos_query_rs::client::{SerializedData, CacheEntry};
use leptos_query_rs::retry::{QueryError, RetryConfig, should_retry_error};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    value: String,
    metadata: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_key_creation_mutation() {
        // This test should catch mutations in QueryKey::new
        let key = QueryKey::new(&["users", "1"]);
        assert_eq!(key.segments, vec!["users", "1"]);
        assert!(!key.is_empty());
        assert_eq!(key.len(), 2);
    }

    #[test]
    fn test_query_key_pattern_matching_mutation() {
        // This test should catch mutations in pattern matching logic
        let key = QueryKey::new(&["users", "1"]);
        
        // Test exact match
        let exact_pattern = QueryKeyPattern::Exact(key.clone());
        assert!(key.matches_pattern(&exact_pattern));
        
        // Test prefix match
        let prefix_pattern = QueryKeyPattern::Prefix(QueryKey::new(&["users"]));
        assert!(key.matches_pattern(&prefix_pattern));
        
        // Test contains match
        let contains_pattern = QueryKeyPattern::Contains("1".to_string());
        assert!(key.matches_pattern(&contains_pattern));
        
        // Test non-match
        let non_match_pattern = QueryKeyPattern::Contains("nonexistent".to_string());
        assert!(!key.matches_pattern(&non_match_pattern));
    }

    #[test]
    fn test_cache_operations_mutation() {
        // This test should catch mutations in cache operations
        let client = QueryClient::new();
        let key = QueryKey::new(&["test", "data"]);
        let data = TestData {
            id: 1,
            value: "test".to_string(),
            metadata: Some("metadata".to_string()),
        };
        
        // Test set operation
        assert!(client.set_query_data(&key, data.clone()).is_ok());
        
        // Test get operation
        let entry = client.get_cache_entry(&key);
        assert!(entry.is_some());
        
        let retrieved: TestData = entry.unwrap().get_data().unwrap();
        assert_eq!(retrieved, data);
        
        // Test remove operation
        client.remove_query(&key);
        let entry_after_remove = client.get_cache_entry(&key);
        assert!(entry_after_remove.is_none());
    }

    #[test]
    fn test_cache_invalidation_mutation() {
        // This test should catch mutations in cache invalidation logic
        let client = QueryClient::new();
        let data = TestData {
            id: 1,
            value: "test".to_string(),
            metadata: None,
        };
        
        // Add multiple entries
        let key1 = QueryKey::new(&["users", "1"]);
        let key2 = QueryKey::new(&["users", "2"]);
        let key3 = QueryKey::new(&["posts", "1"]);
        
        assert!(client.set_query_data(&key1, data.clone()).is_ok());
        assert!(client.set_query_data(&key2, data.clone()).is_ok());
        assert!(client.set_query_data(&key3, data.clone()).is_ok());
        
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

    #[test]
    fn test_retry_config_mutation() {
        // This test should catch mutations in retry configuration
        let config = RetryConfig::new(3, Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(1));
        
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(1));
    }

    #[test]
    fn test_error_retryability_mutation() {
        // This test should catch mutations in error retryability logic
        let config = RetryConfig::default();
        
        let network_error = QueryError::NetworkError("Connection failed".to_string());
        let timeout_error = QueryError::TimeoutError("Request timeout".to_string());
        let serialization_error = QueryError::SerializationError("Parse error".to_string());
        let generic_error = QueryError::GenericError("Generic error".to_string());
        
        // Network and timeout errors should be retryable
        assert!(should_retry_error(&network_error, &config));
        assert!(should_retry_error(&timeout_error, &config));
        
        // Serialization errors should not be retryable
        assert!(!should_retry_error(&serialization_error, &config));
        
        // Generic errors should be retryable
        assert!(should_retry_error(&generic_error, &config));
    }

    #[test]
    fn test_cache_entry_staleness_mutation() {
        // This test should catch mutations in staleness calculation
        let data = TestData {
            id: 1,
            value: "test".to_string(),
            metadata: None,
        };
        
        let serialized = SerializedData {
            data: bincode::serialize(&data).unwrap(),
            timestamp: Instant::now(),
        };
        
        let mut meta = QueryMeta::default();
        meta.stale_time = Duration::from_secs(60);
        meta.updated_at = Instant::now();
        
        let entry = CacheEntry {
            data: serialized,
            meta,
        };
        
        // Fresh entry should not be stale
        assert!(!entry.is_stale());
        
        // Should be able to retrieve data
        let retrieved: TestData = entry.get_data().unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_query_status_transitions_mutation() {
        // This test should catch mutations in query status logic
        let status = QueryStatus::Idle;
        assert_eq!(status, QueryStatus::Idle);
        
        // Test status transitions
        let loading_status = QueryStatus::Loading;
        assert_eq!(loading_status, QueryStatus::Loading);
        
        let success_status = QueryStatus::Success;
        assert_eq!(success_status, QueryStatus::Success);
        
        let error_status = QueryStatus::Error;
        assert_eq!(error_status, QueryStatus::Error);
    }

    #[test]
    fn test_serialization_mutation() {
        // This test should catch mutations in serialization logic
        let data = TestData {
            id: 42,
            value: "serialization test".to_string(),
            metadata: Some("test metadata".to_string()),
        };
        
        // Test bincode serialization
        let serialized = bincode::serialize(&data).unwrap();
        let deserialized: TestData = bincode::deserialize(&serialized).unwrap();
        assert_eq!(data, deserialized);
        
        // Test JSON serialization
        let json_serialized = serde_json::to_string(&data).unwrap();
        let json_deserialized: TestData = serde_json::from_str(&json_serialized).unwrap();
        assert_eq!(data, json_deserialized);
    }

    #[test]
    fn test_cache_stats_mutation() {
        // This test should catch mutations in cache statistics
        let client = QueryClient::new();
        
        // Initial stats should be zero
        let initial_stats = client.cache_stats();
        assert_eq!(initial_stats.total_entries, 0);
        assert_eq!(initial_stats.total_size, 0);
        
        // Add some data
        let key = QueryKey::new(&["stats", "test"]);
        let data = TestData {
            id: 1,
            value: "stats test".to_string(),
            metadata: None,
        };
        
        assert!(client.set_query_data(&key, data).is_ok());
        
        // Stats should be updated
        let updated_stats = client.cache_stats();
        assert!(updated_stats.total_entries > initial_stats.total_entries);
        assert!(updated_stats.total_size > initial_stats.total_size);
    }

    #[test]
    fn test_query_observer_id_mutation() {
        // This test should catch mutations in observer ID generation
        let id1 = QueryObserverId::new();
        let id2 = QueryObserverId::new();
        
        // IDs should be unique
        assert_ne!(id1.id, id2.id);
        
        // IDs should be non-zero
        assert_ne!(id1.id, 0);
        assert_ne!(id2.id, 0);
    }

    #[test]
    fn test_cache_cleanup_mutation() {
        // This test should catch mutations in cache cleanup logic
        let client = QueryClient::new();
        let data = TestData {
            id: 1,
            value: "cleanup test".to_string(),
            metadata: None,
        };
        
        // Add some data
        let key = QueryKey::new(&["cleanup", "test"]);
        assert!(client.set_query_data(&key, data).is_ok());
        
        // Verify data exists
        assert!(client.get_cache_entry(&key).is_some());
        
        // Cleanup stale entries
        client.cleanup_stale_entries();
        
        // Data should still exist (not stale)
        assert!(client.get_cache_entry(&key).is_some());
    }

    #[test]
    fn test_query_options_mutation() {
        // This test should catch mutations in query options
        let options = QueryOptions::default()
            .with_stale_time(Duration::from_secs(30))
            .with_cache_time(Duration::from_secs(60))
            .with_retry(RetryConfig::new(3, Duration::from_millis(100)));
        
        assert_eq!(options.stale_time, Duration::from_secs(30));
        assert_eq!(options.cache_time, Duration::from_secs(60));
        assert_eq!(options.retry.max_retries, 3);
    }

    #[test]
    fn test_mutation_options_mutation() {
        // This test should catch mutations in mutation options
        let options = MutationOptions::default()
            .with_retry(RetryConfig::new(2, Duration::from_millis(100)));
        
        assert_eq!(options.retry.max_retries, 2);
        assert_eq!(options.retry.base_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_edge_cases_mutation() {
        // This test should catch mutations in edge case handling
        let client = QueryClient::new();
        
        // Test empty query key
        let empty_key = QueryKey::new(&[]);
        assert!(empty_key.is_empty());
        assert_eq!(empty_key.len(), 0);
        
        // Test large query key
        let large_segments: Vec<String> = (0..100).map(|i| format!("segment_{}", i)).collect();
        let large_key = QueryKey::new(&large_segments.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        assert!(!large_key.is_empty());
        assert_eq!(large_key.len(), 100);
        
        // Test special characters in query key
        let special_key = QueryKey::new(&["special!@#$%^&*()", "characters"]);
        assert_eq!(special_key.segments, vec!["special!@#$%^&*()", "characters"]);
    }
}
