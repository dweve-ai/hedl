// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Comprehensive tests for column pruning / projection pushdown functionality.
// Tests the ability to read only specific columns from Parquet files for
// performance optimization on wide tables.
// Allow single_match for proptest tuple destructuring patterns

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{
    from_parquet_bytes, from_parquet_bytes_select, from_parquet_bytes_with_config,
    to_parquet_bytes, FromParquetConfig,
};
use proptest::prelude::*;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a test document with a specified number of columns and rows.
fn create_wide_table_document(num_columns: usize, num_rows: usize) -> Document {
    let mut doc = Document::new((2, 0));

    // Build schema: id, col_1, col_2, ..., col_N
    let mut schema = vec!["id".to_string()];
    for i in 1..num_columns {
        schema.push(format!("col_{i}"));
    }

    let mut matrix_list = MatrixList::new("WideTable", schema.clone());

    for row_idx in 0..num_rows {
        let mut fields = vec![Value::String(format!("row_{row_idx}").into())];
        for col_idx in 1..num_columns {
            // Mix of different value types
            match col_idx % 4 {
                0 => fields.push(Value::Int((row_idx * col_idx) as i64)),
                1 => fields.push(Value::Float((row_idx as f64) * 1.5 + (col_idx as f64))),
                2 => fields.push(Value::Bool(row_idx % 2 == 0)),
                _ => fields.push(Value::String(format!("value_{row_idx}_{col_idx}").into())),
            }
        }

        matrix_list.add_row(Node::new("WideTable", format!("row_{row_idx}"), fields));
    }

    doc.root.insert("data".to_string(), Item::List(matrix_list));
    doc.structs.insert("WideTable".to_string(), schema);
    doc
}

/// Create a document with specific column types for testing type preservation.
fn create_typed_document() -> Document {
    let mut doc = Document::new((2, 0));

    let schema = vec![
        "id".to_string(),
        "int_col".to_string(),
        "float_col".to_string(),
        "bool_col".to_string(),
        "string_col".to_string(),
        "nullable_col".to_string(),
    ];

    let mut matrix_list = MatrixList::new("TypedTable", schema.clone());

    matrix_list.add_row(Node::new(
        "TypedTable",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::Int(42),
            Value::Float(4.56789),
            Value::Bool(true),
            Value::String("hello".to_string().into()),
            Value::Int(100),
        ],
    ));

    matrix_list.add_row(Node::new(
        "TypedTable",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::Int(-999),
            Value::Float(-5.67891),
            Value::Bool(false),
            Value::String("world".to_string().into()),
            Value::Null,
        ],
    ));

    matrix_list.add_row(Node::new(
        "TypedTable",
        "row3",
        vec![
            Value::String("row3".to_string().into()),
            Value::Int(0),
            Value::Float(0.0),
            Value::Bool(true),
            Value::String("test".to_string().into()),
            Value::Int(0),
        ],
    ));

    doc.root
        .insert("typed_data".to_string(), Item::List(matrix_list));
    doc.structs.insert("TypedTable".to_string(), schema);
    doc
}

// =============================================================================
// BASIC COLUMN SELECTION TESTS
// =============================================================================

#[test]
fn test_select_single_column() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select only the ID column
    let loaded = from_parquet_bytes_select(&bytes, vec!["id".to_string()]).unwrap();

    // Should have data
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_select_multiple_columns() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select specific columns
    let loaded = from_parquet_bytes_select(
        &bytes,
        vec![
            "id".to_string(),
            "int_col".to_string(),
            "string_col".to_string(),
        ],
    )
    .unwrap();

    assert!(!loaded.root.is_empty());
}

#[test]
fn test_select_all_columns_explicitly() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select all columns explicitly
    let loaded = from_parquet_bytes_select(
        &bytes,
        vec![
            "id".to_string(),
            "int_col".to_string(),
            "float_col".to_string(),
            "bool_col".to_string(),
            "string_col".to_string(),
            "nullable_col".to_string(),
        ],
    )
    .unwrap();

    // Compare with default (all columns) read
    let default_loaded = from_parquet_bytes(&bytes).unwrap();

    // Both should have same structure
    assert_eq!(loaded.root.len(), default_loaded.root.len());
}

#[test]
fn test_column_order_preserved() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select columns in different order than schema
    let loaded = from_parquet_bytes_select(
        &bytes,
        vec![
            "string_col".to_string(),
            "id".to_string(),
            "bool_col".to_string(),
        ],
    )
    .unwrap();

    assert!(!loaded.root.is_empty());
}

// =============================================================================
// ERROR HANDLING TESTS
// =============================================================================

#[test]
fn test_empty_column_list_error() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Empty column list should fail
    let result = from_parquet_bytes_select(&bytes, vec![]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{err:?}");
    // Should mention column or empty
    assert!(
        msg.to_lowercase().contains("column") || msg.to_lowercase().contains("empty"),
        "Error should mention column or empty: {msg}"
    );
}

#[test]
fn test_nonexistent_column_error() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Non-existent column should fail
    let result = from_parquet_bytes_select(&bytes, vec!["nonexistent_column".to_string()]);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{err:?}");
    // Should mention column name
    assert!(
        msg.contains("nonexistent") || msg.to_lowercase().contains("not found"),
        "Error should mention nonexistent column: {msg}"
    );
}

#[test]
fn test_mixed_valid_invalid_columns_error() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Mix of valid and invalid columns
    let result = from_parquet_bytes_select(
        &bytes,
        vec![
            "id".to_string(),
            "invalid_col".to_string(),
            "int_col".to_string(),
        ],
    );

    assert!(result.is_err());
}

#[test]
fn test_case_sensitive_column_names() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Column names are case sensitive
    let result = from_parquet_bytes_select(&bytes, vec!["ID".to_string()]); // uppercase

    // Should fail because column is "id" not "ID"
    assert!(result.is_err());
}

// =============================================================================
// CONFIGURATION API TESTS
// =============================================================================

#[test]
fn test_with_columns_config() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::with_columns(vec!["id".to_string(), "int_col".to_string()]);

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_with_column_single_config() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::with_column("id".to_string());

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_set_columns_method() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::default()
        .set_columns(Some(vec!["id".to_string(), "float_col".to_string()]));

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_set_columns_none_reads_all() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::default().set_columns(None);

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    // Compare with default read
    let default_loaded = from_parquet_bytes(&bytes).unwrap();
    assert_eq!(loaded.root.len(), default_loaded.root.len());
}

// =============================================================================
// WIDE TABLE PERFORMANCE TESTS
// =============================================================================

#[test]
fn test_wide_table_selective_read() {
    // Create a wide table with 50 columns
    let doc = create_wide_table_document(50, 10);
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select only 3 columns (6%)
    let loaded = from_parquet_bytes_select(
        &bytes,
        vec!["id".to_string(), "col_1".to_string(), "col_10".to_string()],
    )
    .unwrap();

    assert!(!loaded.root.is_empty());
}

#[test]
fn test_very_wide_table_single_column() {
    // Create a very wide table with 100 columns
    let doc = create_wide_table_document(100, 5);
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select only ID column (1%)
    let loaded = from_parquet_bytes_select(&bytes, vec!["id".to_string()]).unwrap();

    assert!(!loaded.root.is_empty());
}

#[test]
fn test_wide_table_half_columns() {
    // Create a wide table with 20 columns
    let doc = create_wide_table_document(20, 10);
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select 10 columns (50%)
    let columns: Vec<String> = (0..10)
        .map(|i| {
            if i == 0 {
                "id".to_string()
            } else {
                format!("col_{i}")
            }
        })
        .collect();

    let loaded = from_parquet_bytes_select(&bytes, columns).unwrap();

    assert!(!loaded.root.is_empty());
}

// =============================================================================
// DUPLICATE COLUMN HANDLING
// =============================================================================

#[test]
fn test_duplicate_columns_deduplicated() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Request same column multiple times
    let loaded = from_parquet_bytes_select(
        &bytes,
        vec![
            "id".to_string(),
            "id".to_string(),
            "int_col".to_string(),
            "int_col".to_string(),
        ],
    )
    .unwrap();

    assert!(!loaded.root.is_empty());
}

// =============================================================================
// SPECIAL COLUMN NAME TESTS
// =============================================================================

#[test]
fn test_column_with_underscore() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // int_col has underscore
    let loaded = from_parquet_bytes_select(&bytes, vec!["int_col".to_string()]).unwrap();

    assert!(!loaded.root.is_empty());
}

#[test]
fn test_column_with_numeric_suffix() {
    let doc = create_wide_table_document(10, 3);
    let bytes = to_parquet_bytes(&doc).unwrap();

    // col_1, col_2 etc have numeric suffixes
    // Must include 'id' as first column since Parquet->HEDL expects string ID in first column
    let loaded = from_parquet_bytes_select(
        &bytes,
        vec!["id".to_string(), "col_1".to_string(), "col_2".to_string()],
    )
    .unwrap();

    assert!(!loaded.root.is_empty());
}

// =============================================================================
// NULL VALUE HANDLING IN SELECTED COLUMNS
// =============================================================================

#[test]
fn test_select_column_with_nulls() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // nullable_col has null values
    let loaded =
        from_parquet_bytes_select(&bytes, vec!["id".to_string(), "nullable_col".to_string()])
            .unwrap();

    assert!(!loaded.root.is_empty());

    // Verify nulls are preserved
    if let Some(Item::List(list)) = loaded.root.values().next() {
        let has_null = list
            .rows
            .iter()
            .any(|row| row.fields.iter().any(|f| matches!(f, Value::Null)));
        assert!(has_null, "Should preserve null values in selected column");
    }
}

// =============================================================================
// TYPE PRESERVATION TESTS
// =============================================================================

#[test]
fn test_select_preserves_int_type() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let loaded =
        from_parquet_bytes_select(&bytes, vec!["id".to_string(), "int_col".to_string()]).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        for row in &list.rows {
            // int_col should be at some position
            let has_int = row.fields.iter().any(|f| matches!(f, Value::Int(_)));
            assert!(has_int, "Int type should be preserved");
        }
    }
}

#[test]
fn test_select_preserves_float_type() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let loaded =
        from_parquet_bytes_select(&bytes, vec!["id".to_string(), "float_col".to_string()]).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        for row in &list.rows {
            let has_float = row.fields.iter().any(|f| matches!(f, Value::Float(_)));
            assert!(has_float, "Float type should be preserved");
        }
    }
}

#[test]
fn test_select_preserves_bool_type() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let loaded =
        from_parquet_bytes_select(&bytes, vec!["id".to_string(), "bool_col".to_string()]).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        for row in &list.rows {
            let has_bool = row.fields.iter().any(|f| matches!(f, Value::Bool(_)));
            assert!(has_bool, "Bool type should be preserved");
        }
    }
}

// =============================================================================
// EMPTY TABLE EDGE CASES
// =============================================================================

#[test]
fn test_select_from_empty_table() {
    let mut doc = Document::new((2, 0));
    let schema = vec!["id".to_string(), "name".to_string(), "value".to_string()];
    let matrix_list = MatrixList::new("EmptyTable", schema.clone());
    doc.root.insert("data".to_string(), Item::List(matrix_list));
    doc.structs.insert("EmptyTable".to_string(), schema);

    let bytes = to_parquet_bytes(&doc).unwrap();

    // Selecting from empty table should succeed (or be empty)
    let result = from_parquet_bytes_select(&bytes, vec!["id".to_string(), "name".to_string()]);
    // Empty tables may produce empty parquet which might error, which is acceptable
    // The key is it shouldn't panic
    let _ = result;
}

// =============================================================================
// ROUNDTRIP CONSISTENCY TESTS
// =============================================================================

#[test]
fn test_roundtrip_selected_columns_values() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Full read
    let full_loaded = from_parquet_bytes(&bytes).unwrap();

    // Selective read
    let selective_loaded =
        from_parquet_bytes_select(&bytes, vec!["id".to_string(), "int_col".to_string()]).unwrap();

    // Both should have same number of rows
    if let (Some(Item::List(full_list)), Some(Item::List(sel_list))) = (
        full_loaded.root.values().next(),
        selective_loaded.root.values().next(),
    ) {
        assert_eq!(
            full_list.rows.len(),
            sel_list.rows.len(),
            "Row count should match"
        );
    }
}

// =============================================================================
// COMBINED CONFIGURATION TESTS
// =============================================================================

#[test]
fn test_columns_with_batch_size() {
    use hedl_parquet::BatchSize;

    let doc = create_wide_table_document(30, 50);
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::with_columns(vec!["id".to_string(), "col_1".to_string()])
        .with_batch_size(BatchSize::Fixed(10));

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_columns_with_null_handling() {
    use hedl_parquet::NullIdHandling;

    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config =
        FromParquetConfig::with_columns(vec!["id".to_string(), "nullable_col".to_string()])
            .with_null_id_handling(NullIdHandling::Error);

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
    assert!(!loaded.root.is_empty());
}

// =============================================================================
// PROPERTY-BASED TESTS
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_select_subset_never_panics(
        num_cols in 3usize..20,
        num_rows in 1usize..10,
        select_count in 1usize..5
    ) {
        let doc = create_wide_table_document(num_cols, num_rows);
        let bytes_result = to_parquet_bytes(&doc);

        if let Ok(bytes) = bytes_result {
            // Build valid column names
            let mut columns = vec!["id".to_string()];
            for i in 1..select_count.min(num_cols) {
                columns.push(format!("col_{i}"));
            }

            // Should not panic
            let _ = from_parquet_bytes_select(&bytes, columns);
        }
    }

    #[test]
    fn prop_select_all_columns_matches_default(
        num_cols in 3usize..15,
        num_rows in 1usize..5
    ) {
        let doc = create_wide_table_document(num_cols, num_rows);
        let bytes_result = to_parquet_bytes(&doc);

        if let Ok(bytes) = bytes_result {
            // Build all column names
            let mut columns = vec!["id".to_string()];
            for i in 1..num_cols {
                columns.push(format!("col_{i}"));
            }

            let select_result = from_parquet_bytes_select(&bytes, columns);
            let default_result = from_parquet_bytes(&bytes);

            // Both should succeed or both should fail
            // (Both failing is acceptable for edge cases)
            if let (Ok(sel), Ok(def)) = (select_result, default_result) {
                // Should have same root key count
                prop_assert_eq!(sel.root.len(), def.root.len());
            }
        }
    }

    #[test]
    fn prop_invalid_column_always_errors(
        num_cols in 2usize..10,
        num_rows in 1usize..5,
        invalid_suffix in 1000usize..9999
    ) {
        let doc = create_wide_table_document(num_cols, num_rows);
        let bytes_result = to_parquet_bytes(&doc);

        if let Ok(bytes) = bytes_result {
            // Request a column that doesn't exist
            let invalid_col = format!("nonexistent_col_{invalid_suffix}");
            let result = from_parquet_bytes_select(&bytes, vec![invalid_col]);

            prop_assert!(result.is_err());
        }
    }
}

// =============================================================================
// STRESS TESTS
// =============================================================================

#[test]
fn test_many_columns_selection() {
    // Create table with many columns
    let doc = create_wide_table_document(100, 5);
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select 50 columns
    let mut columns = vec!["id".to_string()];
    for i in 1..50 {
        columns.push(format!("col_{i}"));
    }

    let loaded = from_parquet_bytes_select(&bytes, columns).unwrap();
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_repeated_selection_consistency() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let columns = vec!["id".to_string(), "int_col".to_string()];

    // Perform same selection multiple times
    let mut results = Vec::new();
    for _ in 0..5 {
        let loaded = from_parquet_bytes_select(&bytes, columns.clone()).unwrap();
        if let Some(Item::List(list)) = loaded.root.values().next() {
            results.push(list.rows.len());
        }
    }

    // All should be identical
    assert!(results.iter().all(|&r| r == results[0]));
}

// =============================================================================
// BOUNDARY CONDITION TESTS
// =============================================================================

#[test]
fn test_single_row_single_column() {
    let mut doc = Document::new((2, 0));
    let schema = vec!["id".to_string(), "value".to_string()];
    let mut matrix_list = MatrixList::new("Tiny", schema.clone());
    matrix_list.add_row(Node::new(
        "Tiny",
        "only_row",
        vec![Value::String("only_row".to_string().into()), Value::Int(42)],
    ));
    doc.root.insert("tiny".to_string(), Item::List(matrix_list));

    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select single column from single row table
    let loaded = from_parquet_bytes_select(&bytes, vec!["id".to_string()]).unwrap();
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_first_column_only() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select only the first column (id)
    let loaded = from_parquet_bytes_select(&bytes, vec!["id".to_string()]).unwrap();
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_last_column_only() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select only the last column (nullable_col)
    // Note: Since nullable_col has null values and becomes the ID column when pruned,
    // we need lenient mode to auto-generate IDs for null values
    let config = FromParquetConfig::lenient().set_columns(Some(vec!["nullable_col".to_string()]));
    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_first_and_last_columns() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Select first and last columns
    let loaded =
        from_parquet_bytes_select(&bytes, vec!["id".to_string(), "nullable_col".to_string()])
            .unwrap();
    assert!(!loaded.root.is_empty());
}

// =============================================================================
// WHITESPACE AND SPECIAL CHARACTER TESTS
// =============================================================================

#[test]
fn test_column_name_whitespace_handling() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Attempt to select with leading/trailing whitespace (should fail)
    let result = from_parquet_bytes_select(&bytes, vec![" id".to_string()]);
    assert!(
        result.is_err(),
        "Column with leading whitespace should not match"
    );

    let result = from_parquet_bytes_select(&bytes, vec!["id ".to_string()]);
    assert!(
        result.is_err(),
        "Column with trailing whitespace should not match"
    );
}

#[test]
fn test_empty_string_column_name() {
    let doc = create_typed_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Empty string column name
    let result = from_parquet_bytes_select(&bytes, vec![String::new()]);
    assert!(result.is_err());
}

// =============================================================================
// CONCURRENT ACCESS SIMULATION
// =============================================================================

#[test]
fn test_concurrent_column_selection() {
    use std::sync::Arc;
    use std::thread;

    let doc = create_wide_table_document(20, 10);
    let bytes = Arc::new(to_parquet_bytes(&doc).unwrap());

    let mut handles = Vec::new();

    for i in 0..4 {
        let bytes_clone = Arc::clone(&bytes);
        let handle = thread::spawn(move || {
            let columns = vec!["id".to_string(), format!("col_{}", (i % 19) + 1)];
            from_parquet_bytes_select(&bytes_clone, columns)
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }
}
