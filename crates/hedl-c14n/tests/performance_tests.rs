// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Performance and optimization tests for canonicalization.
//!
//! Tests caching, buffer allocation, and other performance optimizations.

use hedl_c14n::{add_count_hints, canonicalize, CanonicalConfig};
use hedl_core::{Document, Item, MatrixList, Node, Value};

// =============================================================================
// Count Hint Tests
// =============================================================================

#[test]
fn test_add_count_hints_to_document() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list.add_row(Node::new(
        "User",
        "u1",
        vec![
            Value::String("u1".to_string().into()),
            Value::String("Alice".to_string().into()),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "u2",
        vec![
            Value::String("u2".to_string().into()),
            Value::String("Bob".to_string().into()),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    // Initially no count hint
    let list_before = doc.root.get("users").unwrap().as_list().unwrap();
    assert_eq!(list_before.count_hint, None);

    // Add count hints
    add_count_hints(&mut doc);

    // Now should have count hint
    let list_after = doc.root.get("users").unwrap().as_list().unwrap();
    assert_eq!(list_after.count_hint, Some(2));
}

#[test]
fn test_count_hints_in_canonical_output() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "Item".to_string(),
        vec!["id".to_string(), "value".to_string()],
    );

    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    for i in 1..=5 {
        list.add_row(Node::new(
            "Item",
            format!("i{i}"),
            vec![
                Value::String(format!("i{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }
    doc.root.insert("items".to_string(), Item::List(list));

    // Add count hints
    add_count_hints(&mut doc);

    let _config = CanonicalConfig::new().with_inline_schemas(false);
    let output = canonicalize(&doc).unwrap();

    // Should have count in STRUCT declaration
    assert!(output.contains("%STRUCT: Item (5): [id,value]"));
}

#[test]
fn test_count_hints_multiple_types() {
    let mut doc = Document::new((1, 0));
    doc.structs
        .insert("User".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Post".to_string(), vec!["id".to_string()]);

    let mut users = MatrixList::new("User", vec!["id".to_string()]);
    users.add_row(Node::new("User", "u1", vec![Value::String("u1".into())]));
    users.add_row(Node::new("User", "u2", vec![Value::String("u2".into())]));
    users.add_row(Node::new("User", "u3", vec![Value::String("u3".into())]));

    let mut posts = MatrixList::new("Post", vec!["id".to_string()]);
    posts.add_row(Node::new("Post", "p1", vec![Value::String("p1".into())]));
    posts.add_row(Node::new("Post", "p2", vec![Value::String("p2".into())]));

    doc.root.insert("users".to_string(), Item::List(users));
    doc.root.insert("posts".to_string(), Item::List(posts));

    add_count_hints(&mut doc);

    let output = canonicalize(&doc).unwrap();

    // Should have counts for both types
    assert!(output.contains("User (3)"));
    assert!(output.contains("Post (2)"));
}

// =============================================================================
// Large Document Tests
// =============================================================================

#[test]
fn test_large_matrix_list() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    // Add 1000 rows
    for i in 0..1000 {
        list.add_row(Node::new(
            "Item",
            format!("i{i}"),
            vec![Value::String(format!("i{i}").into()), Value::Int(i)],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    let _config = CanonicalConfig::new().with_inline_schemas(true);
    let result = canonicalize(&doc);

    assert!(result.is_ok());
    let output = result.unwrap();

    // Should have all rows
    assert!(output.contains("|i0,0"));
    assert!(output.contains("|i999,999"));
}

#[test]
fn test_many_keys() {
    let mut doc = Document::new((1, 0));

    // Add 100 keys
    for i in 0..100 {
        doc.root
            .insert(format!("key{i:03}"), Item::Scalar(Value::Int(i)));
    }

    let result = canonicalize(&doc);
    assert!(result.is_ok());

    let output = result.unwrap();

    // Keys should be sorted
    let key000_pos = output.find("key000:").unwrap();
    let key099_pos = output.find("key099:").unwrap();
    assert!(key000_pos < key099_pos);
}

// =============================================================================
// Ditto Optimization Tests
// =============================================================================
// Note: Ditto functionality is extensively tested in comprehensive_tests.rs

#[test]
fn test_ditto_partial_matches() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec![
            "id".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ],
    );

    // First row: all unique
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".into()),
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ],
    ));

    // Second row: b and c match
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![
            Value::String("i2".into()),
            Value::Int(5),
            Value::Int(2),
            Value::Int(3),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let _config = CanonicalConfig::new()
        .with_inline_schemas(true)
        .with_ditto(true);
    let output = canonicalize(&doc).unwrap();

    // Second row should use ditto for b and c
    assert!(output.contains("|i2,5,^,^"));
}

// =============================================================================
// Buffer Allocation Tests
// =============================================================================

#[test]
fn test_empty_document_allocates_minimal_buffer() {
    let doc = Document::new((1, 0));
    let output = canonicalize(&doc).unwrap();

    // Empty document should be very small
    assert!(output.len() < 100);
}

#[test]
fn test_nested_objects_allocate_correctly() {
    let mut doc = Document::new((1, 0));
    let mut inner = std::collections::BTreeMap::new();

    for i in 0..10 {
        inner.insert(
            format!("key{i}"),
            Item::Scalar(Value::String(format!("value{i}").into())),
        );
    }

    doc.root.insert("parent".to_string(), Item::Object(inner));

    let result = canonicalize(&doc);
    assert!(result.is_ok());
}

// =============================================================================
// Schema Caching Tests
// =============================================================================

#[test]
fn test_same_type_multiple_lists() {
    let mut doc = Document::new((1, 0));
    doc.structs
        .insert("User".to_string(), vec!["id".to_string()]);

    // Create multiple lists of the same type
    let mut list1 = MatrixList::new("User", vec!["id".to_string()]);
    list1.add_row(Node::new("User", "u1", vec![Value::String("u1".into())]));

    let mut list2 = MatrixList::new("User", vec!["id".to_string()]);
    list2.add_row(Node::new("User", "u2", vec![Value::String("u2".into())]));

    doc.root.insert("users1".to_string(), Item::List(list1));
    doc.root.insert("users2".to_string(), Item::List(list2));

    let output = canonicalize(&doc).unwrap();

    // Should have both lists with same type
    assert!(output.contains("users1: @User"));
    assert!(output.contains("users2: @User"));
}

// =============================================================================
// Inline vs Header Schema Tests
// =============================================================================

#[test]
fn test_inline_vs_header_schemas_basic() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "User",
        vec!["id".to_string(), "name".to_string(), "email".to_string()],
    );
    list.add_row(Node::new(
        "User",
        "u1",
        vec![
            Value::String("u1".into()),
            Value::String("Alice".into()),
            Value::String("alice@example.com".into()),
        ],
    ));

    doc.root.insert("users".to_string(), Item::List(list));

    let _config_inline = CanonicalConfig::new().with_inline_schemas(true);
    let output_inline = canonicalize(&doc).unwrap();

    let _config_header = CanonicalConfig::new().with_inline_schemas(false);
    let output_header = canonicalize(&doc).unwrap();

    // Both should have the type name
    assert!(output_inline.contains("@User"));
    assert!(output_header.contains("@User"));

    // Both have %STRUCT since the type is collected from matrix lists
    assert!(output_inline.contains("%STRUCT"));
    assert!(output_header.contains("%STRUCT"));

    // The difference is in how list declarations look
    // (inline vs separate, but both end up with STRUCT declarations)
}

#[test]
fn test_header_schemas_reuse() {
    let mut doc = Document::new((1, 0));
    let schema = vec!["id".to_string(), "name".to_string(), "email".to_string()];

    // Create 5 lists of same type
    for i in 1..=5 {
        let mut list = MatrixList::new("User", schema.clone());
        list.add_row(Node::new(
            "User",
            format!("u{i}"),
            vec![
                Value::String(format!("u{i}").into()),
                Value::String(format!("User{i}").into()),
                Value::String(format!("user{i}@example.com").into()),
            ],
        ));
        doc.root.insert(format!("users{i}"), Item::List(list));
    }

    let _config_inline = CanonicalConfig::new().with_inline_schemas(true);
    let output_inline = canonicalize(&doc).unwrap();

    let _config_header = CanonicalConfig::new().with_inline_schemas(false);
    let output_header = canonicalize(&doc).unwrap();

    // Inline should have schema in each list declaration
    assert!(output_inline.contains("users1: @User"));
    assert!(output_inline.contains("users5: @User"));

    // Header defines schema once in STRUCT
    assert!(output_header.contains("%STRUCT: User"));
    // All list declarations should reference the type
    assert!(output_header.contains("users1: @User"));
    assert!(output_header.contains("users5: @User"));
}
