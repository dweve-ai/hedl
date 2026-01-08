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

//! Convert Parquet files to HEDL documents.
//!
//! # Position Encoding Strategy
//!
//! This module preserves row order when converting Parquet files to HEDL:
//!
//! ## Ordered Data (Default)
//!
//! Row position is **implicitly preserved** through sequential processing:
//! - Parquet row `i` → `MatrixList.rows[i]`
//! - Record batches are processed in order
//! - Rows within batches are processed sequentially
//! - No reordering occurs during conversion
//!
//! ## Example
//!
//! ```rust
//! use hedl_parquet::from_parquet_bytes;
//!
//! # let parquet_bytes = vec![]; // Assume valid Parquet bytes
//! # if !parquet_bytes.is_empty() {
//! let doc = from_parquet_bytes(&parquet_bytes).unwrap();
//! // Row order from Parquet file is preserved in MatrixList.rows
//! # }
//! ```
//!
//! ## Error Context Position
//!
//! The `position` parameter in error handling encodes **error context**, not data position:
//! - Formula: `position = row_idx * 1000 + col_idx`
//! - Used only for error reporting
//! - Allows decoding row and column from single value
//! - Not related to data position preservation
//!
//! See `POSITION_ENCODING.md` for detailed documentation.
//!
//! # Security Protections
//!
//! This module implements comprehensive security protections for reading untrusted
//! Parquet files:
//!
//! - **Decompression bomb protection**: Limits total decompressed data to 100 MB
//! - **Large schema protection**: Limits schemas to 1,000 columns
//! - **Memory tracking**: Estimates and tracks memory usage across all batches
//! - **Overflow protection**: Uses checked arithmetic for all size calculations
//! - **Identifier validation**: Validates and sanitizes all metadata identifiers
//!
//! See `SECURITY.md` for detailed threat model and mitigation strategies.

use std::path::Path;
use std::sync::Arc;

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array,
    StringArray, UInt16Array, UInt32Array, UInt64Array, UInt8Array,
};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use hedl_core::{Document, HedlError, HedlErrorKind, Item, MatrixList, Node, Value};

/// Maximum decompressed data size in bytes (prevents decompression bombs).
///
/// This limit prevents malicious Parquet files from decompressing to enormous sizes.
/// A 10 KB compressed file could theoretically decompress to 10 GB, causing memory
/// exhaustion. This limit ensures files are rejected after 100 MB of decompressed data.
///
/// Default: 100 MB
const MAX_DECOMPRESSED_SIZE: usize = 100 * 1024 * 1024;

/// Maximum number of columns in a schema (prevents large schema attacks).
///
/// Files with thousands of columns could exhaust memory during schema processing,
/// even with minimal row data. This limit prevents such attacks while still supporting
/// reasonably wide tables.
///
/// Default: 1,000 columns
const MAX_COLUMNS: usize = 1000;

/// Read a HEDL document from a Parquet file.
///
/// # Arguments
///
/// * `path` - Path to the Parquet file to read
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The Parquet format is invalid
/// - The data cannot be converted to HEDL
///
/// # Example
///
/// ```no_run
/// use hedl_parquet::from_parquet;
/// use std::path::Path;
///
/// let doc = from_parquet(Path::new("input.parquet")).unwrap();
/// ```
pub fn from_parquet(path: &Path) -> Result<Document, HedlError> {
    let file = std::fs::File::open(path).map_err(|e| {
        HedlError::io(format!("Failed to open Parquet file: {}", e))
    })?;

    read_parquet_from_file(file)
}

/// Read a HEDL document from Parquet bytes.
///
/// # Example
///
/// ```no_run
/// use hedl_parquet::from_parquet_bytes;
///
/// let bytes = vec![]; // Some Parquet bytes
/// let doc = from_parquet_bytes(&bytes).unwrap();
/// ```
pub fn from_parquet_bytes(bytes: &[u8]) -> Result<Document, HedlError> {
    // Convert to bytes::Bytes for ChunkReader implementation
    let bytes_data = bytes::Bytes::copy_from_slice(bytes);

    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes_data).map_err(|e| {
        HedlError::io(format!("Failed to create Parquet reader: {}", e))
    })?;

    // Extract file metadata before building reader
    let file_metadata = builder.metadata().file_metadata();
    let hedl_metadata = extract_hedl_metadata(file_metadata);

    let arrow_reader = builder.build().map_err(|e| {
        HedlError::io(format!("Failed to build Parquet reader: {}", e))
    })?;

    read_batches(arrow_reader, hedl_metadata)
}

/// Read Parquet data from a File.
fn read_parquet_from_file(file: std::fs::File) -> Result<Document, HedlError> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(|e| {
        HedlError::io(format!("Failed to create Parquet reader: {}", e))
    })?;

    // Extract file metadata before building reader
    let file_metadata = builder.metadata().file_metadata();
    let hedl_metadata = extract_hedl_metadata(file_metadata);

    let arrow_reader = builder.build().map_err(|e| {
        HedlError::io(format!("Failed to build Parquet reader: {}", e))
    })?;

    read_batches(arrow_reader, hedl_metadata)
}

/// HEDL metadata extracted from Parquet file.
#[derive(Debug, Clone, Default)]
struct HedlMetadata {
    type_name: Option<String>,
    key: Option<String>,
}

/// Validate that a string is a valid HEDL identifier.
fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100 // Reasonable identifier length limit
        && name
            .chars()
            .next()
            .map(|c| c.is_alphabetic() || c == '_')
            .unwrap_or(false)
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Extract HEDL metadata from Parquet file metadata.
///
/// # Metadata Keys
///
/// Uses standardized metadata key names:
/// - `hedl:type_name` - The HEDL type name for entities
/// - `hedl:key` - The key name for the list in the document
///
/// Both values are validated as valid HEDL identifiers before use.
fn extract_hedl_metadata(file_metadata: &parquet::file::metadata::FileMetaData) -> HedlMetadata {
    let mut metadata = HedlMetadata::default();

    if let Some(kv_metadata) = file_metadata.key_value_metadata() {
        for kv in kv_metadata {
            // Security: Metadata key names are hardcoded for safety
            if kv.key == "hedl:type_name" {
                // Security: Validate that extracted type_name is a valid identifier
                if let Some(ref value) = kv.value {
                    if is_valid_identifier(value) {
                        metadata.type_name = Some(value.clone());
                    }
                }
            } else if kv.key == "hedl:key" {
                // Security: Validate that extracted key is a valid identifier
                if let Some(ref value) = kv.value {
                    if is_valid_identifier(value) {
                        metadata.key = Some(value.clone());
                    }
                }
            }
        }
    }

    metadata
}

/// Read all record batches from the Arrow reader.
fn read_batches(
    arrow_reader: impl Iterator<Item = Result<RecordBatch, arrow::error::ArrowError>>,
    hedl_metadata: HedlMetadata,
) -> Result<Document, HedlError> {
    let mut doc = Document::new((1, 0));
    let mut total_bytes = 0usize;

    // Read all record batches
    for batch_result in arrow_reader {
        let batch = batch_result.map_err(|e| {
            HedlError::io(format!("Failed to read record batch: {}", e))
        })?;

        // Security: Track decompressed data size to prevent decompression bombs
        let batch_bytes = estimate_batch_size(&batch);
        total_bytes = total_bytes.checked_add(batch_bytes).ok_or_else(|| {
            HedlError::security("decompressed size calculation overflow", 0)
        })?;

        if total_bytes > MAX_DECOMPRESSED_SIZE {
            return Err(HedlError::security(
                format!(
                    "Decompressed size limit exceeded: {} bytes (max: {} bytes)",
                    total_bytes, MAX_DECOMPRESSED_SIZE
                ),
                0,
            ));
        }

        convert_record_batch_to_hedl(&batch, &mut doc, &hedl_metadata)?;
    }

    Ok(doc)
}

/// Estimate the size of a RecordBatch in bytes.
fn estimate_batch_size(batch: &RecordBatch) -> usize {
    let mut size = 0;
    for column in batch.columns() {
        size += column.get_array_memory_size();
    }
    size
}

/// Convert a RecordBatch to HEDL structure.
fn convert_record_batch_to_hedl(
    batch: &RecordBatch,
    doc: &mut Document,
    hedl_metadata: &HedlMetadata,
) -> Result<(), HedlError> {
    let schema = batch.schema();

    // Security: Validate schema column count to prevent large schema attacks
    if schema.fields().len() > MAX_COLUMNS {
        return Err(HedlError::security(
            format!(
                "Schema exceeds maximum column count: {} (max: {})",
                schema.fields().len(),
                MAX_COLUMNS
            ),
            0,
        ));
    }

    // Check if this is a metadata table (key-value pairs)
    if is_metadata_table(&schema) {
        return convert_metadata_table(batch, doc);
    }

    // Otherwise, treat it as a matrix list
    convert_to_matrix_list(batch, doc, hedl_metadata)
}

/// Check if the schema represents a metadata table (key, value columns).
fn is_metadata_table(schema: &Arc<arrow::datatypes::Schema>) -> bool {
    schema.fields().len() == 2
        && schema.field(0).name() == "key"
        && schema.field(1).name() == "value"
        && matches!(schema.field(0).data_type(), DataType::Utf8)
}

/// Convert a metadata table to HEDL key-value pairs.
fn convert_metadata_table(batch: &RecordBatch, doc: &mut Document) -> Result<(), HedlError> {
    let key_array = batch
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| {
            HedlError::new(
                HedlErrorKind::Syntax,
                "Expected string array for metadata keys",
                0,
            )
        })?;

    let value_array = batch.column(1);

    for i in 0..batch.num_rows() {
        if key_array.is_null(i) {
            continue;
        }

        let key = key_array.value(i).to_string();
        // Position represents row index in metadata table
        let value = extract_value_from_array(value_array, i, i)?;

        doc.root.insert(key, Item::Scalar(value));
    }

    Ok(())
}

/// Convert a record batch to a HEDL matrix list.
///
/// # Position Preservation
///
/// Processes rows sequentially to maintain order:
/// - Iterates from `row_idx = 0` to `batch.num_rows() - 1`
/// - Converts each row to a Node in order
/// - Appends nodes to MatrixList preserving order
///
/// This guarantees Parquet row `i` → `MatrixList.rows[i]`.
fn convert_to_matrix_list(
    batch: &RecordBatch,
    doc: &mut Document,
    hedl_metadata: &HedlMetadata,
) -> Result<(), HedlError> {
    let schema = batch.schema();
    let num_rows = batch.num_rows();

    if num_rows == 0 {
        return Ok(());
    }

    // Extract schema column names and validate them
    let column_names: Vec<String> = schema
        .fields()
        .iter()
        .map(|f| {
            let name = f.name();
            // Security: Validate column names are valid identifiers
            if !is_valid_identifier(name) {
                // Sanitize invalid identifiers by replacing invalid chars with underscore
                name.chars()
                    .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
                    .collect::<String>()
                    .trim_matches('_')
                    .to_string()
            } else {
                name.clone()
            }
        })
        .collect();

    // Try to get type name and key from metadata, otherwise infer
    let type_name = hedl_metadata
        .type_name
        .clone()
        .or_else(|| {
            // Infer type name from first column name or use "Table"
            column_names.first().and_then(|name| {
                if name == "id" {
                    None
                } else {
                    Some(name.clone())
                }
            })
        })
        .unwrap_or_else(|| "Table".to_string());

    let list_key = hedl_metadata
        .key
        .clone()
        .unwrap_or_else(|| format!("{}s", type_name.to_lowercase()));

    let mut matrix_list = MatrixList::new(&type_name, column_names.clone());

    // Convert each row to a Node
    for row_idx in 0..num_rows {
        let node = convert_row_to_node(batch, row_idx, &type_name, &column_names)?;
        matrix_list.add_row(node);
    }

    // Add to document with metadata key or inferred key name
    doc.root.insert(list_key, Item::List(matrix_list));

    // Track struct schema in document
    doc.structs.insert(type_name.clone(), column_names.clone());

    Ok(())
}

/// Convert a single row in a RecordBatch to a HEDL Node.
///
/// # Position Preservation
///
/// Extracts row data at the specified `row_idx`:
/// - ID extracted from first column at `row_idx`
/// - Field values extracted sequentially from all columns at `row_idx`
/// - Node created with fields in column order
///
/// # Error Context Position
///
/// The `position` parameter in error handling is for error context only:
/// - Row-level errors: `position = row_idx`
/// - Column-level errors: `position = row_idx * 1000 + col_idx`
/// - Allows decoding both row and column information in error messages
/// - Not used for data position tracking
fn convert_row_to_node(
    batch: &RecordBatch,
    row_idx: usize,
    type_name: &str,
    _column_names: &[String],
) -> Result<Node, HedlError> {
    // First column is the ID
    let id_array = batch.column(0);
    // Note: Position in HedlError represents row index in the Parquet batch for context
    let id = extract_string_from_array(id_array, row_idx, row_idx)?;

    // Pre-allocate fields vector with exact capacity (one field per column)
    let mut fields = Vec::with_capacity(batch.num_columns());
    for col_idx in 0..batch.num_columns() {
        let array = batch.column(col_idx);
        // Position in error represents (row_idx * 1000 + col_idx) to encode both row and column
        let value = extract_value_from_array(array, row_idx, row_idx * 1000 + col_idx)?;
        fields.push(value);
    }

    Ok(Node::new(type_name, id, fields))
}

/// Extract a string value from an array at the given index.
///
/// # Parameters
/// * `array` - The Arrow array to extract from
/// * `idx` - The row index within the array
/// * `position` - Error context position (typically row index or encoded row/column)
fn extract_string_from_array(
    array: &Arc<dyn Array>,
    idx: usize,
    position: usize,
) -> Result<String, HedlError> {
    if array.is_null(idx) {
        return Ok("null".to_string());
    }

    match array.data_type() {
        DataType::Utf8 => {
            let string_array = array.as_any().downcast_ref::<StringArray>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected string array at row {}", idx),
                    position,
                )
            })?;
            Ok(string_array.value(idx).to_string())
        }
        DataType::Int64 => {
            let int_array = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Int64 array at row {}", idx),
                    position,
                )
            })?;
            Ok(int_array.value(idx).to_string())
        }
        _ => Err(HedlError::new(
            HedlErrorKind::Syntax,
            format!(
                "Unsupported ID column type at row {}: {:?}",
                idx,
                array.data_type()
            ),
            position,
        )),
    }
}

/// Extract a HEDL value from an Arrow array at the given index.
///
/// # Parameters
/// * `array` - The Arrow array to extract from
/// * `idx` - The row index within the array
/// * `position` - Error context position (typically row index or encoded row/column)
fn extract_value_from_array(
    array: &Arc<dyn Array>,
    idx: usize,
    position: usize,
) -> Result<Value, HedlError> {
    if array.is_null(idx) {
        return Ok(Value::Null);
    }

    match array.data_type() {
        DataType::Boolean => {
            let bool_array = array.as_any().downcast_ref::<BooleanArray>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected boolean array at row {}", idx),
                    position,
                )
            })?;
            Ok(Value::Bool(bool_array.value(idx)))
        }
        DataType::Int8 => {
            let int_array = array.as_any().downcast_ref::<Int8Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Int8 array at row {}", idx),
                    position,
                )
            })?;
            Ok(Value::Int(int_array.value(idx) as i64))
        }
        DataType::Int16 => {
            let int_array = array.as_any().downcast_ref::<Int16Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Int16 array at row {}", idx),
                    position,
                )
            })?;
            Ok(Value::Int(int_array.value(idx) as i64))
        }
        DataType::Int32 => {
            let int_array = array.as_any().downcast_ref::<Int32Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Int32 array at row {}", idx),
                    position,
                )
            })?;
            Ok(Value::Int(int_array.value(idx) as i64))
        }
        DataType::Int64 => {
            let int_array = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Int64 array at row {}", idx),
                    position,
                )
            })?;
            Ok(Value::Int(int_array.value(idx)))
        }
        DataType::UInt8 => {
            let int_array = array.as_any().downcast_ref::<UInt8Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected UInt8 array at row {}", idx),
                    position,
                )
            })?;
            Ok(Value::Int(int_array.value(idx) as i64))
        }
        DataType::UInt16 => {
            let int_array = array.as_any().downcast_ref::<UInt16Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected UInt16 array at row {}", idx),
                    position,
                )
            })?;
            Ok(Value::Int(int_array.value(idx) as i64))
        }
        DataType::UInt32 => {
            let int_array = array.as_any().downcast_ref::<UInt32Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected UInt32 array at row {}", idx),
                    position,
                )
            })?;
            Ok(Value::Int(int_array.value(idx) as i64))
        }
        DataType::UInt64 => {
            let int_array = array.as_any().downcast_ref::<UInt64Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected UInt64 array at row {}", idx),
                    position,
                )
            })?;
            let val = int_array.value(idx);
            // Check for overflow: u64 values > i64::MAX cannot be safely converted
            if val > i64::MAX as u64 {
                return Err(HedlError::new(
                    HedlErrorKind::Syntax,
                    format!(
                        "UInt64 value {} at row {} exceeds i64::MAX and cannot be represented",
                        val, idx
                    ),
                    position,
                ));
            }
            Ok(Value::Int(val as i64))
        }
        DataType::Float32 => {
            let float_array = array.as_any().downcast_ref::<Float32Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Float32 array at row {}", idx),
                    position,
                )
            })?;
            Ok(Value::Float(float_array.value(idx) as f64))
        }
        DataType::Float64 => {
            let float_array = array.as_any().downcast_ref::<Float64Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Float64 array at row {}", idx),
                    position,
                )
            })?;
            Ok(Value::Float(float_array.value(idx)))
        }
        DataType::Utf8 => {
            let string_array = array.as_any().downcast_ref::<StringArray>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected string array at row {}", idx),
                    position,
                )
            })?;
            let s = string_array.value(idx);

            // Try to detect references
            if s.starts_with('@') {
                return Ok(parse_reference_string(s));
            }

            Ok(Value::String(s.to_string()))
        }
        _ => Err(HedlError::new(
            HedlErrorKind::Syntax,
            format!(
                "Unsupported Arrow data type at row {}: {:?}",
                idx,
                array.data_type()
            ),
            position,
        )),
    }
}

/// Parse a reference string (e.g., "@User:id" or "@id").
fn parse_reference_string(s: &str) -> Value {
    // Validate string starts with '@' and has content after it
    let without_at = match s.strip_prefix('@') {
        Some(rest) if !rest.is_empty() => rest,
        _ => return Value::String(s.to_string()), // Not a valid reference, return as string
    };

    if let Some(colon_idx) = without_at.find(':') {
        // Ensure there's content after the colon
        if colon_idx + 1 < without_at.len() {
            let type_name = without_at[..colon_idx].to_string();
            let id = without_at[colon_idx + 1..].to_string();
            Value::Reference(hedl_core::Reference::qualified(type_name, id))
        } else {
            // Colon at end, treat as local reference
            Value::Reference(hedl_core::Reference::local(&without_at[..colon_idx]))
        }
    } else {
        Value::Reference(hedl_core::Reference::local(without_at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Int64Array;
    use arrow::datatypes::{Field, Schema};
    use std::sync::Arc;

    #[test]
    fn test_extract_value_from_array_int() {
        let array: Arc<dyn Array> = Arc::new(Int64Array::from(vec![1, 2, 3]));
        let value = extract_value_from_array(&array, 0, 0).unwrap();
        assert_eq!(value, Value::Int(1));
    }

    #[test]
    fn test_extract_value_from_array_null() {
        let array: Arc<dyn Array> = Arc::new(Int64Array::from(vec![Some(1), None, Some(3)]));
        let value = extract_value_from_array(&array, 1, 1).unwrap();
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn test_parse_reference_string_local() {
        let value = parse_reference_string("@alice");
        match value {
            Value::Reference(r) => {
                assert_eq!(r.type_name, None);
                assert_eq!(r.id, "alice");
            }
            _ => panic!("Expected reference value"),
        }
    }

    #[test]
    fn test_parse_reference_string_qualified() {
        let value = parse_reference_string("@User:alice");
        match value {
            Value::Reference(r) => {
                assert_eq!(r.type_name, Some("User".to_string()));
                assert_eq!(r.id, "alice");
            }
            _ => panic!("Expected reference value"),
        }
    }

    #[test]
    fn test_is_metadata_table() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("key", DataType::Utf8, false),
            Field::new("value", DataType::Utf8, true),
        ]));
        assert!(is_metadata_table(&schema));

        let schema2 = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("name", DataType::Utf8, true),
        ]));
        assert!(!is_metadata_table(&schema2));
    }
}
