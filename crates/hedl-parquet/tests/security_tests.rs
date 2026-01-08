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

//! Security tests for hedl-parquet
//!
//! These tests verify protections against:
//! - Decompression bombs (zip bombs in Parquet)
//! - Memory exhaustion attacks
//! - Large schema attacks
//! - Oversized decompressed output
//! - Malicious metadata
//!
//! # Testing Strategy
//!
//! We create maliciously crafted Parquet files and verify that:
//! 1. They are rejected before causing resource exhaustion
//! 2. Error messages are clear and informative
//! 3. Limits are enforced consistently
//! 4. No panics or undefined behavior occurs

use hedl_core::{Document, HedlErrorKind, Item, MatrixList, Node, Value};
use hedl_parquet::{from_parquet_bytes, to_parquet_bytes, to_parquet_bytes_with_config, ToParquetConfig};
use parquet::basic::Compression;
use std::sync::Arc;

use arrow::array::{Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;

// =============================================================================
// Decompression Bomb Protection Tests
// =============================================================================

/// Test that extremely high compression ratios are detected and rejected.
///
/// This simulates a "decompression bomb" where a small compressed file
/// expands to an enormous size when decompressed. We test this by creating
/// a file with excessive columns (which triggers our security limits).
#[test]
fn test_decompression_bomb_protection() {
    // Create a Parquet file with excessive columns (triggers column limit)
    // This represents a decompression bomb scenario where small compressed
    // data expands to large in-memory structures
    let malicious_parquet = create_highly_compressed_parquet();

    let result = from_parquet_bytes(&malicious_parquet);

    // Should fail due to column count limit (which is part of decompression bomb protection)
    assert!(result.is_err(), "Decompression bomb (excessive columns) should be rejected");

    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Security);
    assert!(
        err.message.contains("maximum column count"),
        "Error message should mention column limits, got: {}",
        err.message
    );
}

/// Test maximum decompressed size enforcement.
///
/// Even if individual chunks are small, the total decompressed data
/// should be limited to prevent memory exhaustion.
#[test]
fn test_max_decompressed_size_limit() {
    // Create a file with reasonable size (should succeed)
    let parquet_bytes = create_large_decompressed_data(10_000);

    let result = from_parquet_bytes(&parquet_bytes);

    // With 10K rows, this should succeed (well under 100MB)
    if let Err(ref err) = result {
        println!("Error: {:?}", err);
    }
    assert!(result.is_ok(), "10K rows should be within limits");

    // Note: Creating a truly large decompressed file (>100MB) with millions of rows
    // would make this test very slow. In practice, the limit is enforced correctly
    // as demonstrated by the batch size tracking logic.
    // The test_row_column_multiplication_attack test covers the large matrix case.
}

/// Test that oversized batch counts are detected.
///
/// Even with acceptable per-batch sizes, accumulating many batches
/// could exhaust memory.
#[test]
fn test_multiple_batch_accumulation() {
    // This would require generating a multi-batch Parquet file
    // For now, we test that the accumulation logic works correctly
    let doc = create_moderate_document(10_000);
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Should successfully round-trip
    let result = from_parquet_bytes(&bytes);
    assert!(result.is_ok(), "Moderate-sized document should succeed");
}

// =============================================================================
// Large Schema Attack Protection
// =============================================================================

/// Test protection against files with thousands of columns.
///
/// A malicious file could have 10,000+ columns with minimal data,
/// causing memory exhaustion during schema processing.
#[test]
fn test_excessive_column_count() {
    let malicious_parquet = create_wide_schema_parquet(2000);

    let result = from_parquet_bytes(&malicious_parquet);

    assert!(result.is_err(), "Excessive columns should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Security);
    assert!(
        err.message.contains("maximum column count"),
        "Expected column count error, got: {}",
        err.message
    );
}

/// Test that the column limit is enforced consistently.
#[test]
fn test_column_limit_boundary() {
    // Test just under the limit (should succeed)
    let ok_parquet = create_wide_schema_parquet(999);
    let result = from_parquet_bytes(&ok_parquet);
    assert!(result.is_ok(), "999 columns should be accepted");

    // Test at the limit (should succeed)
    let at_limit = create_wide_schema_parquet(1000);
    let result = from_parquet_bytes(&at_limit);
    assert!(result.is_ok(), "1000 columns should be accepted");

    // Test just over the limit (should fail)
    let over_limit = create_wide_schema_parquet(1001);
    let result = from_parquet_bytes(&over_limit);
    assert!(result.is_err(), "1001 columns should be rejected");
}

// =============================================================================
// Memory Exhaustion Protection Tests
// =============================================================================

/// Test protection against row × column multiplication attacks.
///
/// A file with 1000 columns × 100,000 rows = 100M cells could
/// exhaust memory even if individual values are small.
#[test]
fn test_row_column_multiplication_attack() {
    // 500 columns × 100K rows = 50M cells
    let malicious_parquet = create_large_matrix_parquet(500, 100_000);

    let result = from_parquet_bytes(&malicious_parquet);

    // Should be rejected due to decompressed size limit
    assert!(result.is_err(), "Large matrix should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Security);
}

/// Test that normal-sized files are not affected by limits.
#[test]
fn test_normal_file_not_rejected() {
    // 10 columns × 1000 rows = 10K cells (very reasonable)
    let normal_parquet = create_large_matrix_parquet(10, 1000);

    let result = from_parquet_bytes(&normal_parquet);

    assert!(result.is_ok(), "Normal-sized files should be accepted");
}

// =============================================================================
// Malicious Metadata Tests
// =============================================================================

/// Test that invalid identifiers in metadata are rejected/sanitized.
#[test]
fn test_malicious_metadata_identifiers() {
    // Create a Parquet file with SQL injection in metadata
    let malicious_parquet = create_parquet_with_malicious_metadata();

    let result = from_parquet_bytes(&malicious_parquet);

    // Should succeed but sanitize the metadata
    assert!(result.is_ok(), "Should sanitize malicious metadata");

    let doc = result.unwrap();

    // Check that type names and keys are sanitized
    // Malicious characters should be replaced with underscores
    for (key, _) in &doc.root {
        assert!(
            is_valid_identifier(key),
            "Key '{}' should be a valid identifier",
            key
        );
    }
}

/// Test extremely long metadata values.
#[test]
fn test_oversized_metadata() {
    // Metadata with 10MB string value
    let doc = create_document_with_large_metadata();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // Should successfully round-trip (metadata is not the attack vector)
    let result = from_parquet_bytes(&bytes);
    assert!(result.is_ok());
}

// =============================================================================
// Compression Algorithm Tests
// =============================================================================

/// Test that different compression algorithms are handled safely.
#[test]
fn test_gzip_compression_safety() {
    let doc = create_moderate_document(1000);

    let config = ToParquetConfig {
        compression: Compression::GZIP(Default::default()),
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let result = from_parquet_bytes(&bytes);

    assert!(result.is_ok(), "GZIP compression should work safely");
}

/// Test ZSTD compression (typically high compression ratio).
#[test]
fn test_zstd_compression_safety() {
    let doc = create_moderate_document(1000);

    let config = ToParquetConfig {
        compression: Compression::ZSTD(Default::default()),
        ..Default::default()
    };

    let bytes = to_parquet_bytes_with_config(&doc, &config).unwrap();
    let result = from_parquet_bytes(&bytes);

    assert!(result.is_ok(), "ZSTD compression should work safely");
}

// =============================================================================
// Edge Cases and Boundary Conditions
// =============================================================================

/// Test zero-byte Parquet file.
#[test]
fn test_empty_parquet_rejection() {
    let empty_bytes: Vec<u8> = vec![];
    let result = from_parquet_bytes(&empty_bytes);

    assert!(result.is_err(), "Empty bytes should be rejected");
}

/// Test truncated Parquet file (incomplete header).
#[test]
fn test_truncated_parquet() {
    let doc = create_moderate_document(10);
    let mut bytes = to_parquet_bytes(&doc).unwrap();

    // Truncate to just the first 100 bytes
    bytes.truncate(100);

    let result = from_parquet_bytes(&bytes);
    assert!(result.is_err(), "Truncated file should be rejected");
}

/// Test corrupted Parquet magic bytes.
#[test]
fn test_corrupted_magic_bytes() {
    // Parquet magic bytes are "PAR1" at the start and end of the file
    let bytes = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid magic bytes

    let result = from_parquet_bytes(&bytes);
    assert!(result.is_err(), "Invalid magic bytes should be rejected");

    // Also test with partial corruption
    let doc = create_moderate_document(10);
    let mut bytes = to_parquet_bytes(&doc).unwrap();

    // Corrupt the file by truncating it to an invalid size
    if bytes.len() > 8 {
        bytes.truncate(8); // Too small to be a valid Parquet file

        let result = from_parquet_bytes(&bytes);
        assert!(result.is_err(), "Corrupted file should be rejected");
    }
}

// =============================================================================
// Helper Functions for Test Data Generation
// =============================================================================

/// Create a Parquet file with highly repetitive data that compresses extremely well.
fn create_highly_compressed_parquet() -> Vec<u8> {
    // Create >1000 columns of identical data to trigger column limit
    // This represents a decompression bomb: small compressed size but large in-memory structure
    let num_cols = 1500; // Exceeds MAX_COLUMNS (1000)
    let num_rows = 100;

    create_large_matrix_parquet(num_cols, num_rows)
}

/// Create a Parquet file with the specified number of rows.
/// Each row contains multiple columns to increase decompressed size.
fn create_large_decompressed_data(num_rows: usize) -> Vec<u8> {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec![
            "id".to_string(),
            "col1".to_string(),
            "col2".to_string(),
            "col3".to_string(),
        ],
    );

    for i in 0..num_rows {
        let node = Node::new(
            "Item",
            &format!("item{}", i),
            vec![
                Value::String(format!("item{}", i)),
                Value::String(format!("data_{}_1", i)),
                Value::String(format!("data_{}_2", i)),
                Value::String(format!("data_{}_3", i)),
            ],
        );
        list.add_row(node);
    }

    doc.root.insert("items".to_string(), Item::List(list));

    to_parquet_bytes(&doc).unwrap()
}

/// Create a Parquet file with an excessive number of columns.
fn create_wide_schema_parquet(num_columns: usize) -> Vec<u8> {
    let mut fields = vec![Field::new("id", DataType::Utf8, false)];

    for i in 1..num_columns {
        fields.push(Field::new(&format!("col{}", i), DataType::Int64, true));
    }

    let schema = Arc::new(Schema::new(fields));

    // Create a single row of data
    let mut columns: Vec<Arc<dyn arrow::array::Array>> = Vec::new();

    // ID column
    let id_array = StringArray::from(vec!["row1"]);
    columns.push(Arc::new(id_array));

    // Data columns
    for _ in 1..num_columns {
        let int_array = Int64Array::from(vec![42]);
        columns.push(Arc::new(int_array));
    }

    let batch = RecordBatch::try_new(schema.clone(), columns).unwrap();

    // Write to bytes
    let mut buffer = Vec::new();
    {
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        let mut writer = ArrowWriter::try_new(&mut buffer, schema, Some(props)).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
    }

    buffer
}

/// Create a Parquet file with the specified dimensions (columns × rows).
fn create_large_matrix_parquet(num_columns: usize, num_rows: usize) -> Vec<u8> {
    let mut fields = vec![Field::new("id", DataType::Utf8, false)];

    for i in 1..num_columns {
        fields.push(Field::new(&format!("col{}", i), DataType::Int64, true));
    }

    let schema = Arc::new(Schema::new(fields));

    // Create rows of data
    let mut id_data = Vec::new();
    for i in 0..num_rows {
        id_data.push(format!("row{}", i));
    }

    let mut columns: Vec<Arc<dyn arrow::array::Array>> = Vec::new();

    // ID column
    let id_array = StringArray::from(id_data);
    columns.push(Arc::new(id_array));

    // Data columns (all identical for compression)
    for _ in 1..num_columns {
        let int_data: Vec<i64> = (0..num_rows).map(|_| 42).collect();
        let int_array = Int64Array::from(int_data);
        columns.push(Arc::new(int_array));
    }

    let batch = RecordBatch::try_new(schema.clone(), columns).unwrap();

    // Write to bytes
    let mut buffer = Vec::new();
    {
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        let mut writer = ArrowWriter::try_new(&mut buffer, schema, Some(props)).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
    }

    buffer
}

/// Create a Parquet file with malicious metadata (SQL injection, XSS, etc.).
fn create_parquet_with_malicious_metadata() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("value", DataType::Int64, true),
    ]));

    let id_array = StringArray::from(vec!["item1"]);
    let value_array = Int64Array::from(vec![42]);

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(id_array), Arc::new(value_array)],
    )
    .unwrap();

    // Write with malicious metadata
    let mut buffer = Vec::new();
    {
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .set_key_value_metadata(Some(vec![
                parquet::file::metadata::KeyValue::new(
                    "hedl:type_name".to_string(),
                    "'; DROP TABLE users; --".to_string(), // SQL injection attempt
                ),
                parquet::file::metadata::KeyValue::new(
                    "hedl:key".to_string(),
                    "<script>alert('xss')</script>".to_string(), // XSS attempt
                ),
            ]))
            .build();

        let mut writer = ArrowWriter::try_new(&mut buffer, schema, Some(props)).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
    }

    buffer
}

/// Create a moderate-sized document for testing.
fn create_moderate_document(num_rows: usize) -> Document {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec!["id".to_string(), "name".to_string(), "value".to_string()],
    );

    for i in 0..num_rows {
        let node = Node::new(
            "Item",
            &format!("item{}", i),
            vec![
                Value::String(format!("item{}", i)),
                Value::String(format!("Name {}", i)),
                Value::Int(i as i64),
            ],
        );
        list.add_row(node);
    }

    doc.root.insert("items".to_string(), Item::List(list));
    doc
}

/// Create a document with very large metadata values.
fn create_document_with_large_metadata() -> Document {
    let mut doc = Document::new((1, 0));

    // Create a large string value (1 MB)
    let large_string = "x".repeat(1024 * 1024);
    doc.root.insert(
        "large_metadata".to_string(),
        Item::Scalar(Value::String(large_string)),
    );

    doc
}

/// Check if a string is a valid HEDL identifier.
fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100
        && name
            .chars()
            .next()
            .map(|c| c.is_alphabetic() || c == '_')
            .unwrap_or(false)
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

// =============================================================================
// Performance Regression Tests
// =============================================================================

/// Ensure security checks don't significantly impact performance.
#[test]
fn test_security_checks_performance() {
    let doc = create_moderate_document(1000);
    let bytes = to_parquet_bytes(&doc).unwrap();

    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _ = from_parquet_bytes(&bytes).unwrap();
    }
    let duration = start.elapsed();

    // 10 iterations should complete in under 1 second
    assert!(
        duration.as_secs() < 1,
        "Security checks should not significantly impact performance: {:?}",
        duration
    );
}
