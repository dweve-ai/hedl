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

//! Tests for Value::List support in hedl-parquet

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_parquet::{from_parquet_bytes, to_parquet_bytes};

#[test]
fn test_round_trip_with_string_list() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new(
        "User",
        vec!["id".to_string(), "name".to_string(), "roles".to_string()],
    );

    // Add a row with a list of strings
    list.add_row(Node::new(
        "User",
        "alice",
        vec![
            Value::String("alice".to_string().into()),
            Value::String("Alice Smith".to_string().into()),
            Value::List(Box::new(vec![
                Value::String("admin".to_string().into()),
                Value::String("editor".to_string().into()),
                Value::String("viewer".to_string().into()),
            ])),
        ],
    ));

    list.add_row(Node::new(
        "User",
        "bob",
        vec![
            Value::String("bob".to_string().into()),
            Value::String("Bob Jones".to_string().into()),
            Value::List(Box::new(vec![Value::String("viewer".to_string().into())])),
        ],
    ));

    doc.root.insert("users".to_string(), Item::List(list));

    // Round-trip through Parquet
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify the list is preserved
    if let Some(Item::List(restored_list)) = restored.root.get("users") {
        assert_eq!(restored_list.rows.len(), 2);

        // Check first user's roles
        if let Value::List(roles) = &restored_list.rows[0].fields[2] {
            assert_eq!(roles.len(), 3);
            assert_eq!(roles[0].as_str(), Some("admin"));
            assert_eq!(roles[1].as_str(), Some("editor"));
            assert_eq!(roles[2].as_str(), Some("viewer"));
        } else {
            panic!("Expected List value for roles");
        }

        // Check second user's roles
        if let Value::List(roles) = &restored_list.rows[1].fields[2] {
            assert_eq!(roles.len(), 1);
            assert_eq!(roles[0].as_str(), Some("viewer"));
        } else {
            panic!("Expected List value for roles");
        }
    } else {
        panic!("Expected users list in restored document");
    }
}

#[test]
fn test_round_trip_with_empty_list() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "tags".to_string()]);

    list.add_row(Node::new(
        "Item",
        "item1",
        vec![
            Value::String("item1".to_string().into()),
            Value::List(Box::default()),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    // Round-trip through Parquet
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify empty list is preserved
    if let Some(Item::List(restored_list)) = restored.root.get("items") {
        if let Value::List(tags) = &restored_list.rows[0].fields[1] {
            assert_eq!(tags.len(), 0);
        } else {
            panic!("Expected List value for tags");
        }
    } else {
        panic!("Expected items list in restored document");
    }
}

#[test]
fn test_round_trip_with_mixed_type_list() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "values".to_string()]);

    list.add_row(Node::new(
        "Data",
        "data1",
        vec![
            Value::String("data1".to_string().into()),
            Value::List(Box::new(vec![
                Value::Int(42),
                Value::Float(4.56),
                Value::Bool(true),
                Value::String("mixed".to_string().into()),
            ])),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    // Round-trip through Parquet
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify mixed-type list is preserved
    if let Some(Item::List(restored_list)) = restored.root.get("data") {
        if let Value::List(values) = &restored_list.rows[0].fields[1] {
            assert_eq!(values.len(), 4);
            assert_eq!(values[0].as_int(), Some(42));
            assert_eq!(values[1].as_float(), Some(4.56));
            assert_eq!(values[2].as_bool(), Some(true));
            assert_eq!(values[3].as_str(), Some("mixed"));
        } else {
            panic!("Expected List value for values");
        }
    } else {
        panic!("Expected data list in restored document");
    }
}

#[test]
fn test_round_trip_with_reference_list() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Group", vec!["id".to_string(), "members".to_string()]);

    list.add_row(Node::new(
        "Group",
        "group1",
        vec![
            Value::String("group1".to_string().into()),
            Value::List(Box::new(vec![
                Value::Reference(Reference::qualified("User", "alice")),
                Value::Reference(Reference::qualified("User", "bob")),
                Value::Reference(Reference::local("charlie")),
            ])),
        ],
    ));

    doc.root.insert("groups".to_string(), Item::List(list));

    // Round-trip through Parquet
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify reference list is preserved
    if let Some(Item::List(restored_list)) = restored.root.get("groups") {
        if let Value::List(members) = &restored_list.rows[0].fields[1] {
            assert_eq!(members.len(), 3);

            if let Value::Reference(ref r) = members[0] {
                assert_eq!(r.type_name.as_deref(), Some("User"));
                assert_eq!(&*r.id, "alice");
            } else {
                panic!("Expected Reference value");
            }

            if let Value::Reference(ref r) = members[1] {
                assert_eq!(r.type_name.as_deref(), Some("User"));
                assert_eq!(&*r.id, "bob");
            } else {
                panic!("Expected Reference value");
            }

            if let Value::Reference(ref r) = members[2] {
                assert_eq!(r.type_name, None);
                assert_eq!(&*r.id, "charlie");
            } else {
                panic!("Expected Reference value");
            }
        } else {
            panic!("Expected List value for members");
        }
    } else {
        panic!("Expected groups list in restored document");
    }
}

#[test]
fn test_round_trip_with_bool_list() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Config", vec!["id".to_string(), "flags".to_string()]);

    list.add_row(Node::new(
        "Config",
        "config1",
        vec![
            Value::String("config1".to_string().into()),
            Value::List(Box::new(vec![
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
            ])),
        ],
    ));

    doc.root.insert("configs".to_string(), Item::List(list));

    // Round-trip through Parquet
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify bool list is preserved
    if let Some(Item::List(restored_list)) = restored.root.get("configs") {
        if let Value::List(flags) = &restored_list.rows[0].fields[1] {
            assert_eq!(flags.len(), 3);
            assert_eq!(flags[0].as_bool(), Some(true));
            assert_eq!(flags[1].as_bool(), Some(false));
            assert_eq!(flags[2].as_bool(), Some(true));
        } else {
            panic!("Expected List value for flags");
        }
    } else {
        panic!("Expected configs list in restored document");
    }
}
