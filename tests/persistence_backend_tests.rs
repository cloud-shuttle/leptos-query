//! Persistence Backend Tests
//! 
//! These tests verify that different persistence backends work correctly
//! for storing and retrieving query data.

use leptos_query_rs::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    name: String,
    value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "persistence")]
    fn test_local_storage_backend_creation() {
        // RED: Test that LocalStorageBackend can be created
        let backend = LocalStorageBackend::new("test_prefix".to_string());
        assert_eq!(backend.prefix(), "test_prefix");
    }

    #[test]
    #[cfg(feature = "persistence")]
    fn test_local_storage_backend_store_and_retrieve() {
        // RED: Test storing and retrieving data from LocalStorage
        let backend = LocalStorageBackend::new("test_prefix".to_string());
        let key = QueryKey::new(&["test", "data"]);
        let data = TestData {
            id: 1,
            name: "Test Item".to_string(),
            value: "Test Value".to_string(),
        };

        // Store data
        backend.store(&key, &data).unwrap();

        // Retrieve data
        let retrieved = backend.retrieve::<TestData>(&key).unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[test]
    #[cfg(feature = "persistence")]
    fn test_local_storage_backend_remove() {
        // RED: Test removing data from LocalStorage
        let backend = LocalStorageBackend::new("test_prefix".to_string());
        let key = QueryKey::new(&["test", "remove"]);
        let data = TestData {
            id: 2,
            name: "To Remove".to_string(),
            value: "Will be removed".to_string(),
        };

        // Store data
        backend.store(&key, &data).unwrap();

        // Verify it exists
        let retrieved = backend.retrieve::<TestData>(&key).unwrap();
        assert_eq!(retrieved, Some(data.clone()));

        // Remove data
        backend.remove(&key).unwrap();

        // Verify it's gone
        let retrieved = backend.retrieve::<TestData>(&key).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    #[cfg(feature = "persistence")]
    fn test_local_storage_backend_clear() {
        // RED: Test clearing all data from LocalStorage
        let backend = LocalStorageBackend::new("test_prefix".to_string());
        
        // Store multiple items
        for i in 0..5 {
            let key = QueryKey::new(&["test", "clear", &i.to_string()]);
            let data = TestData {
                id: i,
                name: format!("Item {}", i),
                value: format!("Value {}", i),
            };
            backend.store(&key, &data).unwrap();
        }

        // Verify all items exist
        for i in 0..5 {
            let key = QueryKey::new(&["test", "clear", &i.to_string()]);
            let retrieved = backend.retrieve::<TestData>(&key).unwrap();
            assert!(retrieved.is_some());
        }

        // Clear all data
        backend.clear().unwrap();

        // Verify all items are gone
        for i in 0..5 {
            let key = QueryKey::new(&["test", "clear", &i.to_string()]);
            let retrieved = backend.retrieve::<TestData>(&key).unwrap();
            assert_eq!(retrieved, None);
        }
    }

    #[test]
    #[cfg(feature = "persistence")]
    fn test_indexeddb_backend_creation() {
        // RED: Test that IndexedDBBackend can be created
        let backend = IndexedDBBackend::new("test_db".to_string(), "test_store".to_string());
        assert_eq!(backend.db_name(), "test_db");
        assert_eq!(backend.store_name(), "test_store");
    }

    #[test]
    #[cfg(feature = "persistence")]
    fn test_indexeddb_backend_store_and_retrieve() {
        // RED: Test storing and retrieving data from IndexedDB
        let backend = IndexedDBBackend::new("test_db".to_string(), "test_store".to_string());
        let key = QueryKey::new(&["indexeddb", "test"]);
        let data = TestData {
            id: 3,
            name: "IndexedDB Item".to_string(),
            value: "IndexedDB Value".to_string(),
        };

        // Store data
        backend.store(&key, &data).unwrap();

        // Retrieve data
        let retrieved = backend.retrieve::<TestData>(&key).unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[test]
    #[cfg(feature = "persistence")]
    fn test_indexeddb_backend_remove() {
        // RED: Test removing data from IndexedDB
        let backend = IndexedDBBackend::new("test_db".to_string(), "test_store".to_string());
        let key = QueryKey::new(&["indexeddb", "remove"]);
        let data = TestData {
            id: 4,
            name: "To Remove from IDB".to_string(),
            value: "Will be removed from IDB".to_string(),
        };

        // Store data
        backend.store(&key, &data).unwrap();

        // Verify it exists
        let retrieved = backend.retrieve::<TestData>(&key).unwrap();
        assert_eq!(retrieved, Some(data.clone()));

        // Remove data
        backend.remove(&key).unwrap();

        // Verify it's gone
        let retrieved = backend.retrieve::<TestData>(&key).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    #[cfg(feature = "persistence")]
    fn test_persistence_backend_trait() {
        // RED: Test that backends implement the StorageBackend trait
        let local_backend = LocalStorageBackend::new("test_prefix".to_string());
        let indexeddb_backend = IndexedDBBackend::new("test_db".to_string(), "test_store".to_string());
        
        let key = QueryKey::new(&["trait", "test"]);
        let data = TestData {
            id: 5,
            name: "Trait Test".to_string(),
            value: "Trait Value".to_string(),
        };

        // Test LocalStorage backend
        local_backend.store(&key, &data).unwrap();
        let retrieved = local_backend.retrieve::<TestData>(&key).unwrap();
        assert_eq!(retrieved, Some(data.clone()));

        // Test IndexedDB backend
        indexeddb_backend.store(&key, &data).unwrap();
        let retrieved = indexeddb_backend.retrieve::<TestData>(&key).unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[test]
    #[cfg(feature = "persistence")]
    fn test_persistence_backend_performance() {
        // RED: Test performance of persistence backends
        let local_backend = LocalStorageBackend::new("perf_test".to_string());
        let indexeddb_backend = IndexedDBBackend::new("perf_db".to_string(), "perf_store".to_string());
        
        let test_data = TestData {
            id: 999,
            name: "Performance Test".to_string(),
            value: "Performance Value".repeat(100), // Large data
        };

        // Test LocalStorage performance
        let start = std::time::Instant::now();
        for i in 0..100 {
            let key = QueryKey::new(&["perf", "local", &i.to_string()]);
            local_backend.store(&key, &test_data).unwrap();
        }
        let local_store_time = start.elapsed();

        let start = std::time::Instant::now();
        for i in 0..100 {
            let key = QueryKey::new(&["perf", "local", &i.to_string()]);
            local_backend.retrieve::<TestData>(&key).unwrap();
        }
        let local_retrieve_time = start.elapsed();

        // Test IndexedDB performance
        let start = std::time::Instant::now();
        for i in 0..100 {
            let key = QueryKey::new(&["perf", "idb", &i.to_string()]);
            indexeddb_backend.store(&key, &test_data).unwrap();
        }
        let idb_store_time = start.elapsed();

        let start = std::time::Instant::now();
        for i in 0..100 {
            let key = QueryKey::new(&["perf", "idb", &i.to_string()]);
            indexeddb_backend.retrieve::<TestData>(&key).unwrap();
        }
        let idb_retrieve_time = start.elapsed();

        // Performance assertions (should be reasonably fast)
        assert!(local_store_time < Duration::from_secs(1));
        assert!(local_retrieve_time < Duration::from_secs(1));
        assert!(idb_store_time < Duration::from_secs(2));
        assert!(idb_retrieve_time < Duration::from_secs(2));
    }
}

// Fallback tests for when persistence feature is not enabled
#[cfg(not(feature = "persistence"))]
mod fallback_tests {
    use super::*;

    #[test]
    fn test_persistence_feature_not_enabled() {
        // Test that persistence features gracefully degrade when not enabled
        let client = QueryClient::new();
        let key = QueryKey::new(&["fallback", "persistence"]);
        let data = TestData {
            id: 1,
            name: "Fallback Test".to_string(),
            value: "Fallback Value".to_string(),
        };

        // Basic functionality should still work
        client.set_query_data(&key, data.clone()).unwrap();
        let retrieved = client.get_query_data::<TestData>(&key);
        assert_eq!(retrieved, Some(data));
    }
}
