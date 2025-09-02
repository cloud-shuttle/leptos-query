//! Retry logic and error handling for queries

use std::time::Duration;
use std::future::Future;
use serde::{Serialize, Deserialize};

/// Error types that can occur during query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryError {
    /// Network or HTTP errors
    NetworkError(String),
    /// Serialization errors
    SerializationError(String),
    /// Deserialization errors
    DeserializationError(String),
    /// Timeout errors
    TimeoutError(String),
    /// Storage errors for persistence
    StorageError(String),
    /// Generic error with message
    GenericError(String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            QueryError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            QueryError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            QueryError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            QueryError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            QueryError::GenericError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for QueryError {}

/// Configuration for retry behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
    /// Whether to retry on specific error types
    pub retry_on_network_errors: bool,
    pub retry_on_timeout_errors: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            exponential_backoff: true,
            retry_on_network_errors: true,
            retry_on_timeout_errors: true,
        }
    }
}

impl RetryConfig {
    /// Create a retry config with custom settings
    pub fn new(max_retries: usize, base_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay: Duration::from_secs(30),
            exponential_backoff: true,
            retry_on_network_errors: true,
            retry_on_timeout_errors: true,
        }
    }
    
    /// Disable exponential backoff
    pub fn with_fixed_delay(mut self) -> Self {
        self.exponential_backoff = false;
        self
    }
    
    /// Set maximum delay
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }
    
    /// Disable retry on network errors
    pub fn no_network_retry(mut self) -> Self {
        self.retry_on_network_errors = false;
        self
    }
    
    /// Disable retry on timeout errors
    pub fn no_timeout_retry(mut self) -> Self {
        self.retry_on_timeout_errors = false;
        self
    }
}

/// Execute a future with retry logic
pub async fn execute_with_retry<F, Fut, T>(
    query_fn: F,
    config: &RetryConfig,
) -> Result<T, QueryError>
where
    F: Fn() -> Fut + Clone,
    Fut: Future<Output = Result<T, QueryError>>,
{
    let mut last_error = None;
    
    for attempt in 0..=config.max_retries {
        match query_fn().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(error.clone());
                
                // Check if we should retry this error
                if !should_retry_error(&error, config) {
                    return Err(error);
                }
                
                // Don't retry on the last attempt
                if attempt == config.max_retries {
                    break;
                }
                
                // Calculate delay
                let delay = calculate_delay(attempt, config);
                
                // Wait before retrying
                sleep(delay).await;
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| QueryError::GenericError("Unknown error".to_string())))
}

/// Check if an error should be retried
pub fn should_retry_error(error: &QueryError, config: &RetryConfig) -> bool {
    match error {
        QueryError::NetworkError(_) => config.retry_on_network_errors,
        QueryError::TimeoutError(_) => config.retry_on_timeout_errors,
        QueryError::SerializationError(_) | QueryError::DeserializationError(_) => false,
        QueryError::GenericError(_) => true,
        QueryError::StorageError(_) => false, // Storage errors shouldn't be retried
    }
}

/// Calculate delay for retry attempt
fn calculate_delay(attempt: usize, config: &RetryConfig) -> Duration {
    if config.exponential_backoff {
        let delay_ms = config.base_delay.as_millis() as u64 * (2_u64.pow(attempt as u32));
        let delay = Duration::from_millis(delay_ms);
        delay.min(config.max_delay)
    } else {
        config.base_delay
    }
}

/// Sleep function that works in both native and WASM environments
async fn sleep(duration: Duration) {
    #[cfg(target_arch = "wasm32")]
    {
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
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::time::sleep(duration).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new(5, Duration::from_secs(2))
            .with_max_delay(Duration::from_secs(60))
            .with_fixed_delay()
            .no_network_retry();
        
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay, Duration::from_secs(2));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert!(!config.exponential_backoff);
        assert!(!config.retry_on_network_errors);
    }
    
    #[test]
    fn test_should_retry_error() {
        let config = RetryConfig::default();
        
        assert!(should_retry_error(&QueryError::NetworkError("test".to_string()), &config));
        assert!(should_retry_error(&QueryError::TimeoutError("test".to_string()), &config));
        assert!(!should_retry_error(&QueryError::SerializationError("test".to_string()), &config));
    }
    
    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig::new(3, Duration::from_millis(100));
        
        // Exponential backoff
        assert_eq!(calculate_delay(0, &config), Duration::from_millis(100));
        assert_eq!(calculate_delay(1, &config), Duration::from_millis(200));
        assert_eq!(calculate_delay(2, &config), Duration::from_millis(400));
        
        // Fixed delay
        let fixed_config = config.with_fixed_delay();
        assert_eq!(calculate_delay(0, &fixed_config), Duration::from_millis(100));
        assert_eq!(calculate_delay(1, &fixed_config), Duration::from_millis(100));
        assert_eq!(calculate_delay(2, &fixed_config), Duration::from_millis(100));
    }
}