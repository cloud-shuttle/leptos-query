# Release Notes - leptos-query v0.1.0

## 🎉 Initial Release

This is the first official release of `leptos-query`, a powerful, type-safe data fetching and caching library for Leptos applications, inspired by React Query/TanStack Query.

## ✨ Features

### Core Functionality
- **`use_query` Hook**: Reactive data fetching with automatic caching, background updates, and error handling
- **`use_mutation` Hook**: Data mutations with optimistic updates and cache invalidation
- **QueryClient**: Global client for managing queries, mutations, and cache state
- **QueryCache**: In-memory cache with automatic garbage collection
- **Request Deduplication**: Prevents duplicate requests for the same data
- **Retry Logic**: Configurable retry strategies with exponential backoff

### Advanced Features
- **Query Keys**: Flexible key system supporting patterns and invalidation
- **Error Handling**: Comprehensive error types with retryability
- **Loading States**: Built-in loading, error, and success states
- **Background Refetching**: Automatic data updates in the background
- **Cache Persistence**: Optional persistence layer for offline support
- **Leptos Compatibility**: Support for both Leptos 0.6 and 0.8 (future-ready)

### Developer Experience
- **Type Safety**: Full Rust type safety with compile-time guarantees
- **Reactive**: Built on Leptos signals for reactive UI updates
- **Async/Await**: Native async/await support
- **Comprehensive Testing**: Full test suite with integration tests
- **Documentation**: Complete API documentation and guides

## 🚀 Quick Start

```rust
use leptos::*;
use leptos_query::*;

#[component]
fn UserProfile() -> impl IntoView {
    let user_query = use_query(
        || QueryKey::new(&["users", "1"]),
        || || async move {
            // Fetch user data
            fetch_user(1).await
        },
        QueryOptions::default(),
    );

    view! {
        <div>
            {move || match user_query.data.get() {
                Some(user) => view! { <h1>{user.name}</h1> },
                None => view! { <p>"Loading..."</p> },
            }}
        </div>
    }
}
```

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
leptos-query = "0.1.0"
leptos = "0.6"
```

## 🔧 Configuration

### Feature Flags
- `default`: Includes `leptos-0-6` and `csr` features
- `leptos-0-6`: Leptos 0.6 compatibility (default)
- `leptos-0-8`: Leptos 0.8 compatibility (future)
- `csr`: Client-side rendering support
- `ssr`: Server-side rendering support
- `hydrate`: Hydration support
- `persistence`: Cache persistence layer
- `offline`: Offline support (includes persistence)

## 📚 Documentation

- **[API Reference](docs/api-reference.md)**: Complete API documentation
- **[Quick Start Guide](docs/guides/quick-start.md)**: Get started in minutes
- **[Migration Guide](docs/migration.md)**: Migrate from React Query
- **[Common Patterns](docs/guides/common-patterns.md)**: Best practices and patterns

## 🧪 Testing

All tests are passing:
- ✅ **9/9 Library Tests**: Core functionality tests
- ✅ **7/7 Integration Tests**: End-to-end functionality tests
- ✅ **Example Compilation**: Basic usage example compiles successfully

## 🔮 Future Plans

### Leptos 0.8 Compatibility
The library includes a compatibility layer that will make upgrading to Leptos 0.8 seamless when it's released. The compatibility layer uses feature flags to detect the Leptos version and adapt accordingly.

### Planned Features
- Infinite queries for pagination
- Query prefetching
- Advanced cache strategies
- DevTools integration
- Performance monitoring

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Inspired by [React Query](https://tanstack.com/query) and [TanStack Query](https://tanstack.com/query)
- Built for the [Leptos](https://leptos.dev) framework
- Thanks to the Rust and Leptos communities

---

**Repository**: https://github.com/cloud-shuttle/leptos-query  
**Documentation**: https://docs.rs/leptos-query  
**Issues**: https://github.com/cloud-shuttle/leptos-query/issues
