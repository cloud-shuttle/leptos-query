//! Leptos Query - A React Query inspired data fetching library for Leptos
//!
//! This library provides a powerful and flexible way to manage server state
//! in Leptos applications, with features like caching, background updates,
//! optimistic updates, and more.
//!
//! ## Features
//!
//! - **Declarative Data Fetching**: Write queries as simple functions
//! - **Automatic Caching**: Built-in cache with configurable stale times
//! - **Background Updates**: Keep data fresh with background refetching
//! - **Optimistic Updates**: Update UI immediately with rollback on error
//! - **Error Handling**: Comprehensive error handling with retry logic
//! - **Type Safety**: Full type safety with Rust's type system
//! - **WASM Compatible**: Works in both native and web environments
//!
//! ## Quick Start
//!
//! ```rust
//! use leptos::prelude::*;
//! use leptos_query_rs::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Clone, Debug, Serialize, Deserialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     email: String,
//! }
//!
//! async fn fetch_user(id: u32) -> Result<User, QueryError> {
//!     // Your async function here
//!     Ok(User {
//!         id,
//!         name: "John Doe".to_string(),
//!         email: "john@example.com".to_string(),
//!     })
//! }
//!
//! #[component]
//! fn UserProfile(user_id: u32) -> impl IntoView {
//!     let user_query = use_query(
//!         move || QueryKey::new(&["user", &user_id.to_string()]),
//!         move || async move { fetch_user(user_id).await },
//!         QueryOptions::default(),
//!     );
//!
//!     view! {
//!         <div>
//!             {move || {
//!                 if let Some(user) = user_query.data.get() {
//!                     format!("User: {}", user.name)
//!                 } else if user_query.is_loading.get() {
//!                     "Loading...".to_string()
//!                 } else {
//!                     "No user found".to_string()
//!                 }
//!             }}
//!         </div>
//!     }
//! }
//! ```

use leptos::prelude::*;

pub mod client;
pub mod query;
pub mod mutation;
pub mod retry;
pub mod types;
pub mod dedup;
pub mod infinite;
pub mod persistence;
pub mod optimistic;
pub mod devtools;
pub mod sync;

// Re-export main types and functions
pub use client::{QueryClient, SerializedData, CacheEntry};
pub use query::{use_query, QueryOptions, QueryResult};
pub use mutation::{use_mutation, MutationOptions, MutationResult};
pub use retry::{QueryError, RetryConfig, execute_with_retry};
pub use types::{QueryKey, QueryStatus, QueryMeta, QueryKeyPattern, QueryObserverId};
pub use infinite::{use_infinite_query, InfiniteQueryOptions, InfiniteQueryResult, Page, PageInfo};
pub use persistence::{PersistenceManager, PersistenceConfig, StorageBackend};
#[cfg(feature = "persistence")]
pub use persistence::{LocalStorageBackend, IndexedDBBackend};
pub use optimistic::{OptimisticManager, OptimisticConfig, OptimisticUpdate, OptimisticStats};
pub use devtools::{DevToolsManager, DevToolsConfig, DevToolsServer, QueryMetrics, NetworkRequest, CacheOperation, DevToolsEvent, DevToolsExport};
pub use sync::{SyncManager, ConflictResolutionStrategy, NetworkStatus, SyncResult};

/// Provide the QueryClient context to the app
#[component]
pub fn QueryClientProvider(
    children: Children,
) -> impl IntoView {
    let client = QueryClient::new();
    provide_context(client);
    
    children()
}