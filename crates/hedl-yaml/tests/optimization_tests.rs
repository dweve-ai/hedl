// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tests verifying that serialization optimizations maintain correctness
//!
//! These tests ensure that the performance optimizations in `to_yaml.rs`
//! do not change the output or behavior of YAML serialization.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_yaml::{from_yaml, to_yaml, FromYamlConfig, ToYamlConfig};
use std::collections::BTreeMap;

/// Test that string constant optimization doesn't change output
#[test]
fn test_metadata_keys_unchanged() {
    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };
    let mut list = MatrixList::new(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    list.add_row(Node::new(
        "User",
        "u1",
        vec![
            Value::String("u1".to_string().into()),
            Value::String("Alice".to_string().into()),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    let config = ToYamlConfig {
        include_metadata: true,
        ..Default::default()
    };
    let yaml = to_yaml(&doc, &config).unwrap();

    // Verify metadata keys appear in output
    assert!(yaml.contains("__type__"));
    assert!(yaml.contains("__schema__"));
    assert!(yaml.contains("items"));

    // Verify round-trip works
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();
    assert_eq!(restored.root.len(), doc.root.len());
}

/// Test that field name caching works for fields 0-99 and beyond
#[test]
fn test_field_name_caching() {
    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };

    // Create a matrix list with more than 100 columns to test cache boundary
    let mut schema = vec!["id".to_string()];
    for i in 0..105 {
        schema.push(format!("field_{i}"));
    }

    let mut list = MatrixList::new("Record".to_string(), schema.clone());

    // Add a row with all fields populated
    let mut fields = vec![Value::String("r1".to_string().into())];
    for i in 0..105 {
        fields.push(Value::Int(i));
    }
    list.add_row(Node::new("Record", "r1", fields));

    doc.root.insert("records".to_string(), Item::List(list));

    let config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &config).unwrap();

    // Verify all field names appear
    for i in 0..105 {
        assert!(yaml.contains(&format!("field_{i}")), "Missing field_{i}");
    }

    // Verify round-trip
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();
    let restored_list = restored.root.get("records").unwrap().as_list().unwrap();
    assert_eq!(restored_list.schema.len(), 106); // id + 105 fields
}

/// Test that pre-allocation doesn't affect correctness
#[test]
fn test_pre_allocation_correctness() {
    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };

    // Create objects with varying sizes to test capacity estimation
    for i in 0..10 {
        let mut obj = BTreeMap::new();
        for j in 0..i {
            obj.insert(format!("field_{j}"), Item::Scalar(Value::Int(i64::from(j))));
        }
        doc.root.insert(format!("obj_{i}"), Item::Object(obj));
    }

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    assert_eq!(restored.root.len(), doc.root.len());
    for i in 0..10 {
        let obj = restored
            .root
            .get(&format!("obj_{i}"))
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(obj.len(), i);
    }
}

/// Test that schema pre-conversion maintains correctness
#[test]
fn test_schema_pre_conversion() {
    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };
    let mut list = MatrixList::new(
        "Item".to_string(),
        vec!["id".to_string(), "name".to_string(), "value".to_string()],
    );

    // Add multiple rows to ensure schema is reused correctly
    for i in 0..20 {
        list.add_row(Node::new(
            "Item",
            format!("i{i}"),
            vec![
                Value::String(format!("i{i}").into()),
                Value::String(format!("Item {i}").into()),
                Value::Int(i),
            ],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    let restored_list = restored.root.get("items").unwrap().as_list().unwrap();
    assert_eq!(restored_list.rows.len(), 20);
    assert_eq!(restored_list.schema.len(), 3);

    // Verify all rows have correct schema fields
    for row in &restored_list.rows {
        assert_eq!(row.fields.len(), 3);
    }
}

/// Test that type value pre-allocation works correctly
#[test]
fn test_type_value_pre_allocation() {
    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };
    let mut list = MatrixList::new("CustomType".to_string(), vec!["id".to_string()]);

    for i in 0..10 {
        list.add_row(Node::new(
            "CustomType",
            format!("c{i}"),
            vec![Value::String(format!("c{i}").into())],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToYamlConfig {
        include_metadata: true,
        ..Default::default()
    };
    let yaml = to_yaml(&doc, &config).unwrap();

    // Count occurrences of CustomType - should appear once in schema + once per row
    let count = yaml.matches("CustomType").count();
    assert!(
        count >= 10,
        "CustomType should appear at least 10 times, found {count}"
    );

    // Verify round-trip preserves type
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();
    let restored_list = restored.root.get("items").unwrap().as_list().unwrap();
    assert_eq!(restored_list.type_name, "CustomType");
}

/// Test that optimized expression formatting maintains correctness
#[test]
fn test_expression_formatting() {
    use hedl_core::lex::{ExprLiteral, Expression, Span};

    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };
    let mut obj = BTreeMap::new();

    // Test various expression types
    obj.insert(
        "simple".to_string(),
        Item::Scalar(Value::Expression(Box::new(Expression::Identifier {
            name: "x".to_string(),
            span: Span::synthetic(),
        }))),
    );

    obj.insert(
        "call".to_string(),
        Item::Scalar(Value::Expression(Box::new(Expression::Call {
            name: "add".to_string(),
            args: vec![
                Expression::Identifier {
                    name: "a".to_string(),
                    span: Span::synthetic(),
                },
                Expression::Literal {
                    value: ExprLiteral::Int(1),
                    span: Span::synthetic(),
                },
            ],
            span: Span::synthetic(),
        }))),
    );

    doc.root
        .insert("expressions".to_string(), Item::Object(obj));

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();

    // Verify expressions are wrapped correctly
    assert!(yaml.contains("$(x)"));
    assert!(yaml.contains("$(add(a, 1))"));

    // Verify round-trip
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();
    let restored_obj = restored
        .root
        .get("expressions")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(restored_obj.len(), 2);

    let simple_expr = restored_obj.get("simple").unwrap().as_scalar().unwrap();
    if let Value::Expression(e) = simple_expr {
        assert_eq!(e.to_string(), "x");
    } else {
        panic!("Expected expression");
    }
}

/// Test large document to verify all optimizations work together
#[test]
fn test_large_document_optimizations() {
    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };

    // Create a mix of objects, lists, and expressions
    for i in 0..100 {
        let mut obj = BTreeMap::new();
        obj.insert("id".to_string(), Item::Scalar(Value::Int(i)));
        obj.insert(
            "name".to_string(),
            Item::Scalar(Value::String(format!("Item {i}").into())),
        );
        doc.root.insert(format!("obj_{i}"), Item::Object(obj));
    }

    let mut list = MatrixList::new(
        "Record".to_string(),
        vec!["id".to_string(), "value".to_string()],
    );

    for i in 0..100 {
        list.add_row(Node::new(
            "Record",
            format!("r{i}"),
            vec![Value::String(format!("r{i}").into()), Value::Int(i * 10)],
        ));
    }

    doc.root.insert("records".to_string(), Item::List(list));

    let config = ToYamlConfig {
        include_metadata: true,
        ..Default::default()
    };

    let yaml = to_yaml(&doc, &config).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    // Verify document structure preserved
    assert_eq!(restored.root.len(), 101); // 100 objects + 1 list
    let restored_list = restored.root.get("records").unwrap().as_list().unwrap();
    assert_eq!(restored_list.rows.len(), 100);
}

/// Test that optimizations don't affect empty/minimal cases
#[test]
fn test_edge_cases() {
    // Empty document
    let doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };
    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    assert!(!yaml.is_empty());

    // Single scalar
    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Int(42)));
    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();
    assert_eq!(restored.root.len(), 1);

    // Empty list
    let mut doc = Document {
        version: (1, 0),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
    };
    let list = MatrixList::new("Empty".to_string(), vec!["id".to_string()]);
    doc.root.insert("empty".to_string(), Item::List(list));
    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();
    let restored_list = restored.root.get("empty").unwrap().as_list().unwrap();
    assert_eq!(restored_list.rows.len(), 0);
}
