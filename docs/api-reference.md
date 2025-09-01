# API Reference

This document provides a complete reference for all public APIs in the `leptos-query` library.

## Table of Contents

- [Core Types](#core-types)
- [QueryClient](#queryclient)
- [use_query](#use_query)
- [use_mutation](#use_mutation)
- [use_optimistic_mutation](#use_optimistic_mutation)
- [QueryOptions](#queryoptions)
- [MutationOptions](#mutationoptions)
- [QueryResult](#queryresult)
- [MutationResult](#mutationresult)
- [QueryKey](#querykey)
- [QueryKeyPattern](#querykeypattern)
- [QueryError](#queryerror)
- [RetryConfig](#retryconfig)
- [RetryDelay](#retrydelay)

## Core Types

### QueryClient

The main client that manages all queries and mutations.

```rust
pub struct QueryClient {
    // Internal fields
}

impl QueryClient {
    pub fn new(config: QueryClientConfig) -> Self;
    pub fn set_query_data<T>(&self, key: &QueryKey, data: T) -> Result<(), QueryError>;
    pub fn get_query_data<T>(&self, key: &QueryKey) -> Option<T>;
    pub fn invalidate_queries(&self, pattern: &QueryKeyPattern);
    pub fn remove_queries(&self, pattern: &QueryKeyPattern);
    pub fn register_query_observer(&self, key: &QueryKey, observer_id: QueryObserverId);
    pub fn unregister_query_observer(&self, key: &QueryKey, observer_id: &QueryObserverId);
}
```

### QueryClientConfig

Configuration for the QueryClient.

```rust
pub struct QueryClientConfig {
    pub default_stale_time: Duration,
    pub default_cache_time: Duration,
    pub gc_interval: Duration,
    pub max_cache_size: Option<usize>,
    pub default_retry: RetryConfig,
}

impl Default for QueryClientConfig {
    fn default() -> Self {
        Self {
            default_stale_time: Duration::from_secs(0),
            default_cache_time: Duration::from_secs(300),
            gc_interval: Duration::from_secs(60),
            max_cache_size: None,
            default_retry: RetryConfig::default(),
        }
    }
}

impl QueryClientConfig {
    pub fn with_default_stale_time(mut self, stale_time: Duration) -> Self;
    pub fn with_default_cache_time(mut self, cache_time: Duration) -> Self;
    pub fn with_garbage_collection_interval(mut self, interval: Duration) -> Self;
    pub fn with_max_cache_size(mut self, size: usize) -> Self;
    pub fn with_default_retry(mut self, retry: RetryConfig) -> Self;
}
```

## Hooks

### use_query

The main hook for data fetching with automatic caching and background updates.

```rust
pub fn use_query<T, K, F, Fut>(
    key_fn: impl Fn() -> K + 'static,
    query_fn: impl Fn() -> F + Clone + 'static,
    options: QueryOptions,
) -> QueryResult<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
    K: Into<QueryKey>,
    F: FnOnce() -> Fut + Clone + 'static,
    Fut: Future<Output = Result<T, QueryError>> + 'static,
```

**Parameters:**
- `key_fn`: Function that returns the query key
- `query_fn`: Function that returns the actual query function
- `options`: Configuration options for the query

**Returns:** `QueryResult<T>` containing the query state

**Example:**
```rust
let user_query = use_query(
    || &["users", &user_id.to_string()][..],
    || || async move { fetch_user(user_id).await },
    QueryOptions::default()
        .with_stale_time(Duration::from_secs(60))
);
```

### use_mutation

Hook for data mutations with automatic cache invalidation.

```rust
pub fn use_mutation<TData, TVariables, TContext, F, Fut>(
    mutation_fn: F,
    options: MutationOptions<TData, TVariables, TContext>,
) -> MutationResult<TData, TVariables>
where
    TData: Serialize + DeserializeOwned + Clone + 'static,
    TVariables: Clone + 'static,
    TContext: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, QueryError>> + 'static,
```

**Parameters:**
- `mutation_fn`: Function that performs the mutation
- `options`: Configuration options for the mutation

**Returns:** `MutationResult<TData, TVariables>` containing the mutation state

**Example:**
```rust
let create_user_mutation = use_mutation::<User, CreateUserRequest, (), _, _>(
    |request| async move { create_user(request).await },
    MutationOptions::default()
        .with_invalidates(&[&["users"][..]])
);
```

### use_optimistic_mutation

Hook for mutations with optimistic updates.

```rust
pub fn use_optimistic_mutation<TData, TVariables, F, Fut>(
    query_key: QueryKey,
    mutation_fn: F,
    optimistic_update: impl Fn(&TVariables) -> TData + Send + Sync + 'static,
) -> MutationResult<TData, TVariables>
where
    TData: Serialize + DeserializeOwned + Clone + 'static,
    TVariables: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, QueryError>> + 'static,
```

**Parameters:**
- `query_key`: Key of the query to update optimistically
- `mutation_fn`: Function that performs the mutation
- `optimistic_update`: Function that creates optimistic data

**Returns:** `MutationResult<TData, TVariables>` containing the mutation state

**Example:**
```rust
let optimistic_mutation = use_optimistic_mutation(
    QueryKey::new(&["users", "1"]),
    |request| async move { update_user(request).await },
    |request| User {
        id: 1,
        name: request.name.clone(),
        email: request.email.clone(),
    }
);
```

## Options

### QueryOptions

Configuration options for queries.

```rust
pub struct QueryOptions {
    pub enabled: Signal<bool>,
    pub stale_time: Duration,
    pub cache_time: Duration,
    pub refetch_interval: Option<Duration>,
    pub refetch_on_window_focus: bool,
    pub refetch_on_reconnect: bool,
    pub retry: RetryConfig,
    pub keep_previous_data: bool,
    pub suspense: bool,
    pub on_success: Option<Box<dyn Fn(&SerializedData) + Send + Sync>>,
    pub on_error: Option<Box<dyn Fn(&QueryError) + Send + Sync>>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            enabled: Signal::derive(|| true),
            stale_time: Duration::from_secs(0),
            cache_time: Duration::from_secs(300),
            refetch_interval: None,
            refetch_on_window_focus: false,
            refetch_on_reconnect: false,
            retry: RetryConfig::default(),
            keep_previous_data: false,
            suspense: false,
            on_success: None,
            on_error: None,
        }
    }
}

impl QueryOptions {
    pub fn with_stale_time(mut self, stale_time: Duration) -> Self;
    pub fn with_cache_time(mut self, cache_time: Duration) -> Self;
    pub fn with_refetch_interval(mut self, interval: Duration) -> Self;
    pub fn with_refetch_on_window_focus(mut self) -> Self;
    pub fn with_refetch_on_reconnect(mut self) -> Self;
    pub fn with_retry(mut self, retry: RetryConfig) -> Self;
    pub fn keep_previous_data(mut self) -> Self;
    pub fn with_suspense(mut self) -> Self;
    pub fn with_enabled(mut self, enabled: Signal<bool>) -> Self;
    pub fn with_on_success(mut self, callback: Box<dyn Fn(&SerializedData) + Send + Sync>) -> Self;
    pub fn with_on_error(mut self, callback: Box<dyn Fn(&QueryError) + Send + Sync>) -> Self;
}
```

### MutationOptions

Configuration options for mutations.

```rust
pub struct MutationOptions<TData, TVariables, TContext> {
    pub invalidates: Vec<QueryKeyPattern>,
    pub throw_on_error: bool,
    pub retry: RetryConfig,
    pub on_mutate: Option<Box<dyn Fn(&TVariables) -> Option<TContext> + Send + Sync>>,
    pub on_success: Option<Box<dyn Fn(&TData, &TVariables, &Option<TContext>) + Send + Sync>>,
    pub on_error: Option<Box<dyn Fn(&QueryError, &TVariables, &Option<TContext>) + Send + Sync>>,
    pub on_settled: Option<Box<dyn Fn(&Option<TData>, &Option<QueryError>, &TVariables, &Option<TContext>) + Send + Sync>>,
}

impl<TData, TVariables, TContext> Default for MutationOptions<TData, TVariables, TContext> {
    fn default() -> Self {
        Self {
            invalidates: Vec::new(),
            throw_on_error: false,
            retry: RetryConfig::default(),
            on_mutate: None,
            on_success: None,
            on_error: None,
            on_settled: None,
        }
    }
}

impl<TData, TVariables, TContext> MutationOptions<TData, TVariables, TContext> {
    pub fn with_invalidates(mut self, patterns: &[&[&str]]) -> Self;
    pub fn with_throw_on_error(mut self) -> Self;
    pub fn with_retry(mut self, retry: RetryConfig) -> Self;
    pub fn with_on_mutate(mut self, callback: Box<dyn Fn(&TVariables) -> Option<TContext> + Send + Sync>) -> Self;
    pub fn with_on_success(mut self, callback: Box<dyn Fn(&TData, &TVariables, &Option<TContext>) + Send + Sync>) -> Self;
    pub fn with_on_error(mut self, callback: Box<dyn Fn(&QueryError, &TVariables, &Option<TContext>) + Send + Sync>) -> Self;
    pub fn with_on_settled(mut self, callback: Box<dyn Fn(&Option<TData>, &Option<QueryError>, &TVariables, &Option<TContext>) + Send + Sync>) -> Self;
}
```

## Results

### QueryResult

The result of a query hook, containing all the reactive state.

```rust
pub struct QueryResult<T> {
    pub data: Signal<Option<T>>,
    pub error: Signal<Option<QueryError>>,
    pub is_loading: Signal<bool>,
    pub is_fetching: Signal<bool>,
    pub is_idle: Signal<bool>,
    pub is_success: Signal<bool>,
    pub is_error: Signal<bool>,
    pub status: Signal<QueryStatus>,
    pub data_updated_at: Signal<Option<Instant>>,
    pub error_updated_at: Signal<Option<Instant>>,
    pub meta: Signal<QueryMeta>,
    pub refetch: Callback<()>,
    pub invalidate: Callback<()>,
    pub set_data: Callback<T>,
}
```

### MutationResult

The result of a mutation hook, containing all the reactive state.

```rust
pub struct MutationResult<TData, TVariables> {
    pub data: Signal<Option<TData>>,
    pub error: Signal<Option<QueryError>>,
    pub is_idle: Signal<bool>,
    pub is_loading: Signal<bool>,
    pub is_success: Signal<bool>,
    pub is_error: Signal<bool>,
    pub status: Signal<MutationStatus>,
    pub submitted_at: Signal<Option<Instant>>,
    pub variables: Signal<Option<TVariables>>,
    pub mutate: Callback<TVariables>,
    pub mutate_async: Callback<TVariables>,
    pub reset: Callback<()>,
}
```

## Keys and Patterns

### QueryKey

A key used to identify and cache queries.

```rust
pub struct QueryKey {
    pub segments: Vec<String>,
}

impl QueryKey {
    pub fn new(segments: impl IntoIterator<Item = impl ToString>) -> Self;
    pub fn from_parts<T: Serialize>(parts: &[T]) -> Result<Self, QueryError>;
    pub fn matches_pattern(&self, pattern: &QueryKeyPattern) -> bool;
}

impl<T: ToString + std::fmt::Display> From<&[T]> for QueryKey;
impl<T: ToString + std::fmt::Display> From<(T,)> for QueryKey;
```

### QueryKeyPattern

Patterns for matching and invalidating query keys.

```rust
pub enum QueryKeyPattern {
    Exact(QueryKey),
    Prefix(QueryKey),
    Contains(String),
}
```

## Error Types

### QueryError

Comprehensive error types for different failure scenarios.

```rust
#[derive(Clone, Debug, Error)]
pub enum QueryError {
    #[error("Network error: {message}")]
    Network { message: String },
    
    #[error("HTTP error {status}: {message}")]
    Http { status: u16, message: String },
    
    #[error("Request timed out after {duration:?}")]
    Timeout { duration: Duration },
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Custom error: {0}")]
    Custom(String),
}

impl QueryError {
    pub fn network(message: impl Into<String>) -> Self;
    pub fn http(status: u16, message: impl Into<String>) -> Self;
    pub fn timeout(duration: Duration) -> Self;
    pub fn serialization(message: impl Into<String>) -> Self;
    pub fn deserialization(message: impl Into<String>) -> Self;
    pub fn custom(message: impl Into<String>) -> Self;
    pub fn is_retryable(&self) -> bool;
}
```

## Retry Configuration

### RetryConfig

Configuration for retry logic.

```rust
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay: RetryDelay,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay: RetryDelay::Exponential {
                initial: Duration::from_millis(1000),
                multiplier: 2.0,
                max: Duration::from_secs(30),
            },
            jitter: false,
        }
    }
}
```

### RetryDelay

Different delay strategies for retries.

```rust
pub enum RetryDelay {
    Fixed(Duration),
    Exponential {
        initial: Duration,
        multiplier: f64,
        max: Duration,
    },
}
```

## Status Types

### QueryStatus

Status of a query.

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum QueryStatus {
    Idle,
    Loading,
    Success,
    Error,
}
```

### MutationStatus

Status of a mutation.

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum MutationStatus {
    Idle,
    Loading,
    Success,
    Error,
}
```

## Utility Types

### SerializedData

Serialized data for cache storage.

```rust
pub struct SerializedData {
    pub bytes: Vec<u8>,
    pub type_id: std::any::TypeId,
}

impl SerializedData {
    pub fn serialize<T: Serialize + 'static>(data: &T) -> Result<Self, QueryError>;
    pub fn deserialize<T: DeserializeOwned + 'static>(&self) -> Result<T, QueryError>;
}
```

### QueryMeta

Metadata about query execution.

```rust
pub struct QueryMeta {
    pub fetch_count: u32,
    pub error_count: u32,
    pub last_fetch_duration: Option<Duration>,
    pub last_error_duration: Option<Duration>,
}

impl Default for QueryMeta {
    fn default() -> Self {
        Self {
            fetch_count: 0,
            error_count: 0,
            last_fetch_duration: None,
            last_error_duration: None,
        }
    }
}

impl QueryMeta {
    pub fn record_fetch(&mut self, duration: Duration);
    pub fn record_error(&mut self);
}
```
