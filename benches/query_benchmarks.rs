use criterion::{black_box, criterion_group, criterion_main, Criterion};
use leptos::*;
use leptos_query::*;
use std::time::Duration;

// Benchmark data structures
#[derive(Clone, Debug)]
struct BenchmarkUser {
    id: u32,
    name: String,
    email: String,
}

// Simulated API function for benchmarking
async fn benchmark_fetch_user(id: u32) -> Result<BenchmarkUser, String> {
    // Simulate network delay
    std::thread::sleep(Duration::from_millis(10));
    
    Ok(BenchmarkUser {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

// Benchmark: Query creation and execution
fn benchmark_query_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_creation");
    
    group.bench_function("create_query", |b| {
        b.iter(|| {
            let app = create_runtime();
            let _ = app.run_scope(|cx| {
                let _query = use_query(
                    || &["users", "1"][..],
                    || || async move { benchmark_fetch_user(1).await },
                    QueryOptions::default()
                );
                cx
            });
            app.dispose();
        });
    });
    
    group.finish();
}

// Benchmark: Query cache performance
fn benchmark_query_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_cache");
    
    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            let app = create_runtime();
            let _ = app.run_scope(|cx| {
                // First query to populate cache
                let query1 = use_query(
                    || &["users", "1"][..],
                    || || async move { benchmark_fetch_user(1).await },
                    QueryOptions::default()
                );
                
                // Second query to test cache hit
                let query2 = use_query(
                    || &["users", "1"][..],
                    || || async move { benchmark_fetch_user(1).await },
                    QueryOptions::default()
                );
                
                black_box((query1, query2));
                cx
            });
            app.dispose();
        });
    });
    
    group.finish();
}

// Benchmark: Multiple concurrent queries
fn benchmark_concurrent_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_queries");
    
    group.bench_function("multiple_users", |b| {
        b.iter(|| {
            let app = create_runtime();
            let _ = app.run_scope(|cx| {
                let queries: Vec<_> = (1..=10)
                    .map(|id| {
                        use_query(
                            move || &["users", &id.to_string()][..],
                            move || || async move { benchmark_fetch_user(id).await },
                            QueryOptions::default()
                        )
                    })
                    .collect();
                
                black_box(queries);
                cx
            });
            app.dispose();
        });
    });
    
    group.finish();
}

// Benchmark: Query invalidation
fn benchmark_query_invalidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_invalidation");
    
    group.bench_function("invalidate_single", |b| {
        b.iter(|| {
            let app = create_runtime();
            let _ = app.run_scope(|cx| {
                let query_client = use_query_client();
                query_client.invalidate_queries(&["users", "1"]);
                cx
            });
            app.dispose();
        });
    });
    
    group.bench_function("invalidate_pattern", |b| {
        b.iter(|| {
            let app = create_runtime();
            let _ = app.run_scope(|cx| {
                let query_client = use_query_client();
                query_client.invalidate_queries(&["users"]);
                cx
            });
            app.dispose();
        });
    });
    
    group.finish();
}

// Benchmark: Mutation performance
fn benchmark_mutations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutations");
    
    async fn benchmark_create_user(user_data: &(String, String)) -> Result<BenchmarkUser, String> {
        let (name, email) = user_data.clone();
        std::thread::sleep(Duration::from_millis(5));
        
        Ok(BenchmarkUser {
            id: rand::random::<u32>(),
            name,
            email,
        })
    }
    
    group.bench_function("create_mutation", |b| {
        b.iter(|| {
            let app = create_runtime();
            let _ = app.run_scope(|cx| {
                let _mutation = use_mutation(
                    |user_data: &(String, String)| {
                        let data = user_data.clone();
                        async move { benchmark_create_user(&data).await }
                    },
                    MutationOptions::default()
                );
                cx
            });
            app.dispose();
        });
    });
    
    group.finish();
}

// Benchmark: Memory usage patterns
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    group.bench_function("large_dataset", |b| {
        b.iter(|| {
            let app = create_runtime();
            let _ = app.run_scope(|cx| {
                // Simulate large dataset queries
                let queries: Vec<_> = (1..=100)
                    .map(|id| {
                        use_query(
                            move || &["users", &id.to_string()][..],
                            move || || async move { benchmark_fetch_user(id).await },
                            QueryOptions::default()
                        )
                    })
                    .collect();
                
                black_box(queries);
                cx
            });
            app.dispose();
        });
    });
    
    group.finish();
}

// Benchmark: Query key generation
fn benchmark_query_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_keys");
    
    group.bench_function("simple_key", |b| {
        b.iter(|| {
            let key = &["users", "1"][..];
            black_box(key);
        });
    });
    
    group.bench_function("complex_key", |b| {
        b.iter(|| {
            let key = &["users", "1", "posts", "comments"][..];
            black_box(key);
        });
    });
    
    group.bench_function("dynamic_key", |b| {
        b.iter(|| {
            let user_id = 1u32;
            let key = &["users", &user_id.to_string()][..];
            black_box(key);
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
        benchmark_query_creation,
        benchmark_query_cache,
        benchmark_concurrent_queries,
        benchmark_query_invalidation,
        benchmark_mutations,
        benchmark_memory_usage,
        benchmark_query_keys
);

criterion_main!(benches);
