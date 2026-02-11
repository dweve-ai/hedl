// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the root of this repository
// or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for Value::List canonicalization support (HEDL v1.1).
//!
//! These tests verify that the new `Value::List` variant (introduced in HEDL v1.1)
//! is properly formatted in canonical output as `(elem1, elem2, elem3)`.

use hedl_c14n::{canonicalize, CanonicalConfig};
use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use std::collections::BTreeMap;

#[test]
fn test_list_in_key_value() {
    let mut doc = Document::new((1, 1));

    // String list
    let roles = Value::List(Box::new(vec![
        Value::String("admin".to_string().into()),
        Value::String("editor".to_string().into()),
        Value::String("viewer".to_string().into()),
    ]));
    doc.root.insert("roles".to_string(), Item::Scalar(roles));

    // Boolean list
    let flags = Value::List(Box::new(vec![
        Value::Bool(true),
        Value::Bool(false),
        Value::Bool(true),
    ]));
    doc.root.insert("flags".to_string(), Item::Scalar(flags));

    // Integer list
    let numbers = Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    doc.root
        .insert("numbers".to_string(), Item::Scalar(numbers));

    let output = canonicalize(&doc).unwrap();

    assert!(output.contains("flags: (true, false, true)"));
    assert!(output.contains("numbers: (1, 2, 3)"));
    assert!(output.contains("roles: (admin, editor, viewer)"));
}

#[test]
fn test_empty_list() {
    let mut doc = Document::new((1, 1));
    let empty = Value::List(Box::default());
    doc.root.insert("empty".to_string(), Item::Scalar(empty));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("empty: ()"));
}

#[test]
fn test_list_with_mixed_types() {
    let mut doc = Document::new((1, 1));
    let mixed = Value::List(Box::new(vec![
        Value::Int(42),
        Value::String("hello".to_string().into()),
        Value::Bool(true),
        Value::Null,
    ]));
    doc.root.insert("mixed".to_string(), Item::Scalar(mixed));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("mixed: (42, hello, true, ~)"));
}

#[test]
fn test_list_with_references() {
    let mut doc = Document::new((1, 1));
    let refs = Value::List(Box::new(vec![
        Value::Reference(Reference::local("user1")),
        Value::Reference(Reference::qualified("User", "123")),
        Value::Reference(Reference::local("user2")),
    ]));
    doc.root.insert("refs".to_string(), Item::Scalar(refs));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("refs: (@user1, @User:123, @user2)"));
}

#[test]
fn test_nested_lists() {
    let mut doc = Document::new((1, 1));
    let inner1 = Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]));
    let inner2 = Value::List(Box::new(vec![Value::Int(3), Value::Int(4)]));
    let outer = Value::List(Box::new(vec![inner1, inner2]));
    doc.root.insert("nested".to_string(), Item::Scalar(outer));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("nested: ((1, 2), (3, 4))"));
}

#[test]
fn test_list_in_matrix_cell() {
    let mut doc = Document::new((1, 1));
    doc.structs.insert(
        "Config".to_string(),
        vec!["id".to_string(), "tags".to_string()],
    );

    let mut list = MatrixList::new("Config", vec!["id".to_string(), "tags".to_string()]);

    // Add row with list in second column
    let tags = Value::List(Box::new(vec![
        Value::String("prod".to_string().into()),
        Value::String("critical".to_string().into()),
    ]));
    list.add_row(Node::new("Config", "cfg1", vec![Value::Int(1), tags]));

    doc.root.insert("configs".to_string(), Item::List(list));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("|1,(prod, critical)"));
}

#[test]
fn test_list_roundtrip() {
    let mut doc = Document::new((1, 1));
    let list = Value::List(Box::new(vec![
        Value::String("a".to_string().into()),
        Value::String("b".to_string().into()),
        Value::String("c".to_string().into()),
    ]));
    doc.root.insert("items".to_string(), Item::Scalar(list));

    let canonical = canonicalize(&doc).unwrap();

    // Parse back and re-canonicalize
    let parsed = hedl_core::parse(canonical.as_bytes()).unwrap();
    let canonical2 = canonicalize(&parsed).unwrap();

    // Should be identical (idempotent)
    assert_eq!(canonical, canonical2);
}

#[test]
fn test_list_with_always_quoting() {
    let mut doc = Document::new((1, 1));
    let list = Value::List(Box::new(vec![
        Value::String("hello".to_string().into()),
        Value::String("world".to_string().into()),
    ]));
    doc.root.insert("words".to_string(), Item::Scalar(list));

    let config = CanonicalConfig::new().with_quoting(hedl_c14n::QuotingStrategy::Always);
    let output = hedl_c14n::canonicalize_with_config(&doc, &config).unwrap();

    assert!(output.contains("words: (\"hello\", \"world\")"));
}

#[test]
fn test_list_with_floats() {
    let mut doc = Document::new((1, 1));
    let list = Value::List(Box::new(vec![
        Value::Float(1.5),
        Value::Float(2.0),
        Value::Float(4.56),
    ]));
    doc.root.insert("floats".to_string(), Item::Scalar(list));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("floats: (1.5, 2.0, 4.56)"));
}

#[test]
fn test_list_single_element() {
    let mut doc = Document::new((1, 1));
    let list = Value::List(Box::new(vec![Value::String("solo".to_string().into())]));
    doc.root.insert("single".to_string(), Item::Scalar(list));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("single: (solo)"));
}

#[test]
fn test_list_in_nested_object() {
    let mut doc = Document::new((1, 1));

    let mut inner = BTreeMap::new();
    let permissions = Value::List(Box::new(vec![
        Value::String("read".to_string().into()),
        Value::String("write".to_string().into()),
        Value::String("delete".to_string().into()),
    ]));
    inner.insert("permissions".to_string(), Item::Scalar(permissions));

    doc.root.insert("security".to_string(), Item::Object(inner));

    let output = canonicalize(&doc).unwrap();
    assert!(output.contains("permissions: (read, write, delete)"));
}

#[test]
fn test_multiple_lists_in_document() {
    let mut doc = Document::new((1, 1));

    let roles = Value::List(Box::new(vec![
        Value::String("admin".to_string().into()),
        Value::String("user".to_string().into()),
    ]));
    doc.root.insert("roles".to_string(), Item::Scalar(roles));

    let statuses = Value::List(Box::new(vec![
        Value::String("active".to_string().into()),
        Value::String("inactive".to_string().into()),
        Value::String("pending".to_string().into()),
    ]));
    doc.root
        .insert("statuses".to_string(), Item::Scalar(statuses));

    let priorities = Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    doc.root
        .insert("priorities".to_string(), Item::Scalar(priorities));

    let output = canonicalize(&doc).unwrap();

    assert!(output.contains("priorities: (1, 2, 3)"));
    assert!(output.contains("roles: (admin, user)"));
    assert!(output.contains("statuses: (active, inactive, pending)"));
}

#[test]
fn test_list_preserves_order() {
    let mut doc = Document::new((1, 1));
    let list = Value::List(Box::new(vec![
        Value::String("zebra".to_string().into()),
        Value::String("apple".to_string().into()),
        Value::String("mango".to_string().into()),
    ]));
    doc.root.insert("unsorted".to_string(), Item::Scalar(list));

    let output = canonicalize(&doc).unwrap();
    // Should preserve original order, not sort
    assert!(output.contains("unsorted: (zebra, apple, mango)"));
}
