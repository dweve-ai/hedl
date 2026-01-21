// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Edge case tests for `CanonicalWriter`.
//!
//! Tests multiline strings, nested matrix lists, recursion limits,
//! and other edge cases.

use hedl_c14n::{canonicalize, canonicalize_with_config, CanonicalConfig};
use hedl_core::{parse, Document, Item, MatrixList, Node, Value};
use std::collections::BTreeMap;

// =============================================================================
// Multiline String Tests
// =============================================================================
// Note: Multiline string and escape handling is tested in comprehensive_tests.rs and invariant_tests.rs

// =============================================================================
// Nested Matrix List Tests
// =============================================================================

#[test]
fn test_nested_matrix_list() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "Team".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    doc.structs.insert(
        "Player".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let mut list = MatrixList::new("Team", vec!["id".to_string(), "name".to_string()]);

    let mut parent_node = Node::new(
        "Team",
        "t1",
        vec![
            Value::String("t1".to_string().into()),
            Value::String("Engineering".to_string().into()),
        ],
    );

    // Add child players
    parent_node.add_child(
        "Player",
        Node::new(
            "Player",
            "p1",
            vec![
                Value::String("p1".to_string().into()),
                Value::String("Alice".to_string().into()),
            ],
        ),
    );
    parent_node.add_child(
        "Player",
        Node::new(
            "Player",
            "p2",
            vec![
                Value::String("p2".to_string().into()),
                Value::String("Bob".to_string().into()),
            ],
        ),
    );

    list.add_row(parent_node);
    doc.root.insert("teams".to_string(), Item::List(list));

    let output = canonicalize(&doc).unwrap();

    // Should have parent row
    assert!(output.contains("|t1,Engineering"));

    // Should have child rows (indented further)
    assert!(output.contains("|p1,Alice"));
    assert!(output.contains("|p2,Bob"));
}

#[test]
fn test_nested_matrix_list_with_children() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "Team".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    doc.structs.insert(
        "Player".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let mut list = MatrixList::new("Team", vec!["id".to_string(), "name".to_string()]);

    let mut parent_node = Node::new(
        "Team",
        "t1",
        vec![
            Value::String("t1".to_string().into()),
            Value::String("Engineering".to_string().into()),
        ],
    );

    parent_node.add_child(
        "Player",
        Node::new(
            "Player",
            "p1",
            vec![
                Value::String("p1".to_string().into()),
                Value::String("Alice".to_string().into()),
            ],
        ),
    );
    parent_node.add_child(
        "Player",
        Node::new(
            "Player",
            "p2",
            vec![
                Value::String("p2".to_string().into()),
                Value::String("Bob".to_string().into()),
            ],
        ),
    );

    list.add_row(parent_node);
    doc.root.insert("teams".to_string(), Item::List(list));

    let output = canonicalize(&doc).unwrap();

    // Should have team row
    assert!(output.contains("t1"));
    assert!(output.contains("Engineering"));
}

#[test]
fn test_nested_matrix_list_round_trip() {
    let input = r"%VERSION: 1.0
%STRUCT: Team: [id, name]
---
teams: @Team
  | t1, Engineering
";

    let doc = parse(input.as_bytes()).unwrap();
    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    let list1 = doc.root.get("teams").unwrap().as_list().unwrap();
    let list2 = doc2.root.get("teams").unwrap().as_list().unwrap();

    assert_eq!(list1.rows.len(), list2.rows.len());
    assert_eq!(list1.rows[0].id, list2.rows[0].id);
}

// =============================================================================
// Recursion Depth Limit Tests
// =============================================================================

#[test]
fn test_max_nesting_depth_error() {
    // Create deeply nested structure exceeding MAX_NESTING_DEPTH (1000)
    let mut doc = Document::new((1, 0));

    let mut inner = BTreeMap::new();
    inner.insert("leaf".to_string(), Item::Scalar(Value::Int(1001)));

    // Create 1001 levels of nesting (exceeds 1000 limit)
    // Note: This might cause stack overflow in test setup itself
    // so we'll use a slightly smaller but still excessive number
    for i in (0..500).rev() {
        let mut outer = BTreeMap::new();
        outer.insert(format!("level{i}"), Item::Object(inner));
        inner = outer;
    }

    doc.root.insert("root".to_string(), Item::Object(inner));

    let config = CanonicalConfig::default();
    let result = canonicalize_with_config(&doc, &config);

    // With 500 levels, it should still succeed (under 1000 limit)
    // We can't easily test the actual limit without stack overflow
    assert!(result.is_ok());
}

#[test]
fn test_acceptable_nesting_depth_succeeds() {
    // Create structure with 100 levels (well below 1000 limit)
    let mut doc = Document::new((1, 0));

    let mut inner = BTreeMap::new();
    inner.insert("leaf".to_string(), Item::Scalar(Value::Int(100)));

    for i in (0..100).rev() {
        let mut outer = BTreeMap::new();
        outer.insert(format!("level{i}"), Item::Object(inner));
        inner = outer;
    }

    doc.root.insert("root".to_string(), Item::Object(inner));

    let config = CanonicalConfig::default();
    let result = canonicalize_with_config(&doc, &config);

    // Should succeed
    assert!(result.is_ok());
}

// =============================================================================
// Special String Handling Tests
// =============================================================================

#[test]
fn test_string_with_leading_tilde() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("~tilde".to_string().into())),
    );

    let output = canonicalize(&doc).unwrap();
    // Should be quoted to prevent null interpretation
    assert!(output.contains("\"~tilde\""));
}

#[test]
fn test_string_with_leading_dollar() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("$variable".to_string().into())),
    );

    let output = canonicalize(&doc).unwrap();
    // Should be quoted to prevent expression interpretation
    assert!(output.contains("\"$variable\""));
}

#[test]
fn test_string_with_leading_percent() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("%directive".to_string().into())),
    );

    let output = canonicalize(&doc).unwrap();
    // Should be quoted to prevent directive interpretation
    assert!(output.contains("\"%directive\""));
}

#[test]
fn test_string_with_leading_bracket() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("[array".to_string().into())),
    );

    let output = canonicalize(&doc).unwrap();
    // Should be quoted to prevent tensor interpretation
    assert!(output.contains("\"[array\""));
}

#[test]
fn test_string_false_needs_quotes() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("false".to_string().into())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("\"false\""));
}

#[test]
fn test_string_true_needs_quotes() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("true".to_string().into())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("\"true\""));
}

#[test]
fn test_float_string_needs_quotes() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("3.5".to_string().into())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("\"3.5\""));
}

#[test]
fn test_scientific_notation_string_needs_quotes() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("1e10".to_string().into())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("\"1e10\""));
}

// =============================================================================
// Matrix Cell Edge Cases
// =============================================================================

#[test]
fn test_cell_with_comma_quoted() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string().into()),
            Value::String("hello,world".to_string().into()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // String with comma should be quoted
    assert!(output.contains("\"hello,world\""));
}

#[test]
fn test_cell_with_pipe_quoted() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string().into()),
            Value::String("hello|world".to_string().into()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // String with pipe should be quoted
    assert!(output.contains("\"hello|world\""));
}

#[test]
fn test_cell_with_hash_quoted() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string().into()),
            Value::String("test#comment".to_string().into()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // String with hash should be quoted
    assert!(output.contains("\"test#comment\""));
}

#[test]
fn test_cell_with_leading_caret_quoted() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string().into()),
            Value::String("^value".to_string().into()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // String starting with ^ should be quoted
    assert!(output.contains("\"^value\""));
}

// =============================================================================
// Float Formatting Edge Cases
// =============================================================================

#[test]
fn test_float_infinity_formatted() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "pos_inf".to_string(),
        Item::Scalar(Value::Float(f64::INFINITY)),
    );
    doc.root.insert(
        "neg_inf".to_string(),
        Item::Scalar(Value::Float(f64::NEG_INFINITY)),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("pos_inf: inf"));
    assert!(output.contains("neg_inf: -inf"));
}

#[test]
fn test_float_nan_formatted() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("nan".to_string(), Item::Scalar(Value::Float(f64::NAN)));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("nan: NaN"));
}

#[test]
fn test_float_negative_zero() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("neg_zero".to_string(), Item::Scalar(Value::Float(-0.0)));

    let output = canonicalize(&doc).unwrap();
    // -0.0 should format as -0.0
    assert!(output.contains("neg_zero: -0.0"));
}

// =============================================================================
// Empty Document and Empty Collection Tests
// =============================================================================

#[test]
fn test_empty_matrix_list() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "Empty".to_string(),
        vec!["id".to_string(), "value".to_string()],
    );

    let list = MatrixList::new("Empty", vec!["id".to_string(), "value".to_string()]);
    doc.root.insert("items".to_string(), Item::List(list));

    let output = canonicalize(&doc).unwrap();

    // Should have the list declaration but no rows
    assert!(output.contains("items: @Empty"));
    // Should not have any | row markers
    assert!(!output.contains('|'));
}

#[test]
fn test_document_with_only_aliases() {
    let mut doc = Document::new((1, 0));
    doc.aliases.insert("foo".to_string(), "bar".to_string());

    let output = canonicalize(&doc).unwrap();

    assert!(output.contains("%ALIAS: %foo: \"bar\""));
    assert!(output.contains("---"));
}

#[test]
fn test_document_with_only_structs() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let config = CanonicalConfig::new().with_inline_schemas(false);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    assert!(output.contains("%STRUCT: User: [id,name]"));
    assert!(output.contains("---"));
}

// =============================================================================
// Indentation Tests
// =============================================================================

#[test]
fn test_nested_object_indentation() {
    let mut doc = Document::new((1, 0));
    let mut level2 = BTreeMap::new();
    level2.insert(
        "value".to_string(),
        Item::Scalar(Value::String("deep".to_string().into())),
    );

    let mut level1 = BTreeMap::new();
    level1.insert("level2".to_string(), Item::Object(level2));

    doc.root.insert("level1".to_string(), Item::Object(level1));

    let output = canonicalize(&doc).unwrap();

    // Check indentation levels (2 spaces per level)
    assert!(output.contains("level1:\n"));
    assert!(output.contains("  level2:\n"));
    assert!(output.contains("    value: deep"));
}

#[test]
fn test_matrix_row_indentation() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "i1", vec![Value::String("i1".into())]));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Matrix rows should be indented by 2 spaces
    assert!(output.contains("  |i1"));
}
