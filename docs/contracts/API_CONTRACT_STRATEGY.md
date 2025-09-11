# API Contract Strategy for Leptos Query

## 🎯 Overview

This document outlines a comprehensive strategy for implementing robust API contracts and contract testing for the leptos-query library. The goal is to ensure API stability, compatibility, and reliability across versions and integrations.

## 📊 Current State Analysis

### ✅ What We Have
- **API Stability Tests**: Basic contract validation in `tests/api_stability_tests.rs`
- **Comprehensive Documentation**: Complete API reference and design docs
- **Semantic Versioning**: Clear versioning strategy with breaking change policies
- **Type Safety**: Rust's compile-time guarantees for API contracts

### ❌ What We're Missing
- **Formal Contract Specifications**: No OpenAPI/Swagger specs
- **Schema Validation**: No runtime schema validation
- **Consumer-Driven Testing**: No Pact or similar consumer contract testing
- **Version Compatibility Matrix**: No formal compatibility testing
- **API Evolution Tracking**: No formal tracking of API changes

## 🏗️ Implementation Plan

### Phase 1: Foundation (Week 1-2)

#### 1.1 API Contract Specifications
- [ ] **OpenAPI 3.0 Specifications** for REST-like APIs
- [ ] **JSON Schema Definitions** for all data types
- [ ] **TypeScript Definitions** for JavaScript/TypeScript consumers
- [ ] **Rust API Contracts** using trait-based contracts

#### 1.2 Schema Validation Framework
- [ ] **Runtime Schema Validation** using `jsonschema` or `valico`
- [ ] **Input/Output Validation** for all public APIs
- [ ] **Error Schema Validation** for consistent error responses
- [ ] **Serialization Contract Tests** for data integrity

### Phase 2: Contract Testing (Week 3-4)

#### 2.1 Consumer-Driven Contract Testing
- [ ] **Pact Integration** for microservice communication
- [ ] **Consumer Contract Tests** for each API consumer
- [ ] **Provider Contract Tests** to verify API compliance
- [ ] **Contract Verification Pipeline** in CI/CD

#### 2.2 API Version Compatibility
- [ ] **Version Compatibility Matrix** testing
- [ ] **Backward Compatibility Tests** for all versions
- [ ] **Migration Path Validation** for breaking changes
- [ ] **Deprecation Warning Tests** for deprecated APIs

### Phase 3: Advanced Features (Week 5-6)

#### 3.1 API Evolution Tracking
- [ ] **API Change Detection** using AST analysis
- [ ] **Breaking Change Detection** automated scanning
- [ ] **Contract Drift Detection** between versions
- [ ] **API Usage Analytics** for optimization

#### 3.2 Integration Testing
- [ ] **Cross-Platform Contract Tests** (WASM, Native, SSR)
- [ ] **Framework Integration Tests** (Leptos 0.8 compatibility)
- [ ] **Ecosystem Integration Tests** (with other libraries)
- [ ] **Performance Contract Tests** for SLA compliance

## 🔧 Technical Implementation

### 1. Contract Specifications

#### OpenAPI 3.0 Specification
```yaml
openapi: 3.0.3
info:
  title: Leptos Query API
  version: 0.5.1
  description: Data fetching and caching library for Leptos

paths:
  /query:
    post:
      summary: Execute a query
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/QueryRequest'
      responses:
        '200':
          description: Query result
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/QueryResponse'

components:
  schemas:
    QueryRequest:
      type: object
      required: [key, options]
      properties:
        key:
          type: array
          items:
            type: string
        options:
          $ref: '#/components/schemas/QueryOptions'
    
    QueryResponse:
      type: object
      properties:
        data:
          type: object
        error:
          $ref: '#/components/schemas/QueryError'
        status:
          type: string
          enum: [loading, success, error]
```

#### JSON Schema Definitions
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "QueryKey": {
      "type": "array",
      "items": {
        "type": "string"
      },
      "minItems": 1
    },
    "QueryOptions": {
      "type": "object",
      "properties": {
        "enabled": {
          "type": "boolean",
          "default": true
        },
        "stale_time": {
          "type": "integer",
          "minimum": 0
        },
        "cache_time": {
          "type": "integer",
          "minimum": 0
        }
      }
    }
  }
}
```

### 2. Contract Testing Framework

#### Pact Consumer Tests
```rust
use pact_consumer::prelude::*;
use serde_json::json;

#[tokio::test]
async fn test_query_contract() {
    let mut pact_builder = PactBuilder::new("leptos-query-client", "leptos-query-server");
    
    pact_builder
        .interaction("execute query", |mut i| {
            i.given("query client is available");
            i.request
                .post()
                .path("/query")
                .header("content-type", "application/json")
                .json_body(json!({
                    "key": ["user", "123"],
                    "options": {
                        "enabled": true,
                        "stale_time": 0,
                        "cache_time": 300
                    }
                }));
            i.response
                .status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "data": {
                        "id": 123,
                        "name": "John Doe",
                        "email": "john@example.com"
                    },
                    "status": "success"
                }));
        })
        .await;
    
    let pact = pact_builder.build();
    // Verify contract
    pact.verify().await;
}
```

#### Schema Validation Tests
```rust
use jsonschema::{JSONSchema, ValidationError};
use serde_json::Value;

#[test]
fn test_query_options_schema_validation() {
    let schema = include_str!("../schemas/query_options.json");
    let compiled_schema = JSONSchema::compile(&serde_json::from_str(schema).unwrap()).unwrap();
    
    // Valid options
    let valid_options = json!({
        "enabled": true,
        "stale_time": 60,
        "cache_time": 300
    });
    
    let result = compiled_schema.validate(&valid_options);
    assert!(result.is_ok());
    
    // Invalid options
    let invalid_options = json!({
        "enabled": "not_a_boolean",
        "stale_time": -1
    });
    
    let result = compiled_schema.validate(&invalid_options);
    assert!(result.is_err());
}
```

### 3. Version Compatibility Testing

#### Compatibility Matrix Tests
```rust
#[test]
fn test_api_compatibility_matrix() {
    let versions = ["0.4.0", "0.4.1", "0.5.0", "0.5.1"];
    
    for version in &versions {
        // Test that each version can handle the same API calls
        let client = create_client_for_version(version);
        let result = client.execute_query(create_test_query());
        assert!(result.is_ok(), "Version {} failed compatibility test", version);
    }
}

#[test]
fn test_backward_compatibility() {
    // Test that new versions can handle old API calls
    let old_api_call = create_v0_4_0_api_call();
    let new_client = QueryClient::new();
    let result = new_client.execute_legacy_query(old_api_call);
    assert!(result.is_ok(), "Backward compatibility broken");
}
```

## 📁 File Structure

```
leptos-query/
├── contracts/
│   ├── openapi/
│   │   ├── leptos-query-api.yaml
│   │   └── devtools-api.yaml
│   ├── schemas/
│   │   ├── query_options.json
│   │   ├── mutation_options.json
│   │   └── error_schemas.json
│   ├── pacts/
│   │   ├── consumer-tests/
│   │   └── provider-tests/
│   └── compatibility/
│       ├── version-matrix.json
│       └── migration-tests/
├── tests/
│   ├── contracts/
│   │   ├── schema_validation_tests.rs
│   │   ├── pact_consumer_tests.rs
│   │   ├── compatibility_tests.rs
│   │   └── api_evolution_tests.rs
│   └── integration/
│       └── contract_integration_tests.rs
└── docs/
    └── contracts/
        ├── API_CONTRACT_STRATEGY.md
        ├── CONTRACT_TESTING_GUIDE.md
        └── VERSION_COMPATIBILITY.md
```

## 🚀 Implementation Timeline

### Week 1: Foundation
- [ ] Set up contract specification structure
- [ ] Create OpenAPI specifications for core APIs
- [ ] Implement JSON schema definitions
- [ ] Set up schema validation framework

### Week 2: Core Contract Testing
- [ ] Implement schema validation tests
- [ ] Create contract verification pipeline
- [ ] Set up Pact consumer testing
- [ ] Implement basic compatibility tests

### Week 3: Advanced Testing
- [ ] Implement provider contract tests
- [ ] Create version compatibility matrix
- [ ] Set up API evolution tracking
- [ ] Implement breaking change detection

### Week 4: Integration & Documentation
- [ ] Integrate contract tests into CI/CD
- [ ] Create comprehensive documentation
- [ ] Set up monitoring and alerting
- [ ] Performance testing for contracts

## 📊 Success Metrics

### Contract Coverage
- **API Surface Coverage**: 100% of public APIs covered
- **Schema Validation**: 100% of data types validated
- **Version Compatibility**: All supported versions tested
- **Consumer Coverage**: All known consumers tested

### Quality Metrics
- **Contract Compliance**: 100% compliance rate
- **Breaking Change Detection**: 0% undetected breaking changes
- **Schema Validation**: 100% valid data passing validation
- **Performance Impact**: < 1% overhead from contract validation

### Process Metrics
- **Test Execution Time**: < 5 minutes for full contract test suite
- **Contract Update Time**: < 1 day for new contract versions
- **Compatibility Verification**: < 2 hours for full compatibility matrix
- **Documentation Coverage**: 100% of contracts documented

## 🔄 Maintenance Strategy

### Regular Updates
- **Weekly**: Run full contract test suite
- **Monthly**: Update compatibility matrix
- **Quarterly**: Review and update contract specifications
- **Per Release**: Validate all contracts for new versions

### Monitoring & Alerting
- **Contract Violations**: Immediate alerts for any violations
- **Performance Degradation**: Alerts for contract validation overhead
- **Compatibility Issues**: Alerts for compatibility test failures
- **Schema Drift**: Alerts for unexpected schema changes

## 🎯 Next Steps

1. **Immediate**: Start with Phase 1 foundation work
2. **Short-term**: Implement core contract testing framework
3. **Medium-term**: Add advanced features and monitoring
4. **Long-term**: Continuous improvement and optimization

This strategy provides a comprehensive approach to API contract management that will ensure the reliability and stability of the leptos-query library while maintaining excellent developer experience.
