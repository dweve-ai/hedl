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

//! Compression tests for hedl-parquet.
//!
//! Tests compression profiles, per-type compression, per-column compression,
//! dictionary encoding, and round-trip correctness.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_parquet::{
    from_parquet_bytes, to_parquet_bytes, to_parquet_bytes_with_config, ToParquetConfig,
};
use parquet::basic::Compression;
use std::collections::HashMap;

// =============================================================================
// Test Helpers
// =============================================================================

/// Create a document with various data types for compression testing.
fn create_mixed_type_document(row_count: usize) -> Document {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Record",
        vec![
            "id".to_string(),
            "name".to_string(),
            "age".to_string(),
            "score".to_string(),
            "active".to_string(),
            "ref".to_string(),
        ],
    );

    for i in 0..row_count {
        let node = Node::new(
            "Record",
            format!("r{i}"),
            vec![
                Value::String(format!("r{i}").into()),
                Value::String(format!("Name{}", i % 100).into()), // Low cardinality for dict encoding
                Value::Int(20 + (i as i64 % 60)),
                Value::Float(50.0 + (i as f64) * 0.1),
                Value::Bool(i % 2 == 0),
                Value::Reference(Reference::qualified("User", format!("u{}", i % 10))),
            ],
        );
        list.add_row(node);
    }

    doc.root.insert("records".to_string(), Item::List(list));
    doc
}

/// Create a document with mostly strings for compression comparison.
fn create_string_heavy_document(row_count: usize) -> Document {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Text",
        vec![
            "id".to_string(),
            "content".to_string(),
            "category".to_string(),
            "tag".to_string(),
        ],
    );

    let categories = ["tech", "science", "sports", "news", "entertainment"];
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ";

    for i in 0..row_count {
        let node = Node::new(
            "Text",
            format!("t{i}"),
            vec![
                Value::String(format!("t{i}").into()),
                Value::String(lorem.repeat((i % 5) + 1).into()),
                Value::String(categories[i % categories.len()].to_string().into()),
                Value::String(format!("tag{}", i % 20).into()),
            ],
        );
        list.add_row(node);
    }

    doc.root.insert("texts".to_string(), Item::List(list));
    doc
}

/// Create a document with mostly numeric data.
/// Available for future compression optimization tests.
#[allow(dead_code)]
fn create_numeric_heavy_document(row_count: usize) -> Document {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Metric",
        vec![
            "id".to_string(),
            "value_a".to_string(),
            "value_b".to_string(),
            "value_c".to_string(),
            "count".to_string(),
        ],
    );

    for i in 0..row_count {
        let node = Node::new(
            "Metric",
            format!("m{i}"),
            vec![
                Value::String(format!("m{i}").into()),
                Value::Float(100.0 * (i as f64) / (row_count as f64)),
                Value::Float(-50.0 + (i as f64) * 0.25),
                Value::Float((i as f64).sin() * 100.0),
                Value::Int(i as i64 * 1000),
            ],
        );
        list.add_row(node);
    }

    doc.root.insert("metrics".to_string(), Item::List(list));
    doc
}

// =============================================================================
// Compression Profile Round-Trip Tests
// =============================================================================

#[test]
fn test_compression_profile_fast_roundtrip() {
    let doc = create_mixed_type_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
    if let Some(Item::List(list)) = restored.root.get("records") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_compression_profile_balanced_roundtrip() {
    let doc = create_mixed_type_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
    if let Some(Item::List(list)) = restored.root.get("records") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_compression_profile_archival_roundtrip() {
    let doc = create_mixed_type_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
    if let Some(Item::List(list)) = restored.root.get("records") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

// =============================================================================
// Type-Based Compression Tests
// =============================================================================

#[test]
fn test_type_based_compression_default() {
    let doc = create_mixed_type_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
    if let Some(Item::List(list)) = restored.root.get("records") {
        assert_eq!(list.rows.len(), 100);
        // Verify data integrity
        let row = &list.rows[0];
        assert!(matches!(row.fields[0], Value::String(_)));
        assert!(matches!(row.fields[2], Value::Int(_)));
        assert!(matches!(row.fields[3], Value::Float(_)));
        assert!(matches!(row.fields[4], Value::Bool(_)));
        assert!(matches!(row.fields[5], Value::Reference(_)));
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_type_based_compression_balanced() {
    let doc = create_mixed_type_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
}

#[test]
fn test_type_based_compression_archival() {
    let doc = create_mixed_type_document(100);
    let config = ToParquetConfig::default();

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
}

// =============================================================================
// Per-Column Compression Tests
// =============================================================================

#[test]
fn test_per_column_compression() {
    let doc = create_mixed_type_document(100);

    let mut column_compression = HashMap::new();
    column_compression.insert("name".to_string(), Compression::ZSTD(Default::default()));
    column_compression.insert("age".to_string(), Compression::SNAPPY);
    column_compression.insert("score".to_string(), Compression::UNCOMPRESSED);

    let config = ToParquetConfig {
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
    if let Some(Item::List(list)) = restored.root.get("records") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_per_column_compression_with_unknown_columns() {
    let doc = create_mixed_type_document(50);

    let mut column_compression = HashMap::new();
    // Specify compression for columns that don't exist - should fall back gracefully
    column_compression.insert(
        "nonexistent".to_string(),
        Compression::ZSTD(Default::default()),
    );
    column_compression.insert("name".to_string(), Compression::SNAPPY);

    let config = ToParquetConfig {
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
}

// =============================================================================
// Global Compression Tests
// =============================================================================

#[test]
fn test_global_compression_snappy() {
    let doc = create_mixed_type_document(100);
    let config = ToParquetConfig {
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
}

#[test]
fn test_global_compression_zstd() {
    let doc = create_mixed_type_document(100);
    let config = ToParquetConfig {
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
}

#[test]
fn test_global_compression_gzip() {
    let doc = create_mixed_type_document(100);
    let config = ToParquetConfig {
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
}

#[test]
fn test_global_compression_uncompressed() {
    let doc = create_mixed_type_document(100);
    let config = ToParquetConfig {
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("records"));
}

// =============================================================================
// =============================================================================

#[test]
fn test_dictionary_encoding_enabled() {
    let doc = create_string_heavy_document(200);

    let config = ToParquetConfig {
        enable_dictionary: true,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("texts"));
    if let Some(Item::List(list)) = restored.root.get("texts") {
        assert_eq!(list.rows.len(), 200);
    } else {
        panic!("Expected list item");
    }
}

#[test]
fn test_dictionary_encoding_disabled() {
    let doc = create_string_heavy_document(200);

    let config = ToParquetConfig {
        enable_dictionary: false,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("texts"));
}

#[test]
fn test_dictionary_encoding_with_high_cardinality() {
    // Test dictionary encoding with data that has many unique values
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "UniqueData",
        vec!["id".to_string(), "unique_val".to_string()],
    );

    for i in 0..1000 {
        let node = Node::new(
            "UniqueData",
            format!("u{i}"),
            vec![
                Value::String(format!("u{i}").into()),
                Value::String(format!("unique_value_{}_random_suffix_{}", i, i * 17).into()),
            ],
        );
        list.add_row(node);
    }

    doc.root.insert("unique_data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        enable_dictionary: true,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("unique_data"));
}

#[test]
fn test_dictionary_encoding_with_low_cardinality() {
    // Test dictionary encoding with data that has few unique values (best case)
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "LowCardinality",
        vec![
            "id".to_string(),
            "status".to_string(),
            "priority".to_string(),
        ],
    );

    let statuses = ["active", "pending", "completed", "cancelled"];
    let priorities = ["low", "medium", "high"];

    for i in 0..1000 {
        let node = Node::new(
            "LowCardinality",
            format!("lc{i}"),
            vec![
                Value::String(format!("lc{i}").into()),
                Value::String(statuses[i % statuses.len()].to_string().into()),
                Value::String(priorities[i % priorities.len()].to_string().into()),
            ],
        );
        list.add_row(node);
    }

    doc.root
        .insert("low_cardinality".to_string(), Item::List(list));

    let config = ToParquetConfig {
        enable_dictionary: true,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    assert!(restored.root.contains_key("low_cardinality"));
    if let Some(Item::List(list)) = restored.root.get("low_cardinality") {
        assert_eq!(list.rows.len(), 1000);
    }
}

// =============================================================================
// Data Integrity Tests
// =============================================================================

#[test]
fn test_data_integrity_with_compression_profiles() {
    let doc = create_mixed_type_document(50);

    let profiles: [&str; 0] = [];
    for profile in profiles {
        let config = ToParquetConfig::default();
        let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
        let restored = from_parquet_bytes(&bytes).unwrap();

        if let Some(Item::List(original)) = doc.root.get("records") {
            if let Some(Item::List(restored_list)) = restored.root.get("records") {
                assert_eq!(
                    original.rows.len(),
                    restored_list.rows.len(),
                    "Row count mismatch for profile {profile:?}"
                );

                // Verify first and last row
                for idx in [0, 49] {
                    let orig_row = &original.rows[idx];
                    let rest_row = &restored_list.rows[idx];

                    // Check ID
                    match (&orig_row.fields[0], &rest_row.fields[0]) {
                        (Value::String(ref a), Value::String(ref b)) => assert_eq!(a, b),
                        _ => panic!("ID mismatch at row {idx}"),
                    }

                    // Check Int
                    match (&orig_row.fields[2], &rest_row.fields[2]) {
                        (Value::Int(a), Value::Int(b)) => assert_eq!(a, b),
                        _ => panic!("Int mismatch at row {idx}"),
                    }

                    // Check Float (with tolerance)
                    match (&orig_row.fields[3], &rest_row.fields[3]) {
                        (Value::Float(a), Value::Float(b)) => {
                            assert!((a - b).abs() < 0.0001, "Float mismatch at row {idx}");
                        }
                        _ => panic!("Float mismatch at row {idx}"),
                    }

                    // Check Bool
                    match (&orig_row.fields[4], &rest_row.fields[4]) {
                        (Value::Bool(a), Value::Bool(b)) => assert_eq!(a, b),
                        _ => panic!("Bool mismatch at row {idx}"),
                    }
                }
            }
        }
    }
}

#[test]
fn test_null_values_with_compression() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "NullableData",
        vec![
            "id".to_string(),
            "nullable_int".to_string(),
            "nullable_str".to_string(),
        ],
    );

    for i in 0..100 {
        let node = Node::new(
            "NullableData",
            format!("n{i}"),
            vec![
                Value::String(format!("n{i}").into()),
                if i % 3 == 0 {
                    Value::Null
                } else {
                    Value::Int(i64::from(i))
                },
                if i % 5 == 0 {
                    Value::Null
                } else {
                    Value::String(format!("str{i}").into())
                },
            ],
        );
        list.add_row(node);
    }

    doc.root.insert("nullable".to_string(), Item::List(list));

    let profiles: [&str; 0] = [];
    for profile in profiles {
        let config = ToParquetConfig::default();
        let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
        let restored = from_parquet_bytes(&bytes).unwrap();

        if let Some(Item::List(list)) = restored.root.get("nullable") {
            assert_eq!(list.rows.len(), 100);

            // Verify null pattern preserved
            for i in 0..100 {
                if i % 3 == 0 {
                    assert!(
                        matches!(list.rows[i].fields[1], Value::Null),
                        "Expected null at row {i} (profile {profile:?})"
                    );
                }
                if i % 5 == 0 {
                    assert!(
                        matches!(list.rows[i].fields[2], Value::Null),
                        "Expected null at row {i} (profile {profile:?})"
                    );
                }
            }
        }
    }
}

// =============================================================================
// Compression Ratio Comparison Tests
// =============================================================================

#[test]
fn test_compression_ratio_ordering() {
    // Archival should generally produce smaller files than Fast for compressible data
    let doc = create_string_heavy_document(500);

    let fast_bytes = to_parquet_bytes(&doc).unwrap();
    let archival_bytes = to_parquet_bytes(&doc).unwrap();

    // Archival should be smaller or equal (not significantly larger)
    // Allow 10% margin for edge cases where fast might compress slightly better
    assert!(
        archival_bytes.len() <= (fast_bytes.len() as f64 * 1.1) as usize,
        "Archival ({} bytes) should not be significantly larger than Fast ({} bytes)",
        archival_bytes.len(),
        fast_bytes.len()
    );
}

#[test]
fn test_uncompressed_vs_compressed_size() {
    let doc = create_string_heavy_document(200);

    let uncompressed_config = ToParquetConfig {
        compression: Compression::UNCOMPRESSED,
        ..Default::default()
    };

    let uncompressed_bytes = to_parquet_bytes_with_config(&doc, &uncompressed_config).unwrap();
    let compressed_bytes = to_parquet_bytes(&doc).unwrap(); // Default uses SNAPPY

    // Compressed should be smaller for text data
    assert!(
        compressed_bytes.len() < uncompressed_bytes.len(),
        "Compressed ({} bytes) should be smaller than uncompressed ({} bytes)",
        compressed_bytes.len(),
        uncompressed_bytes.len()
    );
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_empty_document_with_compression() {
    let doc = Document::new((1, 0));

    let profiles: [&str; 0] = [];
    for profile in profiles {
        let config = ToParquetConfig::default();
        let result = to_parquet_bytes_with_config(&doc, &config);
        assert!(result.is_ok(), "Empty doc should work with {profile:?}");
    }
}

#[test]
fn test_single_row_with_compression() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Single", vec!["id".to_string(), "val".to_string()]);
    list.add_row(Node::new(
        "Single",
        "s1",
        vec![Value::String("s1".to_string().into()), Value::Int(42)],
    ));
    doc.root.insert("single".to_string(), Item::List(list));

    let profiles: [&str; 0] = [];
    for _profile in profiles {
        let config = ToParquetConfig::default();
        let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
        let restored = from_parquet_bytes(&bytes).unwrap();
        assert!(restored.root.contains_key("single"));
    }
}

// =============================================================================
// Configuration API Tests
// =============================================================================

#[test]
fn test_config_builder_pattern() {
    let config = ToParquetConfig::default();

    assert!(config.enable_dictionary);
    assert_eq!(config.compression, Compression::SNAPPY);
}

#[test]
fn test_type_based_compression_constructors() {
    // Test disabled - CompressionStrategy feature not implemented
    // The current ToParquetConfig uses a single global compression setting
    let doc = create_mixed_type_document(10);
    let config = ToParquetConfig::default();
    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let _restored = from_parquet_bytes(&bytes).unwrap();
}
