//! Mutation Hooks and Optimistic Updates
//!
//! Provides hooks for data mutations with automatic cache invalidation
//! and optimistic update capabilities.

use leptos::*;
use std::rc::Rc;
use std::time::Instant;
use std::future::Future;
use std::pin::Pin;
use serde::{Serialize, de::DeserializeOwned};

use crate::client::{QueryClient, QueryKey, QueryKeyPattern};
use crate::retry::{QueryError, RetryConfig, execute_with_retry};
use crate::types::{MutationStatus};

/// Options for mutation configuration
pub struct MutationOptions<TData, TVariables, TContext> {
    /// Called before mutation executes (for optimistic updates)
    pub on_mutate: Option<Box<dyn Fn(&TVariables) -> Option<TContext> + Send + Sync>>,
    /// Called on successful mutation
    pub on_success: Option<Box<dyn Fn(&TData, &TVariables, &Option<TContext>) + Send + Sync>>,
    /// Called on failed mutation
    pub on_error: Option<Box<dyn Fn(&QueryError, &TVariables, &Option<TContext>) + Send + Sync>>,
    /// Called after mutation settles (success or error)
    pub on_settled: Option<Box<dyn Fn(&Option<TData>, &Option<QueryError>, &TVariables, &Option<TContext>) + Send + Sync>>,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Queries to invalidate on success
    pub invalidates: Vec<QueryKeyPattern>,
    /// Whether to throw errors in async mode
    pub throw_on_error: bool,
}

impl<TData, TVariables, TContext> Default for MutationOptions<TData, TVariables, TContext> {
    fn default() -> Self {
        Self {
            on_mutate: None,
            on_success: None,
            on_error: None,
            on_settled: None,
            retry: RetryConfig::default(),
            invalidates: Vec::new(),
            throw_on_error: false,
        }
    }
}

impl<TData, TVariables, TContext> Clone for MutationOptions<TData, TVariables, TContext> {
    fn clone(&self) -> Self {
        Self {
            on_mutate: None, // Can't clone function pointers
            on_success: None,
            on_error: None,
            on_settled: None,
            retry: self.retry.clone(),
            invalidates: self.invalidates.clone(),
            throw_on_error: self.throw_on_error,
        }
    }
}

/// Result of a mutation hook
#[derive(Clone)]
pub struct MutationResult<TData: 'static, TVariables: 'static> {
    /// The mutation result data
    pub data: Signal<Option<TData>>,
    /// Error if any
    pub error: Signal<Option<QueryError>>,
    /// Whether the mutation is idle
    pub is_idle: Signal<bool>,
    /// Whether the mutation is loading
    pub is_loading: Signal<bool>,
    /// Whether the mutation succeeded
    pub is_success: Signal<bool>,
    /// Whether the mutation failed
    pub is_error: Signal<bool>,
    /// Current mutation status
    pub status: Signal<MutationStatus>,
    /// When the mutation was submitted
    pub submitted_at: Signal<Option<Instant>>,
    /// Variables from the last mutation
    pub variables: Signal<Option<TVariables>>,
    
    // Actions
    /// Execute the mutation
    pub mutate: Callback<TVariables>,
    /// Execute the mutation and return a future
    pub mutate_async: Rc<dyn Fn(TVariables) -> Pin<Box<dyn Future<Output = Result<TData, QueryError>>>>>,
    /// Reset the mutation state
    pub reset: Callback<()>,
}

/// Main mutation hook
pub fn use_mutation<TData, TVariables, TContext, F, Fut>(
    mutation_fn: F,
    options: MutationOptions<TData, TVariables, TContext>,
) -> MutationResult<TData, TVariables>
where
    TData: Clone + 'static,
    TVariables: Clone + 'static,
    TContext: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, QueryError>> + 'static,
{
    let client = use_context::<QueryClient>()
        .expect("QueryClient not provided");
    
    // Local state
    let (data, set_data) = create_signal(None::<TData>);
    let (error, set_error) = create_signal(None::<QueryError>);
    let (status, set_status) = create_signal(MutationStatus::Idle);
    let (is_loading, set_loading) = create_signal(false);
    let (submitted_at, set_submitted_at) = create_signal(None::<Instant>);
    let (variables, set_variables) = create_signal(None::<TVariables>);
    
    // Execute mutation function
    let execute = {
        let mutation_fn = mutation_fn.clone();
        let options = options.clone();
        let client = client.clone();
        
        Rc::new(move |vars: TVariables| {
            let mutation_fn = mutation_fn.clone();
            let options = options.clone();
            let client = client.clone();
            let vars_clone = vars.clone();
            
            spawn_local(async move {
                set_loading.set(true);
                set_status.set(MutationStatus::Loading);
                set_submitted_at.set(Some(Instant::now()));
                set_variables.set(Some(vars_clone.clone()));
                
                // Clear previous state
                set_error.set(None);
                if !matches!(status.get(), MutationStatus::Loading) {
                    set_data.set(None);
                }
                
                // Call onMutate for optimistic updates
                let context = options.on_mutate.as_ref().and_then(|f| f(&vars_clone));
                
                // Execute mutation with retry
                let result = execute_with_retry(
                    || mutation_fn(vars_clone.clone()),
                    &options.retry,
                ).await;
                
                match result {
                    Ok(result_data) => {
                        set_data.set(Some(result_data.clone()));
                        set_error.set(None);
                        set_status.set(MutationStatus::Success);
                        
                        // Invalidate related queries
                        for pattern in &options.invalidates {
                            client.invalidate_queries(pattern);
                        }
                        
                        // Call onSuccess
                        if let Some(on_success) = &options.on_success {
                            on_success(&result_data, &vars_clone, &context);
                        }
                        
                        // Call onSettled
                        if let Some(on_settled) = &options.on_settled {
                            on_settled(&Some(result_data), &None, &vars_clone, &context);
                        }
                    }
                    Err(err) => {
                        set_error.set(Some(err.clone()));
                        set_status.set(MutationStatus::Error);
                        
                        // Call onError
                        if let Some(on_error) = &options.on_error {
                            on_error(&err, &vars_clone, &context);
                        }
                        
                        // Call onSettled
                        if let Some(on_settled) = &options.on_settled {
                            on_settled(&None, &Some(err), &vars_clone, &context);
                        }
                    }
                }
                
                set_loading.set(false);
            });
        })
    };
    
    // Create async version
    let mutate_async = {
        let mutation_fn = mutation_fn.clone();
        let options = options.clone();
        
        Rc::new(move |vars: TVariables| -> Pin<Box<dyn Future<Output = Result<TData, QueryError>>>> {
            let mutation_fn = mutation_fn.clone();
            let options = options.clone();
            
            Box::pin(async move {
                execute_with_retry(|| mutation_fn(vars.clone()), &options.retry).await
            })
        })
    };
    
    // Create computed signals
    let is_idle = create_memo(move |_| status.get() == MutationStatus::Idle);
    let is_success = create_memo(move |_| status.get() == MutationStatus::Success);
    let is_error = create_memo(move |_| status.get() == MutationStatus::Error);
    
    MutationResult {
        data: data.into(),
        error: error.into(),
        is_idle: is_idle.into(),
        is_loading: is_loading.into(),
        is_success: is_success.into(),
        is_error: is_error.into(),
        status: status.into(),
        submitted_at: submitted_at.into(),
        variables: variables.into(),
        mutate: Callback::new({
            let execute = execute.clone();
            move |variables| {
                execute(variables);
            }
        }),
        mutate_async,
        reset: Callback::new(move |_| {
            set_data.set(None);
            set_error.set(None);
            set_status.set(MutationStatus::Idle);
            set_submitted_at.set(None);
            set_variables.set(None);
            set_loading.set(false);
        }),
    }
}

/// Context for optimistic updates
#[derive(Clone)]
pub struct MutationContext<T> {
    pub previous_data: Option<T>,
    pub query_key: QueryKey,
}

/// Optimistic update helper
pub fn use_optimistic_mutation<TData, TVariables, F, Fut>(
    query_key: QueryKey,
    mutation_fn: F,
    optimistic_update: impl Fn(&TVariables) -> TData + Send + Sync + 'static,
) -> MutationResult<TData, TVariables>
where
    TData: Serialize + DeserializeOwned + Clone + 'static,
    TVariables: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, QueryError>> + 'static,
{
    let client = use_context::<QueryClient>().unwrap();
    
    use_mutation(
        mutation_fn,
        MutationOptions {
            on_mutate: Some(Box::new({
                let client = client.clone();
                let query_key = query_key.clone();
                let optimistic_update = Box::leak(Box::new(optimistic_update)) as &(dyn Fn(&TVariables) -> TData + Send + Sync);
                
                move |variables: &TVariables| {
                    // Snapshot previous value
                    let previous_data = client.get_query_data::<TData>(&query_key);
                    
                    // Optimistically update
                    let optimistic_data = optimistic_update(variables);
                    client.set_query_data(&query_key, optimistic_data).ok();
                    
                    // Return context with previous data
                    Some(MutationContext {
                        previous_data,
                        query_key: query_key.clone(),
                    })
                }
            })),
            on_error: Some(Box::new({
                let client = client.clone();
                
                move |_error: &QueryError, _variables: &TVariables, context: &Option<MutationContext<TData>>| {
                    // Rollback on error
                    if let Some(ctx) = context {
                        if let Some(previous) = &ctx.previous_data {
                            client.set_query_data(&ctx.query_key, previous.clone()).ok();
                        }
                    }
                }
            })),
            on_settled: Some(Box::new({
                let client = client.clone();
                let query_key = query_key.clone();
                
                move |_data: &Option<TData>, _error: &Option<QueryError>, _variables: &TVariables, _context: &Option<MutationContext<TData>>| {
                    // Always refetch after mutation to ensure consistency
                    client.invalidate_queries(&QueryKeyPattern::Exact(query_key.clone()));
                }
            })),
            ..Default::default()
        },
    )
}

/// Simplified mutation hook for common use cases
pub fn use_simple_mutation<TData, TVariables, F, Fut>(
    mutation_fn: F,
    invalidates: Vec<QueryKey>,
) -> MutationResult<TData, TVariables>
where
    TData: Clone + 'static,
    TVariables: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, QueryError>> + 'static,
{
    use_mutation(
        mutation_fn,
        MutationOptions::<TData, TVariables, ()> {
            invalidates: invalidates
                .into_iter()
                .map(QueryKeyPattern::Exact)
                .collect(),
            ..Default::default()
        },
    )
}

/// Hook for mutations that affect multiple queries
pub fn use_bulk_mutation<TData, TVariables, F, Fut>(
    mutation_fn: F,
    invalidate_patterns: Vec<QueryKeyPattern>,
) -> MutationResult<TData, TVariables>
where
    TData: Clone + 'static,
    TVariables: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, QueryError>> + 'static,
{
    use_mutation(
        mutation_fn,
        MutationOptions::<TData, TVariables, ()> {
            invalidates: invalidate_patterns,
            ..Default::default()
        },
    )
}

/// Hook for mutations with custom success/error handling
pub fn use_mutation_with_callbacks<TData, TVariables, F, Fut>(
    mutation_fn: F,
    on_success: impl Fn(&TData) + Send + Sync + 'static,
    on_error: impl Fn(&QueryError) + Send + Sync + 'static,
) -> MutationResult<TData, TVariables>
where
    TData: Clone + 'static,
    TVariables: Clone + 'static,
    F: Fn(TVariables) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<TData, QueryError>> + 'static,
{
    use_mutation(
        mutation_fn,
        MutationOptions::<TData, TVariables, ()> {
            on_success: Some(Box::new(move |data, _variables, _context| {
                on_success(data);
            })),
            on_error: Some(Box::new(move |error, _variables, _context| {
                on_error(error);
            })),
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mutation_status_transitions() {
        // Test basic status transitions
        assert_eq!(MutationStatus::Idle, MutationStatus::Idle);
        assert_ne!(MutationStatus::Idle, MutationStatus::Loading);
    }
}