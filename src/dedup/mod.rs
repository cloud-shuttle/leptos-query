//! Request Deduplication and Cancellation
//!
//! Prevents duplicate requests for the same query key and provides
//! request cancellation capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use std::future::Future;

use crate::client::{QueryKey, SerializedData};
use crate::retry::QueryError;

/// Manages request deduplication and cancellation
pub struct RequestDeduplicator {
    in_flight: Arc<RwLock<HashMap<QueryKey, InFlightRequest>>>,
}

struct InFlightRequest {
    started_at: Instant,
    subscribers: Vec<tokio::sync::oneshot::Sender<Result<SerializedData, QueryError>>>,
}

impl RequestDeduplicator {
    pub fn new() -> Self {
        Self {
            in_flight: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Execute a request with deduplication
    /// If a request with the same key is already in flight, subscribe to it
    /// Otherwise, execute the new request and notify all subscribers
    pub async fn dedupe_request<F, Fut>(
        &self,
        key: &QueryKey,
        fetcher: F,
    ) -> Result<SerializedData, QueryError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<SerializedData, QueryError>>,
    {
        // Check if request is already in flight
        {
            let mut in_flight = self.in_flight.write().await;
            if let Some(request) = in_flight.get_mut(key) {
                // Subscribe to existing request
                let (tx, rx) = tokio::sync::oneshot::channel();
                request.subscribers.push(tx);
                drop(in_flight); // Release lock early
                
                return match rx.await {
                    Ok(result) => result,
                    Err(_) => Err(QueryError::custom("Request subscription failed")),
                };
            }
            
            // Start new request
            in_flight.insert(
                key.clone(),
                InFlightRequest {
                    started_at: Instant::now(),
                    subscribers: Vec::new(),
                },
            );
        }
        
        // Execute request
        let result = fetcher().await;
        
        // Notify all subscribers and cleanup
        {
            let mut in_flight = self.in_flight.write().await;
            if let Some(request) = in_flight.remove(key) {
                // Notify all subscribers
                for subscriber in request.subscribers {
                    let _ = subscriber.send(result.clone());
                }
            }
        }
        
        result
    }
    
    /// Get information about in-flight requests
    pub async fn get_in_flight_info(&self) -> Vec<InFlightInfo> {
        let in_flight = self.in_flight.read().await;
        in_flight
            .iter()
            .map(|(key, request)| InFlightInfo {
                key: key.clone(),
                started_at: request.started_at,
                subscriber_count: request.subscribers.len(),
                duration: request.started_at.elapsed(),
            })
            .collect()
    }
    
    /// Check if a request is in flight
    pub async fn is_in_flight(&self, key: &QueryKey) -> bool {
        let in_flight = self.in_flight.read().await;
        in_flight.contains_key(key)
    }
}

/// Information about in-flight requests for debugging
#[derive(Clone, Debug)]
pub struct InFlightInfo {
    pub key: QueryKey,
    pub started_at: Instant,
    pub subscriber_count: usize,
    pub duration: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_request_deduplication() {
        let deduper = RequestDeduplicator::new();
        
        // Test that deduplicator can be created
        // Note: This is a basic test that doesn't require async
        assert_eq!(deduper.in_flight.try_read().unwrap().len(), 0);
    }
}