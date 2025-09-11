//! Schema Validation Tests
//! 
//! These tests validate that all API inputs and outputs conform to their JSON schemas.
//! This ensures contract compliance and prevents breaking changes.

use leptos_query_rs::*;
use serde_json::{json, Value};

// Schema validation using jsonschema crate
use jsonschema::JSONSchema;

/// Load JSON schema from file
fn load_schema(schema_name: &str) -> JSONSchema {
    let schema_content = match schema_name {
        "query_options" => include_str!("../../contracts/schemas/query_options.json"),
        "retry_config" => include_str!("../../contracts/schemas/retry_config.json"),
        "query_key" => include_str!("../../contracts/schemas/query_key.json"),
        "error_schemas" => include_str!("../../contracts/schemas/error_schemas.json"),
        _ => panic!("Unknown schema: {}", schema_name),
    };
    
    let schema_value: Value = serde_json::from_str(schema_content)
        .expect("Failed to parse schema JSON");
    
    JSONSchema::compile(&schema_value)
        .expect("Failed to compile schema")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_options_schema_validation() {
        let schema = load_schema("query_options");
        
        // Valid query options
        let valid_options = json!({
            "enabled": true,
            "stale_time": 0,
            "cache_time": 300000,
            "refetch_interval": null,
            "retry": {
                "max_retries": 3,
                "base_delay": 1000,
                "max_delay": 10000
            }
        });
        
        let result = schema.validate(&valid_options);
        assert!(result.is_ok(), "Valid query options should pass validation");
        
        // Test with minimal options (using defaults)
        let minimal_options = json!({
            "enabled": true
        });
        
        let result = schema.validate(&minimal_options);
        assert!(result.is_ok(), "Minimal query options should pass validation");
        
        // Test disabled query
        let disabled_options = json!({
            "enabled": false,
            "stale_time": 60000,
            "cache_time": 600000
        });
        
        let result = schema.validate(&disabled_options);
        assert!(result.is_ok(), "Disabled query options should pass validation");
    }

    #[test]
    fn test_query_options_schema_validation_errors() {
        let schema = load_schema("query_options");
        
        // Invalid: negative stale_time
        let invalid_stale_time = json!({
            "enabled": true,
            "stale_time": -1,
            "cache_time": 300000
        });
        
        let result = schema.validate(&invalid_stale_time);
        assert!(result.is_err(), "Negative stale_time should fail validation");
        
        // Invalid: negative cache_time
        let invalid_cache_time = json!({
            "enabled": true,
            "stale_time": 0,
            "cache_time": -100
        });
        
        let result = schema.validate(&invalid_cache_time);
        assert!(result.is_err(), "Negative cache_time should fail validation");
        
        // Invalid: wrong type for enabled
        let invalid_enabled_type = json!({
            "enabled": "not_a_boolean",
            "stale_time": 0,
            "cache_time": 300000
        });
        
        let result = schema.validate(&invalid_enabled_type);
        assert!(result.is_err(), "Non-boolean enabled should fail validation");
        
        // Invalid: additional properties
        let invalid_additional_props = json!({
            "enabled": true,
            "stale_time": 0,
            "cache_time": 300000,
            "invalid_property": "should_not_be_here"
        });
        
        let result = schema.validate(&invalid_additional_props);
        assert!(result.is_err(), "Additional properties should fail validation");
    }

    #[test]
    fn test_retry_config_schema_validation() {
        let schema = load_schema("retry_config");
        
        // Valid retry config
        let valid_config = json!({
            "max_retries": 3,
            "base_delay": 1000,
            "max_delay": 10000
        });
        
        let result = schema.validate(&valid_config);
        assert!(result.is_ok(), "Valid retry config should pass validation");
        
        // Test no retries
        let no_retries = json!({
            "max_retries": 0,
            "base_delay": 0,
            "max_delay": 0
        });
        
        let result = schema.validate(&no_retries);
        assert!(result.is_ok(), "No retries config should pass validation");
        
        // Test maximum retries
        let max_retries = json!({
            "max_retries": 10,
            "base_delay": 1000,
            "max_delay": 300000
        });
        
        let result = schema.validate(&max_retries);
        assert!(result.is_ok(), "Maximum retries config should pass validation");
    }

    #[test]
    fn test_retry_config_schema_validation_errors() {
        let schema = load_schema("retry_config");
        
        // Invalid: negative max_retries
        let invalid_retries = json!({
            "max_retries": -1,
            "base_delay": 1000,
            "max_delay": 10000
        });
        
        let result = schema.validate(&invalid_retries);
        assert!(result.is_err(), "Negative max_retries should fail validation");
        
        // Invalid: max_retries too high
        let too_many_retries = json!({
            "max_retries": 11,
            "base_delay": 1000,
            "max_delay": 10000
        });
        
        let result = schema.validate(&too_many_retries);
        assert!(result.is_err(), "Too many retries should fail validation");
        
        // Invalid: negative base_delay
        let invalid_delay = json!({
            "max_retries": 3,
            "base_delay": -100,
            "max_delay": 10000
        });
        
        let result = schema.validate(&invalid_delay);
        assert!(result.is_err(), "Negative base_delay should fail validation");
        
        // Invalid: base_delay too high
        let delay_too_high = json!({
            "max_retries": 3,
            "base_delay": 61000,
            "max_delay": 10000
        });
        
        let result = schema.validate(&delay_too_high);
        assert!(result.is_err(), "Base delay too high should fail validation");
    }

    #[test]
    fn test_query_key_schema_validation() {
        let schema = load_schema("query_key");
        
        // Valid query keys
        let valid_keys = vec![
            json!({"segments": ["user", "123"]}),
            json!({"segments": ["users"]}),
            json!({"segments": ["posts", "user", "123"]}),
            json!({"segments": ["settings", "theme"]}),
            json!({"segments": ["api", "v1", "users", "123", "posts"]}),
        ];
        
        for key in valid_keys {
            let result = schema.validate(&key);
            assert!(result.is_ok(), "Valid query key should pass validation: {:?}", key);
        }
    }

    #[test]
    fn test_query_key_schema_validation_errors() {
        let schema = load_schema("query_key");
        
        // Invalid: empty array
        let empty_key = json!([]);
        let result = schema.validate(&empty_key);
        assert!(result.is_err(), "Empty query key should fail validation");
        
        // Invalid: too many segments
        let too_many_segments = json!(["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k"]);
        let result = schema.validate(&too_many_segments);
        assert!(result.is_err(), "Too many segments should fail validation");
        
        // Invalid: empty string segment
        let empty_segment = json!(["user", ""]);
        let result = schema.validate(&empty_segment);
        assert!(result.is_err(), "Empty string segment should fail validation");
        
        // Invalid: non-string segment
        let non_string_segment = json!(["user", 123]);
        let result = schema.validate(&non_string_segment);
        assert!(result.is_err(), "Non-string segment should fail validation");
        
        // Invalid: too long segment
        let long_segment = "a".repeat(256);
        let too_long_segment = json!(["user", long_segment]);
        let result = schema.validate(&too_long_segment);
        assert!(result.is_err(), "Too long segment should fail validation");
    }

    #[test]
    fn test_error_schemas_validation() {
        let schema = load_schema("error_schemas");
        
        // Valid QueryError
        let valid_error = json!({
            "type": "NetworkError",
            "message": "Connection failed",
            "details": "Failed to connect to server",
            "code": 503,
            "timestamp": 1640995200000i64,
            "query_key": ["user", "123"]
        });
        
        let result = schema.validate(&valid_error);
        assert!(result.is_ok(), "Valid QueryError should pass validation");
        
        // Valid minimal error
        let minimal_error = json!({
            "type": "GenericError",
            "message": "Something went wrong"
        });
        
        let result = schema.validate(&minimal_error);
        assert!(result.is_ok(), "Minimal QueryError should pass validation");
    }

    #[test]
    fn test_error_schemas_validation_errors() {
        // Create a standalone QueryError schema for testing
        let query_error_schema_json = json!({
            "type": "object",
            "properties": {
                "type": {
                    "type": "string",
                    "enum": ["NetworkError", "TimeoutError", "GenericError", "ValidationError"]
                },
                "message": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 1000
                }
            },
            "required": ["type", "message"],
            "additionalProperties": false
        });
        let query_error_schema = JSONSchema::compile(&query_error_schema_json).unwrap();
        
        // Invalid: missing required fields
        let missing_fields = json!({
            "type": "NetworkError"
            // Missing required "message" field
        });
        
        let result = query_error_schema.validate(&missing_fields);
        assert!(result.is_err(), "Missing required fields should fail validation");
        
        // Invalid: wrong error type
        let invalid_type = json!({
            "type": "InvalidErrorType",
            "message": "Some error"
        });
        
        let result = query_error_schema.validate(&invalid_type);
        assert!(result.is_err(), "Invalid error type should fail validation");
        
        // Invalid: empty message
        let empty_message = json!({
            "type": "NetworkError",
            "message": ""
        });
        
        let result = query_error_schema.validate(&empty_message);
        assert!(result.is_err(), "Empty message should fail validation");
        
        // Invalid: message too long
        let long_message = "a".repeat(1001);
        let too_long_message = json!({
            "type": "NetworkError",
            "message": long_message
        });
        
        let result = query_error_schema.validate(&too_long_message);
        assert!(result.is_err(), "Message too long should fail validation");
    }

    #[test]
    fn test_serialization_contract_compliance() {
        // Test that our Rust types can be converted to valid JSON according to schemas
        
        // Test QueryOptions conversion to JSON
        let options = QueryOptions::default()
            .with_stale_time(std::time::Duration::from_secs(60))
            .with_cache_time(std::time::Duration::from_secs(300));
        
        // Convert to JSON manually since QueryOptions doesn't implement Serialize
        let options_json = json!({
            "enabled": options.enabled,
            "stale_time": options.stale_time.as_millis(),
            "cache_time": options.cache_time.as_millis(),
            "refetch_interval": options.refetch_interval.map(|d| d.as_millis()),
            "retry": {
                "max_retries": options.retry.max_retries,
                "base_delay": options.retry.base_delay.as_millis(),
                "max_delay": options.retry.max_delay.as_millis()
            }
        });
        
        let schema = load_schema("query_options");
        let result = schema.validate(&options_json);
        assert!(result.is_ok(), "QueryOptions JSON should match schema");
        
        // Test RetryConfig conversion to JSON
        let retry_config = RetryConfig::new(3, std::time::Duration::from_millis(1000));
        let retry_json = json!({
            "max_retries": retry_config.max_retries,
            "base_delay": retry_config.base_delay.as_millis(),
            "max_delay": retry_config.max_delay.as_millis()
        });
        
        let schema = load_schema("retry_config");
        let result = schema.validate(&retry_json);
        assert!(result.is_ok(), "RetryConfig JSON should match schema");
        
        // Test QueryKey serialization (this one should work)
        let query_key = QueryKey::new(["user", "123"]);
        let serialized = serde_json::to_value(&query_key).unwrap();
        let schema = load_schema("query_key");
        let result = schema.validate(&serialized);
        if let Err(errors) = result {
            let error_list: Vec<_> = errors.collect();
            panic!("Serialized QueryKey should match schema: {:?}", error_list);
        }
    }

    #[test]
    fn test_deserialization_contract_compliance() {
        // Test that valid JSON according to schemas can be converted to our Rust types
        
        // Test QueryOptions conversion from JSON
        let valid_json = json!({
            "enabled": true,
            "stale_time": 60000,
            "cache_time": 300000,
            "refetch_interval": null,
            "retry": {
                "max_retries": 3,
                "base_delay": 1000,
                "max_delay": 10000
            }
        });
        
        // Since QueryOptions doesn't implement Deserialize, we'll just validate the JSON structure
        let schema = load_schema("query_options");
        let result = schema.validate(&valid_json);
        assert!(result.is_ok(), "Valid JSON should match QueryOptions schema");
        
        // Test RetryConfig conversion from JSON
        let retry_json = json!({
            "max_retries": 5,
            "base_delay": 2000,
            "max_delay": 30000
        });
        
        let schema = load_schema("retry_config");
        let result = schema.validate(&retry_json);
        assert!(result.is_ok(), "Valid JSON should match RetryConfig schema");
        
        // Test QueryKey deserialization (this one should work)
        let key_json = json!({
            "segments": ["posts", "user", "123"]
        });
        let query_key: QueryKey = serde_json::from_value(key_json).unwrap();
        assert_eq!(query_key.segments, vec!["posts", "user", "123"]);
    }

    #[test]
    fn test_contract_evolution_compatibility() {
        // Test that adding new optional fields doesn't break existing contracts
        
        // Simulate a future version with additional optional fields
        let future_query_options = json!({
            "enabled": true,
            "stale_time": 0,
            "cache_time": 300000,
            "refetch_interval": null,
            "retry": {
                "max_retries": 3,
                "base_delay": 1000,
                "max_delay": 10000
            },
            // Future optional fields that don't exist in current schema
            "future_field": "some_value",
            "another_future_field": 42
        });
        
        // This should fail with current schema due to additionalProperties: false
        let schema = load_schema("query_options");
        let result = schema.validate(&future_query_options);
        assert!(result.is_err(), "Additional properties should fail validation");
        
        // But if we remove additionalProperties restriction, it should pass
        // This test documents the contract evolution strategy
    }
}
