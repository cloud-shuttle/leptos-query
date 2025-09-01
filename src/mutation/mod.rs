//! Mutation Hooks and Options
//!
//! The main user-facing API for data mutations with optimistic updates.

use leptos::prelude::*;
use leptos::task::spawn_local;
use std::future::Future;
use serde::{Serialize, de::DeserializeOwned};

use crate::client::QueryClient;
use crate::retry::RetryConfig;
use crate::types::QueryKeyPattern;

/// Options for configuring a mutation
#[derive(Clone)]
pub struct MutationOptions {
    /// Whether the mutation should run
    pub enabled: bool,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Whether to invalidate queries on success
    pub invalidate_queries: Option<Vec<QueryKeyPattern>>,
}

impl Default for MutationOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            retry: RetryConfig::default(),
            invalidate_queries: None,
        }
    }
}

impl MutationOptions {
    /// Create options with custom retry configuration
    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }
    
    /// Set queries to invalidate on success
    pub fn invalidate_queries(mut self, patterns: Vec<QueryKeyPattern>) -> Self {
        self.invalidate_queries = Some(patterns);
        self
    }
    
    /// Disable the mutation by default
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Result of a mutation hook
#[derive(Clone)]
pub struct MutationResult<TData: 'static + Send + Sync, TError: 'static + Send + Sync, TVariables: 'static> {
    /// The mutation data
    pub data: Signal<Option<TData>>,
    /// Error if any
    pub error: Signal<Option<TError>>,
    /// Whether the mutation is loading
    pub is_loading: Signal<bool>,
    /// Whether the mutation succeeded
    pub is_success: Signal<bool>,
    /// Whether the mutation failed
    pub is_error: Signal<bool>,
    
    // Actions
    /// Execute the mutation
    pub mutate: Callback<TVariables>,
}

/// Main mutation hook
pub fn use_mutation<TData, TError, TVariables, F, Fut>(
    mutation_fn: F,
    options: MutationOptions,
) -> MutationResult<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
    F: Fn(TVariables) -> Fut + Send + Sync + 'static + Clone,
    Fut: Future<Output = Result<TData, TError>> + 'static,
{
    // Create reactive state
    let (data, set_data) = signal(None::<TData>);
    let (error, set_error) = signal(None::<TError>);
    let (is_loading, set_loading) = signal(false);

    // Get query client from context
    let client = use_context::<QueryClient>().expect("QueryClient not found in context");
    
    // Create mutation function
    let execute_mutation = {
        let client = client.clone();
        let mutation_fn = mutation_fn.clone();
        let options = options.clone();
        
        move |vars: TVariables| {
            let _client = client.clone();
            let mutation_fn = mutation_fn.clone();
            let options = options.clone();
            
            spawn_local(async move {
                set_loading.set(true);
                set_error.set(None);
                
                // Execute the mutation directly without retry for now
                let result = mutation_fn(vars.clone()).await;
                
                match result {
                    Ok(result_data) => {
                        set_data.set(Some(result_data));
                        
                        // Invalidate queries if specified
                        if let Some(patterns) = &options.invalidate_queries {
                            for _pattern in patterns {
                                // TODO: Implement invalidate_queries method
                                // client.invalidate_queries(pattern);
                            }
                        }
                    }
                    Err(err) => {
                        set_error.set(Some(err));
                    }
                }
                
                set_loading.set(false);
            });
        }
    };
    
    // Create computed signals
    let is_success = Memo::new(move |_| data.get().is_some() && error.get().is_none());
    let is_error = Memo::new(move |_| error.get().is_some());
    
    // Create result
    MutationResult {
        data: data.into(),
        error: error.into(),
        is_loading: is_loading.into(),
        is_success: is_success.into(),
        is_error: is_error.into(),
        mutate: Callback::new(execute_mutation),
    }
}

/// Hook for optimistic updates
pub fn use_optimistic_mutation<TData, TError, TVariables, F, Fut>(
    mutation_fn: F,
    options: MutationOptions,
    _on_success: impl Fn(&TData) + Send + Sync + 'static,
    _on_error: impl Fn(&TError) + Send + Sync + 'static,
) -> MutationResult<TData, TError, TVariables>
where
    TData: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
    TError: Clone + Send + Sync + 'static,
    TVariables: Clone + Send + Sync + 'static,
    F: Fn(TVariables) -> Fut + Send + Sync + 'static + Clone,
    Fut: Future<Output = Result<TData, TError>> + 'static,
{
    // For now, just return the regular mutation
    // TODO: Implement optimistic updates
    use_mutation(mutation_fn, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::types::QueryKey;
    
    #[test]
    fn test_mutation_options_builder() {
        let options = MutationOptions::default()
            .with_retry(RetryConfig::new(5, Duration::from_secs(2)))
            .invalidate_queries(vec![QueryKeyPattern::Exact(QueryKey::from("users"))]);
        
        assert_eq!(options.retry.max_retries, 5);
        assert!(options.invalidate_queries.is_some());
    }
}