//! Core types and data structures for the query system

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use std::fmt;

/// Query status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryStatus {
    /// Query is idle (not running)
    Idle,
    /// Query is currently loading
    Loading,
    /// Query completed successfully
    Success,
    /// Query failed with an error
    Error,
}

impl Default for QueryStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// Query key for identifying queries in the cache
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryKey {
    pub segments: Vec<String>,
}

impl fmt::Display for QueryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.segments.join(":"))
    }
}

impl QueryKey {
    /// Create a new query key from segments
    pub fn new(segments: impl IntoIterator<Item = impl ToString>) -> Self {
        Self {
            segments: segments.into_iter().map(|s| s.to_string()).collect(),
        }
    }
    
    /// Create a query key from a single string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self {
            segments: vec![s.into()],
        }
    }
    
    /// Add a segment to the key
    pub fn with_segment(mut self, segment: impl Into<String>) -> Self {
        self.segments.push(segment.into());
        self
    }
    
    /// Get the segments as a slice
    pub fn segments(&self) -> &[String] {
        &self.segments
    }
    
    /// Check if the key is empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
    
    /// Get the number of segments
    pub fn len(&self) -> usize {
        self.segments.len()
    }
    
    /// Check if this key matches a pattern
    pub fn matches_pattern(&self, pattern: &QueryKeyPattern) -> bool {
        match pattern {
            QueryKeyPattern::Exact(key) => self == key,
            QueryKeyPattern::Prefix(prefix) => {
                self.segments.len() >= prefix.segments.len() &&
                self.segments[..prefix.segments.len()] == prefix.segments
            }
            QueryKeyPattern::Contains(substring) => {
                self.segments.iter().any(|segment| segment.contains(substring))
            }
        }
    }
}

/// Convert string slices to QueryKey
impl<T: ToString + std::fmt::Display> From<&[T]> for QueryKey {
    fn from(segments: &[T]) -> Self {
        Self::new(segments)
    }
}

/// Convert single string to QueryKey
impl From<String> for QueryKey {
    fn from(segment: String) -> Self {
        Self::new([segment])
    }
}

/// Convert &str to QueryKey
impl From<&str> for QueryKey {
    fn from(segment: &str) -> Self {
        Self::new([segment.to_string()])
    }
}

/// Convert tuple to QueryKey  
impl<T: ToString + std::fmt::Display> From<(T,)> for QueryKey {
    fn from((a,): (T,)) -> Self {
        Self::new([a])
    }
}

/// Patterns for matching query keys
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryKeyPattern {
    /// Exact match
    Exact(QueryKey),
    /// Prefix match (key starts with this pattern)
    Prefix(QueryKey),
    /// Contains substring match
    Contains(String),
}

/// Observer ID for tracking query observers
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryObserverId {
    pub id: u64,
}

impl QueryObserverId {
    /// Create a new observer ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self {
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl Default for QueryObserverId {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMeta {
    pub status: QueryStatus,
    #[serde(with = "instant_serde")]
    pub updated_at: Instant,
    #[serde(with = "duration_serde")]
    pub stale_time: Duration,
    #[serde(with = "duration_serde")]
    pub cache_time: Duration,
}

impl QueryMeta {
    /// Check if the query is stale
    pub fn is_stale(&self) -> bool {
        let age = Instant::now().duration_since(self.updated_at);
        age > self.stale_time
    }
    
    /// Check if the query has expired
    pub fn is_expired(&self) -> bool {
        let age = Instant::now().duration_since(self.updated_at);
        age > self.cache_time
    }
}

impl Default for QueryMeta {
    fn default() -> Self {
        Self {
            status: QueryStatus::Idle,
            updated_at: Instant::now(),
            stale_time: Duration::from_secs(0),
            cache_time: Duration::from_secs(5 * 60), // 5 minutes
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_key_creation() {
        let key = QueryKey::new(["users", "123"]);
        assert_eq!(key.segments, vec!["users", "123"]);
        
        let key2 = QueryKey::from("single");
        assert_eq!(key2.segments, vec!["single"]);
    }
    
    #[test]
    fn test_query_key_pattern_matching() {
        let key = QueryKey::new(["users", "123", "profile"]);
        
        // Exact match
        let exact_pattern = QueryKeyPattern::Exact(QueryKey::new(["users", "123", "profile"]));
        assert!(key.matches_pattern(&exact_pattern));
        
        // Prefix match
        let prefix_pattern = QueryKeyPattern::Prefix(QueryKey::new(["users"]));
        assert!(key.matches_pattern(&prefix_pattern));
        
        // Contains match
        let contains_pattern = QueryKeyPattern::Contains("123".to_string());
        assert!(key.matches_pattern(&contains_pattern));
    }
    
    #[test]
    fn test_query_meta_stale_check() {
        let mut meta = QueryMeta::default();
        meta.stale_time = Duration::from_secs(60);
        
        // Should not be stale immediately
        assert!(!meta.is_stale());
        
        // Should be stale after waiting
        meta.updated_at = Instant::now() - Duration::from_secs(120);
        assert!(meta.is_stale());
    }
}
