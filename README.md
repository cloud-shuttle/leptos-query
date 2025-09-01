# Leptos Query RS

A powerful, type-safe data fetching and caching library for [Leptos 0.8](https://github.com/leptos-rs/leptos) applications, inspired by React Query/TanStack Query.

> **📦 Note**: This crate is named `leptos-query-rs` to distinguish it from the existing `leptos-query` crate on crates.io. Our implementation focuses on **Leptos 0.8 compatibility**, **comprehensive documentation**, and **AI-assisted development transparency**.

> **🤖 AI-Generated Code Notice**: This repository contains code that was primarily generated with the assistance of Large Language Models (LLMs). See [AI_GENERATED_DISCLAIMER.md](AI_GENERATED_DISCLAIMER.md) for full details about our AI-assisted development approach and quality assurance practices.

## ✨ Features

- **🚀 Leptos 0.8 Native**: Built specifically for Leptos 0.8's modern reactive primitives
- **🔄 Automatic Background Refetching**: Keep data fresh with intelligent refetching
- **🎯 Request Deduplication**: Multiple components requesting the same data? Only one request!
- **⚡ Optimistic Updates**: Update UI immediately, sync with server in background
- **🧠 Intelligent Caching**: Smart cache invalidation and background updates
- **🛡️ Error Handling**: Comprehensive error handling with retry logic
- **📱 Offline Support**: Work offline with cached data (planned)
- **🔧 DevTools**: Built-in development tools for debugging (planned)
- **🎨 Type Safety**: Full Rust type safety with compile-time guarantees

## 🚀 Quick Start

### Installation

```toml
[dependencies]
leptos-query-rs = "0.2"
leptos = "0.8"
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

async fn fetch_user(id: u32) -> Result<User, String> {
    // Your API call here
    Ok(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

#[component]
fn App() -> impl IntoView {
    provide_context(QueryClient::new());
    
    view! {
        <div>
            <UserProfile user_id=1 />
        </div>
    }
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
            {move || match user_query.data.get() {
                Some(user) => view! { <h1>{user.name}</h1> }.into_view(),
                None => view! { <p>"Loading..."</p> }.into_view(),
            }}
        </div>
    }
}
```

## 📚 Documentation

- [API Reference](docs/api-reference.md)
- [Quick Start Guide](docs/guides/quick-start.md)
- [Common Patterns](docs/guides/common-patterns.md)
- [Migration Guide](docs/migration.md)

## 🎯 Key Benefits

### **Leptos 0.8 Optimized**
- **Modern Signal API**: Uses Leptos 0.8's unified `Signal<T>` type
- **Zero-Copy Access**: Leverages guard-based `.read()` and `.write()` methods
- **No Scope Parameter**: Clean component signatures without explicit scope
- **Serialization Ready**: Built-in support for SSR with `Serialize`/`Deserialize`

### **Performance First**
- **Zero Runtime Overhead**: No compatibility layer overhead
- **Efficient Caching**: Smart cache invalidation and background updates
- **Memory Optimized**: Minimal memory footprint with efficient data structures

### **Developer Experience**
- **Familiar API**: React Query/TanStack Query inspired patterns
- **Type Safety**: Full Rust type safety with compile-time guarantees
- **Comprehensive Docs**: Extensive documentation and examples
- **Interactive Demo**: Live demo showcasing all features

## 🔧 Configuration

### Query Options

```rust
let query = use_query(
    move || &["users", &user_id.to_string()][..],
    move || || async move { fetch_user(user_id).await },
    QueryOptions::default()
        .with_stale_time(Duration::from_secs(60))
        .with_cache_time(Duration::from_secs(300))
        .with_refetch_interval(Duration::from_secs(30))
        .with_retry_count(3)
)
```

### Mutation Options

```rust
let mutation = use_mutation(
    move |user: User| async move { create_user(user).await },
    MutationOptions::default()
        .with_retry_count(3)
        .with_on_success(|| println!("User created successfully!"))
        .with_on_error(|error| println!("Error: {}", error))
);
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with specific features
cargo test --features ssr

# Run examples
cargo run --example basic_usage
```

## 📦 Examples

- [Basic Usage](examples/basic_usage.rs) - Simple query example
- [Advanced Usage](examples/advanced_usage.rs) - Complex patterns and mutations

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Leptos](https://github.com/leptos-rs/leptos) - The amazing Rust web framework
- [TanStack Query](https://tanstack.com/query) - Inspiration for the API design
- [React Query](https://react-query.tanstack.com/) - The original inspiration

## 📊 Status

- ✅ **Core Query/Mutation API** - Complete
- ✅ **Caching & Deduplication** - Complete  
- ✅ **Error Handling & Retries** - Complete
- ✅ **Type Safety** - Complete
- 🚧 **DevTools Integration** - In Progress
- 🚧 **Persistence Layer** - Planned
- 🚧 **Offline Support** - Planned

---

**Built with ❤️ for the Rust and Leptos communities**