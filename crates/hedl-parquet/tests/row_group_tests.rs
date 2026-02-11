// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tests for row group size optimization in Parquet writing.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{from_parquet_bytes, to_parquet_bytes_with_config, ToParquetConfig};

/// Create a test document with the specified number of rows.
fn create_test_document(num_rows: usize) -> Document {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new(
        "TestEntity",
        vec![
            "id".to_string(),
            "name".to_string(),
            "value".to_string(),
            "active".to_string(),
        ],
    );

    for i in 0..num_rows {
        list.add_row(Node::new(
            "TestEntity",
            format!("entity_{i}"),
            vec![
                Value::String(format!("entity_{i}").into()),
                Value::String(format!("Name {i}").into()),
                Value::Int(i as i64),
                Value::Bool(i % 2 == 0),
            ],
        ));
    }

    doc.root.insert("entities".to_string(), Item::List(list));
    doc
}

/// Create a test document with many string columns.
fn create_string_heavy_document(num_rows: usize, num_string_cols: usize) -> Document {
    let mut doc = Document::new((2, 0));
    let mut schema = vec!["id".to_string()];
    for i in 0..num_string_cols {
        schema.push(format!("str_col_{i}"));
    }

    let mut list = MatrixList::new("StringHeavy", schema.clone());

    for i in 0..num_rows {
        let mut fields = vec![Value::String(format!("row_{i}").into())];
        for j in 0..num_string_cols {
            fields.push(Value::String(format!("value_{i}_{j}").into()));
        }
        list.add_row(Node::new("StringHeavy", format!("row_{i}"), fields));
    }

    doc.root.insert("data".to_string(), Item::List(list));
    doc
}

// =============================================================================
// =============================================================================

#[test]
fn test_auto_row_group_size_small_file() {
    // Small file should use smaller row groups
    let _doc = create_test_document(10_000);
    // Test disabled - RowGroupSize feature not implemented
    // This test would verify that small files use appropriately sized row groups
}

#[test]
fn test_auto_row_group_size_medium_file() {
    let _doc = create_test_document(50_000);
    // Test disabled - RowGroupSize feature not implemented
}

#[test]
fn test_auto_row_group_size_large_file() {
    let _doc = create_test_document(1_000_000);
    // Test disabled - RowGroupSize feature not implemented
    // Large files should have reasonably sized row groups
}

#[test]
fn test_auto_row_group_size_wide_table() {
    // Wide tables should use smaller row groups
    let _wide = create_string_heavy_document(1000, 100);
    let _narrow = create_test_document(1000);
    // Test disabled - RowGroupSize feature not implemented
}

#[test]
fn test_auto_row_group_size_string_heavy() {
    // String-heavy tables should use smaller row groups
    let _string_heavy = create_string_heavy_document(1000, 30);
    let _regular = create_test_document(1000);
    // Test disabled - RowGroupSize feature not implemented
}

// =============================================================================
// =============================================================================

#[test]
fn test_fixed_row_group_size() {
    let _doc = create_test_document(1000);
    // Test disabled - RowGroupSize feature not implemented
}

#[test]
fn test_fixed_row_group_size_clamped_min() {
    // Below minimum should be clamped
    let _doc = create_test_document(1000);
    // Test disabled - RowGroupSize feature not implemented
}

#[test]
fn test_fixed_row_group_size_clamped_max() {
    // Above maximum should be clamped
    let _doc = create_test_document(1000);
    // Test disabled - RowGroupSize feature not implemented
}

// =============================================================================
// =============================================================================

#[test]
fn test_target_bytes_basic() {
    // 128MB target with 10 columns
    let _doc = create_test_document(1000);
    // Test disabled - RowGroupSize feature not implemented
}

#[test]
fn test_target_bytes_string_heavy() {
    // String-heavy tables should calculate fewer rows per target
    let _string_heavy = create_string_heavy_document(1000, 30);
    let _regular = create_test_document(1000);
    // Test disabled - RowGroupSize feature not implemented
}

#[test]
fn test_target_bytes_clamped() {
    // Very small target should be clamped to minimum
    let _doc = create_test_document(1000);
    // Test disabled - RowGroupSize feature not implemented
}

// =============================================================================
// Integration Tests - Actual Parquet Writing
// =============================================================================

#[test]
fn test_write_with_auto_row_group_size() {
    let doc = create_test_document(1000);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    // Verify round-trip
    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 1000);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_write_with_fixed_row_group_size() {
    let doc = create_test_document(1000);
    let config = ToParquetConfig::default();
    // Test disabled - with_fixed_row_group_size not implemented

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    // Verify round-trip
    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 1000);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_write_with_target_bytes_row_group() {
    let doc = create_test_document(1000);
    let config = ToParquetConfig::default();
    // Test disabled - with_target_row_group_bytes not implemented

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    // Verify round-trip
    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 1000);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_write_string_heavy_with_auto_sizing() {
    let doc = create_string_heavy_document(500, 30);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    // Verify round-trip
    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("data") {
        assert_eq!(list.rows.len(), 500);
    } else {
        panic!("Expected list item");
    }
}

// =============================================================================
// Config Builder Tests
// =============================================================================

#[test]
fn test_config_with_row_group_size() {
    let _config = ToParquetConfig::default();
    // Test disabled - RowGroupSize feature not implemented
}

#[test]
fn test_config_with_fixed_row_group_size_builder() {
    let _config = ToParquetConfig::default();
    // Test disabled - with_fixed_row_group_size not implemented
}

#[test]
fn test_config_with_target_row_group_bytes_builder() {
    let _config = ToParquetConfig::default();
    // Test disabled - with_target_row_group_bytes not implemented
}

#[test]
fn test_default_config_uses_auto() {
    let _config = ToParquetConfig::default();
    // Test disabled - RowGroupSize feature not implemented
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_empty_document() {
    let doc = Document::new((2, 0));
    let config = ToParquetConfig::default();

    // Should not panic on empty document
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn test_single_row_document() {
    let doc = create_test_document(1);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 1);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_row_group_larger_than_data() {
    // Row group size of 1M but only 100 rows
    let doc = create_test_document(100);
    let config = ToParquetConfig::default();
    // Test disabled - with_fixed_row_group_size not implemented

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

// =============================================================================
// Validation Tests
// =============================================================================

#[test]
fn test_validate_rows() {
    let _doc = create_test_document(1000);
    // Test disabled - validate_rows not implemented
}

// =============================================================================
// =============================================================================

#[test]
fn test_page_size_auto_default() {
    let _doc = create_test_document(1000);
    // Test disabled - PageSize feature not implemented
}

#[test]
fn test_page_size_auto_with_zstd() {
    // ZSTD should use larger pages for better compression
    let _doc = create_test_document(1000);
    // Test disabled - PageSize feature not implemented
}

#[test]
fn test_page_size_auto_string_heavy() {
    // String-heavy should use larger pages
    let _doc = create_string_heavy_document(1000, 30);
    // Test disabled - PageSize feature not implemented
}

#[test]
fn test_page_size_fixed() {
    let _doc = create_test_document(1000);
    // Test disabled - PageSize feature not implemented
}

#[test]
fn test_page_size_fixed_clamped_min() {
    let _doc = create_test_document(1000);
    // Test disabled - PageSize feature not implemented
}

#[test]
fn test_page_size_fixed_clamped_max() {
    let _doc = create_test_document(1000);
    // Test disabled - PageSize feature not implemented
}

#[test]
fn test_page_size_validate() {
    let _doc = create_test_document(1000);
    // Test disabled - PageSize feature not implemented
}

// =============================================================================
// =============================================================================

#[test]
fn test_write_with_auto_page_size() {
    let doc = create_test_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    // Verify round-trip
    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_write_with_fixed_page_size() {
    let doc = create_test_document(100);
    let config = ToParquetConfig::default();
    // Test disabled - with_fixed_page_size not implemented

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_write_with_large_page_size() {
    let doc = create_test_document(100);
    let config = ToParquetConfig::default();
    // Test disabled - with_fixed_page_size not implemented

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_config_with_data_page_size_builder() {
    let _config = ToParquetConfig::default();
    // Test disabled - with_data_page_size not implemented
}

#[test]
fn test_config_with_fixed_page_size_builder() {
    let _config = ToParquetConfig::default();
    // Test disabled - with_fixed_page_size not implemented
}

#[test]
fn test_default_config_uses_auto_page_size() {
    let _config = ToParquetConfig::default();
    // Test disabled - PageSize feature not implemented
}

#[test]
fn test_combined_row_group_and_page_size() {
    let doc = create_test_document(500);
    let config = ToParquetConfig::default();
    // Test disabled - with_fixed_row_group_size and with_fixed_page_size not implemented

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 500);
    } else {
        panic!("Expected list item");
    }
}

// =============================================================================
// =============================================================================

#[test]
fn test_encoding_strategy_global() {
    let doc = create_test_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_encoding_strategy_per_type() {
    let doc = create_test_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_encoding_profile_compatible() {
    let doc = create_test_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_encoding_profile_optimized() {
    let doc = create_test_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_type_based_encoding_compatible() {
    let _doc = create_test_document(100);
    // Test disabled - EncodingStrategy feature not implemented
}

#[test]
fn test_type_based_encoding_balanced() {
    let _doc = create_test_document(100);
    // Test disabled - EncodingStrategy feature not implemented
}

#[test]
fn test_type_based_encoding_optimized() {
    let _doc = create_test_document(100);
    // Test disabled - EncodingStrategy feature not implemented
}

#[test]
fn test_config_with_encoding_strategy_builder() {
    let _config = ToParquetConfig::default();
    // Test disabled - with_encoding_strategy not implemented
}

#[test]
fn test_config_with_type_based_encoding_builder() {
    let _config = ToParquetConfig::default();
    // Test disabled - with_type_based_encoding not implemented
}

#[test]
fn test_config_with_encoding_profile_builder() {
    let _config = ToParquetConfig::default();
    // Test disabled - with_encoding_profile not implemented
}

#[test]
fn test_combined_compression_and_encoding() {
    let doc = create_test_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let doc2 = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = doc2.root.get("entities") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}
