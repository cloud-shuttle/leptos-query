use crate::{
    client::QueryClient,
    types::QueryKey,
    retry::RetryConfig,
    QueryError,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use std::{sync::Arc, future::Future};
use crate::QueryObserverId;

/// Configuration for infinite queries
#[derive(Clone, Debug)]
pub struct InfiniteQueryOptions {
    /// Retry configuration for failed requests
    pub retry: RetryConfig,
    /// Whether to keep previous pages when fetching new ones
    pub keep_previous_data: bool,
    /// Maximum number of pages to keep in memory
    pub max_pages: Option<usize>,
    /// Whether to refetch when window regains focus
    pub refetch_on_window_focus: bool,
    /// Whether to refetch when reconnecting to the internet
    pub refetch_on_reconnect: bool,
}

impl Default for InfiniteQueryOptions {
    fn default() -> Self {
        Self {
            retry: RetryConfig::default(),
            keep_previous_data: true,
            max_pages: Some(10),
            refetch_on_window_focus: true,
            refetch_on_reconnect: true,
        }
    }
}

/// Builder for infinite query options
#[derive(Clone)]
pub struct InfiniteQueryOptionsBuilder {
    options: InfiniteQueryOptions,
}

impl Default for InfiniteQueryOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InfiniteQueryOptionsBuilder {
    pub fn new() -> Self {
        Self {
            options: InfiniteQueryOptions::default(),
        }
    }

    pub fn retry(mut self, retry: RetryConfig) -> Self {
        self.options.retry = retry;
        self
    }

    pub fn keep_previous_data(mut self, keep: bool) -> Self {
        self.options.keep_previous_data = keep;
        self
    }

    pub fn max_pages(mut self, max: Option<usize>) -> Self {
        self.options.max_pages = max;
        self
    }

    pub fn refetch_on_window_focus(mut self, refetch: bool) -> Self {
        self.options.refetch_on_window_focus = refetch;
        self
    }

    pub fn refetch_on_reconnect(mut self, refetch: bool) -> Self {
        self.options.refetch_on_reconnect = refetch;
        self
    }

    pub fn build(self) -> InfiniteQueryOptions {
        self.options
    }
}

/// Page information for infinite queries
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageInfo {
    /// Current page number
    pub page: usize,
    /// Items per page
    pub per_page: usize,
    /// Total number of items
    pub total: usize,
    /// Whether there are more pages
    pub has_next: bool,
    /// Whether there are previous pages
    pub has_prev: bool,
}

/// A single page of data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Page<T> {
    /// Page data
    pub data: Vec<T>,
    /// Page metadata
    pub info: PageInfo,
}

/// Infinite query result with pagination support
#[derive(Clone)]
pub struct InfiniteQueryResult<T> {
    /// All pages of data
    pub pages: RwSignal<Vec<Page<T>>>,
    /// Current page number
    pub current_page: RwSignal<usize>,
    /// Whether more data can be loaded
    pub has_next: RwSignal<bool>,
    /// Whether previous data exists
    pub has_prev: RwSignal<bool>,
    /// Loading state
    pub is_loading: RwSignal<bool>,
    /// Error state
    pub error: RwSignal<Option<QueryError>>,
    /// Whether data is stale
    pub is_stale: RwSignal<bool>,
    /// Whether currently fetching
    pub is_fetching: RwSignal<bool>,
    /// Query key
    pub key: QueryKey,
    /// Observer ID
    pub observer_id: QueryObserverId,
    /// Client reference
    client: Arc<QueryClient>,
}

impl<T> InfiniteQueryResult<T>
where
    T: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Get the next page of data
    pub async fn fetch_next_page(&self) -> Result<(), QueryError> {
        let current_page = self.current_page.get();
        let has_next = self.has_next.get();
        
        if !has_next {
            return Ok(());
        }

        // Update loading state
        self.is_loading.set(true);
        
        // Fetch next page
        let next_page = current_page + 1;
        let result = self
            .client
            .fetch_infinite_page::<T>(&self.key, next_page)
            .await?;

        // Update pages
        let result_clone = result.clone();
        self.pages.update(|pages| {
            if let Some(max_pages) = self.client.get_infinite_options(&self.key).max_pages {
                if pages.len() >= max_pages {
                    pages.remove(0); // Remove oldest page
                }
            }
            pages.push(result_clone);
        });

        // Update current page and has_next
        self.current_page.set(next_page);
        self.has_next.set(result.info.has_next);

        self.is_loading.set(false);
        Ok(())
    }

    /// Get the previous page of data
    pub async fn fetch_previous_page(&self) -> Result<(), QueryError> {
        let current_page = self.current_page.get();
        let has_prev = self.has_prev.get();
        
        if !has_prev {
            return Ok(());
        }

        // Update loading state
        self.is_loading.set(true);
        
        // Fetch previous page
        let prev_page = current_page.saturating_sub(1);
        let result = self
            .client
            .fetch_infinite_page::<T>(&self.key, prev_page)
            .await?;

        // Update pages
        let result_clone = result.clone();
        self.pages.update(|pages| {
            pages.insert(0, result_clone);
            
            if let Some(max_pages) = self.client.get_infinite_options(&self.key).max_pages {
                if pages.len() > max_pages {
                    pages.pop(); // Remove newest page
                }
            }
        });

        // Update current page and has_prev
        self.current_page.set(prev_page);
        self.has_prev.set(result.info.has_prev);

        self.is_loading.set(false);
        Ok(())
    }

    /// Refetch all pages
    pub async fn refetch(&self) -> Result<(), QueryError> {
        self.is_fetching.set(true);
        
        // Clear existing pages
        self.pages.set(Vec::new());
        self.current_page.set(0);
        self.has_next.set(true);
        self.has_prev.set(false);
        
        // Fetch first page
        let result = self
            .client
            .fetch_infinite_page::<T>(&self.key, 0)
            .await?;

        // Update state
        let result_clone = result.clone();
        self.pages.set(vec![result_clone]);
        self.has_next.set(result.info.has_next);
        self.is_stale.set(false);
        self.is_fetching.set(false);
        
        Ok(())
    }

    /// Invalidate and refetch
    pub async fn invalidate(&self) -> Result<(), QueryError> {
        // TODO: Implement invalidation when the method is available
        self.refetch().await
    }

    /// Remove all pages from cache
    pub async fn remove(&self) -> Result<(), QueryError> {
        self.client.remove_query(&self.key);
        self.pages.set(Vec::new());
        self.current_page.set(0);
        self.has_next.set(true);
        self.has_prev.set(false);
        Ok(())
    }

    /// Get all data from all pages as a flat vector
    pub fn get_all_data(&self) -> Vec<T> {
        self.pages
            .get()
            .iter()
            .flat_map(|page| page.data.clone())
            .collect()
    }

    /// Get data from a specific page
    pub fn get_page_data(&self, page: usize) -> Option<Vec<T>> {
        self.pages
            .get()
            .get(page)
            .map(|page| page.data.clone())
    }

    /// Get the total number of items across all pages
    pub fn get_total_count(&self) -> usize {
        self.pages
            .get()
            .iter()
            .map(|page| page.info.total)
            .sum()
    }
}

/// Hook for infinite queries with pagination
pub fn use_infinite_query<T, K, F, Fut>(
    key_fn: impl Fn() -> K + 'static,
    query_fn: impl Fn(usize) -> F + Clone + Send + Sync + 'static,
    _options: InfiniteQueryOptions,
) -> InfiniteQueryResult<T>
where
    T: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
    K: Into<QueryKey>,
    F: Future<Output = Result<Page<T>, QueryError>> + Send + 'static,
{
    let client = use_context::<Arc<QueryClient>>()
        .expect("use_infinite_query must be used within QueryClientProvider");

    let key = key_fn().into();
    let observer_id = client.register_infinite_observer(&key);

    // Create signals for state management
    let pages = RwSignal::new(Vec::new());
    let current_page = RwSignal::new(0);
    let has_next = RwSignal::new(true);
    let has_prev = RwSignal::new(false);
    let is_loading = RwSignal::new(false);
    let error = RwSignal::new(None);
    let is_stale = RwSignal::new(false);
    let is_fetching = RwSignal::new(false);

    // Initial fetch
    spawn_local(async move {
        is_loading.set(true);
        
        match query_fn(0).await {
            Ok(page) => {
                let page_clone = page.clone();
                pages.set(vec![page_clone]);
                has_next.set(page.info.has_next);
                is_stale.set(false);
            }
            Err(e) => {
                error.set(Some(e));
            }
        }
        
        is_loading.set(false);
    });

    InfiniteQueryResult {
        pages,
        current_page,
        has_next,
        has_prev,
        is_loading,
        error,
        is_stale,
        is_fetching,
        key,
        observer_id,
        client,
    }
}

/// Builder pattern for infinite query options
impl InfiniteQueryOptions {
    pub fn builder() -> InfiniteQueryOptionsBuilder {
        InfiniteQueryOptionsBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    struct TestItem {
        id: usize,
        name: String,
    }

    // Mock function removed to eliminate warnings

    #[test]
    fn test_infinite_query_options_builder() {
        let options = InfiniteQueryOptions::builder()
            .retry(RetryConfig::default())
            .keep_previous_data(false)
            .max_pages(Some(5))
            .build();

        assert_eq!(options.keep_previous_data, false);
        assert_eq!(options.max_pages, Some(5));
    }

    #[test]
    fn test_page_info() {
        let info = PageInfo {
            page: 1,
            per_page: 10,
            total: 100,
            has_next: true,
            has_prev: true,
        };

        assert_eq!(info.page, 1);
        assert_eq!(info.per_page, 10);
        assert_eq!(info.total, 100);
        assert!(info.has_next);
        assert!(info.has_prev);
    }
}
