# Leptos 0.8 Compatibility Strategy

## Overview

This document outlines our strategy for maintaining compatibility with Leptos 0.8 and ensuring smooth upgrades for users of `leptos-query`.

## Expected Leptos 0.8 Changes

Based on Leptos development patterns and community discussions, we anticipate these potential breaking changes:

### 1. Signal API Changes
- **Current**: `Signal<T>`, `ReadSignal<T>`, `WriteSignal<T>`
- **Expected**: Possible API refinements or new signal types
- **Impact**: High - affects all reactive state in our library

### 2. Effect API Changes
- **Current**: `create_effect`, `create_memo`
- **Expected**: Possible signature changes or new effect types
- **Impact**: High - affects all reactive updates

### 3. Component API Changes
- **Current**: `#[component]`, `impl IntoView`
- **Expected**: Possible macro changes or new component patterns
- **Impact**: Medium - affects component integration

### 4. Context API Changes
- **Current**: `provide_context`, `use_context`
- **Expected**: Possible API refinements
- **Impact**: Medium - affects QueryClient provider

## Compatibility Layer Design

### 1. Version Detection and Feature Flags

```rust
// Cargo.toml
[dependencies]
leptos = { version = "0.6", features = ["csr", "ssr"] }

[features]
leptos-0-6 = ["leptos/0-6"]
leptos-0-8 = ["leptos/0-8"]
default = ["leptos-0-6"]
```

### 2. Conditional Compilation Strategy

```rust
// src/compat/mod.rs
#[cfg(feature = "leptos-0-6")]
pub mod v0_6;

#[cfg(feature = "leptos-0-8")]
pub mod v0_8;

// Re-export the appropriate version
#[cfg(feature = "leptos-0-6")]
pub use v0_6::*;

#[cfg(feature = "leptos-0-8")]
pub use v0_8::*;
```

### 3. Signal Compatibility Layer

```rust
// src/compat/signals.rs
#[cfg(feature = "leptos-0-6")]
pub mod v0_6 {
    use leptos::*;
    
    pub type CompatSignal<T> = Signal<T>;
    pub type CompatReadSignal<T> = ReadSignal<T>;
    pub type CompatWriteSignal<T> = WriteSignal<T>;
    
    pub fn create_compat_signal<T>(initial: T) -> (CompatReadSignal<T>, CompatWriteSignal<T>) {
        create_signal(initial)
    }
    
    pub fn create_compat_memo<T, F>(f: F) -> CompatReadSignal<T>
    where
        F: Fn() -> T + 'static,
        T: Clone + 'static,
    {
        create_memo(f)
    }
}

#[cfg(feature = "leptos-0-8")]
pub mod v0_8 {
    use leptos::*;
    
    // Adapt to new Leptos 0.8 signal API
    pub type CompatSignal<T> = Signal<T>; // Update as needed
    pub type CompatReadSignal<T> = ReadSignal<T>; // Update as needed
    pub type CompatWriteSignal<T> = WriteSignal<T>; // Update as needed
    
    pub fn create_compat_signal<T>(initial: T) -> (CompatReadSignal<T>, CompatWriteSignal<T>) {
        // Adapt to new API
        create_signal(initial)
    }
    
    pub fn create_compat_memo<T, F>(f: F) -> CompatReadSignal<T>
    where
        F: Fn() -> T + 'static,
        T: Clone + 'static,
    {
        // Adapt to new API
        create_memo(f)
    }
}
```

### 4. Effect Compatibility Layer

```rust
// src/compat/effects.rs
#[cfg(feature = "leptos-0-6")]
pub mod v0_6 {
    use leptos::*;
    
    pub fn create_compat_effect<F>(f: F)
    where
        F: Fn() + 'static,
    {
        create_effect(f)
    }
    
    pub fn create_compat_memo<T, F>(f: F) -> ReadSignal<T>
    where
        F: Fn() -> T + 'static,
        T: Clone + 'static,
    {
        create_memo(f)
    }
}

#[cfg(feature = "leptos-0-8")]
pub mod v0_8 {
    use leptos::*;
    
    pub fn create_compat_effect<F>(f: F)
    where
        F: Fn() + 'static,
    {
        // Adapt to new Leptos 0.8 effect API
        create_effect(f)
    }
    
    pub fn create_compat_memo<T, F>(f: F) -> ReadSignal<T>
    where
        F: Fn() -> T + 'static,
        T: Clone + 'static,
    {
        // Adapt to new Leptos 0.8 memo API
        create_memo(f)
    }
}
```

### 5. Component Compatibility Layer

```rust
// src/compat/components.rs
#[cfg(feature = "leptos-0-6")]
pub mod v0_6 {
    use leptos::*;
    
    pub use leptos::component;
    pub use leptos::IntoView;
    
    pub fn create_compat_context<T>(value: T) -> T {
        provide_context(value);
        value
    }
    
    pub fn use_compat_context<T>() -> Option<T>
    where
        T: Clone + 'static,
    {
        use_context::<T>()
    }
}

#[cfg(feature = "leptos-0-8")]
pub mod v0_8 {
    use leptos::*;
    
    // Adapt to new Leptos 0.8 component API
    pub use leptos::component; // Update as needed
    pub use leptos::IntoView; // Update as needed
    
    pub fn create_compat_context<T>(value: T) -> T {
        // Adapt to new context API
        provide_context(value);
        value
    }
    
    pub fn use_compat_context<T>() -> Option<T>
    where
        T: Clone + 'static,
    {
        // Adapt to new context API
        use_context::<T>()
    }
}
```

## Implementation Strategy

### Phase 1: Prepare Compatibility Layer (Current)

1. **Create compatibility modules** with conditional compilation
2. **Add feature flags** for different Leptos versions
3. **Update internal code** to use compatibility layer
4. **Add tests** for both versions

### Phase 2: Leptos 0.8 Release

1. **Monitor Leptos 0.8 changes** and update compatibility layer
2. **Test with both versions** to ensure compatibility
3. **Update documentation** with migration guides
4. **Release new version** with dual support

### Phase 3: Migration Period

1. **Maintain dual support** for 6-12 months
2. **Provide migration tools** and guides
3. **Gradually deprecate** Leptos 0.6 support
4. **Focus on Leptos 0.8** as primary target

## Updated Library Structure

```
src/
├── compat/
│   ├── mod.rs           # Compatibility layer entry point
│   ├── signals.rs       # Signal API compatibility
│   ├── effects.rs       # Effect API compatibility
│   └── components.rs    # Component API compatibility
├── client/
├── query/
├── mutation/
├── retry/
├── dedup/
└── lib.rs
```

## Updated Cargo.toml

```toml
[package]
name = "leptos-query"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core Leptos dependencies with version flexibility
leptos = { version = "0.6", features = ["csr", "ssr"] }

[features]
default = ["leptos-0-6"]
leptos-0-6 = ["leptos/0-6"]
leptos-0-8 = ["leptos/0-8"]

# Other features
csr = ["leptos/csr"]
ssr = ["leptos/ssr", "tokio/rt-multi-thread"]
hydrate = ["leptos/hydrate"]
devtools = []
persistence = []
offline = ["persistence"]
```

## Migration Guide for Users

### For Leptos 0.6 Users

```toml
# Cargo.toml
[dependencies]
leptos-query = { version = "0.1", features = ["leptos-0-6"] }
```

### For Leptos 0.8 Users

```toml
# Cargo.toml
[dependencies]
leptos-query = { version = "0.1", features = ["leptos-0-8"] }
```

### Automatic Detection (Future)

```rust
// In the future, we could auto-detect Leptos version
#[cfg(feature = "auto-detect")]
pub fn auto_detect_leptos_version() -> LeptosVersion {
    // Implementation to detect Leptos version at compile time
}
```

## Testing Strategy

### Dual Version Testing

```rust
// tests/compatibility_tests.rs
#[cfg(feature = "leptos-0-6")]
mod leptos_0_6_tests {
    use leptos_query::*;
    
    #[test]
    fn test_basic_query_leptos_0_6() {
        // Test with Leptos 0.6
    }
}

#[cfg(feature = "leptos-0-8")]
mod leptos_0_8_tests {
    use leptos_query::*;
    
    #[test]
    fn test_basic_query_leptos_0_8() {
        // Test with Leptos 0.8
    }
}
```

### CI/CD Pipeline

```yaml
# .github/workflows/test.yml
jobs:
  test-leptos-0-6:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Test with Leptos 0.6
        run: cargo test --features leptos-0-6

  test-leptos-0-8:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Test with Leptos 0.8
        run: cargo test --features leptos-0-8
```

## Benefits of This Approach

### 1. **Smooth Migration**
- Users can upgrade at their own pace
- No breaking changes for existing users
- Clear migration path

### 2. **Future-Proof**
- Ready for Leptos 0.8 when it releases
- Flexible architecture for future versions
- Maintains backward compatibility

### 3. **Community Support**
- Supports both Leptos versions simultaneously
- Reduces ecosystem fragmentation
- Encourages adoption

### 4. **Development Efficiency**
- Single codebase for both versions
- Shared logic and tests
- Easier maintenance

## Implementation Timeline

### Week 1-2: Foundation
- [ ] Create compatibility layer structure
- [ ] Implement signal compatibility
- [ ] Add feature flags

### Week 3-4: Core Integration
- [ ] Update query module to use compatibility layer
- [ ] Update mutation module to use compatibility layer
- [ ] Update client module to use compatibility layer

### Week 5-6: Testing & Documentation
- [ ] Add dual version tests
- [ ] Update documentation
- [ ] Create migration guides

### Week 7-8: Release Preparation
- [ ] Final testing
- [ ] Documentation review
- [ ] Release preparation

## Conclusion

This compatibility layer approach ensures that `leptos-query` remains future-proof and user-friendly. By preparing for Leptos 0.8 now, we can provide a smooth upgrade path for users while maintaining the stability and reliability of the current implementation.

The strategy balances immediate needs with long-term sustainability, ensuring that the library can evolve with the Leptos ecosystem while providing a stable foundation for users.
