# Common Patterns

This guide covers common patterns and best practices for using Leptos Query effectively.

## Table of Contents

- [Query Key Management](#query-key-management)
- [Error Handling](#error-handling)
- [Loading States](#loading-states)
- [Cache Management](#cache-management)
- [Optimistic Updates](#optimistic-updates)
- [Dependent Queries](#dependent-queries)
- [Conditional Queries](#conditional-queries)
- [Background Updates](#background-updates)
- [Performance Optimization](#performance-optimization)

## Query Key Management

### Consistent Key Structure

Establish a consistent pattern for your query keys:

```rust
// Good: Consistent structure
|| &["users", &user_id.to_string()][..]
|| &["users", &user_id.to_string(), "profile"][..]
|| &["users", &user_id.to_string(), "posts"][..]

// Good: Using QueryKey::new for complex keys
move || {
    let filters = serde_json::to_string(&filters).unwrap();
    QueryKey::new(&["users", &filters])
}

// Avoid: Inconsistent patterns
|| &["user", &user_id.to_string()][..]  // "user" vs "users"
|| &["users", "profile", &user_id.to_string()][..]  // Different order
```

### Key Factories

Create helper functions for generating consistent keys:

```rust
pub struct QueryKeys;

impl QueryKeys {
    pub fn users() -> &'static [&'static str] {
        &["users"]
    }
    
    pub fn user(id: u32) -> Vec<String> {
        vec!["users".to_string(), id.to_string()]
    }
    
    pub fn user_profile(id: u32) -> Vec<String> {
        vec!["users".to_string(), id.to_string(), "profile".to_string()]
    }
    
    pub fn user_posts(id: u32, filters: &PostFilters) -> Vec<String> {
        let mut key = vec!["users".to_string(), id.to_string(), "posts".to_string()];
        if let Ok(filters_str) = serde_json::to_string(filters) {
            key.push(filters_str);
        }
        key
    }
}

// Usage
let user_query = use_query(
    move || QueryKeys::user(user_id).as_slice(),
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
);
```

## Error Handling

### Structured Error Handling

Create custom error types and handle them consistently:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ApiError {
    NotFound(String),
    Unauthorized,
    ValidationError(Vec<String>),
    ServerError(String),
}

impl From<ApiError> for QueryError {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::NotFound(msg) => QueryError::Http { status: 404, message: msg },
            ApiError::Unauthorized => QueryError::Http { status: 401, message: "Unauthorized".to_string() },
            ApiError::ValidationError(errors) => QueryError::Custom(format!("Validation errors: {:?}", errors)),
            ApiError::ServerError(msg) => QueryError::Http { status: 500, message: msg },
        }
    }
}

// Error boundary component
#[component]
fn ErrorBoundary<F, T>(fallback: F, children: ChildrenFn) -> impl IntoView
where
    F: Fn(QueryError) -> T + 'static,
    T: IntoView + 'static,
{
    // Implementation for error boundary
    view! { <div>"Error boundary placeholder"</div> }
}

// Usage in components
{move || {
    if let Some(error) = user_query.error.get() {
        match error {
            QueryError::Http { status: 404, .. } => {
                view! { <div>"User not found"</div> }
            }
            QueryError::Http { status: 401, .. } => {
                view! { <div>"Please log in"</div> }
            }
            QueryError::Network { .. } => {
                view! { <div>"Network error - please check your connection"</div> }
            }
            _ => {
                view! { <div>"An unexpected error occurred"</div> }
            }
        }
    } else {
        view! { <div></div> }
    }
}}
```

### Global Error Handling

Set up global error handlers in your QueryClient configuration:

```rust
let config = QueryClientConfig::default()
    .with_default_retry(RetryConfig {
        max_attempts: 3,
        delay: RetryDelay::Exponential {
            initial: Duration::from_millis(1000),
            multiplier: 2.0,
            max: Duration::from_secs(30),
        },
        jitter: false,
    });

// In your mutation options
MutationOptions::default()
    .with_on_error(Box::new(|error, _vars, _ctx| {
        match error {
            QueryError::Http { status: 401, .. } => {
                // Redirect to login
                window().location().set_href("/login").unwrap();
            }
            QueryError::Network { .. } => {
                // Show offline notification
                show_toast("You're offline. Changes will be saved when you reconnect.");
            }
            _ => {
                // Log error for debugging
                log::error!("Mutation failed: {:?}", error);
            }
        }
    }))
```

## Loading States

### Granular Loading States

Use different loading states for better UX:

```rust
#[component]
fn UserProfile(user_id: u32) -> impl IntoView {
    let user_query = use_query(
        move || &["users", &user_id.to_string()][..],
        move || || async move { fetch_user(user_id).await },
        QueryOptions::default()
    );

    view! {
        <div>
            {move || {
                if user_query.is_loading.get() && user_query.data.get().is_none() {
                    // Initial loading
                    view! { <div class="loading-skeleton">"Loading user..."</div> }
                } else if user_query.is_fetching.get() && user_query.data.get().is_some() {
                    // Background refresh
                    view! { <div class="refreshing-indicator">"Refreshing..."</div> }
                } else if let Some(error) = user_query.error.get() {
                    // Error state
                    view! { <div class="error">"Error: " {error.to_string()}</div> }
                } else if let Some(user) = user_query.data.get() {
                    // Success state
                    view! {
                        <div class="user-profile">
                            <h3>{user.name}</h3>
                            <p>"Email: " {user.email}</p>
                        </div>
                    }
                } else {
                    // No data
                    view! { <div>"No user data available"</div> }
                }
            }}
        </div>
    }
}
```

### Loading Skeletons

Create reusable loading components:

```rust
#[component]
fn LoadingSkeleton() -> impl IntoView {
    view! {
        <div class="skeleton">
            <div class="skeleton-avatar"></div>
            <div class="skeleton-content">
                <div class="skeleton-title"></div>
                <div class="skeleton-text"></div>
                <div class="skeleton-text"></div>
            </div>
        </div>
    }
}

#[component]
fn UserProfileSkeleton() -> impl IntoView {
    view! {
        <div class="user-skeleton">
            <div class="skeleton-avatar"></div>
            <div class="skeleton-name"></div>
            <div class="skeleton-email"></div>
        </div>
    }
}
```

## Cache Management

### Strategic Cache Invalidation

Plan your cache invalidation strategy:

```rust
// Invalidate related queries when data changes
let create_user_mutation = use_mutation::<User, CreateUserRequest, (), _, _>(
    |request| async move { create_user(request).await },
    MutationOptions::default()
        .with_invalidates(&[
            &["users"][..],  // Invalidate user list
            &["stats", "users"][..],  // Invalidate user statistics
        ])
);

let update_user_mutation = use_mutation::<User, UpdateUserRequest, (), _, _>(
    |request| async move { update_user(request).await },
    MutationOptions::default()
        .with_invalidates(&[
            &["users", &request.id.to_string()][..],  // Invalidate specific user
            &["users"][..],  // Invalidate user list (if it shows updated data)
        ])
);

let delete_user_mutation = use_mutation::<(), u32, (), _, _>(
    |id| async move { delete_user(id).await },
    MutationOptions::default()
        .with_invalidates(&[
            &["users"][..],  // Invalidate user list
            &["stats", "users"][..],  // Invalidate user statistics
        ])
        .with_on_success(Box::new(|_, &id, _| {
            // Remove specific user from cache
            let client = use_context::<QueryClient>().unwrap();
            client.remove_queries(&QueryKeyPattern::Exact(QueryKey::new(&["users", &id.to_string()])));
        }))
);
```

### Cache Prefetching

Prefetch data for better UX:

```rust
#[component]
fn UserList() -> impl IntoView {
    let client = use_context::<QueryClient>().unwrap();
    
    let prefetch_user = move |user_id: u32| {
        let client = client.clone();
        spawn_local(async move {
            let _ = client.prefetch_query(
                &QueryKey::new(&["users", &user_id.to_string()]),
                || async move { fetch_user(user_id).await },
                QueryOptions::default()
            ).await;
        });
    };

    view! {
        <div>
            {move || {
                // Your user list rendering
                view! { <div>"User list"</div> }
            }}
        </div>
    }
}
```

## Optimistic Updates

### Optimistic Mutations

Provide instant feedback with optimistic updates:

```rust
#[component]
fn TodoItem(todo: Todo) -> impl IntoView {
    let optimistic_mutation = use_optimistic_mutation(
        QueryKey::new(&["todos", &todo.id.to_string()]),
        |todo_id| async move { toggle_todo(todo_id).await },
        |todo_id| {
            // Create optimistic data
            let mut optimistic_todo = todo.clone();
            optimistic_todo.completed = !optimistic_todo.completed;
            optimistic_todo
        }
    );

    let handle_toggle = move |_| {
        optimistic_mutation.mutate.call(todo.id);
    };

    view! {
        <div class="todo-item">
            <input
                type="checkbox"
                checked=move || todo.completed
                on:change=handle_toggle
            />
            <span class=todo.completed.then(|| "completed")>
                {todo.title}
            </span>
        </div>
    }
}
```

### Optimistic Lists

Handle optimistic updates for list operations:

```rust
#[component]
fn TodoList() -> impl IntoView {
    let add_todo_mutation = use_optimistic_mutation(
        QueryKey::new(&["todos"]),
        |title| async move { create_todo(title).await },
        |title| {
            // Create optimistic todo
            Todo {
                id: 0, // Will be replaced by real ID
                title,
                completed: false,
                created_at: chrono::Utc::now(),
            }
        }
    );

    let handle_add = move |title: String| {
        add_todo_mutation.mutate.call(title);
    };

    view! {
        <div>
            <AddTodoForm on_add=handle_add />
            <TodoListItems />
        </div>
    }
}
```

## Dependent Queries

### Sequential Dependencies

Handle queries that depend on other queries:

```rust
#[component]
fn UserDashboard(user_id: u32) -> impl IntoView {
    // First, fetch user data
    let user_query = use_query(
        move || &["users", &user_id.to_string()][..],
        move || || async move { fetch_user(user_id).await },
        QueryOptions::default()
    );

    // Then, fetch user's posts (depends on user data)
    let posts_query = use_query(
        move || {
            if let Some(user) = user_query.data.get() {
                &["users", &user.id.to_string(), "posts"][..]
            } else {
                &["empty"][..]
            }
        },
        move || {
            let user = user_query.data.get().unwrap();
            || async move { fetch_user_posts(user.id).await }
        },
        QueryOptions::default()
            .with_enabled(Signal::derive(move || user_query.data.get().is_some()))
    );

    // Finally, fetch user's friends (also depends on user data)
    let friends_query = use_query(
        move || {
            if let Some(user) = user_query.data.get() {
                &["users", &user.id.to_string(), "friends"][..]
            } else {
                &["empty"][..]
            }
        },
        move || {
            let user = user_query.data.get().unwrap();
            || async move { fetch_user_friends(user.id).await }
        },
        QueryOptions::default()
            .with_enabled(Signal::derive(move || user_query.data.get().is_some()))
    );

    view! {
        <div>
            {move || {
                if user_query.is_loading.get() {
                    view! { <div>"Loading user..."</div> }
                } else if let Some(user) = user_query.data.get() {
                    view! {
                        <div>
                            <h2>{user.name}</h2>
                            <div class="dashboard-content">
                                <PostsSection posts_query=posts_query />
                                <FriendsSection friends_query=friends_query />
                            </div>
                        </div>
                    }
                } else {
                    view! { <div>"User not found"</div> }
                }
            }}
        </div>
    }
}
```

## Conditional Queries

### Conditional Execution

Only run queries when certain conditions are met:

```rust
#[component]
fn UserProfile(user_id: Signal<Option<u32>>) -> impl IntoView {
    let user_query = use_query(
        move || {
            if let Some(id) = user_id.get() {
                &["users", &id.to_string()][..]
            } else {
                &["empty"][..]
            }
        },
        move || {
            let id = user_id.get().unwrap();
            || async move { fetch_user(id).await }
        },
        QueryOptions::default()
            .with_enabled(Signal::derive(move || user_id.get().is_some()))
    );

    view! {
        <div>
            {move || {
                if user_id.get().is_none() {
                    view! { <div>"Please select a user"</div> }
                } else if user_query.is_loading.get() {
                    view! { <div>"Loading..."</div> }
                } else if let Some(user) = user_query.data.get() {
                    view! { <div>{user.name}</div> }
                } else {
                    view! { <div>"No user data"</div> }
                }
            }}
        </div>
    }
}
```

### Dynamic Queries

Create queries that change based on user input:

```rust
#[component]
fn SearchResults() -> impl IntoView {
    let (search_term, set_search_term) = create_signal(String::new());
    let (filters, set_filters) = create_signal(SearchFilters::default());

    let search_query = use_query(
        move || {
            let term = search_term.get();
            let filters = filters.get();
            QueryKey::new(&["search", &term, &serde_json::to_string(&filters).unwrap()])
        },
        move || {
            let term = search_term.get();
            let filters = filters.get();
            || async move { search_items(term, filters).await }
        },
        QueryOptions::default()
            .with_enabled(Signal::derive(move || !search_term.get().is_empty()))
            .with_stale_time(Duration::from_secs(30)) // Cache search results
    );

    view! {
        <div>
            <input
                placeholder="Search..."
                on:input=move |ev| set_search_term.set(event_target_value(&ev))
            />
            <SearchFilters filters=filters set_filters=set_filters />
            <SearchResultsList query=search_query />
        </div>
    }
}
```

## Background Updates

### Automatic Refetching

Keep data fresh with background updates:

```rust
let user_query = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
        .with_refetch_interval(Duration::from_secs(30)) // Refetch every 30 seconds
        .with_refetch_on_window_focus() // Refetch when window gains focus
        .with_stale_time(Duration::from_secs(60)) // Consider data stale after 1 minute
);
```

### Real-time Updates

Combine with WebSocket or Server-Sent Events:

```rust
#[component]
fn ChatRoom(room_id: u32) -> impl IntoView {
    let messages_query = use_query(
        move || &["rooms", &room_id.to_string(), "messages"][..],
        move || || async move { fetch_messages(room_id).await },
        QueryOptions::default()
            .with_refetch_interval(Duration::from_secs(5)) // Poll every 5 seconds
    );

    // Set up WebSocket connection for real-time updates
    create_effect(move |_| {
        let room_id = room_id;
        spawn_local(async move {
            let mut ws = connect_to_chat_room(room_id).await;
            while let Some(message) = ws.recv().await {
                // Invalidate messages to trigger refetch
                let client = use_context::<QueryClient>().unwrap();
                client.invalidate_queries(&QueryKeyPattern::Exact(
                    QueryKey::new(&["rooms", &room_id.to_string(), "messages"])
                ));
            }
        });
    });

    view! {
        <div class="chat-room">
            <MessageList messages_query=messages_query />
            <MessageInput room_id=room_id />
        </div>
    }
}
```

## Performance Optimization

### Memoization

Use memoization to prevent unnecessary re-renders:

```rust
#[component]
fn ExpensiveComponent(data: Signal<Vec<ComplexData>>) -> impl IntoView {
    let processed_data = create_memo(move |_| {
        data.get().into_iter()
            .filter(|item| item.is_valid())
            .map(|item| item.process())
            .collect::<Vec<_>>()
    });

    view! {
        <div>
            {move || {
                processed_data.get().into_iter()
                    .map(|item| view! { <DataItem item /> })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}
```

### Lazy Loading

Implement lazy loading for large datasets:

```rust
#[component]
fn VirtualizedList() -> impl IntoView {
    let (page, set_page) = create_signal(1);
    let (page_size, _) = create_signal(20);

    let items_query = use_query(
        move || {
            let page = page.get();
            let page_size = page_size.get();
            &["items", &page.to_string(), &page_size.to_string()][..]
        },
        move || {
            let page = page.get();
            let page_size = page_size.get();
            || async move { fetch_items(page, page_size).await }
        },
        QueryOptions::default()
            .keep_previous_data() // Keep previous data while loading new page
    );

    let load_more = move |_| {
        set_page.update(|p| *p += 1);
    };

    view! {
        <div>
            <VirtualizedGrid items_query=items_query />
            <button on:click=load_more disabled=move || items_query.is_loading.get()>
                "Load More"
            </button>
        </div>
    }
}
```

### Query Deduplication

Ensure queries are properly deduplicated:

```rust
// Good: Same key for same data
let user_query_1 = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
);

let user_query_2 = use_query(
    move || &["users", &user_id.to_string()][..], // Same key
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
);

// Bad: Different keys for same data
let user_query_1 = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
);

let user_query_2 = use_query(
    move || &["user", &user_id.to_string()][..], // Different key!
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
);
```

These patterns will help you build robust, performant applications with Leptos Query. Remember to adapt them to your specific use case and always test thoroughly!
