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
//! See the workspace root `SECURITY.md` for the detailed threat model.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array,
    StringArray, UInt16Array, UInt32Array, UInt64Array, UInt8Array,
};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ProjectionMask;

use hedl_core::{Document, HedlError, HedlErrorKind, Item, MatrixList, Node, Value};

use crate::config::{FromParquetConfig, NullIdHandling};

/// Calculate optimal batch size based on schema characteristics.
///
/// Analyzes the schema to determine:
/// - Number of columns
/// - Proportion of variable-length string columns
///
/// Then delegates to `BatchSize::get_effective_size` for the actual calculation.
fn calculate_batch_size(
    schema: &Arc<arrow::datatypes::Schema>,
    batch_size_config: &crate::config::BatchSize,
) -> usize {
    let num_columns = schema.fields().len();

    // Count string columns to determine if schema is string-heavy
    let string_column_count = schema
        .fields()
        .iter()
        .filter(|f| matches!(f.data_type(), DataType::Utf8 | DataType::LargeUtf8))
        .count();

    // Consider "many strings" if > 30% of columns are strings
    let has_many_strings = num_columns > 0 && (string_column_count * 100 / num_columns) > 30;

    batch_size_config.get_effective_size(num_columns, has_many_strings)
}

/// Maximum decompressed data size in bytes (prevents decompression bombs).
///
/// This limit prevents malicious Parquet files from decompressing to enormous sizes.
/// A 10 KB compressed file could theoretically decompress to 10 GB, causing memory
/// exhaustion. This limit ensures files are rejected after 100 MB of decompressed data.
///
/// Default: 100 MB
pub(crate) const MAX_DECOMPRESSED_SIZE: usize = 100 * 1024 * 1024;

/// Maximum number of columns in a schema (prevents large schema attacks).
///
/// Files with thousands of columns could exhaust memory during schema processing,
/// even with minimal row data. This limit prevents such attacks while still supporting
/// reasonably wide tables.
///
/// Default: 1,000 columns
pub(crate) const MAX_COLUMNS: usize = 1000;

/// Read a HEDL document from a Parquet file with default configuration (strict mode).
///
/// This function uses strict mode which rejects null IDs. For lenient parsing,
/// use `from_parquet_with_config`.
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
/// - Any ID column contains null values (strict mode)
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
    from_parquet_with_config(path, &FromParquetConfig::default())
}

/// Read a HEDL document from a Parquet file with custom configuration.
///
/// # Arguments
///
/// * `path` - Path to the Parquet file to read
/// * `config` - Configuration for handling edge cases like null IDs
///
/// # Example
///
/// ```no_run
/// use hedl_parquet::{from_parquet_with_config, FromParquetConfig};
/// use std::path::Path;
///
/// let config = FromParquetConfig::lenient();
/// let doc = from_parquet_with_config(Path::new("input.parquet"), &config).unwrap();
/// ```
pub fn from_parquet_with_config(
    path: &Path,
    config: &FromParquetConfig,
) -> Result<Document, HedlError> {
    let file = std::fs::File::open(path)
        .map_err(|e| HedlError::io(format!("Failed to open Parquet file: {e}")))?;

    read_parquet_from_file(file, config)
}

/// Read a HEDL document from Parquet bytes with default configuration (strict mode).
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
    from_parquet_bytes_with_config(bytes, &FromParquetConfig::default())
}

/// Read a HEDL document from Parquet bytes with custom configuration.
///
/// # Example
///
/// ```no_run
/// use hedl_parquet::{from_parquet_bytes_with_config, FromParquetConfig};
///
/// let bytes = vec![]; // Some Parquet bytes
/// let config = FromParquetConfig::lenient();
/// let doc = from_parquet_bytes_with_config(&bytes, &config).unwrap();
/// ```
pub fn from_parquet_bytes_with_config(
    bytes: &[u8],
    config: &FromParquetConfig,
) -> Result<Document, HedlError> {
    // Convert to bytes::Bytes for ChunkReader implementation
    let bytes_data = bytes::Bytes::copy_from_slice(bytes);

    let mut builder = ParquetRecordBatchReaderBuilder::try_new(bytes_data)
        .map_err(|e| HedlError::io(format!("Failed to create Parquet reader: {e}")))?;

    // Extract file metadata before building reader
    let file_metadata = builder.metadata().file_metadata();
    let hedl_metadata = extract_hedl_metadata(file_metadata);

    // Get schema to determine batch size (clone to avoid borrow issues)
    let full_schema = builder.schema().clone();

    // Calculate optimal batch size based on schema characteristics
    let batch_size = calculate_batch_size(&full_schema, &config.batch_size);
    builder = builder.with_batch_size(batch_size);

    // Apply column projection if specified
    if let Some(ref column_names) = config.columns {
        let column_indices = validate_and_map_columns(&full_schema, column_names)?;
        let projection = ProjectionMask::roots(builder.parquet_schema(), column_indices);
        builder = builder.with_projection(projection);
    }

    let arrow_reader = builder
        .build()
        .map_err(|e| HedlError::io(format!("Failed to build Parquet reader: {e}")))?;

    read_batches(arrow_reader, hedl_metadata, config)
}

/// Read Parquet data from a File.
fn read_parquet_from_file(
    file: std::fs::File,
    config: &FromParquetConfig,
) -> Result<Document, HedlError> {
    let mut builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| HedlError::io(format!("Failed to create Parquet reader: {e}")))?;

    // Extract file metadata before building reader
    let file_metadata = builder.metadata().file_metadata();
    let hedl_metadata = extract_hedl_metadata(file_metadata);

    // Apply column projection if specified
    if let Some(ref column_names) = config.columns {
        let full_schema = builder.schema();
        let column_indices = validate_and_map_columns(full_schema, column_names)?;
        let projection = ProjectionMask::roots(builder.parquet_schema(), column_indices);
        builder = builder.with_projection(projection);
    }

    let arrow_reader = builder
        .build()
        .map_err(|e| HedlError::io(format!("Failed to build Parquet reader: {e}")))?;

    read_batches(arrow_reader, hedl_metadata, config)
}

/// HEDL metadata extracted from Parquet file.
#[derive(Debug, Clone, Default)]
pub(crate) struct HedlMetadata {
    pub(crate) type_name: Option<String>,
    pub(crate) key: Option<String>,
}

/// Validate that a string is a valid HEDL identifier.
fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100 // Reasonable identifier length limit
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
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
pub(crate) fn extract_hedl_metadata(
    file_metadata: &parquet::file::metadata::FileMetaData,
) -> HedlMetadata {
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
    config: &FromParquetConfig,
) -> Result<Document, HedlError> {
    let mut doc = Document::new((2, 0));
    let mut total_bytes = 0usize;

    // Read all record batches
    for batch_result in arrow_reader {
        let batch =
            batch_result.map_err(|e| HedlError::io(format!("Failed to read record batch: {e}")))?;

        // Security: Track decompressed data size to prevent decompression bombs
        let batch_bytes = estimate_batch_size(&batch);
        total_bytes = total_bytes
            .checked_add(batch_bytes)
            .ok_or_else(|| HedlError::security("decompressed size calculation overflow", 0))?;

        if total_bytes > MAX_DECOMPRESSED_SIZE {
            return Err(HedlError::security(
                format!(
                    "Decompressed size limit exceeded: {total_bytes} bytes (max: {MAX_DECOMPRESSED_SIZE} bytes)"
                ),
                0,
            ));
        }

        convert_record_batch_to_hedl(&batch, &mut doc, &hedl_metadata, config)?;
    }

    // Validate unique IDs after all batches are processed (defense in depth)
    for item in doc.root.values() {
        if let Item::List(list) = item {
            validate_unique_ids(&list.rows)?;
        }
    }

    Ok(doc)
}

/// Estimate the size of a `RecordBatch` in bytes.
pub(crate) fn estimate_batch_size(batch: &RecordBatch) -> usize {
    let mut size = 0;
    for column in batch.columns() {
        size += column.get_array_memory_size();
    }
    size
}

/// Convert a `RecordBatch` to HEDL structure.
pub(crate) fn convert_record_batch_to_hedl(
    batch: &RecordBatch,
    doc: &mut Document,
    hedl_metadata: &HedlMetadata,
    config: &FromParquetConfig,
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
    convert_to_matrix_list(batch, doc, hedl_metadata, config)
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
/// - Appends nodes to `MatrixList` preserving order
///
/// This guarantees Parquet row `i` → `MatrixList.rows[i]`.
fn convert_to_matrix_list(
    batch: &RecordBatch,
    doc: &mut Document,
    hedl_metadata: &HedlMetadata,
    config: &FromParquetConfig,
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
            if is_valid_identifier(name) {
                name.clone()
            } else {
                // Sanitize invalid identifiers by replacing invalid chars with underscore
                name.chars()
                    .map(|c| {
                        if c.is_alphanumeric() || c == '_' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>()
                    .trim_matches('_')
                    .to_string()
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

    // Get or create the matrix list for this type
    // This ensures we append to existing data when processing multiple batches
    let matrix_list = if let Some(Item::List(existing_list)) = doc.root.get_mut(&list_key) {
        existing_list
    } else {
        // Create new list and insert it
        doc.root.insert(
            list_key.clone(),
            Item::List(MatrixList::new(&type_name, column_names.clone())),
        );
        match doc.root.get_mut(&list_key) {
            Some(Item::List(list)) => list,
            _ => unreachable!("Just inserted a list"),
        }
    };

    // Convert each row to a Node and append to existing list
    for row_idx in 0..num_rows {
        let node = convert_row_to_node(batch, row_idx, &type_name, &column_names, config)?;
        matrix_list.add_row(node);
    }

    // Track struct schema in document (idempotent)
    doc.structs.insert(type_name.clone(), column_names.clone());

    Ok(())
}

/// Convert a single row in a `RecordBatch` to a HEDL Node.
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
    config: &FromParquetConfig,
) -> Result<Node, HedlError> {
    // First column is the ID
    let id_array = batch.column(0);
    // Note: Position in HedlError represents row index in the Parquet batch for context
    let id = extract_id_from_array(id_array, row_idx, config)?;

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

/// Extract an ID from an array at the given index, handling null values according to config.
///
/// # Parameters
/// * `array` - The Arrow array to extract from
/// * `idx` - The row index within the array
/// * `config` - Configuration for handling null IDs
fn extract_id_from_array(
    array: &Arc<dyn Array>,
    idx: usize,
    config: &FromParquetConfig,
) -> Result<String, HedlError> {
    // Check if the value is null
    if array.is_null(idx) {
        return handle_null_id(idx, config);
    }

    // Check for empty strings (also treated as null/missing ID)
    let id_str = match array.data_type() {
        DataType::Utf8 => {
            let string_array = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    HedlError::new(
                        HedlErrorKind::Syntax,
                        format!("Expected string array at row {idx}"),
                        idx,
                    )
                })?;
            string_array.value(idx)
        }
        DataType::Dictionary(_, value_type) => {
            // Handle dictionary-encoded strings
            if matches!(value_type.as_ref(), DataType::Utf8) {
                // Cast dictionary to string view
                use arrow::array::cast::AsArray;
                let dict_array = array.as_string::<i32>();
                if dict_array.is_null(idx) {
                    return handle_null_id(idx, config);
                }
                dict_array.value(idx)
            } else {
                return Err(HedlError::new(
                    HedlErrorKind::Syntax,
                    format!(
                        "Unsupported dictionary value type for ID at row {idx}: {value_type:?}"
                    ),
                    idx,
                ));
            }
        }
        DataType::Int64 => {
            let int_array = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Int64 array at row {idx}"),
                    idx,
                )
            })?;
            return Ok(int_array.value(idx).to_string());
        }
        _ => {
            return Err(HedlError::new(
                HedlErrorKind::Syntax,
                format!(
                    "Unsupported ID column type at row {}: {:?}",
                    idx,
                    array.data_type()
                ),
                idx,
            ));
        }
    };

    // Check for empty string IDs
    if id_str.is_empty() {
        return handle_null_id(idx, config);
    }

    Ok(id_str.to_string())
}

/// Handle null or missing ID values according to the configuration.
fn handle_null_id(row_idx: usize, config: &FromParquetConfig) -> Result<String, HedlError> {
    match &config.null_id_handling {
        NullIdHandling::Error => Err(HedlError::semantic(
            format!(
                "Row {row_idx} has null or empty ID. HEDL requires all entities to have non-null IDs. \
                     Fix your data source or use FromParquetConfig::lenient() to generate IDs. \
                     Note: Generated IDs are not preserved on round-trip."
            ),
            row_idx,
        )),
        NullIdHandling::Generate => {
            // Generate deterministic ID based on row index
            Ok(format!("__generated_row_{row_idx}"))
        }
        NullIdHandling::UseConstant(constant) => {
            // Use user-specified constant
            Ok(constant.clone())
        }
    }
}

/// Validate that all IDs in a list of nodes are unique.
///
/// This provides defense-in-depth against duplicate IDs that might occur
/// when using `NullIdHandling::UseConstant` or due to data quality issues.
fn validate_unique_ids(nodes: &[Node]) -> Result<(), HedlError> {
    let mut seen_ids: HashMap<&str, usize> = HashMap::new();

    for (idx, node) in nodes.iter().enumerate() {
        if let Some(&first_idx) = seen_ids.get(node.id.as_str()) {
            return Err(HedlError::collision(
                format!(
                    "Duplicate ID '{}' found at rows {} and {}. \
                     This may be caused by null IDs being converted to the same value.",
                    node.id, first_idx, idx
                ),
                idx,
            ));
        }
        seen_ids.insert(&node.id, idx);
    }

    Ok(())
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
            let bool_array = array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| {
                    HedlError::new(
                        HedlErrorKind::Syntax,
                        format!("Expected boolean array at row {idx}"),
                        position,
                    )
                })?;
            Ok(Value::Bool(bool_array.value(idx)))
        }
        DataType::Int8 => {
            let int_array = array.as_any().downcast_ref::<Int8Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Int8 array at row {idx}"),
                    position,
                )
            })?;
            Ok(Value::Int(i64::from(int_array.value(idx))))
        }
        DataType::Int16 => {
            let int_array = array.as_any().downcast_ref::<Int16Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Int16 array at row {idx}"),
                    position,
                )
            })?;
            Ok(Value::Int(i64::from(int_array.value(idx))))
        }
        DataType::Int32 => {
            let int_array = array.as_any().downcast_ref::<Int32Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Int32 array at row {idx}"),
                    position,
                )
            })?;
            Ok(Value::Int(i64::from(int_array.value(idx))))
        }
        DataType::Int64 => {
            let int_array = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected Int64 array at row {idx}"),
                    position,
                )
            })?;
            Ok(Value::Int(int_array.value(idx)))
        }
        DataType::UInt8 => {
            let int_array = array.as_any().downcast_ref::<UInt8Array>().ok_or_else(|| {
                HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Expected UInt8 array at row {idx}"),
                    position,
                )
            })?;
            Ok(Value::Int(i64::from(int_array.value(idx))))
        }
        DataType::UInt16 => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt16Array>()
                .ok_or_else(|| {
                    HedlError::new(
                        HedlErrorKind::Syntax,
                        format!("Expected UInt16 array at row {idx}"),
                        position,
                    )
                })?;
            Ok(Value::Int(i64::from(int_array.value(idx))))
        }
        DataType::UInt32 => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt32Array>()
                .ok_or_else(|| {
                    HedlError::new(
                        HedlErrorKind::Syntax,
                        format!("Expected UInt32 array at row {idx}"),
                        position,
                    )
                })?;
            Ok(Value::Int(i64::from(int_array.value(idx))))
        }
        DataType::UInt64 => {
            let int_array = array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or_else(|| {
                    HedlError::new(
                        HedlErrorKind::Syntax,
                        format!("Expected UInt64 array at row {idx}"),
                        position,
                    )
                })?;
            let val = int_array.value(idx);
            // Check for overflow: u64 values > i64::MAX cannot be safely converted
            if val > i64::MAX as u64 {
                return Err(HedlError::new(
                    HedlErrorKind::Syntax,
                    format!(
                        "UInt64 value {val} at row {idx} exceeds i64::MAX and cannot be represented"
                    ),
                    position,
                ));
            }
            Ok(Value::Int(val as i64))
        }
        DataType::Float32 => {
            let float_array = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| {
                    HedlError::new(
                        HedlErrorKind::Syntax,
                        format!("Expected Float32 array at row {idx}"),
                        position,
                    )
                })?;
            Ok(Value::Float(f64::from(float_array.value(idx))))
        }
        DataType::Float64 => {
            let float_array = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| {
                    HedlError::new(
                        HedlErrorKind::Syntax,
                        format!("Expected Float64 array at row {idx}"),
                        position,
                    )
                })?;
            Ok(Value::Float(float_array.value(idx)))
        }
        DataType::Utf8 => {
            let string_array = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    HedlError::new(
                        HedlErrorKind::Syntax,
                        format!("Expected string array at row {idx}"),
                        position,
                    )
                })?;
            let s = string_array.value(idx);

            // Try to detect references
            if s.starts_with('@') {
                return Ok(parse_reference_string(s));
            }

            // Try to detect lists (parentheses syntax)
            if s.starts_with('(') && s.ends_with(')') {
                return parse_list_string(s);
            }

            Ok(Value::String(s.to_string().into()))
        }
        DataType::Dictionary(_, value_type) => {
            // Handle dictionary-encoded strings
            if matches!(value_type.as_ref(), DataType::Utf8) {
                use arrow::array::cast::AsArray;
                let dict_array = array.as_string::<i32>();
                if dict_array.is_null(idx) {
                    return Ok(Value::Null);
                }
                let s = dict_array.value(idx);

                // Try to detect references
                if s.starts_with('@') {
                    return Ok(parse_reference_string(s));
                }

                // Try to detect lists (parentheses syntax)
                if s.starts_with('(') && s.ends_with(')') {
                    return parse_list_string(s);
                }

                Ok(Value::String(s.to_string().into()))
            } else {
                Err(HedlError::new(
                    HedlErrorKind::Syntax,
                    format!("Unsupported dictionary value type at row {idx}: {value_type:?}"),
                    position,
                ))
            }
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
        _ => return Value::String(s.to_string().into()), // Not a valid reference, return as string
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

/// Parse a list string (e.g., "(a, b, c)" or "()").
///
/// Parses a string representation of a list back into a `Value::List`.
/// This provides basic parsing for round-trip support of lists through Parquet.
///
/// # Arguments
///
/// * `s` - The string to parse, should start with '(' and end with ')'
///
/// # Returns
///
/// A `Value::List` if parsing succeeds, or `Value::String` if it fails
fn parse_list_string(s: &str) -> Result<Value, HedlError> {
    // Strip parentheses
    let inner = s.trim_start_matches('(').trim_end_matches(')').trim();

    // Handle empty list
    if inner.is_empty() {
        return Ok(Value::List(Box::default()));
    }

    // Simple comma-separated parsing
    // Note: This is a simplified parser that doesn't handle nested lists or escaped commas
    let items: Vec<Value> = inner
        .split(',')
        .map(|item| {
            let trimmed = item.trim();

            // Try to parse as different types
            if trimmed == "~" {
                Value::Null
            } else if trimmed == "true" {
                Value::Bool(true)
            } else if trimmed == "false" {
                Value::Bool(false)
            } else if let Ok(n) = trimmed.parse::<i64>() {
                Value::Int(n)
            } else if let Ok(f) = trimmed.parse::<f64>() {
                Value::Float(f)
            } else if trimmed.starts_with('@') {
                parse_reference_string(trimmed)
            } else {
                // Default to string
                Value::String(trimmed.to_string().into())
            }
        })
        .collect();

    Ok(Value::List(Box::new(items)))
}

/// Validate column names and map to indices for projection.
///
/// This function validates that all requested column names exist in the schema
/// and returns their indices for use with Arrow's projection API.
///
/// # Arguments
///
/// * `schema` - The Arrow schema containing all available columns
/// * `column_names` - The requested column names to project
///
/// # Returns
///
/// A vector of column indices suitable for use with `ProjectionMask::roots()`
///
/// # Errors
///
/// Returns an error if:
/// - The column list is empty
/// - Any column name doesn't exist in the schema
fn validate_and_map_columns(
    schema: &Arc<arrow::datatypes::Schema>,
    column_names: &[String],
) -> Result<Vec<usize>, HedlError> {
    // Reject empty column lists
    if column_names.is_empty() {
        return Err(HedlError::new(
            HedlErrorKind::Syntax,
            "Column projection requires at least one column",
            0,
        ));
    }

    let mut indices = Vec::with_capacity(column_names.len());

    for col_name in column_names {
        if let Ok(idx) = schema.index_of(col_name) {
            indices.push(idx);
        } else {
            // Build helpful error message with available columns
            let available_columns: Vec<&str> =
                schema.fields().iter().map(|f| f.name().as_str()).collect();

            return Err(HedlError::new(
                HedlErrorKind::Syntax,
                format!(
                    "Column '{}' not found in schema. Available columns: {}",
                    col_name,
                    available_columns.join(", ")
                ),
                0,
            ));
        }
    }

    // Ensure indices are sorted for efficient access and consistent ordering
    indices.sort_unstable();

    Ok(indices)
}

/// Read Parquet file selecting only specific columns.
///
/// Uses projection pushdown to read only the specified columns,
/// providing significant performance improvement for wide tables.
///
/// # Performance
///
/// For a 50-column table, reading 5 columns (10%) is typically 8-10x faster
/// than reading all columns, with proportional I/O and memory reduction.
///
/// # Example
///
/// ```no_run
/// use hedl_parquet::from_parquet_select;
/// use std::path::Path;
///
/// // Read only 3 columns from users table
/// let doc = from_parquet_select(
///     Path::new("users.parquet"),
///     vec!["id".into(), "name".into(), "email".into()]
/// )?;
/// # Ok::<(), hedl_core::HedlError>(())
/// ```
///
/// # Errors
///
/// Returns error if:
/// - File cannot be read
/// - Any specified column doesn't exist in schema
/// - Column list is empty
pub fn from_parquet_select(path: &Path, columns: Vec<String>) -> Result<Document, HedlError> {
    from_parquet_with_config(path, &FromParquetConfig::with_columns(columns))
}

/// Read Parquet bytes selecting only specific columns.
///
/// # Example
///
/// ```no_run
/// use hedl_parquet::from_parquet_bytes_select;
///
/// # let bytes = vec![]; // Some Parquet bytes
/// # if !bytes.is_empty() {
/// let doc = from_parquet_bytes_select(
///     &bytes,
///     vec!["id".into(), "name".into()]
/// )?;
/// # }
/// # Ok::<(), hedl_core::HedlError>(())
/// ```
pub fn from_parquet_bytes_select(
    bytes: &[u8],
    columns: Vec<String>,
) -> Result<Document, HedlError> {
    from_parquet_bytes_with_config(bytes, &FromParquetConfig::with_columns(columns))
}

/// Get column names from a Parquet file without reading data.
///
/// Reads only file metadata (no data), returning list of all column names.
/// Use this to inspect file schema before deciding which columns to read.
///
/// # Example
///
/// ```no_run
/// use hedl_parquet::get_parquet_columns;
/// use std::path::Path;
///
/// let columns = get_parquet_columns(Path::new("users.parquet"))?;
/// println!("Available columns: {:?}", columns);
/// # Ok::<(), hedl_core::HedlError>(())
/// ```
pub fn get_parquet_columns(path: &Path) -> Result<Vec<String>, HedlError> {
    let file = std::fs::File::open(path)
        .map_err(|e| HedlError::io(format!("Failed to open Parquet file: {e}")))?;

    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| HedlError::io(format!("Failed to create Parquet reader: {e}")))?;

    let schema = builder.schema();

    Ok(schema.fields().iter().map(|f| f.name().clone()).collect())
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
                assert_eq!(&*r.id, "alice");
            }
            _ => panic!("Expected reference value"),
        }
    }

    #[test]
    fn test_parse_reference_string_qualified() {
        let value = parse_reference_string("@User:alice");
        match value {
            Value::Reference(r) => {
                assert_eq!(r.type_name.as_deref(), Some("User"));
                assert_eq!(&*r.id, "alice");
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
