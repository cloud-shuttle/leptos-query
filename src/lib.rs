//! Leptos Query - A powerful data fetching and caching library for Leptos 0.8
//! 
//! This library provides React Query/TanStack Query-like functionality for Leptos applications,
//! with full support for Leptos 0.8's modern reactive primitives.
//! 
//! ## Quick Start
//! 
//! ```rust
//! use leptos::*;
//! use leptos_query_rs::*;
//! 
//! #[component]
//! fn UserProfile(user_id: u32) -> impl IntoView {
//!     let user_query = use_query(
//!         move || &["users", &user_id.to_string()][..],
//!         move || || async move { fetch_user(user_id).await },
//!         QueryOptions::default()
//!     );
//! 
//!     view! {
//!         <div>
//!             {move || match user_query.data.get() {
//!                 Some(user) => view! { <h1>{user.name}</h1> }.into_view(),
//!                 None => view! { <p>"Loading..."</p> }.into_view(),
//!             }}
//!         </div>
//!     }
//! }
//! ```

// Re-export main modules
pub use client::{QueryClient, QueryClientConfig};
pub use query::{use_query, QueryResult};
pub use mutation::{use_mutation, MutationResult};
pub use retry::{QueryError, RetryConfig, RetryDelay};

// Modules
pub mod api;
pub mod client;
pub mod query;
pub mod mutation;
pub mod retry;
pub mod dedup;
pub mod infinite_query;
pub mod persistence;
pub mod devtools;
pub mod types;

// Re-export common types
pub use types::{QueryKey, QueryOptions, MutationOptions, QueryStatus, MutationStatus, QueryObserverId, MutationId, QueryMeta};