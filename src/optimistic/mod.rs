use crate::types::{QueryKey, QueryKeyPattern};
use crate::retry::QueryError;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Optimistic update configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimisticConfig {
    /// Whether optimistic updates are enabled
    pub enabled: bool,
    /// How long to keep optimistic data before reverting
    pub rollback_timeout: Duration,
    /// Whether to show loading states during optimistic updates
    pub show_loading: bool,
    /// Whether to merge optimistic data with existing cache
    pub merge_strategy: MergeStrategy,
}

impl Default for OptimisticConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rollback_timeout: Duration::from_secs(30), // 30 seconds
            show_loading: false,
            merge_strategy: MergeStrategy::Replace,
        }
    }
}

/// Strategy for merging optimistic data with existing cache
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Replace existing data completely
    Replace,
    /// Merge fields, keeping existing values for missing fields
    Merge,
    /// Deep merge nested objects
    DeepMerge,
    /// Custom merge function (not serializable)
    Custom,
}

/// Optimistic update operation
#[derive(Clone, Debug)]
pub struct OptimisticUpdate<T> {
    /// Unique ID for this update
    pub id: String,
    /// The query key being updated
    pub key: QueryKey,
    /// The optimistic data
    pub data: T,
    /// When the update was applied
    pub applied_at: Instant,
    /// Whether this update has been confirmed
    pub confirmed: bool,
    /// Whether this update has been rolled back
    pub rolled_back: bool,
    /// Rollback data (original state)
    pub rollback_data: Option<T>,
}

impl<T> OptimisticUpdate<T> {
    /// Create a new optimistic update
    pub fn new(key: QueryKey, data: T, rollback_data: Option<T>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key,
            data,
            applied_at: Instant::now(),
            confirmed: false,
            rolled_back: false,
            rollback_data,
        }
    }

    /// Check if this update has expired
    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.applied_at.elapsed() > timeout
    }

    /// Mark this update as confirmed
    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    /// Mark this update as rolled back
    pub fn rollback(&mut self) {
        self.rolled_back = true;
    }
}

/// Optimistic update manager
pub struct OptimisticManager<T> {
    /// Configuration for optimistic updates
    config: OptimisticConfig,
    /// Active optimistic updates
    updates: Arc<RwLock<HashMap<String, OptimisticUpdate<T>>>>,
    /// Update history for debugging
    history: Arc<RwLock<Vec<OptimisticUpdate<T>>>>,
}

impl<T: Clone + Serialize + DeserializeOwned> OptimisticManager<T> {
    /// Create a new optimistic update manager
    pub fn new(config: OptimisticConfig) -> Self {
        Self {
            config,
            updates: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Apply an optimistic update
    pub fn apply_update(&self, key: &QueryKey, data: T, rollback_data: Option<T>) -> String {
        let update = OptimisticUpdate::new(key.clone(), data, rollback_data);
        let id = update.id.clone();
        
        let mut updates = self.updates.write();
        updates.insert(id.clone(), update);
        
        // Add to history
        let mut history = self.history.write();
        history.push(updates.get(&id).unwrap().clone());
        
        // Keep only last 100 updates in history
        if history.len() > 100 {
            history.remove(0);
        }
        
        id
    }

    /// Get optimistic data for a key
    pub fn get_optimistic_data(&self, key: &QueryKey) -> Option<T> {
        let updates = self.updates.read();
        
        // Find the most recent unconfirmed update for this key
        updates
            .values()
            .filter(|update| update.key == *key && !update.confirmed && !update.rolled_back)
            .max_by_key(|update| update.applied_at)
            .map(|update| update.data.clone())
    }

    /// Confirm an optimistic update
    pub fn confirm_update(&self, update_id: &str) -> Result<(), QueryError> {
        let mut updates = self.updates.write();
        
        if let Some(update) = updates.get_mut(update_id) {
            update.confirm();
            Ok(())
        } else {
            Err(QueryError::GenericError("Update not found".to_string()))
        }
    }

    /// Rollback an optimistic update
    pub fn rollback_update(&self, update_id: &str) -> Result<Option<T>, QueryError> {
        let mut updates = self.updates.write();
        
        if let Some(update) = updates.get_mut(update_id) {
            update.rollback();
            Ok(update.rollback_data.clone())
        } else {
            Err(QueryError::GenericError("Update not found".to_string()))
        }
    }

    /// Rollback all updates for a specific key
    pub fn rollback_key(&self, key: &QueryKey) -> Vec<T> {
        let mut updates = self.updates.write();
        let mut rollback_data = Vec::new();
        
        for update in updates.values_mut() {
            if update.key == *key && !update.rolled_back {
                update.rollback();
                if let Some(data) = &update.rollback_data {
                    rollback_data.push(data.clone());
                }
            }
        }
        
        rollback_data
    }

    /// Rollback all updates matching a pattern
    pub fn rollback_pattern(&self, pattern: &QueryKeyPattern) -> Vec<T> {
        let mut updates = self.updates.write();
        let mut rollback_data = Vec::new();
        
        for update in updates.values_mut() {
            if update.key.matches_pattern(pattern) && !update.rolled_back {
                update.rollback();
                if let Some(data) = &update.rollback_data {
                    rollback_data.push(data.clone());
                }
            }
        }
        
        rollback_data
    }

    /// Clean up expired updates
    pub fn cleanup_expired(&self) -> usize {
        let mut updates = self.updates.write();
        let initial_count = updates.len();
        
        updates.retain(|_, update| !update.is_expired(self.config.rollback_timeout));
        
        initial_count - updates.len()
    }

    /// Get all active updates
    pub fn get_active_updates(&self) -> Vec<OptimisticUpdate<T>> {
        let updates = self.updates.read();
        updates
            .values()
            .filter(|update| !update.confirmed && !update.rolled_back)
            .cloned()
            .collect()
    }

    /// Get update statistics
    pub fn get_stats(&self) -> OptimisticStats {
        let updates = self.updates.read();
        let history = self.history.read();
        
        let active = updates.values().filter(|u| !u.confirmed && !u.rolled_back).count();
        let confirmed = updates.values().filter(|u| u.confirmed).count();
        let rolled_back = updates.values().filter(|u| u.rolled_back).count();
        let expired = updates.values().filter(|u| u.is_expired(self.config.rollback_timeout)).count();
        
        OptimisticStats {
            active_updates: active,
            confirmed_updates: confirmed,
            rolled_back_updates: rolled_back,
            expired_updates: expired,
            total_history: history.len(),
        }
    }

    /// Clear all updates
    pub fn clear_all(&self) {
        let mut updates = self.updates.write();
        updates.clear();
    }

    /// Clear history
    pub fn clear_history(&self) {
        let mut history = self.history.write();
        history.clear();
    }
}

/// Statistics for optimistic updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimisticStats {
    /// Number of active (unconfirmed, unrolled) updates
    pub active_updates: usize,
    /// Number of confirmed updates
    pub confirmed_updates: usize,
    /// Number of rolled back updates
    pub rolled_back_updates: usize,
    /// Number of expired updates
    pub expired_updates: usize,
    /// Total number of updates in history
    pub total_history: usize,
}

/// Optimistic mutation result
pub struct OptimisticMutationResult<T> {
    /// The optimistic update ID
    pub update_id: String,
    /// Whether the update was applied optimistically
    pub applied: bool,
    /// The optimistic data
    pub optimistic_data: Option<T>,
    /// Rollback function
    pub rollback: Box<dyn Fn() -> Result<(), QueryError> + Send + Sync>,
}

impl<T> OptimisticMutationResult<T> {
    /// Create a new optimistic mutation result
    pub fn new(
        update_id: String,
        applied: bool,
        optimistic_data: Option<T>,
        rollback: Box<dyn Fn() -> Result<(), QueryError> + Send + Sync>,
    ) -> Self {
        Self {
            update_id,
            applied,
            optimistic_data,
            rollback,
        }
    }

    /// Rollback this optimistic update
    pub fn rollback(&self) -> Result<(), QueryError> {
        (self.rollback)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::QueryKey;
    
    #[test]
    fn test_optimistic_update_creation() {
        let config = OptimisticConfig::default();
        let manager = OptimisticManager::<String>::new(config);
        
        let key = QueryKey::from("test");
        let data = "optimistic data".to_string();
        let rollback = "original data".to_string();
        
        let update_id = manager.apply_update(&key, data.clone(), Some(rollback.clone()));
        
        // Check that update was applied
        let optimistic_data = manager.get_optimistic_data(&key);
        assert_eq!(optimistic_data, Some(data));
        
        // Check that update can be confirmed
        assert!(manager.confirm_update(&update_id).is_ok());
        
        // Check that update can be rolled back
        let rollback_data = manager.rollback_update(&update_id);
        assert!(rollback_data.is_ok());
    }
    
    #[test]
    fn test_optimistic_update_rollback() {
        let config = OptimisticConfig::default();
        let manager = OptimisticManager::<String>::new(config);
        
        let key = QueryKey::from("test");
        let original_data = "original data".to_string();
        let optimistic_data = "optimistic data".to_string();
        
        // Apply optimistic update
        let update_id = manager.apply_update(&key, optimistic_data.clone(), Some(original_data.clone()));
        
        // Verify optimistic data is active
        assert_eq!(manager.get_optimistic_data(&key), Some(optimistic_data));
        
        // Rollback the update
        let rollback_result = manager.rollback_update(&update_id);
        assert!(rollback_result.is_ok());
        assert_eq!(rollback_result.unwrap(), Some(original_data));
        
        // Verify optimistic data is no longer active
        assert_eq!(manager.get_optimistic_data(&key), None);
    }
    
    #[test]
    fn test_optimistic_update_expiration() {
        let mut config = OptimisticConfig::default();
        config.rollback_timeout = Duration::from_millis(10); // Very short timeout
        
        let manager = OptimisticManager::<String>::new(config);
        
        let key = QueryKey::from("test");
        let data = "test data".to_string();
        
        manager.apply_update(&key, data, None);
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(20));
        
        // Clean up expired updates
        let cleaned = manager.cleanup_expired();
        assert_eq!(cleaned, 1);
        
        // Verify no optimistic data remains
        assert_eq!(manager.get_optimistic_data(&key), None);
    }
}
