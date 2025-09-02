# DevTools Integration

The DevTools integration provides comprehensive debugging, monitoring, and inspection capabilities for the `leptos-query` library. It allows developers to monitor query performance, track network requests, inspect cache operations, and analyze optimistic updates in real-time.

## Features

### 🔍 **Query Performance Monitoring**
- Track execution times and success rates
- Monitor cache hit rates
- Analyze query patterns and performance trends
- Identify slow queries and bottlenecks

### 🌐 **Network Request Tracking**
- Monitor all HTTP requests and responses
- Track request durations and status codes
- Analyze request headers and body sizes
- Identify failed requests and errors

### 💾 **Cache Operation Inspection**
- Monitor cache hits, misses, and evictions
- Track cache entry sizes and expiration
- Analyze cache performance and efficiency
- Debug cache-related issues

### ⚡ **Optimistic Update Tracking**
- Monitor optimistic update applications
- Track confirmations and rollbacks
- Analyze optimistic update patterns
- Debug optimistic update issues

### 📊 **Real-time Statistics**
- Live query counts and status
- Cache statistics and memory usage
- Performance metrics and trends
- Export capabilities for external analysis

## Quick Start

### 1. Create DevTools Manager

```rust
use leptos_query::devtools::{DevToolsManager, DevToolsConfig};

// Create DevTools with default configuration
let devtools = DevToolsManager::new(DevToolsConfig::default());

// Or customize the configuration
let config = DevToolsConfig {
    enabled: true,
    port: Some(3001),
    max_history: 2000,
    capture_metrics: true,
    capture_network: true,
    capture_cache: true,
};
let devtools = DevToolsManager::new(config);
```

### 2. Provide DevTools Context

```rust
use leptos::*;

#[component]
fn App() -> impl IntoView {
    let devtools = DevToolsManager::new(DevToolsConfig::default());
    
    // Provide DevTools context to the component tree
    provide_context(devtools.clone());
    
    // ... rest of your app
}
```

### 3. Use DevTools in Components

```rust
use leptos::*;
use leptos_query::devtools::DevToolsManager;

#[component]
fn DevToolsPanel() -> impl IntoView {
    let devtools = use_context::<DevToolsManager>().expect("DevTools not found");
    
    // Get various metrics and data
    let metrics = create_memo(move |_| devtools.get_query_metrics());
    let network_history = create_memo(move |_| devtools.get_network_history());
    let cache_stats = create_memo(move |_| devtools.get_cache_stats(&client));
    
    // ... render your DevTools UI
}
```

## Configuration

### DevToolsConfig

```rust
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
```

### Default Configuration

```rust
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
```

## API Reference

### Core Methods

#### Query Performance

```rust
impl DevToolsManager {
    /// Record a query execution start
    pub fn record_query_start(&self, key: &QueryKey);
    
    /// Record a query execution completion
    pub fn record_query_complete(&self, key: &QueryKey, success: bool, duration: Duration);
    
    /// Get query performance metrics
    pub fn get_query_metrics(&self) -> Vec<QueryMetrics>;
    
    /// Get active queries with their durations
    pub fn get_active_queries(&self) -> Vec<ActiveQuery>;
}
```

#### Network Tracking

```rust
impl DevToolsManager {
    /// Record a network request
    pub fn record_network_request(&self, request: NetworkRequest);
    
    /// Get network request history
    pub fn get_network_history(&self) -> Vec<NetworkRequest>;
}
```

#### Cache Monitoring

```rust
impl DevToolsManager {
    /// Record a cache operation
    pub fn record_cache_operation(&self, operation: CacheOperation);
    
    /// Get cache operation history
    pub fn get_cache_history(&self) -> Vec<CacheOperation>;
    
    /// Get cache statistics
    pub fn get_cache_stats(&self, client: &QueryClient) -> CacheStats;
    
    /// Get all cache entries
    pub fn get_cache_entries(&self, client: &QueryClient) -> Vec<(QueryKey, CacheEntry)>;
}
```

#### Optimistic Updates

```rust
impl DevToolsManager {
    /// Record an optimistic update
    pub fn record_optimistic_update(&self, key: &QueryKey, update_id: &str);
    
    /// Record an optimistic update confirmation
    pub fn record_optimistic_confirm(&self, key: &QueryKey, update_id: &str);
    
    /// Record an optimistic update rollback
    pub fn record_optimistic_rollback(&self, key: &QueryKey, update_id: &str);
}
```

#### Data Management

```rust
impl DevToolsManager {
    /// Clear all history
    pub fn clear_history(&self);
    
    /// Export data for external tools
    pub fn export_data(&self) -> DevToolsExport;
    
    /// Import data from external tools
    pub fn import_data(&self, data: DevToolsExport);
}
```

### Data Structures

#### QueryMetrics

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub key: QueryKey,
    pub total_time: Duration,
    pub execution_count: usize,
    pub avg_time: Duration,
    pub last_execution: Option<Instant>,
    pub cache_hit_rate: f64,
    pub error_count: usize,
    pub success_count: usize,
}
```

#### NetworkRequest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub id: String,
    pub key: QueryKey,
    pub url: String,
    pub method: String,
    pub timestamp: Instant,
    pub duration: Option<Duration>,
    pub status: Option<u16>,
    pub error: Option<String>,
    pub headers: HashMap<String, String>,
    pub body_size: Option<usize>,
    pub response_size: Option<usize>,
}
```

#### CacheOperation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheOperation {
    Set { key: QueryKey, size: usize, timestamp: Instant },
    Get { key: QueryKey, hit: bool, timestamp: Instant },
    Remove { key: QueryKey, timestamp: Instant },
    Clear { timestamp: Instant },
    Expire { key: QueryKey, timestamp: Instant },
}
```

#### DevToolsEvent

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevToolsEvent {
    QueryStart { key: QueryKey, timestamp: Instant },
    QueryComplete { key: QueryKey, success: bool, duration: Duration, timestamp: Instant },
    CacheOp { operation: CacheOperation },
    NetworkRequest { request: NetworkRequest },
    OptimisticUpdate { key: QueryKey, update_id: String, timestamp: Instant },
    OptimisticConfirm { key: QueryKey, update_id: String, timestamp: Instant },
    OptimisticRollback { key: QueryKey, update_id: String, timestamp: Instant },
    PersistenceOp { operation: String, key: Option<QueryKey>, timestamp: Instant },
}
```

## Integration Examples

### Basic Integration

```rust
use leptos::*;
use leptos_query::*;
use leptos_query::devtools::DevToolsManager;

#[component]
fn App() -> impl IntoView {
    let devtools = DevToolsManager::new(DevToolsConfig::default());
    provide_context(devtools);
    
    view! {
        <QueryClientProvider>
            <MainContent />
            <DevToolsPanel />
        </QueryClientProvider>
    }
}
```

### Custom DevTools Panel

```rust
#[component]
fn CustomDevToolsPanel() -> impl IntoView {
    let devtools = use_context::<DevToolsManager>().expect("DevTools not found");
    let client = use_query_client();
    
    let (show_details, set_show_details) = create_signal(false);
    
    let metrics = create_memo(move |_| devtools.get_query_metrics());
    let cache_stats = create_memo(move |_| devtools.get_cache_stats(&client));
    
    view! {
        <div class="custom-devtools">
            <h3>"Custom DevTools"</h3>
            
            <div class="summary">
                <p>"Active Queries: " {move || devtools.get_active_queries().len()}</p>
                <p>"Cache Entries: " {move || cache_stats.get().total_entries}</p>
                <p>"Total Metrics: " {move || metrics.get().len()}</p>
            </div>
            
            <button on:click=move |_| set_show_details.set(!show_details.get())>
                {move || if show_details.get() { "Hide Details" } else { "Show Details" }}
            </button>
            
            {move || if show_details.get() {
                view! {
                    <div class="details">
                        <h4>"Query Metrics"</h4>
                        {move || metrics.get().into_iter().map(|metric| {
                            view! {
                                <div class="metric">
                                    <strong>{metric.key}</strong> " - " 
                                    "Executions: " {metric.execution_count} ", " 
                                    "Avg: " {format!("{:.2}ms", metric.avg_time.as_millis() as f64)}
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }
            } else {
                view! { <div></div> }
            }}
        </div>
    }
}
```

### Performance Monitoring

```rust
#[component]
fn PerformanceMonitor() -> impl IntoView {
    let devtools = use_context::<DevToolsManager>().expect("DevTools not found");
    
    let metrics = create_memo(move |_| devtools.get_query_metrics());
    
    let slow_queries = create_memo(move |_| {
        metrics.get()
            .into_iter()
            .filter(|m| m.avg_time > Duration::from_millis(100))
            .collect::<Vec<_>>()
    });
    
    let error_queries = create_memo(move |_| {
        metrics.get()
            .into_iter()
            .filter(|m| m.error_count > 0)
            .collect::<Vec<_>>()
    });
    
    view! {
        <div class="performance-monitor">
            <h3>"Performance Monitor"</h3>
            
            <div class="alerts">
                {move || if !slow_queries.get().is_empty() {
                    view! {
                        <div class="alert warning">
                            "⚠️ " {slow_queries.get().len()} " slow queries detected"
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }}
                
                {move || if !error_queries.get().is_empty() {
                    view! {
                        <div class="alert error">
                            "❌ " {error_queries.get().len()} " queries with errors"
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }}
            </div>
            
            <div class="stats">
                <h4>"Performance Statistics"</h4>
                {move || {
                    let total_queries: usize = metrics.get().iter().map(|m| m.execution_count).sum();
                    let total_time: Duration = metrics.get().iter().map(|m| m.total_time).sum();
                    let avg_time = if total_queries > 0 {
                        total_time / total_queries as u32
                    } else {
                        Duration::ZERO
                    };
                    
                    view! {
                        <div class="stat">
                            <strong>"Total Queries: " {total_queries}</strong><br/>
                            <strong>"Total Time: " {format!("{:.2}s", total_time.as_secs_f64())}</strong><br/>
                            <strong>"Average Time: " {format!("{:.2}ms", avg_time.as_millis() as f64)}</strong>
                        </div>
                    }
                }}
            </div>
        </div>
    }
}
```

## Advanced Usage

### Custom Event Recording

```rust
// Record custom events for your application
let devtools = use_context::<DevToolsManager>().expect("DevTools not found");

// Record a custom persistence operation
devtools.record_persistence_operation("custom_backup", Some(&query_key));

// Record custom network requests
let request = NetworkRequest::new(
    query_key.clone(),
    "https://api.example.com/custom".to_string(),
    "POST".to_string()
);
devtools.record_network_request(request);
```

### Data Export and Import

```rust
// Export DevTools data
let export_data = devtools.export_data();

// Save to file (in a real app)
let json = serde_json::to_string_pretty(&export_data).unwrap();
std::fs::write("devtools-export.json", json).unwrap();

// Import data from external sources
let imported_data = serde_json::from_str::<DevToolsExport>(&json).unwrap();
devtools.import_data(imported_data);
```

### Conditional Monitoring

```rust
// Only enable DevTools in development
#[cfg(debug_assertions)]
let devtools = DevToolsManager::new(DevToolsConfig::default());

#[cfg(not(debug_assertions))]
let devtools = DevToolsManager::new(DevToolsConfig {
    enabled: false,
    ..Default::default()
});
```

## Best Practices

### 1. **Performance Considerations**
- Use `max_history` to limit memory usage
- Disable unused capture features in production
- Consider using conditional compilation for DevTools

### 2. **Integration Patterns**
- Provide DevTools at the top level of your app
- Use context for easy access throughout the component tree
- Create reusable DevTools components

### 3. **Data Management**
- Regularly clear old history data
- Export data for external analysis
- Use DevTools data for performance optimization

### 4. **Error Handling**
- Always handle missing DevTools context gracefully
- Provide fallbacks when DevTools are disabled
- Log errors for debugging

## Troubleshooting

### Common Issues

#### DevTools Context Not Found
```rust
// Error: DevTools not found
let devtools = use_context::<DevToolsManager>().expect("DevTools not found");

// Solution: Ensure DevTools are provided at the top level
provide_context(DevToolsManager::new(DevToolsConfig::default()));
```

#### Performance Impact
```rust
// If DevTools are causing performance issues, reduce capture scope
let config = DevToolsConfig {
    max_history: 100, // Reduce from default 1000
    capture_metrics: true,
    capture_network: false, // Disable network tracking
    capture_cache: false,   // Disable cache tracking
    ..Default::default()
};
```

#### Memory Usage
```rust
// Clear history periodically
let devtools = use_context::<DevToolsManager>().expect("DevTools not found");

// Clear every 1000 events
if devtools.get_event_history().len() > 1000 {
    devtools.clear_history();
}
```

## Future Enhancements

### Planned Features
- **HTTP Server**: Standalone DevTools server for external access
- **Real-time Updates**: WebSocket-based live updates
- **Plugin System**: Extensible DevTools architecture
- **Performance Profiling**: Advanced performance analysis tools
- **Network Simulation**: Test different network conditions

### Custom Extensions
The DevTools system is designed to be extensible. You can:
- Create custom event types
- Implement custom serialization
- Add custom metrics and statistics
- Integrate with external monitoring tools

## Conclusion

The DevTools integration provides powerful debugging and monitoring capabilities for the `leptos-query` library. By following the patterns and best practices outlined in this documentation, you can effectively monitor and optimize your application's data fetching performance.

For more examples and advanced usage patterns, see the `examples/devtools_example.rs` file in the repository.
