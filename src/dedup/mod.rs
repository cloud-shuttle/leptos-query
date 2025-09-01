//! Request deduplication
//!
//! Prevents duplicate requests for the same data by tracking in-flight requests.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::future::Future;
use tokio::sync::oneshot;

use crate::client::SerializedData;
use crate::types::QueryKey;
use crate::retry::QueryError;

// Type alias to reduce complexity
type InFlightMap = Arc<RwLock<HashMap<QueryKey, oneshot::Sender<Result<SerializedData, QueryError>>>>>;

/// Request deduplicator
#[derive(Clone)]
pub struct RequestDeduplicator {
    in_flight: InFlightMap,
}

impl RequestDeduplicator {
    /// Create a new deduplicator
    pub fn new() -> Self {
        Self {
            in_flight: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Execute a request, deduplicating if necessary
    pub async fn execute<T, F, Fut>(
        &self,
        key: QueryKey,
        request_fn: F,
    ) -> Result<T, QueryError>
    where
        T: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static,
        F: FnOnce() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, QueryError>> + Send + Sync + 'static,
    {
        // Check if there's already a request in flight and get receiver if exists
        let existing_receiver = {
            let in_flight = self.in_flight.read().unwrap();
            if let Some(_sender) = in_flight.get(&key) {
                // Subscribe to existing request
                let (_new_sender, receiver) = oneshot::channel::<Result<SerializedData, QueryError>>();
                Some(receiver)
            } else {
                None
            }
        }; // Lock is dropped here
        
        // If we have an existing receiver, wait for the result
        if let Some(receiver) = existing_receiver {
            match receiver.await {
                Ok(result) => {
                    return result.and_then(|data| {
                        bincode::deserialize(&data.data)
                            .map_err(|e| QueryError::SerializationError(e.to_string()))
                    });
                }
                Err(_) => {
                    // The original request failed, we'll start a new one
                }
            }
        }
        
        // Create a new request
        let (sender, _receiver) = oneshot::channel();
        
        // Store the sender
        {
            let mut in_flight = self.in_flight.write().unwrap();
            in_flight.insert(key.clone(), sender);
        }
        
        // Execute the request
        let result = request_fn().await;
        let serialized_result = result.and_then(|data| {
            bincode::serialize(&data)
                .map(|bytes| SerializedData {
                    data: bytes,
                    timestamp: std::time::Instant::now(),
                })
                .map_err(|e| QueryError::SerializationError(e.to_string()))
        });
        
        // Remove from in-flight requests
        {
            let mut in_flight = self.in_flight.write().unwrap();
            in_flight.remove(&key);
        }
        
        // Deserialize the result
        serialized_result.and_then(|data| {
            bincode::deserialize(&data.data)
                .map_err(|e| QueryError::SerializationError(e.to_string()))
        })
    }
    
    /// Check if a request is in flight
    pub fn is_in_flight(&self, key: &QueryKey) -> bool {
        self.in_flight.read().unwrap().contains_key(key)
    }
    
    /// Get the number of in-flight requests
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.read().unwrap().len()
    }
    
    /// Clear all in-flight requests
    pub fn clear(&self) {
        self.in_flight.write().unwrap().clear();
    }
}

impl Default for RequestDeduplicator {
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
    }
    
    #[tokio::test]
    async fn test_deduplication() {
        let dedup = RequestDeduplicator::new();
        let key = QueryKey::from("test");
        
        // Create a slow request function
        let request_fn = || async {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            Ok(TestData { value: 42 })
        };
        
        // Start two concurrent requests
        let future1 = dedup.execute(key.clone(), request_fn);
        let future2 = dedup.execute(key.clone(), request_fn);
        
        // Both should return the same result
        let (result1, result2) = tokio::join!(future1, future2);
        
        assert_eq!(result1.unwrap(), TestData { value: 42 });
        assert_eq!(result2.unwrap(), TestData { value: 42 });
        
        // Should not be in flight anymore
        assert!(!dedup.is_in_flight(&key));
    }
    
    #[tokio::test]
    async fn test_error_propagation() {
        let dedup = RequestDeduplicator::new();
        let key = QueryKey::from("error_test");
        
        let request_fn = || async {
            Err(QueryError::GenericError("Test error".to_string()))
        };
        
        let result: Result<TestData, QueryError> = dedup.execute(key.clone(), request_fn).await;
        assert!(result.is_err());
        
        // Should not be in flight anymore
        assert!(!dedup.is_in_flight(&key));
    }
}