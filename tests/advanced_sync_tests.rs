//! Advanced Sync Integration Tests
//! 
//! These tests verify advanced synchronization features including CRDT operations,
//! conflict resolution, and real-time synchronization.

use leptos_query_rs::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestDocument {
    id: String,
    title: String,
    content: String,
    version: u64,
    last_modified: u64,
}

impl TestDocument {
    fn new(id: String, title: String, content: String) -> Self {
        Self {
            id,
            title,
            content,
            version: 1,
            last_modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn update(&mut self, title: Option<String>, content: Option<String>) {
        if let Some(t) = title {
            self.title = t;
        }
        if let Some(c) = content {
            self.content = c;
        }
        self.version += 1;
        self.last_modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "sync")]
    async fn test_crdt_three_way_merge() {
        // RED: Test three-way merge with CRDT operations
        let mut sync_manager1 = SyncManager::new().await.unwrap();
        let mut sync_manager2 = SyncManager::new().await.unwrap();
        let mut sync_manager3 = SyncManager::new().await.unwrap();

        let key = QueryKey::new(&["documents", "1"]);
        let base_doc = TestDocument::new("1".to_string(), "Base Title".to_string(), "Base content".to_string());

        // All managers start with the same base document
        sync_manager1.store_with_crdt(&key, base_doc.clone()).await.unwrap();
        sync_manager2.store_with_crdt(&key, base_doc.clone()).await.unwrap();
        sync_manager3.store_with_crdt(&key, base_doc.clone()).await.unwrap();

        // Manager 1 makes changes
        let mut doc1 = sync_manager1.get_with_crdt::<TestDocument>(&key).await.unwrap().unwrap();
        doc1.update(Some("Title from Manager 1".to_string()), Some("Content from Manager 1".to_string()));
        sync_manager1.store_with_crdt(&key, doc1).await.unwrap();

        // Manager 2 makes different changes
        let mut doc2 = sync_manager2.get_with_crdt::<TestDocument>(&key).await.unwrap().unwrap();
        doc2.update(Some("Title from Manager 2".to_string()), Some("Content from Manager 2".to_string()));
        sync_manager2.store_with_crdt(&key, doc2).await.unwrap();

        // Manager 3 merges both changes
        sync_manager3.merge_with(&mut sync_manager1).await.unwrap();
        sync_manager3.merge_with(&mut sync_manager2).await.unwrap();

        // The merged document should contain both changes
        let merged_doc = sync_manager3.get_with_crdt::<TestDocument>(&key).await.unwrap().unwrap();
        assert!(merged_doc.title.contains("Manager 1") || merged_doc.title.contains("Manager 2"));
        assert!(merged_doc.content.contains("Manager 1") || merged_doc.content.contains("Manager 2"));
    }

    #[tokio::test]
    #[cfg(feature = "sync")]
    async fn test_conflict_resolution_strategies() {
        // RED: Test different conflict resolution strategies
        let mut sync_manager = SyncManager::new().await.unwrap();
        let key = QueryKey::new(&["conflict", "test"]);

        // Test Last Writer Wins strategy
        let mut doc1 = TestDocument::new("1".to_string(), "First Title".to_string(), "First content".to_string());
        let mut doc2 = TestDocument::new("1".to_string(), "Second Title".to_string(), "Second content".to_string());
        
        // Set different versions to test version-based conflict resolution
        doc1.version = 1;
        doc2.version = 2;

        sync_manager.store_with_crdt(&key, doc1.clone()).await.unwrap();
        sync_manager.store_with_crdt(&key, doc2.clone()).await.unwrap();

        sync_manager.resolve_conflicts(&key, ConflictResolutionStrategy::LastWriterWins).await.unwrap();

        let resolved_doc = sync_manager.get_with_crdt::<TestDocument>(&key).await.unwrap().unwrap();
        assert_eq!(resolved_doc.title, "Second Title");
        assert_eq!(resolved_doc.content, "Second content");
    }

    #[tokio::test]
    #[cfg(feature = "sync")]
    async fn test_offline_operation_queuing() {
        // RED: Test that operations are queued when offline
        let mut sync_manager = SyncManager::new().await.unwrap();
        let key = QueryKey::new(&["offline", "test"]);

        // Set to offline mode
        sync_manager.set_network_status(NetworkStatus::Offline);

        // Try to queue an operation
        let doc = TestDocument::new("1".to_string(), "Offline Title".to_string(), "Offline content".to_string());
        let operation_id = sync_manager.queue_operation(&key, doc.clone()).await.unwrap();

        // Should have a queued operation
        assert!(operation_id.is_some());
        assert!(sync_manager.has_pending_operations());
        assert_eq!(sync_manager.pending_operation_count(), 1);

        // Go back online and process queued operations
        sync_manager.set_network_status(NetworkStatus::Online);
        sync_manager.process_queued_operations().await.unwrap();

        // Should have no pending operations
        assert!(!sync_manager.has_pending_operations());
        assert_eq!(sync_manager.pending_operation_count(), 0);

        // Data should be stored
        let stored_doc = sync_manager.get_with_crdt::<TestDocument>(&key).await.unwrap().unwrap();
        assert_eq!(stored_doc.title, "Offline Title");
    }

    #[tokio::test]
    #[cfg(feature = "sync")]
    async fn test_automatic_sync_with_conflicts() {
        // RED: Test automatic sync that resolves conflicts
        let mut sync_manager1 = SyncManager::new().await.unwrap();
        let mut sync_manager2 = SyncManager::new().await.unwrap();

        let key = QueryKey::new(&["auto", "sync"]);
        let doc = TestDocument::new("1".to_string(), "Original Title".to_string(), "Original content".to_string());

        // Both managers have the same document
        sync_manager1.store_with_crdt(&key, doc.clone()).await.unwrap();
        sync_manager2.store_with_crdt(&key, doc.clone()).await.unwrap();

        // Make conflicting changes
        let mut doc1 = sync_manager1.get_with_crdt::<TestDocument>(&key).await.unwrap().unwrap();
        doc1.update(Some("Manager 1 Title".to_string()), None);
        sync_manager1.store_with_crdt(&key, doc1).await.unwrap();

        let mut doc2 = sync_manager2.get_with_crdt::<TestDocument>(&key).await.unwrap().unwrap();
        doc2.update(Some("Manager 2 Title".to_string()), None);
        sync_manager2.store_with_crdt(&key, doc2).await.unwrap();

        // Auto sync should resolve conflicts
        let sync_result1 = sync_manager1.auto_sync().await.unwrap();
        let sync_result2 = sync_manager2.auto_sync().await.unwrap();

        assert!(sync_result1.synced_operations > 0);
        assert!(sync_result2.synced_operations > 0);
        assert!(sync_result1.conflicts_resolved >= 0);
        assert!(sync_result2.conflicts_resolved >= 0);
    }

    #[tokio::test]
    #[cfg(feature = "sync")]
    async fn test_crdt_operation_ordering() {
        // RED: Test that CRDT operations maintain proper ordering
        let mut sync_manager = SyncManager::new().await.unwrap();
        let key = QueryKey::new(&["ordering", "test"]);

        // Create a document with version 1
        let mut doc = TestDocument::new("1".to_string(), "Title".to_string(), "Content".to_string());
        doc.version = 1;
        sync_manager.store_with_crdt(&key, doc.clone()).await.unwrap();

        // Update with version 2
        doc.version = 2;
        doc.title = "Updated Title".to_string();
        sync_manager.store_with_crdt(&key, doc.clone()).await.unwrap();

        // Try to store an older version (should be ignored or merged properly)
        let mut old_doc = TestDocument::new("1".to_string(), "Old Title".to_string(), "Old content".to_string());
        old_doc.version = 1;
        sync_manager.store_with_crdt(&key, old_doc).await.unwrap();

        // The final document should have the newer version
        let final_doc = sync_manager.get_with_crdt::<TestDocument>(&key).await.unwrap().unwrap();
        assert_eq!(final_doc.version, 2);
        assert_eq!(final_doc.title, "Updated Title");
    }

    #[tokio::test]
    #[cfg(feature = "sync")]
    async fn test_sync_performance_with_large_data() {
        // RED: Test sync performance with large amounts of data
        let mut sync_manager = SyncManager::new().await.unwrap();
        let start_time = std::time::Instant::now();

        // Store 100 documents
        for i in 0..100 {
            let key = QueryKey::new(&["large", "data", &i.to_string()]);
            let doc = TestDocument::new(
                i.to_string(),
                format!("Title {}", i),
                format!("Content with lots of data {}", i).repeat(100), // Large content
            );
            sync_manager.store_with_crdt(&key, doc).await.unwrap();
        }

        let store_time = start_time.elapsed();

        // Auto sync should complete quickly
        let sync_start = std::time::Instant::now();
        let sync_result = sync_manager.auto_sync().await.unwrap();
        let sync_time = sync_start.elapsed();

        // Performance assertions
        assert!(store_time < Duration::from_secs(1)); // Should store 100 docs in under 1 second
        assert!(sync_time < Duration::from_millis(100)); // Should sync in under 100ms
        assert_eq!(sync_result.synced_operations, 100);
    }
}

// Fallback tests for when sync feature is not enabled
#[cfg(not(feature = "sync"))]
mod fallback_tests {
    use super::*;

    #[test]
    fn test_sync_feature_not_enabled() {
        // Test that sync features gracefully degrade when not enabled
        let client = QueryClient::new();
        let key = QueryKey::new(&["fallback", "test"]);
        let doc = TestDocument::new("1".to_string(), "Fallback Title".to_string(), "Fallback content".to_string());

        // Basic functionality should still work
        client.set_query_data(&key, doc.clone()).unwrap();
        let retrieved = client.get_query_data::<TestDocument>(&key);
        assert_eq!(retrieved, Some(doc));
    }
}
