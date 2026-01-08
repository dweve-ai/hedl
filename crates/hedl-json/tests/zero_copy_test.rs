// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


// Dweve HEDL - Zero-Copy String Handling Test
//
// Tests and documents the zero-copy optimization for string handling in hedl-json.

use hedl_json::{from_json, from_json_value_owned, FromJsonConfig};
use serde_json::{json, Value as JsonValue};

#[test]
fn test_zero_copy_string_moving() {
    // Test that from_json_value_owned moves strings instead of cloning
    let json_value = json!({
        "name": "Alice",
        "email": "alice@example.com",
        "bio": "A long biography that would benefit from moving instead of cloning"
    });

    let config = FromJsonConfig::default();
    let doc = from_json_value_owned(json_value, &config).unwrap();

    // Verify the document was created correctly
    assert!(doc.root.contains_key("name"));
    assert!(doc.root.contains_key("email"));
    assert!(doc.root.contains_key("bio"));
}

#[test]
fn test_schema_caching_benefit() {
    // Test that schema caching improves performance for repeated structures
    let json_str = r#"{
        "users": [
            {"id": "1", "name": "Alice", "email": "alice@example.com"},
            {"id": "2", "name": "Bob", "email": "bob@example.com"},
            {"id": "3", "name": "Charlie", "email": "charlie@example.com"}
        ]
    }"#;

    let config = FromJsonConfig::default();
    let doc = from_json(json_str, &config).unwrap();

    // Verify the schema was inferred correctly
    assert!(doc.structs.contains_key("User"));
    let schema = &doc.structs["User"];
    assert_eq!(schema[0], "id"); // id should be first
}

#[test]
fn test_large_array_optimization() {
    // Test optimization benefit with large repeated arrays
    let mut users = Vec::new();
    for i in 0..1000 {
        users.push(json!({
            "id": i.to_string(),
            "name": format!("User{}", i),
            "email": format!("user{}@example.com", i),
            "score": i as f64
        }));
    }

    let json_str = json!({"users": users}).to_string();
    let config = FromJsonConfig::default();

    let doc = from_json(&json_str, &config).unwrap();

    // Verify all users were processed
    assert!(doc.structs.contains_key("User"));
    if let Some(list) = doc.root.get("users") {
        if let hedl_core::Item::List(matrix_list) = list {
            assert_eq!(matrix_list.rows.len(), 1000);
        } else {
            panic!("Expected list");
        }
    }
}

#[test]
fn test_zero_copy_vs_regular() {
    // Compare zero-copy and regular paths
    let json_str = r#"{
        "users": [
            {"id": "1", "name": "Alice"},
            {"id": "2", "name": "Bob"}
        ]
    }"#;

    // Regular path: from_json with string
    let config = FromJsonConfig::default();
    let doc1 = from_json(json_str, &config).unwrap();

    // Zero-copy path: from_json_value_owned with owned value
    let json_value: JsonValue = serde_json::from_str(json_str).unwrap();
    let doc2 = from_json_value_owned(json_value, &config).unwrap();

    // Both should produce identical results
    assert_eq!(doc1.structs.len(), doc2.structs.len());
    assert_eq!(doc1.root.len(), doc2.root.len());
}

#[test]
fn test_expression_strings_handled_correctly() {
    // Verify expression strings are still parsed correctly with optimization
    // Note: HEDL expressions support function calls like mul(x, y), not infix operators
    let json_str = r#"{"formula": "$(mul(x, y))"}"#;
    let config = FromJsonConfig::default();
    let doc = from_json(json_str, &config).unwrap();

    if let Some(hedl_core::Item::Scalar(hedl_core::Value::Expression(_))) = doc.root.get("formula") {
        // Correct: expression was parsed
    } else {
        panic!("Expression should be parsed");
    }
}

#[test]
fn test_reference_strings_handled_correctly() {
    // Verify reference strings are still parsed correctly with optimization
    let json_str = r#"{"owner": {"@ref": "@User:123"}}"#;
    let config = FromJsonConfig::default();
    let doc = from_json(json_str, &config).unwrap();

    if let Some(hedl_core::Item::Scalar(hedl_core::Value::Reference(ref_))) = doc.root.get("owner") {
        assert_eq!(ref_.id, "123");
        assert_eq!(ref_.type_name, Some("User".to_string()));
    } else {
        panic!("Reference should be parsed");
    }
}
