//! Retry Logic and Error Handling
//!
//! Provides configurable retry strategies with exponential backoff,
//! jitter, and custom retry conditions.

use std::time::Duration;
use std::future::Future;
use thiserror::Error;

/// Retry configuration
#[derive(Clone, Debug)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay: RetryDelay,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay: RetryDelay::Exponential {
                initial: Duration::from_millis(1000),
                multiplier: 2.0,
                max: Duration::from_secs(30),
            },
            jitter: true,
        }
    }
}

/// Retry delay strategies
#[derive(Clone, Debug)]
pub enum RetryDelay {
    /// Fixed delay between retries
    Fixed(Duration),
    /// Linear increase: initial + (increment * attempt)
    Linear { initial: Duration, increment: Duration },
    /// Exponential backoff: initial * (multiplier ^ attempt)
    Exponential { initial: Duration, multiplier: f64, max: Duration },
}

impl RetryDelay {
    pub fn calculate(&self, attempt: u32, jitter: bool) -> Duration {
        let base_delay = match self {
            RetryDelay::Fixed(duration) => *duration,
            RetryDelay::Linear { initial, increment } => {
                *initial + (*increment * attempt)
            }
            RetryDelay::Exponential { initial, multiplier, max } => {
                let delay = initial.as_millis() as f64 * multiplier.powi(attempt as i32);
                Duration::from_millis(delay.min(max.as_millis() as f64) as u64)
            }
        };

        if jitter {
            // Add jitter: ±25% of base delay
            let jitter_factor = 0.5 + (js_sys::Math::random() * 0.5); // 0.5 to 1.0
            Duration::from_millis((base_delay.as_millis() as f64 * jitter_factor) as u64)
        } else {
            base_delay
        }
    }
}

/// Execute operation with retry logic
pub async fn execute_with_retry<F, Fut, T>(
    mut operation: F,
    config: &RetryConfig,
) -> Result<T, QueryError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, QueryError>>,
{
    let mut attempt = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                attempt += 1;
                
                // Check if we should retry
                if attempt >= config.max_attempts || !error.is_retryable() {
                    return Err(error);
                }
                
                // Calculate delay and wait
                let delay = config.delay.calculate(attempt - 1, config.jitter);
                sleep(delay).await;
            }
        }
    }
}

/// Query error types with detailed context
#[derive(Clone, Debug, Error)]
pub enum QueryError {
    #[error("Network error: {message}")]
    Network { message: String },
    
    #[error("Request timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String, body: Option<String> },
    
    #[error("Serialization failed: {0}")]
    Serialization(String),
    
    #[error("Deserialization failed: {0}")]
    Deserialization(String),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Request was cancelled")]
    Cancelled,
    
    #[error("Rate limit exceeded, retry after {retry_after_ms}ms")]
    RateLimit { retry_after_ms: u64 },
    
    #[error("Custom error: {message}")]
    Custom { message: String, code: Option<String> },
    
    #[error("Cache error: {message}")]
    Cache { message: String },
}

impl QueryError {
    /// Create a network error with context
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network { 
            message: message.into()
        }
    }
    
    /// Create a network error with source (deprecated, use network instead)
    pub fn network_with_source(message: impl Into<String>, _source: impl Into<String>) -> Self {
        Self::Network { 
            message: message.into()
        }
    }
    
    /// Create an HTTP error
    pub fn http(status: u16, message: impl Into<String>) -> Self {
        Self::Http { 
            status, 
            message: message.into(), 
            body: None 
        }
    }
    
    /// Create an HTTP error with response body
    pub fn http_with_body(
        status: u16, 
        message: impl Into<String>, 
        body: impl Into<String>
    ) -> Self {
        Self::Http { 
            status, 
            message: message.into(), 
            body: Some(body.into()) 
        }
    }
    
    /// Create a timeout error
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout { timeout_ms }
    }
    
    /// Create a custom error
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom { 
            message: message.into(), 
            code: None 
        }
    }
    
    /// Create a custom error with error code
    pub fn custom_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::Custom { 
            message: message.into(), 
            code: Some(code.into()) 
        }
    }
    
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            QueryError::Network { .. } => true,
            QueryError::Timeout { .. } => true,
            QueryError::Http { status, .. } => *status >= 500,
            QueryError::RateLimit { .. } => true,
            QueryError::Cancelled => false,
            QueryError::Serialization(_) => false,
            QueryError::Deserialization(_) => false,
            QueryError::TypeMismatch { .. } => false,
            QueryError::Custom { .. } => false,
            QueryError::Cache { .. } => false,
        }
    }
    
    /// Get suggested retry delay for this error
    pub fn suggested_retry_delay(&self) -> Option<Duration> {
        match self {
            QueryError::RateLimit { retry_after_ms } => {
                Some(Duration::from_millis(*retry_after_ms))
            }
            QueryError::Http { status, .. } if *status == 429 => {
                Some(Duration::from_secs(60)) // Default rate limit backoff
            }
            QueryError::Network { .. } => {
                Some(Duration::from_millis(1000))
            }
            QueryError::Timeout { .. } => {
                Some(Duration::from_millis(2000))
            }
            _ => None,
        }
    }
    
    /// Get error severity for logging and metrics
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            QueryError::Network { .. } => ErrorSeverity::Warning,
            QueryError::Timeout { .. } => ErrorSeverity::Warning,
            QueryError::Http { status, .. } => {
                match *status {
                    400..=499 => ErrorSeverity::Info, // Client errors
                    500..=599 => ErrorSeverity::Error, // Server errors
                    _ => ErrorSeverity::Warning,
                }
            }
            QueryError::RateLimit { .. } => ErrorSeverity::Warning,
            QueryError::Cancelled => ErrorSeverity::Info,
            QueryError::Serialization(_) => ErrorSeverity::Error,
            QueryError::Deserialization(_) => ErrorSeverity::Error,
            QueryError::TypeMismatch { .. } => ErrorSeverity::Error,
            QueryError::Custom { .. } => ErrorSeverity::Warning,
            QueryError::Cache { .. } => ErrorSeverity::Warning,
        }
    }
}

/// Error severity levels for monitoring and logging
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

// Utility sleep function
async fn sleep(duration: Duration) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_delay_calculation() {
        let exponential = RetryDelay::Exponential {
            initial: Duration::from_millis(1000),
            multiplier: 2.0,
            max: Duration::from_secs(30),
        };
        
        assert_eq!(exponential.calculate(0, false), Duration::from_millis(1000));
        assert_eq!(exponential.calculate(1, false), Duration::from_millis(2000));
        assert_eq!(exponential.calculate(2, false), Duration::from_millis(4000));
    }
    
    #[test]
    fn test_error_retryability() {
        assert!(QueryError::network("connection failed").is_retryable());
        assert!(QueryError::timeout(5000).is_retryable());
        assert!(QueryError::http(500, "server error").is_retryable());
        assert!(!QueryError::http(400, "bad request").is_retryable());
        assert!(!QueryError::custom("validation failed").is_retryable());
    }
}