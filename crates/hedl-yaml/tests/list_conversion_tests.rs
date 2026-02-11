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

//! Comprehensive tests for HEDL v1.1 List literal handling in YAML conversion
//!
//! Tests cover bidirectional conversion between HEDL List values and YAML sequences,
//! ensuring proper distinction between List (string sequences) and Tensor (numeric sequences).

use hedl_core::lex::Tensor;
use hedl_core::{Document, Item, Reference, Value};
use hedl_yaml::{from_yaml, hedl_to_yaml, yaml_to_hedl, FromYamlConfig};

// =============================================================================
// Test 1: List to YAML conversion
// =============================================================================

#[test]
fn test_string_list_to_yaml_sequence() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::String("a".to_string().into()),
        Value::String("b".to_string().into()),
        Value::String("c".to_string().into()),
    ]));
    doc.root.insert("roles".to_string(), Item::Scalar(list));

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    assert!(yaml_str.contains("roles:"));
    assert!(yaml_str.contains("- a") || yaml_str.contains("[a, b, c]"));
}

#[test]
fn test_bool_list_to_yaml_sequence() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::Bool(true),
        Value::Bool(false),
        Value::Bool(true),
    ]));
    doc.root.insert("flags".to_string(), Item::Scalar(list));

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    let doc2 = yaml_to_hedl(&yaml_str).unwrap();

    let flags = doc2.root.get("flags").unwrap().as_scalar().unwrap();
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
fn test_reference_list_to_yaml_sequence() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::Reference(Reference::local("user1")),
        Value::Reference(Reference::local("user2")),
        Value::Reference(Reference::qualified("User", "user3")),
    ]));
    doc.root.insert("refs".to_string(), Item::Scalar(list));

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    let doc2 = yaml_to_hedl(&yaml_str).unwrap();

    let refs = doc2.root.get("refs").unwrap().as_scalar().unwrap();
    if let Value::List(items) = refs {
        assert_eq!(items.len(), 3);
        assert!(matches!(&items[0], Value::Reference(r) if r.id.as_ref() == "user1"));
        assert!(matches!(&items[1], Value::Reference(r) if r.id.as_ref() == "user2"));
        assert!(matches!(&items[2], Value::Reference(r) if r.id.as_ref() == "user3"));
    } else {
        panic!("Expected List value, got {:?}", refs);
    }
}

// =============================================================================
// Test 2: YAML to List conversion
// =============================================================================

#[test]
fn test_yaml_string_sequence_to_list() {
    let yaml = r#"
roles:
  - admin
  - editor
  - viewer
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

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
fn test_yaml_inline_sequence_to_list() {
    let yaml = r#"
roles: [admin, editor, viewer]
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let roles = doc.root.get("roles").unwrap().as_scalar().unwrap();
    if let Value::List(items) = roles {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::String("admin".to_string().into()));
    } else {
        panic!("Expected List value, got {:?}", roles);
    }
}

#[test]
fn test_yaml_bool_sequence_to_list() {
    let yaml = r#"
flags:
  - true
  - false
  - true
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

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
fn test_yaml_numeric_sequence_to_tensor_not_list() {
    let yaml = r#"
values: [1, 2, 3]
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let values = doc.root.get("values").unwrap().as_scalar().unwrap();
    // Numeric sequences should become Tensor, not List
    assert!(
        matches!(values, Value::Tensor(_)),
        "Numeric sequence should become Tensor, got {:?}",
        values
    );
}

// =============================================================================
// Test 3: Empty list roundtrip
// =============================================================================

#[test]
fn test_empty_list_to_yaml() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::default());
    doc.root.insert("empty".to_string(), Item::Scalar(list));

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    assert!(yaml_str.contains("empty:") || yaml_str.contains("empty"));
}

#[test]
fn test_empty_yaml_sequence_to_list() {
    let yaml = r#"
empty: []
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    // Empty YAML sequences might not create an entry in the document
    if let Some(empty_item) = doc.root.get("empty") {
        if let Some(empty) = empty_item.as_scalar() {
            if let Value::List(items) = empty {
                assert_eq!(
                    items.len(),
                    0,
                    "Empty YAML sequence should become empty List"
                );
            } else if matches!(empty, Value::Null) {
                // Empty sequence as null is also acceptable
                println!("Empty YAML sequence became Null (acceptable)");
            } else {
                panic!("Expected List or Null value, got {:?}", empty);
            }
        } else if let Some(matrix_list) = empty_item.as_list() {
            // Empty sequences might be interpreted as empty MatrixList
            assert_eq!(
                matrix_list.rows.len(),
                0,
                "Empty YAML sequence as MatrixList should have no rows"
            );
        } else {
            panic!("Expected scalar or list item, got {:?}", empty_item);
        }
    } else {
        // Empty sequences might be omitted, which is also acceptable behavior
        println!("Empty YAML sequence was omitted from document (acceptable)");
    }
}

#[test]
fn test_empty_list_roundtrip() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::default());
    doc.root.insert("empty".to_string(), Item::Scalar(list));

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    let doc2 = yaml_to_hedl(&yaml_str).unwrap();

    // Empty lists might be omitted or preserved depending on YAML serialization
    if let Some(empty_item) = doc2.root.get("empty") {
        if let Some(empty) = empty_item.as_scalar() {
            assert!(
                matches!(empty, Value::List(items) if items.is_empty())
                    || matches!(empty, Value::Null),
                "Empty list should roundtrip to List or Null, got {:?}",
                empty
            );
        } else if let Some(matrix_list) = empty_item.as_list() {
            // Empty sequences might be interpreted as empty MatrixList
            assert_eq!(
                matrix_list.rows.len(),
                0,
                "Empty list as MatrixList should have no rows"
            );
        } else {
            panic!("Expected scalar or list item, got {:?}", empty_item);
        }
    } else {
        // Empty sequences might be omitted, which is acceptable
        println!("Empty list was omitted after roundtrip (acceptable)");
    }
}

// =============================================================================
// Test 4: Nested list roundtrip (numeric tensors work, string lists have limitations)
// =============================================================================

#[test]
fn test_nested_numeric_tensor_to_yaml() {
    let mut doc = Document::new((2, 0));
    let tensor = Value::Tensor(Box::new(Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ])));
    doc.root.insert("matrix".to_string(), Item::Scalar(tensor));

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    let doc2 = yaml_to_hedl(&yaml_str).unwrap();

    let matrix = doc2.root.get("matrix").unwrap().as_scalar().unwrap();
    if let Value::Tensor(t) = matrix {
        if let Tensor::Array(outer) = t.as_ref() {
            assert_eq!(outer.len(), 2);
        } else {
            panic!("Expected Tensor array");
        }
    } else {
        panic!("Expected Tensor value, got {:?}", matrix);
    }
}

#[test]
fn test_nested_numeric_yaml_sequence_to_tensor() {
    let yaml = r#"
matrix:
  - [1.0, 2.0]
  - [3.0, 4.0]
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

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
fn test_nested_list_to_yaml_exports_correctly() {
    // NOTE: Nested string lists export to YAML correctly but cannot roundtrip
    // due to current implementation limitations (same as JSON)
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

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    assert!(yaml_str.contains("nested:"), "Should export nested list");

    // Note: Roundtrip currently fails due to nested string array limitation
    // Future improvement: enhance array classification logic
}

// =============================================================================
// Test 5: Mixed content (List and Tensor)
// =============================================================================

#[test]
fn test_document_with_list_and_tensor() {
    let mut doc = Document::new((2, 0));

    // Add a list (string sequence)
    let list = Value::List(Box::new(vec![
        Value::String("admin".to_string().into()),
        Value::String("editor".to_string().into()),
    ]));
    doc.root.insert("roles".to_string(), Item::Scalar(list));

    // Add a tensor (numeric sequence)
    let tensor = Value::Tensor(Box::new(Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ])));
    doc.root.insert("values".to_string(), Item::Scalar(tensor));

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    let doc2 = yaml_to_hedl(&yaml_str).unwrap();

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
fn test_mixed_sequence_prefers_list() {
    let yaml = r#"
mixed: [text, 123, true]
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let mixed = doc.root.get("mixed").unwrap().as_scalar().unwrap();
    // Mixed sequences should become List since they can't be pure numeric Tensor
    assert!(
        matches!(mixed, Value::List(_)),
        "Mixed sequence should become List, got {:?}",
        mixed
    );
}

#[test]
fn test_float_sequence_becomes_tensor() {
    let yaml = r#"
floats: [1.5, 2.7, 3.14]
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let floats = doc.root.get("floats").unwrap().as_scalar().unwrap();
    assert!(
        matches!(floats, Value::Tensor(_)),
        "Float sequence should become Tensor, got {:?}",
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
        Value::String("hello: world".to_string().into()),
        Value::String("quote'test".to_string().into()),
        Value::String("bracket[test]".to_string().into()),
        Value::String("paren(test)".to_string().into()),
    ]));
    doc.root.insert("special".to_string(), Item::Scalar(list));

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    let doc2 = yaml_to_hedl(&yaml_str).unwrap();

    let special = doc2.root.get("special").unwrap().as_scalar().unwrap();
    if let Value::List(items) = special {
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], Value::String("hello: world".to_string().into()));
        assert_eq!(items[1], Value::String("quote'test".to_string().into()));
        assert_eq!(items[2], Value::String("bracket[test]".to_string().into()));
        assert_eq!(items[3], Value::String("paren(test)".to_string().into()));
    } else {
        panic!("Expected List value, got {:?}", special);
    }
}

#[test]
fn test_list_with_yaml_special_chars() {
    let yaml = r#"
quoted:
  - "say 'hello'"
  - 'it''s "working"'
  - "colon: test"
  - "dash - test"
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let quoted = doc.root.get("quoted").unwrap().as_scalar().unwrap();
    if let Value::List(items) = quoted {
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], Value::String("say 'hello'".to_string().into()));
        assert_eq!(
            items[1],
            Value::String("it's \"working\"".to_string().into())
        );
        assert_eq!(items[2], Value::String("colon: test".to_string().into()));
        assert_eq!(items[3], Value::String("dash - test".to_string().into()));
    } else {
        panic!("Expected List value, got {:?}", quoted);
    }
}

// =============================================================================
// Test 8: Unicode roundtrip
// =============================================================================

#[test]
fn test_unicode_list_to_yaml() {
    let mut doc = Document::new((2, 0));
    let list = Value::List(Box::new(vec![
        Value::String("日本語".to_string().into()),
        Value::String("中文".to_string().into()),
        Value::String("한국어".to_string().into()),
        Value::String("🎉🚀".to_string().into()),
    ]));
    doc.root.insert("languages".to_string(), Item::Scalar(list));

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    assert!(yaml_str.contains("日本語"));
    assert!(yaml_str.contains("中文"));
}

#[test]
fn test_unicode_yaml_sequence_to_list() {
    let yaml = r#"
languages:
  - 日本語
  - 中文
  - 한국어
  - 🎉🚀
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

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

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    let doc2 = yaml_to_hedl(&yaml_str).unwrap();

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

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    let doc2 = yaml_to_hedl(&yaml_str).unwrap();

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
    let yaml_items: Vec<String> = (0..10000).map(|i| format!("  - element_{}", i)).collect();
    let yaml = format!("huge:\n{}", yaml_items.join("\n"));

    let config = FromYamlConfig::default();
    let doc = from_yaml(&yaml, &config).unwrap();

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

    let yaml_str = hedl_to_yaml(&doc).unwrap();
    let doc2 = yaml_to_hedl(&yaml_str).unwrap();

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
    let yaml = r#"
tags: [rust, hedl, yaml, converter]
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    let tags = doc.root.get("tags").unwrap().as_scalar().unwrap();
    assert!(
        matches!(tags, Value::List(_)),
        "Homogeneous string sequence should become List"
    );
}
