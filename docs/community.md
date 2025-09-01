# Community Guidelines

Welcome to the `leptos-query` community! This document outlines our community standards, contribution guidelines, and how to get involved.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for everyone. By participating in this project, you agree to:

- Be respectful and considerate of others
- Use inclusive language
- Be collaborative and constructive
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Join the Community

- **GitHub Discussions**: [Start a discussion](https://github.com/cloud-shuttle/leptos-query/discussions)
- **Issues**: [Report bugs or request features](https://github.com/cloud-shuttle/leptos-query/issues)
- **Discord**: Join the [Leptos Discord](https://discord.gg/leptos) and look for the `#leptos-query` channel
- **Reddit**: Share on [r/rust](https://reddit.com/r/rust) or [r/leptos](https://reddit.com/r/leptos)

### First Contributions

Looking to make your first contribution? Here are some great starting points:

1. **Documentation**: Fix typos, improve examples, or add missing documentation
2. **Examples**: Create new examples or improve existing ones
3. **Tests**: Add test cases for edge cases or missing functionality
4. **Issues**: Help triage and reproduce reported issues

## Contribution Guidelines

### Before You Start

1. **Check existing issues**: Search for similar issues or discussions
2. **Discuss changes**: For significant changes, open a discussion first
3. **Read the docs**: Familiarize yourself with the codebase and documentation

### Development Workflow

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/your-feature-name`
3. **Make your changes**: Follow the coding standards below
4. **Test your changes**: Run `make ci-local` to ensure everything works
5. **Commit your changes**: Use conventional commit messages
6. **Push and create a PR**: Include a clear description of your changes

### Coding Standards

- **Rust**: Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- **Formatting**: Use `cargo fmt` for code formatting
- **Linting**: Ensure `cargo clippy` passes without warnings
- **Documentation**: Add documentation for public APIs
- **Tests**: Include tests for new functionality

### Commit Messages

Use conventional commit messages:

```
feat: add new query invalidation method
fix: resolve memory leak in query cache
docs: update installation instructions
test: add benchmarks for query performance
refactor: simplify query key generation
```

## Project Structure

```
leptos-query/
├── src/                    # Source code
│   ├── api/               # Public API
│   ├── client/            # Query client implementation
│   ├── compat/            # Leptos version compatibility
│   ├── query/             # Query implementation
│   ├── mutation/          # Mutation implementation
│   └── lib.rs             # Library entry point
├── examples/              # Usage examples
├── tests/                 # Test files
├── benches/               # Performance benchmarks
├── docs/                  # Documentation
└── scripts/               # Development scripts
```

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git
- A code editor (VS Code recommended)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/cloud-shuttle/leptos-query.git
cd leptos-query

# Set up development environment
make setup

# Run tests
make test

# Run CI checks locally
make ci-local
```

### Available Commands

```bash
make help          # Show all available commands
make test          # Run all tests
make doc           # Generate documentation
make fmt           # Format code
make clippy        # Run linter
make ci-local      # Run full CI checks
make release       # Build release version
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test integration_tests

# Run with specific features
cargo test --features leptos-0-8

# Run benchmarks
cargo bench
```

### Writing Tests

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test the public API and end-to-end functionality
- **Examples**: Ensure all examples compile and run correctly
- **Benchmarks**: Add performance benchmarks for critical paths

## Documentation

### Writing Documentation

- **API Documentation**: Use Rust doc comments for all public APIs
- **Examples**: Include runnable examples in documentation
- **Guides**: Write clear, step-by-step guides for common use cases
- **Migration**: Provide migration guides for breaking changes

### Documentation Standards

- Use clear, concise language
- Include code examples
- Explain the "why" not just the "how"
- Keep documentation up-to-date with code changes

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **Major**: Breaking changes
- **Minor**: New features (backward compatible)
- **Patch**: Bug fixes (backward compatible)

### Release Checklist

- [ ] All tests pass
- [ ] Documentation is up-to-date
- [ ] Changelog is updated
- [ ] Version is bumped in `Cargo.toml`
- [ ] Release notes are prepared
- [ ] GitHub release is created
- [ ] Crate is published to crates.io

## Community Resources

### Learning Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Leptos Documentation](https://leptos.dev/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

### Tools and Utilities

- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) - Build tool for Leptos
- [wasm-pack](https://github.com/rustwasm/wasm-pack) - WebAssembly packaging
- [trunk](https://github.com/thedodd/trunk) - Web application bundler
- [cargo-watch](https://github.com/watchexec/cargo-watch) - File watching for development

### Related Projects

- [Leptos](https://github.com/leptos-rs/leptos) - The main Leptos framework
- [Leptos Router](https://github.com/leptos-rs/leptos_router) - Routing for Leptos
- [Leptos Meta](https://github.com/leptos-rs/leptos_meta) - Document head management
- [TanStack Query](https://tanstack.com/query) - Inspiration for this project

## Recognition

### Contributors

We recognize and appreciate all contributors to the project. Contributors are listed in:

- [GitHub Contributors](https://github.com/cloud-shuttle/leptos-query/graphs/contributors)
- [Cargo.toml](Cargo.toml) authors field
- [README.md](README.md) contributors section

### Ways to Contribute

- **Code**: Implement features, fix bugs, improve performance
- **Documentation**: Write guides, improve examples, fix typos
- **Testing**: Add tests, improve test coverage, write benchmarks
- **Community**: Help others, answer questions, share examples
- **Feedback**: Report issues, suggest improvements, discuss ideas

## Getting Help

### Common Issues

- **Compilation errors**: Check your Rust version and feature flags
- **Runtime errors**: Enable debug logging and check the console
- **Performance issues**: Run benchmarks and profile your code
- **Integration problems**: Check compatibility with your Leptos version

### Support Channels

1. **GitHub Issues**: For bugs and feature requests
2. **GitHub Discussions**: For questions and general discussion
3. **Discord**: For real-time help and community chat
4. **Documentation**: Check the guides and API reference first

## Future Directions

### Roadmap

- [ ] Infinite queries for pagination
- [ ] Query persistence across sessions
- [ ] DevTools integration
- [ ] More advanced caching strategies
- [ ] Server-side rendering optimizations

### Ideas for Contributions

- **Performance**: Optimize query execution and caching
- **Features**: Implement new query patterns and utilities
- **Ecosystem**: Create integrations with other Leptos libraries
- **Tooling**: Build development tools and debugging utilities
- **Documentation**: Create tutorials, guides, and examples

## Contact

- **Maintainers**: [@cloud-shuttle](https://github.com/cloud-shuttle)
- **Repository**: [github.com/cloud-shuttle/leptos-query](https://github.com/cloud-shuttle/leptos-query)
- **Discord**: [Leptos Discord](https://discord.gg/leptos)

Thank you for being part of the `leptos-query` community! 🚀
