# Contributing to Leptos Query

Thank you for your interest in contributing to Leptos Query! This document provides guidelines and information for contributors.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Feature Requests](#feature-requests)
- [Documentation](#documentation)

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo
- Git

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/your-username/leptos-query.git
   cd leptos-query
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/CloudShuttle/leptos-query.git
   ```

## Development Setup

### Install Dependencies

```bash
# Install wasm-pack for WebAssembly builds
cargo install wasm-pack

# Install cargo-leptos for Leptos development
cargo install cargo-leptos
```

### Build the Project

```bash
# Build the library
cargo build

# Build with all features
cargo build --all-features

# Build for WebAssembly
wasm-pack build --target web
```

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests
cargo test --test integration_tests

# Run examples
cargo check --example basic_usage
```

## Code Style

### Rust Conventions

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- Use `rustfmt` to format code:
  ```bash
  cargo fmt
  ```
- Use `clippy` for linting:
  ```bash
  cargo clippy
  cargo clippy --all-features
  ```

### Naming Conventions

- **Functions**: Use `snake_case`
- **Types**: Use `PascalCase`
- **Constants**: Use `SCREAMING_SNAKE_CASE`
- **Modules**: Use `snake_case`
- **Files**: Use `snake_case.rs`

### Documentation

- Document all public APIs with doc comments
- Include examples in documentation
- Use `cargo doc` to generate documentation:
  ```bash
  cargo doc --open
  ```

### Example Documentation

```rust
/// Fetches a user by their ID.
///
/// This function will automatically cache the result and handle retries
/// according to the configured retry policy.
///
/// # Arguments
///
/// * `id` - The user ID to fetch
///
/// # Returns
///
/// Returns a `Result<User, QueryError>` where `User` is the fetched user data
/// or `QueryError` if the request failed.
///
/// # Examples
///
/// ```rust
/// use leptos_query::*;
///
/// let user_query = use_query(
///     || &["users", "1"][..],
///     || || async move { fetch_user(1).await },
///     QueryOptions::default()
/// );
/// ```
pub async fn fetch_user(id: u32) -> Result<User, QueryError> {
    // Implementation
}
```

## Testing

### Test Structure

- **Unit Tests**: Test individual functions and types
- **Integration Tests**: Test the interaction between components
- **Example Tests**: Ensure examples compile and work correctly

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function(input);
        
        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        // Test async functions
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Test Guidelines

- Test both success and failure cases
- Test edge cases and boundary conditions
- Use descriptive test names
- Keep tests focused and isolated
- Mock external dependencies when appropriate

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html

# Run tests in a specific module
cargo test module_name

# Run tests matching a pattern
cargo test pattern
```

## Pull Request Process

### Before Submitting

1. **Ensure tests pass**: Run `cargo test` and fix any failures
2. **Format code**: Run `cargo fmt`
3. **Check linting**: Run `cargo clippy`
4. **Update documentation**: Add or update doc comments as needed
5. **Add tests**: Include tests for new functionality

### Creating a Pull Request

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** and commit them:
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```

3. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create a Pull Request** on GitHub with:
   - Clear title describing the change
   - Detailed description of what was changed and why
   - Reference to any related issues
   - Screenshots for UI changes (if applicable)

### Commit Message Format

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(query): add support for infinite queries
fix(cache): resolve memory leak in garbage collection
docs(api): update use_query documentation with examples
```

### Review Process

1. **Automated Checks**: Ensure CI passes
2. **Code Review**: Address reviewer feedback
3. **Testing**: Verify changes work as expected
4. **Documentation**: Update docs if needed

## Issue Reporting

### Bug Reports

When reporting bugs, please include:

1. **Clear description** of the problem
2. **Steps to reproduce** the issue
3. **Expected behavior** vs actual behavior
4. **Environment details**:
   - Rust version: `rustc --version`
   - Leptos version
   - Operating system
   - Browser (if applicable)
5. **Minimal reproduction** code
6. **Error messages** or stack traces

### Example Bug Report

```markdown
## Bug Description
The `use_query` hook doesn't properly handle network errors.

## Steps to Reproduce
1. Create a query that will fail
2. Call `use_query` with the failing function
3. Observe that the error state is not properly set

## Expected Behavior
The query should set the error state and display an error message.

## Actual Behavior
The query remains in loading state indefinitely.

## Environment
- Rust: 1.70.0
- Leptos: 0.6.0
- OS: macOS 13.0

## Reproduction Code
```rust
let query = use_query(
    || &["test"][..],
    || || async move { 
        Err(QueryError::Network { message: "Test error".to_string() })
    },
    QueryOptions::default()
);
```
```

## Feature Requests

When requesting features, please include:

1. **Clear description** of the feature
2. **Use case** and motivation
3. **Proposed API** design (if applicable)
4. **Alternatives considered**
5. **Implementation complexity** estimate

### Example Feature Request

```markdown
## Feature Description
Add support for infinite queries with automatic pagination.

## Use Case
Users need to load large datasets in chunks, such as social media feeds or product catalogs.

## Proposed API
```rust
let infinite_query = use_infinite_query(
    || &["posts"][..],
    |page_param| async move { fetch_posts_page(page_param).await },
    InfiniteQueryOptions::default()
        .with_get_next_page_param(|last_page| last_page.next_cursor)
);
```

## Alternatives Considered
- Manual pagination with multiple queries
- Virtual scrolling without infinite queries

## Implementation Complexity
Medium - requires new hook, cache management for pages, and UI utilities.
```

## Documentation

### Contributing to Documentation

1. **Update existing docs** when changing APIs
2. **Add examples** for new features
3. **Keep guides up to date** with latest changes
4. **Test documentation examples** to ensure they work

### Documentation Structure

```
docs/
├── api-reference.md      # Complete API documentation
├── migration.md          # Migration guide
└── guides/
    ├── quick-start.md    # Getting started guide
    └── common-patterns.md # Best practices and patterns
```

### Writing Documentation

- Use clear, concise language
- Include code examples
- Explain concepts before showing code
- Keep examples simple and focused
- Test all code examples

## Getting Help

- **Discord**: Join the [Leptos Discord](https://discord.gg/leptos) for community support
- **GitHub Issues**: Use issues for bugs and feature requests
- **GitHub Discussions**: Use discussions for questions and ideas

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and inclusive in all interactions.

## License

By contributing to Leptos Query, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Leptos Query! 🚀
