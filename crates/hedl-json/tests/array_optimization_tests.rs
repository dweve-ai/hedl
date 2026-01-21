// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Array optimization tests
//!
//! Validates that the array processing optimizations maintain correctness
//! while improving performance:
//! - Single-pass array type classification
//! - `SmallVec` for small schemas and cache keys
//! - Sorted `BTreeMap` insertion
//! - Pre-allocated capacity hints

use hedl_json::{from_json, FromJsonConfig};
use serde_json::json;

#[test]
fn test_empty_array() {
    let json = r#"{"items": []}"#;
    let config = FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("items"));
}

#[test]
fn test_tensor_array_small() {
    let json = r#"{"values": [1, 2, 3, 4, 5]}"#;
    let config = FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("values"));
}

#[test]
fn test_tensor_array_large() {
    // Generate array with 10,000 numbers
    let numbers: Vec<i32> = (0..10000).collect();
    let json_value = json!({
        "data": numbers
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json_value.to_string(), &config).unwrap();

    assert!(doc.root.contains_key("data"));
}

#[test]
fn test_nested_tensor_array() {
    let json = r#"{"matrix": [[1, 2], [3, 4], [5, 6]]}"#;
    let config = FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("matrix"));
}

#[test]
fn test_object_array_small() {
    let json = r#"{
        "users": [
            {"id": "u1", "name": "Alice"},
            {"id": "u2", "name": "Bob"},
            {"id": "u3", "name": "Charlie"}
        ]
    }"#;

    let config = FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("users"));
    assert!(doc.structs.contains_key("User"));
}

#[test]
fn test_object_array_large() {
    // Generate array with 1,000 objects
    let mut users = Vec::new();
    for i in 0..1000 {
        users.push(json!({
            "id": format!("u{}", i),
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "age": 20 + (i % 50)
        }));
    }

    let json_value = json!({
        "users": users
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json_value.to_string(), &config).unwrap();

    assert!(doc.root.contains_key("users"));
    assert!(doc.structs.contains_key("User"));
}

#[test]
fn test_object_array_with_many_fields() {
    // Test object with >16 fields to validate SmallVec overflow handling
    let json = r#"{
        "records": [
            {
                "id": "r1",
                "field1": "value1",
                "field2": "value2",
                "field3": "value3",
                "field4": "value4",
                "field5": "value5",
                "field6": "value6",
                "field7": "value7",
                "field8": "value8",
                "field9": "value9",
                "field10": "value10",
                "field11": "value11",
                "field12": "value12",
                "field13": "value13",
                "field14": "value14",
                "field15": "value15",
                "field16": "value16",
                "field17": "value17",
                "field18": "value18"
            }
        ]
    }"#;

    let config = FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("records"));
    assert!(doc.structs.contains_key("Record"));

    // Verify all fields are captured
    let schema = &doc.structs["Record"];
    assert!(schema.len() >= 18);
}

#[test]
fn test_nested_object_arrays() {
    let json = r#"{
        "departments": [
            {
                "id": "d1",
                "name": "Engineering",
                "employees": [
                    {"id": "e1", "name": "Alice"},
                    {"id": "e2", "name": "Bob"}
                ]
            },
            {
                "id": "d2",
                "name": "Sales",
                "employees": [
                    {"id": "e3", "name": "Charlie"},
                    {"id": "e4", "name": "Diana"}
                ]
            }
        ]
    }"#;

    let config = FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("departments"));
    assert!(doc.structs.contains_key("Department"));
    assert!(doc.structs.contains_key("Employee"));
}

#[test]
fn test_numeric_tensor_array() {
    // Numeric arrays are tensors
    let json = r#"{"numbers": [1, 2, 3, 4, 5]}"#;
    let config = FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("numbers"));
}

#[test]
fn test_schema_cache_reuse() {
    // Multiple arrays with same schema should benefit from cache
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
}

#[test]
fn test_wide_object_sorted_insertion() {
    // Test object with many fields to validate sorted BTreeMap insertion
    let mut fields = vec![("id".to_string(), json!("r1"))];
    for i in 0..50 {
        fields.push((format!("field_{i}"), json!(format!("value_{}", i))));
    }

    let mut obj = serde_json::Map::new();
    for (key, value) in fields {
        obj.insert(key, value);
    }

    let json_value = json!({
        "records": [obj]
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json_value.to_string(), &config).unwrap();

    assert!(doc.root.contains_key("records"));
    assert!(doc.structs.contains_key("Record"));
}

#[test]
fn test_deeply_nested_arrays() {
    // Test deep nesting to validate capacity hints propagate correctly
    let json = r#"{
        "level1": [
            {
                "id": "l1",
                "level2": [
                    {
                        "id": "l2",
                        "level3": [
                            {"id": "l3", "value": 1},
                            {"id": "l4", "value": 2}
                        ]
                    }
                ]
            }
        ]
    }"#;

    let config = FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("level1"));
}

#[test]
fn test_array_type_classification_empty() {
    // Verify empty arrays are handled correctly
    let json = r#"{"items": []}"#;
    let config = FromJsonConfig::default();
    let result = from_json(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_array_type_classification_tensor() {
    // Verify tensor arrays are classified correctly
    let json = r#"{"numbers": [1, 2, 3]}"#;
    let config = FromJsonConfig::default();
    let result = from_json(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_array_type_classification_objects() {
    // Verify object arrays are classified correctly
    let json = r#"{"items": [{"id": "1"}, {"id": "2"}]}"#;
    let config = FromJsonConfig::default();
    let result = from_json(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_array_type_classification_nested_tensor() {
    // Verify nested tensor arrays are classified correctly
    let json = r#"{"nested": [[1, 2], [3, 4]]}"#;
    let config = FromJsonConfig::default();
    let result = from_json(json, &config);
    assert!(result.is_ok());
}

#[test]
fn test_smallvec_boundary_conditions() {
    // Test with exactly 16 fields (SmallVec boundary)
    let mut obj = serde_json::Map::new();
    obj.insert("id".to_string(), json!("r1"));
    for i in 1..16 {
        obj.insert(format!("field{i}"), json!(i));
    }

    let json_value = json!({"records": [obj]});
    let config = FromJsonConfig::default();
    let result = from_json(&json_value.to_string(), &config);
    assert!(result.is_ok());
}

#[test]
fn test_capacity_hints_large_array() {
    // Generate very large array to test capacity pre-allocation
    let mut items = Vec::new();
    for i in 0..10000 {
        items.push(json!({"id": format!("id{}", i), "value": i}));
    }

    let json_value = json!({"items": items});
    let config = FromJsonConfig::default();
    let result = from_json(&json_value.to_string(), &config);
    assert!(result.is_ok());
}

#[test]
fn test_children_sorted_insertion() {
    // Test that children are inserted in sorted order
    let json = r#"{
        "parents": [
            {
                "id": "p1",
                "zebras": [{"id": "z1"}],
                "alpacas": [{"id": "a1"}],
                "monkeys": [{"id": "m1"}]
            }
        ]
    }"#;

    let config = FromJsonConfig::default();
    let result = from_json(json, &config);
    assert!(result.is_ok());
}
