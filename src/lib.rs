//! # Leptos Query
//!
//! A powerful, type-safe data fetching and caching library for Leptos that provides:
//! - Automatic background refetching
//! - Request deduplication  
//! - Optimistic updates
//! - Intelligent caching strategies
//! - Offline support
//! - DevTools integration
//!
//! ## Quick Start
//!
//! ```rust
//! use leptos::*;
//! use leptos_query::*;
//! 
//! #[component]
//! fn App() -> impl IntoView {
//!     provide_context(QueryClient::new(QueryClientConfig::default()));
//!     
//!     view! {
//!         <UserProfile user_id=1 />
//!     }
//! }
//! 
//! #[component] 
//! fn UserProfile(user_id: u32) -> impl IntoView {
//!     let user = use_query(
//!         move || ["users", user_id.to_string()],
//!         move || async move {
//!             // Your fetch logic here
//!             fetch_user(user_id).await
//!         },
//!         QueryOptions::default()
//!     );
//!     
//!     view! {
//!         <div>
//!             {move || match (user.data.get(), user.is_loading.get()) {
//!                 (Some(user_data), _) => view! { <div>{user_data.name}</div> }.into_view(),
//!                 (None, true) => view! { <div>"Loading..."</div> }.into_view(),
//!                 _ => view! { <div>"Error"</div> }.into_view(),
//!             }}
//!         </div>
//!     }
//! }
//! ```

// Re-export core types
pub use client::*;
pub use query::*;
pub use mutation::*;

// Modules
pub mod compat;
pub mod client;
pub mod query;
pub mod mutation;
pub mod retry;
pub mod dedup;

// Common types
pub mod types {
    use std::time::{Duration};
    
    /// Unique identifier for query observers
    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct QueryObserverId(pub uuid::Uuid);
    
    impl QueryObserverId {
        pub fn new() -> Self {
            Self(uuid::Uuid::new_v4())
        }
    }
    
    /// Unique identifier for mutations
    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct MutationId(pub uuid::Uuid);
    
    impl MutationId {
        pub fn new() -> Self {
            Self(uuid::Uuid::new_v4())
        }
    }
    
    /// Query status enum
    #[derive(Clone, Debug, PartialEq)]
    pub enum QueryStatus {
        Idle,
        Loading,
        Success,
        Error,
    }
    
    /// Mutation status enum  
    #[derive(Clone, Debug, PartialEq)]
    pub enum MutationStatus {
        Idle,
        Loading,
        Success,
        Error,
    }
    
    /// Query metadata for analytics and debugging
    #[derive(Clone, Debug, Default)]
    pub struct QueryMeta {
        pub fetch_count: u32,
        pub error_count: u32,
        pub last_fetch_duration: Option<Duration>,
        pub total_fetch_time: Duration,
    }
    
    impl QueryMeta {
        pub fn record_fetch(&mut self, duration: Duration) {
            self.fetch_count += 1;
            self.last_fetch_duration = Some(duration);
            self.total_fetch_time += duration;
        }
        
        pub fn record_error(&mut self) {
            self.error_count += 1;
        }
        
        pub fn average_fetch_time(&self) -> Option<Duration> {
            if self.fetch_count > 0 {
                Some(self.total_fetch_time / self.fetch_count)
            } else {
                None
            }
        }
        
        pub fn success_rate(&self) -> f64 {
            if self.fetch_count == 0 {
                0.0
            } else {
                (self.fetch_count - self.error_count) as f64 / self.fetch_count as f64
            }
        }
    }
}