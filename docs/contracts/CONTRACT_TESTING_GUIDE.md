# Contract Testing Guide

## 🎯 Overview

This guide explains how to implement and maintain contract testing for the leptos-query library. Contract testing ensures that API contracts are stable, compatible, and reliable across versions and platforms.

## 📚 What is Contract Testing?

Contract testing is a methodology that ensures that different services (or in our case, different versions and platforms) can communicate correctly by verifying that they adhere to a shared contract.

### Types of Contracts

1. **API Contracts**: Formal specifications of how APIs should behave
2. **Schema Contracts**: Validation rules for data structures
3. **Compatibility Contracts**: Rules for backward and forward compatibility
4. **Platform Contracts**: Requirements for different runtime environments

## 🏗️ Contract Testing Architecture

### 1. Schema Validation

```rust
use jsonschema::{JSONSchema, ValidationError};
use serde_json::Value;

// Load and compile schema
let schema_content = include_str!("../../contracts/schemas/query_options.json");
let schema_value: Value = serde_json::from_str(schema_content).unwrap();
let schema = JSONSchema::compile(&schema_value).unwrap();

// Validate data
let data = json!({
    "enabled": true,
    "stale_time": 60000,
    "cache_time": 300000
});

let result = schema.validate(&data);
assert!(result.is_ok(), "Data should match schema");
```

### 2. API Evolution Tracking

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiVersion {
    version: String,
    breaking_changes: Vec<String>,
    deprecated_features: Vec<String>,
    new_features: Vec<String>,
}

// Track API changes
let v0_5_0 = ApiVersion {
    version: "0.5.0".to_string(),
    breaking_changes: vec![
        "QueryOptions builder pattern changed".to_string(),
    ],
    deprecated_features: vec![
        "QueryOptions::new() constructor".to_string(),
    ],
    new_features: vec![
        "Infinite queries".to_string(),
        "Optimistic updates".to_string(),
    ],
};
```

### 3. Compatibility Testing

```rust
#[test]
fn test_backward_compatibility() {
    // Test that new versions can handle old API calls
    let old_api_call = create_v0_4_0_api_call();
    let new_client = QueryClient::new();
    let result = new_client.execute_legacy_query(old_api_call);
    assert!(result.is_ok(), "Backward compatibility broken");
}
```

## 🧪 Running Contract Tests

### Prerequisites

Add these dependencies to your `Cargo.toml`:

```toml
[dev-dependencies]
jsonschema = "0.18"
serde_json = "1.0"
pact_consumer = "0.9"  # For consumer-driven contract testing
```

### Running All Contract Tests

```bash
# Run all contract tests
cargo test --test schema_validation_tests
cargo test --test api_evolution_tests
cargo test --test compatibility_tests

# Run with specific features
cargo test --features "devtools,persistence" --test compatibility_tests
```

### Running Specific Test Categories

```bash
# Schema validation only
cargo test --test schema_validation_tests test_query_options_schema_validation

# API evolution only
cargo test --test api_evolution_tests test_breaking_changes_detection

# Compatibility only
cargo test --test compatibility_tests test_cross_platform_compatibility
```

## 📋 Contract Test Categories

### 1. Schema Validation Tests

**Purpose**: Ensure all data structures conform to their JSON schemas.

**Location**: `tests/contracts/schema_validation_tests.rs`

**Key Tests**:
- `test_query_options_schema_validation()`: Validates QueryOptions structure
- `test_retry_config_schema_validation()`: Validates RetryConfig structure
- `test_query_key_schema_validation()`: Validates QueryKey structure
- `test_error_schemas_validation()`: Validates error structures
- `test_serialization_contract_compliance()`: Ensures Rust types serialize correctly
- `test_deserialization_contract_compliance()`: Ensures JSON deserializes correctly

### 2. API Evolution Tests

**Purpose**: Track API changes and ensure migration paths exist.

**Location**: `tests/contracts/api_evolution_tests.rs`

**Key Tests**:
- `test_api_version_compatibility()`: Validates version compatibility matrix
- `test_backward_compatibility_within_major_version()`: Tests backward compatibility
- `test_breaking_changes_detection()`: Ensures breaking changes are documented
- `test_deprecation_warnings()`: Validates deprecation warnings work
- `test_migration_path_validation()`: Ensures migration paths exist
- `test_api_surface_stability()`: Tests API surface stability

### 3. Compatibility Tests

**Purpose**: Ensure the library works across different platforms and environments.

**Location**: `tests/contracts/compatibility_tests.rs`

**Key Tests**:
- `test_cross_platform_compatibility()`: Tests WASM, Native, SSR compatibility
- `test_feature_flag_compatibility()`: Tests different feature combinations
- `test_serialization_compatibility()`: Tests cross-platform serialization
- `test_async_runtime_compatibility()`: Tests different async runtimes
- `test_memory_compatibility()`: Tests memory constraint handling
- `test_concurrent_access_compatibility()`: Tests concurrent access patterns

## 🔧 Writing Contract Tests

### 1. Schema Validation Test Template

```rust
#[test]
fn test_your_schema_validation() {
    let schema = load_schema("your_schema_name");
    
    // Test valid data
    let valid_data = json!({
        "field1": "value1",
        "field2": 42
    });
    
    let result = schema.validate(&valid_data);
    assert!(result.is_ok(), "Valid data should pass validation");
    
    // Test invalid data
    let invalid_data = json!({
        "field1": "value1",
        "field2": "not_a_number"  // Wrong type
    });
    
    let result = schema.validate(&invalid_data);
    assert!(result.is_err(), "Invalid data should fail validation");
}
```

### 2. Compatibility Test Template

```rust
#[test]
fn test_your_compatibility() {
    let platforms = vec!["wasm", "native", "ssr"];
    
    for platform in &platforms {
        let client = create_client_for_platform(platform);
        let query = create_test_query();
        
        let result = client.execute_query(query);
        assert!(result.is_ok(), 
            "Platform '{}' should support functionality", platform);
    }
}
```

### 3. API Evolution Test Template

```rust
#[test]
fn test_your_api_evolution() {
    let matrix = CompatibilityMatrix::new();
    
    // Test that breaking changes are documented
    for version in &matrix.versions {
        if !version.breaking_changes.is_empty() {
            for breaking_change in &version.breaking_changes {
                let migration_path = get_migration_path(&version.version, breaking_change);
                assert!(migration_path.is_some(), 
                    "Breaking change should have migration path");
            }
        }
    }
}
```

## 📊 Contract Test Metrics

### Coverage Requirements

- **Schema Coverage**: 100% of public data structures
- **API Surface Coverage**: 100% of public APIs
- **Platform Coverage**: All supported platforms (WASM, Native, SSR)
- **Version Coverage**: All supported versions
- **Feature Coverage**: All feature flag combinations

### Quality Metrics

- **Test Pass Rate**: 100% for all contract tests
- **Schema Validation**: 100% valid data passes, 100% invalid data fails
- **Compatibility**: 100% backward compatibility within major versions
- **Performance**: < 5% overhead from contract validation

## 🚨 Common Issues and Solutions

### 1. Schema Validation Failures

**Problem**: Schema validation fails unexpectedly.

**Solution**:
```rust
// Check schema compilation
let schema = JSONSchema::compile(&schema_value)
    .expect("Schema compilation failed");

// Validate with detailed error messages
if let Err(errors) = schema.validate(&data) {
    for error in errors {
        eprintln!("Validation error: {}", error);
    }
}
```

### 2. Compatibility Test Failures

**Problem**: Tests fail on specific platforms.

**Solution**:
```rust
// Use conditional compilation
#[cfg(target_arch = "wasm32")]
fn create_wasm_client() -> QueryClient {
    // WASM-specific client creation
}

#[cfg(not(target_arch = "wasm32"))]
fn create_native_client() -> QueryClient {
    // Native-specific client creation
}
```

### 3. API Evolution Test Failures

**Problem**: Breaking changes not properly documented.

**Solution**:
```rust
// Ensure all breaking changes have migration paths
fn validate_breaking_changes(version: &ApiVersion) {
    for breaking_change in &version.breaking_changes {
        let migration_path = get_migration_path(&version.version, breaking_change);
        assert!(migration_path.is_some(), 
            "Breaking change '{}' needs migration path", breaking_change);
    }
}
```

## 🔄 Continuous Integration

### GitHub Actions Workflow

```yaml
name: Contract Tests

on: [push, pull_request]

jobs:
  contract-tests:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        platform: [wasm, native, ssr]
        feature: [default, devtools, persistence]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - name: Run Schema Validation Tests
      run: cargo test --test schema_validation_tests
    
    - name: Run API Evolution Tests
      run: cargo test --test api_evolution_tests
    
    - name: Run Compatibility Tests
      run: cargo test --test compatibility_tests --features ${{ matrix.feature }}
```

### Pre-commit Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running contract tests..."

# Run schema validation
cargo test --test schema_validation_tests --quiet
if [ $? -ne 0 ]; then
    echo "Schema validation tests failed"
    exit 1
fi

# Run API evolution tests
cargo test --test api_evolution_tests --quiet
if [ $? -ne 0 ]; then
    echo "API evolution tests failed"
    exit 1
fi

echo "All contract tests passed!"
```

## 📈 Monitoring and Alerting

### Contract Violation Detection

```rust
// Monitor for contract violations
fn monitor_contract_violations() {
    let violations = detect_contract_violations();
    if !violations.is_empty() {
        eprintln!("Contract violations detected:");
        for violation in violations {
            eprintln!("  - {}", violation);
        }
        // Send alert to monitoring system
        send_alert("Contract violations detected", &violations);
    }
}
```

### Performance Monitoring

```rust
// Monitor contract validation performance
fn monitor_validation_performance() {
    let start = std::time::Instant::now();
    let result = validate_all_contracts();
    let duration = start.elapsed();
    
    if duration.as_millis() > 1000 {
        eprintln!("Contract validation took too long: {:?}", duration);
        // Send performance alert
    }
}
```

## 🎯 Best Practices

### 1. Schema Design

- Use clear, descriptive field names
- Provide comprehensive examples
- Include validation constraints
- Document all fields thoroughly

### 2. Test Organization

- Group related tests together
- Use descriptive test names
- Include both positive and negative test cases
- Test edge cases and error conditions

### 3. Documentation

- Document all contract changes
- Provide migration guides for breaking changes
- Include examples in documentation
- Keep compatibility matrix up to date

### 4. Maintenance

- Run contract tests on every commit
- Update schemas when APIs change
- Monitor for contract violations
- Regular review of compatibility matrix

## 🚀 Future Enhancements

### Planned Features

1. **Automated Contract Generation**: Generate schemas from Rust types
2. **Contract Drift Detection**: Detect when implementations drift from contracts
3. **Performance Contract Testing**: Ensure performance SLAs are met
4. **Visual Contract Documentation**: Generate visual documentation from contracts
5. **Contract Versioning**: Advanced versioning strategies for contracts

### Integration Opportunities

1. **OpenAPI Integration**: Generate OpenAPI specs from contracts
2. **TypeScript Definitions**: Generate TypeScript types from schemas
3. **API Documentation**: Auto-generate API docs from contracts
4. **Client SDK Generation**: Generate client SDKs from contracts

This guide provides a comprehensive foundation for implementing and maintaining contract testing in the leptos-query library. By following these practices, you can ensure API stability, compatibility, and reliability across all versions and platforms.
