# API Design Document
**Leptos Query - Data Fetching & Caching Library**

## Document Information
- **Version**: 1.0
- **Date**: September 2024
- **Status**: Draft
- **Authors**: CloudShuttle Team

## 1. API Design Philosophy

### 1.1 Core Principles
1. **Familiarity**: API closely follows TanStack Query patterns for easy adoption
2. **Type Safety**: Leverage Rust's type system for compile-time guarantees
3. **Reactive Integration**: Deep integration with Leptos signals and effects
4. **Progressive Enhancement**: Start simple, add complexity as needed
5. **Zero-Cost Abstractions**: No runtime overhead from the API design

### 1.2 Design Decisions

#### ADR-001: Query Key Design
**Decision**: Use `Vec<String>` with trait implementations for flexible key construction
```rust
pub struct QueryKey {
    pub segments: Vec<String>,
}

// Flexible construction patterns
impl From<&[&str]> for QueryKey { ... }
impl From<(&str, u32)> for QueryKey { ... }
impl From<String> for QueryKey { ... }
```

**Rationale**:
- Dynamic key construction for computed keys
- Pattern matching for cache invalidation
- Serializable for debugging and persistence
- Flexible ergonomics with multiple From implementations

#### ADR-002: Hook Return Types
**Decision**: Return comprehensive state objects with computed signals
```rust
pub struct QueryResult<T> {
    pub data: Signal<Option<T>>,
    pub error: Signal<Option<QueryError>>,
    pub is_loading: Signal<bool>,
    pub is_fetching: Signal<bool>,
    pub is_success: Signal<bool>,
    pub is_error: Signal<bool>,
    pub refetch: Callback<()>,
    // ... more fields
}
```

**Rationale**:
- Predictable API surface matching React Query
- Reactive updates through Leptos signals
- Comprehensive state coverage for all use cases

## 2. Core API Surface

### 2.1 Query Client

```rust
/// Core client managing all queries and cache
#[derive(Clone)]
pub struct QueryClient {
    // Internal fields hidden
}

impl QueryClient {
    /// Create a new query client with configuration
    pub fn new(config: QueryClientConfig) -> Self;
    
    /// Get cached data
    pub fn get_query_data<T>(&self, key: &QueryKey) -> Option<T>;
    
    /// Set data in cache
    pub fn set_query_data<T>(&self, key: &QueryKey, data: T) -> Result<(), QueryError>;
    
    /// Invalidate queries matching pattern
    pub fn invalidate_queries(&self, pattern: QueryKeyPattern);
    
    /// Remove queries from cache
    pub fn remove_queries(&self, pattern: QueryKeyPattern);
    
    /// Prefetch data for later use
    pub async fn prefetch_query<T, F, Fut>(&self, key: QueryKey, fetcher: F) -> Result<T, QueryError>;
    
    /// Cancel in-flight queries
    pub async fn cancel_queries(&self, pattern: QueryKeyPattern);
}
```

### 2.2 Query Hooks

#### Primary Query Hook
```rust
/// Main query hook for data fetching
pub fn use_query<T, K, F, Fut>(
    key_fn: impl Fn() -> K + 'static,
    query_fn: impl Fn() -> F + Clone + 'static,
    options: QueryOptions,
) -> QueryResult<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
    K: Into<QueryKey>,
    F: FnOnce() -> Fut + Clone + 'static,
    Fut: Future<Output = Result<T, QueryError>> + 'static;

// Usage examples:
let user = use_query(
    move || ["users", user_id.get()],
    move || fetch_user(user_id.get()),
    QueryOptions::default(),
);

// With options:
let posts = use_query(
    move || ["posts", page.get()],
    move || fetch_posts(page.get()),
    QueryOptions {
        stale_time: Duration::from_secs(300),
        refetch_interval: Some(Duration::from_secs(60)),
        ..Default::default()
    },
);
```

#### Simplified Query Hook
```rust
/// Simplified query hook with defaults
pub fn use_simple_query<T, F, Fut>(
    key: impl Into<QueryKey>,
    query_fn: F,
) -> QueryResult<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
    F: Fn() -> Fut + Clone + 'static,
    Fut: Future<Output = Result<T, QueryError>> + 'static;

// Usage:
let todos = use_simple_query(
    ["todos"],
    || fetch_todos(),
);
```

#### Reactive Query Hook
```rust
/// Query hook that depends on reactive state
pub fn use_reactive_query<T, K, F, Fut>(
    key_fn: impl Fn() -> K + 'static,
    query_fn: impl Fn(K) -> F + Clone + 'static,
    options: QueryOptions,
) -> QueryResult<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
    K: Into<QueryKey> + Clone + 'static,
    F: FnOnce() -> Fut + Clone + 'static,
    Fut: Future<Output = Result<T, QueryError>> + 'static;

// Usage:
let search_results = use_reactive_query(
    move || ("search", search_term.get()),
    move |(_prefix, term)| search_api(term),
    QueryOptions::default(),
);
```

### 2.3 Mutation Hooks

#### Primary Mutation Hook
```rust
/// Main mutation hook for data modifications
pub fn use_mutation<TData, TError, TVariables, TContext, F, Fut>(
    mutation_fn: F,
    options: MutationOptions<TData, TError, TVariables, TContext>,
) -> MutationResult<TData, TError, TVariables>
where
    TData: Clone + 'static,
    TError: Clone + 'static,
    TVariables: Clone + 'static,
    TContext: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, TError>> + 'static;

// Usage:
let create_user = use_mutation(
    |user_data: CreateUserDto| async move {
        api::create_user(user_data).await
    },
    MutationOptions {
        invalidates: vec![
            QueryKeyPattern::Prefix(QueryKey::new(["users"])),
            QueryKeyPattern::Exact(QueryKey::new(["user_count"])),
        ],
        on_success: Some(Box::new(|data, _vars, _ctx| {
            show_success_toast(&format!("Created user: {}", data.name));
        })),
        ..Default::default()
    },
);

// Trigger mutation:
create_user.mutate.call(CreateUserDto { 
    name: "John".into(),
    email: "john@example.com".into(),
});
```

#### Optimistic Update Hook
```rust
/// Mutation with built-in optimistic updates
pub fn use_optimistic_mutation<TData, TVariables, F, Fut>(
    query_key: QueryKey,
    mutation_fn: F,
    optimistic_update: impl Fn(&TVariables) -> TData + 'static,
) -> MutationResult<TData, QueryError, TVariables>
where
    TData: Serialize + DeserializeOwned + Clone + 'static,
    TVariables: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, QueryError>> + 'static;

// Usage:
let toggle_todo = use_optimistic_mutation(
    QueryKey::new(["todos"]),
    |todo_id: u32| async move {
        api::toggle_todo(todo_id).await
    },
    |todo_id| {
        // Optimistic update function
        let mut todos = get_current_todos();
        if let Some(todo) = todos.iter_mut().find(|t| t.id == *todo_id) {
            todo.completed = !todo.completed;
        }
        todos
    },
);
```

#### Simplified Mutations
```rust
/// Simple mutation with automatic invalidation
pub fn use_simple_mutation<TData, TVariables, F, Fut>(
    mutation_fn: F,
    invalidates: Vec<QueryKey>,
) -> MutationResult<TData, QueryError, TVariables>;

/// Bulk mutation affecting multiple query patterns
pub fn use_bulk_mutation<TData, TVariables, F, Fut>(
    mutation_fn: F,
    invalidate_patterns: Vec<QueryKeyPattern>,
) -> MutationResult<TData, QueryError, TVariables>;
```

### 2.4 Infinite Query Hook

```rust
/// Hook for paginated data with infinite loading
pub fn use_infinite_query<T, K, F, Fut>(
    key_fn: impl Fn() -> K + 'static,
    query_fn: impl Fn(Option<String>) -> F + Clone + 'static,
    options: InfiniteQueryOptions,
) -> InfiniteQueryResult<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
    K: Into<QueryKey>,
    F: FnOnce(Option<String>) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<Page<T>, QueryError>> + 'static;

// Usage:
let posts = use_infinite_query(
    || ["posts"],
    |cursor| async move {
        fetch_posts_page(cursor).await
    },
    InfiniteQueryOptions {
        get_next_page_param: Box::new(|last_page| last_page.next_cursor),
        ..Default::default()
    },
);

// Load more data:
posts.fetch_next_page.call(());
```

## 3. Configuration API

### 3.1 Query Options

```rust
#[derive(Clone)]
pub struct QueryOptions {
    /// Whether the query should run
    pub enabled: Signal<bool>,
    /// Time before data becomes stale
    pub stale_time: Duration,
    /// Time before data is removed from cache
    pub cache_time: Duration,
    /// Interval for background refetching
    pub refetch_interval: Option<Duration>,
    /// Refetch when window gains focus
    pub refetch_on_window_focus: bool,
    /// Refetch when network reconnects
    pub refetch_on_reconnect: bool,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Keep previous data during refetch
    pub keep_previous_data: bool,
    /// Use suspense for loading states
    pub suspense: bool,
    /// Request timeout
    pub timeout: Option<Duration>,
    /// Initial data
    pub initial_data: Option<Box<dyn Fn() -> Option<SerializedData>>>,
    /// Placeholder data while loading
    pub placeholder_data: Option<Box<dyn Fn() -> Option<SerializedData>>>,
    /// Callbacks
    pub on_success: Option<Callback<SerializedData>>,
    pub on_error: Option<Callback<QueryError>>,
    pub on_settled: Option<Callback<()>>,
}

impl QueryOptions {
    /// Builder pattern methods
    pub fn with_stale_time(mut self, duration: Duration) -> Self;
    pub fn with_cache_time(mut self, duration: Duration) -> Self;
    pub fn with_refetch_interval(mut self, interval: Duration) -> Self;
    pub fn with_retry(mut self, retry: RetryConfig) -> Self;
    pub fn disabled(mut self) -> Self;
    pub fn keep_previous_data(mut self) -> Self;
    pub fn with_suspense(mut self) -> Self;
}
```

### 3.2 Client Configuration

```rust
#[derive(Clone, Debug)]
pub struct QueryClientConfig {
    /// Default time before queries become stale
    pub default_stale_time: Duration,
    /// Default time before queries are garbage collected
    pub default_cache_time: Duration,
    /// How often to run garbage collection
    pub gc_interval: Duration,
    /// Maximum number of queries to keep in cache
    pub max_cache_size: Option<usize>,
    /// Global retry configuration
    pub default_retry: RetryConfig,
}

impl Default for QueryClientConfig {
    fn default() -> Self {
        Self {
            default_stale_time: Duration::from_secs(0),
            default_cache_time: Duration::from_secs(5 * 60),
            gc_interval: Duration::from_secs(60),
            max_cache_size: Some(1000),
            default_retry: RetryConfig::default(),
        }
    }
}
```

## 4. Error Handling API

### 4.1 Error Types

```rust
#[derive(Clone, Debug, Error)]
pub enum QueryError {
    #[error("Network error: {message}")]
    Network { message: String, source: Option<String> },
    
    #[error("Request timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String, body: Option<String> },
    
    #[error("Serialization failed: {message}")]
    Serialization(String),
    
    #[error("Deserialization failed: {message}")]
    Deserialization(String),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Request was cancelled")]
    Cancelled,
    
    #[error("Rate limit exceeded, retry after {retry_after_ms}ms")]
    RateLimit { retry_after_ms: u64 },
    
    #[error("Custom error: {message}")]
    Custom { message: String, code: Option<String> },
}

impl QueryError {
    // Constructor methods
    pub fn network(message: impl Into<String>) -> Self;
    pub fn http(status: u16, message: impl Into<String>) -> Self;
    pub fn timeout(timeout_ms: u64) -> Self;
    pub fn custom(message: impl Into<String>) -> Self;
    
    // Utility methods
    pub fn is_retryable(&self) -> bool;
    pub fn suggested_retry_delay(&self) -> Option<Duration>;
    pub fn severity(&self) -> ErrorSeverity;
}
```

### 4.2 Retry Configuration

```rust
#[derive(Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay: RetryDelay,
    pub retryable_errors: Box<dyn Fn(&QueryError) -> bool>,
    pub jitter: bool,
}

#[derive(Clone)]
pub enum RetryDelay {
    Fixed(Duration),
    Linear { initial: Duration, increment: Duration },
    Exponential { initial: Duration, multiplier: f64, max: Duration },
    Custom(Box<dyn Fn(u32) -> Duration>),
}
```

## 5. Cache Management API

### 5.1 Cache Invalidation

```rust
pub enum QueryKeyPattern {
    /// Exact match
    Exact(QueryKey),
    /// Prefix match (e.g., ["users"] matches ["users", "1"])
    Prefix(QueryKey),
    /// Contains segment (e.g., "drafts" matches ["posts", "drafts", "1"])
    Contains(String),
    /// Custom predicate
    Predicate(Box<dyn Fn(&QueryKey) -> bool>),
}

// Usage examples:
client.invalidate_queries(QueryKeyPattern::Exact(
    QueryKey::new(["user", "123"])
));

client.invalidate_queries(QueryKeyPattern::Prefix(
    QueryKey::new(["users"])
));

client.invalidate_queries(QueryKeyPattern::Contains(
    "drafts".into()
));

client.invalidate_queries(QueryKeyPattern::Predicate(
    Box::new(|key| key.segments.contains(&"temp".to_string()))
));
```

## 6. Persistence API

### 6.1 Persistence Adapter Trait

```rust
pub trait PersistenceAdapter: Send + Sync {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&self, key: &str, value: Vec<u8>);
    fn remove(&self, key: &str);
    fn clear(&self);
}

// Built-in adapters:
pub struct LocalStorageAdapter;
pub struct IndexedDBAdapter;
pub struct MemoryAdapter;
```

### 6.2 Query Persister

```rust
pub struct QueryPersister {
    adapter: Box<dyn PersistenceAdapter>,
    // Internal fields
}

impl QueryPersister {
    pub fn new(adapter: Box<dyn PersistenceAdapter>) -> Self;
    pub fn persist_query(&self, key: &QueryKey, entry: &CacheEntry) -> Result<(), QueryError>;
    pub fn restore_query(&self, key: &QueryKey) -> Option<CacheEntry>;
    pub fn remove_query(&self, key: &QueryKey);
}
```

## 7. DevTools API

```rust
/// DevTools component for development
#[component]
pub fn QueryDevTools() -> impl IntoView;

/// Query inspector utilities
pub struct QueryInspector {
    client: QueryClient,
}

impl QueryInspector {
    pub fn get_all_queries(&self) -> Vec<QueryInfo>;
    pub fn get_query_info(&self, key: &QueryKey) -> Option<QueryInfo>;
    pub fn export_cache_state(&self) -> String;
    pub fn import_cache_state(&self, state: &str) -> Result<(), QueryError>;
}

#[derive(Clone, Debug)]
pub struct QueryInfo {
    pub key: QueryKey,
    pub state: QueryState,
    pub data_size: Option<usize>,
    pub updated_at: Instant,
    pub is_stale: bool,
    pub observer_count: usize,
}
```

## 8. Migration & Compatibility

### 8.1 Migration Utilities

```rust
/// Migration helpers for version upgrades
pub mod migration {
    /// Migrate from v0.1 to v0.2
    pub fn migrate_v0_1_to_v0_2(old_cache: &str) -> Result<String, MigrationError>;
    
    /// Validate cache format version
    pub fn validate_cache_version(cache: &str) -> Result<Version, MigrationError>;
}
```

## 9. Performance API

### 9.1 Metrics Collection

```rust
pub struct QueryMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub active_queries: u32,
    pub total_fetch_time: Duration,
    pub average_fetch_time: Duration,
}

impl QueryClient {
    /// Get current metrics
    pub fn metrics(&self) -> QueryMetrics;
    
    /// Reset metrics
    pub fn reset_metrics(&self);
    
    /// Export metrics in Prometheus format
    pub fn export_prometheus_metrics(&self) -> String;
}
```

## 10. Breaking Change Policy

### 10.1 API Stability Guarantees

1. **Patch versions** (0.1.0 -> 0.1.1): No breaking changes, only bug fixes
2. **Minor versions** (0.1.0 -> 0.2.0): Additive changes only, deprecated features marked
3. **Major versions** (0.x.0 -> 1.0.0): Breaking changes allowed with migration guide

### 10.2 Deprecation Process

1. Mark API as deprecated in code with `#[deprecated]`
2. Add deprecation notice in documentation
3. Provide migration path in deprecation message
4. Keep deprecated API for at least one minor version
5. Remove in next major version

## 11. Future API Considerations

### 11.1 Planned Additions

```rust
// Subscription support (v0.2)
pub fn use_subscription<T>(/* ... */) -> SubscriptionResult<T>;

// Query dependencies (v0.2)  
pub fn use_dependent_query<T>(/* ... */) -> QueryResult<T>;

// Parallel queries (v0.3)
pub fn use_queries<T>(/* ... */) -> Vec<QueryResult<T>>;

// Background sync (v0.3)
pub fn use_background_sync<T>(/* ... */) -> SyncResult<T>;
```

### 11.2 Plugin System (Future)

```rust
pub trait QueryPlugin {
    fn name(&self) -> &str;
    fn on_query_start(&self, key: &QueryKey);
    fn on_query_success(&self, key: &QueryKey, data: &SerializedData);
    fn on_query_error(&self, key: &QueryKey, error: &QueryError);
    fn on_cache_update(&self, key: &QueryKey, entry: &CacheEntry);
}

impl QueryClient {
    pub fn add_plugin(&mut self, plugin: Box<dyn QueryPlugin>);
    pub fn remove_plugin(&mut self, name: &str);
}
```

This API design provides a comprehensive, type-safe, and ergonomic interface for data fetching in Leptos applications while maintaining consistency with established patterns and allowing for future extensibility.