# Leptos 0.8 Compatibility Strategy

## Overview

This document outlines our strategy for maintaining compatibility with Leptos 0.8 and ensuring smooth upgrades for users of `leptos-query`.

## Current Implementation

### Simple Version Detection

We've implemented a lightweight compatibility layer that focuses on version detection and future-proofing:

```rust
use leptos_query::compat::{LeptosVersion, leptos_version, LeptosCompat};

// Get current Leptos version
let version = leptos_version();
println!("Using Leptos version: {}", version.as_str());

// Check if using 0.8 or later
if version.is_0_8_or_later() {
    // Use 0.8-specific features
} else {
    // Use 0.6-compatible features
}
```

### Feature Flags

The library supports feature flags for different Leptos versions:

```toml
# Cargo.toml
[dependencies]
leptos-query = { version = "0.1", features = ["leptos-0-6"] }

# For future Leptos 0.8 support
leptos-query = { version = "0.1", features = ["leptos-0-8"] }
```

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

## Migration Strategy

### Phase 1: Current (Leptos 0.6)
- ✅ **Version detection** implemented
- ✅ **Feature flags** configured
- ✅ **Documentation** prepared
- ✅ **Tests** passing

### Phase 2: Leptos 0.8 Release
1. **Monitor changes** and update compatibility layer
2. **Test with both versions** to ensure compatibility
3. **Update documentation** with migration guides
4. **Release new version** with dual support

### Phase 3: Migration Period
1. **Maintain dual support** for 6-12 months
2. **Provide migration tools** and guides
3. **Gradually deprecate** Leptos 0.6 support
4. **Focus on Leptos 0.8** as primary target

## Implementation Details

### Version Detection

```rust
// src/compat/mod.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeptosVersion {
    V0_6,
    V0_8,
}

impl LeptosVersion {
    pub fn current() -> Self {
        #[cfg(feature = "leptos-0-8")]
        return LeptosVersion::V0_8;
        
        // Default to 0.6 if no version is specified
        LeptosVersion::V0_6
    }
    
    pub fn is_0_8_or_later(&self) -> bool {
        matches!(self, LeptosVersion::V0_8)
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            LeptosVersion::V0_6 => "0.6",
            LeptosVersion::V0_8 => "0.8",
        }
    }
}
```

### Compatibility Trait

```rust
pub trait LeptosCompat {
    fn version() -> LeptosVersion;
    
    fn is_0_8_or_later() -> bool {
        Self::version().is_0_8_or_later()
    }
}

impl LeptosCompat for LeptosVersion {
    fn version() -> LeptosVersion {
        leptos_version()
    }
}
```

## Usage Examples

### Basic Version Detection

```rust
use leptos_query::compat::leptos_version;

#[component]
fn App() -> impl IntoView {
    let version = leptos_version();
    
    view! {
        <div>
            <p>"Using Leptos version: " {version.as_str()}</p>
            {move || {
                if version.is_0_8_or_later() {
                    view! { <p>"Using advanced features"</p> }
                } else {
                    view! { <p>"Using standard features"</p> }
                }
            }}
        </div>
    }
}
```

### Conditional Feature Usage

```rust
use leptos_query::compat::{LeptosVersion, LeptosCompat};

fn setup_query_client() -> QueryClient {
    let config = if <LeptosVersion as LeptosCompat>::is_0_8_or_later() {
        // Use 0.8-specific configuration
        QueryClientConfig::default()
            .with_advanced_features()
    } else {
        // Use 0.6-compatible configuration
        QueryClientConfig::default()
    };
    
    QueryClient::new(config)
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

## Migration Guide for Users

### For Leptos 0.6 Users

```toml
# Cargo.toml
[dependencies]
leptos-query = { version = "0.1", features = ["leptos-0-6"] }
```

### For Leptos 0.8 Users (Future)

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
- Simple, maintainable code
- Shared logic and tests
- Easier maintenance

## Implementation Timeline

### ✅ Completed (Current)
- [x] Create version detection system
- [x] Add feature flags
- [x] Implement compatibility trait
- [x] Add tests
- [x] Update documentation

### 🔄 Future Work
- [ ] Monitor Leptos 0.8 development
- [ ] Update compatibility layer when 0.8 releases
- [ ] Add dual version testing
- [ ] Create migration tools
- [ ] Update examples and documentation

## Conclusion

This simplified compatibility layer approach ensures that `leptos-query` remains future-proof and user-friendly. By preparing for Leptos 0.8 now with a lightweight version detection system, we can provide a smooth upgrade path for users while maintaining the stability and reliability of the current implementation.

The strategy balances immediate needs with long-term sustainability, ensuring that the library can evolve with the Leptos ecosystem while providing a stable foundation for users.

### Key Advantages

1. **Lightweight**: Minimal overhead and complexity
2. **Maintainable**: Simple code that's easy to understand and modify
3. **Future-Ready**: Prepared for Leptos 0.8 when it releases
4. **User-Friendly**: Clear migration path for users
5. **Community-Focused**: Supports ecosystem growth and adoption

The compatibility layer is now ready and will be updated as Leptos 0.8 development progresses.
