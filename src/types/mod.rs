//! Core types and data structures for the query system

use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Query status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryStatus {
    Idle,
    Loading,
    Success,
    Error,
}

/// Query key for identifying queries in the cache
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryKey {
    pub segments: Vec<String>,
}

impl QueryKey {
    /// Create a new query key from segments
    pub fn new(segments: impl IntoIterator<Item = impl ToString>) -> Self {
        Self {
            segments: segments.into_iter().map(|s| s.to_string()).collect(),
        }
    }
    
    /// Create a key with automatic serialization
    pub fn from_parts<T: Serialize>(parts: &[T]) -> Result<Self, serde_json::Error> {
        let segments = parts
            .iter()
            .map(|part| serde_json::to_string(part))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { segments })
    }
    
    /// Pattern matching for cache invalidation
    pub fn matches_pattern(&self, pattern: &QueryKeyPattern) -> bool {
        match pattern {
            QueryKeyPattern::Exact(key) => self == key,
            QueryKeyPattern::Prefix(prefix) => {
                self.segments.starts_with(&prefix.segments)
            }
            QueryKeyPattern::Contains(segment) => {
                self.segments.contains(segment)
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

/// Pattern for matching query keys during invalidation
#[derive(Clone)]
pub enum QueryKeyPattern {
    Exact(QueryKey),
    Prefix(QueryKey),
    Contains(String),
}

/// Metadata about a query
#[derive(Debug, Clone)]
pub struct QueryMeta {
    pub status: QueryStatus,
    pub updated_at: Instant,
    pub stale_time: Duration,
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
