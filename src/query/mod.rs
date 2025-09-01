//! Query Hooks and Options
//!
//! The main user-facing API for data fetching with reactive queries.

use leptos::*;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::future::Future;
use serde::{Serialize, de::DeserializeOwned};

use crate::client::{QueryClient, QueryKey, SerializedData};
use crate::retry::{QueryError, RetryConfig, execute_with_retry};
use crate::types::{QueryObserverId, QueryStatus, QueryMeta};

/// Options for configuring a query
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
    /// Timeout for requests
    pub timeout: Option<Duration>,
    /// Success callback
    pub on_success: Option<Callback<SerializedData>>,
    /// Error callback
    pub on_error: Option<Callback<QueryError>>,
    /// Settled callback (success or error)
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
            timeout: Some(Duration::from_secs(30)),
            on_success: None,
            on_error: None,
            on_settled: None,
        }
    }
}

impl QueryOptions {
    /// Create options with custom stale time
    pub fn with_stale_time(mut self, duration: Duration) -> Self {
        self.stale_time = duration;
        self
    }
    
    /// Create options with custom cache time
    pub fn with_cache_time(mut self, duration: Duration) -> Self {
        self.cache_time = duration;
        self
    }
    
    /// Create options with refetch interval
    pub fn with_refetch_interval(mut self, interval: Duration) -> Self {
        self.refetch_interval = Some(interval);
        self
    }
    
    /// Create options with retry configuration
    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }
    
    /// Disable the query by default
    pub fn disabled(mut self) -> Self {
        self.enabled = Signal::derive(|| false);
        self
    }
    
    /// Enable keep previous data
    pub fn keep_previous_data(mut self) -> Self {
        self.keep_previous_data = true;
        self
    }
    
    /// Enable suspense mode
    pub fn with_suspense(mut self) -> Self {
        self.suspense = true;
        self
    }
}

/// Result of a query hook
#[derive(Clone)]
pub struct QueryResult<T: 'static> {
    /// The query data
    pub data: Signal<Option<T>>,
    /// Error if any
    pub error: Signal<Option<QueryError>>,
    /// Whether initially loading (no data yet)
    pub is_loading: Signal<bool>,
    /// Whether fetching (including background)
    pub is_fetching: Signal<bool>,
    /// Whether the query succeeded
    pub is_success: Signal<bool>,
    /// Whether the query failed
    pub is_error: Signal<bool>,
    /// Whether the query is idle
    pub is_idle: Signal<bool>,
    /// Whether the data is stale
    pub is_stale: Signal<bool>,
    /// When data was last updated
    pub data_updated_at: Signal<Option<Instant>>,
    /// When error occurred
    pub error_updated_at: Signal<Option<Instant>>,
    /// Current status
    pub status: Signal<QueryStatus>,
    /// Query metadata
    pub meta: Signal<QueryMeta>,
    
    // Actions
    /// Refetch the query
    pub refetch: Callback<()>,
    /// Invalidate the query (mark as stale)
    pub invalidate: Callback<()>,
    /// Remove from cache
    pub remove: Callback<()>,
    /// Set data directly
    pub set_data: Callback<T>,
}

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
    let (data_updated_at, set_data_updated_at) = create_signal(None::<Instant>);
    let (error_updated_at, set_error_updated_at) = create_signal(None::<Instant>);
    let (meta, set_meta) = create_signal(QueryMeta::default());
    
    // Observer ID for this query instance
    let observer_id = QueryObserverId::new();
    
    // Register observer and handle cleanup
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
    
    // Fetch function with error handling and caching
    let fetch = {
        let client = client.clone();
        let query_fn = query_fn.clone();
        let key = key.clone();
        let options = options.clone();
        
        Rc::new(move |force_fetch: bool| {
            let client = client.clone();
            let query_fn = query_fn.clone();
            let key = key.get();
            let options = options.clone();
            
            spawn_local(async move {
                // Skip if disabled
                if !options.enabled.get() {
                    return;
                }
                
                let fetch_start = Instant::now();
                
                // Set fetching state
                set_fetching.set(true);
                if data.get().is_none() || force_fetch {
                    set_loading.set(true);
                }
                set_status.set(QueryStatus::Loading);
                
                // Clear previous error if force fetching
                if force_fetch {
                    set_error.set(None);
                }
                
                // Execute query with retry logic
                let result = execute_with_retry(|| query_fn()(), &options.retry).await;
                
                let fetch_duration = fetch_start.elapsed();
                
                // Update metadata
                set_meta.update(|meta| {
                    meta.record_fetch(fetch_duration);
                    match &result {
                        Err(_) => meta.record_error(),
                        _ => {}
                    }
                });
                
                // Handle result
                match result {
                    Ok(data_result) => {
                        // Update cache
                        if let Err(_cache_error) = client.set_query_data(&key, data_result.clone()) {
                            // Log cache error silently for now
                        }
                        
                        // Call success callback before moving data_result
                        if let Some(callback) = &options.on_success {
                            if let Ok(serialized) = SerializedData::serialize(&data_result) {
                                callback.call(serialized);
                            }
                        }
                        
                        // Update local state
                        set_data.set(Some(data_result));
                        set_error.set(None);
                        set_status.set(QueryStatus::Success);
                        set_data_updated_at.set(Some(Instant::now()));
                    }
                    Err(err) => {
                        set_error.set(Some(err.clone()));
                        set_status.set(QueryStatus::Error);
                        set_error_updated_at.set(Some(Instant::now()));
                        
                        // Call error callback
                        if let Some(callback) = &options.on_error {
                            callback.call(err);
                        }
                    }
                }
                
                set_loading.set(false);
                set_fetching.set(false);
                
                // Call settled callback
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
                if let Some(entry) = client.get_cache_entry(&current_key) {
                    set_data.set(Some(cached_data));
                    set_data_updated_at.set(Some(entry.data_updated_at));
                    set_status.set(QueryStatus::Success);
                    set_meta.set(entry.meta.clone());
                    
                    // Check if stale and should refetch
                    if entry.is_stale() {
                        fetch(false); // Background fetch
                    }
                } else {
                    set_data.set(Some(cached_data));
                    set_status.set(QueryStatus::Success);
                }
            } else {
                // No cache, fetch immediately
                fetch(false);
            }
        }
    });
    
    // Setup refetch interval
    if let Some(interval) = options.refetch_interval {
        let fetch_clone = fetch.clone();
        let options_clone = options.clone();
        
        let _ = set_interval_with_handle(
            move || {
                if options_clone.enabled.get() {
                    fetch_clone(false);
                }
            },
            interval,
        );
    }
    
    // Create computed signals
    let is_success = create_memo(move |_| status.get() == QueryStatus::Success);
    let is_error = create_memo(move |_| status.get() == QueryStatus::Error);
    let is_idle = create_memo(move |_| status.get() == QueryStatus::Idle);
    let is_stale = create_memo({
        let client = client.clone();
        let key = key.clone();
        move |_| {
            client.get_cache_entry(&key.get())
                .map(|entry| entry.is_stale())
                .unwrap_or(true)
        }
    });
    
    // Create result
    QueryResult {
        data: data.into(),
        error: error.into(),
        is_loading: is_loading.into(),
        is_fetching: is_fetching.into(),
        is_success: is_success.into(),
        is_error: is_error.into(),
        is_idle: is_idle.into(),
        is_stale: is_stale.into(),
        data_updated_at: data_updated_at.into(),
        error_updated_at: error_updated_at.into(),
        status: status.into(),
        meta: meta.into(),
        
        refetch: Callback::new(move |_| fetch(true)),
        invalidate: Callback::new({
            let client = client.clone();
            let key = key.clone();
            move |_| {
                client.invalidate_queries(&crate::client::QueryKeyPattern::Exact(key.get()));
            }
        }),
        remove: Callback::new({
            let client = client.clone();
            let key = key.clone();
            move |_| {
                client.remove_queries(&crate::client::QueryKeyPattern::Exact(key.get()));
            }
        }),
        set_data: Callback::new({
            let client = client.clone();
            let key = key.clone();
            move |new_data: T| {
                if client.set_query_data(&key.get(), new_data.clone()).is_ok() {
                    set_data.set(Some(new_data));
                }
            }
        }),
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_options_builder() {
        let options = QueryOptions::default()
            .with_stale_time(Duration::from_secs(60))
            .with_cache_time(Duration::from_secs(300))
            .keep_previous_data()
            .with_suspense();
        
        assert_eq!(options.stale_time, Duration::from_secs(60));
        assert_eq!(options.cache_time, Duration::from_secs(300));
        assert!(options.keep_previous_data);
        assert!(options.suspense);
    }
}