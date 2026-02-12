// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Large file handling and memory management tests for hedl-parquet
//!
//! Tests memory limits, batch processing, and large dataset handling.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{
    from_parquet_bytes, from_parquet_bytes_with_config, to_parquet_bytes,
    to_parquet_bytes_with_config, BatchSize, FromParquetConfig, ToParquetConfig,
};
use parquet::basic::Compression;

// =============================================================================
// Large Row Count Tests
// =============================================================================

#[test]
fn test_large_row_count_1k() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..1_000 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 1_000);
    }
}

#[test]
fn test_large_row_count_10k() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..10_000 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 10_000);
    }
}

#[test]
fn test_large_row_count_100k() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..100_000 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        compression: Compression::ZSTD(Default::default()),
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 100_000);
    }
}

// =============================================================================
// Batch Size Impact Tests
// =============================================================================

#[test]
fn test_small_batch_size_processing() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..1_000 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();

    // Read with small batch size
    let config = FromParquetConfig::default().with_batch_size(BatchSize::Fixed(100));
    let restored = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 1_000);
    }
}

#[test]
fn test_large_batch_size_processing() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..5_000 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();

    // Read with large batch size
    let config = FromParquetConfig::default().with_batch_size(BatchSize::Fixed(10_000));
    let restored = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 5_000);
    }
}

#[test]
fn test_adaptive_batch_size() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..2_000 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::default().with_batch_size(BatchSize::Adaptive(5000));
    let restored = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 2_000);
    }
}

// =============================================================================
// Large String Column Tests
// =============================================================================

#[test]
fn test_large_string_values() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Text", vec!["id".to_string(), "content".to_string()]);

    let large_string = "x".repeat(10_000); // 10KB string

    for i in 0..100 {
        list.add_row(Node::new(
            "Text",
            format!("t{i}"),
            vec![
                Value::String(format!("t{i}").into()),
                Value::String(large_string.clone().into()),
            ],
        ));
    }

    doc.root.insert("text".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("text") {
        assert_eq!(list.rows.len(), 100);
        if let Value::String(s) = &list.rows[0].fields[1] {
            assert_eq!(s.len(), 10_000);
        }
    }
}

#[test]
fn test_many_unique_strings() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "unique_val".to_string()]);

    // Create many unique string values (defeats dictionary encoding)
    for i in 0..1_000 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::String(format!("unique_value_{i}_with_suffix").into()),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 1_000);
    }
}

// =============================================================================
// Wide Column Tests
// =============================================================================

#[test]
fn test_wide_table_100_columns() {
    let mut doc = Document::new((2, 0));

    let mut schema = vec!["id".to_string()];
    for i in 1..100 {
        schema.push(format!("col{i}"));
    }

    let mut list = MatrixList::new("Wide", schema.clone());

    // Create a single row with 100 values
    let mut fields = vec![Value::String("w1".to_string().into())];
    for i in 1..100 {
        fields.push(Value::Int(i64::from(i)));
    }

    list.add_row(Node::new("Wide", "w1", fields));
    doc.root.insert("wide".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("wide") {
        assert_eq!(list.schema.len(), 100);
        assert_eq!(list.rows[0].fields.len(), 100);
    }
}

#[test]
fn test_wide_and_tall_table() {
    let mut doc = Document::new((2, 0));

    // 50 columns
    let mut schema = vec!["id".to_string()];
    for i in 1..50 {
        schema.push(format!("col{i}"));
    }

    let mut list = MatrixList::new("WideAndTall", schema.clone());

    // 1000 rows
    for row_idx in 0..1_000 {
        let mut fields = vec![Value::String(format!("r{row_idx}").into())];
        for col_idx in 1..50 {
            fields.push(Value::Int(i64::from(row_idx * 100 + col_idx)));
        }
        list.add_row(Node::new("WideAndTall", format!("r{row_idx}"), fields));
    }

    doc.root.insert("wide_tall".to_string(), Item::List(list));

    let config = ToParquetConfig {
        compression: Compression::ZSTD(Default::default()),
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("wide_tall") {
        assert_eq!(list.schema.len(), 50);
        assert_eq!(list.rows.len(), 1_000);
    }
}

// =============================================================================
// Memory-Efficient Reading Tests
// =============================================================================

#[test]
fn test_column_projection_reduces_memory() {
    let mut doc = Document::new((2, 0));

    // Create 20 columns
    let mut schema = vec!["id".to_string()];
    for i in 1..20 {
        schema.push(format!("col{i}"));
    }

    let mut list = MatrixList::new("Data", schema.clone());

    for row_idx in 0..500 {
        let mut fields = vec![Value::String(format!("r{row_idx}").into())];
        for _ in 1..20 {
            fields.push(Value::Int(42));
        }
        list.add_row(Node::new("Data", format!("r{row_idx}"), fields));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();

    // Read only 3 columns instead of 20
    let config = FromParquetConfig::with_columns(vec!["id".into(), "col1".into(), "col2".into()]);
    let restored = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        // Should only have 3 columns
        assert_eq!(list.schema.len(), 3);
        assert_eq!(list.rows.len(), 500);
    }
}

// =============================================================================
// Compression Effectiveness Tests
// =============================================================================

#[test]
fn test_compression_ratio_high_cardinality() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    // High cardinality: every value is unique
    for i in 0..1_000 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::String(format!("unique_{i}").into()),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let uncompressed_config = ToParquetConfig {
        compression: Compression::UNCOMPRESSED,
        ..Default::default()
    };

    let compressed_config = ToParquetConfig {
        compression: Compression::ZSTD(Default::default()),
        ..Default::default()
    };

    let uncompressed_bytes = to_parquet_bytes_with_config(&doc, &uncompressed_config).unwrap();
    let compressed_bytes = to_parquet_bytes_with_config(&doc, &compressed_config).unwrap();

    // Compressed should be smaller
    assert!(compressed_bytes.len() < uncompressed_bytes.len());
}

#[test]
fn test_compression_ratio_low_cardinality() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "category".to_string()]);

    // Low cardinality: only 5 distinct values
    let categories = ["A", "B", "C", "D", "E"];
    for i in 0..1_000 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::String(categories[i % 5].to_string().into()),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let uncompressed_config = ToParquetConfig {
        compression: Compression::UNCOMPRESSED,
        enable_dictionary: false,
        ..Default::default()
    };

    let compressed_config = ToParquetConfig {
        compression: Compression::ZSTD(Default::default()),
        enable_dictionary: true,
        ..Default::default()
    };

    let uncompressed_bytes = to_parquet_bytes_with_config(&doc, &uncompressed_config).unwrap();
    let compressed_bytes = to_parquet_bytes_with_config(&doc, &compressed_config).unwrap();

    // Should get some compression with dictionary + ZSTD (be conservative with the ratio)
    // Parquet has overhead, so we just check that compression helps
    assert!(compressed_bytes.len() < uncompressed_bytes.len());
}

// =============================================================================
// Mixed Data Type Tests with Large Scale
// =============================================================================

#[test]
fn test_mixed_types_large_scale() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new(
        "Mixed",
        vec![
            "id".to_string(),
            "int_col".to_string(),
            "float_col".to_string(),
            "bool_col".to_string(),
            "string_col".to_string(),
        ],
    );

    for i in 0..2_000 {
        list.add_row(Node::new(
            "Mixed",
            format!("m{i}"),
            vec![
                Value::String(format!("m{i}").into()),
                Value::Int(i64::from(i)),
                Value::Float(f64::from(i) * 1.5),
                Value::Bool(i % 2 == 0),
                Value::String(format!("string_{}", i % 10).into()),
            ],
        ));
    }

    doc.root.insert("mixed".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("mixed") {
        assert_eq!(list.rows.len(), 2_000);
        assert_eq!(list.schema.len(), 5);
    }
}

// =============================================================================
// Null Handling with Large Scale
// =============================================================================

#[test]
fn test_sparse_nulls_large_scale() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "optional".to_string()]);

    for i in 0..5_000 {
        let value = if i % 10 == 0 {
            Value::Null
        } else {
            Value::Int(i64::from(i))
        };

        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![Value::String(format!("d{i}").into()), value],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 5_000);
        // Verify null count
        let null_count = list
            .rows
            .iter()
            .filter(|r| matches!(r.fields[1], Value::Null))
            .count();
        assert_eq!(null_count, 500); // 10% are null
    }
}

// =============================================================================
// Row Count Edge Cases
// =============================================================================

#[test]
fn test_single_row() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Single", vec!["id".to_string()]);
    list.add_row(Node::new(
        "Single",
        "s1",
        vec![Value::String("s1".to_string().into())],
    ));
    doc.root.insert("single".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("single") {
        assert_eq!(list.rows.len(), 1);
    }
}

#[test]
fn test_two_rows() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Pair", vec!["id".to_string()]);
    list.add_row(Node::new(
        "Pair",
        "p1",
        vec![Value::String("p1".to_string().into())],
    ));
    list.add_row(Node::new(
        "Pair",
        "p2",
        vec![Value::String("p2".to_string().into())],
    ));
    doc.root.insert("pair".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("pair") {
        assert_eq!(list.rows.len(), 2);
    }
}

// =============================================================================
// Memory Estimation Tests
// =============================================================================

#[test]
fn test_estimate_batch_size_basic() {
    // Test that batches are processed without exceeding memory limits
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..1_000 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let bytes = to_parquet_bytes(&doc).unwrap();

    // Use small batch to ensure multiple batches are processed
    let config = FromParquetConfig::default().with_batch_size(BatchSize::Fixed(200));
    let restored = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 1_000);
    }
}
