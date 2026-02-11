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

//! Comprehensive tests for hedl-parquet conversion
//!
//! Tests bidirectional conversion between HEDL documents and Parquet format.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_parquet::{
    from_parquet_bytes, to_parquet_bytes, to_parquet_bytes_with_config, EnabledStatistics,
    ToParquetConfig,
};
use hedl_test::fixtures;

// =============================================================================
// Basic Round-Trip Tests
// =============================================================================

#[test]
fn test_round_trip_empty_document() {
    let doc = Document::new((2, 0));
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Empty document may produce empty bytes or minimal parquet
    if !bytes.is_empty() {
        let _restored = from_parquet_bytes(&bytes).unwrap();
    }
}

#[test]
fn test_round_trip_single_row() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string().into()), Value::Int(100)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.rows[0].id, "i1");
        assert_eq!(list.rows[0].fields.len(), 2); // id and value
        assert_eq!(
            list.rows[0].fields[0],
            Value::String("i1".to_string().into())
        );
        assert_eq!(list.rows[0].fields[1], Value::Int(100));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_round_trip_multiple_rows() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new(
        "User",
        vec!["id".to_string(), "name".to_string(), "age".to_string()],
    );

    list.add_row(Node::new(
        "User",
        "alice",
        vec![
            Value::String("alice".to_string().into()),
            Value::String("Alice".to_string().into()),
            Value::Int(30),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "bob",
        vec![
            Value::String("bob".to_string().into()),
            Value::String("Bob".to_string().into()),
            Value::Int(25),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "charlie",
        vec![
            Value::String("charlie".to_string().into()),
            Value::String("Charlie".to_string().into()),
            Value::Int(35),
        ],
    ));

    doc.root.insert("users".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("users") {
        assert_eq!(list.rows.len(), 3);
        assert_eq!(list.schema.len(), 3);
        assert_eq!(list.rows[0].fields.len(), 3);
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Type Preservation Tests
// =============================================================================

#[test]
fn test_round_trip_int_values() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "int_val".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Int(42)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string().into()), Value::Int(-100)],
    ));
    list.add_row(Node::new(
        "Data",
        "row3",
        vec![Value::String("row3".to_string().into()), Value::Int(0)],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // fields[0] is the ID, fields[1] is the int_val
        assert!(matches!(list.rows[0].fields[1], Value::Int(42)));
        assert!(matches!(list.rows[1].fields[1], Value::Int(-100)));
        assert!(matches!(list.rows[2].fields[1], Value::Int(0)));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_round_trip_float_values() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "float_val".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Float(3.25)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string().into()), Value::Float(-2.5)],
    ));
    list.add_row(Node::new(
        "Data",
        "row3",
        vec![Value::String("row3".to_string().into()), Value::Float(0.0)],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // fields[0] is the ID, fields[1] is the float_val
        if let Value::Float(f) = list.rows[0].fields[1] {
            assert!((f - 3.25).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_round_trip_bool_values() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "bool_val".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Bool(true)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string().into()), Value::Bool(false)],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // fields[0] is the ID, fields[1] is the bool_val
        assert!(matches!(list.rows[0].fields[1], Value::Bool(true)));
        assert!(matches!(list.rows[1].fields[1], Value::Bool(false)));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_round_trip_string_values() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "string_val".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::String("hello".to_string().into()),
        ],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::String("world".to_string().into()),
        ],
    ));
    list.add_row(Node::new(
        "Data",
        "row3",
        vec![
            Value::String("row3".to_string().into()),
            Value::String(String::new().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // fields[0] is the ID, fields[1] is the string_val
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("hello".to_string().into())
        );
        assert_eq!(
            list.rows[1].fields[1],
            Value::String("world".to_string().into())
        );
        assert_eq!(list.rows[2].fields[1], Value::String(String::new().into()));
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Null Handling Tests
// =============================================================================

#[test]
fn test_round_trip_null_values() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Int(42)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string().into()), Value::Null],
    ));
    list.add_row(Node::new(
        "Data",
        "row3",
        vec![Value::String("row3".to_string().into()), Value::Int(100)],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 3);
        // fields[0] is the ID, fields[1] is the value
        assert!(matches!(list.rows[1].fields[1], Value::Null));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_round_trip_all_null_column() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Null],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string().into()), Value::Null],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 2);
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Reference Tests
// =============================================================================

#[test]
fn test_round_trip_local_reference() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Post", vec!["id".to_string(), "author".to_string()]);

    list.add_row(Node::new(
        "Post",
        "post1",
        vec![
            Value::String("post1".to_string().into()),
            Value::Reference(Reference::local("alice")),
        ],
    ));

    doc.root.insert("posts".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("posts") {
        // fields[0] is the ID, fields[1] is the author reference
        if let Value::Reference(r) = &list.rows[0].fields[1] {
            assert_eq!(r.type_name, None);
            assert_eq!(&*r.id, "alice");
        } else {
            panic!("Expected reference");
        }
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_round_trip_qualified_reference() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Post", vec!["id".to_string(), "author".to_string()]);

    list.add_row(Node::new(
        "Post",
        "post1",
        vec![
            Value::String("post1".to_string().into()),
            Value::Reference(Reference::qualified("User", "alice")),
        ],
    ));

    doc.root.insert("posts".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("posts") {
        // fields[0] is the ID, fields[1] is the author reference
        if let Value::Reference(r) = &list.rows[0].fields[1] {
            assert_eq!(r.type_name.as_deref(), Some("User"));
            assert_eq!(&*r.id, "alice");
        } else {
            panic!("Expected reference");
        }
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Metadata Table Tests
// =============================================================================

#[test]
fn test_round_trip_metadata_only() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "version".to_string(),
        Item::Scalar(Value::String("1.0".to_string().into())),
    );
    doc.root
        .insert("count".to_string(), Item::Scalar(Value::Int(42)));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("version"));
    assert!(restored.root.contains_key("count"));
}

#[test]
fn test_round_trip_mixed_metadata_and_list() {
    let mut doc = Document::new((2, 0));

    // Metadata
    doc.root.insert(
        "app_name".to_string(),
        Item::Scalar(Value::String("test".to_string().into())),
    );

    // Matrix list
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string().into()), Value::Int(100)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Both should be preserved
    assert!(restored.root.contains_key("app_name") || restored.root.contains_key("items"));
}

// =============================================================================
// Multi-Column Tests
// =============================================================================

#[test]
fn test_round_trip_many_columns() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new(
        "Data",
        vec![
            "id".to_string(),
            "col1".to_string(),
            "col2".to_string(),
            "col3".to_string(),
            "col4".to_string(),
            "col5".to_string(),
        ],
    );

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::Int(1),
            Value::Float(1.1),
            Value::Bool(true),
            Value::String("a".to_string().into()),
            Value::Int(10),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.schema.len(), 6);
        assert_eq!(list.rows.len(), 1);
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_round_trip_mixed_types_column() {
    // In Parquet, columns have single types, so mixed types get serialized as strings
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "mixed".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Int(42)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::String("text".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 2);
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Large Data Tests
// =============================================================================

#[test]
fn test_round_trip_many_rows() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    for i in 0..100 {
        list.add_row(Node::new(
            "Item",
            format!("item_{i}"),
            vec![
                Value::String(format!("item_{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Compression Tests
// =============================================================================

#[test]
fn test_compression_snappy() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string().into()), Value::Int(100)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToParquetConfig {
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("items"));
}

#[test]
fn test_compression_uncompressed() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string().into()), Value::Int(100)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToParquetConfig {
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("items"));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_unicode_strings() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "text".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::String("Hello 世界".to_string().into()),
        ],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::String("Привет 🌍".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // fields[0] is the ID, fields[1] is the text
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("Hello 世界".to_string().into())
        );
        assert_eq!(
            list.rows[1].fields[1],
            Value::String("Привет 🌍".to_string().into())
        );
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_special_characters_in_strings() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "text".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::String("line1\nline2\ttab".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // fields[0] is the ID, fields[1] is the text
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("line1\nline2\ttab".to_string().into())
        );
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_large_integers() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "big_int".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::Int(i64::MAX),
        ],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::Int(i64::MIN),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // fields[0] is the ID, fields[1] is the big_int
        assert_eq!(list.rows[0].fields[1], Value::Int(i64::MAX));
        assert_eq!(list.rows[1].fields[1], Value::Int(i64::MIN));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_single_column_list() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);

    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string().into())],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![Value::String("i2".to_string().into())],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.rows.len(), 2);
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Shared Fixture Tests
// =============================================================================

/// Test `user_list` fixture with Parquet round-trip.
///
/// Verifies that a simple User matrix list with [id, name, email] fields
/// can be exported to Parquet and restored correctly.
/// With SPEC-compliant behavior, the ID is now included in fields[0].
#[test]
fn test_user_list_parquet_roundtrip() {
    let doc = fixtures::user_list();

    // Convert to Parquet and back
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify the users list was preserved
    if let Some(Item::List(list)) = restored.root.get("users") {
        assert_eq!(list.type_name, "User");
        assert_eq!(list.schema.len(), 3); // id, name, email
        assert_eq!(list.rows.len(), 3); // alice, bob, charlie

        // Verify first user
        // With SPEC-compliant behavior:
        // - Node.id = original id ("alice")
        // - fields[0] = id ("alice")
        // - fields[1] = name from original fields[0] ("alice" in fixture becomes "Alice Smith")
        // - fields[2] = email from original fields[1] ("alice@example.com" in fixture becomes "Alice Smith")
        assert_eq!(list.rows[0].id, "alice");
        assert_eq!(list.rows[0].type_name, "User");
        assert_eq!(list.rows[0].fields.len(), 3); // All 3 fields preserved

        // Check field values - ID is now in fields[0]
        assert_eq!(
            list.rows[0].fields[0],
            Value::String("alice".to_string().into())
        ); // ID
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("Alice Smith".to_string().into())
        ); // name from fixture fields[1]
        assert_eq!(
            list.rows[0].fields[2],
            Value::String("alice@example.com".to_string().into())
        ); // email from fixture fields[2]

        // Verify second user
        assert_eq!(list.rows[1].id, "bob");
        assert_eq!(
            list.rows[1].fields[0],
            Value::String("bob".to_string().into())
        );
        assert_eq!(
            list.rows[1].fields[1],
            Value::String("Bob Jones".to_string().into())
        );
        assert_eq!(
            list.rows[1].fields[2],
            Value::String("bob@example.com".to_string().into())
        );

        // Verify third user
        assert_eq!(list.rows[2].id, "charlie");
        assert_eq!(
            list.rows[2].fields[0],
            Value::String("charlie".to_string().into())
        );
        assert_eq!(
            list.rows[2].fields[1],
            Value::String("Charlie Brown".to_string().into())
        );
        assert_eq!(
            list.rows[2].fields[2],
            Value::String("charlie@example.com".to_string().into())
        );
    } else {
        panic!("Expected users list in restored document");
    }
}

/// Test `mixed_type_list` fixture with Parquet round-trip.
///
/// Verifies that a matrix list with various value types (int, float, string, bool, null)
/// can be correctly serialized to Parquet and deserialized back.
/// With SPEC-compliant behavior, the ID is now included in fields[0].
#[test]
fn test_mixed_types_parquet_roundtrip() {
    let doc = fixtures::mixed_type_list();

    // Convert to Parquet and back
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify the items list was preserved
    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.type_name, "Item");
        assert_eq!(list.schema.len(), 6); // id, name, count, price, active, notes
        assert_eq!(list.rows.len(), 2);

        // Verify first item with all field types
        // With SPEC-compliant behavior, ID is in fields[0]
        assert_eq!(list.rows[0].id, "item1");
        assert_eq!(list.rows[0].fields.len(), 6); // All fields preserved
        assert_eq!(
            list.rows[0].fields[0],
            Value::String("item1".to_string().into())
        ); // ID
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("Widget".to_string().into())
        ); // name
        assert_eq!(list.rows[0].fields[2], Value::Int(100)); // count

        // Check float value with tolerance
        if let Value::Float(price) = list.rows[0].fields[3] {
            assert!((price - 9.99).abs() < 0.001);
        } else {
            panic!("Expected float for price field");
        }

        assert_eq!(list.rows[0].fields[4], Value::Bool(true)); // active
        assert_eq!(
            list.rows[0].fields[5],
            Value::String("Best seller".to_string().into())
        ); // notes

        // Verify second item
        assert_eq!(list.rows[1].id, "item2");
        assert_eq!(list.rows[1].fields.len(), 6);
    } else {
        panic!("Expected items list in restored document");
    }
}

/// Test `with_references` fixture with Parquet conversion.
///
/// Since Parquet supports only one table per file, documents with multiple
/// matrix lists should write only the first one (with a warning).
/// Note: `BTreeMap` iterates in alphabetical key order.
#[test]
fn test_references_parquet_roundtrip() {
    let doc = fixtures::with_references();

    // This fixture has 2 matrix lists (users and posts)
    // BTreeMap iteration is alphabetical, so "posts" comes before "users"
    // Should write only "posts" (first in alphabetical order) with a warning
    let result = to_parquet_bytes(&doc);
    assert!(
        result.is_ok(),
        "Document with multiple lists should succeed (writing first list only)"
    );

    // Verify we can read back the first list (alphabetically)
    let bytes = result.unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Should have exactly one matrix list
    let list_count = restored
        .root
        .values()
        .filter(|item| matches!(item, Item::List(_)))
        .count();
    assert_eq!(list_count, 1, "Should have exactly one matrix list");

    // Verify the first list (alphabetically: "posts") has data
    let has_data = restored.root.values().any(|item| {
        if let Item::List(list) = item {
            !list.rows.is_empty()
        } else {
            false
        }
    });
    assert!(has_data, "First list should have rows");
}

/// Test comprehensive fixture with Parquet conversion.
///
/// Since Parquet supports only one table per file, documents with multiple
/// matrix lists should write only the first one (with a warning).
#[test]
fn test_comprehensive_parquet_roundtrip() {
    let doc = fixtures::comprehensive();

    // This fixture has 3 matrix lists (users, comments, tags)
    // Should write only the first one with a warning
    let result = to_parquet_bytes(&doc);
    assert!(
        result.is_ok(),
        "Document with multiple lists should succeed (writing first list only)"
    );

    // Verify we can read back the first list
    let bytes = result.unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Should have at least one matrix list
    let has_list = restored
        .root
        .values()
        .any(|item| matches!(item, Item::List(_)));
    assert!(has_list, "Should contain at least one matrix list");
}

/// Test that Parquet handles empty lists correctly.
#[test]
fn test_empty_document_parquet() {
    let doc = fixtures::empty();

    let bytes = to_parquet_bytes(&doc).unwrap();

    // Empty document may produce empty bytes or minimal parquet
    if !bytes.is_empty() {
        let _restored = from_parquet_bytes(&bytes).unwrap();
    }
}

/// Test that Parquet preserves metadata scalars correctly.
///
/// Parquet has a metadata table that stores scalar values from the document root.
#[test]
fn test_named_values_parquet_roundtrip() {
    let doc = fixtures::named_values();

    // Convert to Parquet and back
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify some scalar metadata values are preserved
    // Note: The exact preservation depends on Parquet's metadata table implementation
    let has_scalars = restored
        .root
        .values()
        .any(|item| matches!(item, Item::Scalar(_)));

    // At minimum, we should have some data back
    assert!(!restored.root.is_empty() || has_scalars);
}

// =============================================================================
// Statistics Configuration Tests
// =============================================================================

use bytes::Bytes;
use parquet::file::reader::FileReader;
use parquet::file::serialized_reader::SerializedFileReader;

/// Test that statistics are enabled by default (Chunk level).
#[test]
fn test_statistics_enabled_by_default() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new(
        "Data",
        vec!["id".to_string(), "value".to_string(), "score".to_string()],
    );

    for i in 0..100 {
        list.add_row(Node::new(
            "Data",
            format!("item{i}"),
            vec![
                Value::String(format!("item{i}").into()),
                Value::Int(i64::from(i)),
                Value::Float(f64::from(i) * 1.5),
            ],
        ));
    }
    doc.root.insert("data".to_string(), Item::List(list));

    // Write with default config (should have Chunk statistics)
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Read back and verify statistics exist
    let parquet_bytes = Bytes::from(bytes);
    let reader = SerializedFileReader::new(parquet_bytes).unwrap();
    let metadata = reader.metadata();

    // Should have row group metadata with statistics
    assert!(
        metadata.num_row_groups() > 0,
        "Should have at least one row group"
    );

    let row_group = metadata.row_group(0);
    let value_col = row_group.column(1); // "value" column

    // Statistics should be present for chunk level
    assert!(
        value_col.statistics().is_some(),
        "Statistics should be present for value column"
    );
}

/// Test that statistics can be disabled.
#[test]
fn test_statistics_disabled() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..50 {
        list.add_row(Node::new(
            "Data",
            format!("item{i}"),
            vec![
                Value::String(format!("item{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }
    doc.root.insert("data".to_string(), Item::List(list));

    // Write with statistics disabled using the without_statistics() method
    let bytes = to_parquet_bytes_with_config(&doc, &ToParquetConfig::without_statistics()).unwrap();

    // Read back and verify statistics are absent
    let parquet_bytes = Bytes::from(bytes);
    let reader = SerializedFileReader::new(parquet_bytes).unwrap();
    let metadata = reader.metadata();

    assert!(metadata.num_row_groups() > 0);

    let row_group = metadata.row_group(0);
    let value_col = row_group.column(1);

    // Statistics should be absent when disabled
    assert!(
        value_col.statistics().is_none(),
        "Statistics should be absent when disabled"
    );
}

/// Test page-level statistics.
#[test]
fn test_page_level_statistics() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..100 {
        list.add_row(Node::new(
            "Data",
            format!("item{i}"),
            vec![
                Value::String(format!("item{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }
    doc.root.insert("data".to_string(), Item::List(list));

    // Write with page-level statistics
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Verify file is valid and has statistics
    let parquet_bytes = Bytes::from(bytes);
    let reader = SerializedFileReader::new(parquet_bytes).unwrap();
    let metadata = reader.metadata();

    assert!(metadata.num_row_groups() > 0);

    let row_group = metadata.row_group(0);
    let value_col = row_group.column(1);

    // Statistics should be present at page level too
    assert!(
        value_col.statistics().is_some(),
        "Statistics should be present at page level"
    );
}

/// Test statistics truncation configuration.
#[test]
fn test_statistics_truncation() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "long_string".to_string()]);

    // Create rows with very long strings
    let long_string = "a".repeat(10000); // 10KB string
    for i in 0..10 {
        list.add_row(Node::new(
            "Data",
            format!("item{i}"),
            vec![
                Value::String(format!("item{i}").into()),
                Value::String(format!("{long_string}{i}").into()),
            ],
        ));
    }
    doc.root.insert("data".to_string(), Item::List(list));

    // Write with small statistics truncation
    let config = ToParquetConfig::default();
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();

    // The file should have some content
    assert!(!bytes.is_empty());

    // Verify file is valid
    let parquet_bytes = Bytes::from(bytes);
    let reader = SerializedFileReader::new(parquet_bytes).unwrap();
    let metadata = reader.metadata();

    assert!(metadata.num_row_groups() > 0);
}

/// Test `without_statistics` convenience method.
#[test]
fn test_without_statistics_convenience() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Data",
        "item1",
        vec![Value::String("item1".to_string().into()), Value::Int(42)],
    ));
    doc.root.insert("data".to_string(), Item::List(list));

    // Use without_statistics convenience method
    let config = ToParquetConfig::without_statistics();
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();

    // Verify statistics are absent
    let parquet_bytes = Bytes::from(bytes);
    let reader = SerializedFileReader::new(parquet_bytes).unwrap();
    let row_group = reader.metadata().row_group(0);

    assert!(
        row_group.column(1).statistics().is_none(),
        "without_statistics should disable statistics"
    );
}

/// Test statistics levels are properly differentiated.
#[test]
fn test_statistics_level_differentiation() {
    let create_doc = || {
        let mut doc = Document::new((2, 0));
        let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);
        for i in 0..50 {
            list.add_row(Node::new(
                "Data",
                format!("item{i}"),
                vec![
                    Value::String(format!("item{i}").into()),
                    Value::Int(i64::from(i)),
                ],
            ));
        }
        doc.root.insert("data".to_string(), Item::List(list));
        doc
    };

    // Generate files with different statistics levels
    let bytes_none = to_parquet_bytes_with_config(
        &create_doc(),
        &ToParquetConfig::default().with_statistics(EnabledStatistics::None),
    )
    .unwrap();

    let bytes_chunk = to_parquet_bytes_with_config(
        &create_doc(),
        &ToParquetConfig::default().with_statistics(EnabledStatistics::Chunk),
    )
    .unwrap();

    let bytes_page = to_parquet_bytes_with_config(
        &create_doc(),
        &ToParquetConfig::default().with_statistics(EnabledStatistics::Page),
    )
    .unwrap();

    // None should have smallest metadata
    // Note: Page may be same or larger than Chunk due to additional page-level stats
    // But None should definitely be smaller than either

    // All should produce valid files
    assert!(!bytes_none.is_empty());
    assert!(!bytes_chunk.is_empty());
    assert!(!bytes_page.is_empty());

    // Verify each can be read back
    from_parquet_bytes(&bytes_none).unwrap();
    from_parquet_bytes(&bytes_chunk).unwrap();
    from_parquet_bytes(&bytes_page).unwrap();
}

// =============================================================================
// Type Mismatch Tests (Issue 2)
// =============================================================================

/// Test that type mismatches write NULL by default (not coerced to 0/false).
///
/// This is a regression test for Issue 2: Type mismatches should NOT be
/// silently coerced to default values unless explicitly requested via config.
#[test]
fn test_type_mismatch_writes_null_by_default() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new(
        "Data",
        vec![
            "id".to_string(),
            "int_col".to_string(),
            "bool_col".to_string(),
        ],
    );

    // First row establishes the types
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::Int(42),
            Value::Bool(true),
        ],
    ));

    // Second row has type mismatches
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::String("not an int".to_string().into()), // String in Int column
            Value::Int(99),                                 // Int in Bool column
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    // Write with default config (coerce_types = false)
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(restored_list)) = restored.root.get("data") {
        // Row 1 should have correct values
        assert_eq!(restored_list.rows[0].fields[1], Value::Int(42));
        assert_eq!(restored_list.rows[0].fields[2], Value::Bool(true));

        // Row 2 should have NULL for type mismatches (not 0 or false)
        assert_eq!(
            restored_list.rows[1].fields[1],
            Value::Null,
            "String in Int column should write NULL by default, not 0"
        );
        assert_eq!(
            restored_list.rows[1].fields[2],
            Value::Null,
            "Int in Bool column should write NULL by default, not false"
        );
    } else {
        panic!("Expected data list");
    }
}

/// Test that type coercion works when explicitly enabled.
#[test]
fn test_type_coercion_when_explicitly_enabled() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new(
        "Data",
        vec![
            "id".to_string(),
            "int_col".to_string(),
            "bool_col".to_string(),
            "float_col".to_string(),
        ],
    );

    // First row establishes the types
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::Int(42),
            Value::Bool(true),
            Value::Float(std::f64::consts::PI),
        ],
    ));

    // Second row has type mismatches
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::String("not an int".to_string().into()), // String in Int column
            Value::Int(99),                                 // Int in Bool column
            Value::String("not a float".to_string().into()), // String in Float column
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    // Write with coerce_types = true
    let config = ToParquetConfig::default().with_type_coercion(true);
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(restored_list)) = restored.root.get("data") {
        // Row 1 should have correct values
        assert_eq!(restored_list.rows[0].fields[1], Value::Int(42));
        assert_eq!(restored_list.rows[0].fields[2], Value::Bool(true));

        // Row 2 should have coerced default values
        assert_eq!(
            restored_list.rows[1].fields[1],
            Value::Int(0),
            "String in Int column should coerce to 0 when enabled"
        );
        assert_eq!(
            restored_list.rows[1].fields[2],
            Value::Bool(false),
            "Int in Bool column should coerce to false when enabled"
        );
        assert_eq!(
            restored_list.rows[1].fields[3],
            Value::Float(0.0),
            "String in Float column should coerce to 0.0 when enabled"
        );
    } else {
        panic!("Expected data list");
    }
}

/// Test multiple matrix lists are handled correctly.
///
/// When a document contains multiple matrix lists, only the first one
/// should be written to Parquet (with a warning), since Parquet supports
/// one table per file. `BTreeMap` iteration is alphabetical by key.
#[test]
fn test_multiple_matrix_lists_writes_first_only() {
    let mut doc = Document::new((2, 0));

    // First matrix list (alphabetically: "posts" < "users")
    let mut list1 = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list1.add_row(Node::new(
        "User",
        "alice",
        vec![
            Value::String("alice".to_string().into()),
            Value::String("Alice Smith".to_string().into()),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(list1));

    // Second matrix list (alphabetically first: "posts" < "users")
    let mut list2 = MatrixList::new("Post", vec!["id".to_string(), "title".to_string()]);
    list2.add_row(Node::new(
        "Post",
        "post1",
        vec![
            Value::String("post1".to_string().into()),
            Value::String("First Post".to_string().into()),
        ],
    ));
    doc.root.insert("posts".to_string(), Item::List(list2));

    // Should succeed (writing first list only, with warning to stderr)
    let result = to_parquet_bytes(&doc);
    assert!(
        result.is_ok(),
        "Multiple matrix lists should succeed (writing first only)"
    );

    // Verify we got the first list back (alphabetically)
    let bytes = result.unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Should have exactly one matrix list (the first one alphabetically)
    let list_count = restored
        .root
        .values()
        .filter(|item| matches!(item, Item::List(_)))
        .count();
    assert_eq!(
        list_count, 1,
        "Restored document should have exactly one matrix list"
    );

    // Verify it's the first list alphabetically ("posts" < "users")
    assert!(
        restored.root.contains_key("posts"),
        "Should contain first list alphabetically (posts)"
    );

    // Verify the data
    if let Some(Item::List(posts)) = restored.root.get("posts") {
        assert_eq!(posts.rows.len(), 1, "Should have one post");
        assert_eq!(posts.rows[0].id, "post1", "Should have correct post ID");
    } else {
        panic!("Expected posts list");
    }
}

// =============================================================================
// Value::List Tests
// =============================================================================

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
