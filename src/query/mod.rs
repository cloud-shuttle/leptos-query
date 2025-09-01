//! Query Hooks and Options
//!
//! The main user-facing API for data fetching with reactive queries.

use leptos::prelude::*;
use leptos::task::spawn_local;
use std::time::Duration;
use std::future::Future;
use serde::{Serialize, de::DeserializeOwned};

use crate::client::QueryClient;
use crate::retry::{QueryError, RetryConfig, execute_with_retry};
use crate::types::{QueryStatus, QueryKey};

/// Options for configuring a query
#[derive(Clone)]
pub struct QueryOptions {
    /// Whether the query should run
    pub enabled: bool,
    /// Time before data becomes stale
    pub stale_time: Duration,
    /// Time before data is removed from cache
    pub cache_time: Duration,
    /// Interval for background refetching
    pub refetch_interval: Option<Duration>,
    /// Retry configuration
    pub retry: RetryConfig,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            stale_time: Duration::from_secs(0),
            cache_time: Duration::from_secs(5 * 60), // 5 minutes
            refetch_interval: None,
            retry: RetryConfig::default(),
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
        self.enabled = false;
        self
    }
}

/// Result of a query hook
#[derive(Clone)]
pub struct QueryResult<T: 'static + Send + Sync> {
    /// The query data
    pub data: Signal<Option<T>>,
    /// Error if any
    pub error: Signal<Option<QueryError>>,
    /// Whether the query is loading
    pub is_loading: Signal<bool>,
    /// Whether the query succeeded
    pub is_success: Signal<bool>,
    /// Whether the query failed
    pub is_error: Signal<bool>,
    /// Current query status
    pub status: Signal<QueryStatus>,
    
    // Actions
    /// Refetch the query
    pub refetch: Callback<()>,
}

/// Main query hook
pub fn use_query<T, F, Fut>(
    key_fn: F,
    query_fn: impl Fn() -> Fut + Clone + Send + Sync + 'static,
    options: QueryOptions,
) -> QueryResult<T>
where
    T: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
    F: Fn() -> QueryKey + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<T, QueryError>> + 'static,
{
    // Create reactive state
    let (data, set_data) = signal(None::<T>);
    let (error, set_error) = signal(None::<QueryError>);
    let (is_loading, set_loading) = signal(true);
    let (status, set_status) = signal(QueryStatus::Loading);

    // Get query client from context
    let client = use_context::<QueryClient>().expect("QueryClient not found in context");
    
    // Create key signal
    let key = Memo::new(move |_| key_fn());
    
    // Create fetch function for initial fetch
    let initial_fetch = {
        let client = client.clone();
        let query_fn = query_fn.clone();
        let options = options.clone();
        
        move |force: bool| {
            let client = client.clone();
            let query_fn = query_fn.clone();
            let options = options.clone();
            
            spawn_local(async move {
                let current_key = key.get();
                
                // Check cache first
                if let Some(cache_entry) = client.get_cache_entry(&current_key) {
                    if !force && !cache_entry.is_stale() {
                        // Use cached data
                        if let Ok(cached_data) = cache_entry.get_data::<T>() {
                            set_data.set(Some(cached_data));
                            set_loading.set(false);
                            set_status.set(QueryStatus::Success);
                            return;
                        }
                    }
                }
                
                // Fetch new data
                set_status.set(QueryStatus::Loading);
                
                let result = execute_with_retry(
                    &query_fn,
                    &options.retry,
                ).await;
                
                match result {
                    Ok(result_data) => {
                        // Cache the data
                        if let Ok(()) = client.set_query_data(&current_key, result_data.clone()) {
                            set_data.set(Some(result_data));
                            set_error.set(None);
                            set_status.set(QueryStatus::Success);
                        }
                    }
                    Err(err) => {
                        set_error.set(Some(err.clone()));
                        set_status.set(QueryStatus::Error);
                    }
                }
                
                set_loading.set(false);
            });
        }
    };
    
    // Create fetch function for refetch
    let refetch_fn = {
        let client = client.clone();
        let query_fn = query_fn.clone();
        let options = options.clone();
        
        move |force: bool| {
            let client = client.clone();
            let query_fn = query_fn.clone();
            let options = options.clone();
            
            spawn_local(async move {
                let current_key = key.get();
                
                // Check cache first
                if let Some(cache_entry) = client.get_cache_entry(&current_key) {
                    if !force && !cache_entry.is_stale() {
                        // Use cached data
                        if let Ok(cached_data) = cache_entry.get_data::<T>() {
                            set_data.set(Some(cached_data));
                            set_loading.set(false);
                            set_status.set(QueryStatus::Success);
                            return;
                        }
                    }
                }
                
                // Fetch new data
                set_status.set(QueryStatus::Loading);
                
                let result = execute_with_retry(
                    &query_fn,
                    &options.retry,
                ).await;
                
                match result {
                    Ok(result_data) => {
                        // Cache the data
                        if let Ok(()) = client.set_query_data(&current_key, result_data.clone()) {
                            set_data.set(Some(result_data));
                            set_error.set(None);
                            set_status.set(QueryStatus::Success);
                        }
                    }
                    Err(err) => {
                        set_error.set(Some(err.clone()));
                        set_status.set(QueryStatus::Error);
                    }
                }
                
                set_loading.set(false);
            });
        }
    };
    
    // Initial fetch
    Effect::new(move |_| {
        if options.enabled {
            let current_key = key.get();
            
            // Check cache first
            if let Some(cache_entry) = client.get_cache_entry(&current_key) {
                if !cache_entry.is_stale() {
                    // Use cached data
                    if let Ok(cached_data) = cache_entry.get_data::<T>() {
                        set_data.set(Some(cached_data));
                        set_loading.set(false);
                        set_status.set(QueryStatus::Success);
                    }
                } else {
                    // Cache is stale, refetch
                    initial_fetch(false);
                }
            } else {
                // No cache, fetch immediately
                initial_fetch(false);
            }
        }
    });
    
    // Create computed signals
    let is_success = Memo::new(move |_| status.get() == QueryStatus::Success);
    let is_error = Memo::new(move |_| status.get() == QueryStatus::Error);
    
    // Create result
    QueryResult {
        data: data.into(),
        error: error.into(),
        is_loading: is_loading.into(),
        is_success: is_success.into(),
        is_error: is_error.into(),
        status: status.into(),
        refetch: Callback::new(move |_| refetch_fn(true)),
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
            .disabled();
        
        assert_eq!(options.stale_time, Duration::from_secs(60));
        assert_eq!(options.cache_time, Duration::from_secs(300));
        assert!(!options.enabled);
    }
}