# Leptos Query

A React Query inspired data fetching library for Leptos applications, providing powerful caching, background updates, and error handling capabilities.

**🚀 Now fully compatible with Leptos 0.8!**

## Features

- **Declarative Data Fetching**: Write queries as simple async functions
- **Automatic Caching**: Built-in cache with configurable stale times
- **Background Updates**: Keep data fresh with background refetching
- **Error Handling**: Comprehensive error handling with retry logic
- **Type Safety**: Full type safety with Rust's type system
- **WASM Compatible**: Works in both native and web environments
- **Leptos 0.8 Ready**: Full compatibility with the latest Leptos framework

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
leptos-query = "0.3.0"
leptos = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Usage

```rust
use leptos::*;
use leptos_query::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

async fn fetch_user(id: u32) -> Result<User, QueryError> {
    // Your async function here
    Ok(User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    })
}

#[component]
fn UserProfile(user_id: u32) -> impl IntoView {
    let user_query = use_query(
        move || QueryKey::new(&["user", &user_id.to_string()]),
        move || async move { fetch_user(user_id).await },
        QueryOptions::default(),
    );

    view! {
        <div>
            {move || match user_query.data.get() {
                Some(user) => view! { <div>"User: " {user.name}</div> },
                None if user_query.is_loading.get() => view! { <div>"Loading..."</div> },
                None => view! { <div>"No user found"</div> },
            }}
        </div>
    }
}
```

### Setup

Wrap your app with the `QueryClientProvider`:

```rust
#[component]
fn App() -> impl IntoView {
    view! {
        <QueryClientProvider>
            <UserProfile user_id=1/>
        </QueryClientProvider>
    }
}
```

## API Reference

### Query Hook

```rust
pub fn use_query<T, F, Fut>(
    key_fn: F,
    query_fn: impl Fn() -> Fut + Clone + Send + Sync + 'static,
    options: QueryOptions,
) -> QueryResult<T>
```

**Parameters:**
- `key_fn`: Function that returns a `QueryKey` for caching
- `query_fn`: Async function that fetches the data
- `options`: Configuration options for the query

**Returns:**
- `QueryResult<T>`: Object containing data, loading state, and actions

### Query Options

```rust
let options = QueryOptions::default()
    .with_stale_time(Duration::from_secs(60))
    .with_cache_time(Duration::from_secs(300))
    .with_refetch_interval(Duration::from_secs(30));
```

### Query Result

```rust
pub struct QueryResult<T> {
    pub data: Signal<Option<T>>,           // The query data
    pub error: Signal<Option<QueryError>>, // Error if any
    pub is_loading: Signal<bool>,          // Whether loading
    pub is_success: Signal<bool>,          // Whether succeeded
    pub is_error: Signal<bool>,            // Whether failed
    pub status: Signal<QueryStatus>,       // Current status
    pub refetch: Callback<()>,             // Refetch function
}
```

### Mutation Hook

```rust
pub fn use_mutation<TData, TError, TVariables, F, Fut>(
    mutation_fn: F,
    options: MutationOptions,
) -> MutationResult<TData, TError, TVariables>
```

**Example:**

```rust
async fn create_user(user: CreateUserRequest) -> Result<User, QueryError> {
    // Your mutation logic here
    Ok(User { /* ... */ })
}

let mutation = use_mutation(
    create_user,
    MutationOptions::default()
        .invalidate_queries(vec![QueryKeyPattern::Exact(QueryKey::from("users"))]),
);

// Execute mutation
mutation.mutate.call(CreateUserRequest { /* ... */ });
```

### Error Handling

The library provides comprehensive error handling:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryError {
    NetworkError(String),
    SerializationError(String),
    DeserializationError(String),
    TimeoutError(String),
    GenericError(String),
}
```

### Retry Configuration

```rust
let retry_config = RetryConfig::new(3, Duration::from_secs(1))
    .with_max_delay(Duration::from_secs(30))
    .with_fixed_delay()
    .no_network_retry();
```

## Advanced Features

### Query Keys

Query keys are used for caching and invalidation:

```rust
// Simple key
QueryKey::from("users")

// Compound key
QueryKey::from(["users", user_id.to_string()])

// With parameters
QueryKey::from_parts(&[user_id, filter]).unwrap()
```

### Cache Invalidation

```rust
// Invalidate specific queries
client.remove_query(&QueryKey::from("users"));

// Invalidate by pattern
client.invalidate_queries(&QueryKeyPattern::Prefix(QueryKey::from("users")));
```

### Background Refetching

```rust
let options = QueryOptions::default()
    .with_refetch_interval(Duration::from_secs(30));
```

## Examples

See the `examples/` directory for complete working examples:

- `basic.rs`: Basic query usage
- `mutations.rs`: Mutation examples
- `caching.rs`: Advanced caching examples

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.