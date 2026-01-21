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

//! Comprehensive tests for `BatchSize` configuration in hedl-parquet.
//!
//! Tests cover:
//! - Auto batch size calculation based on column count and types
//! - Fixed batch size validation and clamping
//! - Adaptive batch size behavior
//! - Boundary conditions (min/max limits)
//! - Integration with actual Parquet reading
//! - Property-based testing for batch size calculations

// Allow constant assertions - these are intentional compile-time sanity checks
#![allow(clippy::assertions_on_constants)]

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{
    from_parquet_bytes_with_config, to_parquet_bytes, BatchSize, FromParquetConfig,
};
use proptest::prelude::*;

// ============================================================================
// BatchSize Unit Tests
// ============================================================================

#[test]
fn test_batch_size_constants() {
    // Verify constants are sensible
    assert!(BatchSize::MIN_BATCH_SIZE > 0);
    assert!(BatchSize::MIN_BATCH_SIZE <= 1000);
    assert!(BatchSize::MAX_BATCH_SIZE >= 10_000);
    assert!(BatchSize::MAX_BATCH_SIZE <= 10_000_000);
    assert!(BatchSize::MIN_BATCH_SIZE < BatchSize::MAX_BATCH_SIZE);

    // Default batch sizes should be within range
    assert!(BatchSize::DEFAULT_NARROW_BATCH_SIZE >= BatchSize::MIN_BATCH_SIZE);
    assert!(BatchSize::DEFAULT_NARROW_BATCH_SIZE <= BatchSize::MAX_BATCH_SIZE);
    assert!(BatchSize::DEFAULT_MEDIUM_BATCH_SIZE >= BatchSize::MIN_BATCH_SIZE);
    assert!(BatchSize::DEFAULT_MEDIUM_BATCH_SIZE <= BatchSize::MAX_BATCH_SIZE);
    assert!(BatchSize::DEFAULT_WIDE_BATCH_SIZE >= BatchSize::MIN_BATCH_SIZE);
    assert!(BatchSize::DEFAULT_WIDE_BATCH_SIZE <= BatchSize::MAX_BATCH_SIZE);

    // Wide tables should use smaller batches than narrow tables
    assert!(BatchSize::DEFAULT_WIDE_BATCH_SIZE < BatchSize::DEFAULT_NARROW_BATCH_SIZE);
}

#[test]
fn test_batch_size_validate_clamps_minimum() {
    // Values below minimum should be clamped up
    assert_eq!(BatchSize::validate(0), BatchSize::MIN_BATCH_SIZE);
    assert_eq!(BatchSize::validate(1), BatchSize::MIN_BATCH_SIZE);
    assert_eq!(
        BatchSize::validate(BatchSize::MIN_BATCH_SIZE - 1),
        BatchSize::MIN_BATCH_SIZE
    );
}

#[test]
fn test_batch_size_validate_clamps_maximum() {
    // Values above maximum should be clamped down
    assert_eq!(BatchSize::validate(usize::MAX), BatchSize::MAX_BATCH_SIZE);
    assert_eq!(
        BatchSize::validate(BatchSize::MAX_BATCH_SIZE + 1),
        BatchSize::MAX_BATCH_SIZE
    );
    assert_eq!(
        BatchSize::validate(BatchSize::MAX_BATCH_SIZE * 2),
        BatchSize::MAX_BATCH_SIZE
    );
}

#[test]
fn test_batch_size_validate_preserves_valid_values() {
    // Values within range should be preserved
    assert_eq!(
        BatchSize::validate(BatchSize::MIN_BATCH_SIZE),
        BatchSize::MIN_BATCH_SIZE
    );
    assert_eq!(
        BatchSize::validate(BatchSize::MAX_BATCH_SIZE),
        BatchSize::MAX_BATCH_SIZE
    );
    assert_eq!(BatchSize::validate(1000), 1000);
    assert_eq!(BatchSize::validate(10_000), 10_000);
    assert_eq!(BatchSize::validate(100_000), 100_000);
}

// ============================================================================
// Auto Batch Size Calculation Tests
// ============================================================================

#[test]
fn test_auto_size_narrow_table() {
    // < 20 columns, no strings
    let size = BatchSize::calculate_auto_size(5, false);
    assert_eq!(size, BatchSize::DEFAULT_NARROW_BATCH_SIZE);

    let size = BatchSize::calculate_auto_size(19, false);
    assert_eq!(size, BatchSize::DEFAULT_NARROW_BATCH_SIZE);
}

#[test]
fn test_auto_size_narrow_table_with_strings() {
    // < 20 columns, with strings (should halve the batch size)
    let size = BatchSize::calculate_auto_size(5, true);
    assert_eq!(
        size,
        BatchSize::validate(BatchSize::DEFAULT_NARROW_BATCH_SIZE / 2)
    );
}

#[test]
fn test_auto_size_medium_table() {
    // 20-49 columns
    let size = BatchSize::calculate_auto_size(20, false);
    assert_eq!(size, BatchSize::DEFAULT_MEDIUM_BATCH_SIZE);

    let size = BatchSize::calculate_auto_size(49, false);
    assert_eq!(size, BatchSize::DEFAULT_MEDIUM_BATCH_SIZE);
}

#[test]
fn test_auto_size_medium_table_with_strings() {
    let size = BatchSize::calculate_auto_size(30, true);
    assert_eq!(
        size,
        BatchSize::validate(BatchSize::DEFAULT_MEDIUM_BATCH_SIZE / 2)
    );
}

#[test]
fn test_auto_size_wide_table() {
    // >= 50 columns
    let size = BatchSize::calculate_auto_size(50, false);
    assert_eq!(size, BatchSize::DEFAULT_WIDE_BATCH_SIZE);

    let size = BatchSize::calculate_auto_size(100, false);
    assert_eq!(size, BatchSize::DEFAULT_WIDE_BATCH_SIZE);

    let size = BatchSize::calculate_auto_size(500, false);
    assert_eq!(size, BatchSize::DEFAULT_WIDE_BATCH_SIZE);
}

#[test]
fn test_auto_size_wide_table_with_strings() {
    let size = BatchSize::calculate_auto_size(100, true);
    assert_eq!(
        size,
        BatchSize::validate(BatchSize::DEFAULT_WIDE_BATCH_SIZE / 2)
    );
}

#[test]
fn test_auto_size_edge_cases() {
    // Edge case: 0 columns
    let size = BatchSize::calculate_auto_size(0, false);
    // Should use narrow table default (0 < 20)
    assert_eq!(size, BatchSize::DEFAULT_NARROW_BATCH_SIZE);

    // Edge case: single column
    let size = BatchSize::calculate_auto_size(1, false);
    assert_eq!(size, BatchSize::DEFAULT_NARROW_BATCH_SIZE);
}

// ============================================================================
// get_effective_size Tests
// ============================================================================

#[test]
fn test_effective_size_auto() {
    let batch_size = BatchSize::Auto;

    // Should delegate to calculate_auto_size
    assert_eq!(
        batch_size.get_effective_size(5, false),
        BatchSize::calculate_auto_size(5, false)
    );
    assert_eq!(
        batch_size.get_effective_size(30, true),
        BatchSize::calculate_auto_size(30, true)
    );
}

#[test]
fn test_effective_size_fixed() {
    let batch_size = BatchSize::Fixed(5000);

    // Should return fixed value (validated)
    assert_eq!(batch_size.get_effective_size(5, false), 5000);
    assert_eq!(batch_size.get_effective_size(100, true), 5000);
}

#[test]
fn test_effective_size_fixed_clamped() {
    // Below minimum
    let batch_size = BatchSize::Fixed(10);
    assert_eq!(
        batch_size.get_effective_size(5, false),
        BatchSize::MIN_BATCH_SIZE
    );

    // Above maximum
    let batch_size = BatchSize::Fixed(10_000_000);
    assert_eq!(
        batch_size.get_effective_size(5, false),
        BatchSize::MAX_BATCH_SIZE
    );
}

#[test]
fn test_effective_size_adaptive() {
    let batch_size = BatchSize::Adaptive(8000);

    // Should return initial size (validated)
    assert_eq!(batch_size.get_effective_size(5, false), 8000);
    assert_eq!(batch_size.get_effective_size(100, true), 8000);
}

#[test]
fn test_effective_size_adaptive_clamped() {
    let batch_size = BatchSize::Adaptive(50);
    assert_eq!(
        batch_size.get_effective_size(5, false),
        BatchSize::MIN_BATCH_SIZE
    );
}

// ============================================================================
// Default Implementation Tests
// ============================================================================

#[test]
fn test_batch_size_default() {
    let default = BatchSize::default();
    assert!(matches!(default, BatchSize::Auto));
}

// ============================================================================
// Integration Tests with Parquet Reading
// ============================================================================

fn create_test_document(num_rows: usize, num_columns: usize) -> Document {
    let schema: Vec<String> = (0..num_columns)
        .map(|i| {
            if i == 0 {
                "id".to_string()
            } else {
                format!("col_{i}")
            }
        })
        .collect();

    let rows: Vec<Node> = (0..num_rows)
        .map(|i| {
            let fields: Vec<Value> = (0..num_columns)
                .map(|col| {
                    if col == 0 {
                        Value::String(format!("row_{i}").into())
                    } else if col % 3 == 0 {
                        Value::Int(i as i64 * col as i64)
                    } else if col % 3 == 1 {
                        Value::Float(i as f64 * 1.5 + col as f64)
                    } else {
                        Value::String(format!("value_{i}_{col}").into())
                    }
                })
                .collect();

            Node {
                type_name: "TestRow".to_string(),
                id: format!("row_{i}"),
                fields: fields.into(),
                children: None,
                child_count: 0,
            }
        })
        .collect();

    let mut root = std::collections::BTreeMap::new();
    root.insert(
        "data".to_string(),
        Item::List(MatrixList {
            type_name: "TestRow".to_string(),
            schema,
            rows,
            count_hint: Some(num_rows),
        }),
    );

    Document {
        version: (1, 0),
        schema_versions: std::collections::BTreeMap::new(),
        aliases: std::collections::BTreeMap::new(),
        structs: std::collections::BTreeMap::new(),
        nests: std::collections::BTreeMap::new(),
        root,
    }
}

#[test]
fn test_batch_size_integration_auto() {
    let doc = create_test_document(100, 10);
    let bytes = to_parquet_bytes(&doc).expect("Failed to write Parquet");

    let config = FromParquetConfig::new().with_batch_size(BatchSize::Auto);
    let loaded = from_parquet_bytes_with_config(&bytes, &config).expect("Failed to read Parquet");

    // Verify all rows are read
    if let Some(Item::List(list)) = loaded.root.get("data") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item 'data'");
    }
}

#[test]
fn test_batch_size_integration_fixed_small() {
    let doc = create_test_document(100, 5);
    let bytes = to_parquet_bytes(&doc).expect("Failed to write Parquet");

    // Use very small batch size to force multiple batches
    let config =
        FromParquetConfig::new().with_batch_size(BatchSize::Fixed(BatchSize::MIN_BATCH_SIZE));
    let loaded = from_parquet_bytes_with_config(&bytes, &config).expect("Failed to read Parquet");

    // Verify all rows are still read correctly
    if let Some(Item::List(list)) = loaded.root.get("data") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item 'data'");
    }
}

#[test]
fn test_batch_size_integration_fixed_large() {
    let doc = create_test_document(50, 5);
    let bytes = to_parquet_bytes(&doc).expect("Failed to write Parquet");

    // Use very large batch size
    let config = FromParquetConfig::new().with_batch_size(BatchSize::Fixed(100_000));
    let loaded = from_parquet_bytes_with_config(&bytes, &config).expect("Failed to read Parquet");

    // Verify all rows are read
    if let Some(Item::List(list)) = loaded.root.get("data") {
        assert_eq!(list.rows.len(), 50);
    } else {
        panic!("Expected list item 'data'");
    }
}

#[test]
fn test_batch_size_integration_adaptive() {
    let doc = create_test_document(200, 8);
    let bytes = to_parquet_bytes(&doc).expect("Failed to write Parquet");

    let config = FromParquetConfig::new().with_batch_size(BatchSize::Adaptive(1000));
    let loaded = from_parquet_bytes_with_config(&bytes, &config).expect("Failed to read Parquet");

    // Verify all rows are read
    if let Some(Item::List(list)) = loaded.root.get("data") {
        assert_eq!(list.rows.len(), 200);
    } else {
        panic!("Expected list item 'data'");
    }
}

#[test]
fn test_batch_size_data_integrity_roundtrip() {
    // Test that batch size doesn't affect data integrity
    let doc = create_test_document(1000, 10);
    let bytes = to_parquet_bytes(&doc).expect("Failed to write Parquet");

    // Read with different batch sizes
    let configs = [
        BatchSize::Fixed(100),
        BatchSize::Fixed(500),
        BatchSize::Fixed(10_000),
        BatchSize::Auto,
        BatchSize::Adaptive(1000),
    ];

    let results: Vec<_> = configs
        .iter()
        .map(|bs| {
            let config = FromParquetConfig::new().with_batch_size(*bs);
            from_parquet_bytes_with_config(&bytes, &config).expect("Failed to read")
        })
        .collect();

    // All should have same row count
    for (i, result) in results.iter().enumerate() {
        if let Some(Item::List(list)) = result.root.get("data") {
            assert_eq!(
                list.rows.len(),
                1000,
                "BatchSize config {i} produced wrong row count"
            );
        } else {
            panic!("Missing data list for config {i}");
        }
    }

    // Verify row IDs are consistent across all batch size configurations
    let first_ids: Vec<String> = if let Some(Item::List(list)) = results[0].root.get("data") {
        list.rows.iter().map(|n| n.id.clone()).collect()
    } else {
        panic!("Missing first result");
    };

    for (i, result) in results.iter().enumerate().skip(1) {
        if let Some(Item::List(list)) = result.root.get("data") {
            let ids: Vec<String> = list.rows.iter().map(|n| n.id.clone()).collect();
            assert_eq!(
                first_ids, ids,
                "BatchSize config {i} produced different row IDs"
            );
        }
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Validation should always clamp to valid range
    #[test]
    fn prop_validate_always_in_range(size in 0usize..=usize::MAX) {
        let validated = BatchSize::validate(size);
        prop_assert!(validated >= BatchSize::MIN_BATCH_SIZE);
        prop_assert!(validated <= BatchSize::MAX_BATCH_SIZE);
    }

    /// Values in valid range should be preserved
    #[test]
    fn prop_validate_preserves_valid(size in BatchSize::MIN_BATCH_SIZE..=BatchSize::MAX_BATCH_SIZE) {
        let validated = BatchSize::validate(size);
        prop_assert_eq!(validated, size);
    }

    /// Auto size should always be in valid range
    #[test]
    fn prop_auto_size_always_valid(num_columns in 0usize..500, has_strings in proptest::bool::ANY) {
        let size = BatchSize::calculate_auto_size(num_columns, has_strings);
        prop_assert!(size >= BatchSize::MIN_BATCH_SIZE);
        prop_assert!(size <= BatchSize::MAX_BATCH_SIZE);
    }

    /// String-heavy tables should use smaller or equal batch sizes
    #[test]
    fn prop_strings_reduce_batch_size(num_columns in 1usize..500) {
        let size_no_strings = BatchSize::calculate_auto_size(num_columns, false);
        let size_with_strings = BatchSize::calculate_auto_size(num_columns, true);
        prop_assert!(size_with_strings <= size_no_strings);
    }

    /// Wider tables should use smaller or equal batch sizes (for same string configuration)
    #[test]
    fn prop_wider_tables_smaller_batches(
        narrow_cols in 1usize..20,
        wide_cols in 50usize..200,
        has_strings in proptest::bool::ANY
    ) {
        let narrow_size = BatchSize::calculate_auto_size(narrow_cols, has_strings);
        let wide_size = BatchSize::calculate_auto_size(wide_cols, has_strings);
        prop_assert!(wide_size <= narrow_size);
    }

    /// get_effective_size should always return valid values
    #[test]
    fn prop_effective_size_always_valid(
        fixed_size in 0usize..=10_000_000,
        adaptive_size in 0usize..=10_000_000,
        num_columns in 0usize..500,
        has_strings in proptest::bool::ANY
    ) {
        let batch_sizes = vec![
            BatchSize::Auto,
            BatchSize::Fixed(fixed_size),
            BatchSize::Adaptive(adaptive_size),
        ];

        for bs in batch_sizes {
            let effective = bs.get_effective_size(num_columns, has_strings);
            prop_assert!(effective >= BatchSize::MIN_BATCH_SIZE);
            prop_assert!(effective <= BatchSize::MAX_BATCH_SIZE);
        }
    }
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
fn test_boundary_at_tier_transitions() {
    // Test boundary at narrow -> medium (19 vs 20)
    let size_19 = BatchSize::calculate_auto_size(19, false);
    let size_20 = BatchSize::calculate_auto_size(20, false);
    assert!(
        size_20 < size_19,
        "Medium table (20 cols) should use smaller batches than narrow (19)"
    );

    // Test boundary at medium -> wide (49 vs 50)
    let size_49 = BatchSize::calculate_auto_size(49, false);
    let size_50 = BatchSize::calculate_auto_size(50, false);
    assert!(
        size_50 < size_49,
        "Wide table (50 cols) should use smaller batches than medium (49)"
    );
}

#[test]
fn test_extreme_column_counts() {
    // Very large number of columns
    let size = BatchSize::calculate_auto_size(10_000, false);
    assert_eq!(size, BatchSize::DEFAULT_WIDE_BATCH_SIZE);

    let size = BatchSize::calculate_auto_size(10_000, true);
    assert_eq!(
        size,
        BatchSize::validate(BatchSize::DEFAULT_WIDE_BATCH_SIZE / 2)
    );
}
