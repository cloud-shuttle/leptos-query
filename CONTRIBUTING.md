# Contributing to Leptos Query

Thank you for your interest in contributing to leptos-query! This guide will help you get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). By participating, you agree to uphold this code.

## Getting Started

### Prerequisites

- Rust 1.70+ (latest stable recommended)
- Node.js 18+ (for Playwright tests)
- Git

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/leptos-query-rs.git
   cd leptos-query-rs
   ```

3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/cloud-shuttle/leptos-query-rs.git
   ```

## Development Setup

### 1. Install Dependencies

```bash
# Install Rust dependencies
cargo build

# Install Node.js dependencies for testing
cd demo
npm install
cd ..
```

### 2. Run Tests

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test unit_tests
cargo test --test integration_tests
cargo test --test property_tests

# Run Playwright tests
cd demo
npx playwright test
cd ..
```

### 3. Run Benchmarks

```bash
# Run performance benchmarks
cargo bench

# Run specific benchmarks
cargo bench --bench query_benchmarks
```

### 4. Check Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check for security issues
cargo audit
```

## Making Changes

### Branch Strategy

- Create a feature branch from `main`:
  ```bash
  git checkout -b feature/your-feature-name
  ```

- Use descriptive branch names:
  - `feature/add-persistence`
  - `fix/cache-memory-leak`
  - `docs/update-api-reference`

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(cache): add TTL support for cache entries

fix(query): resolve race condition in concurrent queries

docs(api): update use_query documentation

test(integration): add tests for error handling
```

### Code Style

- Follow Rust formatting conventions (`cargo fmt`)
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Keep functions small and focused
- Use `clippy` recommendations

## Testing

### Test Categories

1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test component interactions
3. **Property Tests**: Test invariants and edge cases
4. **API Stability Tests**: Ensure public API contracts
5. **Compatibility Tests**: Test Leptos 0.8 compatibility
6. **E2E Tests**: Test complete user workflows

### Writing Tests

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_key_creation() {
        let key = QueryKey::new(&["user", "123"]);
        assert_eq!(key.to_string(), "user:123");
    }
}
```

#### Integration Tests
```rust
#[test]
fn test_query_lifecycle() {
    // Test complete query lifecycle
    let client = QueryClient::new();
    let query = client.create_query(/* ... */);
    
    // Test initialization
    assert!(query.is_loading());
    
    // Test completion
    query.await_completion();
    assert!(query.is_success());
}
```

#### Property Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_cache_invariants(
        keys in prop::collection::vec(any::<String>(), 0..100),
        values in prop::collection::vec(any::<String>(), 0..100)
    ) {
        let mut cache = Cache::new();
        
        // Add items to cache
        for (key, value) in keys.iter().zip(values.iter()) {
            cache.insert(key.clone(), value.clone());
        }
        
        // Test invariants
        assert_eq!(cache.len(), keys.len());
        assert!(cache.len() <= cache.capacity());
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_query_key_creation

# Run tests in release mode
cargo test --release

# Run tests with coverage
cargo tarpaulin --out Html
```

## Documentation

### Code Documentation

- Document all public APIs with `///` comments
- Include examples in documentation
- Use markdown formatting in doc comments
- Document error conditions and edge cases

```rust
/// Creates a new query with the specified key and fetcher function.
///
/// # Arguments
///
/// * `key` - The query key used for caching and invalidation
/// * `fetcher` - The async function that fetches the data
/// * `options` - Optional configuration for the query
///
/// # Examples
///
/// ```rust
/// use leptos_query_rs::*;
///
/// let query = use_query(
///     || QueryKey::new(&["user", &user_id.to_string()]),
///     || async move { fetch_user(user_id).await },
///     QueryOptions::default()
/// );
/// ```
///
/// # Errors
///
/// This function will return an error if the query key is invalid
/// or if the fetcher function panics.
pub fn use_query<Key, Fetcher, Data>(
    key: impl Fn() -> Key,
    fetcher: impl Fn() -> Fetcher,
    options: QueryOptions,
) -> QueryResult<Data> {
    // Implementation
}
```

### README Updates

- Keep the main README up to date
- Include installation instructions
- Provide usage examples
- Link to documentation

### API Documentation

- Update API reference when adding new features
- Include type signatures and examples
- Document breaking changes

## Submitting Changes

### Pull Request Process

1. **Create a Pull Request**:
   - Use a descriptive title
   - Reference any related issues
   - Include a detailed description

2. **PR Description Template**:
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Testing
   - [ ] Unit tests added/updated
   - [ ] Integration tests added/updated
   - [ ] All tests pass
   - [ ] Manual testing completed

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Self-review completed
   - [ ] Documentation updated
   - [ ] No breaking changes (or documented)
   ```

3. **Review Process**:
   - All PRs require review
   - Address review feedback
   - Keep PRs focused and small
   - Rebase on latest main before merging

### CI/CD Requirements

All PRs must pass:

- [ ] Rust compilation
- [ ] All tests pass
- [ ] Code formatting (`cargo fmt`)
- [ ] Linting (`cargo clippy`)
- [ ] Security audit (`cargo audit`)
- [ ] Playwright tests
- [ ] Performance benchmarks

## Release Process

### Version Bumping

- Follow [Semantic Versioning](https://semver.org/)
- Update version in `Cargo.toml`
- Update `CHANGELOG.md`
- Update `RELEASE_NOTES.md`

### Release Checklist

- [ ] All tests pass
- [ ] Documentation updated
- [ ] Changelog updated
- [ ] Release notes written
- [ ] Version bumped
- [ ] Tag created
- [ ] GitHub release created
- [ ] Crates.io publish (if applicable)

## Getting Help

### Resources

- **Documentation**: [docs.leptos-query.rs](https://docs.leptos-query.rs)
- **Examples**: Check the `examples/` directory
- **Issues**: [GitHub Issues](https://github.com/cloud-shuttle/leptos-query-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/cloud-shuttle/leptos-query-rs/discussions)

### Community

- Join our [GitHub Discussions](https://github.com/cloud-shuttle/leptos-query-rs/discussions)
- Follow [@leptos_query](https://twitter.com/leptos_query) on Twitter
- Join the [Leptos Discord](https://discord.gg/leptos)

### Mentorship

New contributors are welcome! We offer:

- **Good First Issues**: Labeled issues for newcomers
- **Mentorship**: Experienced contributors can mentor newcomers
- **Documentation**: Comprehensive guides and examples
- **Community Support**: Helpful community members

## Recognition

Contributors are recognized in:

- **CONTRIBUTORS.md**: List of all contributors
- **Release Notes**: Contributors for each release
- **GitHub**: Contributor graphs and statistics
- **Community**: Public recognition in discussions

---

Thank you for contributing to leptos-query! 🚀
