//! Unit tests for cache operations and invalidation patterns

use leptos_query_rs::*;
use leptos_query_rs::types::{QueryKey, QueryKeyPattern, QueryStatus};
use leptos_query_rs::client::{SerializedData, CacheEntry};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    value: String,
    metadata: Option<String>,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cache_operations() {
        let client = QueryClient::new();
        let key = QueryKey::new(&["test", "data"]);
        let data = TestData {
            id: 1,
            value: "test value".to_string(),
            metadata: Some("metadata".to_string()),
        };
        
        // Test setting data
        assert!(client.set_query_data(&key, data.clone()).is_ok());
        
        // Test getting data
        let entry = client.get_cache_entry(&key);
        assert!(entry.is_some());
        
        let cached_data = entry.unwrap().get_data::<TestData>().unwrap();
        assert_eq!(cached_data, data);
        
        // Test removing data
        client.remove_query(&key);
        assert!(client.get_cache_entry(&key).is_none());
    }

    #[test]
    fn test_cache_with_different_data_types() {
        let client = QueryClient::new();
        
        // Test with string
        let string_key = QueryKey::new(&["string"]);
        let string_data = "Hello, World!".to_string();
        assert!(client.set_query_data(&string_key, string_data.clone()).is_ok());
        let retrieved_string: String = client.get_cache_entry(&string_key).unwrap().get_data().unwrap();
        assert_eq!(retrieved_string, string_data);
        
        // Test with number
        let number_key = QueryKey::new(&["number"]);
        let number_data = 42i32;
        assert!(client.set_query_data(&number_key, number_data).is_ok());
        let retrieved_number: i32 = client.get_cache_entry(&number_key).unwrap().get_data().unwrap();
        assert_eq!(retrieved_number, 42);
        
        // Test with vector
        let vector_key = QueryKey::new(&["vector"]);
        let vector_data = vec![1, 2, 3, 4, 5];
        assert!(client.set_query_data(&vector_key, vector_data.clone()).is_ok());
        let retrieved_vector: Vec<i32> = client.get_cache_entry(&vector_key).unwrap().get_data().unwrap();
        assert_eq!(retrieved_vector, vector_data);
    }

    #[test]
    fn test_cache_invalidation_exact_match() {
        let client = QueryClient::new();
        
        // Set up test data
        let user1_key = QueryKey::new(&["users", "1"]);
        let user2_key = QueryKey::new(&["users", "2"]);
        let post1_key = QueryKey::new(&["posts", "1"]);
        
        let user1 = User { id: 1, name: "John".to_string(), email: "john@example.com".to_string() };
        let user2 = User { id: 2, name: "Jane".to_string(), email: "jane@example.com".to_string() };
        let post1 = Post { id: 1, title: "First Post".to_string(), content: "Content".to_string(), user_id: 1 };
        
        client.set_query_data(&user1_key, user1).unwrap();
        client.set_query_data(&user2_key, user2).unwrap();
        client.set_query_data(&post1_key, post1).unwrap();
        
        // Verify all data is cached
        assert!(client.get_cache_entry(&user1_key).is_some());
        assert!(client.get_cache_entry(&user2_key).is_some());
        assert!(client.get_cache_entry(&post1_key).is_some());
        
        // Invalidate exact match for user1
        let pattern = QueryKeyPattern::Exact(user1_key.clone());
        client.invalidate_queries(&pattern);
        
        // Only user1 should be removed
        assert!(client.get_cache_entry(&user1_key).is_none());
        assert!(client.get_cache_entry(&user2_key).is_some());
        assert!(client.get_cache_entry(&post1_key).is_some());
    }

    #[test]
    fn test_cache_invalidation_prefix_match() {
        let client = QueryClient::new();
        
        // Set up hierarchical data
        let users_key = QueryKey::new(&["users"]);
        let user1_key = QueryKey::new(&["users", "1"]);
        let user2_key = QueryKey::new(&["users", "2"]);
        let user1_posts_key = QueryKey::new(&["users", "1", "posts"]);
        let posts_key = QueryKey::new(&["posts"]);
        let post1_key = QueryKey::new(&["posts", "1"]);
        
        // Add test data
        client.set_query_data(&users_key, vec![1, 2]).unwrap();
        client.set_query_data(&user1_key, User { id: 1, name: "John".to_string(), email: "john@example.com".to_string() }).unwrap();
        client.set_query_data(&user2_key, User { id: 2, name: "Jane".to_string(), email: "jane@example.com".to_string() }).unwrap();
        client.set_query_data(&user1_posts_key, vec!["post1", "post2"]).unwrap();
        client.set_query_data(&posts_key, vec!["all", "posts"]).unwrap();
        client.set_query_data(&post1_key, Post { id: 1, title: "Post".to_string(), content: "Content".to_string(), user_id: 1 }).unwrap();
        
        // Verify all data is cached
        assert!(client.get_cache_entry(&users_key).is_some());
        assert!(client.get_cache_entry(&user1_key).is_some());
        assert!(client.get_cache_entry(&user2_key).is_some());
        assert!(client.get_cache_entry(&user1_posts_key).is_some());
        assert!(client.get_cache_entry(&posts_key).is_some());
        assert!(client.get_cache_entry(&post1_key).is_some());
        
        // Invalidate all users-related queries
        let pattern = QueryKeyPattern::Prefix(QueryKey::new(&["users"]));
        client.invalidate_queries(&pattern);
        
        // All users-related queries should be removed
        assert!(client.get_cache_entry(&users_key).is_none());
        assert!(client.get_cache_entry(&user1_key).is_none());
        assert!(client.get_cache_entry(&user2_key).is_none());
        assert!(client.get_cache_entry(&user1_posts_key).is_none());
        
        // Posts queries should remain
        assert!(client.get_cache_entry(&posts_key).is_some());
        assert!(client.get_cache_entry(&post1_key).is_some());
    }

    #[test]
    fn test_cache_invalidation_contains_match() {
        let client = QueryClient::new();
        
        // Set up test data with various keys
        let user1_key = QueryKey::new(&["users", "1"]);
        let user1_posts_key = QueryKey::new(&["users", "1", "posts"]);
        let user2_key = QueryKey::new(&["users", "2"]);
        let post1_key = QueryKey::new(&["posts", "1"]);
        let comment1_key = QueryKey::new(&["comments", "1"]);
        
        // Add test data
        client.set_query_data(&user1_key, User { id: 1, name: "John".to_string(), email: "john@example.com".to_string() }).unwrap();
        client.set_query_data(&user1_posts_key, vec!["post1", "post2"]).unwrap();
        client.set_query_data(&user2_key, User { id: 2, name: "Jane".to_string(), email: "jane@example.com".to_string() }).unwrap();
        client.set_query_data(&post1_key, Post { id: 1, title: "Post".to_string(), content: "Content".to_string(), user_id: 1 }).unwrap();
        client.set_query_data(&comment1_key, "comment".to_string()).unwrap();
        
        // Invalidate all queries containing "1"
        let pattern = QueryKeyPattern::Contains("1".to_string());
        client.invalidate_queries(&pattern);
        
        // Queries containing "1" should be removed
        assert!(client.get_cache_entry(&user1_key).is_none());
        assert!(client.get_cache_entry(&user1_posts_key).is_none());
        assert!(client.get_cache_entry(&post1_key).is_none());
        
        // Queries not containing "1" should remain
        assert!(client.get_cache_entry(&user2_key).is_some());
        // Note: comment1_key contains "1" so it should be removed
        assert!(client.get_cache_entry(&comment1_key).is_none());
    }

    #[test]
    fn test_cache_clear_all() {
        let client = QueryClient::new();
        
        // Add multiple entries
        let key1 = QueryKey::new(&["test", "1"]);
        let key2 = QueryKey::new(&["test", "2"]);
        let key3 = QueryKey::new(&["other", "data"]);
        
        client.set_query_data(&key1, "data1").unwrap();
        client.set_query_data(&key2, "data2").unwrap();
        client.set_query_data(&key3, "data3").unwrap();
        
        // Verify all entries exist
        assert!(client.get_cache_entry(&key1).is_some());
        assert!(client.get_cache_entry(&key2).is_some());
        assert!(client.get_cache_entry(&key3).is_some());
        
        // Clear all cache
        client.clear_cache();
        
        // All entries should be removed
        assert!(client.get_cache_entry(&key1).is_none());
        assert!(client.get_cache_entry(&key2).is_none());
        assert!(client.get_cache_entry(&key3).is_none());
    }

    #[test]
    fn test_cache_stats() {
        let client = QueryClient::with_settings(
            Duration::from_secs(60), // 1 minute stale time
            Duration::from_secs(300) // 5 minutes cache time
        );
        
        // Initially empty
        let stats = client.cache_stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.stale_entries, 0);
        assert_eq!(stats.total_size, 0);
        
        // Add some data
        let key1 = QueryKey::new(&["test", "1"]);
        let key2 = QueryKey::new(&["test", "2"]);
        let data1 = TestData { id: 1, value: "short".to_string(), metadata: None };
        let data2 = TestData { id: 2, value: "much longer data string".to_string(), metadata: Some("metadata".to_string()) };
        
        client.set_query_data(&key1, data1).unwrap();
        client.set_query_data(&key2, data2).unwrap();
        
        // Check stats
        let stats = client.cache_stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.stale_entries, 0); // Should not be stale immediately
        assert!(stats.total_size > 0); // Should have some size
        
        // Remove one entry
        client.remove_query(&key1);
        let stats = client.cache_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.stale_entries, 0);
    }

    #[test]
    fn test_cache_entry_staleness() {
        let client = QueryClient::with_settings(
            Duration::from_secs(1), // 1 second stale time
            Duration::from_secs(10) // 10 seconds cache time
        );
        
        let key = QueryKey::new(&["test"]);
        let data = TestData { id: 1, value: "test".to_string(), metadata: None };
        
        client.set_query_data(&key, data).unwrap();
        
        // Should not be stale immediately
        let entry = client.get_cache_entry(&key).unwrap();
        assert!(!entry.is_stale());
        
        // Wait for entry to become stale (this is a bit tricky in unit tests)
        // We'll test the staleness logic by creating a manually aged entry
        let mut stale_entry = entry.clone();
        stale_entry.meta.updated_at = Instant::now() - Duration::from_secs(2);
        assert!(stale_entry.is_stale());
    }

    #[test]
    fn test_cache_cleanup_stale_entries() {
        let client = QueryClient::with_settings(
            Duration::from_secs(1), // 1 second stale time
            Duration::from_secs(10) // 10 seconds cache time
        );
        
        let key1 = QueryKey::new(&["fresh"]);
        
        // Add fresh data
        client.set_query_data(&key1, TestData { id: 1, value: "fresh".to_string(), metadata: None }).unwrap();
        
        // Verify entry exists
        assert!(client.get_cache_entry(&key1).is_some());
        
        // Clean up stale entries (this should not remove fresh entries)
        client.cleanup_stale_entries();
        
        // Fresh entry should remain
        assert!(client.get_cache_entry(&key1).is_some());
    }

    #[test]
    fn test_cache_serialization_errors() {
        let client = QueryClient::new();
        let key = QueryKey::new(&["test"]);
        
        // Test with data that can't be serialized (this is tricky in Rust)
        // We'll test the error handling by using a type that should serialize fine
        let data = TestData { id: 1, value: "test".to_string(), metadata: None };
        
        // This should succeed
        assert!(client.set_query_data(&key, data).is_ok());
        
        // Test deserialization with wrong type
        let entry = client.get_cache_entry(&key).unwrap();
        let result: Result<String, _> = entry.get_data();
        assert!(result.is_err()); // Should fail because we're trying to deserialize TestData as String
    }

    #[test]
    fn test_concurrent_cache_access() {
        let client = QueryClient::new();
        let key = QueryKey::new(&["concurrent"]);
        let data = TestData { id: 1, value: "concurrent test".to_string(), metadata: None };
        
        // Test that we can set and get data concurrently
        // (This is more of a smoke test since we're using RwLock internally)
        client.set_query_data(&key, data.clone()).unwrap();
        
        // Multiple reads should work
        let entry1 = client.get_cache_entry(&key);
        let entry2 = client.get_cache_entry(&key);
        
        assert!(entry1.is_some());
        assert!(entry2.is_some());
        
        let data1: TestData = entry1.unwrap().get_data().unwrap();
        let data2: TestData = entry2.unwrap().get_data().unwrap();
        
        assert_eq!(data1, data);
        assert_eq!(data2, data);
    }
}
