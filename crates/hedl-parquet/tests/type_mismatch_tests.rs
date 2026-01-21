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

//! Tests for type mismatch handling.
//!
//! Issue 2 (HIGH): Type mismatches coerced to 0/false
//! - Default behavior: Type mismatches write null (preserves data integrity)
//! - With `coerce_types=true`: Type mismatches coerce to default values (legacy)
//! - Tests verify both null and coercion behaviors

use arrow::array::{Array, BooleanArray, Float64Array, Int64Array};
use arrow::record_batch::RecordBatch;
use bytes::Bytes;
use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{
    from_parquet_bytes, to_parquet_bytes, to_parquet_bytes_with_config, ToParquetConfig,
};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

// Helper to extract RecordBatch from Parquet bytes
fn get_record_batch(bytes: &[u8]) -> RecordBatch {
    let bytes = Bytes::copy_from_slice(bytes);
    let reader = ParquetRecordBatchReaderBuilder::try_new(bytes)
        .unwrap()
        .build()
        .unwrap();
    reader.into_iter().next().unwrap().unwrap()
}

// =============================================================================
// Default Behavior Tests (coerce_types = false, writes null)
// =============================================================================

#[test]
fn test_string_in_int_column_writes_null_by_default() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "number".to_string()]);

    // First row has correct type (Int)
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Int(42)],
    ));

    // Second row has type mismatch (String instead of Int)
    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::String("not_a_number".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    // Convert with default config (coerce_types = false)
    let bytes = to_parquet_bytes(&doc).unwrap();
    let batch = get_record_batch(&bytes);

    // Check the number column
    let number_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap();

    assert_eq!(number_col.value(0), 42, "First row should have value 42");
    assert!(number_col.is_null(1), "Second row should be null, not 0");
}

#[test]
fn test_bool_in_int_column_writes_null_by_default() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "number".to_string()]);

    // First row with correct type to establish schema
    list.add_row(Node::new(
        "Data",
        "row0",
        vec![Value::String("row0".to_string().into()), Value::Int(100)],
    ));

    // Second row with type mismatch
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Bool(true)],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let batch = get_record_batch(&bytes);

    let number_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap();

    assert_eq!(number_col.value(0), 100, "First row should be 100");
    assert!(number_col.is_null(1), "Bool in Int column should be null");
}

#[test]
fn test_string_in_bool_column_writes_null_by_default() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "flag".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Bool(true)],
    ));

    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::String("not_a_bool".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let batch = get_record_batch(&bytes);

    let flag_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap();

    assert!(flag_col.value(0), "First row should be true");
    assert!(flag_col.is_null(1), "Second row should be null, not false");
}

#[test]
fn test_int_in_bool_column_writes_null_by_default() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "flag".to_string()]);

    // First row with correct type to establish schema
    list.add_row(Node::new(
        "Data",
        "row0",
        vec![Value::String("row0".to_string().into()), Value::Bool(false)],
    ));

    // Second row with type mismatch
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Int(1)],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let batch = get_record_batch(&bytes);

    let flag_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap();

    assert!(!flag_col.value(0), "First row should be false");
    assert!(flag_col.is_null(1), "Int in Bool column should be null");
}

#[test]
fn test_string_in_float_column_writes_null_by_default() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Float(3.5)],
    ));

    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::String("not_a_float".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let batch = get_record_batch(&bytes);

    let value_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();

    assert_eq!(value_col.value(0), 3.5, "First row should be 3.5");
    assert!(value_col.is_null(1), "Second row should be null, not 0.0");
}

#[test]
fn test_bool_in_float_column_writes_null_by_default() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    // First row with correct type to establish schema
    list.add_row(Node::new(
        "Data",
        "row0",
        vec![Value::String("row0".to_string().into()), Value::Float(1.5)],
    ));

    // Second row with type mismatch
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Bool(true)],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let batch = get_record_batch(&bytes);

    let value_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();

    assert_eq!(value_col.value(0), 1.5, "First row should be 1.5");
    assert!(value_col.is_null(1), "Bool in Float column should be null");
}

#[test]
fn test_int_to_float_is_allowed() {
    // Int to Float is a valid coercion (widening)
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Float(3.5)],
    ));

    list.add_row(Node::new(
        "Data",
        "row2",
        vec![Value::String("row2".to_string().into()), Value::Int(42)],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let batch = get_record_batch(&bytes);

    let value_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();

    assert_eq!(value_col.value(0), 3.5);
    assert_eq!(value_col.value(1), 42.0, "Int should convert to Float");
}

#[test]
fn test_explicit_null_writes_null() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "number".to_string()]);

    // First row with correct type to establish schema
    list.add_row(Node::new(
        "Data",
        "row0",
        vec![Value::String("row0".to_string().into()), Value::Int(42)],
    ));

    // Second row with explicit null
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Null],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let batch = get_record_batch(&bytes);

    let number_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap();

    assert_eq!(number_col.value(0), 42, "First row should be 42");
    assert!(number_col.is_null(1), "Explicit null should be null");
}

// =============================================================================
// Coercion Enabled Tests (coerce_types = true, legacy behavior)
// =============================================================================

#[test]
fn test_string_in_int_column_coerces_to_zero_when_enabled() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "number".to_string()]);

    // First row with correct type to establish schema
    list.add_row(Node::new(
        "Data",
        "row0",
        vec![Value::String("row0".to_string().into()), Value::Int(100)],
    ));

    // Second row with type mismatch
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::String("not_a_number".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    // Enable coercion
    let config = ToParquetConfig::default().with_type_coercion(true);
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let batch = get_record_batch(&bytes);

    let number_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap();

    assert_eq!(number_col.value(0), 100, "First row should be 100");
    assert_eq!(number_col.value(1), 0, "String should coerce to 0");
}

#[test]
fn test_string_in_bool_column_coerces_to_false_when_enabled() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "flag".to_string()]);

    // First row with correct type to establish schema
    list.add_row(Node::new(
        "Data",
        "row0",
        vec![Value::String("row0".to_string().into()), Value::Bool(true)],
    ));

    // Second row with type mismatch
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::String("not_a_bool".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig::default().with_type_coercion(true);
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let batch = get_record_batch(&bytes);

    let flag_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap();

    assert!(flag_col.value(0), "First row should be true");
    assert!(!flag_col.value(1), "String should coerce to false");
}

#[test]
fn test_string_in_float_column_coerces_to_zero_when_enabled() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    // First row with correct type to establish schema
    list.add_row(Node::new(
        "Data",
        "row0",
        vec![Value::String("row0".to_string().into()), Value::Float(2.5)],
    ));

    // Second row with type mismatch
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::String("not_a_float".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig::default().with_type_coercion(true);
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let batch = get_record_batch(&bytes);

    let value_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();

    assert_eq!(value_col.value(0), 2.5, "First row should be 2.5");
    assert_eq!(value_col.value(1), 0.0, "String should coerce to 0.0");
}

#[test]
fn test_explicit_null_stays_null_with_coercion() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "number".to_string()]);

    // First row with correct type to establish schema
    list.add_row(Node::new(
        "Data",
        "row0",
        vec![Value::String("row0".to_string().into()), Value::Int(42)],
    ));

    // Second row with explicit null
    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Null],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig::default().with_type_coercion(true);
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let batch = get_record_batch(&bytes);

    let number_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap();

    assert_eq!(number_col.value(0), 42, "First row should be 42");
    assert!(
        number_col.is_null(1),
        "Explicit null should stay null even with coercion"
    );
}

// =============================================================================
// Mixed Type Tests
// =============================================================================

#[test]
fn test_mixed_valid_and_invalid_types() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "number".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Int(10)],
    ));

    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::String("invalid".to_string().into()),
        ],
    ));

    list.add_row(Node::new(
        "Data",
        "row3",
        vec![Value::String("row3".to_string().into()), Value::Int(30)],
    ));

    list.add_row(Node::new(
        "Data",
        "row4",
        vec![Value::String("row4".to_string().into()), Value::Bool(true)],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    // Test with default (null on mismatch)
    let bytes = to_parquet_bytes(&doc).unwrap();
    let batch = get_record_batch(&bytes);
    let number_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap();

    assert_eq!(number_col.value(0), 10);
    assert!(number_col.is_null(1), "String should be null");
    assert_eq!(number_col.value(2), 30);
    assert!(number_col.is_null(3), "Bool should be null");
}

#[test]
fn test_mixed_valid_and_invalid_types_with_coercion() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "number".to_string()]);

    list.add_row(Node::new(
        "Data",
        "row1",
        vec![Value::String("row1".to_string().into()), Value::Int(10)],
    ));

    list.add_row(Node::new(
        "Data",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::String("invalid".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    // Test with coercion enabled
    let config = ToParquetConfig::default().with_type_coercion(true);
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let batch = get_record_batch(&bytes);
    let number_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap();

    assert_eq!(number_col.value(0), 10);
    assert_eq!(number_col.value(1), 0, "String should coerce to 0");
}

// =============================================================================
// Data Integrity Tests
// =============================================================================

#[test]
fn test_round_trip_preserves_nulls() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "number".to_string()]);

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
            Value::String("invalid".to_string().into()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    // Convert to Parquet and back
    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    // Check that the invalid value became null
    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 2);
        assert_eq!(list.rows[0].fields[1], Value::Int(42));
        assert_eq!(
            list.rows[1].fields[1],
            Value::Null,
            "Invalid type should round-trip as null"
        );
    } else {
        panic!("Expected list in restored document");
    }
}

#[test]
fn test_config_builder_pattern() {
    let config = ToParquetConfig::default()
        .with_type_coercion(true)
        .with_statistics(hedl_parquet::EnabledStatistics::None);

    assert!(config.coerce_types);
    assert_eq!(config.statistics, hedl_parquet::EnabledStatistics::None);
}
