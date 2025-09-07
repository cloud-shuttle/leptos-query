//! DevTools Tests
//! 
//! These tests verify that DevTools functionality works correctly
//! for monitoring and debugging query operations.

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
    #[cfg(feature = "devtools")]
    fn test_devtools_manager_creation() {
        // RED: Test that DevToolsManager can be created
        let config = DevToolsConfig::default();
        let manager = DevToolsManager::new(config);
        assert!(manager.is_enabled());
    }

    #[test]
    #[cfg(feature = "devtools")]
    fn test_devtools_manager_start_server() {
        // RED: Test that DevTools server can be started
        let config = DevToolsConfig::default();
        let mut manager = DevToolsManager::new(config);
        
        let server = manager.start_server("localhost:3001".to_string()).unwrap();
        assert_eq!(server.port(), 3001);
        assert_eq!(server.host(), "localhost");
    }

    #[test]
    #[cfg(feature = "devtools")]
    fn test_devtools_query_metrics() {
        // RED: Test that query metrics are collected
        let config = DevToolsConfig::default();
        let mut manager = DevToolsManager::new(config);
        
        // Simulate a query operation
        let key = QueryKey::new(&["test", "metrics"]);
        let start_time = std::time::Instant::now();
        
        manager.record_query_start(&key);
        
        // Simulate query completion
        std::thread::sleep(Duration::from_millis(10));
        manager.record_query_success(&key, start_time.elapsed());
        
        let metrics = manager.get_query_metrics(&key);
        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        assert!(metrics.total_requests > 0);
        assert!(metrics.average_response_time > Duration::ZERO);
    }

    #[test]
    #[cfg(feature = "devtools")]
    fn test_devtools_cache_operations() {
        // RED: Test that cache operations are tracked
        let config = DevToolsConfig::default();
        let mut manager = DevToolsManager::new(config);
        
        let key = QueryKey::new(&["test", "cache"]);
        let data = TestData {
            id: 1,
            name: "Test".to_string(),
            value: "Value".to_string(),
        };
        
        // Record cache operations
        manager.record_cache_operation(CacheOperation::Set { key: key.clone(), size: 100, timestamp: std::time::Instant::now() }, &key, Some(&data));
        manager.record_cache_operation(CacheOperation::Get { key: key.clone(), hit: true, timestamp: std::time::Instant::now() }, &key, None::<&TestData>);
        manager.record_cache_operation(CacheOperation::Remove { key: key.clone(), timestamp: std::time::Instant::now() }, &key, None::<&TestData>);
        
        let events = manager.get_recent_events(10);
        assert!(events.len() >= 3);
        
        // Check that we have the right types of events
        let cache_events: Vec<_> = events.iter()
            .filter(|event| matches!(event, DevToolsEvent::CacheOperation { operation: _, timestamp: _ }))
            .collect();
        assert_eq!(cache_events.len(), 3);
    }

    #[test]
    #[cfg(feature = "devtools")]
    fn test_devtools_network_requests() {
        // RED: Test that network requests are tracked
        let config = DevToolsConfig::default();
        let mut manager = DevToolsManager::new(config);
        
        let key = QueryKey::new(&["test", "network"]);
        let request = NetworkRequest::new(key.clone(), "https://api.example.com/test".to_string(), "GET".to_string());
        
        manager.record_network_request(&key, request);
        
        let events = manager.get_recent_events(10);
        assert!(events.len() >= 1);
        
        let network_events: Vec<_> = events.iter()
            .filter(|event| matches!(event, DevToolsEvent::NetworkRequest { request: _ }))
            .collect();
        assert_eq!(network_events.len(), 1);
    }

    #[test]
    #[cfg(feature = "devtools")]
    fn test_devtools_export_data() {
        // RED: Test that DevTools can export data
        let config = DevToolsConfig::default();
        let mut manager = DevToolsManager::new(config);
        
        // Add some test data
        let key = QueryKey::new(&["test", "export"]);
        manager.record_query_start(&key);
        manager.record_query_success(&key, Duration::from_millis(100));
        
        // Add cache operations
        let data = TestData {
            id: 1,
            name: "Test".to_string(),
            value: "Value".to_string(),
        };
        manager.record_cache_operation(CacheOperation::Set { key: key.clone(), size: 100, timestamp: std::time::Instant::now() }, &key, Some(&data));
        
        // Add network request
        let request = NetworkRequest::new(key.clone(), "https://api.example.com/test".to_string(), "GET".to_string());
        manager.record_network_request(&key, request);
        
        let export = manager.export_data();
        assert!(export.timestamp > 0);
        assert!(!export.query_metrics.is_empty());
        assert!(!export.cache_operations.is_empty());
        assert!(!export.network_requests.is_empty());
    }

    #[test]
    #[cfg(feature = "devtools")]
    fn test_devtools_real_time_monitoring() {
        // RED: Test real-time monitoring capabilities
        let config = DevToolsConfig::default();
        let mut manager = DevToolsManager::new(config);
        
        // Start monitoring
        manager.start_monitoring();
        assert!(manager.is_monitoring());
        
        // Simulate some activity
        let key = QueryKey::new(&["test", "monitoring"]);
        manager.record_query_start(&key);
        manager.record_query_success(&key, Duration::from_millis(50));
        
        // Check that monitoring captured the activity
        let recent_events = manager.get_recent_events(5);
        assert!(!recent_events.is_empty());
        
        // Stop monitoring
        manager.stop_monitoring();
        assert!(!manager.is_monitoring());
    }

    #[test]
    #[cfg(feature = "devtools")]
    fn test_devtools_performance_metrics() {
        // RED: Test performance metrics collection
        let config = DevToolsConfig::default();
        let mut manager = DevToolsManager::new(config);
        
        // Simulate multiple queries with different performance characteristics
        for i in 0..10 {
            let key = QueryKey::new(&["perf", "test", &i.to_string()]);
            let start_time = std::time::Instant::now();
            
            manager.record_query_start(&key);
            
            // Simulate different response times
            std::thread::sleep(Duration::from_millis(i * 10));
            manager.record_query_success(&key, start_time.elapsed());
        }
        
        // Check performance statistics
        let stats = manager.get_performance_stats();
        assert!(stats.total_queries >= 10);
        assert!(stats.average_response_time > Duration::ZERO);
        assert!(stats.max_response_time > Duration::ZERO);
        assert!(stats.min_response_time > Duration::ZERO);
    }

    #[test]
    #[cfg(feature = "devtools")]
    fn test_devtools_error_tracking() {
        // RED: Test error tracking
        let config = DevToolsConfig::default();
        let mut manager = DevToolsManager::new(config);
        
        let key = QueryKey::new(&["test", "error"]);
        let error = QueryError::NetworkError("Connection failed".to_string());
        
        manager.record_query_error(&key, &error);
        
        let events = manager.get_recent_events(10);
        let error_events: Vec<_> = events.iter()
            .filter(|event| matches!(event, DevToolsEvent::QueryError { key: _, error: _, timestamp: _ }))
            .collect();
        assert_eq!(error_events.len(), 1);
        
        // Check error statistics
        let stats = manager.get_error_stats();
        assert!(stats.total_errors > 0);
        assert!(stats.error_rate > 0.0);
    }
}

// Fallback tests for when devtools feature is not enabled
#[cfg(not(feature = "devtools"))]
mod fallback_tests {
    use super::*;

    #[test]
    fn test_devtools_feature_not_enabled() {
        // Test that devtools features gracefully degrade when not enabled
        let client = QueryClient::new();
        let key = QueryKey::new(&["fallback", "devtools"]);
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
