// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Schema mapping tests for hedl-parquet
//!
//! Tests type inference, schema generation, and column mapping between HEDL and Parquet.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_parquet::{
    from_parquet_bytes, to_parquet_bytes, to_parquet_bytes_with_config, ToParquetConfig,
};

// =============================================================================
// Type Inference Tests
// =============================================================================

#[test]
fn test_infer_bool_column() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Flag", vec!["id".to_string(), "active".to_string()]);

    list.add_row(Node::new(
        "Flag",
        "f1",
        vec![Value::String("f1".to_string().into()), Value::Bool(true)],
    ));
    list.add_row(Node::new(
        "Flag",
        "f2",
        vec![Value::String("f2".to_string().into()), Value::Bool(false)],
    ));

    doc.root.insert("flags".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("flags") {
        assert_eq!(list.rows.len(), 2);
        assert!(matches!(list.rows[0].fields[1], Value::Bool(true)));
        assert!(matches!(list.rows[1].fields[1], Value::Bool(false)));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_infer_int_column() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Counter", vec!["id".to_string(), "count".to_string()]);

    list.add_row(Node::new(
        "Counter",
        "c1",
        vec![Value::String("c1".to_string().into()), Value::Int(0)],
    ));
    list.add_row(Node::new(
        "Counter",
        "c2",
        vec![Value::String("c2".to_string().into()), Value::Int(i64::MAX)],
    ));
    list.add_row(Node::new(
        "Counter",
        "c3",
        vec![Value::String("c3".to_string().into()), Value::Int(i64::MIN)],
    ));

    doc.root.insert("counters".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("counters") {
        assert_eq!(list.rows.len(), 3);
        assert!(matches!(list.rows[0].fields[1], Value::Int(0)));
        assert!(matches!(list.rows[1].fields[1], Value::Int(v) if v == i64::MAX));
        assert!(matches!(list.rows[2].fields[1], Value::Int(v) if v == i64::MIN));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_infer_float_column() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Metric", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Metric",
        "m1",
        vec![Value::String("m1".to_string().into()), Value::Float(0.0)],
    ));
    list.add_row(Node::new(
        "Metric",
        "m2",
        vec![
            Value::String("m2".to_string().into()),
            Value::Float(std::f64::consts::PI),
        ],
    ));
    list.add_row(Node::new(
        "Metric",
        "m3",
        vec![Value::String("m3".to_string().into()), Value::Float(-1.5)],
    ));
    list.add_row(Node::new(
        "Metric",
        "m4",
        vec![
            Value::String("m4".to_string().into()),
            Value::Float(f64::MAX),
        ],
    ));

    doc.root.insert("metrics".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("metrics") {
        assert_eq!(list.rows.len(), 4);
        assert!(matches!(list.rows[0].fields[1], Value::Float(v) if v == 0.0));
        assert!(
            matches!(list.rows[1].fields[1], Value::Float(v) if (v - std::f64::consts::PI).abs() < 0.01)
        );
        assert!(matches!(list.rows[2].fields[1], Value::Float(v) if v == -1.5));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_infer_string_column() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Text", vec!["id".to_string(), "content".to_string()]);

    list.add_row(Node::new(
        "Text",
        "t1",
        vec![
            Value::String("t1".to_string().into()),
            Value::String("Hello, world!".to_string().into()),
        ],
    ));
    list.add_row(Node::new(
        "Text",
        "t2",
        vec![
            Value::String("t2".to_string().into()),
            Value::String(String::new().into()),
        ],
    ));
    list.add_row(Node::new(
        "Text",
        "t3",
        vec![
            Value::String("t3".to_string().into()),
            Value::String("Unicode: 你好 🌍".to_string().into()),
        ],
    ));

    doc.root.insert("texts".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("texts") {
        assert_eq!(list.rows.len(), 3);
        assert!(matches!(&list.rows[0].fields[1], Value::String(s) if &**s == "Hello, world!"));
        assert!(matches!(&list.rows[1].fields[1], Value::String(s) if s.is_empty()));
        assert!(matches!(&list.rows[2].fields[1], Value::String(s) if &**s == "Unicode: 你好 🌍"));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_reference_serialization() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Post", vec!["id".to_string(), "author".to_string()]);

    list.add_row(Node::new(
        "Post",
        "p1",
        vec![
            Value::String("p1".to_string().into()),
            Value::Reference(Reference::qualified("User", "alice")),
        ],
    ));
    list.add_row(Node::new(
        "Post",
        "p2",
        vec![
            Value::String("p2".to_string().into()),
            Value::Reference(Reference::local("bob")),
        ],
    ));

    doc.root.insert("posts".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("posts") {
        assert_eq!(list.rows.len(), 2);

        // Qualified reference
        if let Value::Reference(r) = &list.rows[0].fields[1] {
            assert_eq!(r.type_name.as_deref(), Some("User"));
            assert_eq!(&*r.id, "alice");
        } else {
            panic!("Expected reference at row 0");
        }

        // Local reference
        if let Value::Reference(r) = &list.rows[1].fields[1] {
            assert_eq!(r.type_name, None);
            assert_eq!(&*r.id, "bob");
        } else {
            panic!("Expected reference at row 1");
        }
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_tensor_serialization() {
    use hedl_core::lex::Tensor as LexTensor;

    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "matrix".to_string()]);

    // Create a 2D tensor using Array of Arrays
    let tensor = LexTensor::Array(vec![
        LexTensor::Array(vec![LexTensor::Scalar(1.0), LexTensor::Scalar(2.0)]),
        LexTensor::Array(vec![LexTensor::Scalar(3.0), LexTensor::Scalar(4.0)]),
    ]);

    list.add_row(Node::new(
        "Data",
        "d1",
        vec![
            Value::String("d1".to_string().into()),
            Value::Tensor(Box::new(tensor)),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Tensors are serialized as strings, so they come back as strings
    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 1);
        // Tensor is now a string representation
        assert!(matches!(&list.rows[0].fields[1], Value::String(_)));
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Mixed Type Column Tests
// =============================================================================

#[test]
fn test_mixed_types_with_nulls() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new(
        "Mixed",
        vec!["id".to_string(), "value".to_string(), "flag".to_string()],
    );

    list.add_row(Node::new(
        "Mixed",
        "m1",
        vec![
            Value::String("m1".to_string().into()),
            Value::Int(42),
            Value::Bool(true),
        ],
    ));
    list.add_row(Node::new(
        "Mixed",
        "m2",
        vec![
            Value::String("m2".to_string().into()),
            Value::Null,
            Value::Bool(false),
        ],
    ));
    list.add_row(Node::new(
        "Mixed",
        "m3",
        vec![
            Value::String("m3".to_string().into()),
            Value::Int(0),
            Value::Null,
        ],
    ));

    doc.root.insert("mixed".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("mixed") {
        assert_eq!(list.rows.len(), 3);
        assert!(matches!(list.rows[0].fields[1], Value::Int(42)));
        assert!(matches!(list.rows[1].fields[1], Value::Null));
        assert!(matches!(list.rows[2].fields[2], Value::Null));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_all_nulls_column() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("NullTest", vec!["id".to_string(), "null_col".to_string()]);

    for i in 0..10 {
        list.add_row(Node::new(
            "NullTest",
            format!("n{i}"),
            vec![Value::String(format!("n{i}").into()), Value::Null],
        ));
    }

    doc.root.insert("nulls".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("nulls") {
        assert_eq!(list.rows.len(), 10);
        for row in &list.rows {
            assert!(matches!(row.fields[1], Value::Null));
        }
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Type Coercion Tests
// =============================================================================

#[test]
fn test_type_coercion_disabled_by_default() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "number".to_string()]);

    // First row establishes Int type
    list.add_row(Node::new(
        "Data",
        "d1",
        vec![Value::String("d1".to_string().into()), Value::Int(42)],
    ));

    // Second row has mismatched type (string instead of int)
    list.add_row(Node::new(
        "Data",
        "d2",
        vec![
            Value::String("d2".to_string().into()),
            Value::String("not a number".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig::default(); // coerce_types = false
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 2);
        assert!(matches!(list.rows[0].fields[1], Value::Int(42)));
        // Type mismatch should write null with coerce_types = false
        assert!(matches!(list.rows[1].fields[1], Value::Null));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_type_coercion_enabled() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "number".to_string()]);

    // First row establishes Int type
    list.add_row(Node::new(
        "Data",
        "d1",
        vec![Value::String("d1".to_string().into()), Value::Int(42)],
    ));

    // Second row has mismatched type
    list.add_row(Node::new(
        "Data",
        "d2",
        vec![
            Value::String("d2".to_string().into()),
            Value::String("not a number".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig::default().with_type_coercion(true);
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 2);
        assert!(matches!(list.rows[0].fields[1], Value::Int(42)));
        // Type mismatch should coerce to 0 with coerce_types = true
        assert!(matches!(list.rows[1].fields[1], Value::Int(0)));
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Wide Schema Tests
// =============================================================================

#[test]
fn test_wide_schema_50_columns() {
    let mut doc = Document::new((2, 0));

    // Create schema with 50 columns
    let mut schema = vec!["id".to_string()];
    for i in 1..50 {
        schema.push(format!("col{i}"));
    }

    let mut list = MatrixList::new("Wide", schema.clone());

    // Create row with 50 values
    let mut fields = vec![Value::String("w1".to_string().into())];
    for i in 1..50 {
        fields.push(Value::Int(i64::from(i)));
    }

    list.add_row(Node::new("Wide", "w1", fields));
    doc.root.insert("wide".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("wide") {
        assert_eq!(list.schema.len(), 50);
        assert_eq!(list.rows[0].fields.len(), 50);
        // Verify all values preserved
        for i in 1..50 {
            assert!(matches!(list.rows[0].fields[i], Value::Int(v) if v == i as i64));
        }
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_narrow_schema_single_column() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Narrow", vec!["id".to_string()]);

    list.add_row(Node::new(
        "Narrow",
        "n1",
        vec![Value::String("n1".to_string().into())],
    ));

    doc.root.insert("narrow".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("narrow") {
        assert_eq!(list.schema.len(), 1);
        assert_eq!(list.rows[0].fields.len(), 1);
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Column Name Sanitization Tests
// =============================================================================

#[test]
fn test_column_name_with_special_chars() {
    let mut doc = Document::new((2, 0));

    // Column names with special characters should be sanitized
    let mut list = MatrixList::new(
        "Special",
        vec![
            "id".to_string(),
            "col with spaces".to_string(),
            "col-with-dashes".to_string(),
        ],
    );

    list.add_row(Node::new(
        "Special",
        "s1",
        vec![
            Value::String("s1".to_string().into()),
            Value::Int(1),
            Value::Int(2),
        ],
    ));

    doc.root.insert("special".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("special") {
        assert_eq!(list.rows.len(), 1);
        // Column names may be sanitized but values preserved
        assert!(matches!(list.rows[0].fields[1], Value::Int(1)));
        assert!(matches!(list.rows[0].fields[2], Value::Int(2)));
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Expression Serialization Tests
// =============================================================================

#[test]
fn test_expression_serialization() {
    use hedl_core::lex::{Expression as LexExpression, SourcePos, Span};

    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Expr", vec!["id".to_string(), "formula".to_string()]);

    // Create a simple expression
    let expr = LexExpression::Identifier {
        name: "a".to_string(),
        span: Span::new(SourcePos::new(0, 0), SourcePos::new(0, 1)),
    };

    list.add_row(Node::new(
        "Expr",
        "e1",
        vec![
            Value::String("e1".to_string().into()),
            Value::Expression(Box::new(expr)),
        ],
    ));

    doc.root.insert("exprs".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Expressions are serialized as strings
    if let Some(Item::List(list)) = restored.root.get("exprs") {
        assert_eq!(list.rows.len(), 1);
        // Expression should be serialized as string with $() wrapper
        assert!(matches!(&list.rows[0].fields[1], Value::String(_)));
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Multiple List Tests
// =============================================================================

#[test]
fn test_multiple_lists_warning() {
    let mut doc = Document::new((2, 0));

    let mut list1 = MatrixList::new("Type1", vec!["id".to_string()]);
    list1.add_row(Node::new(
        "Type1",
        "t1",
        vec![Value::String("t1".to_string().into())],
    ));

    let mut list2 = MatrixList::new("Type2", vec!["id".to_string()]);
    list2.add_row(Node::new(
        "Type2",
        "t2",
        vec![Value::String("t2".to_string().into())],
    ));

    doc.root.insert("list1".to_string(), Item::List(list1));
    doc.root.insert("list2".to_string(), Item::List(list2));

    // This should produce a warning and only write the first list
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Only first list should be preserved (lexicographically)
    assert!(restored.root.contains_key("list1") || restored.root.contains_key("list2"));
    // Cannot have both since Parquet = one table per file
    assert!(!(restored.root.contains_key("list1") && restored.root.contains_key("list2")));
}
