//! Common types for leptos-query-rs
//! 
//! This module provides type-safe abstractions for queries, mutations, and caching.

use std::time::Duration;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Unique identifier for query observers
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryObserverId(pub Uuid);

impl QueryObserverId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for QueryObserverId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for mutations
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationId(pub Uuid);

impl MutationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MutationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Query status enum
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum QueryStatus {
    Idle,
    Loading,
    Success,
    Error,
}

/// Mutation status enum  
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MutationStatus {
    Idle,
    Loading,
    Success,
    Error,
}

/// Query metadata for analytics and debugging
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct QueryMeta {
    pub fetch_count: u32,
    pub error_count: u32,
    pub last_fetch_duration: Option<Duration>,
    pub total_fetch_time: Duration,
}

impl QueryMeta {
    pub fn record_fetch(&mut self, duration: Duration) {
        self.fetch_count += 1;
        self.last_fetch_duration = Some(duration);
        self.total_fetch_time += duration;
    }
    
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }
    
    pub fn average_fetch_time(&self) -> Option<Duration> {
        if self.fetch_count > 0 {
            Some(self.total_fetch_time / self.fetch_count)
        } else {
            None
        }
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.fetch_count == 0 {
            0.0
        } else {
            (self.fetch_count - self.error_count) as f64 / self.fetch_count as f64
        }
    }
}

/// Query key for cache identification
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryKey {
    pub segments: Vec<String>,
}

impl QueryKey {
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }
    
    pub fn from_strs(segments: &[&str]) -> Self {
        Self {
            segments: segments.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl From<Vec<String>> for QueryKey {
    fn from(segments: Vec<String>) -> Self {
        Self::new(segments)
    }
}

impl From<&[&str]> for QueryKey {
    fn from(segments: &[&str]) -> Self {
        Self::from_strs(segments)
    }
}

impl From<String> for QueryKey {
    fn from(segment: String) -> Self {
        Self::new(vec![segment])
    }
}

impl From<&str> for QueryKey {
    fn from(segment: &str) -> Self {
        Self::new(vec![segment])
    }
}

/// Query options for configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryOptions {
    pub stale_time: Option<Duration>,
    pub cache_time: Option<Duration>,
    pub refetch_interval: Option<Duration>,
    pub retry_count: Option<u32>,
    pub retry_delay: Option<Duration>,
    pub enabled: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            stale_time: None,
            cache_time: Some(Duration::from_secs(300)), // 5 minutes
            refetch_interval: None,
            retry_count: Some(3),
            retry_delay: Some(Duration::from_secs(1)),
            enabled: true,
        }
    }
}

impl QueryOptions {
    pub fn with_stale_time(mut self, stale_time: Duration) -> Self {
        self.stale_time = Some(stale_time);
        self
    }
    
    pub fn with_cache_time(mut self, cache_time: Duration) -> Self {
        self.cache_time = Some(cache_time);
        self
    }
    
    pub fn with_refetch_interval(mut self, interval: Duration) -> Self {
        self.refetch_interval = Some(interval);
        self
    }
    
    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = Some(count);
        self
    }
    
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = Some(delay);
        self
    }
    
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Mutation options for configuration
#[derive(Clone, Debug)]
pub struct MutationOptions {
    pub retry_count: Option<u32>,
    pub retry_delay: Option<Duration>,
    pub on_success: Option<Box<dyn Fn() + Send + Sync>>,
    pub on_error: Option<Box<dyn Fn(String) + Send + Sync>>,
}

impl Default for MutationOptions {
    fn default() -> Self {
        Self {
            retry_count: Some(3),
            retry_delay: Some(Duration::from_secs(1)),
            on_success: None,
            on_error: None,
        }
    }
}

impl MutationOptions {
    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = Some(count);
        self
    }
    
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = Some(delay);
        self
    }
    
    pub fn with_on_success<F>(mut self, callback: F) -> Self 
    where F: Fn() + Send + Sync + 'static {
        self.on_success = Some(Box::new(callback));
        self
    }
    
    pub fn with_on_error<F>(mut self, callback: F) -> Self 
    where F: Fn(String) + Send + Sync + 'static {
        self.on_error = Some(Box::new(callback));
        self
    }
}
