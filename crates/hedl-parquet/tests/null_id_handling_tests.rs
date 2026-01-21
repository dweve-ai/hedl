// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Comprehensive tests for NullIdHandling configuration options.
// Tests the different strategies for handling null or missing ID values
// when reading Parquet files into HEDL documents.

// Allow approximate float constants in tests - these are intentional test values
#![allow(clippy::approx_constant)]

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{
    from_parquet_bytes, from_parquet_bytes_with_config, to_parquet_bytes, FromParquetConfig,
    NullIdHandling,
};
use proptest::prelude::*;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a document with standard non-null IDs for baseline testing.
fn create_standard_document() -> Document {
    let mut doc = Document::new((1, 0));
    let schema = vec!["id".to_string(), "name".to_string(), "value".to_string()];
    let mut matrix_list = MatrixList::new("Entity", schema.clone());

    matrix_list.add_row(Node::new(
        "Entity",
        "entity1",
        vec![
            Value::String("entity1".to_string().into()),
            Value::String("First Entity".to_string().into()),
            Value::Int(100),
        ],
    ));

    matrix_list.add_row(Node::new(
        "Entity",
        "entity2",
        vec![
            Value::String("entity2".to_string().into()),
            Value::String("Second Entity".to_string().into()),
            Value::Int(200),
        ],
    ));

    matrix_list.add_row(Node::new(
        "Entity",
        "entity3",
        vec![
            Value::String("entity3".to_string().into()),
            Value::String("Third Entity".to_string().into()),
            Value::Int(300),
        ],
    ));

    doc.root
        .insert("entities".to_string(), Item::List(matrix_list));
    doc.structs.insert("Entity".to_string(), schema);
    doc
}

/// Create a document with mixed content including different value types.
fn create_mixed_content_document() -> Document {
    let mut doc = Document::new((1, 0));
    let schema = vec![
        "id".to_string(),
        "int_val".to_string(),
        "float_val".to_string(),
        "bool_val".to_string(),
        "str_val".to_string(),
    ];
    let mut matrix_list = MatrixList::new("MixedData", schema.clone());

    matrix_list.add_row(Node::new(
        "MixedData",
        "row1",
        vec![
            Value::String("row1".to_string().into()),
            Value::Int(42),
            Value::Float(3.14159),
            Value::Bool(true),
            Value::String("hello".to_string().into()),
        ],
    ));

    matrix_list.add_row(Node::new(
        "MixedData",
        "row2",
        vec![
            Value::String("row2".to_string().into()),
            Value::Int(-999),
            Value::Float(-2.71828),
            Value::Bool(false),
            Value::String("world".to_string().into()),
        ],
    ));

    doc.root
        .insert("mixed".to_string(), Item::List(matrix_list));
    doc.structs.insert("MixedData".to_string(), schema);
    doc
}

/// Create a document with nullable fields (not ID, but other columns).
fn create_document_with_nullable_fields() -> Document {
    let mut doc = Document::new((1, 0));
    let schema = vec![
        "id".to_string(),
        "required_field".to_string(),
        "optional_field".to_string(),
    ];
    let mut matrix_list = MatrixList::new("NullableData", schema.clone());

    matrix_list.add_row(Node::new(
        "NullableData",
        "n1",
        vec![
            Value::String("n1".to_string().into()),
            Value::Int(100),
            Value::String("has_value".to_string().into()),
        ],
    ));

    matrix_list.add_row(Node::new(
        "NullableData",
        "n2",
        vec![
            Value::String("n2".to_string().into()),
            Value::Int(200),
            Value::Null, // nullable field
        ],
    ));

    matrix_list.add_row(Node::new(
        "NullableData",
        "n3",
        vec![
            Value::String("n3".to_string().into()),
            Value::Int(300),
            Value::String("another_value".to_string().into()),
        ],
    ));

    doc.root
        .insert("nullable".to_string(), Item::List(matrix_list));
    doc.structs.insert("NullableData".to_string(), schema);
    doc
}

// =============================================================================
// NullIdHandling::Error TESTS (DEFAULT STRICT MODE)
// =============================================================================

#[test]
fn test_error_mode_is_default() {
    let config = FromParquetConfig::default();
    assert!(
        matches!(config.null_id_handling, NullIdHandling::Error),
        "Default should be Error mode"
    );
}

#[test]
fn test_error_mode_accepts_valid_ids() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::default();
    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    // Should succeed with valid IDs
    assert!(!loaded.root.is_empty());
    if let Some(Item::List(list)) = loaded.root.values().next() {
        assert_eq!(list.rows.len(), 3);
    }
}

#[test]
fn test_strict_config_same_as_default() {
    let strict = FromParquetConfig::strict();
    let default = FromParquetConfig::default();

    assert!(
        matches!(strict.null_id_handling, NullIdHandling::Error),
        "Strict config should use Error mode"
    );
    assert!(
        matches!(default.null_id_handling, NullIdHandling::Error),
        "Default config should use Error mode"
    );
}

// =============================================================================
// NullIdHandling::Generate TESTS (LENIENT MODE)
// =============================================================================

#[test]
fn test_generate_mode_via_lenient_config() {
    let config = FromParquetConfig::lenient();
    assert!(
        matches!(config.null_id_handling, NullIdHandling::Generate),
        "Lenient config should use Generate mode"
    );
}

#[test]
fn test_lenient_mode_accepts_valid_ids() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::lenient();
    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    // Should succeed with valid IDs
    assert!(!loaded.root.is_empty());
    if let Some(Item::List(list)) = loaded.root.values().next() {
        assert_eq!(list.rows.len(), 3);
    }
}

#[test]
fn test_with_null_id_handling_builder() {
    // Test the builder pattern
    let config = FromParquetConfig::default().with_null_id_handling(NullIdHandling::Generate);

    assert!(matches!(config.null_id_handling, NullIdHandling::Generate));
}

// =============================================================================
// NullIdHandling::UseConstant TESTS
// =============================================================================

#[test]
fn test_use_constant_mode_construction() {
    let config = FromParquetConfig::default()
        .with_null_id_handling(NullIdHandling::UseConstant("placeholder_id".to_string()));

    match config.null_id_handling {
        NullIdHandling::UseConstant(ref s) => {
            assert_eq!(s, "placeholder_id");
        }
        _ => panic!("Expected UseConstant mode"),
    }
}

#[test]
fn test_use_constant_accepts_valid_ids() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::default()
        .with_null_id_handling(NullIdHandling::UseConstant("default_id".to_string()));

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    // Should succeed with valid IDs (constant not used)
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_use_constant_empty_string() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Empty string as constant
    let config = FromParquetConfig::default()
        .with_null_id_handling(NullIdHandling::UseConstant(String::new()));

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_use_constant_special_characters() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Constant with special characters
    let config = FromParquetConfig::default()
        .with_null_id_handling(NullIdHandling::UseConstant("__null__".to_string()));

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
    assert!(!loaded.root.is_empty());
}

#[test]
fn test_use_constant_unicode() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Unicode constant
    let config = FromParquetConfig::default()
        .with_null_id_handling(NullIdHandling::UseConstant("空值".to_string()));

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
    assert!(!loaded.root.is_empty());
}

// =============================================================================
// CONFIGURATION BUILDER TESTS
// =============================================================================

#[test]
fn test_config_chaining() {
    use hedl_parquet::BatchSize;

    let config = FromParquetConfig::default()
        .with_null_id_handling(NullIdHandling::Generate)
        .with_batch_size(BatchSize::Fixed(1000));

    assert!(matches!(config.null_id_handling, NullIdHandling::Generate));
    assert!(matches!(config.batch_size, BatchSize::Fixed(1000)));
}

#[test]
fn test_config_with_columns_and_null_handling() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::with_columns(vec!["id".to_string(), "name".to_string()])
        .with_null_id_handling(NullIdHandling::Generate);

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
    assert!(!loaded.root.is_empty());
}

// =============================================================================
// NULLABLE NON-ID FIELD TESTS
// =============================================================================

#[test]
fn test_nullable_non_id_fields_preserved_error_mode() {
    let doc = create_document_with_nullable_fields();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::default(); // Error mode

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        // Find the row with null optional_field
        let has_null_field = list
            .rows
            .iter()
            .any(|row| row.fields.iter().any(|f| matches!(f, Value::Null)));
        assert!(has_null_field, "Null non-ID fields should be preserved");
    }
}

#[test]
fn test_nullable_non_id_fields_preserved_generate_mode() {
    let doc = create_document_with_nullable_fields();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::lenient(); // Generate mode

    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        let has_null_field = list
            .rows
            .iter()
            .any(|row| row.fields.iter().any(|f| matches!(f, Value::Null)));
        assert!(
            has_null_field,
            "Null non-ID fields should be preserved in lenient mode"
        );
    }
}

// =============================================================================
// ROUNDTRIP CONSISTENCY TESTS
// =============================================================================

#[test]
fn test_roundtrip_error_mode() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::default();
    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    // Write back
    let bytes2 = to_parquet_bytes(&loaded).unwrap();
    let loaded2 = from_parquet_bytes_with_config(&bytes2, &config).unwrap();

    // Should have same number of items
    assert_eq!(loaded.root.len(), loaded2.root.len());
}

#[test]
fn test_roundtrip_lenient_mode() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::lenient();
    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    // Write back
    let bytes2 = to_parquet_bytes(&loaded).unwrap();
    let loaded2 = from_parquet_bytes_with_config(&bytes2, &config).unwrap();

    // Should have same number of items
    assert_eq!(loaded.root.len(), loaded2.root.len());
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_single_row_valid_id() {
    let mut doc = Document::new((1, 0));
    let schema = vec!["id".to_string(), "value".to_string()];
    let mut matrix_list = MatrixList::new("Single", schema.clone());

    matrix_list.add_row(Node::new(
        "Single",
        "only_row",
        vec![Value::String("only_row".to_string().into()), Value::Int(42)],
    ));

    doc.root
        .insert("single".to_string(), Item::List(matrix_list));

    let bytes = to_parquet_bytes(&doc).unwrap();

    // Test with all modes
    let error_loaded =
        from_parquet_bytes_with_config(&bytes, &FromParquetConfig::default()).unwrap();
    let lenient_loaded =
        from_parquet_bytes_with_config(&bytes, &FromParquetConfig::lenient()).unwrap();

    assert!(!error_loaded.root.is_empty());
    assert!(!lenient_loaded.root.is_empty());
}

#[test]
fn test_many_rows_all_valid_ids() {
    let mut doc = Document::new((1, 0));
    let schema = vec!["id".to_string(), "index".to_string()];
    let mut matrix_list = MatrixList::new("ManyRows", schema.clone());

    for i in 0..100 {
        matrix_list.add_row(Node::new(
            "ManyRows",
            format!("row_{i}"),
            vec![
                Value::String(format!("row_{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }

    doc.root.insert("many".to_string(), Item::List(matrix_list));

    let bytes = to_parquet_bytes(&doc).unwrap();

    // All modes should work
    let error_loaded =
        from_parquet_bytes_with_config(&bytes, &FromParquetConfig::default()).unwrap();
    let lenient_loaded =
        from_parquet_bytes_with_config(&bytes, &FromParquetConfig::lenient()).unwrap();

    if let Some(Item::List(list)) = error_loaded.root.values().next() {
        assert_eq!(list.rows.len(), 100);
    }
    if let Some(Item::List(list)) = lenient_loaded.root.values().next() {
        assert_eq!(list.rows.len(), 100);
    }
}

// =============================================================================
// ID VALUE TYPE TESTS
// =============================================================================

#[test]
fn test_string_id_type() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let loaded = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        for row in &list.rows {
            // First field should be string ID
            assert!(
                matches!(&row.fields[0], Value::String(_)),
                "ID should be string type"
            );
        }
    }
}

#[test]
fn test_id_values_preserved() {
    let doc = create_standard_document();
    let original_ids: Vec<String> = vec![
        "entity1".to_string(),
        "entity2".to_string(),
        "entity3".to_string(),
    ];

    let bytes = to_parquet_bytes(&doc).unwrap();
    let loaded = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        let loaded_ids: Vec<Box<str>> = list
            .rows
            .iter()
            .map(|r| {
                if let Value::String(s) = &r.fields[0] {
                    s.clone()
                } else {
                    String::new().into()
                }
            })
            .collect();

        // All original IDs should be present
        for original_id in &original_ids {
            let id_as_box: Box<str> = original_id.clone().into();
            assert!(
                loaded_ids.contains(&id_as_box),
                "ID '{original_id}' should be preserved"
            );
        }
    }
}

// =============================================================================
// SPECIAL ID VALUE TESTS
// =============================================================================

#[test]
fn test_id_with_special_chars_preserved() {
    let mut doc = Document::new((1, 0));
    let schema = vec!["id".to_string(), "value".to_string()];
    let mut matrix_list = MatrixList::new("Special", schema.clone());

    let special_ids = [
        "id_with_underscore",
        "id123numeric",
        "CamelCaseId",
        "UPPERCASE",
    ];

    for (i, id) in special_ids.iter().enumerate() {
        matrix_list.add_row(Node::new(
            "Special",
            *id,
            vec![
                Value::String((*id).to_string().into()),
                Value::Int(i as i64),
            ],
        ));
    }

    doc.root
        .insert("special".to_string(), Item::List(matrix_list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let loaded = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        assert_eq!(list.rows.len(), special_ids.len());
    }
}

#[test]
fn test_id_with_unicode_preserved() {
    let mut doc = Document::new((1, 0));
    let schema = vec!["id".to_string(), "value".to_string()];
    let mut matrix_list = MatrixList::new("Unicode", schema.clone());

    let unicode_ids = ["id_αβγ", "id_日本語", "id_emoji🎉"];

    for (i, id) in unicode_ids.iter().enumerate() {
        matrix_list.add_row(Node::new(
            "Unicode",
            *id,
            vec![
                Value::String((*id).to_string().into()),
                Value::Int(i as i64),
            ],
        ));
    }

    doc.root
        .insert("unicode".to_string(), Item::List(matrix_list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let loaded = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        assert_eq!(list.rows.len(), unicode_ids.len());
    }
}

#[test]
fn test_long_id_preserved() {
    let mut doc = Document::new((1, 0));
    let schema = vec!["id".to_string(), "value".to_string()];
    let mut matrix_list = MatrixList::new("LongId", schema.clone());

    // Create a very long ID
    let long_id = "x".repeat(1000);
    matrix_list.add_row(Node::new(
        "LongId",
        &long_id,
        vec![Value::String(long_id.clone().into()), Value::Int(42)],
    ));

    doc.root.insert("long".to_string(), Item::List(matrix_list));

    let bytes = to_parquet_bytes(&doc).unwrap();
    let loaded = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        if let Value::String(s) = &list.rows[0].fields[0] {
            assert_eq!(s.len(), 1000);
        }
    }
}

// =============================================================================
// EMPTY STRING ID TESTS
// =============================================================================

#[test]
fn test_empty_string_id() {
    let mut doc = Document::new((1, 0));
    let schema = vec!["id".to_string(), "value".to_string()];
    let mut matrix_list = MatrixList::new("EmptyId", schema.clone());

    // Empty string as ID (not null, just empty)
    matrix_list.add_row(Node::new(
        "EmptyId",
        "",
        vec![Value::String(String::new().into()), Value::Int(42)],
    ));

    doc.root
        .insert("empty_id".to_string(), Item::List(matrix_list));

    let bytes = to_parquet_bytes(&doc).unwrap();

    // Default mode rejects empty string IDs as they can't uniquely identify entities
    let error_result = from_parquet_bytes(&bytes);
    assert!(
        error_result.is_err(),
        "Empty string ID should be rejected in default mode"
    );

    // Lenient mode auto-generates an ID for empty strings
    let config = FromParquetConfig::lenient();
    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        assert_eq!(list.rows.len(), 1);
        // Node's ID should be auto-generated, not empty
        assert!(
            !list.rows[0].id.is_empty(),
            "Empty string ID should be replaced with generated ID in lenient mode, got: '{}'",
            list.rows[0].id
        );
        // The generated ID should follow the expected pattern
        assert!(
            list.rows[0].id.starts_with("__generated_row_"),
            "Generated ID should match expected pattern, got: '{}'",
            list.rows[0].id
        );
    }
}

// =============================================================================
// MIXED CONTENT TYPE TESTS
// =============================================================================

#[test]
fn test_mixed_types_with_error_mode() {
    let doc = create_mixed_content_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::default();
    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        assert_eq!(list.rows.len(), 2);

        // Check types are preserved
        for row in &list.rows {
            assert!(row.fields.iter().any(|f| matches!(f, Value::Int(_))));
            assert!(row.fields.iter().any(|f| matches!(f, Value::Float(_))));
            assert!(row.fields.iter().any(|f| matches!(f, Value::Bool(_))));
        }
    }
}

#[test]
fn test_mixed_types_with_lenient_mode() {
    let doc = create_mixed_content_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    let config = FromParquetConfig::lenient();
    let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();

    if let Some(Item::List(list)) = loaded.root.values().next() {
        assert_eq!(list.rows.len(), 2);
    }
}

// =============================================================================
// PROPERTY-BASED TESTS
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_valid_ids_accepted_all_modes(
        num_rows in 1usize..20,
        id_prefix in "[a-z]{3,10}"
    ) {
        let mut doc = Document::new((1, 0));
        let schema = vec!["id".to_string(), "value".to_string()];
        let mut matrix_list = MatrixList::new("PropTest", schema.clone());

        for i in 0..num_rows {
            let id = format!("{id_prefix}_{i}");
            matrix_list.add_row(Node::new(
                "PropTest",
                &id,
                vec![Value::String(id.clone().into()), Value::Int(i as i64)],
            ));
        }

        doc.root.insert("prop".to_string(), Item::List(matrix_list));

        let bytes_result = to_parquet_bytes(&doc);

        if let Ok(bytes) = bytes_result {
            // Error mode should work
            let error_result = from_parquet_bytes_with_config(&bytes, &FromParquetConfig::default());
            prop_assert!(error_result.is_ok());

            // Lenient mode should work
            let lenient_result = from_parquet_bytes_with_config(&bytes, &FromParquetConfig::lenient());
            prop_assert!(lenient_result.is_ok());

            // UseConstant mode should work
            let const_config = FromParquetConfig::default()
                .with_null_id_handling(NullIdHandling::UseConstant("fallback".to_string()));
            let const_result = from_parquet_bytes_with_config(&bytes, &const_config);
            prop_assert!(const_result.is_ok());
        }
    }

    #[test]
    fn prop_row_count_preserved_all_modes(num_rows in 1usize..50) {
        let mut doc = Document::new((1, 0));
        let schema = vec!["id".to_string(), "index".to_string()];
        let mut matrix_list = MatrixList::new("CountTest", schema.clone());

        for i in 0..num_rows {
            matrix_list.add_row(Node::new(
                "CountTest",
                format!("id_{i}"),
                vec![Value::String(format!("id_{i}").into()), Value::Int(i as i64)],
            ));
        }

        doc.root.insert("count".to_string(), Item::List(matrix_list));

        if let Ok(bytes) = to_parquet_bytes(&doc) {
            let configs = vec![
                FromParquetConfig::default(),
                FromParquetConfig::lenient(),
                FromParquetConfig::default()
                    .with_null_id_handling(NullIdHandling::UseConstant("x".to_string())),
            ];

            for config in configs {
                if let Ok(loaded) = from_parquet_bytes_with_config(&bytes, &config) {
                    if let Some(Item::List(list)) = loaded.root.values().next() {
                        prop_assert_eq!(list.rows.len(), num_rows);
                    }
                }
            }
        }
    }

    #[test]
    fn prop_constant_id_preserved_in_config(constant_id in "[a-z0-9_]{1,50}") {
        let config = FromParquetConfig::default()
            .with_null_id_handling(NullIdHandling::UseConstant(constant_id.clone()));

        match config.null_id_handling {
            NullIdHandling::UseConstant(ref s) => prop_assert_eq!(s, &constant_id),
            _ => prop_assert!(false, "Expected UseConstant mode"),
        }
    }
}

// =============================================================================
// CONCURRENT ACCESS TESTS
// =============================================================================

#[test]
fn test_concurrent_reads_different_modes() {
    use std::sync::Arc;
    use std::thread;

    let doc = create_standard_document();
    let bytes = Arc::new(to_parquet_bytes(&doc).unwrap());

    let modes = vec![
        FromParquetConfig::default(),
        FromParquetConfig::lenient(),
        FromParquetConfig::default()
            .with_null_id_handling(NullIdHandling::UseConstant("const".to_string())),
    ];

    let mut handles = Vec::new();

    for config in modes {
        let bytes_clone = Arc::clone(&bytes);
        let handle = thread::spawn(move || from_parquet_bytes_with_config(&bytes_clone, &config));
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }
}

// =============================================================================
// STRESS TESTS
// =============================================================================

#[test]
fn test_large_document_all_modes() {
    let mut doc = Document::new((1, 0));
    let schema = vec![
        "id".to_string(),
        "col1".to_string(),
        "col2".to_string(),
        "col3".to_string(),
    ];
    let mut matrix_list = MatrixList::new("Large", schema.clone());

    for i in 0..500 {
        matrix_list.add_row(Node::new(
            "Large",
            format!("item_{i}"),
            vec![
                Value::String(format!("item_{i}").into()),
                Value::Int(i64::from(i)),
                Value::Float(f64::from(i) * 1.5),
                Value::String(format!("data_{i}").into()),
            ],
        ));
    }

    doc.root
        .insert("large".to_string(), Item::List(matrix_list));

    let bytes = to_parquet_bytes(&doc).unwrap();

    // Test all modes handle large documents
    let error_loaded =
        from_parquet_bytes_with_config(&bytes, &FromParquetConfig::default()).unwrap();
    let lenient_loaded =
        from_parquet_bytes_with_config(&bytes, &FromParquetConfig::lenient()).unwrap();

    if let Some(Item::List(list)) = error_loaded.root.values().next() {
        assert_eq!(list.rows.len(), 500);
    }
    if let Some(Item::List(list)) = lenient_loaded.root.values().next() {
        assert_eq!(list.rows.len(), 500);
    }
}

#[test]
fn test_repeated_config_changes() {
    let doc = create_standard_document();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Repeatedly change configuration and read
    for i in 0..10 {
        let config = match i % 3 {
            0 => FromParquetConfig::default(),
            1 => FromParquetConfig::lenient(),
            _ => FromParquetConfig::default()
                .with_null_id_handling(NullIdHandling::UseConstant(format!("const_{i}"))),
        };

        let loaded = from_parquet_bytes_with_config(&bytes, &config).unwrap();
        assert!(!loaded.root.is_empty());
    }
}

// =============================================================================
// CLONE AND DEBUG TRAIT TESTS
// =============================================================================

#[test]
fn test_null_id_handling_debug() {
    let error = NullIdHandling::Error;
    let generate = NullIdHandling::Generate;
    let constant = NullIdHandling::UseConstant("test".to_string());

    // Debug should produce meaningful output
    let error_debug = format!("{error:?}");
    let generate_debug = format!("{generate:?}");
    let constant_debug = format!("{constant:?}");

    assert!(error_debug.contains("Error"));
    assert!(generate_debug.contains("Generate"));
    assert!(constant_debug.contains("UseConstant"));
    assert!(constant_debug.contains("test"));
}

#[test]
fn test_null_id_handling_clone() {
    let original = NullIdHandling::UseConstant("value".to_string());
    let cloned = original.clone();

    match (original, cloned) {
        (NullIdHandling::UseConstant(ref o), NullIdHandling::UseConstant(ref c)) => {
            assert_eq!(o, c);
        }
        _ => panic!("Clone should preserve variant"),
    }
}

#[test]
fn test_from_parquet_config_clone() {
    use hedl_parquet::BatchSize;

    let config = FromParquetConfig::default()
        .with_null_id_handling(NullIdHandling::UseConstant("value".to_string()))
        .with_batch_size(BatchSize::Fixed(5000))
        .set_columns(Some(vec!["col1".to_string(), "col2".to_string()]));

    let cloned = config.clone();

    // Verify clone preserves all fields
    match (&config.null_id_handling, &cloned.null_id_handling) {
        (NullIdHandling::UseConstant(ref o), NullIdHandling::UseConstant(ref c)) => {
            assert_eq!(o, c);
        }
        _ => panic!("Null handling clone failed"),
    }

    assert_eq!(config.columns, cloned.columns);
    assert!(matches!(cloned.batch_size, BatchSize::Fixed(5000)));
}
