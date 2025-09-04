use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use leptos_query_rs::*;
use leptos_query_rs::retry::{QueryError, should_retry_error};
use leptos_query_rs::types::{QueryKey, QueryKeyPattern};
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
            let matches = key.matches_pattern(&pattern);
            black_box(matches);
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

// Benchmark: Cache operations with different data sizes
fn benchmark_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("set_query_data", size),
            size,
            |b, &size| {
                let client = QueryClient::new();
                let data = vec![0u8; size];
                let key = QueryKey::new(&["bench", "data"]);
                
                b.iter(|| {
                    let _ = client.set_query_data(&key, black_box(&data));
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("get_query_data", size),
            size,
            |b, &size| {
                let client = QueryClient::new();
                let key = QueryKey::new(&["bench", "data"]);
                let data = vec![0u8; size];
                let _ = client.set_query_data(&key, &data);
                
                b.iter(|| {
                    let entry = client.get_cache_entry(&key);
                    black_box(entry);
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark: Cache invalidation patterns
fn benchmark_cache_invalidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_invalidation");
    
    group.bench_function("exact_invalidation", |b| {
        let client = QueryClient::new();
        let data = BenchmarkUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // Pre-populate cache
        for i in 0..100 {
            let key = QueryKey::new(&["users", &i.to_string()]);
            let _ = client.set_query_data(&key, data.clone());
        }
        
        b.iter(|| {
            let key = QueryKey::new(&["users", "50"]);
            let pattern = QueryKeyPattern::Exact(key);
            client.invalidate_queries(&pattern);
        });
    });
    
    group.bench_function("prefix_invalidation", |b| {
        let client = QueryClient::new();
        let data = BenchmarkUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // Pre-populate cache
        for i in 0..100 {
            let key = QueryKey::new(&["users", &i.to_string()]);
            let _ = client.set_query_data(&key, data.clone());
        }
        
        b.iter(|| {
            let pattern = QueryKeyPattern::Prefix(QueryKey::new(&["users"]));
            client.invalidate_queries(&pattern);
        });
    });
    
    group.bench_function("contains_invalidation", |b| {
        let client = QueryClient::new();
        let data = BenchmarkUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // Pre-populate cache
        for i in 0..100 {
            let key = QueryKey::new(&["users", &i.to_string()]);
            let _ = client.set_query_data(&key, data.clone());
        }
        
        b.iter(|| {
            let pattern = QueryKeyPattern::Contains("5".to_string());
            client.invalidate_queries(&pattern);
        });
    });
    
    group.finish();
}

// Benchmark: Concurrent cache access
fn benchmark_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_access");
    
    group.bench_function("concurrent_reads", |b| {
        let client = QueryClient::new();
        let key = QueryKey::new(&["concurrent", "test"]);
        let data = BenchmarkUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        let _ = client.set_query_data(&key, data);
        
        b.iter(|| {
            // Simulate concurrent reads
            for _ in 0..10 {
                let entry = client.get_cache_entry(&key);
                black_box(entry);
            }
        });
    });
    
    group.bench_function("concurrent_writes", |b| {
        let client = QueryClient::new();
        let data = BenchmarkUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        b.iter(|| {
            // Simulate concurrent writes
            for i in 0..10 {
                let key = QueryKey::new(&["concurrent", &i.to_string()]);
                let _ = client.set_query_data(&key, data.clone());
            }
        });
    });
    
    group.finish();
}

// Benchmark: Memory usage and garbage collection
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    group.bench_function("cache_growth", |b| {
        let client = QueryClient::new();
        let data = BenchmarkUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        b.iter(|| {
            // Add many entries to test memory growth
            for i in 0..1000 {
                let key = QueryKey::new(&["memory", &i.to_string()]);
                let _ = client.set_query_data(&key, data.clone());
            }
            
            // Get stats
            let stats = client.cache_stats();
            black_box(stats);
        });
    });
    
    group.bench_function("cache_cleanup", |b| {
        let client = QueryClient::new();
        let data = BenchmarkUser {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // Pre-populate cache
        for i in 0..1000 {
            let key = QueryKey::new(&["cleanup", &i.to_string()]);
            let _ = client.set_query_data(&key, data.clone());
        }
        
        b.iter(|| {
            client.cleanup_stale_entries();
            let stats = client.cache_stats();
            black_box(stats);
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
        benchmark_query_options,
        benchmark_cache_operations,
        benchmark_cache_invalidation,
        benchmark_concurrent_access,
        benchmark_memory_usage
);

criterion_main!(benches);
