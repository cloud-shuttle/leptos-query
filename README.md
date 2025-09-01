# Leptos Query

[![CI](https://github.com/cloud-shuttle/leptos-query/workflows/CI/badge.svg)](https://github.com/cloud-shuttle/leptos-query/actions?query=workflow%3ACI)
[![Crates.io](https://img.shields.io/crates/v/leptos-query)](https://crates.io/crates/leptos-query)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/leptos-query)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)

A powerful, type-safe data fetching and caching library for [Leptos](https://github.com/leptos-rs/leptos) applications, inspired by React Query/TanStack Query.

> **📦 Note**: This crate is named `leptos-query-rs` to distinguish it from the existing `leptos-query` crate on crates.io. Our implementation focuses on **comprehensive documentation**, **AI-assisted development transparency**, and **future-ready architecture**.

> **🤖 AI-Generated Code Notice**: This repository contains code that was primarily generated with the assistance of Large Language Models (LLMs). See [AI_GENERATED_DISCLAIMER.md](AI_GENERATED_DISCLAIMER.md) for full details about our AI-assisted development approach and quality assurance practices.

## ✨ Features

- **🔄 Automatic Caching**: Intelligent cache management with configurable stale times
- **🔄 Background Refetching**: Keep data fresh with automatic background updates
- **🔄 Request Deduplication**: Prevent duplicate requests for the same data
- **🔄 Optimistic Updates**: Update UI immediately while mutations are in flight
- **🔄 Retry Logic**: Configurable retry strategies with exponential backoff
- **🔄 Type Safety**: Full TypeScript-like type safety with Rust's type system
- **🔄 Suspense Support**: Built-in support for React Suspense patterns
- **🔄 Error Handling**: Comprehensive error types and handling strategies

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
leptos-query-rs = "0.1.0"
```

### Basic Usage

```rust
use leptos::*;
use leptos_query_rs::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Mock API function
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
            .with_stale_time(Duration::from_secs(60))
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

// Set up the app with QueryClient
#[component]
fn App() -> impl IntoView {
    view! {
        <QueryClientProvider>
            <UserProfile user_id=1 />
        </QueryClientProvider>
    }
}
```

## 🛠️ Development

This project includes a comprehensive Makefile for common development tasks:

```bash
# Show all available commands
make help

# Common development workflow
make dev          # Format, lint, and test
make test         # Run all tests
make doc          # Generate documentation
make release      # Build release version
```

For a complete list of commands, run `make help`.

## 📚 Documentation

- **[API Reference](./docs/api-reference.md)** - Complete API documentation
- **[Guides](./docs/guides/)** - Usage guides and tutorials
- **[Examples](./examples/)** - Code examples and patterns
- **[Migration Guide](./docs/migration.md)** - Coming from React Query?

## 🔧 Configuration

### Leptos Version Compatibility

Leptos Query supports multiple Leptos versions through feature flags:

```toml
# For Leptos 0.6 (current)
[dependencies]
leptos-query-rs = { version = "0.1", features = ["leptos-0-6"] }
leptos = "0.6"

# For Leptos 0.8 (planned)
[dependencies]
leptos-query-rs = { version = "0.1", features = ["leptos-0-8"] }
leptos = "0.8"

> **Note**: Leptos 0.8 support is planned for future releases. The library currently supports Leptos 0.6 with infrastructure prepared for future version compatibility.
```

You can also detect the current Leptos version at runtime:

```rust
use leptos_query::compat::leptos_version;

let version = leptos_version();
println!("Using Leptos version: {}", version.as_str());
```

### QueryClient Setup

```rust
use leptos_query_rs::*;

#[component]
fn App() -> impl IntoView {
    let config = QueryClientConfig::default()
        .with_default_stale_time(Duration::from_secs(60))
        .with_default_cache_time(Duration::from_secs(300))
        .with_garbage_collection_interval(Duration::from_secs(60));

    view! {
        <QueryClientProvider config>
            // Your app components
        </QueryClientProvider>
    }
}
```

### Query Options

```rust
let query = use_query(
    || &["users", "1"][..],
    || || async move { fetch_user(1).await },
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
        .with_suspense()
);
```

## 🔄 Mutations

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

async fn create_user(request: CreateUserRequest) -> Result<User, QueryError> {
    // API call to create user
    Ok(User { id: 999, name: request.name, email: request.email })
}

#[component]
fn CreateUserForm() -> impl IntoView {
    let create_user_mutation = use_mutation::<User, CreateUserRequest, (), _, _>(
        |request| async move { create_user(request).await },
        MutationOptions::default()
            .with_invalidates(&[&["users"][..]])
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
    }
}
```

## 🎯 Key Concepts

### Query Keys
Query keys are used to identify and cache data. They can be simple strings or complex arrays:

```rust
// Simple key
|| &["users"][..]

// Complex key with parameters
|| &["users", &user_id.to_string(), "profile"][..]

// Using QueryKey::new for dynamic keys
move || {
    let id_str = user_id.get().to_string();
    QueryKey::new(&["users", &id_str])
}
```

### Cache Invalidation
Invalidate related queries when data changes:

```rust
// Invalidate all user queries
client.invalidate_queries(&QueryKeyPattern::Prefix(QueryKey::new(&["users"])));

// Invalidate specific user
client.invalidate_queries(&QueryKeyPattern::Exact(QueryKey::new(&["users", "1"])));

// Invalidate queries containing "profile"
client.invalidate_queries(&QueryKeyPattern::Contains("profile".to_string()));
```

### Error Handling
Comprehensive error types for different scenarios:

```rust
match error {
    QueryError::Network { message } => {
        // Handle network errors
    }
    QueryError::Http { status, message } => {
        // Handle HTTP errors
    }
    QueryError::Timeout { duration } => {
        // Handle timeouts
    }
    QueryError::Serialization(message) => {
        // Handle serialization errors
    }
    QueryError::Deserialization(message) => {
        // Handle deserialization errors
    }
    QueryError::Custom(message) => {
        // Handle custom errors
    }
}
```

## 🎮 Interactive Demo

Try our **[interactive demo](demo/static-demo.html)** to see leptos-query in action! The demo showcases:

- 🔄 **Automatic Caching** - Watch data get cached and shared across components
- 🛡️ **Error Handling** - Test error scenarios with auto-retry logic
- 🔄 **Mutations** - Update data with optimistic UI updates
- 📄 **Infinite Queries** - Load paginated data with infinite scrolling
- ⚡ **Background Refetch** - Keep data fresh automatically
- 📊 **Real-time Status** - Monitor cache, loading, and error states

The demo is fully interactive and runs in your browser - no installation required!

## 🚀 Advanced Features

### Optimistic Updates
Update the UI immediately while mutations are in flight:

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

### Background Refetching
Keep data fresh with automatic background updates:

```rust
let query = use_query(
    || &["users", "1"][..],
    || || async move { fetch_user(1).await },
    QueryOptions::default()
        .with_refetch_interval(Duration::from_secs(30))
        .with_refetch_on_window_focus()
);
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](./CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [React Query](https://tanstack.com/query) and [TanStack Query](https://tanstack.com/query)
- Built for the amazing [Leptos](https://github.com/leptos-rs/leptos) framework
- Thanks to the Rust and WebAssembly communities

---

**Ready to build amazing reactive applications? Get started with Leptos Query today!** 🚀