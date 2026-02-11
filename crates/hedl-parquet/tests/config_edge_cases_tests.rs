// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Configuration edge case tests for hedl-parquet
//!
//! Tests `ToParquetConfig` and `FromParquetConfig` edge cases, including statistics,
//! dictionary encoding, and batch size configurations.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_parquet::{
    from_parquet_bytes, to_parquet_bytes_with_config, BatchSize, EnabledStatistics,
    FromParquetConfig, ToParquetConfig,
};
use parquet::basic::Compression;
use parquet::file::properties::WriterVersion;

// =============================================================================
// ToParquetConfig Tests
// =============================================================================

#[test]
fn test_default_config() {
    let config = ToParquetConfig::default();
    assert!(matches!(config.compression, Compression::SNAPPY));
    assert!(matches!(config.writer_version, WriterVersion::PARQUET_2_0));
    assert!(config.enable_dictionary);
    assert!(matches!(config.statistics, EnabledStatistics::Chunk));
    assert!(!config.coerce_types);
}

#[test]
fn test_config_without_statistics() {
    let config = ToParquetConfig::without_statistics();
    assert!(matches!(config.statistics, EnabledStatistics::None));
}

#[test]
fn test_config_with_statistics_page() {
    let config = ToParquetConfig::default().with_statistics(EnabledStatistics::Page);
    assert!(matches!(config.statistics, EnabledStatistics::Page));
}

#[test]
fn test_config_with_type_coercion_enabled() {
    let config = ToParquetConfig::default().with_type_coercion(true);
    assert!(config.coerce_types);
}

#[test]
fn test_config_with_type_coercion_disabled() {
    let config = ToParquetConfig::default().with_type_coercion(false);
    assert!(!config.coerce_types);
}

// =============================================================================
// Compression Tests
// =============================================================================

#[test]
fn test_compression_uncompressed() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Data",
        "d1",
        vec![Value::String("d1".to_string().into()), Value::Int(42)],
    ));
    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        compression: Compression::UNCOMPRESSED,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    // Verify round-trip
    let restored = from_parquet_bytes(&bytes).unwrap();
    assert!(restored.root.contains_key("data"));
}

#[test]
fn test_compression_gzip() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Data",
        "d1",
        vec![Value::String("d1".to_string().into()), Value::Int(42)],
    ));
    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        compression: Compression::GZIP(Default::default()),
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let restored = from_parquet_bytes(&bytes).unwrap();
    assert!(restored.root.contains_key("data"));
}

#[test]
fn test_compression_zstd() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Data",
        "d1",
        vec![Value::String("d1".to_string().into()), Value::Int(42)],
    ));
    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        compression: Compression::ZSTD(Default::default()),
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let restored = from_parquet_bytes(&bytes).unwrap();
    assert!(restored.root.contains_key("data"));
}

#[test]
fn test_compression_lz4() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Data",
        "d1",
        vec![Value::String("d1".to_string().into()), Value::Int(42)],
    ));
    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        compression: Compression::LZ4,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    assert!(!bytes.is_empty());

    let restored = from_parquet_bytes(&bytes).unwrap();
    assert!(restored.root.contains_key("data"));
}

// =============================================================================
// Dictionary Encoding Tests
// =============================================================================

#[test]
fn test_dictionary_encoding_enabled() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "category".to_string()]);

    // Repeated values benefit from dictionary encoding
    for i in 0..100 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::String("category_a".to_string().into()),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        enable_dictionary: true,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();

    let restored = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 100);
    }
}

#[test]
fn test_dictionary_encoding_disabled() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "category".to_string()]);

    for i in 0..50 {
        list.add_row(Node::new(
            "Data",
            format!("d{i}"),
            vec![
                Value::String(format!("d{i}").into()),
                Value::String("category_a".to_string().into()),
            ],
        ));
    }

    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        enable_dictionary: false,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();

    let restored = from_parquet_bytes(&bytes).unwrap();
    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 50);
    }
}

// =============================================================================
// WriterVersion Tests
// =============================================================================

#[test]
fn test_writer_version_1_0() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string()]);
    list.add_row(Node::new(
        "Data",
        "d1",
        vec![Value::String("d1".to_string().into())],
    ));
    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        writer_version: WriterVersion::PARQUET_1_0,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();
    assert!(restored.root.contains_key("data"));
}

#[test]
fn test_writer_version_2_0() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string()]);
    list.add_row(Node::new(
        "Data",
        "d1",
        vec![Value::String("d1".to_string().into())],
    ));
    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        writer_version: WriterVersion::PARQUET_2_0,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();
    assert!(restored.root.contains_key("data"));
}

// =============================================================================
// Statistics Level Tests
// =============================================================================

#[test]
fn test_statistics_none() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..10 {
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
        statistics: EnabledStatistics::None,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 10);
    }
}

#[test]
fn test_statistics_chunk() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..10 {
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
        statistics: EnabledStatistics::Chunk,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 10);
    }
}

#[test]
fn test_statistics_page() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..10 {
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
        statistics: EnabledStatistics::Page,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 10);
    }
}

// =============================================================================
// FromParquetConfig Tests
// =============================================================================

#[test]
fn test_from_config_default() {
    let config = FromParquetConfig::default();
    assert!(matches!(
        config.null_id_handling,
        hedl_parquet::NullIdHandling::Error
    ));
    assert!(config.columns.is_none());
    assert!(matches!(config.batch_size, BatchSize::Auto));
    assert!(config.filter.is_none());
}

#[test]
fn test_from_config_lenient() {
    let config = FromParquetConfig::lenient();
    assert!(matches!(
        config.null_id_handling,
        hedl_parquet::NullIdHandling::Generate
    ));
}

#[test]
fn test_from_config_strict() {
    let config = FromParquetConfig::strict();
    assert!(matches!(
        config.null_id_handling,
        hedl_parquet::NullIdHandling::Error
    ));
}

#[test]
fn test_from_config_with_column() {
    let config = FromParquetConfig::with_column("id".into());
    assert!(config.columns.is_some());
    assert_eq!(config.columns.unwrap().len(), 1);
}

#[test]
fn test_from_config_with_columns() {
    let config = FromParquetConfig::with_columns(vec!["id".into(), "name".into()]);
    assert!(config.columns.is_some());
    assert_eq!(config.columns.unwrap().len(), 2);
}

#[test]
fn test_from_config_set_columns() {
    let config = FromParquetConfig::default().set_columns(Some(vec!["a".into(), "b".into()]));
    assert!(config.columns.is_some());
}

// =============================================================================
// BatchSize Tests
// =============================================================================

#[test]
fn test_batch_size_constants() {
    assert_eq!(BatchSize::MIN_BATCH_SIZE, 100);
    assert_eq!(BatchSize::MAX_BATCH_SIZE, 1_048_576);
    assert_eq!(BatchSize::DEFAULT_NARROW_BATCH_SIZE, 65_536);
    assert_eq!(BatchSize::DEFAULT_MEDIUM_BATCH_SIZE, 32_768);
    assert_eq!(BatchSize::DEFAULT_WIDE_BATCH_SIZE, 16_384);
    assert_eq!(BatchSize::DEFAULT_ADAPTIVE_BATCH_SIZE, 32_768);
}

#[test]
fn test_batch_size_validate_min() {
    assert_eq!(BatchSize::validate(50), BatchSize::MIN_BATCH_SIZE);
}

#[test]
fn test_batch_size_validate_max() {
    assert_eq!(BatchSize::validate(2_000_000), BatchSize::MAX_BATCH_SIZE);
}

#[test]
fn test_batch_size_validate_in_range() {
    assert_eq!(BatchSize::validate(10_000), 10_000);
}

#[test]
fn test_batch_size_auto_narrow() {
    let size = BatchSize::calculate_auto_size(10, false);
    assert_eq!(size, BatchSize::DEFAULT_NARROW_BATCH_SIZE);
}

#[test]
fn test_batch_size_auto_medium() {
    let size = BatchSize::calculate_auto_size(30, false);
    assert_eq!(size, BatchSize::DEFAULT_MEDIUM_BATCH_SIZE);
}

#[test]
fn test_batch_size_auto_wide() {
    let size = BatchSize::calculate_auto_size(60, false);
    assert_eq!(size, BatchSize::DEFAULT_WIDE_BATCH_SIZE);
}

#[test]
fn test_batch_size_auto_with_strings() {
    let size = BatchSize::calculate_auto_size(10, true);
    assert_eq!(size, BatchSize::DEFAULT_NARROW_BATCH_SIZE / 2);
}

#[test]
fn test_batch_size_get_effective_auto() {
    let batch_size = BatchSize::Auto;
    let size = batch_size.get_effective_size(10, false);
    assert_eq!(size, BatchSize::DEFAULT_NARROW_BATCH_SIZE);
}

#[test]
fn test_batch_size_get_effective_fixed() {
    let batch_size = BatchSize::Fixed(10_000);
    let size = batch_size.get_effective_size(10, false);
    assert_eq!(size, 10_000);
}

#[test]
fn test_batch_size_get_effective_fixed_out_of_range() {
    let batch_size = BatchSize::Fixed(50);
    let size = batch_size.get_effective_size(10, false);
    assert_eq!(size, BatchSize::MIN_BATCH_SIZE);
}

#[test]
fn test_batch_size_get_effective_adaptive() {
    let batch_size = BatchSize::Adaptive(20_000);
    let size = batch_size.get_effective_size(10, false);
    assert_eq!(size, 20_000);
}

#[test]
fn test_from_config_with_batch_size_auto() {
    let config = FromParquetConfig::default().with_batch_size(BatchSize::Auto);
    assert!(matches!(config.batch_size, BatchSize::Auto));
}

#[test]
fn test_from_config_with_batch_size_fixed() {
    let config = FromParquetConfig::default().with_batch_size(BatchSize::Fixed(5000));
    assert!(matches!(config.batch_size, BatchSize::Fixed(5000)));
}

#[test]
fn test_from_config_with_batch_size_adaptive() {
    let config = FromParquetConfig::default().with_batch_size(BatchSize::Adaptive(8192));
    assert!(matches!(config.batch_size, BatchSize::Adaptive(8192)));
}

// =============================================================================
// Combined Config Tests
// =============================================================================

#[test]
fn test_combined_compression_and_statistics() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

    for i in 0..100 {
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
        statistics: EnabledStatistics::Page,
        enable_dictionary: true,
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();

    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 100);
    }
}

#[test]
fn test_combined_config_all_options() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Data",
        "d1",
        vec![Value::String("d1".to_string().into()), Value::Int(42)],
    ));
    doc.root.insert("data".to_string(), Item::List(list));

    let config = ToParquetConfig {
        compression: Compression::GZIP(Default::default()),
        writer_version: WriterVersion::PARQUET_2_0,
        enable_dictionary: false,
        statistics: EnabledStatistics::None,
        coerce_types: true,
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let restored = from_parquet_bytes(&bytes).unwrap();
    assert!(restored.root.contains_key("data"));
}

// =============================================================================
// Config Clone Tests
// =============================================================================

#[test]
fn test_to_config_clone() {
    let config1 = ToParquetConfig::default();
    let config2 = config1.clone();

    // Both should be equivalent
    assert!(matches!(config1.compression, Compression::SNAPPY));
    assert!(matches!(config2.compression, Compression::SNAPPY));
}

#[test]
fn test_from_config_clone() {
    let config1 = FromParquetConfig::default();
    let config2 = config1.clone();

    assert!(matches!(config1.batch_size, BatchSize::Auto));
    assert!(matches!(config2.batch_size, BatchSize::Auto));
}

// =============================================================================
// Batch Size Equality Tests
// =============================================================================

#[test]
fn test_batch_size_equality() {
    assert_eq!(BatchSize::Auto, BatchSize::Auto);
    assert_eq!(BatchSize::Fixed(1000), BatchSize::Fixed(1000));
    assert_ne!(BatchSize::Fixed(1000), BatchSize::Fixed(2000));
    assert_eq!(BatchSize::Adaptive(5000), BatchSize::Adaptive(5000));
}
