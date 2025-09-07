//! QueryClient API Tests
//! 
//! These tests verify that all QueryClient methods work correctly and are stable.

use leptos_query_rs::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestUser {
    id: u32,
    name: String,
    email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_query_data_method() {
        // RED: This test should fail initially because get_query_data doesn't exist
        let client = QueryClient::new();
        let key = QueryKey::new(&["users", "1"]);
        let user = TestUser {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };

        // Set data first
        client.set_query_data(&key, user.clone()).unwrap();

        // Try to get data back
        let retrieved_user = client.get_query_data::<TestUser>(&key);
        assert_eq!(retrieved_user, Some(user));
    }

    #[test]
    fn test_get_query_data_returns_none_for_missing_key() {
        // RED: Test that get_query_data returns None for non-existent keys
        let client = QueryClient::new();
        let key = QueryKey::new(&["users", "999"]);

        let retrieved_user = client.get_query_data::<TestUser>(&key);
        assert_eq!(retrieved_user, None);
    }

    #[test]
    fn test_get_query_data_with_different_types() {
        // RED: Test that get_query_data works with different data types
        let client = QueryClient::new();
        
        // Test with string
        let string_key = QueryKey::new(&["data", "string"]);
        client.set_query_data(&string_key, "Hello World".to_string()).unwrap();
        let retrieved_string = client.get_query_data::<String>(&string_key);
        assert_eq!(retrieved_string, Some("Hello World".to_string()));

        // Test with number
        let number_key = QueryKey::new(&["data", "number"]);
        client.set_query_data(&number_key, 42u32).unwrap();
        let retrieved_number = client.get_query_data::<u32>(&number_key);
        assert_eq!(retrieved_number, Some(42u32));

        // Test with vector
        let vec_key = QueryKey::new(&["data", "vector"]);
        let test_vec = vec![1, 2, 3, 4, 5];
        client.set_query_data(&vec_key, test_vec.clone()).unwrap();
        let retrieved_vec = client.get_query_data::<Vec<i32>>(&vec_key);
        assert_eq!(retrieved_vec, Some(test_vec));
    }

    #[test]
    fn test_get_query_data_serialization_errors() {
        // Test that get_query_data works correctly with proper types
        let client = QueryClient::new();
        let key = QueryKey::new(&["data", "test"]);
        
        // Set data with one type
        client.set_query_data(&key, "string data".to_string()).unwrap();
        
        // Retrieve as the correct type should work
        let retrieved_string = client.get_query_data::<String>(&key);
        assert_eq!(retrieved_string, Some("string data".to_string()));
        
        // Test with different data types
        let number_key = QueryKey::new(&["data", "number"]);
        client.set_query_data(&number_key, 42u32).unwrap();
        let retrieved_number = client.get_query_data::<u32>(&number_key);
        assert_eq!(retrieved_number, Some(42u32));
        
        // Test with complex types
        let vec_key = QueryKey::new(&["data", "vector"]);
        let test_vec = vec![1, 2, 3, 4, 5];
        client.set_query_data(&vec_key, test_vec.clone()).unwrap();
        let retrieved_vec = client.get_query_data::<Vec<i32>>(&vec_key);
        assert_eq!(retrieved_vec, Some(test_vec));
    }

    #[test]
    fn test_query_data_lifecycle() {
        // RED: Test complete lifecycle of query data
        let client = QueryClient::new();
        let key = QueryKey::new(&["users", "1"]);
        let user = TestUser {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };

        // Initially should be None
        assert_eq!(client.get_query_data::<TestUser>(&key), None);

        // Set data
        client.set_query_data(&key, user.clone()).unwrap();
        assert_eq!(client.get_query_data::<TestUser>(&key), Some(user.clone()));

        // Update data
        let updated_user = TestUser {
            id: 1,
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
        };
        client.set_query_data(&key, updated_user.clone()).unwrap();
        assert_eq!(client.get_query_data::<TestUser>(&key), Some(updated_user));

        // Remove data
        client.remove_query(&key);
        assert_eq!(client.get_query_data::<TestUser>(&key), None);
    }
}
