//! Query Client and Cache Implementation
//!
//! The core of the Leptos Query system, managing all queries, mutations,
//! and caching behavior.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use leptos::*;
use serde::Serialize;

use crate::types::*;
use crate::retry::*;

/// Global query client configuration
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
            default_cache_time: Duration::from_secs(5 * 60), // 5 minutes
            gc_interval: Duration::from_secs(60), // 1 minute
            max_cache_size: Some(1000),
            default_retry: RetryConfig::default(),
        }
    }
}

/// Global query client that manages all queries and mutations
#[derive(Clone)]
pub struct QueryClient {
    cache: Arc<RwLock<QueryCache>>,
    config: QueryClientConfig,
    query_observers: Arc<RwLock<HashMap<QueryKey, Vec<QueryObserverId>>>>,
}

/// Internal cache structure
pub struct QueryCache {
    queries: HashMap<QueryKey, CacheEntry>,
    // Garbage collection tracking
    access_times: HashMap<QueryKey, Instant>,
}

impl QueryCache {
    pub fn new() -> Self {
        Self {
            queries: HashMap::new(),
            access_times: HashMap::new(),
        }
    }
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

impl CacheEntry {
    pub fn is_stale(&self) -> bool {
        let age = Instant::now().duration_since(self.updated_at);
        age > self.stale_time
    }
    
    pub fn is_expired(&self) -> bool {
        let age = Instant::now().duration_since(self.updated_at);
        age > self.cache_time
    }
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
    pub fn deserialize<T: serde::de::DeserializeOwned + 'static>(&self) -> Result<T, QueryError> {
        if self.type_id != std::any::TypeId::of::<T>() {
            return Err(QueryError::TypeMismatch { expected: std::any::type_name::<T>().to_string(), actual: "unknown".to_string() });
        }
        bincode::deserialize(&self.bytes).map_err(|e| QueryError::Deserialization(e.to_string()))
    }
    
    pub fn serialize<T: Serialize + 'static>(data: &T) -> Result<Self, QueryError> {
        Ok(Self {
            bytes: bincode::serialize(data).map_err(|e| QueryError::Serialization(e.to_string()))?,
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
            .map(|part| serde_json::to_string(part).map_err(|e| QueryError::Serialization(e.to_string())))
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
        }
    }
}

/// Convert string slices to QueryKey
impl<T: ToString + std::fmt::Display> From<&[T]> for QueryKey {
    fn from(segments: &[T]) -> Self {
        Self::new(segments)
    }
}

/// Convert tuple to QueryKey  
impl<T: ToString + std::fmt::Display> From<(T,)> for QueryKey {
    fn from((a,): (T,)) -> Self {
        Self::new([a])
    }
}

/// Pattern for matching query keys during invalidation
#[derive(Clone)]
pub enum QueryKeyPattern {
    Exact(QueryKey),
    Prefix(QueryKey),
    Contains(String),
}

impl QueryClient {
    pub fn new(config: QueryClientConfig) -> Self {
        let client = Self {
            cache: Arc::new(RwLock::new(QueryCache::new())),
            config,
            query_observers: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Start garbage collection
        client.start_garbage_collection();
        
        client
    }
    
    /// Get data from cache
    pub fn get_query_data<T: serde::de::DeserializeOwned + 'static>(
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
    
    /// Get cache entry for inspection
    pub fn get_cache_entry(&self, key: &QueryKey) -> Option<CacheEntry> {
        let cache = self.cache.read().unwrap();
        cache.queries.get(key).cloned()
    }
    
    /// Invalidate queries matching a pattern
    pub fn invalidate_queries(&self, pattern: &QueryKeyPattern) {
        let mut cache = self.cache.write().unwrap();
        let keys_to_invalidate: Vec<QueryKey> = cache
            .queries
            .keys()
            .filter(|key| key.matches_pattern(pattern))
            .cloned()
            .collect();
        
        for key in keys_to_invalidate {
            if let Some(entry) = cache.queries.get_mut(&key) {
                // Mark as stale
                entry.updated_at = Instant::now() - entry.stale_time - Duration::from_secs(1);
            }
        }
    }
    
    /// Remove queries from cache
    pub fn remove_queries(&self, pattern: &QueryKeyPattern) {
        let mut cache = self.cache.write().unwrap();
        cache.queries.retain(|key, _| !key.matches_pattern(pattern));
    }
    
    /// Register a query observer
    pub fn register_query_observer(&self, key: &QueryKey, observer_id: QueryObserverId) {
        let mut observers = self.query_observers.write().unwrap();
        observers.entry(key.clone()).or_insert_with(Vec::new).push(observer_id);
    }
    
    /// Unregister a query observer
    pub fn unregister_query_observer(&self, key: &QueryKey, observer_id: &QueryObserverId) {
        let mut observers = self.query_observers.write().unwrap();
        if let Some(observer_list) = observers.get_mut(key) {
            observer_list.retain(|id| id != observer_id);
            if observer_list.is_empty() {
                observers.remove(key);
            }
        }
    }
    
    fn start_garbage_collection(&self) {
        let cache = self.cache.clone();
        let interval = self.config.gc_interval;
        let max_size = self.config.max_cache_size;
        
        spawn_local(async move {
            loop {
                sleep(interval).await;
                
                let now = Instant::now();
                let mut cache = cache.write().unwrap();
                
                // Remove expired entries
                let keys_to_remove: Vec<QueryKey> = cache
                    .queries
                    .iter()
                    .filter(|(_, entry)| {
                        let age = now.duration_since(entry.updated_at);
                        age >= entry.cache_time
                    })
                    .map(|(key, _)| key.clone())
                    .collect();
                
                for key in keys_to_remove {
                    cache.queries.remove(&key);
                    cache.access_times.remove(&key);
                }
                
                // Enforce max cache size if specified
                if let Some(max) = max_size {
                    if cache.queries.len() > max {
                        // Remove oldest entries
                        let mut entries: Vec<_> = cache.queries.iter().collect();
                        entries.sort_by_key(|(_, entry)| entry.updated_at);
                        
                        let to_remove = cache.queries.len() - max;
                        let keys_to_remove: Vec<QueryKey> = entries.iter().take(to_remove).map(|(key, _)| (*key).clone()).collect();
                        
                        for key in keys_to_remove {
                            cache.queries.remove(&key);
                            cache.access_times.remove(&key);
                        }
                    }
                }
            }
        });
    }
}

/// Provide the QueryClient context to the app
#[component]
pub fn QueryClientProvider(
    #[prop(optional)] config: Option<QueryClientConfig>,
    children: Children,
) -> impl IntoView {
    let client = QueryClient::new(config.unwrap_or_default());
    provide_context(client);
    
    children()
}

// Utility function for sleeping in WASM
async fn sleep(duration: Duration) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve, 
                duration.as_millis() as i32
            )
            .unwrap();
    });
    
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}