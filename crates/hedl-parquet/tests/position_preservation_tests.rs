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

//! Position preservation tests for HEDL-Parquet conversion.
//!
//! These tests verify that row order is correctly preserved during round-trip
//! conversion between HEDL documents and Parquet format.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{from_parquet_bytes, to_parquet_bytes};

// =============================================================================
// Implicit Position Preservation Tests
// =============================================================================

/// Test that row order is preserved for a simple ordered list.
///
/// Verifies the default behavior where row position is implicitly preserved
/// through sequential processing without any explicit position column.
#[test]
fn test_position_preservation_simple_ordered() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    // Add rows in specific order
    let items = vec![
        ("first", 1),
        ("second", 2),
        ("third", 3),
        ("fourth", 4),
        ("fifth", 5),
    ];

    for (id, value) in &items {
        list.add_row(Node::new(
            "Item",
            *id,
            vec![Value::String(id.to_string()), Value::Int(*value)],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify row order is preserved
    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.rows.len(), 5);

        for (i, (expected_id, expected_value)) in items.iter().enumerate() {
            assert_eq!(list.rows[i].id, *expected_id);
            assert_eq!(
                list.rows[i].fields[0],
                Value::String(expected_id.to_string())
            );
            assert_eq!(list.rows[i].fields[1], Value::Int(*expected_value));
        }
    } else {
        panic!("Expected items list");
    }
}

/// Test position preservation with 100 rows to verify no reordering at scale.
#[test]
fn test_position_preservation_large_dataset() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Row", vec!["id".to_string(), "seq".to_string()]);

    // Add 100 rows with sequential IDs
    for i in 0..100 {
        list.add_row(Node::new(
            "Row",
            format!("row_{:03}", i),
            vec![
                Value::String(format!("row_{:03}", i)),
                Value::Int(i as i64),
            ],
        ));
    }

    doc.root.insert("rows".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify all 100 rows are in correct order
    if let Some(Item::List(list)) = restored.root.get("rows") {
        assert_eq!(list.rows.len(), 100);

        for i in 0..100 {
            let expected_id = format!("row_{:03}", i);
            assert_eq!(list.rows[i].id, expected_id);
            assert_eq!(list.rows[i].fields[0], Value::String(expected_id));
            assert_eq!(list.rows[i].fields[1], Value::Int(i as i64));
        }
    } else {
        panic!("Expected rows list");
    }
}

/// Test position preservation with reverse-sorted data.
///
/// Verifies that the implementation does not sort data by any column,
/// but preserves the exact insertion order.
#[test]
fn test_position_preservation_reverse_sorted() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "priority".to_string()]);

    // Add rows in descending priority order (not alphabetically by ID)
    let items = vec![
        ("urgent", 10),
        ("high", 8),
        ("medium", 5),
        ("low", 3),
        ("deferred", 1),
    ];

    for (id, priority) in &items {
        list.add_row(Node::new(
            "Item",
            *id,
            vec![Value::String(id.to_string()), Value::Int(*priority)],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify order is preserved (not sorted by ID or priority)
    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.rows.len(), 5);

        for (i, (expected_id, expected_priority)) in items.iter().enumerate() {
            assert_eq!(list.rows[i].id, *expected_id);
            assert_eq!(
                list.rows[i].fields[0],
                Value::String(expected_id.to_string())
            );
            assert_eq!(list.rows[i].fields[1], Value::Int(*expected_priority));
        }
    } else {
        panic!("Expected items list");
    }
}

/// Test position preservation with identical values.
///
/// Verifies that rows with identical field values maintain their order,
/// proving that no stable sort or other reordering occurs.
#[test]
fn test_position_preservation_identical_values() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Record", vec!["id".to_string(), "status".to_string()]);

    // Add rows with identical status values
    let ids = vec!["rec1", "rec2", "rec3", "rec4", "rec5"];

    for id in &ids {
        list.add_row(Node::new(
            "Record",
            *id,
            vec![
                Value::String(id.to_string()),
                Value::String("active".to_string()),
            ],
        ));
    }

    doc.root.insert("records".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify order is preserved even with identical field values
    if let Some(Item::List(list)) = restored.root.get("records") {
        assert_eq!(list.rows.len(), 5);

        for (i, expected_id) in ids.iter().enumerate() {
            assert_eq!(list.rows[i].id, *expected_id);
            assert_eq!(
                list.rows[i].fields[0],
                Value::String(expected_id.to_string())
            );
            assert_eq!(
                list.rows[i].fields[1],
                Value::String("active".to_string())
            );
        }
    } else {
        panic!("Expected records list");
    }
}

// =============================================================================
// Explicit Position Column Tests
// =============================================================================

/// Test explicit position column preservation.
///
/// Verifies that when a dedicated "position" column is included in the schema,
/// its values are correctly preserved during round-trip conversion.
#[test]
fn test_explicit_position_column_preservation() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Task",
        vec![
            "id".to_string(),
            "position".to_string(),
            "title".to_string(),
        ],
    );

    // Add rows with explicit position values
    let tasks = vec![
        ("task1", 0, "First task"),
        ("task2", 1, "Second task"),
        ("task3", 2, "Third task"),
        ("task4", 3, "Fourth task"),
    ];

    for (id, position, title) in &tasks {
        list.add_row(Node::new(
            "Task",
            *id,
            vec![
                Value::String(id.to_string()),
                Value::Int(*position),
                Value::String(title.to_string()),
            ],
        ));
    }

    doc.root.insert("tasks".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify both implicit order and explicit position column
    if let Some(Item::List(list)) = restored.root.get("tasks") {
        assert_eq!(list.rows.len(), 4);
        assert_eq!(list.schema.len(), 3); // id, position, title

        for (i, (expected_id, expected_position, expected_title)) in tasks.iter().enumerate() {
            // Verify implicit position (row order)
            assert_eq!(list.rows[i].id, *expected_id);

            // Verify explicit position column
            assert_eq!(list.rows[i].fields[0], Value::String(expected_id.to_string()));
            assert_eq!(list.rows[i].fields[1], Value::Int(*expected_position));
            assert_eq!(
                list.rows[i].fields[2],
                Value::String(expected_title.to_string())
            );
        }
    } else {
        panic!("Expected tasks list");
    }
}

/// Test explicit position column with non-sequential values.
///
/// Verifies that explicit position values are preserved even when they
/// don't match the row index (e.g., after reordering or filtering).
#[test]
fn test_explicit_position_non_sequential() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec![
            "id".to_string(),
            "original_position".to_string(),
            "value".to_string(),
        ],
    );

    // Add rows with non-sequential position values (e.g., after filtering)
    let items = vec![
        ("item_a", 0, "first"),
        ("item_c", 2, "third"),
        ("item_e", 4, "fifth"),
        ("item_g", 6, "seventh"),
    ];

    for (id, position, value) in &items {
        list.add_row(Node::new(
            "Item",
            *id,
            vec![
                Value::String(id.to_string()),
                Value::Int(*position),
                Value::String(value.to_string()),
            ],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify original position values are preserved
    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.rows.len(), 4);

        for (i, (expected_id, expected_position, expected_value)) in items.iter().enumerate() {
            assert_eq!(list.rows[i].id, *expected_id);
            assert_eq!(
                list.rows[i].fields[0],
                Value::String(expected_id.to_string())
            );
            assert_eq!(list.rows[i].fields[1], Value::Int(*expected_position));
            assert_eq!(
                list.rows[i].fields[2],
                Value::String(expected_value.to_string())
            );
        }
    } else {
        panic!("Expected items list");
    }
}

// =============================================================================
// Multi-Type Position Preservation Tests
// =============================================================================

/// Test position preservation with mixed data types.
///
/// Verifies that row order is maintained regardless of the types
/// of values in different columns.
#[test]
fn test_position_preservation_mixed_types() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Data",
        vec![
            "id".to_string(),
            "int_val".to_string(),
            "float_val".to_string(),
            "bool_val".to_string(),
            "string_val".to_string(),
        ],
    );

    // Add rows with various data types
    let data = vec![
        ("row1", 10, 1.5, true, "alpha"),
        ("row2", 20, 2.5, false, "beta"),
        ("row3", 30, 3.5, true, "gamma"),
        ("row4", 40, 4.5, false, "delta"),
    ];

    for (id, int_val, float_val, bool_val, string_val) in &data {
        list.add_row(Node::new(
            "Data",
            *id,
            vec![
                Value::String(id.to_string()),
                Value::Int(*int_val),
                Value::Float(*float_val),
                Value::Bool(*bool_val),
                Value::String(string_val.to_string()),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify order and all field types
    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 4);

        for (i, (expected_id, expected_int, expected_float, expected_bool, expected_string)) in
            data.iter().enumerate()
        {
            assert_eq!(list.rows[i].id, *expected_id);
            assert_eq!(
                list.rows[i].fields[0],
                Value::String(expected_id.to_string())
            );
            assert_eq!(list.rows[i].fields[1], Value::Int(*expected_int));

            // Float comparison with tolerance
            if let Value::Float(actual_float) = list.rows[i].fields[2] {
                assert!((actual_float - expected_float).abs() < 0.001);
            } else {
                panic!("Expected float value");
            }

            assert_eq!(list.rows[i].fields[3], Value::Bool(*expected_bool));
            assert_eq!(
                list.rows[i].fields[4],
                Value::String(expected_string.to_string())
            );
        }
    } else {
        panic!("Expected data list");
    }
}

/// Test position preservation with null values.
///
/// Verifies that rows containing null values maintain their position
/// in the sequence.
#[test]
fn test_position_preservation_with_nulls() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Record",
        vec![
            "id".to_string(),
            "value1".to_string(),
            "value2".to_string(),
        ],
    );

    // Add rows with some null values
    list.add_row(Node::new(
        "Record",
        "rec1",
        vec![
            Value::String("rec1".to_string()),
            Value::Int(100),
            Value::String("data".to_string()),
        ],
    ));
    list.add_row(Node::new(
        "Record",
        "rec2",
        vec![Value::String("rec2".to_string()), Value::Null, Value::Null],
    ));
    list.add_row(Node::new(
        "Record",
        "rec3",
        vec![
            Value::String("rec3".to_string()),
            Value::Int(300),
            Value::Null,
        ],
    ));
    list.add_row(Node::new(
        "Record",
        "rec4",
        vec![
            Value::String("rec4".to_string()),
            Value::Null,
            Value::String("more".to_string()),
        ],
    ));

    doc.root.insert("records".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify order is preserved with null values
    if let Some(Item::List(list)) = restored.root.get("records") {
        assert_eq!(list.rows.len(), 4);

        // Check specific positions
        assert_eq!(list.rows[0].id, "rec1");
        assert_eq!(list.rows[1].id, "rec2");
        assert_eq!(list.rows[2].id, "rec3");
        assert_eq!(list.rows[3].id, "rec4");

        // Verify null values in correct positions
        assert!(matches!(list.rows[1].fields[1], Value::Null));
        assert!(matches!(list.rows[1].fields[2], Value::Null));
        assert!(matches!(list.rows[2].fields[2], Value::Null));
        assert!(matches!(list.rows[3].fields[1], Value::Null));
    } else {
        panic!("Expected records list");
    }
}

// =============================================================================
// Edge Case Position Tests
// =============================================================================

/// Test position preservation with single row.
#[test]
fn test_position_preservation_single_row() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Item",
        "only",
        vec![Value::String("only".to_string()), Value::Int(42)],
    ));

    doc.root.insert("items".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify single row is preserved
    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.rows[0].id, "only");
        assert_eq!(list.rows[0].fields[1], Value::Int(42));
    } else {
        panic!("Expected items list");
    }
}

/// Test position preservation with very large row count (1000 rows).
///
/// This stress test verifies that position is correctly maintained
/// even with large datasets that may involve multiple record batches.
#[test]
fn test_position_preservation_stress_1000_rows() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Entry", vec!["id".to_string(), "index".to_string()]);

    // Add 1000 rows
    for i in 0..1000 {
        list.add_row(Node::new(
            "Entry",
            format!("entry_{:04}", i),
            vec![
                Value::String(format!("entry_{:04}", i)),
                Value::Int(i as i64),
            ],
        ));
    }

    doc.root.insert("entries".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify all 1000 rows in correct order
    if let Some(Item::List(list)) = restored.root.get("entries") {
        assert_eq!(list.rows.len(), 1000);

        // Check first, middle, and last entries
        assert_eq!(list.rows[0].id, "entry_0000");
        assert_eq!(list.rows[0].fields[1], Value::Int(0));

        assert_eq!(list.rows[500].id, "entry_0500");
        assert_eq!(list.rows[500].fields[1], Value::Int(500));

        assert_eq!(list.rows[999].id, "entry_0999");
        assert_eq!(list.rows[999].fields[1], Value::Int(999));

        // Spot check 10 random positions
        for i in (0..1000).step_by(100) {
            let expected_id = format!("entry_{:04}", i);
            assert_eq!(list.rows[i].id, expected_id);
            assert_eq!(list.rows[i].fields[0], Value::String(expected_id));
            assert_eq!(list.rows[i].fields[1], Value::Int(i as i64));
        }
    } else {
        panic!("Expected entries list");
    }
}

/// Test that position is preserved when rows have unicode IDs.
#[test]
fn test_position_preservation_unicode_ids() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "name".to_string()]);

    let items = vec![
        ("日本", "Japan"),
        ("中国", "China"),
        ("한국", "Korea"),
        ("Россия", "Russia"),
        ("العربية", "Arabic"),
    ];

    for (id, name) in &items {
        list.add_row(Node::new(
            "Item",
            *id,
            vec![Value::String(id.to_string()), Value::String(name.to_string())],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    // Round-trip conversion
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Verify unicode IDs are in correct order
    if let Some(Item::List(list)) = restored.root.get("items") {
        assert_eq!(list.rows.len(), 5);

        for (i, (expected_id, expected_name)) in items.iter().enumerate() {
            assert_eq!(list.rows[i].id, *expected_id);
            assert_eq!(
                list.rows[i].fields[0],
                Value::String(expected_id.to_string())
            );
            assert_eq!(
                list.rows[i].fields[1],
                Value::String(expected_name.to_string())
            );
        }
    } else {
        panic!("Expected items list");
    }
}
