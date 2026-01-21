// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Property-based tests for array optimizations
//!
//! Validates that optimizations maintain correctness across a wide range of inputs

use hedl_json::{from_json, FromJsonConfig};
use serde_json::json;

/// Property: Empty arrays should always parse successfully
#[test]
fn prop_empty_arrays_always_parse() {
    let json = r#"{"items": []}"#;
    let config = FromJsonConfig::default();
    let result = from_json(json, &config);
    assert!(result.is_ok());
}

/// Property: Tensor arrays of any size should parse successfully
#[test]
fn prop_tensor_arrays_parse_all_sizes() {
    for size in [0, 1, 10, 100, 1000, 10000] {
        let numbers: Vec<i32> = (0..size).collect();
        let json_value = json!({"data": numbers});
        let config = FromJsonConfig::default();
        let result = from_json(&json_value.to_string(), &config);
        assert!(
            result.is_ok(),
            "Failed for tensor array size {}: {:?}",
            size,
            result.err()
        );
    }
}

/// Property: Object arrays with varying schema sizes should parse
#[test]
fn prop_object_arrays_parse_varying_schemas() {
    for field_count in [1, 5, 10, 16, 20, 32, 50] {
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), json!("test"));

        for i in 0..field_count {
            obj.insert(format!("field{i}"), json!(i));
        }

        let json_value = json!({"records": [obj]});
        let config = FromJsonConfig::default();
        let result = from_json(&json_value.to_string(), &config);
        assert!(
            result.is_ok(),
            "Failed for schema with {} fields: {:?}",
            field_count,
            result.err()
        );
    }
}

/// Property: Schema cache should work with multiple arrays of same structure
#[test]
fn prop_schema_cache_correctness() {
    let json = r#"{
        "users": [
            {"id": "u1", "name": "Alice"},
            {"id": "u2", "name": "Bob"}
        ],
        "admins": [
            {"id": "a1", "name": "Admin1"},
            {"id": "a2", "name": "Admin2"}
        ]
    }"#;

    let config = FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("users"));
    assert!(doc.root.contains_key("admins"));
    assert!(doc.structs.contains_key("User"));
    assert!(doc.structs.contains_key("Admin"));
}

/// Property: Nested arrays should maintain structure at all depths
#[test]
fn prop_nested_arrays_maintain_structure() {
    for depth in 1..=5 {
        let mut json_value = json!([{"id": "leaf", "value": 42}]);

        for _ in 0..depth {
            json_value = json!([{"id": "parent", "children": json_value}]);
        }

        let json_obj = json!({"root": json_value});
        let config = FromJsonConfig::default();
        let result = from_json(&json_obj.to_string(), &config);

        assert!(
            result.is_ok(),
            "Failed for nesting depth {}: {:?}",
            depth,
            result.err()
        );
    }
}

/// Property: Wide objects should use correct insertion strategy
#[test]
fn prop_wide_objects_insertion_strategy() {
    // Test boundary cases for adaptive strategy (threshold = 32)
    for field_count in [1, 16, 31, 32, 33, 64] {
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), json!("test"));

        for i in 0..field_count {
            obj.insert(format!("field{i}"), json!(i));
        }

        let json_value = json!(obj);
        let config = FromJsonConfig::default();
        let result = from_json(&json_value.to_string(), &config);

        assert!(
            result.is_ok(),
            "Failed for object with {} fields: {:?}",
            field_count,
            result.err()
        );
    }
}

/// Property: `SmallVec` boundary (16 fields) should work correctly
#[test]
fn prop_smallvec_boundary() {
    for field_count in [14, 15, 16, 17, 18] {
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), json!("test"));

        for i in 0..field_count {
            obj.insert(format!("field{i}"), json!(i));
        }

        let json_value = json!({"items": [obj]});
        let config = FromJsonConfig::default();
        let result = from_json(&json_value.to_string(), &config);

        assert!(
            result.is_ok(),
            "Failed at SmallVec boundary with {} fields: {:?}",
            field_count,
            result.err()
        );
    }
}

/// Property: Children insertion strategy boundary (8 children)
#[test]
fn prop_children_insertion_boundary() {
    for child_count in [1, 7, 8, 9, 16] {
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), json!("parent"));

        for i in 0..child_count {
            obj.insert(format!("child{i}"), json!([{"id": format!("c{}", i)}]));
        }

        let json_value = json!({"parents": [obj]});
        let config = FromJsonConfig::default();
        let result = from_json(&json_value.to_string(), &config);

        assert!(
            result.is_ok(),
            "Failed with {} children: {:?}",
            child_count,
            result.err()
        );
    }
}

/// Property: Homogeneous arrays should parse correctly
#[test]
fn prop_homogeneous_arrays_parse() {
    // Test different types of homogeneous arrays
    let test_cases = vec![
        (json!([1, 2, 3]), "all numbers"),
        (json!([[1, 2], [3, 4]]), "nested arrays"),
        (json!([{"id": "1"}, {"id": "2"}]), "all objects"),
    ];

    for (mixed_array, description) in test_cases {
        let json_value = json!({"data": mixed_array});
        let config = FromJsonConfig::default();
        let result = from_json(&json_value.to_string(), &config);

        assert!(
            result.is_ok(),
            "Failed for {}: {:?}",
            description,
            result.err()
        );
    }
}

/// Property: Large arrays should not cause stack overflow
#[test]
fn prop_large_arrays_no_overflow() {
    let sizes = [10_000, 50_000, 100_000];

    for size in sizes {
        let numbers: Vec<i32> = (0..size).collect();
        let json_value = json!({"data": numbers});
        let config = FromJsonConfig::default();
        let result = from_json(&json_value.to_string(), &config);

        assert!(
            result.is_ok(),
            "Failed for large array size {}: {:?}",
            size,
            result.err()
        );
    }
}

/// Property: Deeply nested structures should parse correctly
#[test]
fn prop_deep_nesting_correctness() {
    let json = r#"{
        "level1": [
            {
                "id": "l1",
                "level2": [
                    {
                        "id": "l2",
                        "level3": [
                            {
                                "id": "l3",
                                "level4": [
                                    {"id": "l4", "value": 1}
                                ]
                            }
                        ]
                    }
                ]
            }
        ]
    }"#;

    let config = FromJsonConfig::default();
    let result = from_json(json, &config);
    assert!(
        result.is_ok(),
        "Failed for deep nesting: {:?}",
        result.err()
    );
}
