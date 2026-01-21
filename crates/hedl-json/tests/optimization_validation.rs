// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Simple validation tests that don't require hedl-test dependency

use hedl_json::{from_json, FromJsonConfig};

#[test]
fn test_basic_array_parsing() {
    let json = r#"{"items": [1, 2, 3]}"#;
    let config = FromJsonConfig::default();
    let result = from_json(json, &config);
    assert!(
        result.is_ok(),
        "Failed to parse basic array: {:?}",
        result.err()
    );
}

#[test]
fn test_object_array_parsing() {
    let json = r#"{"users": [{"id": "1"}, {"id": "2"}]}"#;
    let config = FromJsonConfig::default();
    let result = from_json(json, &config);
    assert!(
        result.is_ok(),
        "Failed to parse object array: {:?}",
        result.err()
    );
}

#[test]
fn test_large_tensor_array() {
    let numbers: Vec<i32> = (0..10000).collect();
    let json_value = serde_json::json!({
        "data": numbers
    });

    let config = FromJsonConfig::default();
    let result = from_json(&json_value.to_string(), &config);
    assert!(
        result.is_ok(),
        "Failed to parse large tensor array: {:?}",
        result.err()
    );
}

#[test]
fn test_large_object_array() {
    let mut users = Vec::new();
    for i in 0..1000 {
        users.push(serde_json::json!({
            "id": format!("u{}", i),
            "name": format!("User {}", i)
        }));
    }

    let json_value = serde_json::json!({
        "users": users
    });

    let config = FromJsonConfig::default();
    let result = from_json(&json_value.to_string(), &config);
    assert!(
        result.is_ok(),
        "Failed to parse large object array: {:?}",
        result.err()
    );
}

#[test]
fn test_wide_object() {
    let mut obj = serde_json::Map::new();
    obj.insert("id".to_string(), serde_json::json!("r1"));
    for i in 0..50 {
        obj.insert(format!("field{i}"), serde_json::json!(i));
    }

    let json_value = serde_json::json!({
        "records": [obj]
    });

    let config = FromJsonConfig::default();
    let result = from_json(&json_value.to_string(), &config);
    assert!(
        result.is_ok(),
        "Failed to parse wide object: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_arrays() {
    let json = r#"{
        "departments": [
            {
                "id": "d1",
                "employees": [
                    {"id": "e1"},
                    {"id": "e2"}
                ]
            }
        ]
    }"#;

    let config = FromJsonConfig::default();
    let result = from_json(json, &config);
    assert!(
        result.is_ok(),
        "Failed to parse nested arrays: {:?}",
        result.err()
    );
}
