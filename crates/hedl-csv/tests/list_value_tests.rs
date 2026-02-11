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

//! Tests for Value::List support added in v1.1.
//!
//! Tests bidirectional CSV conversion for the new List variant.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_csv::{from_csv, to_csv};

// =============================================================================
// Value::List to CSV Tests
// =============================================================================

#[test]
fn test_list_string_values_to_csv() {
    let mut doc = Document::new((1, 1));
    let mut list = MatrixList::new("User", vec!["id".to_string(), "roles".to_string()]);
    list.add_row(Node::new(
        "User",
        "1",
        vec![
            Value::String("1".to_string().into()),
            Value::List(Box::new(vec![
                Value::String("admin".to_string().into()),
                Value::String("editor".to_string().into()),
                Value::String("viewer".to_string().into()),
            ])),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("(admin, editor, viewer)"));
}

#[test]
fn test_list_bool_values_to_csv() {
    let mut doc = Document::new((1, 1));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "flags".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string().into()),
            Value::List(Box::new(vec![
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
            ])),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("(true, false, true)"));
}

#[test]
fn test_list_empty_to_csv() {
    let mut doc = Document::new((1, 1));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "list".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string().into()),
            Value::List(Box::default()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("()"));
}

#[test]
fn test_list_integer_values_to_csv() {
    let mut doc = Document::new((1, 1));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "numbers".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string().into()),
            Value::List(Box::new(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
                Value::Int(5),
            ])),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("(1, 2, 3, 4, 5)"));
}

#[test]
fn test_list_mixed_types_to_csv() {
    let mut doc = Document::new((1, 1));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "mixed".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string().into()),
            Value::List(Box::new(vec![
                Value::String("hello".to_string().into()),
                Value::Int(42),
                Value::Bool(true),
            ])),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("(hello, 42, true)"));
}

// =============================================================================
// CSV to Value::List Tests
// =============================================================================

#[test]
fn test_list_from_csv_string_values() {
    let csv = "id,roles\n1,\"(admin, editor, viewer)\"";
    let doc = from_csv(csv, "User", &["roles"]).unwrap();
    let list = doc.get("users").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 1);
    let list_val = &list.rows[0].fields[1];
    if let Value::List(items) = list_val {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::String("admin".to_string().into()));
        assert_eq!(items[1], Value::String("editor".to_string().into()));
        assert_eq!(items[2], Value::String("viewer".to_string().into()));
    } else {
        panic!("Expected list value, got {:?}", list_val);
    }
}

#[test]
fn test_list_from_csv_bool_values() {
    let csv = "id,flags\n1,\"(true, false, true)\"";
    let doc = from_csv(csv, "Item", &["flags"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 1);
    let list_val = &list.rows[0].fields[1];
    if let Value::List(items) = list_val {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::Bool(true));
        assert_eq!(items[1], Value::Bool(false));
        assert_eq!(items[2], Value::Bool(true));
    } else {
        panic!("Expected list value");
    }
}

#[test]
fn test_list_from_csv_empty() {
    let csv = "id,list\n1,()";
    let doc = from_csv(csv, "Item", &["list"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 1);
    let list_val = &list.rows[0].fields[1];
    if let Value::List(items) = list_val {
        assert_eq!(items.len(), 0);
    } else {
        panic!("Expected empty list value");
    }
}

#[test]
fn test_list_from_csv_integer_values() {
    let csv = "id,numbers\n1,\"(1, 2, 3, 4, 5)\"";
    let doc = from_csv(csv, "Item", &["numbers"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 1);
    let list_val = &list.rows[0].fields[1];
    if let Value::List(items) = list_val {
        assert_eq!(items.len(), 5);
        assert_eq!(items[0], Value::Int(1));
        assert_eq!(items[1], Value::Int(2));
        assert_eq!(items[2], Value::Int(3));
        assert_eq!(items[3], Value::Int(4));
        assert_eq!(items[4], Value::Int(5));
    } else {
        panic!("Expected list value");
    }
}

// =============================================================================
// Roundtrip Tests
// =============================================================================

#[test]
fn test_list_roundtrip_string_values() {
    let mut doc = Document::new((1, 1));
    let mut list = MatrixList::new("User", vec!["id".to_string(), "tags".to_string()]);
    list.add_row(Node::new(
        "User",
        "1",
        vec![
            Value::String("1".to_string().into()),
            Value::List(Box::new(vec![
                Value::String("rust".to_string().into()),
                Value::String("wasm".to_string().into()),
                Value::String("hedl".to_string().into()),
            ])),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(&csv, "User", &["tags"]).unwrap();
    let list2 = doc2.get("users").unwrap().as_list().unwrap();

    assert_eq!(list2.rows.len(), 1);
    let list_val = &list2.rows[0].fields[1];
    if let Value::List(items) = list_val {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::String("rust".to_string().into()));
        assert_eq!(items[1], Value::String("wasm".to_string().into()));
        assert_eq!(items[2], Value::String("hedl".to_string().into()));
    } else {
        panic!("Expected list value after roundtrip");
    }
}

#[test]
fn test_list_roundtrip_mixed_types() {
    let mut doc = Document::new((1, 1));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "data".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string().into()),
            Value::List(Box::new(vec![
                Value::Int(42),
                Value::String("test".to_string().into()),
                Value::Bool(true),
                Value::Float(4.56),
            ])),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(&csv, "Item", &["data"]).unwrap();
    let list2 = doc2.get("items").unwrap().as_list().unwrap();

    assert_eq!(list2.rows.len(), 1);
    let list_val = &list2.rows[0].fields[1];
    if let Value::List(items) = list_val {
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], Value::Int(42));
        assert_eq!(items[1], Value::String("test".to_string().into()));
        assert_eq!(items[2], Value::Bool(true));
        // Float comparison with tolerance
        if let Value::Float(f) = items[3] {
            assert!((f - 4.56).abs() < 0.001);
        } else {
            panic!("Expected float value");
        }
    } else {
        panic!("Expected list value after roundtrip");
    }
}

#[test]
fn test_list_with_references() {
    let csv = "id,refs\n1,\"(@user1, @User:alice, @User:bob)\"";
    let doc = from_csv(csv, "Item", &["refs"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 1);
    let list_val = &list.rows[0].fields[1];
    if let Value::List(items) = list_val {
        assert_eq!(items.len(), 3);

        if let Value::Reference(r) = &items[0] {
            assert_eq!(&*r.id, "user1");
            assert_eq!(r.type_name, None);
        } else {
            panic!("Expected reference value");
        }

        if let Value::Reference(r) = &items[1] {
            assert_eq!(&*r.id, "alice");
            assert_eq!(r.type_name.as_deref(), Some("User"));
        } else {
            panic!("Expected reference value");
        }

        if let Value::Reference(r) = &items[2] {
            assert_eq!(&*r.id, "bob");
            assert_eq!(r.type_name.as_deref(), Some("User"));
        } else {
            panic!("Expected reference value");
        }
    } else {
        panic!("Expected list value");
    }
}

#[test]
fn test_list_single_item() {
    let csv = "id,item\n1,(single)";
    let doc = from_csv(csv, "Item", &["item"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 1);
    let list_val = &list.rows[0].fields[1];
    if let Value::List(items) = list_val {
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], Value::String("single".to_string().into()));
    } else {
        panic!("Expected list value");
    }
}

#[test]
fn test_list_in_csv_requires_quoting_due_to_commas() {
    // CSV values containing commas MUST be quoted per CSV specification.
    // The list literal (a, b, c) contains commas, so it must be quoted in CSV.
    let csv = "id,list\n1,\"(a, b, c)\"";
    let doc = from_csv(csv, "Item", &["list"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 1);
    let list_val = &list.rows[0].fields[1];
    if let Value::List(items) = list_val {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], Value::String("a".to_string().into()));
        assert_eq!(items[1], Value::String("b".to_string().into()));
        assert_eq!(items[2], Value::String("c".to_string().into()));
    } else {
        panic!("Expected list value");
    }
}
