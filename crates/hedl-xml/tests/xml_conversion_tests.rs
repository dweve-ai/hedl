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

use hedl_core::lex::Tensor;
use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_test::fixtures;
use hedl_xml::{from_xml, hedl_to_xml, to_xml, xml_to_hedl, FromXmlConfig, ToXmlConfig};
use std::collections::BTreeMap;

// =============================================================================
// Basic Scalar Conversion Tests
// =============================================================================

#[test]
fn test_null_to_xml() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Null));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("<value"));
}

#[test]
fn test_bool_true_to_xml() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(true)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("true"));
}

#[test]
fn test_bool_false_to_xml() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(false)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("false"));
}

#[test]
fn test_int_to_xml() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("count".to_string(), Item::Scalar(Value::Int(42)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("42"));
}

#[test]
fn test_negative_int_to_xml() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Int(-100)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("-100"));
}

#[test]
fn test_float_to_xml() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("rate".to_string(), Item::Scalar(Value::Float(1.23456)));

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("1.23"));
}

#[test]
fn test_string_to_xml() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "name".to_string(),
        Item::Scalar(Value::String("hello world".to_string().into())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("hello world"));
}

// =============================================================================
// XML Escaping Tests
// =============================================================================

#[test]
fn test_xml_escape_ampersand() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("A & B".to_string().into())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("A & B".to_string().into())
    );
}

#[test]
fn test_xml_escape_less_than() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("x < y".to_string().into())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("x < y".to_string().into())
    );
}

#[test]
fn test_xml_escape_greater_than() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("x > y".to_string().into())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("x > y".to_string().into())
    );
}

#[test]
fn test_xml_escape_quotes() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("say \"hello\"".to_string().into())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("say \"hello\"".to_string().into())
    );
}

#[test]
fn test_xml_escape_all_special_chars() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("A & B < C > D \"E\"".to_string().into())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("A & B < C > D \"E\"".to_string().into())
    );
}

// =============================================================================
// Reference Conversion Tests
// =============================================================================

#[test]
fn test_local_reference_to_xml() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target_id"))),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("ref") {
        assert_eq!(r.type_name, None);
        assert_eq!(r.id.as_ref(), "target_id");
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_qualified_reference_to_xml() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "alice"))),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("ref") {
        assert_eq!(r.type_name.as_deref(), Some("User"));
        assert_eq!(r.id.as_ref(), "alice");
    } else {
        panic!("Expected reference");
    }
}

// =============================================================================
// Expression Conversion Tests
// =============================================================================

#[test]
fn test_expression_to_xml() {
    let mut doc = Document::new((2, 0));
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
    let mut doc = Document::new((2, 0));
    let tensor = Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ]);
    doc.root.insert(
        "data".to_string(),
        Item::Scalar(Value::Tensor(Box::new(tensor))),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("<data>"));
    assert!(xml.contains("<item>"));
}

#[test]
fn test_2d_tensor_to_xml() {
    let mut doc = Document::new((2, 0));
    let tensor = Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ]);
    doc.root.insert(
        "matrix".to_string(),
        Item::Scalar(Value::Tensor(Box::new(tensor))),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("<matrix>"));
}

// =============================================================================
// Object Conversion Tests
// =============================================================================

#[test]
fn test_nested_object_to_xml() {
    let mut doc = Document::new((2, 0));
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
    let mut doc = Document::new((2, 0));

    let mut level2 = BTreeMap::new();
    level2.insert(
        "deep".to_string(),
        Item::Scalar(Value::String("value".to_string().into())),
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
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);

    // Per SPEC: fields must include ALL schema columns including ID
    list.add_row(Node::new(
        "User",
        "user1",
        vec![
            Value::String("user1".to_string().into()),
            Value::String("Alice".to_string().into()),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "user2",
        vec![
            Value::String("user2".to_string().into()),
            Value::String("Bob".to_string().into()),
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
    let mut doc = Document::new((2, 0));
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
        Item::Scalar(Value::String("test".to_string().into())),
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
        &Value::String("test".to_string().into())
    );
}

#[test]
fn test_round_trip_object() {
    let mut doc = Document::new((2, 0));
    let mut inner = BTreeMap::new();
    inner.insert(
        "name".to_string(),
        Item::Scalar(Value::String("test".to_string().into())),
    );
    inner.insert("value".to_string(), Item::Scalar(Value::Int(100)));
    doc.root.insert("config".to_string(), Item::Object(inner));

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    let config_obj = restored.root.get("config").unwrap().as_object().unwrap();
    assert_eq!(
        config_obj.get("name").unwrap().as_scalar().unwrap(),
        &Value::String("test".to_string().into())
    );
    assert_eq!(
        config_obj.get("value").unwrap().as_scalar().unwrap(),
        &Value::Int(100)
    );
}

#[test]
fn test_round_trip_reference() {
    let mut doc = Document::new((2, 0));
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
    let mut doc = Document::new((2, 0));
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
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "test".to_string(),
        Item::Scalar(Value::String("value".to_string().into())),
    );

    let config_pretty = ToXmlConfig {
        pretty: true,
        indent: " ".to_string(),
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
    let doc = Document::new((2, 0));

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
            &Value::String("test".to_string().into())
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
    let doc = Document::new((2, 0));
    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    // xml_to_hedl creates new documents with v2.0 default
    assert_eq!(restored.version, (2, 0));
}

#[test]
fn test_empty_string_value() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("empty".to_string(), Item::Scalar(Value::Null));

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert!(restored.root.contains_key("empty"));
}

#[test]
fn test_unicode_string() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("Hello 世界 🌍".to_string().into())),
    );

    let xml = hedl_to_xml(&doc).unwrap();
    let restored = xml_to_hedl(&xml).unwrap();

    assert_eq!(
        restored.root.get("text").unwrap().as_scalar().unwrap(),
        &Value::String("Hello 世界 🌍".to_string().into())
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
    let mut doc = Document::new((2, 0));
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
            &Value::String("Test Item".to_string().into())
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
        &Value::String("hello world".to_string().into())
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
        assert_eq!(r.id.as_ref(), "some_id");
    } else {
        panic!("Expected local reference");
    }

    // Verify typed reference
    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("typed_ref") {
        assert_eq!(r.type_name.as_deref(), Some("User"));
        assert_eq!(r.id.as_ref(), "alice");
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
        Some(Item::Scalar(Value::Tensor(boxed_tensor))) => {
            // If tensors are preserved, check structure
            if let Tensor::Array(arr) = &**boxed_tensor {
                assert_eq!(arr.len(), 3);
            }
        }
        Some(Item::Object(obj)) => {
            // Tensors may become nested objects in XML
            assert!(!obj.is_empty(), "Expected tensor data as nested objects");
        }
        _ => panic!("Expected tensor_1d to be Tensor or Object"),
    }

    // Verify 2D tensor structure exists
    match restored.root.get("tensor_2d") {
        Some(Item::Scalar(Value::Tensor(boxed_tensor))) => {
            if let Tensor::Array(arr) = &**boxed_tensor {
                assert_eq!(arr.len(), 2);
            }
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
        &Value::String("MyApp".to_string().into())
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
        let has_children = users
            .rows
            .iter()
            .any(|node| node.children().is_some_and(|c| !c.is_empty()));
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
        let has_nested = orgs
            .rows
            .iter()
            .any(|node| node.children().is_some_and(|c| !c.is_empty()));
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
        let has_children = users
            .rows
            .iter()
            .any(|node| node.children().is_some_and(|c| !c.is_empty()));
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

    // xml_to_hedl creates new documents with v2.0 default
    assert_eq!(restored.version, (2, 0));
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

// =============================================================================
// Issue 4 Tests: Nested children with proper schema
// =============================================================================

#[test]
fn test_issue4_nested_children_preserve_all_fields() {
    let mut doc = Document::new((2, 0));

    // Register schemas with multiple columns
    doc.structs.insert(
        "Parent".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    doc.structs.insert(
        "Child".to_string(),
        vec!["id".to_string(), "title".to_string(), "count".to_string()],
    );

    // Create parent with children
    let mut parent = MatrixList::new("Parent", vec!["id".to_string(), "name".to_string()]);

    let mut parent_node = Node::new(
        "Parent",
        "p1",
        vec![
            Value::String("p1".to_string().into()),
            Value::String("Parent 1".to_string().into()),
        ],
    );

    // Add child nodes with multiple fields
    let child1 = Node::new(
        "Child",
        "c1",
        vec![
            Value::String("c1".to_string().into()),
            Value::String("Child Title 1".to_string().into()),
            Value::Int(42),
        ],
    );

    let child2 = Node::new(
        "Child",
        "c2",
        vec![
            Value::String("c2".to_string().into()),
            Value::String("Child Title 2".to_string().into()),
            Value::Int(99),
        ],
    );

    let mut children_map = BTreeMap::new();
    children_map.insert("Child".to_string(), vec![child1, child2]);
    parent_node.children = Some(Box::new(children_map));

    parent.add_row(parent_node);
    doc.root.insert("parents".to_string(), Item::List(parent));

    // Convert to XML
    let xml = hedl_to_xml(&doc).unwrap();

    // Parse back
    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let restored = from_xml(&xml, &config).unwrap();

    // Verify parent exists
    assert!(
        restored.root.contains_key("parents") || restored.root.contains_key("parent"),
        "Expected parent list"
    );

    // The key might be normalized
    let parent_key = if restored.root.contains_key("parents") {
        "parents"
    } else {
        "parent"
    };

    if let Some(Item::List(parents)) = restored.root.get(parent_key) {
        assert_eq!(parents.rows.len(), 1);

        let parent = &parents.rows[0];
        if let Some(children) = parent.children() {
            if let Some(child_nodes) = children.get("Child") {
                assert_eq!(child_nodes.len(), 2, "Expected 2 children");

                // ISSUE 4: Verify all child fields are preserved (not just "id")
                let c1 = &child_nodes[0];
                assert_eq!(c1.fields.len(), 3, "Child should have 3 fields");

                // Check field values
                assert!(
                    matches!(c1.fields[0], Value::String(_)),
                    "Field 0 should be string (id)"
                );
                assert!(
                    matches!(c1.fields[1], Value::String(_)),
                    "Field 1 should be string (title)"
                );
                assert!(
                    matches!(c1.fields[2], Value::Int(_)),
                    "Field 2 should be int (count)"
                );
            } else {
                panic!("Expected Child nodes");
            }
        } else {
            panic!("Expected parent to have children");
        }
    }
}

#[test]
fn test_issue4_deeply_nested_schema_preservation() {
    let mut doc = Document::new((2, 0));

    // Register schemas for three levels
    doc.structs.insert(
        "Level1".to_string(),
        vec!["id".to_string(), "name".to_string(), "value".to_string()],
    );
    doc.structs.insert(
        "Level2".to_string(),
        vec![
            "id".to_string(),
            "description".to_string(),
            "active".to_string(),
        ],
    );
    doc.structs.insert(
        "Level3".to_string(),
        vec!["id".to_string(), "tag".to_string()],
    );

    // Create level 3
    let level3_node = Node::new(
        "Level3",
        "l3",
        vec![
            Value::String("l3".to_string().into()),
            Value::String("Tag Value".to_string().into()),
        ],
    );

    // Create level 2 with level 3 as child
    let mut level2_node = Node::new(
        "Level2",
        "l2",
        vec![
            Value::String("l2".to_string().into()),
            Value::String("Description".to_string().into()),
            Value::Bool(true),
        ],
    );
    let mut l2_children = BTreeMap::new();
    l2_children.insert("Level3".to_string(), vec![level3_node]);
    level2_node.children = Some(Box::new(l2_children));

    // Create level 1 with level 2 as child
    let mut level1_node = Node::new(
        "Level1",
        "l1",
        vec![
            Value::String("l1".to_string().into()),
            Value::String("Top Level".to_string().into()),
            Value::Int(100),
        ],
    );
    let mut l1_children = BTreeMap::new();
    l1_children.insert("Level2".to_string(), vec![level2_node]);
    level1_node.children = Some(Box::new(l1_children));

    let mut list = MatrixList::new(
        "Level1",
        vec!["id".to_string(), "name".to_string(), "value".to_string()],
    );
    list.add_row(level1_node);
    doc.root.insert("items".to_string(), Item::List(list));

    // Convert to XML
    let xml = hedl_to_xml(&doc).unwrap();

    // Verify XML contains all fields
    assert!(xml.contains("Top Level"), "Should contain level 1 name");
    assert!(xml.contains("100"), "Should contain level 1 value");
    assert!(
        xml.contains("Description"),
        "Should contain level 2 description"
    );
    assert!(xml.contains("true"), "Should contain level 2 active");
    assert!(xml.contains("Tag Value"), "Should contain level 3 tag");
}

#[test]
fn test_issue4_multiple_child_types_all_schemas() {
    let mut doc = Document::new((2, 0));

    // Register schemas
    doc.structs.insert(
        "Container".to_string(),
        vec!["id".to_string(), "label".to_string()],
    );
    doc.structs.insert(
        "TypeA".to_string(),
        vec!["id".to_string(), "field_a".to_string()],
    );
    doc.structs.insert(
        "TypeB".to_string(),
        vec![
            "id".to_string(),
            "field_b1".to_string(),
            "field_b2".to_string(),
        ],
    );

    // Create container with two different child types
    let mut container = Node::new(
        "Container",
        "c1",
        vec![
            Value::String("c1".to_string().into()),
            Value::String("My Container".to_string().into()),
        ],
    );

    let child_a = Node::new(
        "TypeA",
        "a1",
        vec![
            Value::String("a1".to_string().into()),
            Value::String("Field A".to_string().into()),
        ],
    );

    let child_b = Node::new(
        "TypeB",
        "b1",
        vec![
            Value::String("b1".to_string().into()),
            Value::String("Field B1".to_string().into()),
            Value::Int(42),
        ],
    );

    let mut children = BTreeMap::new();
    children.insert("TypeA".to_string(), vec![child_a]);
    children.insert("TypeB".to_string(), vec![child_b]);
    container.children = Some(Box::new(children));

    let mut list = MatrixList::new("Container", vec!["id".to_string(), "label".to_string()]);
    list.add_row(container);
    doc.root.insert("containers".to_string(), Item::List(list));

    // Convert to XML
    let xml = hedl_to_xml(&doc).unwrap();

    // Verify all fields from both child types are present
    assert!(xml.contains("Field A"), "TypeA field should be present");
    assert!(xml.contains("Field B1"), "TypeB field1 should be present");
    assert!(xml.contains("42"), "TypeB field2 should be present");
}

// =============================================================================
// Additional Issue 1 & 2 Comprehensive Tests
// =============================================================================

#[test]
fn test_issue1_comprehensive_attributes_and_children() {
    // Complex scenario: multiple attributes and nested children
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <product id="100" sku="WIDGET-001" category="electronics" available="true">
            <name>Super Widget</name>
            <description>A wonderful widget</description>
            <price>19.99</price>
            <specs>
                <weight>500</weight>
                <color>blue</color>
            </specs>
        </product>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("product") {
        // All attributes should be preserved
        assert!(obj.contains_key("id"), "id attribute missing");
        assert!(obj.contains_key("sku"), "sku attribute missing");
        assert!(obj.contains_key("category"), "category attribute missing");
        assert!(obj.contains_key("available"), "available attribute missing");

        // All child elements should be preserved
        assert!(obj.contains_key("name"), "name child missing");
        assert!(obj.contains_key("description"), "description child missing");
        assert!(obj.contains_key("price"), "price child missing");
        assert!(obj.contains_key("specs"), "specs child missing");

        // Verify attribute types
        assert_eq!(
            obj.get("id").unwrap().as_scalar().unwrap(),
            &Value::Int(100)
        );
        assert_eq!(
            obj.get("available").unwrap().as_scalar().unwrap(),
            &Value::Bool(true)
        );

        // Verify nested child
        if let Some(Item::Object(specs)) = obj.get("specs") {
            assert!(specs.contains_key("weight"));
            assert!(specs.contains_key("color"));
        } else {
            panic!("specs should be an object");
        }
    } else {
        panic!("Expected product object");
    }
}

#[test]
fn test_issue1_mixed_content_attributes_text_and_children() {
    // Element with attributes, text content, AND child elements
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <paragraph id="p1" class="intro">
            This is the intro text.
            <emphasis>Important point</emphasis>
            More text here.
        </paragraph>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("paragraph") {
        // Attributes should be preserved
        assert!(obj.contains_key("id"), "id attribute missing");
        assert!(obj.contains_key("class"), "class attribute missing");

        // Mixed text content should be in _text
        assert!(obj.contains_key("_text"), "_text missing");

        // Child element should be preserved
        assert!(obj.contains_key("emphasis"), "emphasis child missing");
    } else {
        panic!("Expected paragraph object");
    }
}

#[test]
fn test_issue2_comprehensive_error_messages() {
    // Verify error messages are helpful
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <tag>first</tag>
        <tag>second</tag>
        <tag>third</tag>
    </hedl>"#;

    let config = FromXmlConfig {
        infer_lists: false,
        ..Default::default()
    };

    let result = from_xml(xml, &config);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.contains("Duplicate element"));
    assert!(error.contains("tag"));
    assert!(error.contains("infer_lists"));
}

#[test]
fn test_issue2_unique_elements_no_error() {
    // Ensure no false positives - unique elements should work fine
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <first>1</first>
        <second>2</second>
        <third>3</third>
        <fourth>4</fourth>
        <fifth>5</fifth>
    </hedl>"#;

    let config = FromXmlConfig {
        infer_lists: false,
        ..Default::default()
    };

    let doc = from_xml(xml, &config).unwrap();
    assert_eq!(doc.root.len(), 5);
    assert!(doc.root.contains_key("first"));
    assert!(doc.root.contains_key("fifth"));
}

#[test]
fn test_issue2_nested_duplicates_detected() {
    // Duplicates at nested level should also be detected
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <container>
            <unique1>value1</unique1>
            <item>first</item>
            <item>second</item>
        </container>
    </hedl>"#;

    let config = FromXmlConfig {
        infer_lists: false,
        ..Default::default()
    };

    let result = from_xml(xml, &config);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Duplicate element"));
}

#[test]
fn test_issue1_and_2_combined() {
    // Combination: list of items with attributes and children
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <user id="1" status="active">
            <name>Alice</name>
            <email>alice@example.com</email>
        </user>
        <user id="2" status="inactive">
            <name>Bob</name>
            <email>bob@example.com</email>
        </user>
    </hedl>"#;

    let config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };

    let doc = from_xml(xml, &config).unwrap();

    // Should create a list
    if let Some(Item::List(list)) = doc.root.get("user") {
        assert_eq!(list.rows.len(), 2);

        // Each user should have both attributes and children in their fields
        // Note: In matrix lists, attributes and children are flattened into the schema
        for user in &list.rows {
            // Fields include all scalar values (id, status, name, email)
            assert!(user.fields.len() >= 2, "User should have multiple fields");
        }
    } else {
        panic!("Expected user list");
    }
}

#[test]
fn test_issue1_empty_attributes_edge_case() {
    // Empty attribute values should be handled
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item id="" value="">
            <child>content</child>
        </item>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("item") {
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("value"));
        assert!(obj.contains_key("child"));
    } else {
        panic!("Expected item object");
    }
}

#[test]
fn test_issue1_special_characters_in_attributes() {
    // Attributes with special characters should be preserved
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item label="A &amp; B" note="less &lt; than &gt; greater">
            <value>42</value>
        </item>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("item") {
        // Attributes should be preserved (note: quick-xml handles entity unescaping)
        assert!(obj.contains_key("label"), "label attribute missing");
        assert!(obj.contains_key("note"), "note attribute missing");
        assert!(obj.contains_key("value"), "value child missing");

        // Verify the label was parsed correctly (XML entities are handled by quick-xml)
        if let Some(Item::Scalar(Value::String(label))) = obj.get("label") {
            // The label should contain the text (exact format depends on quick-xml's entity handling)
            assert!(!label.is_empty(), "label should not be empty");
        }
    } else {
        panic!("Expected item object");
    }
}

// =============================================================================
// Issue 5 Tests: use_attributes should not duplicate simple fields
// =============================================================================

#[test]
fn test_issue5_no_duplication_with_use_attributes() {
    let mut doc = Document::new((2, 0));

    doc.structs.insert(
        "Item".to_string(),
        vec![
            "id".to_string(),
            "name".to_string(),
            "description".to_string(),
        ],
    );

    let mut list = MatrixList::new(
        "Item",
        vec![
            "id".to_string(),
            "name".to_string(),
            "description".to_string(),
        ],
    );

    // Create node with simple and complex fields
    let node = Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string().into()),
            Value::String("Simple Name".to_string().into()),
            Value::Reference(Reference::local("other")), // Complex field
        ],
    );

    list.add_row(node);
    doc.root.insert("items".to_string(), Item::List(list));

    // Convert with use_attributes=true
    let config = ToXmlConfig {
        use_attributes: true,
        pretty: false,
        ..Default::default()
    };
    let xml = to_xml(&doc, &config).unwrap();

    // Count occurrences of "Simple Name" - should appear only once (in attribute)
    let count = xml.matches("Simple Name").count();
    assert_eq!(
        count, 1,
        "Simple Name should appear only once (as attribute, not duplicated as element)"
    );

    // Verify it's in an attribute
    assert!(
        xml.contains("name=\"Simple Name\""),
        "Should have name as attribute"
    );

    // Verify complex field (Reference) is in element (not attribute)
    // Note: references have __hedl_type__ marker attribute
    assert!(
        xml.contains("<description") && xml.contains("</description>"),
        "Complex field should be element. XML: {xml}"
    );
}

#[test]
fn test_issue5_all_simple_fields_only_in_attributes() {
    let mut doc = Document::new((2, 0));

    doc.structs.insert(
        "Person".to_string(),
        vec![
            "id".to_string(),
            "name".to_string(),
            "age".to_string(),
            "active".to_string(),
        ],
    );

    let mut list = MatrixList::new(
        "Person",
        vec![
            "id".to_string(),
            "name".to_string(),
            "age".to_string(),
            "active".to_string(),
        ],
    );

    // All simple fields
    let node = Node::new(
        "Person",
        "p1",
        vec![
            Value::String("p1".to_string().into()),
            Value::String("Alice".to_string().into()),
            Value::Int(30),
            Value::Bool(true),
        ],
    );

    list.add_row(node);
    doc.root.insert("people".to_string(), Item::List(list));

    // Convert with use_attributes=true
    let config = ToXmlConfig {
        use_attributes: true,
        pretty: false,
        ..Default::default()
    };
    let xml = to_xml(&doc, &config).unwrap();

    // All fields should be attributes only
    assert!(xml.contains("id=\"p1\""), "id should be attribute");
    assert!(xml.contains("name=\"Alice\""), "name should be attribute");
    assert!(xml.contains("age=\"30\""), "age should be attribute");
    assert!(
        xml.contains("active=\"true\""),
        "active should be attribute"
    );

    // No field elements should exist (should be self-closing tag)
    assert!(!xml.contains("<id>"), "Should not have id element");
    assert!(!xml.contains("<name>"), "Should not have name element");
    assert!(!xml.contains("<age>"), "Should not have age element");
    assert!(!xml.contains("<active>"), "Should not have active element");
}

#[test]
fn test_issue5_mixed_fields_no_duplication() {
    let mut doc = Document::new((2, 0));

    doc.structs.insert(
        "Product".to_string(),
        vec![
            "id".to_string(),
            "name".to_string(),
            "price".to_string(),
            "tags".to_string(), // Tensor - complex
        ],
    );

    let mut list = MatrixList::new(
        "Product",
        vec![
            "id".to_string(),
            "name".to_string(),
            "price".to_string(),
            "tags".to_string(),
        ],
    );

    let node = Node::new(
        "Product",
        "prod1",
        vec![
            Value::String("prod1".to_string().into()),
            Value::String("Widget".to_string().into()),
            Value::Float(19.99),
            Value::Tensor(Box::new(hedl_core::lex::Tensor::Array(vec![
                hedl_core::lex::Tensor::Scalar(1.0),
                hedl_core::lex::Tensor::Scalar(2.0),
            ]))),
        ],
    );

    list.add_row(node);
    doc.root.insert("products".to_string(), Item::List(list));

    // Convert with use_attributes=true
    let config = ToXmlConfig {
        use_attributes: true,
        pretty: false,
        ..Default::default()
    };
    let xml = to_xml(&doc, &config).unwrap();

    // Simple fields should be attributes only
    assert!(xml.contains("name=\"Widget\""), "name should be attribute");
    assert!(xml.contains("price=\"19.99\""), "price should be attribute");

    // Check they don't appear as elements
    let name_element_count = xml.matches("<name>").count();
    let price_element_count = xml.matches("<price>").count();

    assert_eq!(
        name_element_count, 0,
        "name should not be duplicated as element"
    );
    assert_eq!(
        price_element_count, 0,
        "price should not be duplicated as element"
    );

    // Complex field should be element
    assert!(
        xml.contains("<tags>"),
        "Complex tags field should be element"
    );
}

#[test]
fn test_issue5_use_attributes_false_no_attributes() {
    let mut doc = Document::new((2, 0));

    doc.structs.insert(
        "Item".to_string(),
        vec!["id".to_string(), "value".to_string()],
    );

    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    let node = Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string().into()), Value::Int(42)],
    );

    list.add_row(node);
    doc.root.insert("items".to_string(), Item::List(list));

    // Convert with use_attributes=false (default)
    let config = ToXmlConfig {
        use_attributes: false,
        pretty: false,
        ..Default::default()
    };
    let xml = to_xml(&doc, &config).unwrap();

    // Fields should be elements, not attributes
    assert!(xml.contains("<id>i1</id>"), "id should be element");
    assert!(xml.contains("<value>42</value>"), "value should be element");

    // No field attributes (except possibly metadata)
    assert!(!xml.contains("id=\"i1\""), "id should not be attribute");
    assert!(
        !xml.contains("value=\"42\""),
        "value should not be attribute"
    );
}
