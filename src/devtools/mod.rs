use crate::client::{QueryClient, CacheEntry, CacheStats};
use crate::types::QueryKey;
use crate::persistence::PersistenceManager;
use crate::optimistic::{OptimisticManager, OptimisticStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use uuid::Uuid;

/// DevTools configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DevToolsConfig {
    /// Whether DevTools are enabled
    pub enabled: bool,
    /// Port for the DevTools server (if applicable)
    pub port: Option<u16>,
    /// Maximum number of events to keep in history
    pub max_history: usize,
    /// Whether to capture performance metrics
    pub capture_metrics: bool,
    /// Whether to capture network requests
    pub capture_network: bool,
    /// Whether to capture cache operations
    pub capture_cache: bool,
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: Some(3001),
            max_history: 1000,
            capture_metrics: true,
            capture_network: true,
            capture_cache: true,
        }
    }
}

/// Performance metrics for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    /// Query key
    pub key: QueryKey,
    /// Total execution time
    #[serde(with = "duration_serde")]
    pub total_time: Duration,
    /// Number of executions
    pub execution_count: usize,
    /// Average execution time
    #[serde(with = "duration_serde")]
    pub avg_time: Duration,
    /// Last execution time
    #[serde(with = "option_instant_serde")]
    pub last_execution: Option<Instant>,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Error count
    pub error_count: usize,
    /// Success count
    pub success_count: usize,
    /// Total requests
    pub total_requests: usize,
    /// Average response time
    #[serde(with = "duration_serde")]
    pub average_response_time: Duration,
}

impl QueryMetrics {
    /// Create new metrics for a query
    pub fn new(key: QueryKey) -> Self {
        Self {
            key,
            total_time: Duration::ZERO,
            execution_count: 0,
            avg_time: Duration::ZERO,
            last_execution: None,
            cache_hit_rate: 0.0,
            error_count: 0,
            success_count: 0,
            total_requests: 0,
            average_response_time: Duration::ZERO,
        }
    }

    /// Record an execution
    pub fn record_execution(&mut self, duration: Duration, success: bool) {
        self.total_time += duration;
        self.execution_count += 1;
        self.avg_time = self.total_time / self.execution_count as u32;
        self.last_execution = Some(Instant::now());
        self.total_requests += 1;
        self.average_response_time = self.avg_time;
        
        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }
    }

    /// Update cache hit rate
    pub fn update_cache_hit_rate(&mut self, hits: usize, total: usize) {
        if total > 0 {
            self.cache_hit_rate = hits as f64 / total as f64;
        }
    }
}

/// Network request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Unique request ID
    pub id: String,
    /// Query key
    pub key: QueryKey,
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Request timestamp
    #[serde(with = "instant_serde")]
    pub timestamp: Instant,
    /// Request duration
    #[serde(with = "option_duration_serde")]
    pub duration: Option<Duration>,
    /// Response status
    pub status: Option<u16>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Option<String>,
    /// Request body size
    pub body_size: Option<usize>,
    /// Response body size
    pub response_size: Option<usize>,
}

impl NetworkRequest {
    /// Create a new network request
    pub fn new(key: QueryKey, url: String, method: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            key,
            url,
            method,
            timestamp: Instant::now(),
            duration: None,
            status: None,
            error: None,
            headers: HashMap::new(),
            body: None,
            body_size: None,
            response_size: None,
        }
    }

    /// Mark request as completed
    pub fn complete(&mut self, status: u16, duration: Duration, response_size: Option<usize>) {
        self.status = Some(status);
        self.duration = Some(duration);
        self.response_size = response_size;
    }

    /// Mark request as failed
    pub fn fail(&mut self, error: String, duration: Duration) {
        self.error = Some(error);
        self.duration = Some(duration);
    }
}

/// Cache operation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheOperation {
    /// Cache entry was set
    Set { key: QueryKey, size: usize, #[serde(with = "instant_serde")] timestamp: Instant },
    /// Cache entry was retrieved
    Get { key: QueryKey, hit: bool, #[serde(with = "instant_serde")] timestamp: Instant },
    /// Cache entry was removed
    Remove { key: QueryKey, #[serde(with = "instant_serde")] timestamp: Instant },
    /// Cache was cleared
    Clear { #[serde(with = "instant_serde")] timestamp: Instant },
    /// Cache entry expired
    Expire { key: QueryKey, #[serde(with = "instant_serde")] timestamp: Instant },
}

/// DevTools event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevToolsEvent {
    /// Query execution started
    QueryStart { key: QueryKey, #[serde(with = "instant_serde")] timestamp: Instant },
    /// Query execution completed
    QueryComplete { key: QueryKey, success: bool, #[serde(with = "duration_serde")] duration: Duration, #[serde(with = "instant_serde")] timestamp: Instant },
    /// Cache operation
    CacheOp { operation: CacheOperation },
    /// Network request
    NetworkRequest { request: NetworkRequest },
    /// Optimistic update
    OptimisticUpdate { key: QueryKey, update_id: String, #[serde(with = "instant_serde")] timestamp: Instant },
    /// Optimistic update confirmed
    OptimisticConfirm { key: QueryKey, update_id: String, #[serde(with = "instant_serde")] timestamp: Instant },
    /// Optimistic update rolled back
    OptimisticRollback { key: QueryKey, update_id: String, #[serde(with = "instant_serde")] timestamp: Instant },
    /// Persistence operation
    PersistenceOp { operation: String, key: Option<QueryKey>, #[serde(with = "instant_serde")] timestamp: Instant },
    /// Query error
    QueryError { key: QueryKey, error: String, #[serde(with = "instant_serde")] timestamp: Instant },
    /// Cache operation
    CacheOperation { operation: CacheOperation, #[serde(with = "instant_serde")] timestamp: Instant },
}

/// DevTools manager
pub struct DevToolsManager {
    /// Configuration
    config: DevToolsConfig,
    /// Query performance metrics
    metrics: Arc<RwLock<HashMap<QueryKey, QueryMetrics>>>,
    /// Network request history
    network_history: Arc<RwLock<Vec<NetworkRequest>>>,
    /// Cache operation history
    cache_history: Arc<RwLock<Vec<CacheOperation>>>,
    /// Event history
    event_history: Arc<RwLock<Vec<DevToolsEvent>>>,
    /// Active queries
    active_queries: Arc<RwLock<HashMap<QueryKey, Instant>>>,
}

impl DevToolsManager {
    /// Create a new DevTools manager
    pub fn new(config: DevToolsConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            network_history: Arc::new(RwLock::new(Vec::new())),
            cache_history: Arc::new(RwLock::new(Vec::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            active_queries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if DevTools is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Start the DevTools server
    pub fn start_server(&mut self, address: String) -> Result<DevToolsServer, String> {
        if !self.config.enabled {
            return Err("DevTools is not enabled".to_string());
        }

        // Parse address (e.g., "localhost:3001")
        let parts: Vec<&str> = address.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid address format. Expected 'host:port'".to_string());
        }

        let host = parts[0].to_string();
        let port: u16 = parts[1].parse().map_err(|_| "Invalid port number".to_string())?;

        let manager = Arc::new(DevToolsManager::new(self.config.clone()));
        let config = DevToolsConfig::default();
        Ok(DevToolsServer::new(manager, config))
    }

    /// Record a query execution start
    pub fn record_query_start(&self, key: &QueryKey) {
        if !self.config.capture_metrics {
            return;
        }

        let mut active = self.active_queries.write();
        active.insert(key.clone(), Instant::now());

        let event = DevToolsEvent::QueryStart {
            key: key.clone(),
            timestamp: Instant::now(),
        };
        self.record_event(event);
    }

    /// Record a query execution completion
    pub fn record_query_complete(&self, key: &QueryKey, success: bool, duration: Duration) {
        if !self.config.capture_metrics {
            return;
        }

        // Remove from active queries
        let mut active = self.active_queries.write();
        active.remove(key);

        // Update metrics
        let mut metrics = self.metrics.write();
        let query_metrics = metrics.entry(key.clone()).or_insert_with(|| QueryMetrics::new(key.clone()));
        query_metrics.record_execution(duration, success);

        let event = DevToolsEvent::QueryComplete {
            key: key.clone(),
            success,
            duration,
            timestamp: Instant::now(),
        };
        self.record_event(event);
    }

    /// Record a successful query execution (convenience method)
    pub fn record_query_success(&self, key: &QueryKey, duration: Duration) {
        self.record_query_complete(key, true, duration);
    }

    /// Record a query error
    pub fn record_query_error(&self, key: &QueryKey, error: &crate::retry::QueryError) {
        if !self.config.capture_metrics {
            return;
        }

        // Remove from active queries
        let mut active = self.active_queries.write();
        active.remove(key);

        let event = DevToolsEvent::QueryError {
            key: key.clone(),
            error: error.to_string(),
            timestamp: Instant::now(),
        };
        self.record_event(event);
    }

    /// Record a network request
    pub fn record_network_request(&self, _key: &QueryKey, request: NetworkRequest) {
        if !self.config.capture_network {
            return;
        }

        let mut history = self.network_history.write();
        history.push(request.clone());

        // Keep only the last N requests
        if history.len() > self.config.max_history {
            history.remove(0);
        }

        let event = DevToolsEvent::NetworkRequest { request };
        self.record_event(event);
    }

    /// Record a cache operation
    pub fn record_cache_operation(&self, operation: CacheOperation, _key: &QueryKey, _data: Option<&impl Serialize>) {
        if !self.config.capture_cache {
            return;
        }

        let mut history = self.cache_history.write();
        history.push(operation.clone());

        // Keep only the last N operations
        if history.len() > self.config.max_history {
            history.remove(0);
        }

        let event = DevToolsEvent::CacheOperation { operation, timestamp: std::time::Instant::now() };
        self.record_event(event);
    }

    /// Record an optimistic update
    pub fn record_optimistic_update(&self, key: &QueryKey, update_id: &str) {
        let event = DevToolsEvent::OptimisticUpdate {
            key: key.clone(),
            update_id: update_id.to_string(),
            timestamp: Instant::now(),
        };
        self.record_event(event);
    }

    /// Record an optimistic update confirmation
    pub fn record_optimistic_confirm(&self, key: &QueryKey, update_id: &str) {
        let event = DevToolsEvent::OptimisticConfirm {
            key: key.clone(),
            update_id: update_id.to_string(),
            timestamp: Instant::now(),
        };
        self.record_event(event);
    }

    /// Record an optimistic update rollback
    pub fn record_optimistic_rollback(&self, key: &QueryKey, update_id: &str) {
        let event = DevToolsEvent::OptimisticRollback {
            key: key.clone(),
            update_id: update_id.to_string(),
            timestamp: Instant::now(),
        };
        self.record_event(event);
    }

    /// Record a persistence operation
    pub fn record_persistence_operation(&self, operation: &str, key: Option<&QueryKey>) {
        let event = DevToolsEvent::PersistenceOp {
            operation: operation.to_string(),
            key: key.cloned(),
            timestamp: Instant::now(),
        };
        self.record_event(event);
    }

    /// Record a generic event
    fn record_event(&self, event: DevToolsEvent) {
        let mut history = self.event_history.write();
        history.push(event);

        // Keep only the last N events
        if history.len() > self.config.max_history {
            history.remove(0);
        }
    }

    /// Get query metrics
    pub fn get_query_metrics(&self, _key: &QueryKey) -> Option<QueryMetrics> {
        // For now, return the first metric if any exist
        let metrics = self.metrics.read();
        metrics.values().next().cloned()
    }

    /// Get network request history
    pub fn get_network_history(&self) -> Vec<NetworkRequest> {
        let history = self.network_history.read();
        history.clone()
    }

    /// Get cache operation history
    pub fn get_cache_history(&self) -> Vec<CacheOperation> {
        let history = self.cache_history.read();
        history.clone()
    }

    /// Get event history
    pub fn get_event_history(&self) -> Vec<DevToolsEvent> {
        let history = self.event_history.read();
        history.clone()
    }

    /// Get active queries
    pub fn get_active_queries(&self) -> Vec<ActiveQuery> {
        let active = self.active_queries.read();
        let now = Instant::now();
        active
            .iter()
            .map(|(key, start_time)| ActiveQuery {
                key: key.clone(),
                duration: now.duration_since(*start_time),
            })
            .collect()
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self, client: &QueryClient) -> CacheStats {
        client.cache_stats()
    }

    /// Get cache entries
    pub fn get_cache_entries(&self, client: &QueryClient) -> Vec<(QueryKey, CacheEntry)> {
        client.get_cache_entries()
    }

    /// Get optimistic update statistics
    pub fn get_optimistic_stats(&self, manager: &OptimisticManager<String>) -> OptimisticStats {
        manager.get_stats()
    }

    /// Get persistence statistics
    pub fn get_persistence_stats(&self, manager: &PersistenceManager) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("offline_queue_size".to_string(), manager.get_offline_queue().len());
        stats.insert("cache_persisted".to_string(), if manager.is_cache_persisted() { 1 } else { 0 });
        stats
    }

    /// Clear all history
    pub fn clear_history(&self) {
        let mut metrics = self.metrics.write();
        let mut network = self.network_history.write();
        let mut cache = self.cache_history.write();
        let mut events = self.event_history.write();
        let mut active = self.active_queries.write();

        metrics.clear();
        network.clear();
        cache.clear();
        events.clear();
        active.clear();
    }

    /// Export data for external tools
    pub fn export_data(&self) -> DevToolsExport {
        DevToolsExport {
            query_metrics: self.metrics.read().values().cloned().collect(),
            network_requests: self.get_network_history(),
            cache_operations: self.get_cache_history(),
            event_history: self.get_event_history(),
            active_queries: self.get_active_queries(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        }
    }

    /// Get recent events (last N events)
    pub fn get_recent_events(&self, count: usize) -> Vec<DevToolsEvent> {
        let history = self.event_history.read();
        let start = if history.len() > count {
            history.len() - count
        } else {
            0
        };
        history[start..].to_vec()
    }

    /// Start monitoring (placeholder for real-time monitoring)
    pub fn start_monitoring(&mut self) {
        // In a real implementation, this would start a background task
        // For now, we just ensure monitoring is enabled
        self.config.enabled = true;
    }

    /// Stop monitoring
    pub fn stop_monitoring(&mut self) {
        // In a real implementation, this would stop the background task
        // For now, we just disable monitoring
        self.config.enabled = false;
    }

    /// Check if monitoring is active
    pub fn is_monitoring(&self) -> bool {
        self.config.enabled
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let metrics = self.metrics.read();
        let mut total_queries = 0;
        let mut total_time = Duration::ZERO;
        let mut max_time = Duration::ZERO;
        let mut min_time = Duration::from_secs(u64::MAX);

        for query_metrics in metrics.values() {
            total_queries += query_metrics.execution_count;
            total_time += query_metrics.total_time;
            if query_metrics.total_time > max_time {
                max_time = query_metrics.total_time;
            }
            if query_metrics.total_time < min_time {
                min_time = query_metrics.total_time;
            }
        }

        let average_time = if total_queries > 0 {
            total_time / total_queries as u32
        } else {
            Duration::ZERO
        };

        PerformanceStats {
            total_queries,
            average_response_time: average_time,
            max_response_time: max_time,
            min_response_time: if min_time == Duration::from_secs(u64::MAX) { Duration::ZERO } else { min_time },
        }
    }

    /// Get error statistics
    pub fn get_error_stats(&self) -> ErrorStats {
        let events = self.event_history.read();
        let mut total_errors = 0;
        let mut total_events = events.len();

        for event in events.iter() {
            if matches!(event, DevToolsEvent::QueryError { .. }) {
                total_errors += 1;
            }
        }

        let error_rate = if total_events > 0 {
            total_errors as f64 / total_events as f64
        } else {
            0.0
        };

        ErrorStats {
            total_errors,
            error_rate,
        }
    }

    /// Import data from external tools
    pub fn import_data(&self, data: DevToolsExport) {
        let mut metrics = self.metrics.write();
        let mut network = self.network_history.write();
        let mut cache = self.cache_history.write();
        let mut events = self.event_history.write();

        // Import metrics
        for metric in data.query_metrics {
            metrics.insert(metric.key.clone(), metric);
        }

        // Import network history
        network.extend(data.network_requests);

        // Import cache history
        cache.extend(data.cache_operations);

        // Import event history
        events.extend(data.event_history);
    }
}

/// Active query with duration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveQuery {
    /// Query key
    pub key: QueryKey,
    /// Duration since start
    #[serde(with = "duration_serde")]
    pub duration: Duration,
}

/// DevTools data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsExport {
    /// Query metrics
    pub query_metrics: Vec<QueryMetrics>,
    /// Network request history
    pub network_requests: Vec<NetworkRequest>,
    /// Cache operation history
    pub cache_operations: Vec<CacheOperation>,
    /// Event history
    pub event_history: Vec<DevToolsEvent>,
    /// Active queries
    pub active_queries: Vec<ActiveQuery>,
    /// Export timestamp
    pub timestamp: u64,
}

/// DevTools server (placeholder for future implementation)
pub struct DevToolsServer {
    /// Manager instance
    #[allow(dead_code)]
    manager: Arc<DevToolsManager>,
    /// Server configuration
    #[allow(dead_code)]
    config: DevToolsConfig,
}

impl DevToolsServer {
    /// Create a new DevTools server
    pub fn new(manager: Arc<DevToolsManager>, config: DevToolsConfig) -> Self {
        Self { manager, config }
    }

    /// Start the DevTools server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // This would implement an actual HTTP server
        // For now, just return Ok
        Ok(())
    }

    /// Stop the DevTools server
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn port(&self) -> u16 {
        3001 // Default port
    }

    pub fn host(&self) -> &str {
        "localhost" // Default host
    }
}

/// Performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_queries: usize,
    pub average_response_time: Duration,
    pub max_response_time: Duration,
    pub min_response_time: Duration,
}

/// Error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total_errors: usize,
    pub error_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::QueryKey;

    #[test]
    fn test_devtools_manager_creation() {
        let config = DevToolsConfig::default();
        let manager = DevToolsManager::new(config);
        
        let key = QueryKey::new(&["test"]);
        assert!(manager.get_query_metrics(&key).is_none());
        assert_eq!(manager.get_network_history().len(), 0);
        assert_eq!(manager.get_cache_history().len(), 0);
    }

    #[test]
    fn test_query_metrics_recording() {
        let config = DevToolsConfig::default();
        let manager = DevToolsManager::new(config);
        
        let key = QueryKey::from("test");
        manager.record_query_start(&key);
        manager.record_query_complete(&key, true, Duration::from_millis(100));
        
        let metrics = manager.get_query_metrics(&key);
        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.execution_count, 1);
        assert_eq!(metrics.success_count, 1);
    }

    #[test]
    fn test_network_request_recording() {
        let config = DevToolsConfig::default();
        let manager = DevToolsManager::new(config);
        
        let key = QueryKey::from("test");
        let request = NetworkRequest::new(key, "https://api.example.com/data".to_string(), "GET".to_string());
        let key = QueryKey::new(&["test"]);
        manager.record_network_request(&key, request);
        
        let history = manager.get_network_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].method, "GET");
    }

    #[test]
    fn test_cache_operation_recording() {
        let config = DevToolsConfig::default();
        let manager = DevToolsManager::new(config);
        
        let key = QueryKey::from("test");
        let operation = CacheOperation::Set {
            key: key.clone(),
            size: 1024,
            timestamp: Instant::now(),
        };
        let key = QueryKey::new(&["test"]);
        manager.record_cache_operation(operation, &key, None::<&String>);
        
        let history = manager.get_cache_history();
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_history_limits() {
        let mut config = DevToolsConfig::default();
        config.max_history = 5;
        let manager = DevToolsManager::new(config);
        
        // Add more events than the limit
        for i in 0..10 {
            let key = QueryKey::from(format!("test{}", i));
            manager.record_query_start(&key);
        }
        
        // Should only keep the last 5 events
        let events = manager.get_event_history();
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_export_import() {
        let config = DevToolsConfig::default();
        let manager = DevToolsManager::new(config);
        
        let key = QueryKey::from("test");
        manager.record_query_start(&key);
        manager.record_query_complete(&key, true, Duration::from_millis(100));
        
        let export = manager.export_data();
        assert_eq!(export.query_metrics.len(), 1);
        
        // Clear and reimport
        manager.clear_history();
        let key = QueryKey::new(&["test"]);
        assert!(manager.get_query_metrics(&key).is_none());
        
        manager.import_data(export);
        let key = QueryKey::new(&["test"]);
        assert!(manager.get_query_metrics(&key).is_some());
    }
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

/// Serialization helpers for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Serialization helpers for Option<Duration>
mod option_duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => d.as_secs().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = Option::<u64>::deserialize(deserializer)?;
        Ok(secs.map(Duration::from_secs))
    }
}

/// Serialization helpers for Option<Instant>
mod option_instant_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(instant: &Option<Instant>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match instant {
            Some(inst) => {
                let system_time = SystemTime::now() - inst.elapsed();
                let duration = system_time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
                duration.serialize(serializer)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Instant>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let duration = Option::<Duration>::deserialize(deserializer)?;
        Ok(duration.map(|d| {
            let system_time = UNIX_EPOCH + d;
            let now = SystemTime::now();
            let elapsed = now.duration_since(system_time).unwrap_or(Duration::ZERO);
            Instant::now() - elapsed
        }))
    }
}
