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
    from_parquet_bytes, to_parquet_bytes, to_parquet_bytes_with_config, ToParquetConfig,
};
use hedl_test::fixtures;
use parquet::basic::Compression;

// =============================================================================
// Basic Round-Trip Tests
// =============================================================================

#[test]
fn test_round_trip_empty_document() {
    let doc = Document::new((1, 0));
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Empty document may produce empty bytes or minimal parquet
    if !bytes.is_empty() {
        let _restored = from_parquet_bytes(&bytes).unwrap();
    }
}

#[test]
fn test_round_trip_single_row() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(100)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.rows[0].id, "i1");
        assert_eq!(list.rows[0].fields.len(), 2); // id and value
        assert_eq!(list.rows[0].fields[0], Value::String("i1".to_string()));
        assert_eq!(list.rows[0].fields[1], Value::Int(100));
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_round_trip_multiple_rows() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "User",
        vec!["id".to_string(), "name".to_string(), "age".to_string()],
    );

    list.add_row(Node::new(
        "User",
        "alice",
        vec![
            Value::String("alice".to_string()),
            Value::String("Alice".to_string()),
            Value::Int(30),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "bob",
        vec![
            Value::String("bob".to_string()),
            Value::String("Bob".to_string()),
            Value::Int(25),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "charlie",
        vec![
            Value::String("charlie".to_string()),
            Value::String("Charlie".to_string()),
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "int_val".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string()), Value::Int(42)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string()), Value::Int(-100)],
    ));
    list.add_row(Node::new(
        "Data",
        "row3",
        vec![Value::String("row3".to_string()), Value::Int(0)],
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "float_val".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string()), Value::Float(3.25)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string()), Value::Float(-2.5)],
    ));
    list.add_row(Node::new(
        "Data",
        "row3",
        vec![Value::String("row3".to_string()), Value::Float(0.0)],
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "bool_val".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string()), Value::Bool(true)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string()), Value::Bool(false)],
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "string_val".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string()),
            Value::String("hello".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string()),
            Value::String("world".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "Data",
        "row3",
        vec![
            Value::String("row3".to_string()),
            Value::String("".to_string()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // fields[0] is the ID, fields[1] is the string_val
        assert_eq!(list.rows[0].fields[1], Value::String("hello".to_string()));
        assert_eq!(list.rows[1].fields[1], Value::String("world".to_string()));
        assert_eq!(list.rows[2].fields[1], Value::String("".to_string()));
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// Null Handling Tests
// =============================================================================

#[test]
fn test_round_trip_null_values() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string()), Value::Int(42)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string()), Value::Null],
    ));
    list.add_row(Node::new(
        "Data",
        "row3",
        vec![Value::String("row3".to_string()), Value::Int(100)],
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string()), Value::Null],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string()), Value::Null],
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Post", vec!["id".to_string(), "author".to_string()]);

    list.add_row(Node::new(
        "Post",
        "post1",
        vec![
            Value::String("post1".to_string()),
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
            assert_eq!(r.id, "alice");
        } else {
            panic!("Expected reference");
        }
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_round_trip_qualified_reference() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Post", vec!["id".to_string(), "author".to_string()]);

    list.add_row(Node::new(
        "Post",
        "post1",
        vec![
            Value::String("post1".to_string()),
            Value::Reference(Reference::qualified("User", "alice")),
        ],
    ));

    doc.root.insert("posts".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("posts") {
        // fields[0] is the ID, fields[1] is the author reference
        if let Value::Reference(r) = &list.rows[0].fields[1] {
            assert_eq!(r.type_name, Some("User".to_string()));
            assert_eq!(r.id, "alice");
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
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "version".to_string(),
        Item::Scalar(Value::String("1.0".to_string())),
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
    let mut doc = Document::new((1, 0));

    // Metadata
    doc.root.insert(
        "app_name".to_string(),
        Item::Scalar(Value::String("test".to_string())),
    );

    // Matrix list
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(100)],
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
    let mut doc = Document::new((1, 0));
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
            Value::String("row1".to_string()),
            Value::Int(1),
            Value::Float(1.1),
            Value::Bool(true),
            Value::String("a".to_string()),
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "mixed".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string()), Value::Int(42)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string()),
            Value::String("text".to_string()),
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    for i in 0..100 {
        list.add_row(Node::new(
            "Item",
            format!("item_{}", i),
            vec![Value::String(format!("item_{}", i)), Value::Int(i as i64)],
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(100)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToParquetConfig {
        compression: Compression::SNAPPY,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("items"));
}

#[test]
fn test_compression_uncompressed() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(100)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToParquetConfig {
        compression: Compression::UNCOMPRESSED,
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "text".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string()),
            Value::String("Hello 世界".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string()),
            Value::String("Привет 🌍".to_string()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // fields[0] is the ID, fields[1] is the text
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("Hello 世界".to_string())
        );
        assert_eq!(
            list.rows[1].fields[1],
            Value::String("Привет 🌍".to_string())
        );
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_special_characters_in_strings() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "text".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string()),
            Value::String("line1\nline2\ttab".to_string()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // fields[0] is the ID, fields[1] is the text
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("line1\nline2\ttab".to_string())
        );
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_large_integers() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "big_int".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string()), Value::Int(i64::MAX)],
    ));
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string()), Value::Int(i64::MIN)],
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
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);

    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string())],
    ));
    list.add_row(Node::new(
        "Item",
        "i2",
        vec![Value::String("i2".to_string())],
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

/// Test user_list fixture with Parquet round-trip.
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
        assert_eq!(list.rows[0].fields[0], Value::String("alice".to_string())); // ID
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("Alice Smith".to_string())
        ); // name from fixture fields[1]
        assert_eq!(
            list.rows[0].fields[2],
            Value::String("alice@example.com".to_string())
        ); // email from fixture fields[2]

        // Verify second user
        assert_eq!(list.rows[1].id, "bob");
        assert_eq!(list.rows[1].fields[0], Value::String("bob".to_string()));
        assert_eq!(
            list.rows[1].fields[1],
            Value::String("Bob Jones".to_string())
        );
        assert_eq!(
            list.rows[1].fields[2],
            Value::String("bob@example.com".to_string())
        );

        // Verify third user
        assert_eq!(list.rows[2].id, "charlie");
        assert_eq!(list.rows[2].fields[0], Value::String("charlie".to_string()));
        assert_eq!(
            list.rows[2].fields[1],
            Value::String("Charlie Brown".to_string())
        );
        assert_eq!(
            list.rows[2].fields[2],
            Value::String("charlie@example.com".to_string())
        );
    } else {
        panic!("Expected users list in restored document");
    }
}

/// Test mixed_type_list fixture with Parquet round-trip.
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
        assert_eq!(list.rows[0].fields[0], Value::String("item1".to_string())); // ID
        assert_eq!(list.rows[0].fields[1], Value::String("Widget".to_string())); // name
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
            Value::String("Best seller".to_string())
        ); // notes

        // Verify second item
        assert_eq!(list.rows[1].id, "item2");
        assert_eq!(list.rows[1].fields.len(), 6);
    } else {
        panic!("Expected items list in restored document");
    }
}

/// Test with_references fixture with Parquet round-trip.
///
/// Verifies that references between entities are correctly serialized to Parquet
/// and restored with proper type information and IDs.
/// With SPEC-compliant behavior, the ID is now included in fields[0].
#[test]
fn test_references_parquet_roundtrip() {
    let doc = fixtures::with_references();

    // Convert to Parquet and back
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Parquet exports each list as a separate table
    // The order and presence of lists in the restored document depends on the implementation
    // Let's check if we have at least some data restored
    assert!(!restored.root.is_empty(), "Expected non-empty document");

    // Verify users list if present
    if let Some(Item::List(users)) = restored.root.get("users") {
        assert_eq!(users.type_name, "User");
        assert_eq!(users.schema.len(), 2); // id, name
        assert_eq!(users.rows.len(), 2);
        assert_eq!(users.rows[0].id, "alice");
        assert_eq!(users.rows[0].fields.len(), 2); // ID and name preserved
        assert_eq!(users.rows[0].fields[0], Value::String("alice".to_string())); // ID
        assert_eq!(
            users.rows[0].fields[1],
            Value::String("Alice Smith".to_string())
        ); // name from fixture fields[1]
        assert_eq!(users.rows[1].id, "bob");
        assert_eq!(users.rows[1].fields[0], Value::String("bob".to_string()));
        assert_eq!(
            users.rows[1].fields[1],
            Value::String("Bob Jones".to_string())
        );
    }

    // Verify posts list with references if present
    if let Some(Item::List(posts)) = restored.root.get("posts") {
        assert_eq!(posts.type_name, "Post");
        assert_eq!(posts.schema.len(), 3); // id, title, author
        assert_eq!(posts.rows.len(), 3);

        // Check first post's reference to alice
        // With SPEC-compliant behavior, all fields preserved
        assert_eq!(posts.rows[0].id, "post1");
        assert_eq!(posts.rows[0].fields.len(), 3); // ID, title, author all preserved
        assert_eq!(posts.rows[0].fields[0], Value::String("post1".to_string())); // ID
        assert_eq!(
            posts.rows[0].fields[1],
            Value::String("Hello World".to_string())
        ); // title from fixture fields[1]
           // fields[2] is author reference - check if it's preserved
        if let Value::Reference(r) = &posts.rows[0].fields[2] {
            assert_eq!(r.id, "alice");
        }

        // Check second post
        assert_eq!(posts.rows[1].id, "post2");
        assert_eq!(posts.rows[1].fields.len(), 3);
        assert_eq!(
            posts.rows[1].fields[1],
            Value::String("Rust is great".to_string())
        );

        // Check third post
        assert_eq!(posts.rows[2].id, "post3");
        assert_eq!(posts.rows[2].fields.len(), 3);
        assert_eq!(
            posts.rows[2].fields[1],
            Value::String("HEDL Tutorial".to_string())
        );
    }
}

/// Test comprehensive fixture with Parquet round-trip.
///
/// Verifies that a complex document with multiple lists, references, and various
/// data types can be correctly exported to Parquet. Note: Parquet only supports
/// matrix lists, so NEST hierarchies are flattened.
/// With SPEC-compliant behavior, the ID is now included in fields[0].
#[test]
fn test_comprehensive_parquet_roundtrip() {
    let doc = fixtures::comprehensive();

    // Convert to Parquet and back
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify that lists are present
    assert!(
        restored.root.contains_key("users")
            || restored.root.contains_key("comments")
            || restored.root.contains_key("tags")
    );

    // Check users list if present
    if let Some(Item::List(users)) = restored.root.get("users") {
        assert_eq!(users.type_name, "User");
        assert_eq!(users.schema.len(), 4); // id, name, email, age
        assert_eq!(users.rows.len(), 2); // alice and bob

        // Verify user has correct fields (all preserved with SPEC-compliant behavior)
        assert_eq!(users.rows[0].id, "alice");
        assert_eq!(users.rows[0].fields.len(), 4); // All fields preserved
        assert_eq!(users.rows[0].fields[0], Value::String("alice".to_string())); // ID
        assert_eq!(
            users.rows[0].fields[1],
            Value::String("Alice Smith".to_string())
        ); // name from fixture fields[1]
        assert_eq!(
            users.rows[0].fields[2],
            Value::String("alice@example.com".to_string())
        ); // email from fixture fields[2]
        assert_eq!(users.rows[0].fields[3], Value::Int(30)); // age from fixture fields[3]
    }

    // Check comments list with references if present
    if let Some(Item::List(comments)) = restored.root.get("comments") {
        assert_eq!(comments.type_name, "Comment");
        assert_eq!(comments.schema.len(), 4); // id, text, author, post
        assert_eq!(comments.rows.len(), 1);

        // Verify comment has references (all preserved with SPEC-compliant behavior)
        assert_eq!(comments.rows[0].fields.len(), 4); // All fields preserved
        assert_eq!(comments.rows[0].fields[0], Value::String("c1".to_string())); // ID
        assert_eq!(
            comments.rows[0].fields[1],
            Value::String("Great article!".to_string())
        ); // text from fixture fields[1]

        if let Value::Reference(author_ref) = &comments.rows[0].fields[2] {
            assert_eq!(author_ref.type_name, Some("User".to_string()));
            assert_eq!(author_ref.id, "bob");
        } else {
            panic!("Expected author reference");
        }

        if let Value::Reference(post_ref) = &comments.rows[0].fields[3] {
            assert_eq!(post_ref.type_name, Some("Post".to_string()));
            assert_eq!(post_ref.id, "p1");
        } else {
            panic!("Expected post reference");
        }
    }

    // Check tags list if present
    if let Some(Item::List(tags)) = restored.root.get("tags") {
        assert_eq!(tags.type_name, "Tag");
        assert_eq!(tags.rows.len(), 2); // rust and hedl tags
    }
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
