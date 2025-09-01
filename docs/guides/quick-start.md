# Quick Start Guide

Get up and running with Leptos Query in just a few minutes!

## Installation

Add Leptos Query to your `Cargo.toml`:

```toml
[dependencies]
leptos = "0.6"
leptos_query = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
```

## Basic Setup

### 1. Set up the QueryClient

Wrap your app with `QueryClientProvider`:

```rust
use leptos::*;
use leptos_query::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <QueryClientProvider>
            <YourApp />
        </QueryClientProvider>
    }
}
```

### 2. Create a Simple Query

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

// Your API function
async fn fetch_user(id: u32) -> Result<User, QueryError> {
    // Simulate API call
    Ok(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

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
                if user_query.is_loading.get() {
                    view! { <div>"Loading..."</div> }
                } else if let Some(error) = user_query.error.get() {
                    view! { <div>"Error: " {error.to_string()}</div> }
                } else if let Some(user) = user_query.data.get() {
                    view! {
                        <div>
                            <h3>{user.name}</h3>
                            <p>"Email: " {user.email}</p>
                        </div>
                    }
                } else {
                    view! { <div>"No data"</div> }
                }
            }}
            <button on:click=move |_| user_query.refetch.call(())>
                "Refresh"
            </button>
        </div>
    }
}
```

## Adding Mutations

### 1. Create a Mutation

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

async fn create_user(request: CreateUserRequest) -> Result<User, QueryError> {
    // Your API call here
    Ok(User {
        id: 999, // Mock ID
        name: request.name,
        email: request.email,
    })
}

#[component]
fn CreateUserForm() -> impl IntoView {
    let create_user_mutation = use_mutation::<User, CreateUserRequest, (), _, _>(
        |request| async move { create_user(request).await },
        MutationOptions::default()
            .with_invalidates(&[&["users"][..]]) // Invalidate user queries
    );

    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());

    let handle_submit = move |_| {
        let request = CreateUserRequest {
            name: name.get(),
            email: email.get(),
        };
        create_user_mutation.mutate.call(request);
    };

    view! {
        <form on:submit=handle_submit>
            <input
                placeholder="Name"
                on:input=move |ev| set_name.set(event_target_value(&ev))
            />
            <input
                placeholder="Email"
                on:input=move |ev| set_email.set(event_target_value(&ev))
            />
            <button type="submit" disabled=move || create_user_mutation.is_loading.get()>
                {move || if create_user_mutation.is_loading.get() { "Creating..." } else { "Create User" }}
            </button>
        </form>
        
        {move || {
            if let Some(error) = create_user_mutation.error.get() {
                view! { <div style="color: red">"Error: " {error.to_string()}</div> }
            } else if let Some(user) = create_user_mutation.data.get() {
                view! { <div style="color: green">"Created user: " {user.name}</div> }
            } else {
                view! { <div></div> }
            }
        }}
    }
}
```

## Advanced Configuration

### 1. Custom QueryClient Configuration

```rust
#[component]
fn App() -> impl IntoView {
    let config = QueryClientConfig::default()
        .with_default_stale_time(Duration::from_secs(60))
        .with_default_cache_time(Duration::from_secs(300))
        .with_garbage_collection_interval(Duration::from_secs(60));

    view! {
        <QueryClientProvider config>
            <YourApp />
        </QueryClientProvider>
    }
}
```

### 2. Query Options

```rust
let user_query = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
        .with_stale_time(Duration::from_secs(60))
        .with_cache_time(Duration::from_secs(300))
        .with_retry(RetryConfig {
            max_attempts: 3,
            delay: RetryDelay::Exponential {
                initial: Duration::from_millis(100),
                multiplier: 2.0,
                max: Duration::from_secs(1),
            },
            jitter: false,
        })
        .with_refetch_interval(Duration::from_secs(30))
        .keep_previous_data()
);
```

### 3. Optimistic Updates

```rust
let optimistic_mutation = use_optimistic_mutation(
    QueryKey::new(&["users", "1"]),
    |request| async move { update_user(request).await },
    |request| User {
        id: 1,
        name: request.name.clone(),
        email: request.email.clone(),
    }
);
```

## Common Patterns

### 1. Conditional Queries

```rust
let user_query = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
        .with_enabled(Signal::derive(move || user_id.get() > 0))
);
```

### 2. Dependent Queries

```rust
let user_query = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
);

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
```

### 3. Cache Invalidation

```rust
let client = use_context::<QueryClient>().unwrap();

// Invalidate all user queries
client.invalidate_queries(&QueryKeyPattern::Prefix(QueryKey::new(&["users"])));

// Invalidate specific user
client.invalidate_queries(&QueryKeyPattern::Exact(QueryKey::new(&["users", "1"])));

// Invalidate queries containing "profile"
client.invalidate_queries(&QueryKeyPattern::Contains("profile".to_string()));
```

## Error Handling

```rust
let user_query = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
        .with_on_error(Box::new(|error| {
            log::error!("Query failed: {:?}", error);
        }))
);

// In your view
{move || {
    if let Some(error) = user_query.error.get() {
        match error {
            QueryError::Network { message } => {
                view! { <div>"Network error: " {message}</div> }
            }
            QueryError::Http { status, message } => {
                view! { <div>"HTTP {status}: " {message}</div> }
            }
            QueryError::Timeout { duration } => {
                view! { <div>"Request timed out after " {duration.as_secs()} " seconds"</div> }
            }
            _ => {
                view! { <div>"Error: " {error.to_string()}</div> }
            }
        }
    } else {
        view! { <div></div> }
    }
}}
```

## Next Steps

- Check out the [API Reference](../api-reference.md) for complete documentation
- Explore [Common Patterns](./common-patterns.md) for advanced usage
- See [Examples](../../examples/) for real-world implementations
- Learn about [Performance Optimization](./performance.md) for production apps

Happy coding! 🚀
