//! Test utilities and helpers for leptos-query tests

use leptos_query_rs::*;
use leptos_query_rs::retry::{QueryError, RetryConfig};
use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Test data structures for use in tests
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TestUser {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TestPost {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub user_id: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TestComment {
    pub id: u32,
    pub post_id: u32,
    pub content: String,
    pub author_id: u32,
}

/// Mock API functions for testing
pub mod mock_api {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

    /// Mock fetch user function
    pub async fn fetch_user(id: u32) -> Result<TestUser, QueryError> {
        CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        if id == 0 {
            return Err(QueryError::GenericError("User not found".to_string()));
        }
        
        Ok(TestUser {
            id,
            name: format!("User {}", id),
            email: format!("user{}@example.com", id),
        })
    }

    /// Mock fetch user with configurable delay
    pub async fn fetch_user_with_delay(id: u32, delay_ms: u64) -> Result<TestUser, QueryError> {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        fetch_user(id).await
    }

    /// Mock fetch user that fails after a certain number of calls
    pub async fn fetch_user_with_failure_after(id: u32, fail_after: u32) -> Result<TestUser, QueryError> {
        let current_count = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        
        if current_count >= fail_after {
            return Err(QueryError::NetworkError("Simulated network failure".to_string()));
        }
        
        fetch_user(id).await
    }

    /// Mock fetch posts for a user
    pub async fn fetch_user_posts(user_id: u32) -> Result<Vec<TestPost>, QueryError> {
        tokio::time::sleep(Duration::from_millis(5)).await;
        
        if user_id == 0 {
            return Err(QueryError::GenericError("User not found".to_string()));
        }
        
        Ok(vec![
            TestPost {
                id: 1,
                title: format!("Post 1 by User {}", user_id),
                content: "Content of post 1".to_string(),
                user_id,
            },
            TestPost {
                id: 2,
                title: format!("Post 2 by User {}", user_id),
                content: "Content of post 2".to_string(),
                user_id,
            },
        ])
    }

    /// Mock create user function
    pub async fn create_user(name: String, email: String) -> Result<TestUser, QueryError> {
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        if email.contains("error") {
            return Err(QueryError::GenericError("Invalid email".to_string()));
        }
        
        Ok(TestUser {
            id: 999, // Mock ID
            name,
            email,
        })
    }

    /// Mock update user function
    pub async fn update_user(id: u32, name: String, email: String) -> Result<TestUser, QueryError> {
        tokio::time::sleep(Duration::from_millis(15)).await;
        
        if id == 0 {
            return Err(QueryError::GenericError("User not found".to_string()));
        }
        
        Ok(TestUser { id, name, email })
    }

    /// Mock delete user function
    pub async fn delete_user(id: u32) -> Result<(), QueryError> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        if id == 0 {
            return Err(QueryError::GenericError("User not found".to_string()));
        }
        
        Ok(())
    }

    /// Reset call count for testing
    pub fn reset_call_count() {
        CALL_COUNT.store(0, Ordering::SeqCst);
    }

    /// Get current call count
    pub fn get_call_count() -> u32 {
        CALL_COUNT.load(Ordering::SeqCst)
    }
}

/// Test client factory
pub fn create_test_client() -> QueryClient {
    QueryClient::with_settings(
        Duration::from_secs(0), // Always stale for testing
        Duration::from_secs(300), // 5 minutes cache time
    )
}

/// Test client with custom settings
pub fn create_test_client_with_settings(stale_time: Duration, cache_time: Duration) -> QueryClient {
    QueryClient::with_settings(stale_time, cache_time)
}

/// Test retry config for fast testing
pub fn create_fast_retry_config() -> RetryConfig {
    RetryConfig::new(2, Duration::from_millis(10))
        .with_max_delay(Duration::from_millis(100))
        .with_fixed_delay()
}

/// Test retry config for slow testing
pub fn create_slow_retry_config() -> RetryConfig {
    RetryConfig::new(3, Duration::from_millis(100))
        .with_max_delay(Duration::from_secs(1))
        .with_exponential_delay()
}

/// Sample test data
pub mod test_data {
    use super::*;

    pub fn sample_user(id: u32) -> TestUser {
        TestUser {
            id,
            name: format!("User {}", id),
            email: format!("user{}@example.com", id),
        }
    }

    pub fn sample_users() -> Vec<TestUser> {
        vec![
            sample_user(1),
            sample_user(2),
            sample_user(3),
        ]
    }

    pub fn sample_post(id: u32, user_id: u32) -> TestPost {
        TestPost {
            id,
            title: format!("Post {}", id),
            content: format!("Content of post {}", id),
            user_id,
        }
    }

    pub fn sample_posts_for_user(user_id: u32) -> Vec<TestPost> {
        vec![
            sample_post(1, user_id),
            sample_post(2, user_id),
        ]
    }
}

/// Test query keys
pub mod test_keys {
    use super::*;

    pub fn user_key(id: u32) -> QueryKey {
        QueryKey::new(&["users", &id.to_string()])
    }

    pub fn users_key() -> QueryKey {
        QueryKey::new(&["users"])
    }

    pub fn user_posts_key(user_id: u32) -> QueryKey {
        QueryKey::new(&["users", &user_id.to_string(), "posts"])
    }

    pub fn post_key(id: u32) -> QueryKey {
        QueryKey::new(&["posts", &id.to_string()])
    }

    pub fn posts_key() -> QueryKey {
        QueryKey::new(&["posts"])
    }
}

/// Test patterns for cache invalidation
pub mod test_patterns {
    use super::*;

    pub fn all_users_pattern() -> QueryKeyPattern {
        QueryKeyPattern::Prefix(QueryKey::new(&["users"]))
    }

    pub fn specific_user_pattern(id: u32) -> QueryKeyPattern {
        QueryKeyPattern::Exact(QueryKey::new(&["users", &id.to_string()]))
    }

    pub fn posts_pattern() -> QueryKeyPattern {
        QueryKeyPattern::Prefix(QueryKey::new(&["posts"]))
    }

    pub fn contains_pattern(substring: &str) -> QueryKeyPattern {
        QueryKeyPattern::Contains(substring.to_string())
    }
}

/// Test assertions helpers
pub mod assertions {
    use super::*;

    /// Assert that a query result is in loading state
    pub fn assert_loading<T>(result: &QueryResult<T>) {
        assert!(result.is_loading.get());
        assert_eq!(result.status.get(), QueryStatus::Loading);
        assert!(result.data.get().is_none());
        assert!(result.error.get().is_none());
    }

    /// Assert that a query result is in success state
    pub fn assert_success<T: PartialEq>(result: &QueryResult<T>, expected_data: &T) {
        assert!(!result.is_loading.get());
        assert!(result.is_success.get());
        assert_eq!(result.status.get(), QueryStatus::Success);
        assert_eq!(result.data.get(), Some(expected_data.clone()));
        assert!(result.error.get().is_none());
    }

    /// Assert that a query result is in error state
    pub fn assert_error<T>(result: &QueryResult<T>) {
        assert!(!result.is_loading.get());
        assert!(result.is_error.get());
        assert_eq!(result.status.get(), QueryStatus::Error);
        assert!(result.data.get().is_none());
        assert!(result.error.get().is_some());
    }

    /// Assert that a query result is in idle state
    pub fn assert_idle<T>(result: &QueryResult<T>) {
        assert!(!result.is_loading.get());
        assert!(!result.is_success.get());
        assert!(!result.is_error.get());
        assert_eq!(result.status.get(), QueryStatus::Idle);
        assert!(result.data.get().is_none());
        assert!(result.error.get().is_none());
    }
}

/// Test timing utilities
pub mod timing {
    use std::time::{Duration, Instant};

    /// Measure execution time of a function
    pub fn measure_time<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Assert that execution time is within expected range
    pub fn assert_time_within<F, R>(f: F, min: Duration, max: Duration) -> R
    where
        F: FnOnce() -> R,
    {
        let (result, duration) = measure_time(f);
        assert!(duration >= min, "Execution too fast: {:?} < {:?}", duration, min);
        assert!(duration <= max, "Execution too slow: {:?} > {:?}", duration, max);
        result
    }
}

/// Test data generators
pub mod generators {
    use super::*;
    use rand::Rng;

    /// Generate random test user
    pub fn random_user() -> TestUser {
        let mut rng = rand::thread_rng();
        let id = rng.gen_range(1..1000);
        TestUser {
            id,
            name: format!("Random User {}", id),
            email: format!("user{}@example.com", id),
        }
    }

    /// Generate multiple random users
    pub fn random_users(count: usize) -> Vec<TestUser> {
        (0..count).map(|_| random_user()).collect()
    }

    /// Generate random query key
    pub fn random_key() -> QueryKey {
        let mut rng = rand::thread_rng();
        let segments = (0..rng.gen_range(1..4))
            .map(|i| format!("segment_{}", i))
            .collect::<Vec<_>>();
        QueryKey::new(segments)
    }
}
