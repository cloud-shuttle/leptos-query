use crate::retry::QueryError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use web_sys::Storage;

/// Trait for storage backends
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store data with a key
    async fn store(&self, key: &str, data: &[u8]) -> Result<(), QueryError>;
    
    /// Retrieve data by key
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, QueryError>;
    
    /// Remove data by key
    async fn remove(&self, key: &str) -> Result<(), QueryError>;
    
    /// List all keys
    async fn list_keys(&self) -> Result<Vec<String>, QueryError>;
    
    /// Clear all data
    async fn clear(&self) -> Result<(), QueryError>;
    
    /// Get total size of stored data
    async fn size(&self) -> Result<usize, QueryError>;
}

/// In-memory storage backend for testing and fallback
pub struct MemoryBackend {
    data: Arc<parking_lot::RwLock<HashMap<String, Vec<u8>>>>,
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self {
            data: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl StorageBackend for MemoryBackend {
    async fn store(&self, key: &str, data: &[u8]) -> Result<(), QueryError> {
        let mut map = self.data.write();
        map.insert(key.to_string(), data.to_vec());
        Ok(())
    }
    
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, QueryError> {
        let map = self.data.read();
        Ok(map.get(key).cloned())
    }
    
    async fn remove(&self, key: &str) -> Result<(), QueryError> {
        let mut map = self.data.write();
        map.remove(key);
        Ok(())
    }
    
    async fn list_keys(&self) -> Result<Vec<String>, QueryError> {
        let map = self.data.read();
        Ok(map.keys().cloned().collect())
    }
    
    async fn clear(&self) -> Result<(), QueryError> {
        let mut map = self.data.write();
        map.clear();
        Ok(())
    }
    
    async fn size(&self) -> Result<usize, QueryError> {
        let map = self.data.read();
        Ok(map.len())
    }
}

/// Web localStorage backend
#[cfg(target_arch = "wasm32")]
pub struct LocalStorageBackend {
    storage: Storage,
}

#[cfg(target_arch = "wasm32")]
impl LocalStorageBackend {
    pub fn new() -> Result<Self, QueryError> {
        let window = web_sys::window().ok_or_else(|| {
            QueryError::StorageError("window not available".to_string())
        })?;
        
        let storage = window.local_storage().map_err(|_| {
            QueryError::StorageError("localStorage not available".to_string())
        })?.ok_or_else(|| {
            QueryError::StorageError("localStorage not available".to_string())
        })?;
        
        Ok(Self { storage })
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait]
impl StorageBackend for LocalStorageBackend {
    async fn store(&self, key: &str, data: &[u8]) -> Result<(), QueryError> {
        let encoded = base64::encode(data);
        self.storage.set_item(key, &encoded).map_err(|_| {
            QueryError::StorageError("Failed to store data".to_string())
        })?;
        Ok(())
    }
    
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, QueryError> {
        let encoded = self.storage.get_item(key).map_err(|_| {
            QueryError::StorageError("Failed to retrieve data".to_string())
        })?;
        
        match encoded {
            Some(encoded) => {
                let data = base64::decode(&encoded).map_err(|_| {
                    QueryError::StorageError("Failed to decode data".to_string())
                })?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }
    
    async fn remove(&self, key: &str) -> Result<(), QueryError> {
        self.storage.remove_item(key).map_err(|_| {
            QueryError::StorageError("Failed to remove data".to_string())
        })?;
        Ok(())
    }
    
    async fn list_keys(&self) -> Result<Vec<String>, QueryError> {
        // localStorage doesn't have a direct way to list keys
        // This is a limitation - we'd need to maintain a separate index
        Ok(vec![])
    }
    
    async fn clear(&self) -> Result<(), QueryError> {
        self.storage.clear().map_err(|_| {
            QueryError::StorageError("Failed to clear storage".to_string())
        })?;
        Ok(())
    }
    
    async fn size(&self) -> Result<usize, QueryError> {
        // localStorage doesn't provide size information
        Ok(0)
    }
}

/// Configuration for persistence
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Whether persistence is enabled
    pub enabled: bool,
    /// Storage backend type
    pub backend: PersistenceBackend,
    /// Maximum size of cache in bytes
    pub max_size: Option<usize>,
    /// Whether to compress data
    pub compress: bool,
    /// Encryption key (optional)
    pub encryption_key: Option<String>,
    /// Whether to persist offline queue
    pub persist_offline_queue: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: PersistenceBackend::Memory,
            max_size: Some(10 * 1024 * 1024), // 10MB
            compress: false,
            encryption_key: None,
            persist_offline_queue: true,
        }
    }
}

/// Available storage backends
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PersistenceBackend {
    /// In-memory storage (for testing)
    Memory,
    /// Web localStorage
    LocalStorage,
    /// IndexedDB (future)
    IndexedDB,
}

/// Persistence manager for cache and offline queue
pub struct PersistenceManager {
    #[allow(dead_code)]
    config: PersistenceConfig,
    backend: Box<dyn StorageBackend + Send + Sync>,
}

impl PersistenceManager {
    /// Create a new persistence manager
    pub async fn new(config: PersistenceConfig) -> Result<Self, QueryError> {
        let backend = Self::create_backend(&config).await?;
        
        Ok(Self {
            config,
            backend,
        })
    }
    
    /// Create a storage backend based on configuration
    async fn create_backend(config: &PersistenceConfig) -> Result<Box<dyn StorageBackend + Send + Sync>, QueryError> {
        match &config.backend {
            PersistenceBackend::Memory => {
                Ok(Box::new(MemoryBackend::new()))
            }
            PersistenceBackend::LocalStorage => {
                #[cfg(target_arch = "wasm32")]
                {
                    LocalStorageBackend::new().map(|b| Box::new(b) as Box<dyn StorageBackend + Send + Sync>)
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Err(QueryError::StorageError("localStorage not available on this platform".to_string()))
                }
            }
            PersistenceBackend::IndexedDB => {
                Err(QueryError::StorageError("IndexedDB backend not yet implemented".to_string()))
            }
        }
    }
    
    /// Store a cache entry
    pub async fn store_cache_entry(&self, key: &crate::types::QueryKey, entry: &crate::client::CacheEntry) -> Result<(), QueryError> {
        let data = bincode::serialize(entry)
            .map_err(|e| QueryError::StorageError(format!("Serialization failed: {}", e)))?;
        
        let key_str = key.to_string();
        self.backend.store(&key_str, &data).await
    }
    
    /// Retrieve a cache entry
    pub async fn retrieve_cache_entry(&self, key: &crate::types::QueryKey) -> Result<Option<crate::client::CacheEntry>, QueryError> {
        let key_str = key.to_string();
        if let Some(data) = self.backend.retrieve(&key_str).await? {
            let entry: crate::client::CacheEntry = bincode::deserialize(&data)
                .map_err(|e| QueryError::StorageError(format!("Deserialization failed: {}", e)))?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }
    
    /// Remove a cache entry
    pub async fn remove_cache_entry(&self, key: &crate::types::QueryKey) -> Result<(), QueryError> {
        let key_str = key.to_string();
        self.backend.remove(&key_str).await
    }
    
    /// List all cached keys
    pub async fn list_cached_keys(&self) -> Result<Vec<crate::types::QueryKey>, QueryError> {
        let keys = self.backend.list_keys().await?;
        let mut query_keys = Vec::new();
        
        for key_str in keys {
            // Try to parse as QueryKey
            if let Ok(key) = serde_json::from_str(&key_str) {
                query_keys.push(key);
            }
        }
        
        Ok(query_keys)
    }
    
    /// Clear all cache data
    pub async fn clear_cache(&self) -> Result<(), QueryError> {
        self.backend.clear().await
    }
    
    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats, QueryError> {
        let size = self.backend.size().await?;
        Ok(StorageStats {
            total_entries: size,
            total_size_bytes: 0, // Would need to calculate this
        })
    }
    
    /// Add a request to the offline queue
    pub async fn add_to_offline_queue(&self, request: OfflineRequest) -> Result<(), QueryError> {
        let data = bincode::serialize(&request)
            .map_err(|e| QueryError::StorageError(format!("Serialization failed: {}", e)))?;
        
        let key = format!("offline_queue_{}", request.timestamp.elapsed().as_millis());
        self.backend.store(&key, &data).await
    }
    
    /// Process the offline queue
    pub async fn process_offline_queue(&self) -> Result<Vec<OfflineRequest>, QueryError> {
        let keys = self.backend.list_keys().await?;
        let mut requests = Vec::new();
        
        for key in keys {
            if key.starts_with("offline_queue_") {
                if let Some(data) = self.backend.retrieve(&key).await? {
                    if let Ok(request) = bincode::deserialize::<OfflineRequest>(&data) {
                        requests.push(request);
                    }
                }
                // Remove the processed request
                let _ = self.backend.remove(&key).await;
            }
        }
        
        Ok(requests)
    }

    /// Get the offline queue
    pub fn get_offline_queue(&self) -> Vec<OfflineRequest> {
        // This is a simplified implementation
        // In a real implementation, this would read from storage
        Vec::new()
    }

    /// Check if cache is persisted
    pub fn is_cache_persisted(&self) -> bool {
        // For now, return true if we have any persistence backend
        // In a real implementation, this would check actual persistence status
        true
    }
}

/// Storage statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total number of entries
    pub total_entries: usize,
    /// Total size in bytes
    pub total_size_bytes: usize,
}

/// Offline request for queueing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfflineRequest {
    /// Type of request
    pub request_type: OfflineRequestType,
    /// Request data (serialized)
    pub data: Vec<u8>,
    /// Timestamp when request was queued
    #[serde(with = "instant_serde")]
    pub timestamp: Instant,
    /// Retry count
    pub retry_count: u32,
}

/// Types of offline requests
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OfflineRequestType {
    /// Query request
    Query,
    /// Mutation request
    Mutation,
    /// Cache invalidation
    Invalidate,
    /// Cache removal
    Remove,
}

/// Serialization helpers for Instant
mod instant_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert Instant to SystemTime for serialization
        let system_time = SystemTime::now() - instant.elapsed();
        let duration = system_time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
        duration.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let duration = Duration::deserialize(deserializer)?;
        let system_time = UNIX_EPOCH + duration;
        let now = SystemTime::now();
        let elapsed = now.duration_since(system_time).unwrap_or(Duration::ZERO);
        Ok(Instant::now() - elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_backend() {
        let backend = MemoryBackend::new();
        
        // Test store and retrieve
        backend.store("test_key", b"test_data").await.unwrap();
        let data = backend.retrieve("test_key").await.unwrap();
        assert_eq!(data, Some(b"test_data".to_vec()));
        
        // Test remove
        backend.remove("test_key").await.unwrap();
        let data = backend.retrieve("test_key").await.unwrap();
        assert_eq!(data, None);
        
        // Test list keys
        backend.store("key1", b"data1").await.unwrap();
        backend.store("key2", b"data2").await.unwrap();
        let keys = backend.list_keys().await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        
        // Test clear
        backend.clear().await.unwrap();
        let keys = backend.list_keys().await.unwrap();
        assert_eq!(keys.len(), 0);
    }
    
    #[tokio::test]
    async fn test_persistence_manager() {
        let config = PersistenceConfig::default();
        let manager = PersistenceManager::new(config).await.unwrap();
        
        // Test stats
        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }
    
    #[tokio::test]
    async fn test_offline_queue() {
        let config = PersistenceConfig::default();
        let manager = PersistenceManager::new(config).await.unwrap();
        
        let request = OfflineRequest {
            request_type: OfflineRequestType::Query,
            data: b"test_data".to_vec(),
            timestamp: Instant::now(),
            retry_count: 0,
        };
        
        manager.add_to_offline_queue(request.clone()).await.unwrap();
        let requests = manager.process_offline_queue().await.unwrap();
        assert_eq!(requests.len(), 1);
    }
}
