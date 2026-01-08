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

//! Comprehensive tests for hedl-yaml conversion
//!
//! Tests bidirectional conversion between HEDL documents and YAML.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_core::lex::Tensor;
use hedl_test::fixtures;
use hedl_yaml::{from_yaml, hedl_to_yaml, to_yaml, yaml_to_hedl, FromYamlConfig, ToYamlConfig};
use std::collections::BTreeMap;

// =============================================================================
// Basic Scalar Conversion Tests
// =============================================================================

#[test]
fn test_null_to_yaml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Null));

    let yaml = hedl_to_yaml(&doc).unwrap();
    assert!(yaml.contains("null") || yaml.contains("~"));
}

#[test]
fn test_bool_true_to_yaml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(true)));

    let yaml = hedl_to_yaml(&doc).unwrap();
    assert!(yaml.contains("true"));
}

#[test]
fn test_bool_false_to_yaml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(false)));

    let yaml = hedl_to_yaml(&doc).unwrap();
    assert!(yaml.contains("false"));
}

#[test]
fn test_int_to_yaml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("count".to_string(), Item::Scalar(Value::Int(42)));

    let yaml = hedl_to_yaml(&doc).unwrap();
    assert!(yaml.contains("42"));
}

#[test]
fn test_negative_int_to_yaml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Int(-100)));

    let yaml = hedl_to_yaml(&doc).unwrap();
    assert!(yaml.contains("-100"));
}

#[test]
fn test_float_to_yaml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("pi".to_string(), Item::Scalar(Value::Float(3.5)));

    let yaml = hedl_to_yaml(&doc).unwrap();
    assert!(yaml.contains("3.5"));
}

#[test]
fn test_string_to_yaml() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "name".to_string(),
        Item::Scalar(Value::String("hello world".to_string())),
    );

    let yaml = hedl_to_yaml(&doc).unwrap();
    assert!(yaml.contains("hello world"));
}

// =============================================================================
// Reference Conversion Tests
// =============================================================================

#[test]
fn test_local_reference_to_yaml() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target_id"))),
    );

    let yaml = hedl_to_yaml(&doc).unwrap();
    assert!(yaml.contains("@target_id"));
}

#[test]
fn test_qualified_reference_to_yaml() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "alice"))),
    );

    let yaml = hedl_to_yaml(&doc).unwrap();
    assert!(yaml.contains("@User:alice"));
}

#[test]
fn test_reference_from_yaml() {
    // References are encoded as mappings with @ref key
    let yaml = "ref:\n  '@ref': '@User:alice'\n";
    let doc = yaml_to_hedl(yaml).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = doc.root.get("ref") {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "alice");
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_local_reference_from_yaml() {
    // References are encoded as mappings with @ref key
    let yaml = "ref:\n  '@ref': '@some_id'\n";
    let doc = yaml_to_hedl(yaml).unwrap();

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
fn test_expression_to_yaml() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "expr".to_string(),
        Item::Scalar(hedl_test::expr_value("add(x, multiply(y, 2))")),
    );

    let yaml = hedl_to_yaml(&doc).unwrap();
    assert!(yaml.contains("$(add(x, multiply(y, 2)))"));
}

#[test]
fn test_expression_from_yaml() {
    let yaml = "expr: '$(add(a, b))'\n";
    let doc = yaml_to_hedl(yaml).unwrap();

    if let Some(Item::Scalar(Value::Expression(e))) = doc.root.get("expr") {
        assert_eq!(e.to_string(), "add(a, b)");
    } else {
        panic!("Expected expression");
    }
}

// =============================================================================
// Tensor Conversion Tests
// =============================================================================

#[test]
fn test_1d_tensor_to_yaml() {
    let mut doc = Document::new((1, 0));
    let tensor = Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ]);
    doc.root
        .insert("data".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let yaml = hedl_to_yaml(&doc).unwrap();
    // Should contain array syntax
    assert!(yaml.contains("1") && yaml.contains("2") && yaml.contains("3"));
}

#[test]
fn test_2d_tensor_to_yaml() {
    let mut doc = Document::new((1, 0));
    let tensor = Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ]);
    doc.root
        .insert("matrix".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(rows)))) = restored.root.get("matrix") {
        assert_eq!(rows.len(), 2);
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_tensor_from_yaml() {
    let yaml = "data:\n  - 1.0\n  - 2.0\n  - 3.0\n";
    let doc = yaml_to_hedl(yaml).unwrap();

    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(items)))) = doc.root.get("data") {
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected tensor");
    }
}

// =============================================================================
// Object Conversion Tests
// =============================================================================

#[test]
fn test_nested_object_to_yaml() {
    let mut doc = Document::new((1, 0));
    let mut inner = BTreeMap::new();
    inner.insert("x".to_string(), Item::Scalar(Value::Int(10)));
    inner.insert("y".to_string(), Item::Scalar(Value::Int(20)));
    doc.root.insert("point".to_string(), Item::Object(inner));

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    let point = restored.root.get("point").unwrap().as_object().unwrap();
    assert_eq!(
        point.get("x").unwrap().as_scalar().unwrap(),
        &Value::Int(10)
    );
    assert_eq!(
        point.get("y").unwrap().as_scalar().unwrap(),
        &Value::Int(20)
    );
}

#[test]
fn test_deeply_nested_object_to_yaml() {
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

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    let l1 = restored.root.get("level1").unwrap().as_object().unwrap();
    let l2 = l1.get("level2").unwrap().as_object().unwrap();
    let l3 = l2.get("level3").unwrap().as_object().unwrap();
    assert_eq!(
        l3.get("value").unwrap().as_scalar().unwrap(),
        &Value::String("deep".to_string())
    );
}

#[test]
fn test_object_from_yaml() {
    let yaml = "config:\n  host: localhost\n  port: 8080\n";
    let doc = yaml_to_hedl(yaml).unwrap();

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
fn test_matrix_list_to_yaml() {
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

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    if let Some(Item::List(list)) = restored.root.get("users") {
        assert_eq!(list.rows.len(), 2);
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_matrix_list_from_yaml() {
    let yaml = r#"
users:
  - id: alice
    name: Alice
    age: 30
  - id: bob
    name: Bob
    age: 25
"#;

    let doc = yaml_to_hedl(yaml).unwrap();

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
fn test_round_trip_all_scalar_types() {
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

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

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
        "local_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("item1"))),
    );
    doc.root.insert(
        "qualified_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "user1"))),
    );

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("local_ref") {
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "item1");
    } else {
        panic!("Expected local reference");
    }

    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("qualified_ref") {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "user1");
    } else {
        panic!("Expected qualified reference");
    }
}

#[test]
fn test_round_trip_expression() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "expr".to_string(),
        Item::Scalar(hedl_test::expr_value("add(x, 1)")),
    );

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("expr") {
        assert_eq!(e.to_string(), "add(x, 1)");
    } else {
        panic!("Expected expression");
    }
}

#[test]
fn test_round_trip_nested_tensor() {
    let mut doc = Document::new((1, 0));
    let tensor = Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ]);
    doc.root
        .insert("matrix".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(rows)))) = restored.root.get("matrix") {
        assert_eq!(rows.len(), 2);
        if let Tensor::Array(cols) = &rows[0] {
            assert_eq!(cols.len(), 2);
        }
    } else {
        panic!("Expected tensor");
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_invalid_yaml_error() {
    let result = yaml_to_hedl("{ invalid: yaml: [");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("YAML parse error"));
}

#[test]
fn test_non_mapping_root_error() {
    let result = yaml_to_hedl("- item1\n- item2\n");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Root must be a YAML mapping"));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_document() {
    let doc = Document::new((1, 0));
    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    assert_eq!(restored.version, (1, 0));
}

#[test]
fn test_empty_string_value() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "empty".to_string(),
        Item::Scalar(Value::String("".to_string())),
    );

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

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

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("Hello 世界 🌍".to_string())
    );
}

#[test]
fn test_multiline_string() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("line1\nline2\nline3".to_string())),
    );

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    if let Some(Item::Scalar(Value::String(s))) = restored.root.get("text") {
        assert!(s.contains("line1"));
        assert!(s.contains("line2"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_yaml_anchors_and_aliases() {
    // Test basic anchor and alias (not merge keys which may not be supported)
    let yaml = r#"
defaults: &defaults
  timeout: 30
  retries: 3

production:
  settings: *defaults
"#;

    let doc = yaml_to_hedl(yaml).unwrap();

    // Verify defaults are parsed
    let defaults = doc.root.get("defaults").unwrap().as_object().unwrap();
    assert_eq!(
        defaults.get("timeout").unwrap().as_scalar().unwrap(),
        &Value::Int(30)
    );
    assert_eq!(
        defaults.get("retries").unwrap().as_scalar().unwrap(),
        &Value::Int(3)
    );

    // Verify alias is resolved - settings should contain the same values as defaults
    let prod = doc.root.get("production").unwrap().as_object().unwrap();
    let settings = prod.get("settings").unwrap().as_object().unwrap();
    assert_eq!(
        settings.get("timeout").unwrap().as_scalar().unwrap(),
        &Value::Int(30)
    );
    assert_eq!(
        settings.get("retries").unwrap().as_scalar().unwrap(),
        &Value::Int(3)
    );
}

#[test]
fn test_special_yaml_characters_in_string() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("colon: value, bracket: [test]".to_string())),
    );

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("colon: value, bracket: [test]".to_string())
    );
}

// =============================================================================
// Config Tests
// =============================================================================

#[test]
fn test_from_yaml_config_version() {
    let yaml = "data: 42\n";
    let config = FromYamlConfig {
        version: (2, 1),
        default_type_name: "Item".to_string(),
        ..Default::default()
    };

    let doc = from_yaml(yaml, &config).unwrap();
    assert_eq!(doc.version, (2, 1));
}

#[test]
fn test_to_yaml_config_pretty() {
    let mut doc = Document::new((1, 0));
    let mut inner = BTreeMap::new();
    inner.insert(
        "key".to_string(),
        Item::Scalar(Value::String("value".to_string())),
    );
    doc.root.insert("object".to_string(), Item::Object(inner));

    let config = ToYamlConfig {
        include_metadata: true,
        ..Default::default()
    };

    let yaml = to_yaml(&doc, &config).unwrap();
    // Should have proper indentation
    assert!(yaml.contains("object:"));
}

// =============================================================================
// Type Inference Tests
// =============================================================================

#[test]
fn test_yaml_type_inference_int() {
    let yaml = "value: 42\n";
    let doc = yaml_to_hedl(yaml).unwrap();
    assert_eq!(
        doc.root.get("value").unwrap().as_scalar().unwrap(),
        &Value::Int(42)
    );
}

#[test]
fn test_yaml_type_inference_float() {
    let yaml = "value: 3.25\n";
    let doc = yaml_to_hedl(yaml).unwrap();

    if let Some(Item::Scalar(Value::Float(f))) = doc.root.get("value") {
        assert!((f - 3.25).abs() < 0.001);
    } else {
        panic!("Expected float");
    }
}

#[test]
fn test_yaml_type_inference_bool() {
    let yaml = "active: true\ninactive: false\n";
    let doc = yaml_to_hedl(yaml).unwrap();
    assert_eq!(
        doc.root.get("active").unwrap().as_scalar().unwrap(),
        &Value::Bool(true)
    );
    assert_eq!(
        doc.root.get("inactive").unwrap().as_scalar().unwrap(),
        &Value::Bool(false)
    );
}

#[test]
fn test_yaml_type_inference_null() {
    let yaml = "value: null\nalso_null: ~\n";
    let doc = yaml_to_hedl(yaml).unwrap();
    assert_eq!(
        doc.root.get("value").unwrap().as_scalar().unwrap(),
        &Value::Null
    );
    assert_eq!(
        doc.root.get("also_null").unwrap().as_scalar().unwrap(),
        &Value::Null
    );
}

// =============================================================================
// Shared Test Fixture Round-Trip Tests
// =============================================================================

#[test]
fn test_scalars_roundtrip_yaml() {
    let doc = fixtures::scalars();

    // Convert to YAML and back
    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

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

    // Float comparisons with tolerance
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
fn test_special_strings_roundtrip_yaml() {
    let doc = fixtures::special_strings();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify special string handling
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
fn test_references_roundtrip_yaml() {
    let doc = fixtures::references();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

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
fn test_expressions_roundtrip_yaml() {
    let doc = fixtures::expressions();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify expressions are preserved
    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("simple_expr") {
        assert_eq!(e.to_string(), "now()");
    } else {
        panic!("Expected simple_expr");
    }

    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("var_expr") {
        assert_eq!(e.to_string(), "user.name");
    } else {
        panic!("Expected var_expr");
    }

    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("complex_expr") {
        assert_eq!(e.to_string(), "concat(\"hello\", \"world\")");
    } else {
        panic!("Expected complex_expr");
    }
}

#[test]
fn test_tensors_roundtrip_yaml() {
    let doc = fixtures::tensors();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify 1D tensor
    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(items)))) = restored.root.get("tensor_1d")
    {
        assert_eq!(items.len(), 3);
        if let Tensor::Scalar(v) = items[0] {
            assert!((v - 1.0).abs() < 0.001);
        }
    } else {
        panic!("Expected tensor_1d");
    }

    // Verify 2D tensor
    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(rows)))) = restored.root.get("tensor_2d") {
        assert_eq!(rows.len(), 2);
        if let Tensor::Array(cols) = &rows[0] {
            assert_eq!(cols.len(), 2);
        } else {
            panic!("Expected array in row");
        }
    } else {
        panic!("Expected tensor_2d");
    }

    // Verify 3D tensor
    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(layers)))) = restored.root.get("tensor_3d")
    {
        assert_eq!(layers.len(), 2);
    } else {
        panic!("Expected tensor_3d");
    }

    // Note: Empty tensors [] are not valid HEDL syntax, so empty sequences
    // in YAML are converted to empty matrix lists instead.
    // The original tensor_empty value becomes an empty matrix list.
    if let Some(Item::List(matrix_list)) = restored.root.get("tensor_empty") {
        assert!(matrix_list.rows.is_empty());
    } else {
        panic!("Expected tensor_empty as empty matrix list");
    }
}

#[test]
fn test_named_values_roundtrip_yaml() {
    let doc = fixtures::named_values();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify named values are preserved
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
fn test_user_list_roundtrip_yaml() {
    let doc = fixtures::user_list();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify basic structure is preserved
    if let Some(Item::List(list)) = restored.root.get("users") {
        assert_eq!(list.type_name, "User"); // Singularized by YAML conversion
        assert_eq!(list.rows.len(), 3);

        // Verify first user
        let alice = &list.rows[0];
        assert_eq!(alice.id, "alice");
        // Per SPEC: fields include ALL schema columns including ID
        assert_eq!(alice.fields.len(), 3);
    } else {
        panic!("Expected users list");
    }
}

#[test]
fn test_mixed_type_list_roundtrip_yaml() {
    let doc = fixtures::mixed_type_list();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify basic structure with mixed types
    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.type_name, "Item"); // Singularized
        assert_eq!(list.rows.len(), 2);

        let item1 = &list.rows[0];
        assert_eq!(item1.id, "item1");
        // Per SPEC: fields include ALL schema columns including ID
        assert_eq!(item1.fields.len(), 6);
    } else {
        panic!("Expected items list");
    }
}

#[test]
fn test_with_references_roundtrip_yaml() {
    let doc = fixtures::with_references();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify users list
    if let Some(Item::List(users)) = restored.root.get("users") {
        assert_eq!(users.type_name, "User"); // Singularized
        assert_eq!(users.rows.len(), 2);
    } else {
        panic!("Expected users list");
    }

    // Verify posts with references
    if let Some(Item::List(posts)) = restored.root.get("posts") {
        assert_eq!(posts.type_name, "Post"); // Singularized
        assert_eq!(posts.rows.len(), 3);
        // Per SPEC: fields include ALL schema columns including ID
        assert_eq!(posts.rows[0].fields.len(), 3);
    } else {
        panic!("Expected posts list");
    }
}

#[test]
fn test_with_nest_roundtrip_yaml() {
    let doc = fixtures::with_nest();

    // Default YAML config has include_children=true, so children are serialized and restored
    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify basic structure
    if let Some(Item::List(users)) = restored.root.get("users") {
        assert_eq!(users.rows.len(), 2);

        // Verify users and their nested posts are preserved
        let alice = users.rows.iter().find(|n| n.id == "alice").unwrap();
        assert!(alice.children.contains_key("posts"));
        let alice_posts = alice.children.get("posts").unwrap();
        assert_eq!(alice_posts.len(), 2);

        let bob = users.rows.iter().find(|n| n.id == "bob").unwrap();
        assert!(bob.children.contains_key("posts"));
        let bob_posts = bob.children.get("posts").unwrap();
        assert_eq!(bob_posts.len(), 1);
    } else {
        panic!("Expected users list");
    }
}

#[test]
fn test_deep_nest_roundtrip_yaml() {
    let doc = fixtures::deep_nest();

    // Default YAML config has include_children=true
    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify basic structure with nested children preserved
    if let Some(Item::List(orgs)) = restored.root.get("organizations") {
        assert_eq!(orgs.rows.len(), 1);

        let acme = &orgs.rows[0];
        assert_eq!(acme.id, "acme");

        // Children are preserved with default config
        assert!(acme.children.contains_key("departments"));
        let departments = acme.children.get("departments").unwrap();
        assert_eq!(departments.len(), 1);
        assert_eq!(departments[0].id, "engineering");

        // Nested employees within department
        assert!(departments[0].children.contains_key("employees"));
        let employees = departments[0].children.get("employees").unwrap();
        assert_eq!(employees.len(), 2);
    } else {
        panic!("Expected organizations list");
    }
}

#[test]
fn test_edge_cases_roundtrip_yaml() {
    let doc = fixtures::edge_cases();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify extreme integers
    assert_eq!(
        restored.root.get("large_int").unwrap().as_scalar().unwrap(),
        &Value::Int(i64::MAX)
    );
    assert_eq!(
        restored.root.get("small_int").unwrap().as_scalar().unwrap(),
        &Value::Int(i64::MIN)
    );

    // Verify extreme floats (may have precision issues, so just check they exist)
    assert!(matches!(
        restored
            .root
            .get("tiny_float")
            .unwrap()
            .as_scalar()
            .unwrap(),
        Value::Float(_)
    ));
    assert!(matches!(
        restored
            .root
            .get("large_float")
            .unwrap()
            .as_scalar()
            .unwrap(),
        Value::Float(_)
    ));

    // Verify long string
    if let Some(Item::Scalar(Value::String(s))) = restored.root.get("long_string") {
        assert_eq!(s.len(), 10000);
        assert!(s.chars().all(|c| c == 'x'));
    } else {
        panic!("Expected long_string");
    }

    // Verify special characters only
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
fn test_comprehensive_roundtrip_yaml() {
    let doc = fixtures::comprehensive();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

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
        assert_eq!(e.to_string(), "multiply(config_max_items, 2)");
    } else {
        panic!("Expected computed expression");
    }

    // Verify tensor
    if let Some(Item::Scalar(Value::Tensor(Tensor::Array(items)))) = restored.root.get("weights") {
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected weights tensor");
    }

    // Verify users (nested posts ARE preserved with default config)
    if let Some(Item::List(users)) = restored.root.get("users") {
        assert_eq!(users.rows.len(), 2);

        let alice = users.rows.iter().find(|n| n.id == "alice").unwrap();
        assert_eq!(alice.id, "alice");
        // Children ARE preserved with default YAML config (include_children=true)
        assert!(alice.children.contains_key("posts"));
        let alice_posts = alice.children.get("posts").unwrap();
        assert_eq!(alice_posts.len(), 1);

        // bob has no children in fixture
        let bob = users.rows.iter().find(|n| n.id == "bob").unwrap();
        assert!(bob.children.is_empty());
    } else {
        panic!("Expected users list");
    }

    // Verify comments with references
    if let Some(Item::List(comments)) = restored.root.get("comments") {
        assert_eq!(comments.type_name, "Comment"); // Singularized
        assert_eq!(comments.rows.len(), 1);

        let comment = &comments.rows[0];
        assert_eq!(comment.id, "c1");
        // Per SPEC: fields include ALL schema columns including ID
        assert_eq!(comment.fields.len(), 4);
    } else {
        panic!("Expected comments list");
    }

    // Verify tags
    if let Some(Item::List(tags)) = restored.root.get("tags") {
        assert_eq!(tags.type_name, "Tag"); // Singularized
        assert_eq!(tags.rows.len(), 2);
    } else {
        panic!("Expected tags list");
    }

    // Note: YAML does not preserve struct and NEST metadata
    // The structure is preserved through the data itself
}

#[test]
fn test_empty_roundtrip_yaml() {
    let doc = fixtures::empty();

    let yaml = hedl_to_yaml(&doc).unwrap();
    let restored = yaml_to_hedl(&yaml).unwrap();

    // Verify empty document
    assert_eq!(restored.version, (1, 0));
    assert!(restored.root.is_empty());
    assert!(restored.structs.is_empty());
    assert!(restored.nests.is_empty());
}
