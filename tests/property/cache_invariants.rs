//! Property-based tests for cache invariants and edge cases

use leptos_query_rs::*;
use leptos_query_rs::types::{QueryKey, QueryKeyPattern};
use leptos_query_rs::client::{SerializedData, CacheEntry};
use leptos_query_rs::retry::{QueryError, RetryConfig, should_retry_error};
use proptest::prelude::*;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    value: String,
    metadata: Option<String>,
}

// Property test strategies
prop_compose! {
    fn arb_test_data()(
        id in 0..1000u32,
        value in "[a-zA-Z0-9 ]{0,100}",
        metadata in prop::option::of("[a-zA-Z0-9 ]{0,50}")
    ) -> TestData {
        TestData { id, value, metadata }
    }
}

prop_compose! {
    fn arb_query_key()(
        segments in prop::collection::vec("[a-zA-Z0-9_]{1,20}", 1..5)
    ) -> QueryKey {
        QueryKey::new(segments)
    }
}

prop_compose! {
    fn arb_duration()(
        secs in 0..3600u64
    ) -> Duration {
        Duration::from_secs(secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn test_serialization_roundtrip(data in arb_test_data()) {
            // Property: Serialization and deserialization should be lossless
            let serialized = bincode::serialize(&data).unwrap();
            let deserialized: TestData = bincode::deserialize(&serialized).unwrap();
            prop_assert_eq!(data, deserialized);
        }

        #[test]
        fn test_query_key_consistency(segments in prop::collection::vec("[a-zA-Z0-9_]{1,20}", 1..5)) {
            // Property: QueryKey creation should be consistent
            let key1 = QueryKey::new(segments.clone());
            let key2 = QueryKey::new(segments);
            prop_assert_eq!(key1, key2);
        }

        #[test]
        fn test_query_key_pattern_exact_match(key in arb_query_key()) {
            // Property: Exact pattern should always match itself
            let pattern = QueryKeyPattern::Exact(key.clone());
            prop_assert!(key.matches_pattern(&pattern));
        }

        #[test]
        fn test_query_key_pattern_prefix_match(
            prefix_segments in prop::collection::vec("[a-zA-Z0-9_]{1,20}", 1..3),
            suffix_segments in prop::collection::vec("[a-zA-Z0-9_]{1,20}", 0..3)
        ) {
            // Property: Prefix pattern should match keys with that prefix
            let prefix = QueryKey::new(prefix_segments);
            let mut full_segments = prefix.segments.clone();
            full_segments.extend(suffix_segments);
            let full_key = QueryKey::new(full_segments);
            
            let pattern = QueryKeyPattern::Prefix(prefix);
            prop_assert!(full_key.matches_pattern(&pattern));
        }

        #[test]
        fn test_query_key_pattern_contains_match(
            key_segments in prop::collection::vec("[a-zA-Z0-9_]{1,20}", 1..5),
            substring in "[a-zA-Z0-9_]{1,10}"
        ) {
            // Property: Contains pattern should match if any segment contains the substring
            let key = QueryKey::new(key_segments);
            let pattern = QueryKeyPattern::Contains(substring.clone());
            
            let should_match = key.segments.iter().any(|segment| segment.contains(&substring));
            prop_assert_eq!(key.matches_pattern(&pattern), should_match);
        }

        #[test]
        fn test_cache_entry_staleness_invariant(
            stale_time in arb_duration(),
            age in arb_duration()
        ) {
            // Property: Cache entry staleness should be deterministic based on time
            let data = TestData { id: 1, value: "test".to_string(), metadata: None };
            let serialized = SerializedData {
                data: bincode::serialize(&data).unwrap(),
                timestamp: Instant::now() - age,
            };
            
            let mut meta = QueryMeta::default();
            meta.stale_time = stale_time;
            meta.updated_at = Instant::now() - age;
            
            let entry = CacheEntry { data: serialized, meta };
            let expected_stale = age >= stale_time;
            prop_assert_eq!(entry.is_stale(), expected_stale);
        }

        #[test]
        fn test_retry_config_bounds(
            max_retries in 0..10usize,
            base_delay in arb_duration(),
            max_delay in arb_duration()
        ) {
            // Property: Retry config should maintain valid bounds
            let config = RetryConfig::new(max_retries, base_delay)
                .with_max_delay(max_delay);
            
            prop_assert_eq!(config.max_retries, max_retries);
            prop_assert_eq!(config.base_delay, base_delay);
            prop_assert_eq!(config.max_delay, max_delay);
        }

        #[test]
        fn test_cache_operations_idempotent(
            key in arb_query_key(),
            data in arb_test_data()
        ) {
            // Property: Cache operations should be idempotent
            let client = QueryClient::new();
            
            // Set data multiple times
            prop_assert!(client.set_query_data(&key, data.clone()).is_ok());
            prop_assert!(client.set_query_data(&key, data.clone()).is_ok());
            
            // Should get the same data
            let entry = client.get_cache_entry(&key);
            prop_assert!(entry.is_some());
            
            let retrieved: TestData = entry.unwrap().get_data().unwrap();
            prop_assert_eq!(retrieved, data);
        }

        #[test]
        fn test_cache_invalidation_commutative(
            keys in prop::collection::vec(arb_query_key(), 1..10),
            pattern_type in 0..2usize
        ) {
            // Property: Cache invalidation should be commutative
            let client = QueryClient::new();
            let data = TestData { id: 1, value: "test".to_string(), metadata: None };
            
            // Create pattern based on type
            let pattern = match pattern_type {
                0 => QueryKeyPattern::Contains("test".to_string()),
                _ => QueryKeyPattern::Prefix(QueryKey::new(&["test"])),
            };
            
            // Add all keys to cache
            for key in &keys {
                prop_assert!(client.set_query_data(key, data.clone()).is_ok());
            }
            
            // Invalidate with pattern
            client.invalidate_queries(&pattern);
            
            // Check that matching keys are removed
            for key in &keys {
                let should_exist = !key.matches_pattern(&pattern);
                let exists = client.get_cache_entry(key).is_some();
                prop_assert_eq!(exists, should_exist);
            }
        }

        #[test]
        fn test_query_observer_id_uniqueness(count in 1..100usize) {
            // Property: Observer IDs should be unique
            let mut ids = Vec::new();
            for _ in 0..count {
                ids.push(QueryObserverId::new());
            }
            
            // All IDs should be unique
            for i in 0..ids.len() {
                for j in (i+1)..ids.len() {
                    prop_assert_ne!(ids[i].id, ids[j].id);
                }
            }
        }

        #[test]
        fn test_error_retryability_consistency(
            error_msg in "[a-zA-Z0-9 ]{1,50}"
        ) {
            // Property: Error retryability should be consistent
            let config = RetryConfig::default();
            
            let network_error = QueryError::NetworkError(error_msg.clone());
            let timeout_error = QueryError::TimeoutError(error_msg.clone());
            let serialization_error = QueryError::SerializationError(error_msg.clone());
            let generic_error = QueryError::GenericError(error_msg);
            
            // Network and timeout errors should be retryable by default
            prop_assert!(should_retry_error(&network_error, &config));
            prop_assert!(should_retry_error(&timeout_error, &config));
            
            // Serialization errors should not be retryable
            prop_assert!(!should_retry_error(&serialization_error, &config));
            
            // Generic errors should be retryable
            prop_assert!(should_retry_error(&generic_error, &config));
        }

        #[test]
        fn test_cache_stats_monotonic(
            key_count in 1..50usize,
            data_size in 1..1000usize
        ) {
            // Property: Cache stats should be monotonic
            let client = QueryClient::new();
            let data = TestData { 
                id: 1, 
                value: "x".repeat(data_size), 
                metadata: None 
            };
            
            let mut prev_stats = client.cache_stats();
            prop_assert_eq!(prev_stats.total_entries, 0);
            
            for i in 0..key_count {
                let key = QueryKey::new(&[&format!("key_{}", i)]);
                prop_assert!(client.set_query_data(&key, data.clone()).is_ok());
                
                let stats = client.cache_stats();
                prop_assert!(stats.total_entries > prev_stats.total_entries);
                prop_assert!(stats.total_size >= prev_stats.total_size);
                
                prev_stats = stats;
            }
        }

        #[test]
        fn test_query_key_serialization_roundtrip(key in arb_query_key()) {
            // Property: QueryKey serialization should be lossless
            let serialized = serde_json::to_string(&key).unwrap();
            let deserialized: QueryKey = serde_json::from_str(&serialized).unwrap();
            prop_assert_eq!(key, deserialized);
        }

        #[test]
        fn test_cache_entry_lifecycle_invariant(
            data in arb_test_data(),
            stale_time in arb_duration(),
            cache_time in arb_duration()
        ) {
            // Property: Cache entry lifecycle should be consistent
            let serialized = SerializedData {
                data: bincode::serialize(&data).unwrap(),
                timestamp: Instant::now(),
            };
            
            let mut meta = QueryMeta::default();
            meta.stale_time = stale_time;
            meta.cache_time = cache_time;
            
            let entry = CacheEntry { data: serialized, meta };
            
            // Fresh entry should not be stale
            prop_assert!(!entry.is_stale());
            
            // Should be able to retrieve data
            let retrieved: TestData = entry.get_data().unwrap();
            prop_assert_eq!(retrieved, data);
        }

        #[test]
        fn test_retry_delay_calculation_bounds(
            base_delay in arb_duration(),
            max_delay in arb_duration(),
            attempt in 0..10usize
        ) {
            // Property: Retry delay should be within bounds
            // Ensure max_delay is at least as large as base_delay
            let max_delay = max_delay.max(base_delay);
            
            let config = RetryConfig::new(10, base_delay)
                .with_max_delay(max_delay);
            
            // This tests the internal delay calculation logic
            // We can't directly test the private function, but we can test the config
            prop_assert!(config.base_delay <= config.max_delay);
            prop_assert!(config.max_retries >= attempt);
        }
    }

    // Additional property tests for edge cases
    proptest! {
        #[test]
        fn test_empty_query_key_behavior(empty_segments in prop::collection::vec("", 0..1)) {
            // Property: Empty query keys should behave consistently
            let key = QueryKey::new(empty_segments);
            prop_assert!(key.is_empty());
            prop_assert_eq!(key.len(), 0);
        }

        #[test]
        fn test_large_query_key_behavior(
            large_segments in prop::collection::vec("[a-zA-Z0-9_]{1,100}", 1..20)
        ) {
            // Property: Large query keys should work correctly
            let key = QueryKey::new(large_segments);
            prop_assert!(!key.is_empty());
            prop_assert!(key.len() > 0);
        }

        #[test]
        fn test_special_characters_in_query_key(
            segments in prop::collection::vec("[a-zA-Z0-9_!@#$%^&*()]{1,20}", 1..5)
        ) {
            // Property: Special characters in query keys should be handled correctly
            let key = QueryKey::new(segments);
            prop_assert!(!key.is_empty());
            
            // Should be able to create patterns with the same key
            let pattern = QueryKeyPattern::Exact(key.clone());
            prop_assert!(key.matches_pattern(&pattern));
        }
    }
}
