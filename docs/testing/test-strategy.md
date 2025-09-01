# Testing Strategy
**Leptos Query - Comprehensive Testing Plan**

## Document Information
- **Version**: 1.0  
- **Date**: September 2024
- **Status**: Draft
- **Authors**: CloudShuttle Team

## 1. Testing Philosophy

### 1.1 Testing Pyramid

```
        ┌─────────────┐
        │    E2E      │ (10% - High Value, High Cost)
        │   Tests     │ 
        └─────────────┘
      ┌─────────────────┐
      │  Integration    │ (25% - Medium Value, Medium Cost)
      │     Tests       │
      └─────────────────┘
    ┌─────────────────────┐
    │    Unit Tests       │ (60% - High Value, Low Cost)
    │                     │
    └─────────────────────┘
  ┌─────────────────────────┐
  │   Property Tests        │ (5% - Invariant Validation)
  │                         │
  └─────────────────────────┘
```

### 1.2 Testing Principles

1. **Fast Feedback**: Unit tests provide immediate feedback
2. **Realistic Testing**: Integration tests verify component interactions
3. **User-Centric E2E**: End-to-end tests validate user workflows
4. **Property-Based**: Property tests verify invariants and edge cases
5. **Deterministic**: All tests must be reliable and reproducible

### 1.3 Coverage Requirements

| Test Level | Coverage Target | Rationale |
|------------|-----------------|-----------|
| Unit Tests | 90% line coverage | Comprehensive API coverage |
| Integration | All major flows | Component interaction validation |
| E2E | Critical user paths | Real-world scenario validation |
| Property | Core invariants | Edge case and invariant validation |

## 2. Test Categories and Implementation

### 2.1 Unit Tests (60% of test suite)

#### Cache System Tests

```rust
// tests/unit/cache_tests.rs

#[cfg(test)]
mod cache_tests {
    use super::*;
    use leptos_query::*;
    use std::time::Duration;

    #[test]
    fn test_cache_entry_lifecycle() {
        let mut entry = CacheEntry {
            data: Some(SerializedData::serialize(&"test data").unwrap()),
            error: None,
            state: QueryState::Success,
            updated_at: Instant::now(),
            data_updated_at: Instant::now(),
            stale_time: Duration::from_secs(60),
            cache_time: Duration::from_secs(300),
            meta: QueryMeta::default(),
        };
        
        // Should not be stale immediately
        assert!(!entry.is_stale());
        
        // Should not be expired
        assert!(!entry.is_expired());
        
        // Simulate time passing
        entry.updated_at = Instant::now() - Duration::from_secs(61);
        assert!(entry.is_stale());
    }

    #[test]
    fn test_cache_set_and_get() {
        leptos_test::create_test_runtime(|| {
            let client = QueryClient::new(QueryClientConfig::default());
            let key = QueryKey::new(["test", "key"]);
            let data = TestUser { id: 1, name: "John".into() };
            
            // Set data in cache
            client.set_query_data(&key, &data).unwrap();
            
            // Retrieve data from cache
            let retrieved = client.get_query_data::<TestUser>(&key);
            assert_eq!(retrieved, Some(data));
        });
    }

    #[test]
    fn test_cache_invalidation_patterns() {
        leptos_test::create_test_runtime(|| {
            let client = QueryClient::new(QueryClientConfig::default());
            
            // Set multiple cache entries
            for i in 0..5 {
                let key = QueryKey::new(["users", i.to_string()]);
                client.set_query_data(&key, &TestUser { 
                    id: i, 
                    name: format!("User {}", i) 
                }).unwrap();
            }
            
            // Test prefix invalidation
            client.invalidate_queries(QueryKeyPattern::Prefix(
                QueryKey::new(["users"])
            ));
            
            // All should be marked stale
            for i in 0..5 {
                let key = QueryKey::new(["users", i.to_string()]);
                let entry = client.get_cache_entry(&key).unwrap();
                assert!(entry.is_stale());
            }
        });
    }

    #[test]
    fn test_garbage_collection() {
        leptos_test::create_test_runtime(|| async {
            let config = QueryClientConfig {
                gc_interval: Duration::from_millis(50),
                default_cache_time: Duration::from_millis(100),
                ..Default::default()
            };
            let client = QueryClient::new(config);
            
            // Add cache entry
            let key = QueryKey::new(["temp"]);
            client.set_query_data(&key, &"temporary data").unwrap();
            
            // Entry should exist
            assert!(client.get_query_data::<String>(&key).is_some());
            
            // Wait for garbage collection
            tokio::time::sleep(Duration::from_millis(200)).await;
            
            // Entry should be removed
            assert!(client.get_query_data::<String>(&key).is_none());
        });
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestUser {
        id: u32,
        name: String,
    }
}
```

#### Query Key Tests

```rust
// tests/unit/key_tests.rs

#[cfg(test)]
mod key_tests {
    use super::*;
    use leptos_query::*;

    #[test]
    fn test_query_key_creation() {
        let key = QueryKey::new(["users", "123", "posts"]);
        assert_eq!(key.segments, vec!["users", "123", "posts"]);
    }

    #[test]
    fn test_query_key_from_conversions() {
        // From slice
        let key1: QueryKey = ["users", "123"].into();
        assert_eq!(key1.segments, vec!["users", "123"]);
        
        // From tuple
        let key2: QueryKey = ("users", 123u32).into();
        assert_eq!(key2.segments, vec!["users", "123"]);
        
        // From triple
        let key3: QueryKey = ("posts", 123u32, "comments").into();
        assert_eq!(key3.segments, vec!["posts", "123", "comments"]);
    }

    #[test]
    fn test_query_key_pattern_matching() {
        let key = QueryKey::new(["users", "123", "posts", "456"]);
        
        // Exact match
        assert!(key.matches_pattern(&QueryKeyPattern::Exact(
            QueryKey::new(["users", "123", "posts", "456"])
        )));
        
        // Prefix match
        assert!(key.matches_pattern(&QueryKeyPattern::Prefix(
            QueryKey::new(["users"])
        )));
        assert!(key.matches_pattern(&QueryKeyPattern::Prefix(
            QueryKey::new(["users", "123"])
        )));
        
        // Contains match
        assert!(key.matches_pattern(&QueryKeyPattern::Contains(
            "posts".to_string()
        )));
        
        // Predicate match
        assert!(key.matches_pattern(&QueryKeyPattern::Predicate(
            Box::new(|k| k.segments.len() > 3)
        )));
    }

    #[test]
    fn test_query_key_serialization() {
        let key = QueryKey::new(["users", "123", "posts"]);
        let serialized = serde_json::to_string(&key).unwrap();
        let deserialized: QueryKey = serde_json::from_str(&serialized).unwrap();
        assert_eq!(key, deserialized);
    }
}
```

#### Error Handling Tests

```rust
// tests/unit/error_tests.rs

#[cfg(test)]
mod error_tests {
    use super::*;
    use leptos_query::*;

    #[test]
    fn test_query_error_types() {
        let network_error = QueryError::network("Connection failed");
        assert!(network_error.is_retryable());
        assert_eq!(network_error.severity(), ErrorSeverity::Warning);
        
        let timeout_error = QueryError::timeout(5000);
        assert!(timeout_error.is_retryable());
        assert_eq!(
            timeout_error.suggested_retry_delay(), 
            Some(Duration::from_millis(2000))
        );
        
        let http_error = QueryError::http(404, "Not Found");
        assert!(!http_error.is_retryable());
        
        let server_error = QueryError::http(500, "Internal Server Error");
        assert!(server_error.is_retryable());
    }

    #[test]
    fn test_error_context_preservation() {
        let error = QueryError::network_with_source(
            "Failed to connect", 
            "DNS resolution failed"
        );
        
        match error {
            QueryError::Network { message, source } => {
                assert_eq!(message, "Failed to connect");
                assert_eq!(source, Some("DNS resolution failed".to_string()));
            }
            _ => panic!("Expected network error"),
        }
    }

    #[test]
    fn test_retry_delay_calculation() {
        let exponential = RetryDelay::Exponential {
            initial: Duration::from_millis(1000),
            multiplier: 2.0,
            max: Duration::from_secs(30),
        };
        
        assert_eq!(exponential.calculate(0, false), Duration::from_millis(1000));
        assert_eq!(exponential.calculate(1, false), Duration::from_millis(2000));
        assert_eq!(exponential.calculate(2, false), Duration::from_millis(4000));
        
        // Test jitter (should be within range)
        let with_jitter = exponential.calculate(1, true);
        assert!(with_jitter >= Duration::from_millis(1000));
        assert!(with_jitter <= Duration::from_millis(2000));
    }
}
```

### 2.2 Integration Tests (25% of test suite)

#### Query-Cache Integration

```rust
// tests/integration/query_cache_integration.rs

#[cfg(test)]
mod integration_tests {
    use super::*;
    use leptos_query::*;
    use leptos_test::*;

    #[tokio::test]
    async fn test_query_cache_integration() {
        create_test_runtime(|| async {
            let client = QueryClient::new(QueryClientConfig::default());
            
            // Mock API function
            let fetch_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
            let fetch_count_clone = fetch_count.clone();
            
            let fetch_user = move |id: u32| {
                let count = fetch_count_clone.clone();
                async move {
                    count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok(TestUser { id, name: format!("User {}", id) })
                }
            };
            
            // First query should fetch
            let key = QueryKey::new(["users", "1"]);
            let result1 = client.prefetch_query(key.clone(), || fetch_user(1)).await;
            assert!(result1.is_ok());
            assert_eq!(fetch_count.load(std::sync::atomic::Ordering::SeqCst), 1);
            
            // Second identical query should use cache
            let result2 = client.get_query_data::<TestUser>(&key);
            assert!(result2.is_some());
            assert_eq!(fetch_count.load(std::sync::atomic::Ordering::SeqCst), 1);
            
            // Invalidate and refetch
            client.invalidate_queries(QueryKeyPattern::Exact(key.clone()));
            let result3 = client.prefetch_query(key, || fetch_user(1)).await;
            assert!(result3.is_ok());
            assert_eq!(fetch_count.load(std::sync::atomic::Ordering::SeqCst), 2);
        }).await;
    }

    #[test]
    fn test_mutation_invalidation_integration() {
        create_test_runtime(|| async {
            let client = QueryClient::new(QueryClientConfig::default());
            
            // Setup initial cache
            let users_key = QueryKey::new(["users"]);
            let initial_users = vec![
                TestUser { id: 1, name: "John".into() },
                TestUser { id: 2, name: "Jane".into() },
            ];
            client.set_query_data(&users_key, &initial_users).unwrap();
            
            // Create mutation that invalidates users
            let create_user_mutation = |_: CreateUserDto| async {
                Ok(TestUser { id: 3, name: "Bob".into() })
            };
            
            // Execute mutation
            let new_user = CreateUserDto { name: "Bob".into() };
            let result = create_user_mutation(new_user).await;
            assert!(result.is_ok());
            
            // Invalidate users cache (simulating mutation behavior)
            client.invalidate_queries(QueryKeyPattern::Prefix(users_key.clone()));
            
            // Cache entry should be marked as stale
            let entry = client.get_cache_entry(&users_key).unwrap();
            assert!(entry.is_stale());
        });
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestUser {
        id: u32,
        name: String,
    }

    #[derive(Clone, serde::Serialize)]
    struct CreateUserDto {
        name: String,
    }
}
```

### 2.3 End-to-End Tests (10% of test suite)

#### Real Application Flows

```rust
// tests/e2e/user_management_flow.rs

#[cfg(test)]
mod e2e_tests {
    use leptos::*;
    use leptos_query::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_complete_user_management_flow() {
        // Setup test application
        let app = create_test_app();
        
        // 1. Load user list (should show loading state)
        let users_loading = app.query_selector(".users-loading").unwrap();
        assert!(users_loading.is_some());
        
        // 2. Wait for users to load
        app.wait_for_selector(".user-list", 5000).await.unwrap();
        let user_list = app.query_selector(".user-list").unwrap();
        assert!(user_list.is_some());
        
        // 3. Click create user button
        let create_button = app.query_selector(".create-user-button").unwrap()
            .unwrap();
        create_button.click();
        
        // 4. Fill out create user form
        let name_input = app.query_selector("input[name='name']").unwrap()
            .unwrap();
        name_input.set_value("Test User");
        
        let email_input = app.query_selector("input[name='email']").unwrap()
            .unwrap();
        email_input.set_value("test@example.com");
        
        // 5. Submit form
        let submit_button = app.query_selector("button[type='submit']").unwrap()
            .unwrap();
        submit_button.click();
        
        // 6. Wait for success message
        app.wait_for_selector(".success-message", 5000).await.unwrap();
        
        // 7. Verify user appears in list
        app.wait_for_text(".user-list", "Test User", 5000).await.unwrap();
        
        // 8. Test error handling - try to create duplicate user
        create_button.click();
        name_input.set_value("Test User");
        email_input.set_value("test@example.com");
        submit_button.click();
        
        // 9. Should show error message
        app.wait_for_selector(".error-message", 5000).await.unwrap();
        let error_message = app.query_selector(".error-message").unwrap()
            .unwrap();
        assert!(error_message.text_content().unwrap().contains("already exists"));
    }

    #[wasm_bindgen_test]
    async fn test_offline_behavior() {
        let app = create_test_app();
        
        // 1. Load data while online
        app.wait_for_selector(".user-list", 5000).await.unwrap();
        
        // 2. Go offline
        app.set_network_offline().await;
        
        // 3. Try to create user (should queue)
        let create_button = app.query_selector(".create-user-button").unwrap()
            .unwrap();
        create_button.click();
        
        // Fill form and submit
        app.fill_form(".create-user-form", &[
            ("name", "Offline User"),
            ("email", "offline@example.com"),
        ]).await;
        
        // 4. Should show queued message
        app.wait_for_selector(".mutation-queued", 5000).await.unwrap();
        
        // 5. Come back online
        app.set_network_online().await;
        
        // 6. Should automatically sync and show user
        app.wait_for_text(".user-list", "Offline User", 10000).await.unwrap();
    }

    // Mock test application helper
    struct TestApp {
        // Test app implementation
    }

    fn create_test_app() -> TestApp {
        TestApp {
            // Initialize test application with mock API
        }
    }

    impl TestApp {
        async fn query_selector(&self, selector: &str) -> Option<web_sys::Element> {
            // Implementation
        }

        async fn wait_for_selector(&self, selector: &str, timeout: u32) -> Result<(), JsValue> {
            // Implementation
        }

        async fn wait_for_text(&self, selector: &str, text: &str, timeout: u32) -> Result<(), JsValue> {
            // Implementation
        }

        async fn fill_form(&self, form_selector: &str, fields: &[(&str, &str)]) {
            // Implementation
        }

        async fn set_network_offline(&self) {
            // Mock network offline
        }

        async fn set_network_online(&self) {
            // Mock network online
        }
    }
}
```

### 2.4 Property Tests (5% of test suite)

#### Cache Invariants

```rust
// tests/property/cache_invariants.rs

use proptest::prelude::*;
use leptos_query::*;

proptest! {
    #[test]
    fn test_serialization_roundtrip(data: Vec<u8>) {
        let serialized = SerializedData::serialize(&data)?;
        let deserialized: Vec<u8> = serialized.deserialize()?;
        prop_assert_eq!(data, deserialized);
    }

    #[test]
    fn test_query_key_consistency(segments: Vec<String>) {
        let key1 = QueryKey::new(segments.clone());
        let key2 = QueryKey::new(segments);
        prop_assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_entry_staleness(
        stale_time_ms: u64,
        age_ms: u64
    ) {
        let stale_time = Duration::from_millis(stale_time_ms.min(86400000)); // Max 1 day
        let age = Duration::from_millis(age_ms.min(86400000));
        
        let entry = CacheEntry {
            data: Some(SerializedData::serialize(&"test")?),
            error: None,
            state: QueryState::Success,
            updated_at: Instant::now() - age,
            data_updated_at: Instant::now() - age,
            stale_time,
            cache_time: Duration::from_secs(3600),
            meta: QueryMeta::default(),
        };
        
        let expected_stale = age > stale_time;
        prop_assert_eq!(entry.is_stale(), expected_stale);
    }

    #[test]
    fn test_retry_delay_monotonic(
        initial_ms: u32,
        multiplier: f64,
        attempt: u32,
    ) {
        // Constrain inputs to reasonable ranges
        let initial_ms = initial_ms.min(60000); // Max 1 minute
        let multiplier = multiplier.max(1.0).min(10.0); // 1.0 to 10.0
        let attempt = attempt.min(10); // Max 10 attempts
        
        let delay = RetryDelay::Exponential {
            initial: Duration::from_millis(initial_ms as u64),
            multiplier,
            max: Duration::from_secs(300), // 5 minutes max
        };
        
        if attempt > 0 {
            let delay1 = delay.calculate(attempt - 1, false);
            let delay2 = delay.calculate(attempt, false);
            
            // Delays should be monotonically increasing (until max)
            prop_assert!(delay2 >= delay1);
        }
    }
}
```

## 3. Testing Infrastructure

### 3.1 Test Utilities

```rust
// tests/utils/mod.rs

pub struct TestRuntime {
    runtime: tokio::runtime::Runtime,
}

impl TestRuntime {
    pub fn new() -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }

    pub fn block_on<F, T>(&self, future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        self.runtime.block_on(future)
    }
}

pub struct MockApiServer {
    server: wiremock::MockServer,
}

impl MockApiServer {
    pub async fn start() -> Self {
        Self {
            server: wiremock::MockServer::start().await,
        }
    }

    pub fn mock_get_users(&self) -> wiremock::Mock {
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/api/users"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([
                        {"id": 1, "name": "John Doe", "email": "john@example.com"},
                        {"id": 2, "name": "Jane Smith", "email": "jane@example.com"}
                    ]))
            )
    }

    pub fn mock_create_user(&self) -> wiremock::Mock {
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/api/users"))
            .respond_with(
                wiremock::ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({
                        "id": 3,
                        "name": "New User",
                        "email": "newuser@example.com"
                    }))
            )
    }

    pub fn url(&self) -> String {
        self.server.uri()
    }
}

pub fn create_test_client() -> QueryClient {
    QueryClient::new(QueryClientConfig {
        default_stale_time: Duration::from_secs(0), // Always stale for testing
        default_cache_time: Duration::from_secs(300),
        gc_interval: Duration::from_secs(1), // Fast GC for testing
        max_cache_size: Some(100),
        default_retry: RetryConfig {
            max_attempts: 2, // Fewer retries for faster tests
            delay: RetryDelay::Fixed(Duration::from_millis(10)),
            retryable_errors: Box::new(|_| true),
            jitter: false, // Deterministic for testing
        },
    })
}
```

### 3.2 Test Data Management

```rust
// tests/fixtures/mod.rs

use leptos_query::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestUser {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestPost {
    pub id: u32,
    pub user_id: u32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

pub struct TestDataSet {
    pub users: Vec<TestUser>,
    pub posts: Vec<TestPost>,
}

impl TestDataSet {
    pub fn sample() -> Self {
        Self {
            users: vec![
                TestUser {
                    id: 1,
                    name: "John Doe".into(),
                    email: "john@example.com".into(),
                    created_at: "2024-01-01T00:00:00Z".into(),
                },
                TestUser {
                    id: 2,
                    name: "Jane Smith".into(),
                    email: "jane@example.com".into(),
                    created_at: "2024-01-02T00:00:00Z".into(),
                },
            ],
            posts: vec![
                TestPost {
                    id: 1,
                    user_id: 1,
                    title: "First Post".into(),
                    body: "This is the first post content".into(),
                    published: true,
                },
                TestPost {
                    id: 2,
                    user_id: 1,
                    title: "Second Post".into(),
                    body: "This is the second post content".into(),
                    published: false,
                },
            ],
        }
    }

    pub fn populate_client(&self, client: &QueryClient) -> Result<(), QueryError> {
        // Populate cache with test data
        client.set_query_data(&QueryKey::new(["users"]), &self.users)?;
        
        for user in &self.users {
            let user_key = QueryKey::new(["users", user.id.to_string()]);
            client.set_query_data(&user_key, user)?;
            
            let user_posts: Vec<_> = self.posts
                .iter()
                .filter(|p| p.user_id == user.id)
                .cloned()
                .collect();
            
            let posts_key = QueryKey::new(["users", user.id.to_string(), "posts"]);
            client.set_query_data(&posts_key, &user_posts)?;
        }
        
        client.set_query_data(&QueryKey::new(["posts"]), &self.posts)?;
        
        Ok(())
    }
}
```

## 4. Continuous Testing

### 4.1 Automated Testing Pipeline

```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  unit-tests:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run unit tests
        run: cargo test --lib --all-features
      - name: Generate coverage
        run: cargo tarpaulin --out Xml --skip-clean

  integration-tests:
    name: Integration Tests  
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run integration tests
        run: cargo test --test integration --all-features

  wasm-tests:
    name: WASM Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      - name: Run WASM tests
        run: |
          wasm-pack test --headless --chrome
          wasm-pack test --headless --firefox

  e2e-tests:
    name: E2E Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Install Playwright
        run: npx playwright install
      - name: Run E2E tests
        run: cargo test --test e2e --all-features

  property-tests:
    name: Property Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run property tests
        run: cargo test --test property --all-features --release
```

### 4.2 Performance Testing

```rust
// benches/query_performance.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use leptos_query::*;

fn benchmark_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("set_query_data", size),
            size,
            |b, &size| {
                let client = create_test_client();
                let data = vec![0u8; size];
                
                b.iter(|| {
                    let key = QueryKey::new(["bench", "data"]);
                    client.set_query_data(&key, black_box(&data)).unwrap();
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("get_query_data", size),
            size,
            |b, &size| {
                let client = create_test_client();
                let key = QueryKey::new(["bench", "data"]);
                let data = vec![0u8; size];
                client.set_query_data(&key, &data).unwrap();
                
                b.iter(|| {
                    black_box(client.get_query_data::<Vec<u8>>(&key));
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_cache_operations);
criterion_main!(benches);
```

## 5. Test Metrics and Monitoring

### 5.1 Coverage Reporting

```bash
# Generate coverage report
cargo tarpaulin --all-features --out Html --output-dir target/coverage

# Upload to codecov
bash <(curl -s https://codecov.io/bash) -f target/coverage/cobertura.xml
```

### 5.2 Test Performance Monitoring

```rust
// tests/metrics/mod.rs

pub struct TestMetrics {
    pub test_duration: Duration,
    pub memory_usage: u64,
    pub cache_hit_rate: f64,
}

impl TestMetrics {
    pub fn collect() -> Self {
        Self {
            test_duration: Duration::from_secs(0), // Implement actual measurement
            memory_usage: 0, // Implement actual measurement
            cache_hit_rate: 0.0, // Implement actual measurement
        }
    }

    pub fn report(&self) {
        println!("Test Duration: {:?}", self.test_duration);
        println!("Memory Usage: {} bytes", self.memory_usage);
        println!("Cache Hit Rate: {:.2}%", self.cache_hit_rate * 100.0);
    }
}
```

## 6. Testing Best Practices

### 6.1 Test Organization

1. **One Test, One Concern**: Each test should verify one specific behavior
2. **Descriptive Names**: Test names should clearly describe what they're testing
3. **Arrange-Act-Assert**: Structure tests with clear setup, action, and verification
4. **Independent Tests**: Tests should not depend on each other
5. **Fast Tests**: Unit tests should run in milliseconds

### 6.2 Mock and Stub Guidelines

```rust
// Good: Focused mock
#[tokio::test]
async fn test_network_error_handling() {
    let mut server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api/users"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    
    let result = fetch_users(&server.uri()).await;
    assert!(matches!(result, Err(QueryError::Http { status: 500, .. })));
}

// Bad: Over-mocking
#[tokio::test]
async fn test_everything_with_mocks() {
    // Don't mock everything - test real interactions where possible
}
```

### 6.3 Flaky Test Prevention

```rust
// Good: Deterministic test
#[test]
fn test_cache_expiry() {
    let mut entry = create_cache_entry();
    entry.updated_at = Instant::now() - Duration::from_secs(301);
    entry.cache_time = Duration::from_secs(300);
    
    assert!(entry.is_expired());
}

// Bad: Time-dependent test
#[tokio::test]
async fn test_cache_expiry_flaky() {
    let entry = create_cache_entry();
    tokio::time::sleep(Duration::from_millis(301)).await; // Flaky!
    assert!(entry.is_expired());
}
```

This comprehensive testing strategy ensures Leptos Query maintains high quality, reliability, and performance across all supported platforms and use cases.