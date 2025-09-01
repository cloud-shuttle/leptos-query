//! Query Client
//!
//! The main client for managing query state, caching, and background updates.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Serialize, de::DeserializeOwned};

use crate::types::{QueryKey, QueryStatus, QueryMeta};
use crate::retry::QueryError;

/// Serialized data for caching
#[derive(Clone, Debug)]
pub struct SerializedData {
    pub data: Vec<u8>,
    pub timestamp: Instant,
}

/// Cache entry for a query
#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub data: SerializedData,
    pub meta: QueryMeta,
}

impl CacheEntry {
    /// Check if the cache entry is stale
    pub fn is_stale(&self) -> bool {
        self.meta.is_stale()
    }
    
    /// Get the cached data
    pub fn get_data<T: DeserializeOwned>(&self) -> Result<T, QueryError> {
        bincode::deserialize(&self.data.data)
            .map_err(|e| QueryError::SerializationError(e.to_string()))
    }
}

/// The main query client
#[derive(Clone)]
pub struct QueryClient {
    cache: Arc<RwLock<HashMap<QueryKey, CacheEntry>>>,
    stale_time: Duration,
    cache_time: Duration,
}

impl QueryClient {
    /// Create a new query client
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            stale_time: Duration::from_secs(0),
            cache_time: Duration::from_secs(5 * 60), // 5 minutes
        }
    }
    
    /// Create a new query client with custom settings
    pub fn with_settings(stale_time: Duration, cache_time: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            stale_time,
            cache_time,
        }
    }
    
    /// Get a cache entry for a query key
    pub fn get_cache_entry(&self, key: &QueryKey) -> Option<CacheEntry> {
        let cache = self.cache.read().ok()?;
        cache.get(key).cloned()
    }
    
    /// Set query data in the cache
    pub fn set_query_data<T: Serialize>(
        &self,
        key: &QueryKey,
        data: T,
    ) -> Result<(), QueryError> {
        let serialized = bincode::serialize(&data)
            .map_err(|e| QueryError::SerializationError(e.to_string()))?;
        
        let entry = CacheEntry {
            data: SerializedData {
                data: serialized,
                timestamp: Instant::now(),
            },
            meta: QueryMeta {
                status: QueryStatus::Success,
                updated_at: Instant::now(),
                stale_time: self.stale_time,
                cache_time: self.cache_time,
            },
        };
        
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(key.clone(), entry);
        }
        
        Ok(())
    }
    
    /// Remove a query from the cache
    pub fn remove_query(&self, key: &QueryKey) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(key);
        }
    }
    
    /// Clear all queries from the cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        if let Ok(cache) = self.cache.read() {
            CacheStats {
                total_entries: cache.len(),
                stale_entries: cache.values()
                    .filter(|entry| entry.is_stale())
                    .count(),
            }
        } else {
            CacheStats {
                total_entries: 0,
                stale_entries: 0,
            }
        }
    }
    
    /// Clean up stale entries
    pub fn cleanup_stale_entries(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.retain(|_, entry| !entry.is_stale());
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub stale_entries: usize,
}

impl Default for QueryClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct TestData {
        value: i32,
        text: String,
    }
    
    #[test]
    fn test_cache_operations() {
        let client = QueryClient::new();
        let key = QueryKey::from("test");
        let data = TestData {
            value: 42,
            text: "hello".to_string(),
        };
        
        // Set data
        assert!(client.set_query_data(&key, data.clone()).is_ok());
        
        // Get data
        let entry = client.get_cache_entry(&key);
        assert!(entry.is_some());
        
        let cached_data = entry.unwrap().get_data::<TestData>().unwrap();
        assert_eq!(cached_data, data);
        
        // Remove data
        client.remove_query(&key);
        assert!(client.get_cache_entry(&key).is_none());
    }
    
    #[test]
    fn test_cache_stats() {
        let client = QueryClient::new();
        let key1 = QueryKey::from("test1");
        let key2 = QueryKey::from("test2");
        
        client.set_query_data(&key1, TestData { value: 1, text: "a".to_string() }).unwrap();
        client.set_query_data(&key2, TestData { value: 2, text: "b".to_string() }).unwrap();
        
        let stats = client.cache_stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.stale_entries, 0);
    }
}