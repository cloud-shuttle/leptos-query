Leptos Query - Comprehensive Data Fetching/Caching Layer Design
Overview
A powerful, type-safe data fetching and caching library for Leptos that provides automatic background refetching, request deduplication, optimistic updates, and intelligent caching strategies - similar to TanStack Query but leveraging Rust's ownership model and type system.
Core Architecture
1. Query Client & Cache Design
rust// leptos-query/src/client/mod.rs

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use leptos::*;
use serde::{Serialize, Deserialize};

/// Global query client that manages all queries and mutations
#[derive(Clone)]
pub struct QueryClient {
    cache: Arc<RwLock<QueryCache>>,
    config: QueryClientConfig,
    query_observers: Arc<RwLock<HashMap<QueryKey, Vec<QueryObserverId>>>>,
    mutation_observers: Arc<RwLock<HashMap<MutationKey, Vec<MutationObserverId>>>>,
    request_deduper: Arc<RwLock<RequestDeduplicator>>,
}

/// Internal cache structure
pub struct QueryCache {
    queries: HashMap<QueryKey, CacheEntry>,
    mutations: HashMap<MutationKey, MutationState>,
    // Garbage collection tracking
    access_times: HashMap<QueryKey, Instant>,
    // Subscribe to cache updates
    listeners: HashMap<QueryKey, Vec<CacheListener>>,
}

/// Individual cache entry
#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub data: Option<SerializedData>,
    pub error: Option<QueryError>,
    pub state: QueryState,
    pub updated_at: Instant,
    pub data_updated_at: Instant,
    pub stale_time: Duration,
    pub cache_time: Duration,
    pub meta: QueryMeta,
}

/// Query state machine
#[derive(Clone, Debug, PartialEq)]
pub enum QueryState {
    Idle,
    Loading,
    Fetching,  // Background fetch while showing stale data
    Success,
    Error,
}

/// Serialized data that can be stored in cache
#[derive(Clone, Debug)]
pub struct SerializedData {
    pub bytes: Vec<u8>,
    pub type_id: std::any::TypeId,
}

impl SerializedData {
    pub fn deserialize<T: DeserializeOwned + 'static>(&self) -> Result<T, QueryError> {
        if self.type_id != std::any::TypeId::of::<T>() {
            return Err(QueryError::TypeMismatch);
        }
        bincode::deserialize(&self.bytes).map_err(QueryError::Deserialization)
    }
    
    pub fn serialize<T: Serialize + 'static>(data: &T) -> Result<Self, QueryError> {
        Ok(Self {
            bytes: bincode::serialize(data).map_err(QueryError::Serialization)?,
            type_id: std::any::TypeId::of::<T>(),
        })
    }
}

/// Query key for cache identification
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct QueryKey {
    pub segments: Vec<String>,
}

impl QueryKey {
    pub fn new(segments: impl IntoIterator<Item = impl ToString>) -> Self {
        Self {
            segments: segments.into_iter().map(|s| s.to_string()).collect(),
        }
    }
    
    /// Create a key with automatic serialization
    pub fn from_parts<T: Serialize>(parts: &[T]) -> Result<Self, QueryError> {
        let segments = parts
            .iter()
            .map(|part| serde_json::to_string(part).map_err(QueryError::Serialization))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { segments })
    }
    
    /// Pattern matching for cache invalidation
    pub fn matches_pattern(&self, pattern: &QueryKeyPattern) -> bool {
        match pattern {
            QueryKeyPattern::Exact(key) => self == key,
            QueryKeyPattern::Prefix(prefix) => {
                self.segments.starts_with(&prefix.segments)
            }
            QueryKeyPattern::Contains(segment) => {
                self.segments.contains(segment)
            }
            QueryKeyPattern::Predicate(pred) => pred(self),
        }
    }
}

/// Pattern for matching query keys during invalidation
pub enum QueryKeyPattern {
    Exact(QueryKey),
    Prefix(QueryKey),
    Contains(String),
    Predicate(Box<dyn Fn(&QueryKey) -> bool>),
}

impl QueryClient {
    pub fn new(config: QueryClientConfig) -> Self {
        let client = Self {
            cache: Arc::new(RwLock::new(QueryCache::new())),
            config,
            query_observers: Arc::new(RwLock::new(HashMap::new())),
            mutation_observers: Arc::new(RwLock::new(HashMap::new())),
            request_deduper: Arc::new(RwLock::new(RequestDeduplicator::new())),
        };
        
        // Start garbage collection
        client.start_garbage_collection();
        
        // Setup global event listeners
        client.setup_event_listeners();
        
        client
    }
    
    /// Get data from cache
    pub fn get_query_data<T: DeserializeOwned + 'static>(
        &self,
        key: &QueryKey,
    ) -> Option<T> {
        let cache = self.cache.read().unwrap();
        cache.queries.get(key)
            .and_then(|entry| entry.data.as_ref())
            .and_then(|data| data.deserialize().ok())
    }
    
    /// Set data in cache
    pub fn set_query_data<T: Serialize + 'static>(
        &self,
        key: &QueryKey,
        data: T,
    ) -> Result<(), QueryError> {
        let mut cache = self.cache.write().unwrap();
        let serialized = SerializedData::serialize(&data)?;
        
        if let Some(entry) = cache.queries.get_mut(key) {
            entry.data = Some(serialized);
            entry.data_updated_at = Instant::now();
            entry.state = QueryState::Success;
            
            // Notify listeners
            cache.notify_listeners(key);
        } else {
            // Create new entry
            cache.queries.insert(
                key.clone(),
                CacheEntry {
                    data: Some(serialized),
                    error: None,
                    state: QueryState::Success,
                    updated_at: Instant::now(),
                    data_updated_at: Instant::now(),
                    stale_time: self.config.default_stale_time,
                    cache_time: self.config.default_cache_time,
                    meta: QueryMeta::default(),
                },
            );
        }
        
        Ok(())
    }
    
    /// Invalidate queries matching a pattern
    pub fn invalidate_queries(&self, pattern: QueryKeyPattern) {
        let mut cache = self.cache.write().unwrap();
        let keys_to_invalidate: Vec<QueryKey> = cache
            .queries
            .keys()
            .filter(|key| key.matches_pattern(&pattern))
            .cloned()
            .collect();
        
        for key in keys_to_invalidate {
            if let Some(entry) = cache.queries.get_mut(&key) {
                // Mark as stale
                entry.updated_at = Instant::now() - entry.stale_time - Duration::from_secs(1);
                
                // Trigger refetch for active observers
                if let Ok(observers) = self.query_observers.read() {
                    if observers.contains_key(&key) {
                        // Schedule refetch
                        self.schedule_refetch(&key);
                    }
                }
            }
        }
    }
    
    /// Remove queries from cache
    pub fn remove_queries(&self, pattern: QueryKeyPattern) {
        let mut cache = self.cache.write().unwrap();
        cache.queries.retain(|key, _| !key.matches_pattern(&pattern));
    }
    
    /// Prefetch a query
    pub async fn prefetch_query<T, F, Fut>(
        &self,
        key: QueryKey,
        fetcher: F,
        options: Option<QueryOptions>,
    ) -> Result<T, QueryError>
    where
        T: Serialize + DeserializeOwned + Clone + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, QueryError>>,
    {
        // Check if data exists and is fresh
        if let Some(data) = self.get_query_data::<T>(&key) {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.queries.get(&key) {
                if !entry.is_stale() {
                    return Ok(data);
                }
            }
        }
        
        // Fetch new data
        let result = fetcher().await?;
        self.set_query_data(&key, &result)?;
        Ok(result)
    }
    
    fn start_garbage_collection(&self) {
        let cache = self.cache.clone();
        let interval = self.config.gc_interval;
        
        spawn_local(async move {
            loop {
                sleep(interval).await;
                
                let now = Instant::now();
                let mut cache = cache.write().unwrap();
                
                // Remove expired entries
                cache.queries.retain(|key, entry| {
                    let age = now.duration_since(entry.updated_at);
                    let has_observers = cache.listeners.contains_key(key);
                    
                    // Keep if has observers or not expired
                    has_observers || age < entry.cache_time
                });
            }
        });
    }
}
2. Query Hook Implementation
rust// leptos-query/src/query/mod.rs

use leptos::*;
use std::rc::Rc;

/// Options for configuring a query
#[derive(Clone)]
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
    pub optimistic_data: Option<Box<dyn Fn() -> Option<SerializedData>>>,
    pub on_success: Option<Callback<SerializedData>>,
    pub on_error: Option<Callback<QueryError>>,
    pub on_settled: Option<Callback<()>>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            enabled: Signal::derive(|| true),
            stale_time: Duration::from_secs(0),
            cache_time: Duration::from_secs(5 * 60), // 5 minutes
            refetch_interval: None,
            refetch_on_window_focus: true,
            refetch_on_reconnect: true,
            retry: RetryConfig::default(),
            keep_previous_data: false,
            suspense: false,
            optimistic_data: None,
            on_success: None,
            on_error: None,
            on_settled: None,
        }
    }
}

/// Result of a query
#[derive(Clone)]
pub struct QueryResult<T> {
    pub data: Signal<Option<T>>,
    pub error: Signal<Option<QueryError>>,
    pub is_loading: Signal<bool>,
    pub is_fetching: Signal<bool>,
    pub is_success: Signal<bool>,
    pub is_error: Signal<bool>,
    pub is_idle: Signal<bool>,
    pub is_stale: Signal<bool>,
    pub data_updated_at: Signal<Option<Instant>>,
    pub error_updated_at: Signal<Option<Instant>>,
    pub status: Signal<QueryStatus>,
    
    // Actions
    pub refetch: Callback<()>,
    pub invalidate: Callback<()>,
    pub remove: Callback<()>,
    pub set_data: Callback<T>,
}

/// Main query hook
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
{
    let client = use_context::<QueryClient>()
        .expect("QueryClient not provided. Wrap your app with QueryClientProvider");
    
    // Reactive key
    let key = create_memo(move |_| key_fn().into());
    
    // Local state
    let (data, set_data) = create_signal(None::<T>);
    let (error, set_error) = create_signal(None::<QueryError>);
    let (is_loading, set_loading) = create_signal(false);
    let (is_fetching, set_fetching) = create_signal(false);
    let (status, set_status) = create_signal(QueryStatus::Idle);
    
    // Observer ID for this query instance
    let observer_id = QueryObserverId::new();
    
    // Register observer
    create_effect({
        let client = client.clone();
        let key = key.clone();
        let observer_id = observer_id.clone();
        
        move |_| {
            let current_key = key.get();
            client.register_query_observer(&current_key, observer_id.clone());
            
            // Cleanup on key change or unmount
            on_cleanup({
                let client = client.clone();
                let key = current_key.clone();
                let observer_id = observer_id.clone();
                move || {
                    client.unregister_query_observer(&key, &observer_id);
                }
            });
        }
    });
    
    // Fetch function with deduplication
    let fetch = {
        let client = client.clone();
        let query_fn = query_fn.clone();
        let key = key.clone();
        let options = options.clone();
        
        Rc::new(move || {
            let client = client.clone();
            let query_fn = query_fn.clone();
            let key = key.get();
            let options = options.clone();
            
            spawn_local(async move {
                // Check if request is already in flight
                if let Some(result) = client.check_deduped_request(&key).await {
                    handle_query_result(result, &set_data, &set_error, &set_status);
                    return;
                }
                
                // Set loading state
                set_fetching.set(true);
                if data.get().is_none() {
                    set_loading.set(true);
                }
                
                // Execute with retry logic
                let result = execute_with_retry(
                    || query_fn(),
                    &options.retry,
                ).await;
                
                // Update cache and local state
                match result {
                    Ok(data_result) => {
                        client.set_query_data(&key, &data_result).ok();
                        set_data.set(Some(data_result));
                        set_error.set(None);
                        set_status.set(QueryStatus::Success);
                        
                        if let Some(callback) = &options.on_success {
                            callback.call(());
                        }
                    }
                    Err(err) => {
                        set_error.set(Some(err.clone()));
                        set_status.set(QueryStatus::Error);
                        
                        if let Some(callback) = &options.on_error {
                            callback.call(err);
                        }
                    }
                }
                
                set_loading.set(false);
                set_fetching.set(false);
                
                if let Some(callback) = &options.on_settled {
                    callback.call(());
                }
            });
        })
    };
    
    // Initial fetch and cache subscription
    create_effect({
        let client = client.clone();
        let key = key.clone();
        let fetch = fetch.clone();
        let options = options.clone();
        
        move |_| {
            if !options.enabled.get() {
                return;
            }
            
            let current_key = key.get();
            
            // Check cache first
            if let Some(cached_data) = client.get_query_data::<T>(&current_key) {
                let cache_entry = client.get_cache_entry(&current_key);
                
                if let Some(entry) = cache_entry {
                    set_data.set(Some(cached_data));
                    
                    // Check if stale
                    if entry.is_stale() {
                        fetch();
                    }
                } else {
                    set_data.set(Some(cached_data));
                }
            } else {
                // No cache, fetch immediately
                fetch();
            }
        }
    });
    
    // Setup refetch interval
    if let Some(interval) = options.refetch_interval {
        set_interval_with_handle(
            move || {
                if options.enabled.get() {
                    fetch();
                }
            },
            interval,
        );
    }
    
    // Window focus refetching
    if options.refetch_on_window_focus {
        let fetch = fetch.clone();
        window_event_listener("focus", move |_| {
            if options.enabled.get() {
                fetch();
            }
        });
    }
    
    // Network reconnect refetching
    if options.refetch_on_reconnect {
        let fetch = fetch.clone();
        window_event_listener("online", move |_| {
            if options.enabled.get() {
                fetch();
            }
        });
    }
    
    // Create result
    QueryResult {
        data: data.into(),
        error: error.into(),
        is_loading: is_loading.into(),
        is_fetching: is_fetching.into(),
        is_success: create_memo(move |_| status.get() == QueryStatus::Success).into(),
        is_error: create_memo(move |_| status.get() == QueryStatus::Error).into(),
        is_idle: create_memo(move |_| status.get() == QueryStatus::Idle).into(),
        is_stale: create_memo({
            let client = client.clone();
            let key = key.clone();
            move |_| {
                client.get_cache_entry(&key.get())
                    .map(|entry| entry.is_stale())
                    .unwrap_or(true)
            }
        }).into(),
        data_updated_at: Signal::derive(move || {
            client.get_cache_entry(&key.get())
                .map(|entry| entry.data_updated_at)
        }),
        error_updated_at: Signal::derive(move || {
            error.get().map(|_| Instant::now())
        }),
        status: status.into(),
        refetch: Callback::new(move |_| fetch()),
        invalidate: Callback::new({
            let client = client.clone();
            let key = key.clone();
            move |_| {
                client.invalidate_queries(QueryKeyPattern::Exact(key.get()));
            }
        }),
        remove: Callback::new({
            let client = client.clone();
            let key = key.clone();
            move |_| {
                client.remove_queries(QueryKeyPattern::Exact(key.get()));
            }
        }),
        set_data: Callback::new({
            let client = client.clone();
            let key = key.clone();
            move |new_data: T| {
                client.set_query_data(&key.get(), new_data).ok();
                set_data.set(Some(new_data));
            }
        }),
    }
}
3. Mutation Hook Implementation
rust// leptos-query/src/mutation/mod.rs

use leptos::*;

/// Options for mutations
#[derive(Clone)]
pub struct MutationOptions<TData, TError, TVariables, TContext> {
    pub on_mutate: Option<Box<dyn Fn(&TVariables) -> Option<TContext>>>,
    pub on_success: Option<Box<dyn Fn(&TData, &TVariables, &Option<TContext>)>>,
    pub on_error: Option<Box<dyn Fn(&TError, &TVariables, &Option<TContext>)>>,
    pub on_settled: Option<Box<dyn Fn(&Option<TData>, &Option<TError>, &TVariables, &Option<TContext>)>>,
    pub retry: RetryConfig,
}

/// Result of a mutation
#[derive(Clone)]
pub struct MutationResult<TData, TError, TVariables> {
    pub data: Signal<Option<TData>>,
    pub error: Signal<Option<TError>>,
    pub is_idle: Signal<bool>,
    pub is_loading: Signal<bool>,
    pub is_success: Signal<bool>,
    pub is_error: Signal<bool>,
    pub status: Signal<MutationStatus>,
    pub submitted_at: Signal<Option<Instant>>,
    
    // Actions
    pub mutate: Callback<TVariables>,
    pub mutate_async: Rc<dyn Fn(TVariables) -> Pin<Box<dyn Future<Output = Result<TData, TError>>>>>,
    pub reset: Callback<()>,
}

/// Mutation hook with optimistic updates
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
    Fut: Future<Output = Result<TData, TError>> + 'static,
{
    let client = use_context::<QueryClient>()
        .expect("QueryClient not provided");
    
    // Local state
    let (data, set_data) = create_signal(None::<TData>);
    let (error, set_error) = create_signal(None::<TError>);
    let (status, set_status) = create_signal(MutationStatus::Idle);
    let (is_loading, set_loading) = create_signal(false);
    let (submitted_at, set_submitted_at) = create_signal(None::<Instant>);
    
    // Mutation ID for tracking
    let mutation_id = MutationId::new();
    
    // Execute mutation
    let execute = {
        let mutation_fn = mutation_fn.clone();
        let options = options.clone();
        let client = client.clone();
        
        Rc::new(move |variables: TVariables| {
            let mutation_fn = mutation_fn.clone();
            let options = options.clone();
            let client = client.clone();
            
            spawn_local(async move {
                set_loading.set(true);
                set_status.set(MutationStatus::Loading);
                set_submitted_at.set(Some(Instant::now()));
                
                // Call onMutate for optimistic updates
                let context = options.on_mutate.as_ref().and_then(|f| f(&variables));
                
                // Execute mutation with retry
                let result = execute_with_retry(
                    || mutation_fn(variables.clone()),
                    &options.retry,
                ).await;
                
                match result {
                    Ok(result_data) => {
                        set_data.set(Some(result_data.clone()));
                        set_error.set(None);
                        set_status.set(MutationStatus::Success);
                        
                        // Call onSuccess
                        if let Some(on_success) = &options.on_success {
                            on_success(&result_data, &variables, &context);
                        }
                        
                        // Call onSettled
                        if let Some(on_settled) = &options.on_settled {
                            on_settled(&Some(result_data), &None, &variables, &context);
                        }
                    }
                    Err(err) => {
                        set_error.set(Some(err.clone()));
                        set_status.set(MutationStatus::Error);
                        
                        // Call onError
                        if let Some(on_error) = &options.on_error {
                            on_error(&err, &variables, &context);
                        }
                        
                        // Call onSettled
                        if let Some(on_settled) = &options.on_settled {
                            on_settled(&None, &Some(err), &variables, &context);
                        }
                    }
                }
                
                set_loading.set(false);
            });
        })
    };
    
    MutationResult {
        data: data.into(),
        error: error.into(),
        is_idle: create_memo(move |_| status.get() == MutationStatus::Idle).into(),
        is_loading: is_loading.into(),
        is_success: create_memo(move |_| status.get() == MutationStatus::Success).into(),
        is_error: create_memo(move |_| status.get() == MutationStatus::Error).into(),
        status: status.into(),
        submitted_at: submitted_at.into(),
        mutate: Callback::new({
            let execute = execute.clone();
            move |variables| {
                execute(variables);
            }
        }),
        mutate_async: Rc::new({
            let mutation_fn = mutation_fn.clone();
            move |variables| {
                Box::pin(mutation_fn(variables))
            }
        }),
        reset: Callback::new(move |_| {
            set_data.set(None);
            set_error.set(None);
            set_status.set(MutationStatus::Idle);
            set_submitted_at.set(None);
        }),
    }
}

/// Optimistic update helper
pub fn use_optimistic_mutation<TData, TVariables, F, Fut>(
    query_key: QueryKey,
    mutation_fn: F,
    optimistic_update: impl Fn(&TVariables) -> TData + 'static,
) -> MutationResult<TData, QueryError, TVariables>
where
    TData: Serialize + DeserializeOwned + Clone + 'static,
    TVariables: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, QueryError>> + 'static,
{
    let client = use_context::<QueryClient>().unwrap();
    
    use_mutation(
        mutation_fn,
        MutationOptions {
            on_mutate: Some(Box::new({
                let client = client.clone();
                let query_key = query_key.clone();
                let optimistic_update = optimistic_update.clone();
                
                move |variables: &TVariables| {
                    // Cancel outgoing refetches
                    client.cancel_queries(&query_key);
                    
                    // Snapshot previous value
                    let previous_data = client.get_query_data::<TData>(&query_key);
                    
                    // Optimistically update
                    let optimistic_data = optimistic_update(variables);
                    client.set_query_data(&query_key, optimistic_data).ok();
                    
                    // Return context with previous data
                    Some(MutationContext {
                        previous_data,
                        query_key: query_key.clone(),
                    })
                }
            })),
            on_error: Some(Box::new({
                let client = client.clone();
                
                move |_error, _variables, context: &Option<MutationContext<TData>>| {
                    // Rollback on error
                    if let Some(ctx) = context {
                        if let Some(previous) = &ctx.previous_data {
                            client.set_query_data(&ctx.query_key, previous.clone()).ok();
                        }
                    }
                }
            })),
            on_settled: Some(Box::new({
                let client = client.clone();
                let query_key = query_key.clone();
                
                move |_data, _error, _variables, _context| {
                    // Always refetch after mutation
                    client.invalidate_queries(QueryKeyPattern::Exact(query_key.clone()));
                }
            })),
            retry: RetryConfig::default(),
        },
    )
}
4. Infinite Query Implementation
rust// leptos-query/src/infinite_query/mod.rs

use leptos::*;

/// Page of data for infinite queries
#[derive(Clone, Debug)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_next_page: bool,
}

/// Options for infinite queries
#[derive(Clone)]
pub struct InfiniteQueryOptions {
    pub get_next_page_param: Box<dyn Fn(&Page<SerializedData>) -> Option<String>>,
    pub get_previous_page_param: Option<Box<dyn Fn(&Page<SerializedData>) -> Option<String>>>,
    pub max_pages: Option<usize>,
    // Inherits from QueryOptions
    pub query_options: QueryOptions,
}

/// Result of an infinite query
#[derive(Clone)]
pub struct InfiniteQueryResult<T> {
    pub pages: Signal<Vec<Page<T>>>,
    pub page_params: Signal<Vec<Option<String>>>,
    pub is_loading: Signal<bool>,
    pub is_fetching: Signal<bool>,
    pub is_fetching_next_page: Signal<bool>,
    pub is_fetching_previous_page: Signal<bool>,
    pub has_next_page: Signal<bool>,
    pub has_previous_page: Signal<bool>,
    pub is_success: Signal<bool>,
    pub is_error: Signal<bool>,
    pub error: Signal<Option<QueryError>>,
    
    // Actions
    pub fetch_next_page: Callback<()>,
    pub fetch_previous_page: Callback<()>,
    pub refetch: Callback<()>,
    pub remove: Callback<()>,
}

/// Infinite query hook
pub fn use_infinite_query<T, K, F, Fut>(
    key_fn: impl Fn() -> K + 'static,
    query_fn: impl Fn(Option<String>) -> F + Clone + 'static,
    options: InfiniteQueryOptions,
) -> InfiniteQueryResult<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
    K: Into<QueryKey>,
    F: FnOnce(Option<String>) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<Page<T>, QueryError>> + 'static,
{
    let client = use_context::<QueryClient>().unwrap();
    
    // State
    let (pages, set_pages) = create_signal(Vec::<Page<T>>::new());
    let (page_params, set_page_params) = create_signal(Vec::<Option<String>>::new());
    let (is_loading, set_loading) = create_signal(false);
    let (is_fetching, set_fetching) = create_signal(false);
    let (is_fetching_next, set_fetching_next) = create_signal(false);
    let (is_fetching_prev, set_fetching_prev) = create_signal(false);
    let (error, set_error) = create_signal(None::<QueryError>);
    
    // Computed has_next_page
    let has_next_page = create_memo(move |_| {
        pages.get().last().map(|p| p.has_next_page).unwrap_or(true)
    });
    
    let has_previous_page = create_memo(move |_| {
        // Check if we can fetch previous pages
        options.get_previous_page_param.is_some() && !pages.get().is_empty()
    });
    
    // Fetch a specific page
    let fetch_page = {
        let query_fn = query_fn.clone();
        let options = options.clone();
        
        Rc::new(move |page_param: Option<String>, direction: FetchDirection| {
            let query_fn = query_fn.clone();
            
            spawn_local(async move {
                match direction {
                    FetchDirection::Forward => set_fetching_next.set(true),
                    FetchDirection::Backward => set_fetching_prev.set(true),
                }
                
                let result = query_fn(page_param.clone()).await;
                
                match result {
                    Ok(page) => {
                        match direction {
                            FetchDirection::Forward => {
                                set_pages.update(|p| p.push(page.clone()));
                                set_page_params.update(|p| p.push(page_param));
                            }
                            FetchDirection::Backward => {
                                set_pages.update(|p| p.insert(0, page.clone()));
                                set_page_params.update(|p| p.insert(0, page_param));
                            }
                        }
                        set_error.set(None);
                    }
                    Err(err) => {
                        set_error.set(Some(err));
                    }
                }
                
                set_fetching_next.set(false);
                set_fetching_prev.set(false);
                set_fetching.set(false);
                set_loading.set(false);
            });
        })
    };
    
    // Initial fetch
    create_effect({
        let fetch_page = fetch_page.clone();
        
        move |_| {
            if pages.get().is_empty() && options.query_options.enabled.get() {
                set_loading.set(true);
                fetch_page(None, FetchDirection::Forward);
            }
        }
    });
    
    // Fetch next page
    let fetch_next_page = {
        let fetch_page = fetch_page.clone();
        let options = options.clone();
        
        Callback::new(move |_| {
            if !has_next_page.get() || is_fetching_next.get() {
                return;
            }
            
            if let Some(last_page) = pages.get().last() {
                let next_param = (options.get_next_page_param)(last_page);
                if let Some(param) = next_param {
                    fetch_page(Some(param), FetchDirection::Forward);
                }
            }
        })
    };
    
    // Fetch previous page
    let fetch_previous_page = {
        let fetch_page = fetch_page.clone();
        let options = options.clone();
        
        Callback::new(move |_| {
            if !has_previous_page.get() || is_fetching_prev.get() {
                return;
            }
            
            if let Some(get_prev) = &options.get_previous_page_param {
                if let Some(first_page) = pages.get().first() {
                    let prev_param = get_prev(first_page);
                    if let Some(param) = prev_param {
                        fetch_page(Some(param), FetchDirection::Backward);
                    }
                }
            }
        })
    };
    
    InfiniteQueryResult {
        pages: pages.into(),
        page_params: page_params.into(),
        is_loading: is_loading.into(),
        is_fetching: is_fetching.into(),
        is_fetching_next_page: is_fetching_next.into(),
        is_fetching_previous_page: is_fetching_prev.into(),
        has_next_page: has_next_page.into(),
        has_previous_page: has_previous_page.into(),
        is_success: create_memo(move |_| !pages.get().is_empty() && error.get().is_none()).into(),
        is_error: create_memo(move |_| error.get().is_some()).into(),
        error: error.into(),
        fetch_next_page,
        fetch_previous_page,
        refetch: Callback::new({
            let fetch_page = fetch_page.clone();
            move |_| {
                set_pages.set(Vec::new());
                set_page_params.set(Vec::new());
                set_loading.set(true);
                fetch_page(None, FetchDirection::Forward);
            }
        }),
        remove: Callback::new({
            let client = client.clone();
            let key_fn = key_fn.clone();
            move |_| {
                let key = key_fn().into();
                client.remove_queries(QueryKeyPattern::Exact(key));
                set_pages.set(Vec::new());
                set_page_params.set(Vec::new());
            }
        }),
    }
}

/// Virtual infinite query for large datasets
pub fn use_virtual_infinite_query<T, K, F, Fut>(
    key_fn: impl Fn() -> K + 'static,
    query_fn: impl Fn(Range<usize>) -> F + Clone + 'static,
    options: VirtualInfiniteQueryOptions,
) -> VirtualInfiniteQueryResult<T>
where
    T: Clone + 'static,
    K: Into<QueryKey>,
    F: FnOnce(Range<usize>) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<Vec<T>, QueryError>> + 'static,
{
    // Implementation for virtualized scrolling with dynamic loading
    // This would integrate with virtual scrolling libraries
}
5. Request Deduplication & Cancellation
rust// leptos-query/src/dedup/mod.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages request deduplication
pub struct RequestDeduplicator {
    in_flight: Arc<RwLock<HashMap<QueryKey, InFlightRequest>>>,
}

struct InFlightRequest {
    started_at: Instant,
    subscribers: Vec<oneshot::Sender<Result<SerializedData, QueryError>>>,
    abort_controller: AbortController,
}

impl RequestDeduplicator {
    pub fn new() -> Self {
        Self {
            in_flight: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a request and return existing if in flight
    pub async fn dedupe_request<F, Fut>(
        &self,
        key: &QueryKey,
        fetcher: F,
    ) -> Result<SerializedData, QueryError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<SerializedData, QueryError>>,
    {
        // Check if request is in flight
        {
            let mut in_flight = self.in_flight.write().await;
            if let Some(request) = in_flight.get_mut(key) {
                // Subscribe to existing request
                let (tx, rx) = oneshot::channel();
                request.subscribers.push(tx);
                drop(in_flight); // Release lock
                return rx.await.unwrap();
            }
            
            // Start new request
            let abort_controller = AbortController::new();
            in_flight.insert(
                key.clone(),
                InFlightRequest {
                    started_at: Instant::now(),
                    subscribers: Vec::new(),
                    abort_controller: abort_controller.clone(),
                },
            );
        }
        
        // Execute request
        let result = tokio::select! {
            result = fetcher() => result,
            _ = abort_controller.aborted() => {
                Err(QueryError::Cancelled)
            }
        };
        
        // Notify subscribers and cleanup
        {
            let mut in_flight = self.in_flight.write().await;
            if let Some(request) = in_flight.remove(key) {
                for subscriber in request.subscribers {
                    let _ = subscriber.send(result.clone());
                }
            }
        }
        
        result
    }
    
    /// Cancel a request
    pub async fn cancel(&self, key: &QueryKey) {
        let in_flight = self.in_flight.read().await;
        if let Some(request) = in_flight.get(key) {
            request.abort_controller.abort();
        }
    }
    
    /// Cancel all requests matching a pattern
    pub async fn cancel_queries(&self, pattern: &QueryKeyPattern) {
        let in_flight = self.in_flight.read().await;
        for (key, request) in in_flight.iter() {
            if key.matches_pattern(pattern) {
                request.abort_controller.abort();
            }
        }
    }
}

/// Abort controller for cancelling requests
#[derive(Clone)]
pub struct AbortController {
    aborted: Arc<RwLock<bool>>,
    listeners: Arc<RwLock<Vec<oneshot::Sender<()>>>>,
}

impl AbortController {
    pub fn new() -> Self {
        Self {
            aborted: Arc::new(RwLock::new(false)),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn abort(&self) {
        let mut aborted = self.aborted.write().unwrap();
        *aborted = true;
        
        let mut listeners = self.listeners.write().unwrap();
        for listener in listeners.drain(..) {
            let _ = listener.send(());
        }
    }
    
    pub async fn aborted(&self) {
        if *self.aborted.read().unwrap() {
            return;
        }
        
        let (tx, rx) = oneshot::channel();
        self.listeners.write().unwrap().push(tx);
        let _ = rx.await;
    }
}
6. Retry & Error Handling
rust// leptos-query/src/retry/mod.rs

/// Retry configuration
#[derive(Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay: RetryDelay,
    pub retryable_errors: Box<dyn Fn(&QueryError) -> bool>,
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
            retryable_errors: Box::new(|error| {
                // Retry on network errors, not on 4xx
                matches!(error, QueryError::Network(_) | QueryError::Timeout)
            }),
        }
    }
}

#[derive(Clone)]
pub enum RetryDelay {
    Fixed(Duration),
    Linear { initial: Duration, increment: Duration },
    Exponential { initial: Duration, multiplier: f64, max: Duration },
    Custom(Box<dyn Fn(u32) -> Duration>),
}

impl RetryDelay {
    pub fn calculate(&self, attempt: u32) -> Duration {
        match self {
            RetryDelay::Fixed(duration) => *duration,
            RetryDelay::Linear { initial, increment } => {
                *initial + (*increment * attempt)
            }
            RetryDelay::Exponential { initial, multiplier, max } => {
                let delay = initial.as_millis() as f64 * multiplier.powi(attempt as i32);
                Duration::from_millis(delay.min(max.as_millis() as f64) as u64)
            }
            RetryDelay::Custom(f) => f(attempt),
        }
    }
}

/// Execute with retry logic
pub async fn execute_with_retry<F, Fut, T>(
    mut operation: F,
    config: &RetryConfig,
) -> Result<T, QueryError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, QueryError>>,
{
    let mut attempt = 0;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                attempt += 1;
                
                if attempt >= config.max_attempts || !(config.retryable_errors)(&error) {
                    return Err(error);
                }
                
                let delay = config.delay.calculate(attempt);
                sleep(delay).await;
            }
        }
    }
}

/// Query error types
#[derive(Clone, Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("HTTP error: {status}")]
    Http { status: u16, message: String },
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Type mismatch")]
    TypeMismatch,
    
    #[error("Request cancelled")]
    Cancelled,
    
    #[error("Custom error: {0}")]
    Custom(String),
}
7. Persistence & Offline Support
rust// leptos-query/src/persistence/mod.rs

use web_sys::Storage;

/// Persistence adapter trait
pub trait PersistenceAdapter: Send + Sync {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&self, key: &str, value: Vec<u8>);
    fn remove(&self, key: &str);
    fn clear(&self);
}

/// LocalStorage adapter
pub struct LocalStorageAdapter {
    storage: Storage,
}

impl LocalStorageAdapter {
    pub fn new() -> Result<Self, QueryError> {
        let window = web_sys::window().ok_or(QueryError::Custom("No window".into()))?;
        let storage = window
            .local_storage()
            .map_err(|_| QueryError::Custom("No localStorage".into()))?
            .ok_or(QueryError::Custom("localStorage not available".into()))?;
        
        Ok(Self { storage })
    }
}

impl PersistenceAdapter for LocalStorageAdapter {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.storage
            .get_item(key)
            .ok()
            .flatten()
            .and_then(|s| base64::decode(s).ok())
    }
    
    fn set(&self, key: &str, value: Vec<u8>) {
        let encoded = base64::encode(&value);
        let _ = self.storage.set_item(key, &encoded);
    }
    
    fn remove(&self, key: &str) {
        let _ = self.storage.remove_item(key);
    }
    
    fn clear(&self) {
        let _ = self.storage.clear();
    }
}

/// IndexedDB adapter for larger data
pub struct IndexedDBAdapter {
    db_name: String,
    store_name: String,
}

impl IndexedDBAdapter {
    pub async fn new(db_name: String, store_name: String) -> Result<Self, QueryError> {
        // Initialize IndexedDB
        // Implementation details...
        Ok(Self { db_name, store_name })
    }
}

/// Persist queries to storage
pub struct QueryPersister {
    adapter: Box<dyn PersistenceAdapter>,
    serializer: Box<dyn Fn(&CacheEntry) -> Result<Vec<u8>, QueryError>>,
    deserializer: Box<dyn Fn(Vec<u8>) -> Result<CacheEntry, QueryError>>,
}

impl QueryPersister {
    pub fn new(adapter: Box<dyn PersistenceAdapter>) -> Self {
        Self {
            adapter,
            serializer: Box::new(|entry| {
                bincode::serialize(entry).map_err(|e| QueryError::Serialization(e.to_string()))
            }),
            deserializer: Box::new(|bytes| {
                bincode::deserialize(&bytes).map_err(|e| QueryError::Deserialization(e.to_string()))
            }),
        }
    }
    
    pub fn persist_query(&self, key: &QueryKey, entry: &CacheEntry) -> Result<(), QueryError> {
        let key_str = format!("query:{}", key.segments.join(":"));
        let serialized = (self.serializer)(entry)?;
        self.adapter.set(&key_str, serialized);
        Ok(())
    }
    
    pub fn restore_query(&self, key: &QueryKey) -> Option<CacheEntry> {
        let key_str = format!("query:{}", key.segments.join(":"));
        self.adapter
            .get(&key_str)
            .and_then(|bytes| (self.deserializer)(bytes).ok())
    }
    
    pub fn remove_query(&self, key: &QueryKey) {
        let key_str = format!("query:{}", key.segments.join(":"));
        self.adapter.remove(&key_str);
    }
}

/// Offline mutation queue
pub struct OfflineMutationQueue {
    queue: Arc<RwLock<Vec<PendingMutation>>>,
    persister: Option<QueryPersister>,
}

#[derive(Clone, Serialize, Deserialize)]
struct PendingMutation {
    id: String,
    mutation_key: String,
    variables: SerializedData,
    created_at: SystemTime,
    retry_count: u32,
}

impl OfflineMutationQueue {
    pub fn new(persister: Option<QueryPersister>) -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            persister,
        }
    }
    
    pub async fn add_mutation(&self, mutation: PendingMutation) {
        let mut queue = self.queue.write().await;
        queue.push(mutation.clone());
        
        // Persist to storage
        if let Some(persister) = &self.persister {
            // Serialize and persist queue
        }
    }
    
    pub async fn process_queue<F, Fut>(&self, executor: F)
    where
        F: Fn(PendingMutation) -> Fut,
        Fut: Future<Output = Result<(), QueryError>>,
    {
        let mut queue = self.queue.write().await;
        let mut failed = Vec::new();
        
        for mutation in queue.drain(..) {
            match executor(mutation.clone()).await {
                Ok(()) => {
                    // Successfully processed
                }
                Err(_) => {
                    // Re-queue failed mutations
                    failed.push(mutation);
                }
            }
        }
        
        *queue = failed;
    }
}
8. DevTools Integration
rust// leptos-query/src/devtools/mod.rs

use leptos::*;

/// Query DevTools component
#[component]
pub fn QueryDevTools() -> impl IntoView {
    let client = use_context::<QueryClient>().unwrap();
    let (is_open, set_open) = create_signal(false);
    let (selected_query, set_selected_query) = create_signal(None::<QueryKey>);
    
    // Get all queries from cache
    let queries = create_memo(move |_| {
        let cache = client.cache.read().unwrap();
        cache.queries
            .iter()
            .map(|(key, entry)| QueryInfo {
                key: key.clone(),
                state: entry.state.clone(),
                data_size: entry.data.as_ref().map(|d| d.bytes.len()),
                updated_at: entry.updated_at,
                is_stale: entry.is_stale(),
            })
            .collect::<Vec<_>>()
    });
    
    view! {
        <div class="query-devtools">
            <button
                class="devtools-toggle"
                on:click=move |_| set_open.update(|o| *o = !*o)
            >
                "🔍 Query DevTools"
            </button>
            
            <Show when=move || is_open.get()>
                <div class="devtools-panel">
                    <div class="devtools-header">
                        <h3>"Query Inspector"</h3>
                        <div class="devtools-stats">
                            <span>"Total: " {move || queries.get().len()}</span>
                            <span>"Active: " {move || {
                                queries.get().iter()
                                    .filter(|q| q.state == QueryState::Fetching)
                                    .count()
                            }}</span>
                            <span>"Stale: " {move || {
                                queries.get().iter()
                                    .filter(|q| q.is_stale)
                                    .count()
                            }}</span>
                        </div>
                    </div>
                    
                    <div class="devtools-content">
                        <div class="query-list">
                            <For
                                each=move || queries.get()
                                key=|query| query.key.clone()
                                children=move |query| {
                                    let key = query.key.clone();
                                    view! {
                                        <div
                                            class="query-item"
                                            class:selected=move || {
                                                selected_query.get() == Some(key.clone())
                                            }
                                            on:click=move |_| {
                                                set_selected_query.set(Some(key.clone()))
                                            }
                                        >
                                            <div class="query-key">
                                                {query.key.segments.join(" > ")}
                                            </div>
                                            <div class="query-status">
                                                <StatusIndicator state={query.state} />
                                                {query.is_stale.then(|| view! {
                                                    <span class="stale-badge">"Stale"</span>
                                                })}
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        </div>
                        
                        <Show when=move || selected_query.get().is_some()>
                            <QueryDetails
                                client={client.clone()}
                                query_key={selected_query.get().unwrap()}
                            />
                        </Show>
                    </div>
                    
                    <div class="devtools-actions">
                        <button on:click=move |_| {
                            client.invalidate_queries(QueryKeyPattern::Predicate(
                                Box::new(|_| true)
                            ));
                        }>
                            "Invalidate All"
                        </button>
                        <button on:click=move |_| {
                            client.remove_queries(QueryKeyPattern::Predicate(
                                Box::new(|_| true)
                            ));
                        }>
                            "Clear Cache"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn QueryDetails(
    client: QueryClient,
    query_key: QueryKey,
) -> impl IntoView {
    let query_data = create_memo(move |_| {
        client.get_cache_entry(&query_key)
    });
    
    view! {
        <div class="query-details">
            <h4>"Query Details"</h4>
            {move || query_data.get().map(|entry| view! {
                <div>
                    <div>"State: " {format!("{:?}", entry.state)}</div>
                    <div>"Updated: " {format!("{:?}", entry.updated_at)}</div>
                    <div>"Stale Time: " {format!("{:?}", entry.stale_time)}</div>
                    <div>"Cache Time: " {format!("{:?}", entry.cache_time)}</div>
                    
                    <div class="query-actions">
                        <button on:click=move |_| {
                            client.invalidate_queries(
                                QueryKeyPattern::Exact(query_key.clone())
                            );
                        }>
                            "Invalidate"
                        </button>
                        <button on:click=move |_| {
                            client.remove_queries(
                                QueryKeyPattern::Exact(query_key.clone())
                            );
                        }>
                            "Remove"
                        </button>
                    </div>
                    
                    <Show when=move || entry.data.is_some()>
                        <details>
                            <summary>"Data"</summary>
                            <pre class="query-data">
                                {move || {
                                    entry.data.as_ref().map(|d| {
                                        format!("{} bytes", d.bytes.len())
                                    })
                                }}
                            </pre>
                        </details>
                    </Show>
                </div>
            })}
        </div>
    }
}
9. Type-Safe API Client Integration
rust// leptos-query/src/api/mod.rs

use leptos::*;

/// Trait for type-safe API definitions
pub trait TypedApi {
    type Error: Clone + 'static;
}

/// Macro for generating type-safe API clients
#[macro_export]
macro_rules! create_api {
    (
        $(
            #[$meta:meta]
        )*
        pub struct $name:ident {
            base_url: $base_url:expr,
            $(
                $(#[$endpoint_meta:meta])*
                fn $method:ident($($param:ident: $param_type:ty),*) -> $return_type:ty;
            )*
        }
    ) => {
        $(#[$meta])*
        pub struct $name {
            base_url: String,
            client: reqwest::Client,
        }
        
        impl $name {
            pub fn new(base_url: impl Into<String>) -> Self {
                Self {
                    base_url: base_url.into(),
                    client: reqwest::Client::new(),
                }
            }
            
            $(
                $(#[$endpoint_meta])*
                pub async fn $method(&self, $($param: $param_type),*) -> Result<$return_type, QueryError> {
                    // Generated API call implementation
                    todo!()
                }
            )*
        }
    };
}

/// Example API definition
create_api! {
    pub struct UserApi {
        base_url: "https://api.example.com",
        
        fn get_user(id: u32) -> User;
        fn list_users(page: u32, limit: u32) -> Page<User>;
        fn create_user(data: CreateUserDto) -> User;
        fn update_user(id: u32, data: UpdateUserDto) -> User;
        fn delete_user(id: u32) -> ();
    }
}

/// Hook for using the API with queries
pub fn use_api_query<T, F, Fut>(
    endpoint: &'static str,
    params: impl Fn() -> QueryKey + 'static,
    fetcher: impl Fn() -> F + Clone + 'static,
    options: QueryOptions,
) -> QueryResult<T>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
    F: FnOnce() -> Fut + Clone + 'static,
    Fut: Future<Output = Result<T, QueryError>> + 'static,
{
    use_query(
        move || {
            let mut key = vec![endpoint.to_string()];
            key.extend(params().segments);
            QueryKey::new(key)
        },
        fetcher,
        options,
    )
}

/// Hook for API mutations
pub fn use_api_mutation<TData, TVariables, F, Fut>(
    endpoint: &'static str,
    mutation_fn: F,
    invalidates: Vec<&'static str>,
) -> MutationResult<TData, QueryError, TVariables>
where
    TData: Clone + 'static,
    TVariables: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, QueryError>> + 'static,
{
    let client = use_context::<QueryClient>().unwrap();
    
    use_mutation(
        mutation_fn,
        MutationOptions {
            on_success: Some(Box::new(move |_data, _variables, _context| {
                // Invalidate related queries
                for pattern in &invalidates {
                    client.invalidate_queries(QueryKeyPattern::Prefix(
                        QueryKey::new(vec![pattern.to_string()])
                    ));
                }
            })),
            ..Default::default()
        },
    )
}
Usage Examples
Basic Query
rustuse leptos_query::*;

#[derive(Clone, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[component]
fn UserProfile(user_id: u32) -> impl IntoView {
    let user = use_query(
        move || ["users", user_id.to_string()],
        move || async move {
            // Fetch user from API
            let response = reqwest::get(format!("/api/users/{}", user_id))
                .await
                .map_err(|e| QueryError::Network(e.to_string()))?;
            
            response.json::<User>()
                .await
                .map_err(|e| QueryError::Deserialization(e.to_string()))
        },
        QueryOptions {
            stale_time: Duration::from_secs(60),
            ..Default::default()
        },
    );
    
    view! {
        <div>
            <Show
                when=move || user.is_loading.get()
                fallback=move || view! {
                    <Show
                        when=move || user.is_error.get()
                        fallback=move || {
                            let user_data = user.data.get().unwrap();
                            view! {
                                <div>
                                    <h1>{&user_data.name}</h1>
                                    <p>{&user_data.email}</p>
                                </div>
                            }
                        }
                    >
                        <div>"Error loading user"</div>
                    </Show>
                }
            >
                <div>"Loading..."</div>
            </Show>
        </div>
    }
}
Optimistic Update
rust#[component]
fn TodoList() -> impl IntoView {
    let todos = use_query(
        || ["todos"],
        || async { fetch_todos().await },
        Default::default(),
    );
    
    let add_todo = use_optimistic_mutation(
        QueryKey::new(["todos"]),
        |new_todo: CreateTodoDto| async move {
            api::create_todo(new_todo).await
        },
        |new_todo: &CreateTodoDto| {
            // Return optimistic todo
            Todo {
                id: 0, // Temporary ID
                title: new_todo.title.clone(),
                completed: false,
                created_at: Utc::now(),
            }
        },
    );
    
    let (new_todo, set_new_todo) = create_signal(String::new());
    
    view! {
        <div>
            <form on:submit=move |e| {
                e.prevent_default();
                add_todo.mutate.call(CreateTodoDto {
                    title: new_todo.get(),
                });
                set_new_todo.set(String::new());
            }>
                <input
                    type="text"
                    value=new_todo
                    on:input=move |e| set_new_todo.set(event_target_value(&e))
                />
                <button type="submit">"Add Todo"</button>
            </form>
            
            <Show when=move || todos.data.get().is_some()>
                <ul>
                    <For
                        each=move || todos.data.get().unwrap()
                        key=|todo| todo.id
                        children=move |todo| view! {
                            <li>{todo.title}</li>
                        }
                    />
                </ul>
            </Show>
        </div>
    }
}
Infinite Scroll
rust#[component]
fn InfinitePostList() -> impl IntoView {
    let posts = use_infinite_query(
        || ["posts"],
        |cursor| async move {
            fetch_posts(cursor).await
        },
        InfiniteQueryOptions {
            get_next_page_param: Box::new(|last_page| {
                last_page.next_cursor.clone()
            }),
            query_options: Default::default(),
            ..Default::default()
        },
    );
    
    // Intersection observer for infinite scroll
    let load_more_ref = create_node_ref::<html::Div>();
    
    create_effect(move |_| {
        if let Some(element) = load_more_ref.get() {
            let observer = web_sys::IntersectionObserver::new(
                move |entries, _| {
                    if entries[0].is_intersecting() && posts.has_next_page.get() {
                        posts.fetch_next_page.call(());
                    }
                }
            );
            observer.observe(&element);
        }
    });
    
    view! {
        <div>
            <For
                each=move || posts.pages.get().into_iter().flat_map(|p| p.data)
                key=|post| post.id
                children=move |post| view! {
                    <article>
                        <h2>{post.title}</h2>
                        <p>{post.content}</p>
                    </article>
                }
            />
            
            <Show when=move || posts.is_fetching_next_page.get()>
                <div>"Loading more..."</div>
            </Show>
            
            <div ref=load_more_ref />
        </div>
    }
}
Integration with Your Stack
rust// Integration with your radix-leptos and shadcn-ui components
use radix_leptos::*;
use shadcn_ui::*;
use leptos_query::*;

#[component]
fn DataTable<T: Clone + 'static>(
    query_key: QueryKey,
    fetcher: impl Fn() -> Pin<Box<dyn Future<Output = Result<Vec<T>, QueryError>>>> + 'static,
    columns: Vec<Column<T>>,
) -> impl IntoView {
    let data = use_query(
        move || query_key.clone(),
        fetcher,
        QueryOptions {
            refetch_interval: Some(Duration::from_secs(30)),
            ..Default::default()
        },
    );
    
    view! {
        <Card>
            <CardHeader>
                <CardTitle>"Data Table"</CardTitle>
                <div class="flex gap-2">
                    <Button
                        on:click=move |_| data.refetch.call(())
                        disabled=move || data.is_fetching.get()
                    >
                        <RefreshIcon class="mr-2" />
                        "Refresh"
                    </Button>
                    
                    <Show when=move || data.is_stale.get()>
                        <Badge variant="secondary">"Stale"</Badge>
                    </Show>
                </div>
            </CardHeader>
            <CardContent>
                <Show
                    when=move || data.is_loading.get()
                    fallback=move || {
                        <Show
                            when=move || data.data.get().is_some()
                            fallback=|| view! { <div>"No data"</div> }
                        >
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <For
                                            each=move || columns.clone()
                                            key=|col| col.id.clone()
                                            children=move |col| view! {
                                                <TableHead>{col.header}</TableHead>
                                            }
                                        />
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    <For
                                        each=move || data.data.get().unwrap()
                                        key=|item| item.id
                                        children=move |item| view! {
                                            <TableRow>
                                                <For
                                                    each=move || columns.clone()
                                                    key=|col| col.id.clone()
                                                    children=move |col| view! {
                                                        <TableCell>
                                                            {(col.render)(&item)}
                                                        </TableCell>
                                                    }
                                                />
                                            </TableRow>
                                        }
                                    />
                                </TableBody>
                            </Table>
                        </Show>
                    }
                >
                    <Skeleton class="h-[200px]" />
                </Show>
            </CardContent>
        </Card>
    }
}
This comprehensive data fetching/caching layer provides:

Automatic caching with configurable TTL
Request deduplication to prevent duplicate network requests
Optimistic updates for instant UI feedback
Background refetching to keep data fresh
Infinite queries for pagination
Offline support with mutation queue
DevTools for debugging
Type-safe API integration
Retry logic with exponential backoff
Persistence to localStorage/IndexedDB
