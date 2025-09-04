# Test-Driven Development (TDD) Implementation Guide

## 🚀 Overview

This guide demonstrates how to implement comprehensive Test-Driven Development in Rust projects, based on the successful TDD implementation in `leptos-query-rs`. The guide covers multiple testing approaches, best practices, and practical examples.

## 🎯 What is TDD?

Test-Driven Development is a software development methodology that follows this cycle:

1. **Red**: Write a failing test
2. **Green**: Write minimal code to make the test pass
3. **Refactor**: Improve the code while keeping tests passing

## 🏗️ TDD Infrastructure Setup

### 1. Project Structure

Organize your tests by category for maintainability:

```
your-project/
├── src/
│   └── lib.rs
├── tests/
│   ├── unit/           # Unit tests for individual modules
│   ├── integration/    # Integration tests between modules
│   ├── property/       # Property-based tests
│   ├── mutation/       # Mutation tests for quality validation
│   └── utils/          # Shared test utilities
├── benches/            # Performance benchmarks
├── examples/           # Working examples
└── Cargo.toml
```

### 2. Dependencies Configuration

Add testing dependencies to `Cargo.toml`:

```toml
[dev-dependencies]
# Core testing
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Property-based testing
proptest = "1.4"

# Performance testing
criterion = { version = "0.7", features = ["html_reports"] }

# E2E testing (if applicable)
wasm-bindgen-test = "0.3"

# Test utilities
tempfile = "3.0"
rand = "0.8"

[[test]]
name = "unit_tests"
harness = false

[[test]]
name = "integration_tests"
harness = false

[[test]]
name = "property_tests"
harness = false

[[test]]
name = "mutation_tests"
harness = false

[[bench]]
name = "performance_benchmarks"
harness = false
```

## 🧪 Testing Categories Implementation

### 1. Unit Tests

Unit tests focus on testing individual functions and modules in isolation.

#### 1.1 Basic Unit Test Structure

```rust
// tests/unit/module_tests.rs
use your_crate::module::{Function, Struct};

#[test]
fn test_function_basic_behavior() {
    // Arrange
    let input = "test_input";
    
    // Act
    let result = Function::process(input);
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "expected_output");
}

#[test]
fn test_function_edge_cases() {
    // Test empty input
    let result = Function::process("");
    assert!(result.is_err());
    
    // Test very long input
    let long_input = "a".repeat(10000);
    let result = Function::process(&long_input);
    assert!(result.is_ok());
}

#[test]
fn test_struct_creation() {
    let instance = Struct::new("name", 42);
    assert_eq!(instance.name(), "name");
    assert_eq!(instance.value(), 42);
}
```

#### 1.2 Async Unit Tests

```rust
#[tokio::test]
async fn test_async_function() {
    // Arrange
    let input = "async_input";
    
    // Act
    let result = Function::process_async(input).await;
    
    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let handles: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                Function::process_async(&format!("input_{}", i)).await
            })
        })
        .collect();
    
    let results = futures::future::join_all(handles).await;
    
    for result in results {
        assert!(result.unwrap().is_ok());
    }
}
```

### 2. Integration Tests

Integration tests verify that multiple modules work together correctly.

#### 2.1 Basic Integration Test

```rust
// tests/integration/module_integration.rs
use your_crate::{ModuleA, ModuleB, IntegrationLayer};

#[test]
fn test_modules_work_together() {
    // Arrange
    let module_a = ModuleA::new();
    let module_b = ModuleB::new();
    let integration = IntegrationLayer::new(module_a, module_b);
    
    // Act
    let result = integration.process_data("test_data");
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status(), "processed");
}

#[tokio::test]
async fn test_async_integration() {
    let integration = IntegrationLayer::new_async().await;
    
    let result = integration.process_async("async_data").await;
    assert!(result.is_ok());
}
```

#### 2.2 Error Handling Integration

```rust
#[test]
fn test_error_propagation() {
    let integration = IntegrationLayer::new_with_error_handling();
    
    // Test that errors from ModuleA propagate correctly
    let result = integration.process_with_error("invalid_data");
    assert!(result.is_err());
    
    match result.unwrap_err() {
        IntegrationError::ModuleAError(_) => (), // Expected
        _ => panic!("Unexpected error type"),
    }
}
```

### 3. Property-Based Testing

Property-based testing uses `proptest` to generate random inputs and verify invariants.

#### 3.1 Basic Property Tests

```rust
// tests/property/invariants.rs
use proptest::prelude::*;
use your_crate::module::Function;

proptest! {
    #[test]
    fn test_function_idempotent(input: String) {
        // Property: Applying the function twice should give the same result
        let first_result = Function::process(&input);
        let second_result = Function::process(&input);
        
        prop_assert_eq!(first_result, second_result);
    }
    
    #[test]
    fn test_function_preserves_length(input: String) {
        // Property: Output length should be related to input length
        if let Ok(result) = Function::process(&input) {
            prop_assert!(result.len() >= input.len());
        }
    }
    
    #[test]
    fn test_function_commutative(a: String, b: String) {
        // Property: Function should be commutative for certain operations
        let result_ab = Function::combine(&a, &b);
        let result_ba = Function::combine(&b, &a);
        
        prop_assert_eq!(result_ab, result_ba);
    }
}
```

#### 3.2 Advanced Property Tests

```rust
proptest! {
    #[test]
    fn test_cache_invariants(
        keys in prop::collection::vec(any::<String>(), 0..100),
        values in prop::collection::vec(any::<u32>(), 0..100)
    ) {
        let mut cache = Cache::new();
        
        // Insert data
        for (key, value) in keys.iter().zip(values.iter()) {
            cache.insert(key, *value);
        }
        
        // Property: All inserted values should be retrievable
        for (key, expected_value) in keys.iter().zip(values.iter()) {
            prop_assert_eq!(cache.get(key), Some(expected_value));
        }
        
        // Property: Cache size should not exceed capacity
        prop_assert!(cache.len() <= cache.capacity());
    }
    
    #[test]
    fn test_serialization_roundtrip(data: TestData) {
        // Property: Serialization followed by deserialization should reproduce original data
        let serialized = serde_json::to_string(&data).unwrap();
        let deserialized: TestData = serde_json::from_str(&serialized).unwrap();
        
        prop_assert_eq!(data, deserialized);
    }
}
```

### 4. Performance Testing

Performance testing uses `criterion` to benchmark critical operations.

#### 4.1 Basic Benchmarks

```rust
// benches/performance_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use your_crate::module::Function;

fn benchmark_basic_operation(c: &mut Criterion) {
    c.bench_function("basic_operation", |b| {
        b.iter(|| {
            let input = "benchmark_input";
            black_box(Function::process(input))
        })
    });
}

fn benchmark_with_different_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("operation_by_size");
    
    for size in [10, 100, 1000, 10000].iter() {
        let input = "a".repeat(*size);
        group.bench_with_input(
            format!("size_{}", size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(Function::process(&input))
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_basic_operation, benchmark_with_different_sizes);
criterion_main!(benches);
```

#### 4.2 Advanced Benchmarks

```rust
fn benchmark_concurrent_operations(c: &mut Criterion) {
    c.bench_function("concurrent_operations", |b| {
        b.iter(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let handles: Vec<_> = (0..100)
                    .map(|i| {
                        tokio::spawn(async move {
                            Function::process_async(&format!("input_{}", i)).await
                        })
                    })
                    .collect();
                
                let results = futures::future::join_all(handles).await;
                black_box(results)
            })
        })
    });
}

fn benchmark_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_operations", |b| {
        b.iter(|| {
            let mut cache = Cache::new();
            
            // Insert data
            for i in 0..1000 {
                cache.insert(format!("key_{}", i), i);
            }
            
            // Access data
            for i in 0..1000 {
                black_box(cache.get(&format!("key_{}", i)));
            }
            
            // Clear cache
            cache.clear();
        })
    });
}
```

### 5. Mutation Testing

Mutation testing validates test quality by introducing intentional bugs.

#### 5.1 Manual Mutation Tests

```rust
// tests/mutation/manual_mutation_tests.rs
use your_crate::module::{Function, Cache};

#[test]
fn test_cache_operations_catch_mutations() {
    let mut cache = Cache::new();
    
    // Original behavior
    cache.insert("key", "value");
    assert_eq!(cache.get("key"), Some("value"));
    
    // Test that our tests would catch if insert was broken
    // This simulates a mutation where insert doesn't work
    // In real mutation testing, this would be automated
    
    // Test that our tests would catch if get was broken
    // This simulates a mutation where get returns wrong value
}

#[test]
fn test_function_behavior_catches_mutations() {
    // Test that our tests would catch various function mutations
    let result = Function::process("input");
    
    // Test would catch if function returned wrong result
    assert_eq!(result.unwrap(), "expected_output");
    
    // Test would catch if function panicked
    let result = Function::process("another_input");
    assert!(result.is_ok());
}
```

## 🔄 TDD Workflow Implementation

### 1. Red-Green-Refactor Cycle

#### Step 1: Write Failing Test (Red)

```rust
#[test]
fn test_new_feature() {
    // This test will fail because the feature doesn't exist yet
    let result = NewFeature::process("input");
    assert_eq!(result.unwrap(), "expected_output");
}
```

#### Step 2: Write Minimal Implementation (Green)

```rust
// In src/lib.rs
pub struct NewFeature;

impl NewFeature {
    pub fn process(input: &str) -> Result<String, Error> {
        // Minimal implementation to make test pass
        Ok("expected_output".to_string())
    }
}
```

#### Step 3: Refactor (Refactor)

```rust
impl NewFeature {
    pub fn process(input: &str) -> Result<String, Error> {
        // Improved implementation while keeping tests passing
        if input.is_empty() {
            return Err(Error::EmptyInput);
        }
        
        Ok(format!("processed_{}", input))
    }
}
```

### 2. Test-First Development Example

Let's implement a simple cache with TDD:

#### Test 1: Basic Cache Creation

```rust
#[test]
fn test_cache_creation() {
    let cache = Cache::new();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}
```

#### Implementation 1: Basic Cache

```rust
pub struct Cache {
    data: HashMap<String, String>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
```

#### Test 2: Insert and Retrieve

```rust
#[test]
fn test_cache_insert_and_retrieve() {
    let mut cache = Cache::new();
    cache.insert("key", "value");
    
    assert_eq!(cache.get("key"), Some("value"));
    assert_eq!(cache.len(), 1);
}
```

#### Implementation 2: Insert and Retrieve

```rust
impl Cache {
    pub fn insert(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}
```

#### Test 3: Cache Eviction

```rust
#[test]
fn test_cache_eviction() {
    let mut cache = Cache::with_capacity(2);
    
    cache.insert("key1", "value1");
    cache.insert("key2", "value2");
    cache.insert("key3", "value3"); // Should evict key1
    
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get("key1"), None);
    assert_eq!(cache.get("key2"), Some("value2"));
    assert_eq!(cache.get("key3"), Some("value3"));
}
```

#### Implementation 3: Cache with Eviction

```rust
use std::collections::HashMap;

pub struct Cache {
    data: HashMap<String, String>,
    capacity: usize,
}

impl Cache {
    pub fn new() -> Self {
        Self::with_capacity(100)
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::new(),
            capacity,
        }
    }
    
    pub fn insert(&mut self, key: &str, value: &str) {
        if self.data.len() >= self.capacity && !self.data.contains_key(key) {
            // Simple eviction: remove first key
            if let Some(first_key) = self.data.keys().next().cloned() {
                self.data.remove(&first_key);
            }
        }
        
        self.data.insert(key.to_string(), value.to_string());
    }
}
```

## 🎯 Best Practices

### 1. Test Organization

- **Group related tests** in the same module
- **Use descriptive test names** that explain the behavior
- **Follow AAA pattern**: Arrange, Act, Assert
- **Keep tests independent** and isolated

### 2. Test Data Management

```rust
// tests/utils/mod.rs
pub struct TestData {
    pub id: u32,
    pub name: String,
    pub value: f64,
}

impl TestData {
    pub fn new(id: u32, name: &str, value: f64) -> Self {
        Self {
            id,
            name: name.to_string(),
            value,
        }
    }
    
    pub fn sample() -> Vec<Self> {
        vec![
            Self::new(1, "Alice", 100.0),
            Self::new(2, "Bob", 200.0),
            Self::new(3, "Charlie", 300.0),
        ]
    }
}
```

### 3. Mock and Stub Implementation

```rust
pub trait DataProvider {
    async fn fetch_data(&self, id: u32) -> Result<TestData, Error>;
}

pub struct MockDataProvider {
    data: HashMap<u32, TestData>,
}

impl MockDataProvider {
    pub fn new() -> Self {
        let mut data = HashMap::new();
        data.insert(1, TestData::new(1, "Mock Alice", 100.0));
        data.insert(2, TestData::new(2, "Mock Bob", 200.0));
        
        Self { data }
    }
}

impl DataProvider for MockDataProvider {
    async fn fetch_data(&self, id: u32) -> Result<TestData, Error> {
        self.data.get(&id)
            .cloned()
            .ok_or(Error::NotFound)
    }
}
```

### 4. Test Configuration

```rust
// tests/config.rs
pub struct TestConfig {
    pub timeout: Duration,
    pub retry_count: u32,
    pub log_level: log::Level,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            retry_count: 3,
            log_level: log::Level::Info,
        }
    }
}

pub fn setup_test_environment(config: TestConfig) {
    // Initialize logging
    env_logger::Builder::new()
        .filter_level(config.log_level)
        .init();
    
    // Set up test timeouts
    std::env::set_var("TEST_TIMEOUT", config.timeout.as_secs().to_string());
}
```

## 🚨 Common Pitfalls and Solutions

### 1. Test Flakiness

**Problem**: Tests that sometimes pass and sometimes fail.

**Solutions**:
- Use deterministic test data
- Avoid time-based tests
- Mock external dependencies
- Use proper cleanup in tests

```rust
#[test]
fn test_with_deterministic_data() {
    // Use fixed seed for random number generation
    let mut rng = StdRng::seed_from_u64(42);
    
    // Generate deterministic test data
    let test_data = generate_test_data(&mut rng);
    
    // Test with deterministic data
    let result = process_data(test_data);
    assert!(result.is_ok());
}
```

### 2. Slow Tests

**Problem**: Tests that take too long to run.

**Solutions**:
- Use smaller test datasets
- Mock expensive operations
- Run slow tests separately
- Use test parallelization

```rust
// Separate slow tests
#[test]
#[ignore]
fn test_slow_operation() {
    // This test is slow and runs separately
    let result = slow_operation();
    assert!(result.is_ok());
}

// Fast tests run by default
#[test]
fn test_fast_operation() {
    let result = fast_operation();
    assert!(result.is_ok());
}
```

### 3. Test Maintenance

**Problem**: Tests that break when implementation changes.

**Solutions**:
- Test behavior, not implementation
- Use stable interfaces
- Abstract test setup
- Keep tests focused

```rust
// Good: Test behavior
#[test]
fn test_user_creation() {
    let user = User::new("Alice", "alice@example.com");
    assert_eq!(user.name(), "Alice");
    assert_eq!(user.email(), "alice@example.com");
}

// Bad: Test implementation details
#[test]
fn test_user_internal_fields() {
    let user = User::new("Alice", "alice@example.com");
    // Don't test internal field values directly
    // assert_eq!(user.name_field, "Alice"); // Implementation detail
}
```

## 📊 Measuring TDD Success

### 1. Test Coverage Metrics

```bash
# Install cargo-tarpaulin for coverage
cargo install cargo-tarpaulin

# Run coverage analysis
cargo tarpaulin --out Html --output-dir coverage
```

### 2. Performance Metrics

```bash
# Run benchmarks
cargo bench

# Compare with previous runs
cargo bench -- --save-baseline new
cargo bench -- --baseline new
```

### 3. Quality Metrics

- **Test Pass Rate**: Should be 100%
- **Code Coverage**: Aim for >90%
- **Performance Regressions**: 0% tolerance
- **Bug Reports**: Reduced frequency

## 🔮 Advanced TDD Techniques

### 1. Contract Testing

```rust
pub trait DataContract {
    fn validate_input(&self, input: &str) -> Result<(), ValidationError>;
    fn process_data(&self, input: &str) -> Result<String, ProcessingError>;
    fn validate_output(&self, output: &str) -> Result<(), ValidationError>;
}

impl<T: DataContract> T {
    pub fn process_with_validation(&self, input: &str) -> Result<String, Error> {
        // Validate input
        self.validate_input(input)?;
        
        // Process data
        let output = self.process_data(input)?;
        
        // Validate output
        self.validate_output(&output)?;
        
        Ok(output)
    }
}
```

### 2. Behavior-Driven Development (BDD)

```rust
use cucumber::{given, when, then};

#[given("a user with name {string}")]
async fn given_user_with_name(world: &mut MyWorld, name: String) {
    world.user = Some(User::new(&name, "test@example.com"));
}

#[when("the user creates a post with title {string}")]
async fn when_user_creates_post(world: &mut MyWorld, title: String) {
    if let Some(user) = &world.user {
        world.post = Some(user.create_post(&title, "Post content"));
    }
}

#[then("the post should be saved")]
async fn then_post_should_be_saved(world: &mut MyWorld) {
    assert!(world.post.is_some());
    // Additional assertions...
}
```

## 📚 Resources and Further Reading

### 1. Rust Testing Resources
- [Rust Book - Testing Chapter](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust by Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/)

### 2. TDD Resources
- "Test-Driven Development: By Example" by Kent Beck
- "Growing Object-Oriented Software, Guided by Tests" by Steve Freeman and Nat Pryce
- "Working Effectively with Legacy Code" by Michael Feathers

### 3. Property-Based Testing
- [Proptest Book](https://altsysrq.github.io/proptest-book/)
- "Property-Based Testing with PropEr, Erlang, and Elixir" by Fred Hebert

---

**This guide demonstrates how to implement comprehensive TDD in Rust projects. The key is to start simple, build incrementally, and maintain high test quality throughout the development process.**
