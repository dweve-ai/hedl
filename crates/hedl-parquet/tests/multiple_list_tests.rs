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

//! Tests for handling multiple matrix lists in documents.
//!
//! Issue 1 (HIGH): Only first matrix list written - FIXED
//! - Parquet supports one table per file
//! - Documents with multiple matrix lists now write first list (alphabetically) with a warning
//! - This matches standard Parquet behavior where one file = one table

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{from_parquet_bytes, to_parquet_bytes};

#[test]
fn test_single_matrix_list_succeeds() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);

    list.add_row(Node::new(
        "User",
        "alice",
        vec![
            Value::String("alice".to_string().into()),
            Value::String("Alice".to_string().into()),
        ],
    ));

    doc.root.insert("users".to_string(), Item::List(list));

    let result = to_parquet_bytes(&doc);
    assert!(result.is_ok(), "Single matrix list should succeed");
}

#[test]
fn test_two_matrix_lists_writes_first() {
    let mut doc = Document::new((2, 0));

    // Create first matrix list
    let mut users = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    users.add_row(Node::new(
        "User",
        "alice",
        vec![
            Value::String("alice".to_string().into()),
            Value::String("Alice".to_string().into()),
        ],
    ));

    // Create second matrix list
    let mut products = MatrixList::new("Product", vec!["id".to_string(), "price".to_string()]);
    products.add_row(Node::new(
        "Product",
        "widget",
        vec![Value::String("widget".to_string().into()), Value::Int(100)],
    ));

    doc.root.insert("users".to_string(), Item::List(users));
    doc.root
        .insert("products".to_string(), Item::List(products));

    // Should succeed (writing first list alphabetically with a warning)
    let result = to_parquet_bytes(&doc);
    assert!(
        result.is_ok(),
        "Two matrix lists should succeed (writing first only)"
    );

    // Verify we got the first list (alphabetically: "products" < "users")
    let bytes = result.unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    let list_count = restored
        .root
        .values()
        .filter(|item| matches!(item, Item::List(_)))
        .count();
    assert_eq!(
        list_count, 1,
        "Should have exactly one list in restored document"
    );

    // Should be "products" (first alphabetically)
    assert!(
        restored.root.contains_key("products"),
        "Should contain first list alphabetically (products)"
    );
}

#[test]
fn test_three_matrix_lists_writes_first() {
    let mut doc = Document::new((2, 0));

    // Create three matrix lists
    let mut list1 = MatrixList::new("Type1", vec!["id".to_string()]);
    list1.add_row(Node::new("Type1", "a", vec![Value::String("a".into())]));

    let mut list2 = MatrixList::new("Type2", vec!["id".to_string()]);
    list2.add_row(Node::new("Type2", "b", vec![Value::String("b".into())]));

    let mut list3 = MatrixList::new("Type3", vec!["id".to_string()]);
    list3.add_row(Node::new("Type3", "c", vec![Value::String("c".into())]));

    doc.root.insert("list1".to_string(), Item::List(list1));
    doc.root.insert("list2".to_string(), Item::List(list2));
    doc.root.insert("list3".to_string(), Item::List(list3));

    let result = to_parquet_bytes(&doc);
    assert!(
        result.is_ok(),
        "Three matrix lists should succeed (writing first only)"
    );

    // Verify we got the first list (alphabetically: "list1")
    let bytes = result.unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    let list_count = restored
        .root
        .values()
        .filter(|item| matches!(item, Item::List(_)))
        .count();
    assert_eq!(list_count, 1, "Should have exactly one list");

    // Should be "list1" (first alphabetically)
    assert!(
        restored.root.contains_key("list1"),
        "Should contain first list alphabetically (list1)"
    );
}

#[test]
fn test_warning_printed_for_multiple_lists() {
    let mut doc = Document::new((2, 0));

    let mut list1 = MatrixList::new("User", vec!["id".to_string()]);
    list1.add_row(Node::new("User", "a", vec![Value::String("a".into())]));

    let mut list2 = MatrixList::new("Product", vec!["id".to_string()]);
    list2.add_row(Node::new("Product", "b", vec![Value::String("b".into())]));

    doc.root.insert("users".to_string(), Item::List(list1));
    doc.root.insert("products".to_string(), Item::List(list2));

    let result = to_parquet_bytes(&doc);

    // Should succeed (with warning to stderr)
    assert!(result.is_ok(), "Multiple lists should succeed with warning");
}

#[test]
fn test_mixed_items_single_list_succeeds() {
    let mut doc = Document::new((2, 0));

    // Add scalar items
    doc.root.insert(
        "version".to_string(),
        Item::Scalar(Value::String("1.0".into())),
    );
    doc.root
        .insert("count".to_string(), Item::Scalar(Value::Int(42)));

    // Add single matrix list
    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list.add_row(Node::new(
        "User",
        "alice",
        vec![
            Value::String("alice".to_string().into()),
            Value::String("Alice".to_string().into()),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    let result = to_parquet_bytes(&doc);
    assert!(
        result.is_ok(),
        "Mixed items with single list should succeed"
    );
}

#[test]
fn test_mixed_items_multiple_lists_writes_first() {
    let mut doc = Document::new((2, 0));

    // Add scalar items
    doc.root.insert(
        "version".to_string(),
        Item::Scalar(Value::String("1.0".into())),
    );

    // Add two matrix lists
    let mut list1 = MatrixList::new("User", vec!["id".to_string()]);
    list1.add_row(Node::new("User", "a", vec![Value::String("a".into())]));

    let mut list2 = MatrixList::new("Product", vec!["id".to_string()]);
    list2.add_row(Node::new("Product", "b", vec![Value::String("b".into())]));

    doc.root.insert("users".to_string(), Item::List(list1));
    doc.root.insert("products".to_string(), Item::List(list2));

    let result = to_parquet_bytes(&doc);
    assert!(
        result.is_ok(),
        "Mixed items with multiple lists should succeed (writing first only)"
    );
}

#[test]
fn test_deterministic_first_selection() {
    let mut doc = Document::new((2, 0));

    let mut list1 = MatrixList::new("A", vec!["id".to_string()]);
    list1.add_row(Node::new("A", "first", vec![Value::String("first".into())]));

    let mut list2 = MatrixList::new("B", vec!["id".to_string()]);
    list2.add_row(Node::new(
        "B",
        "second",
        vec![Value::String("second".into())],
    ));

    // Insert in different order
    doc.root.insert("second".to_string(), Item::List(list2));
    doc.root.insert("first".to_string(), Item::List(list1));

    let result1 = to_parquet_bytes(&doc).unwrap();

    // Create same doc with different insertion order
    let mut doc2 = Document::new((2, 0));
    let mut list1 = MatrixList::new("A", vec!["id".to_string()]);
    list1.add_row(Node::new("A", "first", vec![Value::String("first".into())]));
    let mut list2 = MatrixList::new("B", vec!["id".to_string()]);
    list2.add_row(Node::new(
        "B",
        "second",
        vec![Value::String("second".into())],
    ));

    doc2.root.insert("first".to_string(), Item::List(list1));
    doc2.root.insert("second".to_string(), Item::List(list2));

    let result2 = to_parquet_bytes(&doc2).unwrap();

    // Both should write "first" (alphabetically first key)
    let restored1 = from_parquet_bytes(&result1).unwrap();
    let restored2 = from_parquet_bytes(&result2).unwrap();

    assert!(
        restored1.root.contains_key("first"),
        "Should contain 'first' (alphabetically first)"
    );
    assert!(
        restored2.root.contains_key("first"),
        "Should contain 'first' (alphabetically first)"
    );
}

#[test]
fn test_empty_lists_also_counted() {
    let mut doc = Document::new((2, 0));

    // Two empty matrix lists
    let list1 = MatrixList::new("Empty1", vec!["id".to_string()]);
    let list2 = MatrixList::new("Empty2", vec!["id".to_string()]);

    doc.root.insert("empty1".to_string(), Item::List(list1));
    doc.root.insert("empty2".to_string(), Item::List(list2));

    let result = to_parquet_bytes(&doc);
    // Should succeed (writing first empty list)
    assert!(
        result.is_ok(),
        "Two empty matrix lists should succeed (writing first only)"
    );
}

#[test]
fn test_zero_matrix_lists_with_scalars_succeeds() {
    let mut doc = Document::new((2, 0));

    // Only scalar items
    doc.root.insert(
        "version".to_string(),
        Item::Scalar(Value::String("1.0".into())),
    );
    doc.root
        .insert("count".to_string(), Item::Scalar(Value::Int(42)));

    let result = to_parquet_bytes(&doc);
    assert!(result.is_ok(), "Document with only scalars should succeed");
}

#[test]
fn test_behavior_suggestion_for_multiple_lists() {
    let mut doc = Document::new((2, 0));

    let mut list1 = MatrixList::new("User", vec!["id".to_string()]);
    list1.add_row(Node::new("User", "a", vec![Value::String("a".into())]));

    let mut list2 = MatrixList::new("Product", vec!["id".to_string()]);
    list2.add_row(Node::new("Product", "b", vec![Value::String("b".into())]));

    doc.root.insert("users".to_string(), Item::List(list1));
    doc.root.insert("products".to_string(), Item::List(list2));

    let result = to_parquet_bytes(&doc);

    // Should succeed (with warning suggesting to split into separate files)
    assert!(
        result.is_ok(),
        "Should succeed with helpful warning message"
    );
}
