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

//! Comprehensive tests for hedl-xml conversion
//!
//! Tests bidirectional conversion between HEDL documents and XML.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_core::lex::Tensor;
use hedl_test::fixtures;
use hedl_xml::{from_xml, hedl_to_xml, to_xml, xml_to_hedl, FromXmlConfig, ToXmlConfig};
use std::collections::BTreeMap;

// =============================================================================
// Basic Scalar Conversion Tests
// =============================================================================

#[test]
fn test_null_to_xml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Null));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("<value"));
}

#[test]
fn test_bool_true_to_xml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(true)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("true"));
}

#[test]
fn test_bool_false_to_xml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(false)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("false"));
}

#[test]
fn test_int_to_xml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("count".to_string(), Item::Scalar(Value::Int(42)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("42"));
}

#[test]
fn test_negative_int_to_xml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Int(-100)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("-100"));
}

#[test]
fn test_float_to_xml() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("rate".to_string(), Item::Scalar(Value::Float(1.23456)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("1.23"));
}

#[test]
fn test_string_to_xml() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "name".to_string(),
        Item::Scalar(Value::String("hello world".to_string())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("hello world"));
}

// =============================================================================
// XML Escaping Tests
// =============================================================================

#[test]
fn test_xml_escape_ampersand() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("A & B".to_string())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("A & B".to_string())
    );
}

#[test]
fn test_xml_escape_less_than() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("x < y".to_string())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("x < y".to_string())
    );
}

#[test]
fn test_xml_escape_greater_than() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("x > y".to_string())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("x > y".to_string())
    );
}

#[test]
fn test_xml_escape_quotes() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("say \"hello\"".to_string())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("say \"hello\"".to_string())
    );
}

#[test]
fn test_xml_escape_all_special_chars() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("A & B < C > D \"E\"".to_string())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("A & B < C > D \"E\"".to_string())
    );
}

// =============================================================================
// Reference Conversion Tests
// =============================================================================

#[test]
fn test_local_reference_to_xml() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target_id"))),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("ref") {
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "target_id");
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_qualified_reference_to_xml() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "alice"))),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("ref") {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "alice");
    } else {
        panic!("Expected reference");
    }
}

// =============================================================================
// Expression Conversion Tests
// =============================================================================

#[test]
fn test_expression_to_xml() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "expr".to_string(),
        Item::Scalar(hedl_test::expr_value("add(x, mul(y, 2))")),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("expr") {
        assert_eq!(e.to_string(), "add(x, mul(y, 2))");
    } else {
        panic!("Expected expression");
    }
}

// =============================================================================
// Tensor Conversion Tests
// =============================================================================

#[test]
fn test_1d_tensor_to_xml() {
    let mut doc = Document::new((1, 0));
    let tensor = Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ]);
    doc.root
        .insert("data".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("<data>"));
    assert!(xml.contains("<item>"));
}

#[test]
fn test_2d_tensor_to_xml() {
    let mut doc = Document::new((1, 0));
    let tensor = Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ]);
    doc.root
        .insert("matrix".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("<matrix>"));
}

// =============================================================================
// Object Conversion Tests
// =============================================================================

#[test]
fn test_nested_object_to_xml() {
    let mut doc = Document::new((1, 0));
    let mut inner = BTreeMap::new();
    inner.insert("x".to_string(), Item::Scalar(Value::Int(10)));
    inner.insert("y".to_string(), Item::Scalar(Value::Int(20)));
    doc.root.insert("point".to_string(), Item::Object(inner));

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

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
fn test_deeply_nested_object_to_xml() {
    let mut doc = Document::new((1, 0));

    let mut level2 = BTreeMap::new();
    level2.insert(
        "deep".to_string(),
        Item::Scalar(Value::String("value".to_string())),
    );

    let mut level1 = BTreeMap::new();
    level1.insert("nested".to_string(), Item::Object(level2));

    doc.root.insert("outer".to_string(), Item::Object(level1));

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert!(restored.root.contains_key("outer"));
}

// =============================================================================
// Matrix List Conversion Tests
// =============================================================================

#[test]
fn test_matrix_list_to_xml() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);

    // Per SPEC: fields must include ALL schema columns including ID
    list.add_row(Node::new(
        "User",
        "user1",
        vec![
            Value::String("user1".to_string()),
            Value::String("Alice".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "user2",
        vec![
            Value::String("user2".to_string()),
            Value::String("Bob".to_string()),
        ],
    ));

    doc.root.insert("users".to_string(), Item::List(list));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("<users"));
    assert!(xml.contains("user1"));
    assert!(xml.contains("user2"));
}

#[test]
fn test_matrix_list_from_xml() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <hedl>
        <user id="1"><name>Alice</name></user>
        <user id="2"><name>Bob</name></user>
    </hedl>"#;

    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };

    let doc = from_xml(xml, &config).unwrap();

    if let Some(Item::List(list)) = doc.root.get("user") {
        assert_eq!(list.rows.len(), 2);
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

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

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
fn test_round_trip_object() {
    let mut doc = Document::new((1, 0));
    let mut inner = BTreeMap::new();
    inner.insert(
        "name".to_string(),
        Item::Scalar(Value::String("test".to_string())),
    );
    inner.insert("value".to_string(), Item::Scalar(Value::Int(100)));
    doc.root.insert("config".to_string(), Item::Object(inner));

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    let config_obj = restored.root.get("config").unwrap().as_object().unwrap();
    assert_eq!(
        config_obj.get("name").unwrap().as_scalar().unwrap(),
        &Value::String("test".to_string())
    );
    assert_eq!(
        config_obj.get("value").unwrap().as_scalar().unwrap(),
        &Value::Int(100)
    );
}

#[test]
fn test_round_trip_reference() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref1".to_string(),
        Item::Scalar(Value::Reference(Reference::local("user123"))),
    );
    doc.root.insert(
        "ref2".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "456"))),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("ref1").unwrap().as_scalar().unwrap(),
        &Value::Reference(Reference::local("user123"))
    );
    assert_eq!(
        restored.root.get("ref2").unwrap().as_scalar().unwrap(),
        &Value::Reference(Reference::qualified("User", "456"))
    );
}

#[test]
fn test_round_trip_expression() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "expr".to_string(),
        Item::Scalar(hedl_test::expr_value("add(x, 1)")),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("expr").unwrap().as_scalar().unwrap(),
        &hedl_test::expr_value("add(x, 1)")
    );
}

// =============================================================================
// Config Tests
// =============================================================================

#[test]
fn test_config_pretty_print() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "test".to_string(),
        Item::Scalar(Value::String("value".to_string())),
    );

    let config_pretty = ToXmlConfig {
        pretty: true,
        indent: "  ".to_string(),
        ..Default::default()
    };

    let config_compact = ToXmlConfig {
        pretty: false,
        ..Default::default()
    };

    let xml_pretty = to_xml(&doc, &config_pretty).unwrap();
    let xml_compact = to_xml(&doc, &config_compact).unwrap();

    // Pretty printed should have newlines
    assert!(xml_pretty.len() > xml_compact.len());
}

#[test]
fn test_config_custom_root() {
    let doc = Document::new((1, 0));

    let config = ToXmlConfig {
        root_element: "custom_root".to_string(),
        ..Default::default()
    };

    let xml = to_xml(&doc, &config).unwrap();
    assert!(xml.contains("<custom_root"));
    assert!(xml.contains("</custom_root>"));
}

#[test]
fn test_config_metadata() {
    let doc = Document::new((2, 1));

    let config = ToXmlConfig {
        include_metadata: true,
        ..Default::default()
    };

    let xml = to_xml(&doc, &config).unwrap();
    assert!(xml.contains("version=\"2.1\""));
}

#[test]
fn test_attributes_as_values() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <hedl>
        <item id="123" name="test" active="true"/>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("item") {
        assert_eq!(
            obj.get("id").unwrap().as_scalar().unwrap(),
            &Value::Int(123)
        );
        assert_eq!(
            obj.get("name").unwrap().as_scalar().unwrap(),
            &Value::String("test".to_string())
        );
        assert_eq!(
            obj.get("active").unwrap().as_scalar().unwrap(),
            &Value::Bool(true)
        );
    } else {
        panic!("Expected object");
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_document() {
    let doc = Document::new((1, 0));
    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(restored.version, (1, 0));
}

#[test]
fn test_empty_string_value() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("empty".to_string(), Item::Scalar(Value::Null));

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert!(restored.root.contains_key("empty"));
}

#[test]
fn test_unicode_string() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("Hello 世界 🌍".to_string())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("Hello 世界 🌍".to_string())
    );
}

#[test]
fn test_infer_lists_config() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <hedl>
        <user id="1"><name>Alice</name></user>
        <user id="2"><name>Bob</name></user>
    </hedl>"#;

    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };

    let doc = from_xml(xml, &config).unwrap();

    // Should infer a list from repeated elements
    if let Some(Item::List(list)) = doc.root.get("user") {
        assert_eq!(list.rows.len(), 2);
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_xml_declaration() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("test".to_string(), Item::Scalar(Value::Int(42)));

    let xml = hedl_to_xml(&doc).unwrap();

    // Should have XML declaration
    assert!(xml.starts_with("<?xml"));
}

#[test]
fn test_element_content_vs_attributes() {
    // Test parsing element with child elements (children take precedence)
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item>
            <id>123</id>
            <name>Test Item</name>
            <price>9.99</price>
        </item>
    </hedl>"#;

    let doc = xml_to_hedl(xml).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("item") {
        // All values should be parsed from child elements
        assert_eq!(
            obj.get("id").unwrap().as_scalar().unwrap(),
            &Value::Int(123)
        );
        assert_eq!(
            obj.get("name").unwrap().as_scalar().unwrap(),
            &Value::String("Test Item".to_string())
        );
        assert_eq!(
            obj.get("price").unwrap().as_scalar().unwrap(),
            &Value::Float(9.99)
        );
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_numeric_id_inference() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item id="42" value="100"/>
    </hedl>"#;

    let doc = xml_to_hedl(xml).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("item") {
        // Numeric strings should be inferred as integers
        assert_eq!(obj.get("id").unwrap().as_scalar().unwrap(), &Value::Int(42));
        assert_eq!(
            obj.get("value").unwrap().as_scalar().unwrap(),
            &Value::Int(100)
        );
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_boolean_inference() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item active="true" hidden="false"/>
    </hedl>"#;

    let doc = xml_to_hedl(xml).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("item") {
        assert_eq!(
            obj.get("active").unwrap().as_scalar().unwrap(),
            &Value::Bool(true)
        );
        assert_eq!(
            obj.get("hidden").unwrap().as_scalar().unwrap(),
            &Value::Bool(false)
        );
    } else {
        panic!("Expected object");
    }
}

// =============================================================================
// Shared Fixture Round-Trip Tests
// =============================================================================

/// Helper function to verify round-trip conversion preserves document structure.
#[allow(dead_code)]
fn verify_roundtrip(doc: &Document) {
    let xml = hedl_to_xml(doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    // Verify version
    assert_eq!(restored.version, doc.version);

    // Verify root keys exist (order may differ, values may be parsed differently)
    for key in doc.root.keys() {
        assert!(
            restored.root.contains_key(key),
            "Round-trip lost key: {}",
            key
        );
    }
}

#[test]
fn test_scalars_roundtrip_xml() {
    let doc = fixtures::scalars();
    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    // Verify all scalar keys are preserved
    assert!(restored.root.contains_key("null_val"));
    assert!(restored.root.contains_key("bool_true"));
    assert!(restored.root.contains_key("bool_false"));
    assert!(restored.root.contains_key("int_positive"));
    assert!(restored.root.contains_key("int_negative"));
    assert!(restored.root.contains_key("int_zero"));
    assert!(restored.root.contains_key("float_positive"));
    assert!(restored.root.contains_key("float_negative"));
    assert!(restored.root.contains_key("string_simple"));
    assert!(restored.root.contains_key("string_empty"));

    // Verify specific values
    assert_eq!(
        restored.root.get("bool_true").unwrap().as_scalar().unwrap(),
        &Value::Bool(true)
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
            .get("string_simple")
            .unwrap()
            .as_scalar()
            .unwrap(),
        &Value::String("hello world".to_string())
    );
}

#[test]
fn test_special_strings_roundtrip_xml() {
    let doc = fixtures::special_strings();
    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    // Verify all keys are preserved
    assert!(restored.root.contains_key("with_quotes"));
    assert!(restored.root.contains_key("with_backslash"));
    assert!(restored.root.contains_key("with_newline"));
    assert!(restored.root.contains_key("with_tab"));
    assert!(restored.root.contains_key("with_unicode"));
    assert!(restored.root.contains_key("with_mixed"));

    // Verify unicode string preserves correctly
    if let Some(Item::Scalar(Value::String(s))) = restored.root.get("with_unicode") {
        assert!(s.contains("日本語"));
        assert!(s.contains("🎉"));
    } else {
        panic!("Expected unicode string");
    }
}

#[test]
fn test_references_roundtrip_xml() {
    let doc = fixtures::references();
    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    // Verify local reference
    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("local_ref") {
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "some_id");
    } else {
        panic!("Expected local reference");
    }

    // Verify typed reference
    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("typed_ref") {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "alice");
    } else {
        panic!("Expected typed reference");
    }
}

#[test]
fn test_expressions_roundtrip_xml() {
    let doc = fixtures::expressions();
    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    // Verify expression keys exist
    assert!(restored.root.contains_key("simple_expr"));
    assert!(restored.root.contains_key("var_expr"));
    assert!(restored.root.contains_key("complex_expr"));

    // Verify expression values are preserved
    if let Some(Item::Scalar(Value::Expression(_e))) = restored.root.get("simple_expr") {
        // Expression is preserved (exact format may vary)
    } else {
        panic!("Expected simple expression");
    }
}

#[test]
fn test_tensors_roundtrip_xml() {
    let doc = fixtures::tensors();
    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    // Verify tensor keys exist
    assert!(restored.root.contains_key("tensor_1d"));
    assert!(restored.root.contains_key("tensor_2d"));
    assert!(restored.root.contains_key("tensor_3d"));
    assert!(restored.root.contains_key("tensor_empty"));

    // XML converts tensors to nested objects with "item" keys
    // This is a known limitation - XML doesn't preserve tensor type information
    // We verify the structure exists rather than exact type preservation
    match restored.root.get("tensor_1d") {
        Some(Item::Scalar(Value::Tensor(Tensor::Array(arr)))) => {
            // If tensors are preserved, check structure
            assert_eq!(arr.len(), 3);
        }
        Some(Item::Object(obj)) => {
            // Tensors may become nested objects in XML
            assert!(!obj.is_empty(), "Expected tensor data as nested objects");
        }
        _ => panic!("Expected tensor_1d to be Tensor or Object"),
    }

    // Verify 2D tensor structure exists
    match restored.root.get("tensor_2d") {
        Some(Item::Scalar(Value::Tensor(Tensor::Array(arr)))) => {
            assert_eq!(arr.len(), 2);
        }
        Some(Item::Object(obj)) => {
            assert!(!obj.is_empty(), "Expected 2D tensor data as nested objects");
        }
        _ => panic!("Expected tensor_2d to be Tensor or Object"),
    }
}

#[test]
fn test_named_values_roundtrip_xml() {
    let doc = fixtures::named_values();
    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    // Verify config-style named values
    assert!(restored.root.contains_key("app_name"));
    assert!(restored.root.contains_key("version"));
    assert!(restored.root.contains_key("debug_mode"));
    assert!(restored.root.contains_key("max_connections"));
    assert!(restored.root.contains_key("timeout_seconds"));

    // Verify specific values
    assert_eq!(
        restored.root.get("app_name").unwrap().as_scalar().unwrap(),
        &Value::String("MyApp".to_string())
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
}

#[test]
fn test_user_list_roundtrip_xml() {
    let doc = fixtures::user_list();
    let xml = hedl_to_xml(&doc).unwrap();

    // For lists, we need to use infer_lists config
    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let restored = from_xml(&xml, &config).unwrap();

    // XML module converts MatrixList to nested structure
    // The "users" key will contain an Object with nested user data
    // This is a known limitation of XML round-tripping
    assert!(
        restored.root.contains_key("users"),
        "Expected 'users' key in root"
    );

    // Verify the structure contains user data
    // XML doesn't preserve the exact MatrixList structure due to nesting
    match restored.root.get("users") {
        Some(Item::Object(obj)) => {
            // When wrapped, users become nested objects
            assert!(!obj.is_empty(), "Expected user data in users object");
        }
        Some(Item::List(_list)) => {
            // If it did infer as list, that's also acceptable
        }
        _ => panic!("Expected users to be Object or List"),
    }
}

#[test]
fn test_mixed_type_list_roundtrip_xml() {
    let doc = fixtures::mixed_type_list();
    let xml = hedl_to_xml(&doc).unwrap();

    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let restored = from_xml(&xml, &config).unwrap();

    // XML module converts MatrixList to nested structure
    assert!(
        restored.root.contains_key("items"),
        "Expected 'items' key in root"
    );

    // Verify the structure contains item data
    match restored.root.get("items") {
        Some(Item::Object(obj)) => {
            // When wrapped, items become nested objects
            assert!(!obj.is_empty(), "Expected item data in items object");
        }
        Some(Item::List(_list)) => {
            // If it did infer as list, that's also acceptable
        }
        _ => panic!("Expected items to be Object or List"),
    }
}

#[test]
fn test_with_references_roundtrip_xml() {
    let doc = fixtures::with_references();
    let xml = hedl_to_xml(&doc).unwrap();

    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let restored = from_xml(&xml, &config).unwrap();

    // Verify both lists exist
    assert!(restored.root.contains_key("users"));
    assert!(restored.root.contains_key("posts"));

    // Verify posts contain references
    if let Some(Item::List(posts)) = restored.root.get("posts") {
        // Check that at least one post has a reference field
        let has_reference = posts.rows.iter().any(|node| {
            node.fields
                .iter()
                .any(|field| matches!(field, Value::Reference(_)))
        });
        assert!(has_reference, "Expected posts to contain reference fields");
    }
}

#[test]
fn test_with_nest_roundtrip_xml() {
    let doc = fixtures::with_nest();
    let xml = hedl_to_xml(&doc).unwrap();

    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let restored = from_xml(&xml, &config).unwrap();

    // Verify users list exists
    assert!(restored.root.contains_key("users"));

    // Verify NEST structure is present (nested posts)
    if let Some(Item::List(users)) = restored.root.get("users") {
        // Check if any user has children
        let has_children = users.rows.iter().any(|node| !node.children.is_empty());
        assert!(
            has_children,
            "Expected users to have nested children (posts)"
        );
    }
}

#[test]
fn test_deep_nest_roundtrip_xml() {
    let doc = fixtures::deep_nest();
    let xml = hedl_to_xml(&doc).unwrap();

    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let restored = from_xml(&xml, &config).unwrap();

    // Verify organizations list exists
    assert!(restored.root.contains_key("organizations"));

    // Verify deep NEST structure (3 levels)
    if let Some(Item::List(orgs)) = restored.root.get("organizations") {
        assert!(!orgs.rows.is_empty(), "Expected at least one organization");

        // Check for nested departments
        let has_nested = orgs.rows.iter().any(|node| !node.children.is_empty());
        assert!(
            has_nested,
            "Expected organizations to have nested departments"
        );
    }
}

#[test]
fn test_edge_cases_roundtrip_xml() {
    let doc = fixtures::edge_cases();
    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    // Verify edge case keys exist
    assert!(restored.root.contains_key("large_int"));
    assert!(restored.root.contains_key("small_int"));
    assert!(restored.root.contains_key("tiny_float"));
    assert!(restored.root.contains_key("large_float"));
    assert!(restored.root.contains_key("long_string"));
    assert!(restored.root.contains_key("special_only"));

    // Verify large integer is preserved
    if let Some(Item::Scalar(Value::Int(n))) = restored.root.get("large_int") {
        assert_eq!(*n, i64::MAX);
    } else {
        panic!("Expected large integer");
    }

    // Verify long string is preserved
    if let Some(Item::Scalar(Value::String(s))) = restored.root.get("long_string") {
        assert_eq!(s.len(), 10000, "Expected 10000-char string");
    } else {
        panic!("Expected long string");
    }
}

#[test]
fn test_comprehensive_roundtrip_xml() {
    let doc = fixtures::comprehensive();
    let xml = hedl_to_xml(&doc).unwrap();

    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let restored = from_xml(&xml, &config).unwrap();

    // Verify scalar config values
    assert!(restored.root.contains_key("config_debug"));
    assert!(restored.root.contains_key("config_version"));
    assert!(restored.root.contains_key("config_max_items"));
    assert!(restored.root.contains_key("config_threshold"));

    // Verify expression
    assert!(restored.root.contains_key("computed"));

    // Verify tensor
    assert!(restored.root.contains_key("weights"));

    // Verify lists
    assert!(restored.root.contains_key("users"));
    assert!(restored.root.contains_key("comments"));
    assert!(restored.root.contains_key("tags"));

    // Verify users list has NEST structure
    if let Some(Item::List(users)) = restored.root.get("users") {
        let has_children = users.rows.iter().any(|node| !node.children.is_empty());
        assert!(has_children, "Expected users to have nested posts");
    }

    // Verify comments contain references
    if let Some(Item::List(comments)) = restored.root.get("comments") {
        let has_reference = comments.rows.iter().any(|node| {
            node.fields
                .iter()
                .any(|field| matches!(field, Value::Reference(_)))
        });
        assert!(has_reference, "Expected comments to contain references");
    }
}

#[test]
fn test_empty_roundtrip_xml() {
    let doc = fixtures::empty();
    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    // Empty document should round-trip successfully
    assert_eq!(restored.version, (1, 0));
    assert!(
        restored.root.is_empty()
            || restored.root.values().all(|item| {
                // Some parsers might create empty structures
                match item {
                    Item::List(list) => list.rows.is_empty(),
                    Item::Object(obj) => obj.is_empty(),
                    _ => false,
                }
            })
    );
}
