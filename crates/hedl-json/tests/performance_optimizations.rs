// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tests for performance optimizations
//!
//! This test suite validates that performance optimizations maintain correctness
//! while reducing allocations and improving throughput.

// Allow approximate float constants in tests - these are intentional test values
#![allow(clippy::approx_constant)]

use hedl_json::string_cache::{clear_string_cache, string_cache_stats};
use hedl_json::{from_json, from_json_value_owned, FromJsonConfig};
use serde_json::json;

#[test]
fn test_string_interning_functionality() {
    clear_string_cache();

    // Test the string interning cache API
    use hedl_json::string_cache::intern_string;
    use std::sync::Arc;

    let s1 = intern_string("test_field");
    let s2 = intern_string("test_field");

    // Same Arc pointer (interned)
    assert!(Arc::ptr_eq(&s1, &s2));

    // Check cache stats
    let stats = string_cache_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert!(stats.hit_rate() > 0.4);
}

#[test]
fn test_empty_array_handling() {
    let json = r#"{"items": []}"#;
    let config = FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();

    // Empty arrays should create empty matrix lists
    assert!(doc.root.contains_key("items"));
}

#[test]
fn test_large_object_performance() {
    // Create JSON with many keys to test object construction
    let mut obj = serde_json::Map::new();
    for i in 0..100 {
        obj.insert(format!("field_{i}"), json!(format!("value_{}", i)));
    }
    let json = serde_json::Value::Object(obj);

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_nested_objects() {
    let json = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "deep"
                }
            }
        }
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_mixed_array_types() {
    let json = json!({
        "numbers": [1, 2, 3, 4, 5],
        "objects": [
            {"id": "1", "value": "first"},
            {"id": "2", "value": "second"}
        ]
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_tensor_arrays() {
    let json = json!({
        "vector": [1.0, 2.0, 3.0],
        "matrix": [[1.0, 2.0], [3.0, 4.0]],
        "tensor3d": [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_schema_inference_caching() {
    clear_string_cache();

    // Multiple arrays with same schema
    let json = json!({
        "users": [
            {"id": "1", "name": "Alice", "age": 30},
            {"id": "2", "name": "Bob", "age": 25}
        ]
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    if let Err(e) = &result {
        eprintln!("Error: {e:?}");
    }
    assert!(result.is_ok());
}

#[test]
fn test_unicode_strings() {
    let json = json!({
        "japanese": "こんにちは",
        "emoji": "🦀 Rust",
        "mixed": "Hello 世界"
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_special_characters() {
    let json = json!({
        "email": "user@example.com",
        "price": "$100",
        "reference": "@special",
        "expression": "$(value)"
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_null_handling() {
    let json = json!({
        "null_value": null,
        "object_with_null": {"key": null}
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_boolean_values() {
    let json = json!({
        "enabled": true,
        "disabled": false
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    if let Err(e) = &result {
        eprintln!("Error: {e:?}");
    }
    assert!(result.is_ok());
}

#[test]
fn test_numeric_values() {
    let json = json!({
        "integer": 42,
        "negative": -17,
        "float": 3.14159,
        "scientific": 1.23e-4,
        "large": 9223372036854775807i64
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_metadata_exclusion() {
    let json = json!({
        "normal_field": "value",
        "__metadata": "should_be_excluded",
        "__internal": {"nested": "data"}
    });

    let config = FromJsonConfig::default();
    let doc = from_json_value_owned(json, &config).unwrap();

    // Metadata fields should be excluded
    assert!(doc.root.contains_key("normal_field"));
    assert!(!doc.root.contains_key("__metadata"));
    assert!(!doc.root.contains_key("__internal"));
}

#[test]
fn test_child_relationships() {
    let json = json!({
        "users": [
            {
                "id": "1",
                "name": "Alice",
                "posts": [
                    {"id": "p1", "title": "First Post"},
                    {"id": "p2", "title": "Second Post"}
                ]
            },
            {
                "id": "2",
                "name": "Bob",
                "posts": [
                    {"id": "p3", "title": "Bob's Post"}
                ]
            }
        ]
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    assert!(result.is_ok());

    let doc = result.unwrap();
    // Should have Users struct registered
    assert!(doc.structs.contains_key("User"));
}

#[test]
fn test_max_depth_limit() {
    // Create deeply nested JSON
    let mut json = json!("value");
    for _ in 0..100 {
        json = json!({"nested": json});
    }

    let config = FromJsonConfig::builder().max_depth(50).build();

    let result = from_json_value_owned(json, &config);
    assert!(result.is_err());
}

#[test]
fn test_max_array_size_limit() {
    let large_array: Vec<i32> = (0..100_000).collect();
    let json = json!({"array": large_array});

    let config = FromJsonConfig::builder().max_array_size(10_000).build();

    let result = from_json_value_owned(json, &config);
    assert!(result.is_err());
}

#[test]
fn test_max_string_length_limit() {
    let long_string = "x".repeat(1_000_000);
    let json = json!({"long_field": long_string});

    let config = FromJsonConfig::builder().max_string_length(100_000).build();

    let result = from_json_value_owned(json, &config);
    assert!(result.is_err());
}

#[test]
fn test_max_object_size_limit() {
    let mut obj = serde_json::Map::new();
    for i in 0..10_000 {
        obj.insert(format!("field_{i}"), json!(i));
    }
    let json = serde_json::Value::Object(obj);

    let config = FromJsonConfig::builder().max_object_size(1_000).build();

    let result = from_json_value_owned(json, &config);
    assert!(result.is_err());
}

#[test]
fn test_unlimited_config() {
    let mut obj = serde_json::Map::new();
    for i in 0..1000 {
        obj.insert(format!("field_{i}"), json!(i));
    }
    let json = serde_json::Value::Object(obj);

    let config = FromJsonConfig::builder().unlimited().build();

    let result = from_json_value_owned(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_realistic_dataset() {
    // Simulate a realistic AI/ML dataset
    let json = json!({
        "metadata": {
            "version": "1.0",
            "created_at": "2024-01-01T00:00:00Z"
        },
        "users": [
            {
                "id": "u1",
                "name": "Alice Smith",
                "email": "alice@example.com",
                "age": 30,
                "embeddings": [0.1, 0.2, 0.3, 0.4, 0.5]
            },
            {
                "id": "u2",
                "name": "Bob Johnson",
                "email": "bob@example.com",
                "age": 25,
                "embeddings": [0.2, 0.3, 0.4, 0.5, 0.6]
            }
        ],
        "training_data": {
            "samples": 1000,
            "accuracy": 0.95,
            "loss": 0.05
        }
    });

    let config = FromJsonConfig::default();
    let result = from_json_value_owned(json, &config);
    if let Err(e) = &result {
        eprintln!("Error: {e:?}");
    }
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert!(doc.root.contains_key("metadata"));
    assert!(doc.root.contains_key("users"));
    assert!(doc.root.contains_key("training_data"));
}
