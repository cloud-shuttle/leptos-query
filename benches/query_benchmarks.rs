use criterion::{criterion_group, criterion_main, Criterion};
use leptos_query_rs::*;
use leptos_query_rs::retry::{QueryError, should_retry_error};
use std::hint::black_box;
use std::time::Duration;

// Benchmark data structures
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct BenchmarkUser {
    id: u32,
    name: String,
    email: String,
}

// Benchmark: Query key creation and manipulation
fn benchmark_query_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_keys");
    
    group.bench_function("simple_key", |b| {
        b.iter(|| {
            let key = QueryKey::new(&["users", "1"]);
            black_box(key);
        });
    });
    
    group.bench_function("complex_key", |b| {
        b.iter(|| {
            let key = QueryKey::new(&["users", "1", "posts", "comments"]);
            black_box(key);
        });
    });
    
    group.bench_function("dynamic_key", |b| {
        b.iter(|| {
            let user_id = 1u32;
            let key = QueryKey::new(&["users", &user_id.to_string()]);
            black_box(key);
        });
    });
    
    group.bench_function("key_pattern_matching", |b| {
        b.iter(|| {
            let key = QueryKey::new(&["users", "1"]);
            let pattern = QueryKeyPattern::Prefix(QueryKey::new(&["users"]));
            // Note: QueryKeyPattern doesn't have a matches method in current API
            black_box((key, pattern));
        });
    });
    
    group.finish();
}

// Benchmark: Query client operations
fn benchmark_query_client(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_client");
    
    group.bench_function("client_creation", |b| {
        b.iter(|| {
            let client = QueryClient::new();
            black_box(client);
        });
    });
    
    group.bench_function("cache_operations", |b| {
        b.iter(|| {
            let client = QueryClient::new();
            let key = QueryKey::new(&["test", "1"]);
            let data = BenchmarkUser {
                id: 1,
                name: "Test User".to_string(),
                email: "test@example.com".to_string(),
            };
            
            // Set data
            let _ = client.set_query_data(&key, data.clone());
            
            // Get data
            let entry = client.get_cache_entry(&key);
            let retrieved = entry.and_then(|e| e.get_data::<BenchmarkUser>().ok());
            
            black_box(retrieved);
        });
    });
    
    group.bench_function("cache_removal", |b| {
        b.iter(|| {
            let client = QueryClient::new();
            let key = QueryKey::new(&["test", "1"]);
            let data = BenchmarkUser {
                id: 1,
                name: "Test User".to_string(),
                email: "test@example.com".to_string(),
            };
            
            // Set data
            let _ = client.set_query_data(&key, data);
            
            // Remove data
            client.remove_query(&key);
            
            // Check if removed
            let entry = client.get_cache_entry(&key);
            black_box(entry);
        });
    });
    
    group.finish();
}

// Benchmark: Retry configuration and error handling
fn benchmark_retry_logic(c: &mut Criterion) {
    let mut group = c.benchmark_group("retry_logic");
    
    group.bench_function("retry_config_creation", |b| {
        b.iter(|| {
            let config = RetryConfig::new(3, Duration::from_millis(100))
                .with_max_delay(Duration::from_secs(1));
            black_box(config);
        });
    });
    
    group.bench_function("error_retryability", |b| {
        b.iter(|| {
            let config = RetryConfig::default();
            let network_error = QueryError::NetworkError("Connection failed".to_string());
            let timeout_error = QueryError::TimeoutError("Request timeout".to_string());
            let generic_error = QueryError::GenericError("Generic error".to_string());
            
            let network_retryable = should_retry_error(&network_error, &config);
            let timeout_retryable = should_retry_error(&timeout_error, &config);
            let generic_retryable = should_retry_error(&generic_error, &config);
            
            black_box((network_retryable, timeout_retryable, generic_retryable));
        });
    });
    
    group.finish();
}

// Benchmark: Serialization and deserialization
fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    group.bench_function("serde_json_serialize", |b| {
        b.iter(|| {
            let user = BenchmarkUser {
                id: 1,
                name: "Test User".to_string(),
                email: "test@example.com".to_string(),
            };
            let serialized = serde_json::to_string(&user).unwrap();
            black_box(serialized);
        });
    });
    
    group.bench_function("serde_json_deserialize", |b| {
        let json = r#"{"id":1,"name":"Test User","email":"test@example.com"}"#;
        b.iter(|| {
            let user: BenchmarkUser = serde_json::from_str(json).unwrap();
            black_box(user);
        });
    });
    
    group.bench_function("bincode_serialize", |b| {
        b.iter(|| {
            let user = BenchmarkUser {
                id: 1,
                name: "Test User".to_string(),
                email: "test@example.com".to_string(),
            };
            let serialized = bincode::serialize(&user).unwrap();
            black_box(serialized);
        });
    });
    
    group.bench_function("bincode_deserialize", |b| {
        let data = bincode::serialize(&BenchmarkUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }).unwrap();
        b.iter(|| {
            let user: BenchmarkUser = bincode::deserialize(&data).unwrap();
            black_box(user);
        });
    });
    
    group.finish();
}

// Benchmark: Query options and configuration
fn benchmark_query_options(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_options");
    
    group.bench_function("options_builder", |b| {
        b.iter(|| {
            let options = QueryOptions::default()
                .with_stale_time(Duration::from_secs(30))
                .with_cache_time(Duration::from_secs(60))
                .with_retry(RetryConfig::new(3, Duration::from_millis(100)));
            black_box(options);
        });
    });
    
    group.bench_function("mutation_options_builder", |b| {
        b.iter(|| {
            let options = MutationOptions::default()
                .with_retry(RetryConfig::new(2, Duration::from_millis(100)));
            black_box(options);
        });
    });
    
    group.finish();
}

// Configure criterion
criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2));
    targets = 
        benchmark_query_keys,
        benchmark_query_client,
        benchmark_retry_logic,
        benchmark_serialization,
        benchmark_query_options
);

criterion_main!(benches);
