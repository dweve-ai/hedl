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

//! Comprehensive tests for hedl-json conversion
//!
//! Tests bidirectional conversion between HEDL documents and JSON.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_json::{
    from_json, from_json_value, hedl_to_json, json_to_hedl, to_json, to_json_value, FromJsonConfig,
    ToJsonConfig,
};
use hedl_core::lex::Tensor;
use hedl_test::fixtures;
use serde_json::{json, Value as JsonValue};
use std::collections::BTreeMap;

// =============================================================================
// Basic Scalar Conversion Tests
// =============================================================================

#[test]
fn test_null_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Null));

    let json = hedl_to_json(&doc).unwrap();
    assert!(json.contains("null"));
}

#[test]
fn test_bool_true_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(true)));

    let json = hedl_to_json(&doc).unwrap();
    assert!(json.contains("true"));
}

#[test]
fn test_bool_false_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(false)));

    let json = hedl_to_json(&doc).unwrap();
    assert!(json.contains("false"));
}

#[test]
fn test_int_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("count".to_string(), Item::Scalar(Value::Int(42)));

    let json = hedl_to_json(&doc).unwrap();
    assert!(json.contains("42"));
}

#[test]
fn test_negative_int_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Int(-100)));

    let json = hedl_to_json(&doc).unwrap();
    assert!(json.contains("-100"));
}

#[test]
fn test_float_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("pi".to_string(), Item::Scalar(Value::Float(3.5)));

    let json = hedl_to_json(&doc).unwrap();
    assert!(json.contains("3.5"));
}

#[test]
fn test_string_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "name".to_string(),
        Item::Scalar(Value::String("hello world".to_string())),
    );

    let json = hedl_to_json(&doc).unwrap();
    assert!(json.contains("\"hello world\""));
}

#[test]
fn test_string_with_special_chars_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("line1\nline2\ttab".to_string())),
    );

    let json = hedl_to_json(&doc).unwrap();
    // JSON should escape newlines and tabs
    let parsed: JsonValue = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["text"].as_str().unwrap(), "line1\nline2\ttab");
}

// =============================================================================
// Reference Conversion Tests
// =============================================================================

#[test]
fn test_local_reference_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target_id"))),
    );

    let json = hedl_to_json(&doc).unwrap();
    assert!(json.contains("@ref"));
    assert!(json.contains("@target_id"));
}

#[test]
fn test_qualified_reference_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "alice"))),
    );

    let json = hedl_to_json(&doc).unwrap();
    assert!(json.contains("@User:alice"));
}

#[test]
fn test_reference_from_json() {
    let json = r#"{"ref": {"@ref": "@User:alice"}}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = doc.root.get("ref") {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "alice");
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_local_reference_from_json() {
    let json = r#"{"ref": {"@ref": "@some_id"}}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = doc.root.get("ref") {
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "some_id");
    } else {
        panic!("Expected reference");
    }
}

// =============================================================================
// Expression Conversion Tests
// =============================================================================

#[test]
fn test_expression_to_json() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "expr".to_string(),
        Item::Scalar(hedl_test::expr_value("add(x, multiply(y, 2))")),
    );

    let json = hedl_to_json(&doc).unwrap();
    assert!(json.contains("$(add(x, multiply(y, 2)))"));
}

#[test]
fn test_expression_from_json() {
    let json = r#"{"expr": "$(add(a, b))"}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Some(Item::Scalar(Value::Expression(e))) = doc.root.get("expr") {
        assert_eq!(format!("{}", e), "add(a, b)");
    } else {
        panic!("Expected expression");
    }
}

#[test]
fn test_nested_expression_from_json() {
    let json = r#"{"expr": "$(f(g(x), y))"}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Some(Item::Scalar(Value::Expression(e))) = doc.root.get("expr") {
        assert_eq!(format!("{}", e), "f(g(x), y)");
    } else {
        panic!("Expected expression");
    }
}

// =============================================================================
// Tensor Conversion Tests
// =============================================================================

#[test]
fn test_1d_tensor_to_json() {
    let mut doc = Document::new((1, 0));
    let tensor = Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ]);
    doc.root
        .insert("data".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let json = hedl_to_json(&doc).unwrap();
    let parsed: JsonValue = serde_json::from_str(&json).unwrap();

    assert!(parsed["data"].is_array());
    assert_eq!(parsed["data"].as_array().unwrap().len(), 3);
}

#[test]
fn test_2d_tensor_to_json() {
    let mut doc = Document::new((1, 0));
    let tensor = Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ]);
    doc.root
        .insert("matrix".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let json = hedl_to_json(&doc).unwrap();
    let parsed: JsonValue = serde_json::from_str(&json).unwrap();

    assert!(parsed["matrix"].is_array());
    assert_eq!(parsed["matrix"].as_array().unwrap().len(), 2);
    assert!(parsed["matrix"][0].is_array());
}

#[test]
fn test_tensor_from_json() {
    let json = r#"{"data": [1.0, 2.0, 3.0]}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Some(Item::Scalar(Value::Tensor(t))) = doc.root.get("data") {
        if let Tensor::Array(items) = t {
            assert_eq!(items.len(), 3);
        } else {
            panic!("Expected tensor array");
        }
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_nested_tensor_from_json() {
    let json = r#"{"matrix": [[1, 2], [3, 4]]}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(rows)))) = doc.root.get("matrix") {
        assert_eq!(rows.len(), 2);
        if let Tensor::Array(cols) = &rows[0] {
            assert_eq!(cols.len(), 2);
        } else {
            panic!("Expected nested array");
        }
    } else {
        panic!("Expected nested tensor");
    }
}

// =============================================================================
// Object Conversion Tests
// =============================================================================

#[test]
fn test_nested_object_to_json() {
    let mut doc = Document::new((1, 0));
    let mut inner = BTreeMap::new();
    inner.insert("x".to_string(), Item::Scalar(Value::Int(10)));
    inner.insert("y".to_string(), Item::Scalar(Value::Int(20)));
    doc.root.insert("point".to_string(), Item::Object(inner));

    let json = hedl_to_json(&doc).unwrap();
    let parsed: JsonValue = serde_json::from_str(&json).unwrap();

    assert!(parsed["point"].is_object());
    assert_eq!(parsed["point"]["x"], 10);
    assert_eq!(parsed["point"]["y"], 20);
}

#[test]
fn test_deeply_nested_object_to_json() {
    let mut doc = Document::new((1, 0));

    let mut level3 = BTreeMap::new();
    level3.insert(
        "value".to_string(),
        Item::Scalar(Value::String("deep".to_string())),
    );

    let mut level2 = BTreeMap::new();
    level2.insert("level3".to_string(), Item::Object(level3));

    let mut level1 = BTreeMap::new();
    level1.insert("level2".to_string(), Item::Object(level2));

    doc.root.insert("level1".to_string(), Item::Object(level1));

    let json = hedl_to_json(&doc).unwrap();
    let parsed: JsonValue = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["level1"]["level2"]["level3"]["value"], "deep");
}

#[test]
fn test_object_from_json() {
    let json = r#"{"config": {"host": "localhost", "port": 8080}}"#;
    let doc = json_to_hedl(json).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("config") {
        assert_eq!(
            obj.get("host").unwrap().as_scalar().unwrap(),
            &Value::String("localhost".to_string())
        );
        assert_eq!(
            obj.get("port").unwrap().as_scalar().unwrap(),
            &Value::Int(8080)
        );
    } else {
        panic!("Expected object");
    }
}

// =============================================================================
// Matrix List Conversion Tests
// =============================================================================

#[test]
fn test_matrix_list_to_json() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "User",
        vec!["id".to_string(), "name".to_string(), "age".to_string()],
    );

    list.add_row(Node::new(
        "User",
        "alice",
        vec![Value::String("Alice".to_string()), Value::Int(30)],
    ));
    list.add_row(Node::new(
        "User",
        "bob",
        vec![Value::String("Bob".to_string()), Value::Int(25)],
    ));

    doc.root.insert("users".to_string(), Item::List(list));

    let json = hedl_to_json(&doc).unwrap();
    let parsed: JsonValue = serde_json::from_str(&json).unwrap();

    assert!(parsed["users"].is_array());
    assert_eq!(parsed["users"].as_array().unwrap().len(), 2);
}

#[test]
fn test_matrix_list_with_metadata() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new("Item", "i1", vec![Value::Int(100)]));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToJsonConfig {
        include_metadata: true,
        flatten_lists: false,
        include_children: false,
    };

    let json = to_json(&doc, &config).unwrap();
    let parsed: JsonValue = serde_json::from_str(&json).unwrap();

    // Should have __type__ and __schema__ metadata
    assert!(parsed["items"]["__type__"].is_string());
    assert!(parsed["items"]["__schema__"].is_array());
}

#[test]
fn test_matrix_list_flattened() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new("Item", "i1", vec![Value::Int(100)]));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToJsonConfig {
        include_metadata: false,
        flatten_lists: true,
        include_children: false,
    };

    let json = to_json(&doc, &config).unwrap();
    let parsed: JsonValue = serde_json::from_str(&json).unwrap();

    // Should be a plain array
    assert!(parsed["items"].is_array());
}

#[test]
fn test_matrix_list_from_json() {
    let json = r#"{
        "users": [
            {"id": "alice", "name": "Alice", "age": 30},
            {"id": "bob", "name": "Bob", "age": 25}
        ]
    }"#;

    let doc = json_to_hedl(json).unwrap();

    if let Some(Item::List(list)) = doc.root.get("users") {
        assert_eq!(list.rows.len(), 2);
        assert_eq!(list.rows[0].id, "alice");
        assert_eq!(list.rows[1].id, "bob");
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Round-Trip Tests
// =============================================================================

#[test]
fn test_round_trip_scalars() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("null_val".to_string(), Item::Scalar(Value::Null));
    doc.root
        .insert("bool_val".to_string(), Item::Scalar(Value::Bool(true)));
    doc.root
        .insert("int_val".to_string(), Item::Scalar(Value::Int(42)));
    doc.root
        .insert("float_val".to_string(), Item::Scalar(Value::Float(3.25)));
    doc.root.insert(
        "string_val".to_string(),
        Item::Scalar(Value::String("test".to_string())),
    );

    let json = hedl_to_json(&doc).unwrap();
    let restored = json_to_hedl(&json).unwrap();

    assert_eq!(
        restored.root.get("bool_val").unwrap().as_scalar().unwrap(),
        &Value::Bool(true)
    );
    assert_eq!(
        restored.root.get("int_val").unwrap().as_scalar().unwrap(),
        &Value::Int(42)
    );
    assert_eq!(
        restored
            .root
            .get("string_val")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::String("test".to_string())
    );
}

#[test]
fn test_round_trip_reference() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("Type", "id"))),
    );

    let json = hedl_to_json(&doc).unwrap();
    let restored = json_to_hedl(&json).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("ref") {
        assert_eq!(r.type_name, Some("Type".to_string()));
        assert_eq!(r.id, "id");
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_round_trip_expression() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "expr".to_string(),
        Item::Scalar(hedl_test::expr_value("add(x, 1)")),
    );

    let json = hedl_to_json(&doc).unwrap();
    let restored = json_to_hedl(&json).unwrap();

    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("expr") {
        assert_eq!(format!("{}", e), "add(x, 1)");
    } else {
        panic!("Expected expression");
    }
}

#[test]
fn test_round_trip_tensor() {
    let mut doc = Document::new((1, 0));
    let tensor = Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ]);
    doc.root
        .insert("data".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let json = hedl_to_json(&doc).unwrap();
    let restored = json_to_hedl(&json).unwrap();

    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(items)))) = restored.root.get("data") {
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_round_trip_nested_object() {
    let mut doc = Document::new((1, 0));
    let mut inner = BTreeMap::new();
    inner.insert(
        "key".to_string(),
        Item::Scalar(Value::String("value".to_string())),
    );
    doc.root.insert("outer".to_string(), Item::Object(inner));

    let json = hedl_to_json(&doc).unwrap();
    let restored = json_to_hedl(&json).unwrap();

    let outer = restored.root.get("outer").unwrap().as_object().unwrap();
    assert_eq!(
        outer.get("key").unwrap().as_scalar().unwrap(),
        &Value::String("value".to_string())
    );
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_invalid_json_error() {
    let result = json_to_hedl("{ invalid json }");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("JSON parse error"));
}

#[test]
fn test_non_object_root_error() {
    let result = json_to_hedl("[1, 2, 3]");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Root must be a JSON object"));
}

#[test]
fn test_invalid_reference_format() {
    let json = r#"{"ref": {"@ref": "invalid"}}"#;
    let result = json_to_hedl(json);
    assert!(result.is_err());
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_document() {
    let doc = Document::new((1, 0));
    let json = hedl_to_json(&doc).unwrap();
    let restored = json_to_hedl(&json).unwrap();

    assert_eq!(restored.version, (1, 0));
    assert!(restored.root.is_empty());
}

#[test]
fn test_empty_string_value() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "empty".to_string(),
        Item::Scalar(Value::String("".to_string())),
    );

    let json = hedl_to_json(&doc).unwrap();
    let restored = json_to_hedl(&json).unwrap();

    assert_eq!(
        restored.root.get("empty").unwrap().as_scalar().unwrap(),
        &Value::String("".to_string())
    );
}

#[test]
fn test_unicode_string() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("Hello 世界 🌍".to_string())),
    );

    let json = hedl_to_json(&doc).unwrap();
    let restored = json_to_hedl(&json).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("Hello 世界 🌍".to_string())
    );
}

#[test]
fn test_large_integer() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("big".to_string(), Item::Scalar(Value::Int(i64::MAX)));

    let json = hedl_to_json(&doc).unwrap();
    let restored = json_to_hedl(&json).unwrap();

    assert_eq!(
        restored.root.get("big").unwrap().as_scalar().unwrap(),
        &Value::Int(i64::MAX)
    );
}

#[test]
fn test_negative_float() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Float(-123.456)));

    let json = hedl_to_json(&doc).unwrap();
    let restored = json_to_hedl(&json).unwrap();

    if let Some(Item::Scalar(Value::Float(f))) = restored.root.get("value") {
        assert!((f + 123.456).abs() < 0.001);
    } else {
        panic!("Expected float");
    }
}

#[test]
fn test_special_json_characters_in_string() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String(
            r#"quote: " backslash: \ newline"#.to_string(),
        )),
    );

    let json = hedl_to_json(&doc).unwrap();
    // Should produce valid JSON
    let _: JsonValue = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_metadata_keys_skipped() {
    // JSON with __metadata__ keys should skip them
    let json = r#"{"__version__": "1.0", "data": 42}"#;
    let doc = json_to_hedl(json).unwrap();

    // __version__ should be skipped
    assert!(!doc.root.contains_key("__version__"));
    assert!(doc.root.contains_key("data"));
}

// =============================================================================
// Config Tests
// =============================================================================

#[test]
fn test_from_json_config_version() {
    let json = r#"{"data": 42}"#;
    let mut config = FromJsonConfig::default();
    config.version = (2, 1);

    let doc = from_json(json, &config).unwrap();
    assert_eq!(doc.version, (2, 1));
}

#[test]
fn test_to_json_value_direct() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("test".to_string(), Item::Scalar(Value::Int(42)));

    let config = ToJsonConfig::default();
    let value = to_json_value(&doc, &config).unwrap();

    assert!(value.is_object());
    assert_eq!(value["test"], 42);
}

#[test]
fn test_from_json_value_direct() {
    let value = json!({"name": "test", "count": 10});
    let config = FromJsonConfig::default();

    let doc = from_json_value(&value, &config).unwrap();

    assert_eq!(
        doc.root.get("name").unwrap().as_scalar().unwrap(),
        &Value::String("test".to_string())
    );
    assert_eq!(
        doc.root.get("count").unwrap().as_scalar().unwrap(),
        &Value::Int(10)
    );
}

// =============================================================================
// Shared Fixture Round-Trip Tests
// =============================================================================

#[test]
fn test_scalars_roundtrip_json() {
    let doc = fixtures::scalars();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify all scalar types are preserved
    assert_eq!(
        restored.root.get("null_val").unwrap().as_scalar().unwrap(),
        &Value::Null
    );
    assert_eq!(
        restored.root.get("bool_true").unwrap().as_scalar().unwrap(),
        &Value::Bool(true)
    );
    assert_eq!(
        restored
            .root
            .get("bool_false")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::Bool(false)
    );
    assert_eq!(
        restored
            .root
            .get("int_positive")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::Int(42)
    );
    assert_eq!(
        restored
            .root
            .get("int_negative")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::Int(-17)
    );
    assert_eq!(
        restored.root.get("int_zero").unwrap().as_scalar().unwrap(),
        &Value::Int(0)
    );

    // Float comparison with tolerance
    if let Some(Item::Scalar(Value::Float(f))) = restored.root.get("float_positive") {
        assert!((f - 3.5).abs() < 0.00001);
    } else {
        panic!("Expected float_positive");
    }

    assert_eq!(
        restored
            .root
            .get("string_simple")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::String("hello world".to_string())
    );
    assert_eq!(
        restored
            .root
            .get("string_empty")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::String(String::new())
    );
}

#[test]
fn test_special_strings_roundtrip_json() {
    let doc = fixtures::special_strings();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify special characters are preserved
    assert_eq!(
        restored
            .root
            .get("with_quotes")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::String("He said \"hello\" and 'goodbye'".to_string())
    );
    assert_eq!(
        restored
            .root
            .get("with_backslash")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::String("path\\to\\file".to_string())
    );
    assert_eq!(
        restored
            .root
            .get("with_newline")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::String("line1\nline2\nline3".to_string())
    );
    assert_eq!(
        restored.root.get("with_tab").unwrap().as_scalar().unwrap(),
        &Value::String("col1\tcol2\tcol3".to_string())
    );
    assert_eq!(
        restored
            .root
            .get("with_unicode")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::String("日本語 中文 한국어 emoji: 🎉".to_string())
    );
}

#[test]
fn test_references_roundtrip_json() {
    let doc = fixtures::references();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify local reference
    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("local_ref") {
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "some_id");
    } else {
        panic!("Expected local_ref");
    }

    // Verify typed reference
    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("typed_ref") {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "alice");
    } else {
        panic!("Expected typed_ref");
    }
}

#[test]
fn test_expressions_roundtrip_json() {
    let doc = fixtures::expressions();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify expressions are preserved
    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("simple_expr") {
        assert_eq!(format!("{}", e), "now()");
    } else {
        panic!("Expected simple_expr");
    }

    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("var_expr") {
        assert_eq!(format!("{}", e), "user.name");
    } else {
        panic!("Expected var_expr");
    }

    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("complex_expr") {
        assert_eq!(format!("{}", e), "concat(\"hello\", \"world\")");
    } else {
        panic!("Expected complex_expr");
    }
}

#[test]
fn test_tensors_roundtrip_json() {
    let doc = fixtures::tensors();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify 1D tensor
    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(items)))) = restored.root.get("tensor_1d")
    {
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected tensor_1d");
    }

    // Verify 2D tensor
    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(rows)))) = restored.root.get("tensor_2d") {
        assert_eq!(rows.len(), 2);
        if let Tensor::Array(cols) = &rows[0] {
            assert_eq!(cols.len(), 2);
        } else {
            panic!("Expected nested array in tensor_2d");
        }
    } else {
        panic!("Expected tensor_2d");
    }

    // Verify 3D tensor
    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(depth)))) = restored.root.get("tensor_3d")
    {
        assert_eq!(depth.len(), 2);
    } else {
        panic!("Expected tensor_3d");
    }

    // Note: Empty tensors [] are not valid HEDL syntax, so empty arrays
    // in JSON are converted to empty matrix lists instead.
    // The original tensor_empty value becomes an empty matrix list.
    if let Some(Item::List(matrix_list)) = restored.root.get("tensor_empty") {
        assert!(matrix_list.rows.is_empty());
    } else {
        panic!("Expected tensor_empty as empty matrix list");
    }
}

#[test]
fn test_named_values_roundtrip_json() {
    let doc = fixtures::named_values();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify named values
    assert_eq!(
        restored.root.get("app_name").unwrap().as_scalar().unwrap(),
        &Value::String("MyApp".to_string())
    );
    assert_eq!(
        restored.root.get("version").unwrap().as_scalar().unwrap(),
        &Value::String("1.0.0".to_string())
    );
    assert_eq!(
        restored
            .root
            .get("debug_mode")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::Bool(true)
    );
    assert_eq!(
        restored
            .root
            .get("max_connections")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::Int(100)
    );
    assert_eq!(
        restored
            .root
            .get("deprecated_feature")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::Null
    );
}

#[test]
fn test_user_list_roundtrip_json() {
    let doc = fixtures::user_list();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify list structure
    // Type name is singularized and capitalized: "users" -> "User"
    // Schema is sorted alphabetically: id, email, name (with id first)
    if let Some(Item::List(list)) = restored.root.get("users") {
        assert_eq!(list.type_name, "User"); // Singularized from "users"
        assert_eq!(list.rows.len(), 3);
        // Schema is sorted alphabetically with id first
        assert_eq!(
            list.schema,
            vec!["id".to_string(), "email".to_string(), "name".to_string()]
        );

        // Verify first user
        // Per SPEC: fields array includes ALL schema columns (including ID)
        // fields[0] = id, fields[1] = email, fields[2] = name
        let alice = &list.rows[0];
        assert_eq!(alice.id, "alice");
        assert_eq!(alice.fields[0], Value::String("alice".to_string())); // id
        assert_eq!(
            alice.fields[1],
            Value::String("alice@example.com".to_string())
        ); // email
        assert_eq!(alice.fields[2], Value::String("Alice Smith".to_string())); // name
    } else {
        panic!("Expected users list");
    }
}

#[test]
fn test_mixed_type_list_roundtrip_json() {
    let doc = fixtures::mixed_type_list();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify mixed types in list
    // Type name is singularized and capitalized: "items" -> "Item"
    // Schema is sorted alphabetically: id, active, count, name, notes, price
    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.type_name, "Item"); // Singularized from "items"
        assert_eq!(list.rows.len(), 2);

        // Expected schema order (alphabetical, with id first):
        // id, active, count, name, notes, price
        // Per SPEC: fields array includes ALL columns (including id)
        let item1 = &list.rows[0];
        assert_eq!(item1.id, "item1");

        // fields[0] = id (string)
        assert_eq!(item1.fields[0], Value::String("item1".to_string()));

        // fields[1] = active (bool)
        assert_eq!(item1.fields[1], Value::Bool(true));

        // fields[2] = count (int)
        assert_eq!(item1.fields[2], Value::Int(100));

        // fields[3] = name (string)
        assert_eq!(item1.fields[3], Value::String("Widget".to_string()));

        // fields[4] = notes (string)
        assert_eq!(item1.fields[4], Value::String("Best seller".to_string()));

        // fields[5] = price (float)
        if let Value::Float(f) = item1.fields[5] {
            assert!((f - 9.99).abs() < 0.01);
        } else {
            panic!("Expected float price");
        }

        // Verify second item has null in notes
        let item2 = &list.rows[1];
        assert_eq!(item2.fields[4], Value::Null); // notes field
    } else {
        panic!("Expected items list");
    }
}

#[test]
fn test_with_references_roundtrip_json() {
    let doc = fixtures::with_references();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify posts with author references
    // Type name is singularized and capitalized: "posts" -> "Post"
    // Schema is sorted alphabetically: id, author, title
    if let Some(Item::List(list)) = restored.root.get("posts") {
        assert_eq!(list.type_name, "Post"); // Singularized from "posts"
        assert_eq!(list.rows.len(), 3);

        // Verify reference in first post
        // Per SPEC: fields includes ALL columns
        // fields[0] = id, fields[1] = author (reference), fields[2] = title
        let post1 = &list.rows[0];
        if let Value::Reference(r) = &post1.fields[1] {
            assert_eq!(r.type_name, Some("User".to_string()));
            assert_eq!(r.id, "alice");
        } else {
            panic!("Expected reference in post author");
        }
    } else {
        panic!("Expected posts list");
    }
}

#[test]
fn test_with_nest_roundtrip_json() {
    let doc = fixtures::with_nest();

    // Convert to JSON without children (children are not supported in from_json currently)
    let config = ToJsonConfig {
        include_metadata: false,
        flatten_lists: true,
        include_children: false, // Don't export children for now
    };
    let json = to_json(&doc, &config).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify basic list structure
    if let Some(Item::List(list)) = restored.root.get("users") {
        assert_eq!(list.rows.len(), 2);
        let alice = list.rows.iter().find(|n| n.id == "alice").unwrap();
        assert_eq!(alice.id, "alice");

        // Children were not exported, so they won't be in the restored document
    } else {
        panic!("Expected users list");
    }
}

#[test]
fn test_deep_nest_roundtrip_json() {
    let doc = fixtures::deep_nest();

    // Convert to JSON without children (children are not supported in from_json currently)
    let config = ToJsonConfig {
        include_metadata: false,
        flatten_lists: true,
        include_children: false, // Don't export children for now
    };
    let json = to_json(&doc, &config).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify basic list structure
    if let Some(Item::List(list)) = restored.root.get("organizations") {
        assert_eq!(list.rows.len(), 1);
        let acme = &list.rows[0];
        assert_eq!(acme.id, "acme");

        // Children were not exported, so they won't be in the restored document
    } else {
        panic!("Expected organizations list");
    }
}

#[test]
fn test_edge_cases_roundtrip_json() {
    let doc = fixtures::edge_cases();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify large integer
    assert_eq!(
        restored.root.get("large_int").unwrap().as_scalar().unwrap(),
        &Value::Int(i64::MAX)
    );
    assert_eq!(
        restored.root.get("small_int").unwrap().as_scalar().unwrap(),
        &Value::Int(i64::MIN)
    );

    // Verify extreme floats
    if let Some(Item::Scalar(Value::Float(f))) = restored.root.get("tiny_float") {
        assert!((f - f64::MIN_POSITIVE).abs() < f64::EPSILON);
    } else {
        panic!("Expected tiny_float");
    }

    // Verify long string (should be preserved)
    if let Some(Item::Scalar(Value::String(s))) = restored.root.get("long_string") {
        assert_eq!(s.len(), 10000);
        assert!(s.chars().all(|c| c == 'x'));
    } else {
        panic!("Expected long_string");
    }

    // Verify special characters
    assert_eq!(
        restored
            .root
            .get("special_only")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::String("\n\t\r\\\"'".to_string())
    );
}

#[test]
fn test_comprehensive_roundtrip_json() {
    let doc = fixtures::comprehensive();

    // Convert to JSON without children to avoid parsing issues
    // (children parsing is not implemented in from_json currently)
    let config = ToJsonConfig {
        include_metadata: false,
        flatten_lists: true,
        include_children: false, // Don't include children to allow clean round-trip
    };
    let json = to_json(&doc, &config).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify scalar values
    assert_eq!(
        restored
            .root
            .get("config_debug")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::Bool(true)
    );
    assert_eq!(
        restored
            .root
            .get("config_version")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::String("1.0.0".to_string())
    );
    assert_eq!(
        restored
            .root
            .get("config_max_items")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::Int(1000)
    );

    // Verify expression
    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("computed") {
        assert_eq!(format!("{}", e), "multiply(config_max_items, 2)");
    } else {
        panic!("Expected computed expression");
    }

    // Verify tensor
    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(items)))) = restored.root.get("weights") {
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected weights tensor");
    }

    // Verify users list (type name singularized: "users" -> "User")
    if let Some(Item::List(list)) = restored.root.get("users") {
        assert_eq!(list.type_name, "User"); // Singularized from key name
        assert_eq!(list.rows.len(), 2);

        let alice = list.rows.iter().find(|n| n.id == "alice").unwrap();
        assert_eq!(alice.id, "alice");
        // Children not included in this test
    } else {
        panic!("Expected users list");
    }

    // Verify comments with references
    // Schema is sorted alphabetically: id, author, post, text
    // Per SPEC: fields includes ALL columns including id
    if let Some(Item::List(list)) = restored.root.get("comments") {
        assert_eq!(list.rows.len(), 1);
        let comment = &list.rows[0];

        // fields[0] = id
        // fields[1] = author (reference)
        // fields[2] = post (reference)
        // fields[3] = text (string)

        // Verify author reference
        if let Value::Reference(r) = &comment.fields[1] {
            assert_eq!(r.type_name, Some("User".to_string()));
            assert_eq!(r.id, "bob");
        } else {
            panic!("Expected author reference");
        }

        // Verify post reference
        if let Value::Reference(r) = &comment.fields[2] {
            assert_eq!(r.type_name, Some("Post".to_string()));
            assert_eq!(r.id, "p1");
        } else {
            panic!("Expected post reference");
        }
    } else {
        panic!("Expected comments list");
    }

    // Verify tags list (type name singularized: "tags" -> "Tag")
    if let Some(Item::List(list)) = restored.root.get("tags") {
        assert_eq!(list.type_name, "Tag"); // Singularized from key name
        assert_eq!(list.rows.len(), 2);
    } else {
        panic!("Expected tags list");
    }
}

#[test]
fn test_empty_roundtrip_json() {
    let doc = fixtures::empty();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse back to HEDL
    let restored = json_to_hedl(&json).unwrap();

    // Verify empty document
    assert_eq!(restored.version, (1, 0));
    assert!(restored.root.is_empty());
    assert!(restored.structs.is_empty());
    assert!(restored.nests.is_empty());
}
