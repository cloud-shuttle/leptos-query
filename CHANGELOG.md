# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2025-09-06

### Added
- **Comprehensive Test Suite**: Added 111+ tests covering unit, integration, property, API stability, and compatibility testing
- **Real-World Examples**: Added three complete example applications:
  - Todo App with CRUD operations and caching
  - Blog App with post management and real-time updates
  - Weather App with data fetching and forecast display
- **Performance Benchmarks**: Added comprehensive benchmarking suite with Criterion.rs
- **CI/CD Pipeline**: Added GitHub Actions workflows for automated testing and deployment
- **Cross-Browser Testing**: Added Playwright tests for cross-browser compatibility
- **API Stability Tests**: Added tests to ensure public API contracts remain stable
- **Leptos 0.8 Compatibility**: Added comprehensive compatibility tests for Leptos 0.8
- **Documentation**: Added comprehensive documentation including:
  - API reference
  - Quick start guide
  - Common patterns guide
  - Migration guide
  - Community guidelines

### Changed
- **API Exports**: Added `SerializedData` and `CacheEntry` to public exports
- **Feature Flags**: Reorganized feature flags for better WASM compatibility:
  - `native`: For native Rust applications
  - `wasm`: For WebAssembly applications
  - `csr`: For client-side rendering
  - `ssr`: For server-side rendering
  - `hydrate`: For hydration
- **Dependencies**: Made `tokio` and `reqwest` optional dependencies to improve WASM compatibility
- **Import Paths**: Standardized import paths across all examples and documentation

### Fixed
- **Doctest Failures**: Fixed all doctest compilation errors
- **WASM Compatibility**: Resolved WASM build issues with proper feature flag configuration
- **Type Annotations**: Added explicit type annotations for better type inference
- **Signal Destructuring**: Fixed Leptos 0.8 signal destructuring in compatibility tests
- **Callback Types**: Fixed callback type annotations for Leptos 0.8 compatibility

### Performance
- **Query Keys**: 119-257 ns for various key operations
- **Query Client**: 21-915 ns for client operations
- **Retry Logic**: 1-2 ns for retry operations
- **Serialization**: 82-140 ns for JSON/bincode operations
- **Cache Operations**: 46 ns to 4.6 µs depending on cache size
- **Cache Invalidation**: 93 ns to 4.2 µs for different invalidation patterns
- **Concurrent Access**: 685 ns to 6.3 µs for concurrent operations
- **Memory Usage**: 546 µs for cache growth, 7-17 ns for cleanup

### Security
- **Dependency Updates**: Updated all dependencies to latest secure versions
- **Code Quality**: Added comprehensive linting and formatting checks

## [0.4.2] - 2025-09-01

### Added
- Initial release with basic query functionality
- Basic caching and invalidation
- Error handling and retry logic
- Basic documentation

### Changed
- Improved error handling
- Enhanced caching strategies

### Fixed
- Memory leak issues
- Race conditions in concurrent access

## [0.4.1] - 2025-08-28

### Added
- Basic query client implementation
- Simple caching mechanism
- Error handling

### Fixed
- Initial compilation issues
- Basic functionality bugs

## [0.4.0] - 2025-08-25

### Added
- Initial project setup
- Basic project structure
- Core query functionality
- Basic documentation

---

## Release Notes

### v0.5.0 - "The Testing Release"

This release focuses on stability, testing, and real-world usability. We've added comprehensive test coverage, real-world examples, and performance benchmarks to ensure leptos-query is ready for production use.

**Key Highlights:**
- 🧪 **111+ Tests**: Comprehensive test coverage across all functionality
- 📚 **3 Real-World Examples**: Todo, Blog, and Weather apps demonstrating real usage
- ⚡ **Performance Benchmarks**: Sub-microsecond performance for most operations
- 🌐 **Cross-Browser Support**: Tested across Chrome, Firefox, Safari, and mobile browsers
- 🔧 **CI/CD Pipeline**: Automated testing and deployment
- 📖 **Comprehensive Documentation**: API reference, guides, and examples

**Breaking Changes:**
- None! This release maintains full backward compatibility.

**Migration Guide:**
- No migration required for existing users
- New feature flags available for better WASM compatibility
- Enhanced error handling and type safety

**What's Next:**
- v1.0.0 will focus on advanced features like offline support, persistence, and devtools
- Community feedback and real-world usage validation
- Performance optimizations based on benchmark results

---

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
