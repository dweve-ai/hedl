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

//! Comprehensive tests for HEDL v1.1 List literal handling in JSON conversion
//!
//! Tests cover bidirectional conversion between HEDL List values and JSON arrays,
//! ensuring proper distinction between List (string arrays) and Tensor (numeric arrays).

use hedl_core::lex::Tensor;
use hedl_core::{Document, Item, Reference, Value};
use hedl_json::{from_json, hedl_to_json, json_to_hedl, FromJsonConfig};
use serde_json::{json, Value as JsonValue};

// =============================================================================
// Test 1: List to JSON conversion
// =============================================================================

#[test]
fn test_string_list_to_json_array() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::String("a".to_string().into()),
        Value::String("b".to_string().into()),
        Value::String("c".to_string().into()),
    ]));
    doc.root.insert("roles".to_string(), Item::Scalar(list));

    let json_str = hedl_to_json(&doc).unwrap();
    let json_val: JsonValue = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        json_val["roles"],
        json!(["a", "b", "c"]),
        "List should convert to JSON array"
    );
}

#[test]
fn test_bool_list_to_json_array() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::Bool(true),
        Value::Bool(false),
        Value::Bool(true),
    ]));
    doc.root.insert("flags".to_string(), Item::Scalar(list));

    let json_str = hedl_to_json(&doc).unwrap();
    let json_val: JsonValue = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        json_val["flags"],
        json!([true, false, true]),
        "Bool list should convert to JSON array"
    );
}

#[test]
fn test_reference_list_to_json_array() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::Reference(Reference::local("user1")),
        Value::Reference(Reference::local("user2")),
        Value::Reference(Reference::qualified("User", "user3")),
    ]));
    doc.root.insert("refs".to_string(), Item::Scalar(list));

    let json_str = hedl_to_json(&doc).unwrap();
    let json_val: JsonValue = serde_json::from_str(&json_str).unwrap();

    assert!(
        json_val["refs"].is_array(),
        "Reference list should convert to JSON array"
    );
    let refs = json_val["refs"].as_array().unwrap();
    assert_eq!(refs.len(), 3);
    assert_eq!(refs[0]["@ref"].as_str().unwrap(), "@user1");
    assert_eq!(refs[1]["@ref"].as_str().unwrap(), "@user2");
    assert_eq!(refs[2]["@ref"].as_str().unwrap(), "@User:user3");
}

// =============================================================================
// Test 2: JSON to List conversion
// =============================================================================

#[test]
fn test_json_string_array_to_list() {
    let json = json!({
        "roles": ["admin", "editor", "viewer"]
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let roles = doc.root.get("roles").unwrap().as_scalar().unwrap();
    if let Value::List(items) = roles {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::String("admin".to_string().into()));
        assert_eq!(items[1], Value::String("editor".to_string().into()));
        assert_eq!(items[2], Value::String("viewer".to_string().into()));
    } else {
        panic!("Expected List value, got {:?}", roles);
    }
}

#[test]
fn test_json_bool_array_to_list() {
    let json = json!({
        "flags": [true, false, true]
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let flags = doc.root.get("flags").unwrap().as_scalar().unwrap();
    if let Value::List(items) = flags {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Bool(true));
        assert_eq!(items[1], Value::Bool(false));
        assert_eq!(items[2], Value::Bool(true));
    } else {
        panic!("Expected List value, got {:?}", flags);
    }
}

#[test]
fn test_json_numeric_array_to_tensor_not_list() {
    let json = json!({
        "values": [1, 2, 3]
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let values = doc.root.get("values").unwrap().as_scalar().unwrap();
    // Numeric arrays should become Tensor, not List
    assert!(
        matches!(values, Value::Tensor(_)),
        "Numeric array should become Tensor, got {:?}",
        values
    );
}

// =============================================================================
// Test 3: Empty list roundtrip
// =============================================================================

#[test]
fn test_empty_list_to_json() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::default());
    doc.root.insert("empty".to_string(), Item::Scalar(list));

    let json_str = hedl_to_json(&doc).unwrap();
    let json_val: JsonValue = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        json_val["empty"],
        json!([]),
        "Empty list should convert to empty JSON array"
    );
}

#[test]
fn test_empty_json_array_to_list() {
    let json = json!({
        "empty": []
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let empty = doc.root.get("empty").unwrap().as_scalar().unwrap();
    if let Value::List(items) = empty {
        assert_eq!(items.len(), 0, "Empty JSON array should become empty List");
    } else {
        panic!("Expected List value, got {:?}", empty);
    }
}

#[test]
fn test_empty_list_roundtrip() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::default());
    doc.root.insert("empty".to_string(), Item::Scalar(list));

    let json_str = hedl_to_json(&doc).unwrap();
    let doc2 = json_to_hedl(&json_str).unwrap();

    let empty = doc2.root.get("empty").unwrap().as_scalar().unwrap();
    assert!(
        matches!(empty, Value::List(items) if items.is_empty()),
        "Empty list should roundtrip, got {:?}",
        empty
    );
}

// =============================================================================
// Test 4: Nested list roundtrip (numeric tensors work, string lists have limitations)
// =============================================================================

#[test]
fn test_nested_numeric_tensor_to_json() {
    let mut doc = Document::new((2, 0));
    let tensor = Value::Tensor(Box::new(Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ])));
    doc.root.insert("matrix".to_string(), Item::Scalar(tensor));

    let json_str = hedl_to_json(&doc).unwrap();
    let json_val: JsonValue = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        json_val["matrix"],
        json!([[1.0, 2.0], [3.0, 4.0]]),
        "Nested numeric tensor should convert to nested JSON array"
    );
}

#[test]
fn test_nested_numeric_json_array_to_tensor() {
    let json = json!({
        "matrix": [[1.0, 2.0], [3.0, 4.0]]
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let matrix = doc.root.get("matrix").unwrap().as_scalar().unwrap();
    if let Value::Tensor(t) = matrix {
        if let Tensor::Array(outer) = t.as_ref() {
            assert_eq!(outer.len(), 2);
            if let Tensor::Array(inner) = &outer[0] {
                assert_eq!(inner.len(), 2);
            } else {
                panic!("Expected nested Tensor array");
            }
        } else {
            panic!("Expected Tensor array");
        }
    } else {
        panic!("Expected Tensor value, got {:?}", matrix);
    }
}

#[test]
fn test_nested_list_to_json_exports_correctly() {
    // NOTE: Nested string lists export to JSON correctly but cannot roundtrip
    // due to current implementation limitations in classify_array()
    let mut doc = Document::new((2, 0));
    let inner1 = Value::List(Box::new(vec![
        Value::String("a".to_string().into()),
        Value::String("b".to_string().into()),
    ]));
    let inner2 = Value::List(Box::new(vec![
        Value::String("c".to_string().into()),
        Value::String("d".to_string().into()),
    ]));
    let outer = Value::List(Box::new(vec![inner1, inner2]));
    doc.root.insert("nested".to_string(), Item::Scalar(outer));

    let json_str = hedl_to_json(&doc).unwrap();
    let json_val: JsonValue = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        json_val["nested"],
        json!([["a", "b"], ["c", "d"]]),
        "Nested list should export to nested JSON array"
    );

    // Note: Roundtrip currently fails due to nested string array limitation
    // Future improvement: enhance classify_array() to detect string arrays
}

// =============================================================================
// Test 5: Mixed content (List and Tensor)
// =============================================================================

#[test]
fn test_document_with_list_and_tensor() {
    let mut doc = Document::new((2, 0));

    // Add a list (string array)
    let list = Value::List(Box::new(vec![
        Value::String("admin".to_string().into()),
        Value::String("editor".to_string().into()),
    ]));
    doc.root.insert("roles".to_string(), Item::Scalar(list));

    // Add a tensor (numeric array)
    let tensor = Value::Tensor(Box::new(Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ])));
    doc.root.insert("values".to_string(), Item::Scalar(tensor));

    let json_str = hedl_to_json(&doc).unwrap();
    let json_val: JsonValue = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json_val["roles"], json!(["admin", "editor"]));
    assert_eq!(json_val["values"], json!([1.0, 2.0, 3.0]));

    // Roundtrip
    let doc2 = json_to_hedl(&json_str).unwrap();
    let roles = doc2.root.get("roles").unwrap().as_scalar().unwrap();
    let values = doc2.root.get("values").unwrap().as_scalar().unwrap();

    assert!(
        matches!(roles, Value::List(_)),
        "Roles should remain a List"
    );
    assert!(
        matches!(values, Value::Tensor(_)),
        "Values should remain a Tensor"
    );
}

// =============================================================================
// Test 6: List vs Tensor distinction
// =============================================================================

#[test]
fn test_mixed_array_prefers_list() {
    let json = json!({
        "mixed": ["text", 123, true]
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let mixed = doc.root.get("mixed").unwrap().as_scalar().unwrap();
    // Mixed arrays should become List since they can't be pure numeric Tensor
    assert!(
        matches!(mixed, Value::List(_)),
        "Mixed array should become List, got {:?}",
        mixed
    );
}

#[test]
fn test_float_array_becomes_tensor() {
    let json = json!({
        "floats": [1.5, 2.7, 4.56]
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let floats = doc.root.get("floats").unwrap().as_scalar().unwrap();
    assert!(
        matches!(floats, Value::Tensor(_)),
        "Float array should become Tensor, got {:?}",
        floats
    );
}

// =============================================================================
// Test 7: Special characters in list elements
// =============================================================================

#[test]
fn test_list_with_special_chars() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::String("hello, world".to_string().into()),
        Value::String("quote\"test".to_string().into()),
        Value::String("bracket[test]".to_string().into()),
        Value::String("paren(test)".to_string().into()),
    ]));
    doc.root.insert("special".to_string(), Item::Scalar(list));

    let json_str = hedl_to_json(&doc).unwrap();
    let doc2 = json_to_hedl(&json_str).unwrap();

    let special = doc2.root.get("special").unwrap().as_scalar().unwrap();
    if let Value::List(items) = special {
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], Value::String("hello, world".to_string().into()));
        assert_eq!(items[1], Value::String("quote\"test".to_string().into()));
        assert_eq!(items[2], Value::String("bracket[test]".to_string().into()));
        assert_eq!(items[3], Value::String("paren(test)".to_string().into()));
    } else {
        panic!("Expected List value, got {:?}", special);
    }
}

#[test]
fn test_list_with_escaped_quotes() {
    let json = json!({
        "quoted": ["say \"hello\"", "it's \"working\""]
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let quoted = doc.root.get("quoted").unwrap().as_scalar().unwrap();
    if let Value::List(items) = quoted {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Value::String("say \"hello\"".to_string().into()));
        assert_eq!(
            items[1],
            Value::String("it's \"working\"".to_string().into())
        );
    } else {
        panic!("Expected List value, got {:?}", quoted);
    }
}

// =============================================================================
// Test 8: Unicode roundtrip
// =============================================================================

#[test]
fn test_unicode_list_to_json() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::String("日本語".to_string().into()),
        Value::String("中文".to_string().into()),
        Value::String("한국어".to_string().into()),
        Value::String("🎉🚀".to_string().into()),
    ]));
    doc.root.insert("languages".to_string(), Item::Scalar(list));

    let json_str = hedl_to_json(&doc).unwrap();
    let json_val: JsonValue = serde_json::from_str(&json_str).unwrap();

    assert_eq!(
        json_val["languages"],
        json!(["日本語", "中文", "한국어", "🎉🚀"])
    );
}

#[test]
fn test_unicode_json_array_to_list() {
    let json = json!({
        "languages": ["日本語", "中文", "한국어", "🎉🚀"]
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let languages = doc.root.get("languages").unwrap().as_scalar().unwrap();
    if let Value::List(items) = languages {
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], Value::String("日本語".to_string().into()));
        assert_eq!(items[1], Value::String("中文".to_string().into()));
        assert_eq!(items[2], Value::String("한국어".to_string().into()));
        assert_eq!(items[3], Value::String("🎉🚀".to_string().into()));
    } else {
        panic!("Expected List value, got {:?}", languages);
    }
}

#[test]
fn test_unicode_list_roundtrip() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::String("Ñoño".to_string().into()),
        Value::String("Москва".to_string().into()),
        Value::String("Αθήνα".to_string().into()),
    ]));
    doc.root.insert("cities".to_string(), Item::Scalar(list));

    let json_str = hedl_to_json(&doc).unwrap();
    let doc2 = json_to_hedl(&json_str).unwrap();

    let cities = doc2.root.get("cities").unwrap().as_scalar().unwrap();
    if let Value::List(items) = cities {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::String("Ñoño".to_string().into()));
        assert_eq!(items[1], Value::String("Москва".to_string().into()));
        assert_eq!(items[2], Value::String("Αθήνα".to_string().into()));
    } else {
        panic!("Expected List value after roundtrip, got {:?}", cities);
    }
}

// =============================================================================
// Test 9: Large lists (performance)
// =============================================================================

#[test]
fn test_large_list_conversion() {
    let mut doc = Document::new((2, 0));
    let items: Vec<Value> = (0..1000)
        .map(|i| Value::String(format!("item_{}", i).into()))
        .collect();
    let list = Value::List(Box::new(items));
    doc.root.insert("large".to_string(), Item::Scalar(list));

    let json_str = hedl_to_json(&doc).unwrap();
    let doc2 = json_to_hedl(&json_str).unwrap();

    let large = doc2.root.get("large").unwrap().as_scalar().unwrap();
    if let Value::List(items) = large {
        assert_eq!(items.len(), 1000);
        assert_eq!(items[0], Value::String("item_0".to_string().into()));
        assert_eq!(items[999], Value::String("item_999".to_string().into()));
    } else {
        panic!("Expected List value after roundtrip, got {:?}", large);
    }
}

#[test]
fn test_very_large_list_performance() {
    let json_array: Vec<String> = (0..10000).map(|i| format!("element_{}", i)).collect();
    let json = json!({
        "huge": json_array
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let huge = doc.root.get("huge").unwrap().as_scalar().unwrap();
    if let Value::List(items) = huge {
        assert_eq!(items.len(), 10000);
    } else {
        panic!("Expected List value, got {:?}", huge);
    }
}

// =============================================================================
// Test 10: Mixed value types in lists
// =============================================================================

#[test]
fn test_list_with_null_values() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::String("value1".to_string().into()),
        Value::Null,
        Value::String("value3".to_string().into()),
    ]));
    doc.root.insert("nullable".to_string(), Item::Scalar(list));

    let json_str = hedl_to_json(&doc).unwrap();
    let json_val: JsonValue = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json_val["nullable"], json!(["value1", null, "value3"]));

    let doc2 = json_to_hedl(&json_str).unwrap();
    let nullable = doc2.root.get("nullable").unwrap().as_scalar().unwrap();
    if let Value::List(items) = nullable {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::String("value1".to_string().into()));
        assert_eq!(items[1], Value::Null);
        assert_eq!(items[2], Value::String("value3".to_string().into()));
    } else {
        panic!("Expected List value, got {:?}", nullable);
    }
}

#[test]
fn test_homogeneous_string_list() {
    let json = json!({
        "tags": ["rust", "hedl", "json", "converter"]
    });

    let config = FromJsonConfig::default();
    let doc = from_json(&json.to_string(), &config).unwrap();

    let tags = doc.root.get("tags").unwrap().as_scalar().unwrap();
    assert!(
        matches!(tags, Value::List(_)),
        "Homogeneous string array should become List"
    );
}
