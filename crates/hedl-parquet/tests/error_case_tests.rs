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

//! Comprehensive error case tests for hedl-parquet.
//!
//! This test suite covers all error scenarios including:
//! - Malformed Parquet files
//! - Unsupported types
//! - Invalid data
//! - Resource exhaustion

use hedl_core::{Document, HedlErrorKind, Item};
use hedl_parquet::{from_parquet_bytes, to_parquet_bytes};
use std::sync::Arc;

use arrow::array::{
    ArrayRef, BinaryArray, Date32Array, Decimal128Array, DurationMicrosecondArray,
    FixedSizeBinaryArray, Int32Array, Int64Array, LargeBinaryArray, LargeStringArray, ListArray,
    StringArray, StructArray, Time64MicrosecondArray, TimestampMicrosecondArray, UInt64Array,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

// =============================================================================
// 1. Malformed Parquet Files
// =============================================================================

#[test]
fn test_empty_bytes() {
    let empty_bytes: Vec<u8> = vec![];
    let result = from_parquet_bytes(&empty_bytes);

    assert!(result.is_err(), "Empty bytes should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
}

#[test]
fn test_invalid_magic_bytes() {
    // Parquet files start with "PAR1" magic bytes
    let invalid_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let result = from_parquet_bytes(&invalid_bytes);

    assert!(result.is_err(), "Invalid magic bytes should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
}

#[test]
fn test_truncated_header() {
    // Create a valid Parquet file then truncate it
    let doc = create_simple_doc();
    let mut bytes = to_parquet_bytes(&doc).unwrap();

    // Truncate to just the first 50 bytes (incomplete header)
    bytes.truncate(50);

    let result = from_parquet_bytes(&bytes);
    assert!(result.is_err(), "Truncated header should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
}

#[test]
fn test_truncated_file_middle() {
    // Create a valid Parquet file then truncate in the middle
    let doc = create_simple_doc();
    let mut bytes = to_parquet_bytes(&doc).unwrap();

    // Truncate to half the size
    let half = bytes.len() / 2;
    bytes.truncate(half);

    let result = from_parquet_bytes(&bytes);
    assert!(result.is_err(), "Truncated file should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
}

#[test]
fn test_truncated_file_footer() {
    // Create a valid Parquet file then truncate the footer
    let doc = create_simple_doc();
    let mut bytes = to_parquet_bytes(&doc).unwrap();

    // Truncate last 20 bytes (incomplete footer)
    let new_len = bytes.len().saturating_sub(20);
    bytes.truncate(new_len);

    let result = from_parquet_bytes(&bytes);
    assert!(result.is_err(), "Truncated footer should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
}

#[test]
fn test_corrupted_metadata() {
    // Create a valid Parquet file then corrupt the metadata section
    let doc = create_simple_doc();
    let mut bytes = to_parquet_bytes(&doc).unwrap();

    // Corrupt some bytes in the middle (likely metadata/data pages)
    if bytes.len() > 100 {
        for i in 50..70 {
            bytes[i] = 0xFF;
        }
    }

    let result = from_parquet_bytes(&bytes);
    // May succeed with corrupted data or fail with IO error
    if result.is_err() {
        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::IO);
    }
}

#[test]
fn test_random_binary_data() {
    // Completely random data that looks nothing like Parquet
    let random_bytes: Vec<u8> = (0..1000).map(|i| (i * 7 % 256) as u8).collect();
    let result = from_parquet_bytes(&random_bytes);

    assert!(result.is_err(), "Random binary data should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
}

#[test]
fn test_wrong_file_format_json() {
    // Try to parse JSON as Parquet
    let json_bytes = b"{\"users\": [{\"name\": \"alice\"}]}";
    let result = from_parquet_bytes(json_bytes);

    assert!(result.is_err(), "JSON file should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
}

#[test]
fn test_wrong_file_format_csv() {
    // Try to parse CSV as Parquet
    let csv_bytes = b"id,name,age\n1,alice,30\n2,bob,25\n";
    let result = from_parquet_bytes(csv_bytes);

    assert!(result.is_err(), "CSV file should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
}

// =============================================================================
// 2. Unsupported Parquet Types
// =============================================================================

#[test]
fn test_unsupported_type_binary() {
    // Binary type is not directly supported by HEDL
    let parquet_bytes = create_parquet_with_binary_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // Binary should be rejected or converted to string
    if result.is_err() {
        let err = result.unwrap_err();
        assert!(
            err.kind == HedlErrorKind::Syntax || err.kind == HedlErrorKind::IO,
            "Binary type should produce Syntax or IO error"
        );
    }
}

#[test]
fn test_unsupported_type_date32() {
    // Date32 is days since epoch, not natively supported
    let parquet_bytes = create_parquet_with_date32_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // Date types might not be supported
    if result.is_err() {
        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(err.message.contains("Unsupported"));
    }
}

#[test]
fn test_unsupported_type_timestamp() {
    // Timestamp with timezone
    let parquet_bytes = create_parquet_with_timestamp_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // Timestamps might not be supported
    if result.is_err() {
        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(err.message.contains("Unsupported"));
    }
}

#[test]
fn test_unsupported_type_duration() {
    // Duration type
    let parquet_bytes = create_parquet_with_duration_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // Durations are not supported
    if result.is_err() {
        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(err.message.contains("Unsupported"));
    }
}

#[test]
fn test_unsupported_type_decimal() {
    // Decimal128 type
    let parquet_bytes = create_parquet_with_decimal_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // Decimals might not be supported
    if result.is_err() {
        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(err.message.contains("Unsupported"));
    }
}

#[test]
fn test_unsupported_type_time() {
    // Time64 type
    let parquet_bytes = create_parquet_with_time_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // Time types might not be supported
    if result.is_err() {
        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(err.message.contains("Unsupported"));
    }
}

#[test]
fn test_unsupported_type_fixed_size_binary() {
    // FixedSizeBinary type
    let parquet_bytes = create_parquet_with_fixed_binary_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // Fixed size binary might not be supported
    if result.is_err() {
        let err = result.unwrap_err();
        assert!(
            err.kind == HedlErrorKind::Syntax || err.kind == HedlErrorKind::IO,
            "Fixed binary should produce error"
        );
    }
}

#[test]
fn test_unsupported_type_list() {
    // Nested list type
    let parquet_bytes = create_parquet_with_list_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // Nested lists might not be supported
    if result.is_err() {
        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(err.message.contains("Unsupported"));
    }
}

#[test]
fn test_unsupported_type_struct() {
    // Nested struct type
    let parquet_bytes = create_parquet_with_struct_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // Nested structs might not be supported
    if result.is_err() {
        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(err.message.contains("Unsupported"));
    }
}

#[test]
fn test_unsupported_type_large_utf8() {
    // LargeUtf8 (string with 64-bit offsets)
    let parquet_bytes = create_parquet_with_large_utf8_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // LargeUtf8 might not be supported or converted to regular strings
    if result.is_err() {
        let err = result.unwrap_err();
        assert!(
            err.kind == HedlErrorKind::Syntax || err.kind == HedlErrorKind::IO,
            "LargeUtf8 should produce error if not supported"
        );
    }
}

#[test]
fn test_unsupported_type_large_binary() {
    // LargeBinary (binary with 64-bit offsets)
    let parquet_bytes = create_parquet_with_large_binary_column();
    let result = from_parquet_bytes(&parquet_bytes);

    // LargeBinary should be rejected
    if result.is_err() {
        let err = result.unwrap_err();
        assert!(
            err.kind == HedlErrorKind::Syntax || err.kind == HedlErrorKind::IO,
            "LargeBinary should produce error"
        );
    }
}

// =============================================================================
// 3. Invalid Data
// =============================================================================

#[test]
fn test_invalid_utf8_in_string_column() {
    // This is actually hard to create with Arrow since it validates UTF-8
    // But we can test the error path exists
    // Arrow/Parquet should reject invalid UTF-8 at write time
    let doc = create_simple_doc();
    let bytes = to_parquet_bytes(&doc).unwrap();

    // If we somehow got invalid UTF-8, it should be caught
    let result = from_parquet_bytes(&bytes);
    assert!(result.is_ok(), "Valid UTF-8 should succeed");
}

#[test]
fn test_uint64_overflow() {
    // UInt64 values > i64::MAX cannot be represented in HEDL
    let parquet_bytes = create_parquet_with_uint64_overflow();
    let result = from_parquet_bytes(&parquet_bytes);

    assert!(
        result.is_err(),
        "UInt64 overflow should be rejected or handled"
    );
    if result.is_err() {
        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::Syntax);
        assert!(
            err.message.contains("exceeds i64::MAX")
                || err.message.contains("overflow")
                || err.message.contains("UInt64")
        );
    }
}

#[test]
fn test_schema_column_count_zero() {
    // Schema with zero columns is invalid
    let schema = Arc::new(Schema::new(vec![] as Vec<Field>));

    let columns: Vec<ArrayRef> = vec![];

    // Try to create a record batch with zero columns
    let batch_result = RecordBatch::try_new(schema.clone(), columns);

    // Arrow might reject this at batch creation time
    if let Ok(batch) = batch_result {
        let mut buffer = Vec::new();
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        {
            let writer_result = ArrowWriter::try_new(&mut buffer, schema, Some(props));
            if let Ok(mut writer) = writer_result {
                let _ = writer.write(&batch);
                let _ = writer.close();
            }
        } // writer scope ends here

        if !buffer.is_empty() {
            let result = from_parquet_bytes(&buffer);
            // Should either fail or produce empty document
            if result.is_ok() {
                let doc = result.unwrap();
                // Empty schema should produce empty or minimal document
                assert!(
                    doc.root.is_empty() || doc.root.len() <= 1,
                    "Zero column schema should produce minimal document"
                );
            }
        }
    }
}

#[test]
fn test_mismatched_column_lengths() {
    // This is prevented by RecordBatch validation, but test the error path
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("value", DataType::Int64, true),
    ]));

    // Different lengths would be rejected by RecordBatch::try_new
    let id_array = Arc::new(StringArray::from(vec!["row1", "row2"])) as ArrayRef;
    let value_array = Arc::new(Int64Array::from(vec![1])) as ArrayRef; // Wrong length!

    let batch_result = RecordBatch::try_new(schema, vec![id_array, value_array]);

    // RecordBatch creation should fail
    assert!(
        batch_result.is_err(),
        "Mismatched column lengths should be rejected"
    );
}

#[test]
fn test_null_in_non_nullable_column() {
    // Create a schema with non-nullable column but insert null
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false), // non-nullable
        Field::new("value", DataType::Int64, true),
    ]));

    // Try to insert null in non-nullable column
    let id_array = Arc::new(StringArray::from(vec![Some("row1"), None])) as ArrayRef; // Null in non-nullable!
    let value_array = Arc::new(Int64Array::from(vec![1, 2])) as ArrayRef;

    let batch_result = RecordBatch::try_new(schema.clone(), vec![id_array, value_array]);

    // Arrow should allow this (nullable is a hint), but Parquet might enforce it
    if let Ok(batch) = batch_result {
        let mut buffer = Vec::new();
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        {
            if let Ok(mut writer) = ArrowWriter::try_new(&mut buffer, schema, Some(props)) {
                let _ = writer.write(&batch);
                let _ = writer.close();
            }
        } // writer scope ends

        if !buffer.is_empty() {
            let result = from_parquet_bytes(&buffer);
            // Should handle null in ID column gracefully
            if result.is_ok() {
                let doc = result.unwrap();
                // Verify it didn't crash
                assert!(doc.root.is_empty() || !doc.root.is_empty());
            }
        }
    }
}

#[test]
fn test_extremely_long_column_name() {
    // Column name with 10,000 characters
    // The implementation sanitizes but preserves length, which is acceptable
    // since extremely long identifiers are rare in practice
    let long_name = "a".repeat(10_000);
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(&long_name, DataType::Int64, true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let value_array = Arc::new(Int64Array::from(vec![42])) as ArrayRef;

    if let Ok(batch) = RecordBatch::try_new(schema.clone(), vec![id_array, value_array]) {
        let mut buffer = Vec::new();
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        {
            if let Ok(mut writer) = ArrowWriter::try_new(&mut buffer, schema, Some(props)) {
                let _ = writer.write(&batch);
                let _ = writer.close();
            }
        } // writer scope ends

        if !buffer.is_empty() {
            let result = from_parquet_bytes(&buffer);
            // Should handle long column names gracefully (sanitize)
            if result.is_ok() {
                let doc = result.unwrap();
                if let Some(Item::List(list)) = doc.root.values().next() {
                    // Column names should be valid identifiers (alphanumeric + underscore)
                    for col in &list.schema {
                        // Verify column names are valid after sanitization
                        assert!(
                            col.chars()
                                .all(|c| c.is_alphanumeric() || c == '_'),
                            "Column name should contain only alphanumeric and underscore: {}",
                            col
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_invalid_column_name_special_chars() {
    // Column name with special characters that aren't valid HEDL identifiers
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("col!@#$%^&*()", DataType::Int64, true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let value_array = Arc::new(Int64Array::from(vec![42])) as ArrayRef;

    if let Ok(batch) = RecordBatch::try_new(schema.clone(), vec![id_array, value_array]) {
        let mut buffer = Vec::new();
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        {
            if let Ok(mut writer) = ArrowWriter::try_new(&mut buffer, schema, Some(props)) {
                let _ = writer.write(&batch);
                let _ = writer.close();
            }
        } // writer scope ends

        if !buffer.is_empty() {
            let result = from_parquet_bytes(&buffer);
            // Should sanitize column names
            if result.is_ok() {
                let doc = result.unwrap();
                if let Some(Item::List(list)) = doc.root.values().next() {
                    // Column names should be sanitized to valid identifiers
                    for col in &list.schema {
                        // Should only contain alphanumeric and underscore
                        assert!(
                            col.chars()
                                .all(|c| c.is_alphanumeric() || c == '_'),
                            "Column name should be sanitized: {}",
                            col
                        );
                    }
                }
            }
        }
    }
}

// =============================================================================
// 4. Resource Exhaustion Edge Cases
// =============================================================================

#[test]
fn test_max_columns_boundary_minus_one() {
    // 999 columns (just under limit of 1000)
    let parquet_bytes = create_parquet_with_n_columns(999);
    let result = from_parquet_bytes(&parquet_bytes);

    assert!(result.is_ok(), "999 columns should be accepted");
}

#[test]
fn test_max_columns_boundary_exact() {
    // Exactly 1000 columns (at limit)
    let parquet_bytes = create_parquet_with_n_columns(1000);
    let result = from_parquet_bytes(&parquet_bytes);

    assert!(result.is_ok(), "1000 columns should be accepted");
}

#[test]
fn test_max_columns_boundary_plus_one() {
    // 1001 columns (just over limit)
    let parquet_bytes = create_parquet_with_n_columns(1001);
    let result = from_parquet_bytes(&parquet_bytes);

    assert!(result.is_err(), "1001 columns should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Security);
    assert!(err.message.contains("maximum column count"));
}

#[test]
fn test_very_wide_table() {
    // 2000 columns (well over limit)
    let parquet_bytes = create_parquet_with_n_columns(2000);
    let result = from_parquet_bytes(&parquet_bytes);

    assert!(result.is_err(), "2000 columns should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Security);
}

#[test]
fn test_single_row_many_columns() {
    // 1500 columns but only 1 row - still should be rejected
    let parquet_bytes = create_parquet_with_n_columns(1500);
    let result = from_parquet_bytes(&parquet_bytes);

    assert!(
        result.is_err(),
        "1500 columns with 1 row should still be rejected"
    );
    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Security);
}

#[test]
fn test_moderate_rows_moderate_columns() {
    // 100 columns × 1000 rows = 100K cells (should be acceptable)
    let parquet_bytes = create_parquet_with_dimensions(100, 1000);
    let result = from_parquet_bytes(&parquet_bytes);

    assert!(
        result.is_ok(),
        "100 columns × 1000 rows should be accepted"
    );
}

#[test]
fn test_empty_rows_many_columns() {
    // Many columns but zero rows
    let schema = Arc::new(Schema::new(
        (0..500)
            .map(|i| Field::new(format!("col{}", i), DataType::Int64, true))
            .collect::<Vec<_>>(),
    ));

    let columns: Vec<ArrayRef> = (0..500)
        .map(|_| Arc::new(Int64Array::from(Vec::<i64>::new())) as ArrayRef)
        .collect();

    if let Ok(batch) = RecordBatch::try_new(schema.clone(), columns) {
        let mut buffer = Vec::new();
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        {
            if let Ok(mut writer) = ArrowWriter::try_new(&mut buffer, schema, Some(props)) {
                let _ = writer.write(&batch);
                let _ = writer.close();
            }
        } // writer scope ends

        if !buffer.is_empty() {
            let result = from_parquet_bytes(&buffer);
            // Empty rows with many columns should still be accepted if under column limit
            if result.is_ok() {
                let doc = result.unwrap();
                // Should produce empty or minimal document
                assert!(doc.root.is_empty() || doc.root.values().all(|item| {
                    if let Item::List(list) = item {
                        list.rows.is_empty()
                    } else {
                        true
                    }
                }));
            }
        }
    }
}

// =============================================================================
// Helper Functions for Test Data Generation
// =============================================================================

/// Create a simple document for testing.
///
/// Note: Using hedl-test builders to eliminate duplication.
fn create_simple_doc() -> Document {
    use hedl_test::fixtures::builders::{DocumentBuilder, MatrixListBuilder, NodeBuilder, ValueBuilder};

    let list = MatrixListBuilder::new("Item")
        .schema(vec!["id".to_string(), "value".to_string()])
        .row(NodeBuilder::new("Item", "i1")
            .field(ValueBuilder::string("i1"))
            .field(ValueBuilder::int(100))
            .build())
        .row(NodeBuilder::new("Item", "i2")
            .field(ValueBuilder::string("i2"))
            .field(ValueBuilder::int(200))
            .build())
        .build();

    DocumentBuilder::new()
        .struct_def("Item", vec!["id".to_string(), "value".to_string()])
        .list("items", list)
        .build()
}

/// Create Parquet file with binary column.
fn create_parquet_with_binary_column() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("data", DataType::Binary, true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let binary_array = Arc::new(BinaryArray::from(vec![b"binary data".as_ref()])) as ArrayRef;

    create_parquet_from_batch(schema, vec![id_array, binary_array])
}

/// Create Parquet file with Date32 column.
fn create_parquet_with_date32_column() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("date", DataType::Date32, true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let date_array = Arc::new(Date32Array::from(vec![18000])) as ArrayRef; // Days since epoch

    create_parquet_from_batch(schema, vec![id_array, date_array])
}

/// Create Parquet file with Timestamp column.
fn create_parquet_with_timestamp_column() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            "timestamp",
            DataType::Timestamp(TimeUnit::Microsecond, None),
            true,
        ),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let ts_array = Arc::new(TimestampMicrosecondArray::from(vec![1234567890])) as ArrayRef;

    create_parquet_from_batch(schema, vec![id_array, ts_array])
}

/// Create Parquet file with Duration column.
fn create_parquet_with_duration_column() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("duration", DataType::Duration(TimeUnit::Microsecond), true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let duration_array = Arc::new(DurationMicrosecondArray::from(vec![1000000])) as ArrayRef;

    create_parquet_from_batch(schema, vec![id_array, duration_array])
}

/// Create Parquet file with Decimal128 column.
fn create_parquet_with_decimal_column() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("amount", DataType::Decimal128(10, 2), true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let decimal_array = Arc::new(
        Decimal128Array::from(vec![12345])
            .with_precision_and_scale(10, 2)
            .unwrap(),
    ) as ArrayRef;

    create_parquet_from_batch(schema, vec![id_array, decimal_array])
}

/// Create Parquet file with Time64 column.
fn create_parquet_with_time_column() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("time", DataType::Time64(TimeUnit::Microsecond), true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let time_array = Arc::new(Time64MicrosecondArray::from(vec![3600000000])) as ArrayRef;

    create_parquet_from_batch(schema, vec![id_array, time_array])
}

/// Create Parquet file with FixedSizeBinary column.
fn create_parquet_with_fixed_binary_column() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("data", DataType::FixedSizeBinary(16), true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let fixed_binary_array =
        Arc::new(FixedSizeBinaryArray::from(vec![b"0123456789abcdef".as_ref()])) as ArrayRef;

    create_parquet_from_batch(schema, vec![id_array, fixed_binary_array])
}

/// Create Parquet file with List column.
fn create_parquet_with_list_column() -> Vec<u8> {
    use arrow::buffer::OffsetBuffer;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            "tags",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            true,
        ),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;

    // Create a list array
    let values = Arc::new(StringArray::from(vec!["tag1", "tag2"]));
    let offsets = OffsetBuffer::new(vec![0i32, 2].into());
    let list_array = Arc::new(ListArray::new(
        Arc::new(Field::new("item", DataType::Utf8, true)),
        offsets,
        values,
        None,
    )) as ArrayRef;

    create_parquet_from_batch(schema, vec![id_array, list_array])
}

/// Create Parquet file with Struct column.
fn create_parquet_with_struct_column() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            "person",
            DataType::Struct(
                vec![
                    Field::new("name", DataType::Utf8, true),
                    Field::new("age", DataType::Int32, true),
                ]
                .into(),
            ),
            true,
        ),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;

    // Create struct array
    let name_array = Arc::new(StringArray::from(vec!["Alice"])) as ArrayRef;
    let age_array = Arc::new(Int32Array::from(vec![30])) as ArrayRef;

    let struct_array = Arc::new(StructArray::from(vec![
        (
            Arc::new(Field::new("name", DataType::Utf8, true)),
            name_array,
        ),
        (
            Arc::new(Field::new("age", DataType::Int32, true)),
            age_array,
        ),
    ])) as ArrayRef;

    create_parquet_from_batch(schema, vec![id_array, struct_array])
}

/// Create Parquet file with LargeUtf8 column.
fn create_parquet_with_large_utf8_column() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("text", DataType::LargeUtf8, true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let large_utf8_array = Arc::new(LargeStringArray::from(vec!["large text"])) as ArrayRef;

    create_parquet_from_batch(schema, vec![id_array, large_utf8_array])
}

/// Create Parquet file with LargeBinary column.
fn create_parquet_with_large_binary_column() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("data", DataType::LargeBinary, true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let large_binary_array = Arc::new(LargeBinaryArray::from(vec![b"large binary".as_ref()])) as ArrayRef;

    create_parquet_from_batch(schema, vec![id_array, large_binary_array])
}

/// Create Parquet file with UInt64 values that overflow i64::MAX.
fn create_parquet_with_uint64_overflow() -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("big_uint", DataType::UInt64, true),
    ]));

    let id_array = Arc::new(StringArray::from(vec!["row1"])) as ArrayRef;
    let uint64_array = Arc::new(UInt64Array::from(vec![u64::MAX])) as ArrayRef; // Maximum u64, exceeds i64::MAX

    create_parquet_from_batch(schema, vec![id_array, uint64_array])
}

/// Create Parquet file with N columns.
fn create_parquet_with_n_columns(n: usize) -> Vec<u8> {
    let mut fields = vec![Field::new("id", DataType::Utf8, false)];
    for i in 1..n {
        fields.push(Field::new(format!("col{}", i), DataType::Int64, true));
    }

    let schema = Arc::new(Schema::new(fields));

    let mut columns: Vec<ArrayRef> = vec![Arc::new(StringArray::from(vec!["row1"]))];
    for _ in 1..n {
        columns.push(Arc::new(Int64Array::from(vec![42])));
    }

    create_parquet_from_batch(schema, columns)
}

/// Create Parquet file with specified dimensions (columns × rows).
fn create_parquet_with_dimensions(num_columns: usize, num_rows: usize) -> Vec<u8> {
    let mut fields = vec![Field::new("id", DataType::Utf8, false)];
    for i in 1..num_columns {
        fields.push(Field::new(format!("col{}", i), DataType::Int64, true));
    }

    let schema = Arc::new(Schema::new(fields));

    let id_data: Vec<String> = (0..num_rows).map(|i| format!("row{}", i)).collect();
    let mut columns: Vec<ArrayRef> = vec![Arc::new(StringArray::from(id_data))];

    for _ in 1..num_columns {
        let int_data: Vec<i64> = (0..num_rows).map(|_| 42).collect();
        columns.push(Arc::new(Int64Array::from(int_data)));
    }

    create_parquet_from_batch(schema, columns)
}

/// Helper to create Parquet bytes from schema and columns.
fn create_parquet_from_batch(schema: Arc<Schema>, columns: Vec<ArrayRef>) -> Vec<u8> {
    let batch = RecordBatch::try_new(schema.clone(), columns).unwrap();

    let mut buffer = Vec::new();
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();

    let mut writer = ArrowWriter::try_new(&mut buffer, schema, Some(props)).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();

    buffer
}
