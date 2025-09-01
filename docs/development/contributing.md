# Contributing Guide

Thank you for your interest in contributing to Leptos Query! This document outlines the development process, coding standards, and guidelines for contributors.

## 🚀 Getting Started

### Prerequisites

- **Rust**: 1.70.0 or higher
- **Node.js**: For running examples and testing
- **wasm-pack**: For WebAssembly testing
- **cargo-leptos**: For Leptos-specific tooling

### Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/CloudShuttle/leptos-query.git
   cd leptos-query
   ```

2. **Install tools**:
   ```bash
   # Install wasm-pack for WASM testing
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   
   # Install cargo-leptos
   cargo install cargo-leptos
   
   # Install additional tools
   cargo install cargo-tarpaulin  # Code coverage
   cargo install cargo-audit      # Security auditing
   ```

3. **Verify setup**:
   ```bash
   cargo check --all-features
   cargo test --all-features
   wasm-pack test --headless --chrome
   ```

## 🏗️ Project Structure

```
leptos-query/
├── src/
│   ├── lib.rs              # Public API and re-exports
│   ├── client/             # QueryClient implementation
│   │   ├── mod.rs
│   │   └── config.rs
│   ├── query/              # Query hooks
│   │   └── mod.rs
│   ├── mutation/           # Mutation hooks
│   │   └── mod.rs
│   ├── cache/              # Cache implementation
│   │   ├── mod.rs
│   │   ├── entry.rs
│   │   └── gc.rs
│   ├── retry/              # Retry logic and error handling
│   │   └── mod.rs
│   ├── dedup/              # Request deduplication
│   │   └── mod.rs
│   ├── persistence/        # Storage adapters
│   │   └── mod.rs
│   └── devtools/           # Development tools
│       └── mod.rs
├── tests/
│   ├── unit/               # Unit tests
│   ├── integration/        # Integration tests
│   └── e2e/               # End-to-end tests
├── benches/               # Performance benchmarks
├── examples/              # Example applications
├── docs/                  # Documentation
└── .github/               # CI/CD workflows
```

## 📝 Coding Standards

### Rust Code Style

We follow standard Rust conventions with some additional guidelines:

#### Naming Conventions

- **Types**: `PascalCase` (e.g., `QueryClient`, `CacheEntry`)
- **Functions**: `snake_case` (e.g., `use_query`, `get_cache_entry`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_CACHE_TIME`)
- **Generics**: Single capital letter or descriptive (e.g., `T`, `TData`)

#### Module Organization

```rust
// Good: Clear, focused modules
mod cache {
    mod entry;
    mod gc;
    pub use entry::*;
}

// Bad: Everything in one module
mod everything;
```

#### Documentation Requirements

All public APIs must have doc comments with examples:

```rust
/// Fetches data from the server with automatic caching.
/// 
/// # Examples
/// 
/// ```rust
/// let users = use_query(
///     || ["users"],
///     || fetch_users(),
///     QueryOptions::default()
/// );
/// ```
/// 
/// # Arguments
/// 
/// * `key_fn` - Function that returns the query key
/// * `query_fn` - Async function that fetches the data
/// * `options` - Configuration options for the query
pub fn use_query<T, K, F, Fut>(/* ... */) -> QueryResult<T> {
    // Implementation
}
```

#### Error Handling

- Use `Result<T, QueryError>` for fallible operations
- Provide context in error messages
- Never panic in library code
- Use the `?` operator for error propagation

```rust
// Good
pub fn set_query_data<T>(&self, key: &QueryKey, data: T) -> Result<(), QueryError> 
where
    T: Serialize + 'static,
{
    let serialized = SerializedData::serialize(&data)
        .map_err(|e| QueryError::Serialization(format!("Failed to serialize data: {}", e)))?;
    
    // ... rest of implementation
}

// Bad
pub fn set_query_data<T>(&self, key: &QueryKey, data: T) {
    let serialized = SerializedData::serialize(&data).unwrap(); // Don't panic!
    // ...
}
```

### Testing Standards

#### Unit Tests

Every public function should have unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_key_creation() {
        let key = QueryKey::new(["users", "123"]);
        assert_eq!(key.segments, vec!["users", "123"]);
    }
    
    #[test]
    fn test_query_key_pattern_matching() {
        let key = QueryKey::new(["users", "123", "posts"]);
        let pattern = QueryKeyPattern::Prefix(QueryKey::new(["users"]));
        assert!(key.matches_pattern(&pattern));
    }
}
```

#### Integration Tests

Test interactions between components:

```rust
#[cfg(test)]
mod integration_tests {
    use leptos_test::*;
    use leptos_query::*;
    
    #[test]
    fn test_query_cache_integration() {
        create_test_runtime(|| {
            let client = QueryClient::new(Default::default());
            
            // Test full query lifecycle
            let key = QueryKey::new(["test"]);
            let data = "test data";
            
            client.set_query_data(&key, data).unwrap();
            let retrieved = client.get_query_data::<&str>(&key);
            
            assert_eq!(retrieved, Some(data));
        });
    }
}
```

#### Property Tests

Use `proptest` for invariant testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_serialization_roundtrip(data: Vec<u8>) {
        let serialized = SerializedData::serialize(&data).unwrap();
        let deserialized: Vec<u8> = serialized.deserialize().unwrap();
        prop_assert_eq!(data, deserialized);
    }
}
```

## 🔄 Development Workflow

### Git Workflow

We use a **Git Flow** inspired workflow:

#### Branches

- **`main`** - Stable releases only
- **`develop`** - Integration branch for next release
- **`feature/*`** - New features
- **`fix/*`** - Bug fixes
- **`docs/*`** - Documentation updates
- **`perf/*`** - Performance improvements

#### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): subject

body

footer
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix  
- `docs`: Documentation only
- `style`: Code style changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding tests
- `chore`: Maintenance tasks

**Examples**:
```
feat(query): add infinite query support

Implement use_infinite_query hook with pagination support.
Includes automatic page param management and background
refetching of stale pages.

Closes #123
```

```
fix(cache): prevent memory leak in observer cleanup

Observer references were not being properly cleaned up
when components unmounted, causing memory leaks in
long-running applications.

Fixes #456
```

### Pull Request Process

1. **Create Feature Branch**:
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/your-feature-name
   ```

2. **Make Changes**:
   - Write code following our style guide
   - Add tests for new functionality
   - Update documentation
   - Run tests locally

3. **Commit Changes**:
   ```bash
   git add .
   git commit -m "feat(scope): your feature description"
   ```

4. **Push and Create PR**:
   ```bash
   git push origin feature/your-feature-name
   # Create PR on GitHub
   ```

5. **PR Requirements**:
   - [ ] All tests pass
   - [ ] Code coverage >90% for new code
   - [ ] Documentation updated
   - [ ] Changelog entry added
   - [ ] No breaking changes (unless major version)

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run specific test categories
cargo test --test unit
cargo test --test integration

# Run WASM tests
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox

# Run with coverage
cargo tarpaulin --all-features --out Html
```

### Writing Tests

#### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Unit tests for individual functions
    mod unit {
        use super::*;
        
        #[test]
        fn test_individual_function() {
            // Test implementation
        }
    }
    
    // Integration tests for component interaction
    mod integration {
        use super::*;
        use leptos_test::*;
        
        #[test]
        fn test_component_integration() {
            create_test_runtime(|| {
                // Test implementation
            });
        }
    }
}
```

#### Mock Data

Create reusable test fixtures:

```rust
// tests/fixtures/mod.rs
pub struct TestData {
    pub users: Vec<User>,
    pub posts: Vec<Post>,
}

impl TestData {
    pub fn new() -> Self {
        Self {
            users: vec![
                User { id: 1, name: "John".into(), email: "john@test.com".into() },
                User { id: 2, name: "Jane".into(), email: "jane@test.com".into() },
            ],
            posts: vec![
                Post { id: 1, user_id: 1, title: "Test Post".into(), body: "Content".into() },
            ],
        }
    }
}
```

#### Async Testing

```rust
#[tokio::test]
async fn test_async_operation() {
    let client = QueryClient::new(Default::default());
    
    let result = client.prefetch_query(
        QueryKey::new(["test"]),
        || async { Ok("test data".to_string()) },
        None,
    ).await;
    
    assert!(result.is_ok());
}
```

### Performance Testing

Write benchmarks for critical paths:

```rust
// benches/cache_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use leptos_query::*;

fn cache_operations(c: &mut Criterion) {
    let client = QueryClient::new(Default::default());
    
    c.bench_function("cache_set", |b| {
        b.iter(|| {
            let key = QueryKey::new(["bench", "key"]);
            client.set_query_data(&key, black_box("test data")).unwrap();
        })
    });
    
    c.bench_function("cache_get", |b| {
        // Setup
        let key = QueryKey::new(["bench", "key"]);
        client.set_query_data(&key, "test data").unwrap();
        
        b.iter(|| {
            black_box(client.get_query_data::<&str>(&key));
        })
    });
}

criterion_group!(benches, cache_operations);
criterion_main!(benches);
```

## 📚 Documentation

### Types of Documentation

1. **API Documentation**: Rustdoc comments in source code
2. **User Guides**: Markdown files in `docs/user/`
3. **Developer Docs**: Technical documentation in `docs/development/`
4. **Examples**: Working examples in `examples/`

### Writing Documentation

#### API Documentation

```rust
/// Executes a query with automatic caching and background refetching.
///
/// This hook provides a declarative way to fetch, cache, and synchronize
/// server data in your Leptos components. It automatically handles loading
/// states, error states, and background refetching.
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// let users = use_query(
///     || ["users"],
///     || fetch_users(),
///     QueryOptions::default()
/// );
/// ```
///
/// With reactive keys:
/// ```rust
/// let user = use_query(
///     move || ["users", user_id.get().to_string()],
///     move || fetch_user(user_id.get()),
///     QueryOptions::default()
/// );
/// ```
///
/// # Arguments
///
/// * `key_fn` - A function that returns the query key. Must be reactive.
/// * `query_fn` - An async function that fetches the data.
/// * `options` - Configuration options for the query behavior.
///
/// # Returns
///
/// Returns a `QueryResult<T>` containing:
/// - `data` - The fetched data (None while loading)
/// - `error` - Any error that occurred during fetching
/// - `is_loading` - Whether the query is loading for the first time
/// - `is_fetching` - Whether the query is currently fetching (including background)
/// - `refetch` - Function to manually refetch the data
/// - And more state indicators...
///
/// # Panics
///
/// Panics if no `QueryClient` is provided in the Leptos context.
pub fn use_query<T, K, F, Fut>(/* ... */) -> QueryResult<T> {
    // Implementation
}
```

#### User Guides

Write clear, example-driven guides:

```markdown
# Working with Mutations

Mutations in Leptos Query handle data modifications like creating, updating,
or deleting resources on your server.

## Basic Usage

```rust
let create_user = use_mutation(
    |user: CreateUserDto| async move {
        api::create_user(user).await
    },
    MutationOptions::default()
);

// Trigger the mutation
create_user.mutate.call(CreateUserDto {
    name: "John Doe".into(),
    email: "john@example.com".into(),
});
```

## Automatic Invalidation

Mutations can automatically invalidate related queries:

```rust
let create_user = use_mutation(
    |user: CreateUserDto| async move { /* ... */ },
    MutationOptions {
        invalidates: vec![
            QueryKeyPattern::Prefix(QueryKey::new(["users"])),
        ],
        ..Default::default()
    }
);
```
```

## 🐛 Debugging

### Logging

Use the `log` crate for debugging:

```rust
use log::{debug, info, warn, error};

pub fn cache_operation(&self, key: &QueryKey) {
    debug!("Cache operation for key: {:?}", key);
    
    match self.perform_operation(key) {
        Ok(result) => {
            info!("Cache operation successful for key: {:?}", key);
            result
        },
        Err(e) => {
            error!("Cache operation failed for key: {:?}, error: {}", key, e);
            return Err(e);
        }
    }
}
```

### DevTools

Enable development tools:

```rust
#[cfg(debug_assertions)]
use leptos_query::QueryDevTools;

#[component]
fn App() -> impl IntoView {
    view! {
        <div>
            <YourApp />
            
            // Only in development
            #[cfg(debug_assertions)]
            <QueryDevTools />
        </div>
    }
}
```

## 🚀 Release Process

### Version Bumping

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0 → 2.0.0): Breaking changes
- **MINOR** (1.0.0 → 1.1.0): New features, backwards compatible
- **PATCH** (1.0.0 → 1.0.1): Bug fixes, backwards compatible

### Changelog

Update `CHANGELOG.md` with each change:

```markdown
# Changelog

## [Unreleased]

### Added
- New infinite query hook
- Optimistic update support

### Changed
- Improved error messages

### Fixed
- Memory leak in cache cleanup

## [0.1.0] - 2024-09-01

### Added
- Initial release
- Basic query and mutation hooks
- Cache management
```

## 💡 Contributing Ideas

Looking for ways to contribute? Here are some ideas:

### Code Contributions
- 🐛 **Bug Fixes**: Check our [issues](https://github.com/CloudShuttle/leptos-query/issues)
- ✨ **New Features**: Implement features from our [roadmap](https://github.com/CloudShuttle/leptos-query/projects)
- 🚀 **Performance**: Optimize critical paths
- 🧪 **Testing**: Improve test coverage

### Documentation Contributions
- 📚 **Guides**: Write tutorials and how-to guides
- 🎯 **Examples**: Create example applications
- 🔧 **API Docs**: Improve inline documentation
- 🌍 **Translations**: Translate documentation

### Community Contributions
- 💬 **Support**: Help users in discussions
- 🎤 **Speaking**: Give talks about Leptos Query
- ✍️ **Writing**: Blog about your experience
- 🎨 **Design**: Improve UI/UX of DevTools

## 📞 Getting Help

- **Discord**: Join the Leptos community Discord
- **GitHub Issues**: For bugs and feature requests
- **Discussions**: For questions and community help

## 📄 License

By contributing to Leptos Query, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

Thank you for contributing to Leptos Query! 🦀✨