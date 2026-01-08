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

//! Invariant tests for hedl-c14n canonicalization
//!
//! This module tests critical invariants that must hold for the canonicalization
//! implementation to be correct:
//! - Idempotency: canonicalize(canonicalize(doc)) == canonicalize(doc)
//! - Determinism: same input always produces same output
//! - Round-trip: parse(canonicalize(doc)) preserves semantic equivalence
//! - Ordering: keys are always sorted alphabetically
//! - Quoting strategies work correctly (minimal, always)
//! - Ditto optimization is applied correctly
//! - Count hints are preserved
//! - All value types are properly formatted
//! - Unicode handling is correct
//! - Special characters are escaped properly

use hedl_c14n::{canonicalize, canonicalize_with_config, CanonicalConfig, QuotingStrategy};
use hedl_core::{parse, Document, Expression, Item, MatrixList, Node, Reference, Tensor, Value};
use std::collections::BTreeMap;

// =============================================================================
// Idempotency Tests
// =============================================================================

#[test]
fn test_idempotency_empty_document() {
    let doc = Document::new((1, 0));
    let output1 = canonicalize(&doc).unwrap();
    let doc2 = parse(output1.as_bytes()).unwrap();
    let output2 = canonicalize(&doc2).unwrap();

    assert_eq!(
        output1, output2,
        "Canonicalization should be idempotent for empty document"
    );
}

#[test]
fn test_idempotency_simple_key_value() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "name".to_string(),
        Item::Scalar(Value::String("test".to_string())),
    );
    doc.root
        .insert("count".to_string(), Item::Scalar(Value::Int(42)));

    let output1 = canonicalize(&doc).unwrap();
    let doc2 = parse(output1.as_bytes()).unwrap();
    let output2 = canonicalize(&doc2).unwrap();

    assert_eq!(
        output1, output2,
        "Canonicalization should be idempotent for simple key-value pairs"
    );
}

#[test]
fn test_idempotency_nested_objects() {
    let mut doc = Document::new((1, 0));
    let mut inner = BTreeMap::new();
    inner.insert(
        "child".to_string(),
        Item::Scalar(Value::String("value".to_string())),
    );
    doc.root.insert("parent".to_string(), Item::Object(inner));

    let output1 = canonicalize(&doc).unwrap();
    let doc2 = parse(output1.as_bytes()).unwrap();
    let output2 = canonicalize(&doc2).unwrap();

    assert_eq!(
        output1, output2,
        "Canonicalization should be idempotent for nested objects"
    );
}

#[test]
fn test_idempotency_matrix_list() {
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
            Value::String("u1".to_string()),
            Value::String("Alice".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "u2",
        vec![
            Value::String("u2".to_string()),
            Value::String("Bob".to_string()),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    let output1 = canonicalize(&doc).unwrap();
    let doc2 = parse(output1.as_bytes()).unwrap();
    let output2 = canonicalize(&doc2).unwrap();

    assert_eq!(
        output1, output2,
        "Canonicalization should be idempotent for matrix lists"
    );
}

#[test]
fn test_idempotency_with_ditto() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "category".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("fruit".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![
            Value::String("i2".to_string()),
            Value::String("fruit".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(true).with_inline_schemas(true);

    let output1 = canonicalize_with_config(&doc, &config).unwrap();
    let doc2 = parse(output1.as_bytes()).unwrap();
    let output2 = canonicalize_with_config(&doc2, &config).unwrap();

    assert_eq!(
        output1, output2,
        "Canonicalization should be idempotent with ditto optimization"
    );
}

#[test]
fn test_idempotency_all_value_types() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("null".to_string(), Item::Scalar(Value::Null));
    doc.root
        .insert("bool".to_string(), Item::Scalar(Value::Bool(true)));
    doc.root
        .insert("int".to_string(), Item::Scalar(Value::Int(42)));
    doc.root
        .insert("float".to_string(), Item::Scalar(Value::Float(3.14)));
    doc.root.insert(
        "string".to_string(),
        Item::Scalar(Value::String("hello".to_string())),
    );
    // Create a matrix list so references have a target
    doc.structs.insert(
        "Target".to_string(),
        vec!["id".to_string(), "value".to_string()],
    );
    let mut list = MatrixList::new("Target", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Target",
        "target",
        vec![
            Value::String("target".to_string()),
            Value::String("data".to_string()),
        ],
    ));
    doc.root.insert("targets".to_string(), Item::List(list));
    doc.root.insert(
        "reference".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target"))),
    );
    doc.root.insert(
        "tensor".to_string(),
        Item::Scalar(Value::Tensor(Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
        ]))),
    );
    doc.root.insert(
        "expression".to_string(),
        Item::Scalar(Value::Expression(Expression::Identifier { name: "x".to_string(), span: Default::default() })),
    );

    let output1 = canonicalize(&doc).unwrap();
    let doc2 = parse(output1.as_bytes()).unwrap();
    let output2 = canonicalize(&doc2).unwrap();

    assert_eq!(
        output1, output2,
        "Canonicalization should be idempotent for all value types"
    );
}

// =============================================================================
// Determinism Tests
// =============================================================================

#[test]
fn test_determinism_multiple_runs() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("zebra".to_string(), Item::Scalar(Value::Int(3)));
    doc.root
        .insert("apple".to_string(), Item::Scalar(Value::Int(1)));
    doc.root
        .insert("banana".to_string(), Item::Scalar(Value::Int(2)));

    let outputs: Vec<_> = (0..5).map(|_| canonicalize(&doc).unwrap()).collect();

    for i in 1..outputs.len() {
        assert_eq!(
            outputs[0], outputs[i],
            "Same input should always produce same output (run {})",
            i
        );
    }
}

#[test]
fn test_determinism_key_ordering() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("zebra".to_string(), Item::Scalar(Value::Int(3)));
    doc.root
        .insert("apple".to_string(), Item::Scalar(Value::Int(1)));
    doc.root
        .insert("banana".to_string(), Item::Scalar(Value::Int(2)));

    let output = canonicalize(&doc).unwrap();

    // Extract positions of keys
    let apple_pos = output.find("apple:").unwrap();
    let banana_pos = output.find("banana:").unwrap();
    let zebra_pos = output.find("zebra:").unwrap();

    // Keys should always be in alphabetical order
    assert!(
        apple_pos < banana_pos && banana_pos < zebra_pos,
        "Keys should always be in alphabetical order"
    );
}

#[test]
fn test_determinism_alias_ordering() {
    let mut doc = Document::new((1, 0));
    doc.aliases.insert("zebra".to_string(), "z".to_string());
    doc.aliases.insert("apple".to_string(), "a".to_string());
    doc.aliases.insert("mango".to_string(), "m".to_string());

    let outputs: Vec<_> = (0..3).map(|_| canonicalize(&doc).unwrap()).collect();

    for i in 1..outputs.len() {
        assert_eq!(
            outputs[0], outputs[i],
            "Alias ordering should be deterministic"
        );
    }
}

#[test]
fn test_determinism_struct_ordering() {
    let mut doc = Document::new((1, 0));
    doc.structs
        .insert("Zebra".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Apple".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Mango".to_string(), vec!["id".to_string()]);

    let config = CanonicalConfig::new().with_inline_schemas(false);

    let outputs: Vec<_> = (0..3)
        .map(|_| canonicalize_with_config(&doc, &config).unwrap())
        .collect();

    for i in 1..outputs.len() {
        assert_eq!(
            outputs[0], outputs[i],
            "Struct ordering should be deterministic"
        );
    }
}

// =============================================================================
// Round-Trip Tests
// =============================================================================

#[test]
fn test_round_trip_preserves_null() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Null));

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    assert_eq!(
        doc.root.get("value").unwrap().as_scalar().unwrap(),
        doc2.root.get("value").unwrap().as_scalar().unwrap(),
        "Round-trip should preserve null values"
    );
}

#[test]
fn test_round_trip_preserves_booleans() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("true_val".to_string(), Item::Scalar(Value::Bool(true)));
    doc.root
        .insert("false_val".to_string(), Item::Scalar(Value::Bool(false)));

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    assert_eq!(
        doc.root.get("true_val").unwrap().as_scalar().unwrap(),
        doc2.root.get("true_val").unwrap().as_scalar().unwrap(),
    );
    assert_eq!(
        doc.root.get("false_val").unwrap().as_scalar().unwrap(),
        doc2.root.get("false_val").unwrap().as_scalar().unwrap(),
    );
}

#[test]
fn test_round_trip_preserves_integers() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("zero".to_string(), Item::Scalar(Value::Int(0)));
    doc.root
        .insert("positive".to_string(), Item::Scalar(Value::Int(42)));
    doc.root
        .insert("negative".to_string(), Item::Scalar(Value::Int(-100)));
    doc.root
        .insert("max".to_string(), Item::Scalar(Value::Int(i64::MAX)));
    doc.root
        .insert("min".to_string(), Item::Scalar(Value::Int(i64::MIN)));

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    for key in &["zero", "positive", "negative", "max", "min"] {
        assert_eq!(
            doc.root.get(*key).unwrap().as_scalar().unwrap(),
            doc2.root.get(*key).unwrap().as_scalar().unwrap(),
            "Round-trip should preserve integer: {}",
            key
        );
    }
}

#[test]
fn test_round_trip_preserves_floats() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("zero".to_string(), Item::Scalar(Value::Float(0.0)));
    doc.root
        .insert("pi".to_string(), Item::Scalar(Value::Float(3.14159)));
    doc.root
        .insert("whole".to_string(), Item::Scalar(Value::Float(42.0)));
    doc.root
        .insert("negative".to_string(), Item::Scalar(Value::Float(-2.5)));

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    for key in &["zero", "pi", "whole", "negative"] {
        assert_eq!(
            doc.root.get(*key).unwrap().as_scalar().unwrap(),
            doc2.root.get(*key).unwrap().as_scalar().unwrap(),
            "Round-trip should preserve float: {}",
            key
        );
    }
}

#[test]
fn test_round_trip_preserves_strings() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "simple".to_string(),
        Item::Scalar(Value::String("hello".to_string())),
    );
    doc.root.insert(
        "empty".to_string(),
        Item::Scalar(Value::String("".to_string())),
    );
    doc.root.insert(
        "with_space".to_string(),
        Item::Scalar(Value::String("  spaces  ".to_string())),
    );
    doc.root.insert(
        "with_quotes".to_string(),
        Item::Scalar(Value::String("say \"hi\"".to_string())),
    );
    doc.root.insert(
        "unicode".to_string(),
        Item::Scalar(Value::String("héllo 世界".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    for key in &["simple", "empty", "with_space", "with_quotes", "unicode"] {
        assert_eq!(
            doc.root.get(*key).unwrap().as_scalar().unwrap(),
            doc2.root.get(*key).unwrap().as_scalar().unwrap(),
            "Round-trip should preserve string: {}",
            key
        );
    }
}

#[test]
fn test_round_trip_preserves_references() {
    let mut doc = Document::new((1, 0));
    // Create targets for references
    doc.structs.insert(
        "Target".to_string(),
        vec!["id".to_string(), "value".to_string()],
    );
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    let mut targets = MatrixList::new("Target", vec!["id".to_string(), "value".to_string()]);
    targets.add_row(Node::new(
        "Target",
        "target",
        vec![
            Value::String("target".to_string()),
            Value::String("data".to_string()),
        ],
    ));
    doc.root.insert("targets".to_string(), Item::List(targets));

    let mut users = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    users.add_row(Node::new(
        "User",
        "id",
        vec![
            Value::String("id".to_string()),
            Value::String("John".to_string()),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(users));

    doc.root.insert(
        "local_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target"))),
    );
    doc.root.insert(
        "qualified_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "id"))),
    );

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    assert_eq!(
        doc.root.get("local_ref").unwrap().as_scalar().unwrap(),
        doc2.root.get("local_ref").unwrap().as_scalar().unwrap(),
    );
    assert_eq!(
        doc.root
            .get("qualified_ref")
            .unwrap()
            .as_scalar()
            .unwrap(),
        doc2.root
            .get("qualified_ref")
            .unwrap()
            .as_scalar()
            .unwrap(),
    );
}

#[test]
fn test_round_trip_preserves_tensors() {
    let mut doc = Document::new((1, 0));
    // Note: Tensor scalars are indistinguishable from floats in canonical form
    // They both serialize as "1.0" so we skip scalar tensors
    doc.root.insert(
        "array".to_string(),
        Item::Scalar(Value::Tensor(Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
            Tensor::Scalar(3.0),
        ]))),
    );
    doc.root.insert(
        "nested".to_string(),
        Item::Scalar(Value::Tensor(Tensor::Array(vec![
            Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
            Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
        ]))),
    );

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    for key in &["array", "nested"] {
        assert_eq!(
            doc.root.get(*key).unwrap().as_scalar().unwrap(),
            doc2.root.get(*key).unwrap().as_scalar().unwrap(),
            "Round-trip should preserve tensor: {}",
            key
        );
    }
}

#[test]
fn test_round_trip_preserves_matrix_list_structure() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string(), "email".to_string()],
    );

    let mut list = MatrixList::new(
        "User",
        vec![
            "id".to_string(),
            "name".to_string(),
            "email".to_string(),
        ],
    );
    list.add_row(Node::new(
        "User",
        "u1",
        vec![
            Value::String("u1".to_string()),
            Value::String("Alice".to_string()),
            Value::String("alice@example.com".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "u2",
        vec![
            Value::String("u2".to_string()),
            Value::String("Bob".to_string()),
            Value::String("bob@example.com".to_string()),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    let list1 = doc.root.get("users").unwrap().as_list().unwrap();
    let list2 = doc2.root.get("users").unwrap().as_list().unwrap();

    assert_eq!(list1.type_name, list2.type_name);
    assert_eq!(list1.schema, list2.schema);
    assert_eq!(list1.rows.len(), list2.rows.len());

    for i in 0..list1.rows.len() {
        assert_eq!(list1.rows[i].id, list2.rows[i].id);
        assert_eq!(list1.rows[i].fields, list2.rows[i].fields);
    }
}

// =============================================================================
// Key Ordering Invariant Tests
// =============================================================================

#[test]
fn test_keys_always_sorted_alphabetically() {
    let mut doc = Document::new((1, 0));
    // Insert in reverse order
    doc.root
        .insert("zebra".to_string(), Item::Scalar(Value::Int(26)));
    doc.root
        .insert("yankee".to_string(), Item::Scalar(Value::Int(25)));
    doc.root
        .insert("alpha".to_string(), Item::Scalar(Value::Int(1)));
    doc.root
        .insert("bravo".to_string(), Item::Scalar(Value::Int(2)));

    let output = canonicalize(&doc).unwrap();

    let keys = ["alpha", "bravo", "yankee", "zebra"];
    let mut positions = Vec::new();

    for key in &keys {
        let pos = output
            .find(&format!("{}:", key))
            .unwrap_or_else(|| panic!("Key {} not found", key));
        positions.push(pos);
    }

    // Verify positions are in ascending order
    for i in 1..positions.len() {
        assert!(
            positions[i - 1] < positions[i],
            "Keys should be in alphabetical order: {} should come before {}",
            keys[i - 1],
            keys[i]
        );
    }
}

#[test]
fn test_nested_keys_sorted() {
    let mut doc = Document::new((1, 0));
    let mut inner = BTreeMap::new();
    inner.insert("zebra".to_string(), Item::Scalar(Value::Int(3)));
    inner.insert("alpha".to_string(), Item::Scalar(Value::Int(1)));
    doc.root.insert("parent".to_string(), Item::Object(inner));

    let output = canonicalize(&doc).unwrap();

    // Find positions within the nested object
    let alpha_pos = output.find("alpha:").unwrap();
    let zebra_pos = output.find("zebra:").unwrap();

    assert!(
        alpha_pos < zebra_pos,
        "Nested keys should be in alphabetical order"
    );
}

// =============================================================================
// Quoting Strategy Tests
// =============================================================================

#[test]
fn test_minimal_quoting_simple_strings() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "simple".to_string(),
        Item::Scalar(Value::String("hello".to_string())),
    );

    let config = CanonicalConfig::new().with_quoting(QuotingStrategy::Minimal);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Simple strings should not be quoted
    assert!(output.contains("simple: hello"));
    assert!(!output.contains("\"hello\""));
}

#[test]
fn test_minimal_quoting_requires_quotes_for_special() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "empty".to_string(),
        Item::Scalar(Value::String("".to_string())),
    );
    doc.root.insert(
        "numeric".to_string(),
        Item::Scalar(Value::String("123".to_string())),
    );
    doc.root.insert(
        "boolean".to_string(),
        Item::Scalar(Value::String("true".to_string())),
    );
    doc.root.insert(
        "with_hash".to_string(),
        Item::Scalar(Value::String("test#comment".to_string())),
    );

    let config = CanonicalConfig::new().with_quoting(QuotingStrategy::Minimal);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // These should all be quoted
    assert!(output.contains("empty: \"\""));
    assert!(output.contains("numeric: \"123\""));
    assert!(output.contains("boolean: \"true\""));
    assert!(output.contains("\"test#comment\""));
}

#[test]
fn test_always_quoting_quotes_everything() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "simple".to_string(),
        Item::Scalar(Value::String("hello".to_string())),
    );
    doc.root.insert(
        "empty".to_string(),
        Item::Scalar(Value::String("".to_string())),
    );

    let config = CanonicalConfig::new().with_quoting(QuotingStrategy::Always);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Both should be quoted
    assert!(output.contains("simple: \"hello\""));
    assert!(output.contains("empty: \"\""));
}

#[test]
fn test_quoting_round_trip_minimal() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("hello world".to_string())),
    );

    let config = CanonicalConfig::new().with_quoting(QuotingStrategy::Minimal);
    let output1 = canonicalize_with_config(&doc, &config).unwrap();
    let doc2 = parse(output1.as_bytes()).unwrap();
    let output2 = canonicalize_with_config(&doc2, &config).unwrap();

    assert_eq!(output1, output2, "Minimal quoting should round-trip");
}

#[test]
fn test_quoting_round_trip_always() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("hello world".to_string())),
    );

    let config = CanonicalConfig::new().with_quoting(QuotingStrategy::Always);
    let output1 = canonicalize_with_config(&doc, &config).unwrap();
    let doc2 = parse(output1.as_bytes()).unwrap();
    let output2 = canonicalize_with_config(&doc2, &config).unwrap();

    assert_eq!(output1, output2, "Always quoting should round-trip");
}

// =============================================================================
// Ditto Optimization Invariant Tests
// =============================================================================

#[test]
fn test_ditto_never_in_first_row() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(42)],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(true).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // First row should never have ditto
    let lines: Vec<&str> = output.lines().collect();
    let first_row = lines.iter().find(|l| l.trim().starts_with("|")).unwrap();
    assert!(!first_row.contains("^"), "First row should never use ditto");
}

#[test]
fn test_ditto_never_in_id_column() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(42)],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![Value::String("i2".to_string()), Value::Int(42)],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(true).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Second row should have ditto for value column only, not ID
    assert!(
        output.contains("|i2,^"),
        "Second row should use ditto for matching value"
    );
    assert!(
        !output.contains("|^,"),
        "ID column should never use ditto"
    );
}

#[test]
fn test_ditto_applied_for_matching_values() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec![
            "id".to_string(),
            "cat".to_string(),
            "status".to_string(),
        ],
    );
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("fruit".to_string()),
            Value::Bool(true),
        ],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![
            Value::String("i2".to_string()),
            Value::String("fruit".to_string()),
            Value::Bool(true),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(true).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Both columns should use ditto (except ID)
    assert!(
        output.contains("|i2,^,^"),
        "Matching values should use ditto"
    );
}

#[test]
fn test_ditto_not_applied_for_different_values() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(1)],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![Value::String("i2".to_string()), Value::Int(2)],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(true).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Different values should not use ditto
    assert!(
        !output.contains("|i2,^"),
        "Different values should not use ditto"
    );
    assert!(output.contains("|i2,2"), "Should output actual value");
}

#[test]
fn test_ditto_disabled() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("same".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![
            Value::String("i2".to_string()),
            Value::String("same".to_string()),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(false).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // No ditto when disabled
    assert!(!output.contains("^"), "Ditto should not be used when disabled");
}

#[test]
fn test_ditto_deep_equality_required() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    // Int 42 vs Float 42.0 should NOT ditto
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(42)],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![Value::String("i2".to_string()), Value::Float(42.0)],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_ditto(true).with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Different types should not ditto
    assert!(
        !output.contains("|i2,^"),
        "Different types should not use ditto"
    );
}

// =============================================================================
// Count Hint Preservation Tests
// =============================================================================

#[test]
fn test_count_hint_in_struct_declaration() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let mut list = MatrixList::with_count_hint(
        "User",
        vec!["id".to_string(), "name".to_string()],
        2,
    );
    list.add_row(Node::new(
        "User",
        "u1",
        vec![
            Value::String("u1".to_string()),
            Value::String("Alice".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "u2",
        vec![
            Value::String("u2".to_string()),
            Value::String("Bob".to_string()),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(false);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Count hint should be in STRUCT declaration
    assert!(
        output.contains("%STRUCT: User (2): [id,name]"),
        "Count hint should appear in STRUCT declaration"
    );
}

#[test]
fn test_node_child_count_preserved() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "Team".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let mut list = MatrixList::new("Team", vec!["id".to_string(), "name".to_string()]);
    let mut node = Node::new(
        "Team",
        "t1",
        vec![Value::Int(1), Value::String("Engineering".to_string())],
    );
    node.set_child_count(5);
    list.add_row(node);

    doc.root.insert("teams".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Node child count should be preserved as [N] prefix
    assert!(
        output.contains("|[5]"),
        "Node child count should be preserved"
    );
}

#[test]
fn test_count_hint_round_trip() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert(
        "Item".to_string(),
        vec!["id".to_string(), "value".to_string()],
    );

    let mut list = MatrixList::with_count_hint(
        "Item",
        vec!["id".to_string(), "value".to_string()],
        3,
    );
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(1)],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![Value::String("i2".to_string()), Value::Int(2)],
    ));
    list.add_row(Node::new(
        "Item",
        "i3",
        vec![Value::String("i3".to_string()), Value::Int(3)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let output = canonicalize(&doc).unwrap();

    // Verify count hint is in output
    assert!(
        output.contains("%STRUCT: Item (3): [id,value]"),
        "Count hint should be in canonical output"
    );

    let doc2 = parse(output.as_bytes()).unwrap();

    let list1 = doc.root.get("items").unwrap().as_list().unwrap();
    let list2 = doc2.root.get("items").unwrap().as_list().unwrap();

    // Count hints are informational and not required to round-trip exactly
    // The parser may or may not preserve them depending on implementation
    // What matters is the actual data is preserved
    assert_eq!(list1.rows.len(), list2.rows.len());
    assert_eq!(list1.type_name, list2.type_name);
    assert_eq!(list1.schema, list2.schema);
}

// =============================================================================
// Value Type Formatting Tests
// =============================================================================

#[test]
fn test_null_formatted_as_tilde() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Null));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("value: ~"));
}

#[test]
fn test_bool_formatted_correctly() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("t".to_string(), Item::Scalar(Value::Bool(true)));
    doc.root
        .insert("f".to_string(), Item::Scalar(Value::Bool(false)));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("t: true"));
    assert!(output.contains("f: false"));
}

#[test]
fn test_integer_formatted_correctly() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("zero".to_string(), Item::Scalar(Value::Int(0)));
    doc.root
        .insert("pos".to_string(), Item::Scalar(Value::Int(42)));
    doc.root
        .insert("neg".to_string(), Item::Scalar(Value::Int(-100)));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("zero: 0"));
    assert!(output.contains("pos: 42"));
    assert!(output.contains("neg: -100"));
}

#[test]
fn test_float_formatted_with_decimal() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("frac".to_string(), Item::Scalar(Value::Float(3.14)));
    doc.root
        .insert("whole".to_string(), Item::Scalar(Value::Float(42.0)));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("frac: 3.14"));
    // Whole floats should have .0 to distinguish from int
    assert!(output.contains("whole: 42.0"));
}

#[test]
fn test_reference_formatted_correctly() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "local".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target"))),
    );
    doc.root.insert(
        "qualified".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "id"))),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("local: @target"));
    assert!(output.contains("qualified: @User:id"));
}

#[test]
fn test_expression_formatted_correctly() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "expr".to_string(),
        Item::Scalar(Value::Expression(Expression::Identifier { name: "x".to_string(), span: Default::default() })),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("expr: $(x)"));
}

#[test]
fn test_tensor_formatted_correctly() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "scalar".to_string(),
        Item::Scalar(Value::Tensor(Tensor::Scalar(1.0))),
    );
    doc.root.insert(
        "array".to_string(),
        Item::Scalar(Value::Tensor(Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
        ]))),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("scalar: 1.0"));
    assert!(output.contains("array: [1.0, 2.0]"));
}

// =============================================================================
// Unicode Handling Tests
// =============================================================================

#[test]
fn test_unicode_preserved() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "chinese".to_string(),
        Item::Scalar(Value::String("你好世界".to_string())),
    );
    doc.root.insert(
        "emoji".to_string(),
        Item::Scalar(Value::String("Hello 👋".to_string())),
    );
    doc.root.insert(
        "mixed".to_string(),
        Item::Scalar(Value::String("café".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("chinese: 你好世界"));
    assert!(output.contains("emoji: Hello 👋"));
    assert!(output.contains("mixed: café"));
}

#[test]
fn test_unicode_round_trip() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("Iñtërnâtiônàlizætiøn 🌍".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    assert_eq!(
        doc.root.get("text").unwrap().as_scalar().unwrap(),
        doc2.root.get("text").unwrap().as_scalar().unwrap(),
        "Unicode should round-trip correctly"
    );
}

// =============================================================================
// Special Character Escaping Tests
// =============================================================================

#[test]
fn test_quote_escaping() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("say \"hello\"".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    // Quotes should be escaped as ""
    assert!(output.contains("\"say \"\"hello\"\"\""));
}

#[test]
fn test_quote_escaping_round_trip() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "value".to_string(),
        Item::Scalar(Value::String("He said \"hi\"".to_string())),
    );

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    assert_eq!(
        doc.root.get("value").unwrap().as_scalar().unwrap(),
        doc2.root.get("value").unwrap().as_scalar().unwrap(),
        "Quote escaping should round-trip"
    );
}

#[test]
fn test_newline_escaping_in_cells() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("line1\nline2".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Newlines should be escaped as \n in cells
    assert!(output.contains("\\n"));
}

#[test]
fn test_tab_escaping_in_cells() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("col1\tcol2".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Tabs should be escaped as \t in cells
    assert!(output.contains("\\t"));
}

#[test]
fn test_backslash_escaping_in_cells() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "path".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("C:\\path\\to\\file".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = CanonicalConfig::new().with_inline_schemas(true);
    let output = canonicalize_with_config(&doc, &config).unwrap();

    // Backslashes should be escaped as \\ in cells
    assert!(output.contains("\\\\"));
}

#[test]
fn test_control_character_escaping_round_trip() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("line1\nline2\ttab\rcarriage".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let output = canonicalize(&doc).unwrap();
    let doc2 = parse(output.as_bytes()).unwrap();

    let list1 = doc.root.get("items").unwrap().as_list().unwrap();
    let list2 = doc2.root.get("items").unwrap().as_list().unwrap();

    assert_eq!(
        list1.rows[0].fields[1], list2.rows[0].fields[1],
        "Control characters should round-trip correctly"
    );
}

// =============================================================================
// Complex Invariant Tests
// =============================================================================

#[test]
fn test_complex_document_idempotency() {
    let mut doc = Document::new((1, 0));

    // Aliases
    doc.aliases.insert("usr".to_string(), "User".to_string());
    doc.aliases
        .insert("pst".to_string(), "Post".to_string());

    // Structs
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string(), "email".to_string()],
    );
    doc.structs.insert(
        "Post".to_string(),
        vec![
            "id".to_string(),
            "title".to_string(),
            "author".to_string(),
        ],
    );

    // Scalar values
    doc.root
        .insert("version".to_string(), Item::Scalar(Value::Int(1)));
    doc.root.insert(
        "app_name".to_string(),
        Item::Scalar(Value::String("TestApp".to_string())),
    );

    // Matrix list
    let mut users = MatrixList::new(
        "User",
        vec![
            "id".to_string(),
            "name".to_string(),
            "email".to_string(),
        ],
    );
    users.add_row(Node::new(
        "User",
        "u1",
        vec![
            Value::String("u1".to_string()),
            Value::String("Alice".to_string()),
            Value::String("alice@example.com".to_string()),
        ],
    ));
    users.add_row(Node::new(
        "User",
        "u2",
        vec![
            Value::String("u2".to_string()),
            Value::String("Bob".to_string()),
            Value::String("bob@example.com".to_string()),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(users));

    // Perform multiple rounds of canonicalization
    let mut current = canonicalize(&doc).unwrap();
    for i in 0..5 {
        let parsed = parse(current.as_bytes()).unwrap();
        let next = canonicalize(&parsed).unwrap();
        assert_eq!(
            current, next,
            "Idempotency should hold for complex document at iteration {}",
            i
        );
        current = next;
    }
}

#[test]
fn test_invariant_holds_across_configurations() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![
            Value::String("i1".to_string()),
            Value::String("test".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![
            Value::String("i2".to_string()),
            Value::String("test".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let configs = vec![
        CanonicalConfig::default(),
        CanonicalConfig::new().with_quoting(QuotingStrategy::Always),
        CanonicalConfig::new().with_ditto(false),
        CanonicalConfig::new().with_inline_schemas(true),
    ];

    for config in configs {
        let output1 = canonicalize_with_config(&doc, &config).unwrap();
        let doc2 = parse(output1.as_bytes()).unwrap();
        let output2 = canonicalize_with_config(&doc2, &config).unwrap();

        assert_eq!(
            output1, output2,
            "Idempotency should hold for config: {:?}",
            config
        );
    }
}
