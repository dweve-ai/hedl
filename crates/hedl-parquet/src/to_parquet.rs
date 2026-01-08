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

//! Convert HEDL documents to Parquet format.
//!
//! # Position Encoding Strategy
//!
//! This module preserves row order when converting HEDL documents to Parquet:
//!
//! ## Ordered Data (Default)
//!
//! Row position is **implicitly preserved** through sequential processing:
//! - `MatrixList.rows[i]` → Parquet row `i`
//! - Row order in HEDL matches row order in Parquet file
//! - No additional storage overhead
//! - Guaranteed by implementation (no reordering occurs)
//!
//! ## Example
//!
//! ```rust
//! use hedl_core::{Document, MatrixList, Node, Value, Item};
//! use hedl_parquet::to_parquet_bytes;
//!
//! let mut doc = Document::new((1, 0));
//! let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
//!
//! // Rows are added in order
//! list.add_row(Node::new("User", "alice", vec![
//!     Value::String("alice".to_string()),
//!     Value::String("Alice".to_string()),
//! ]));
//! list.add_row(Node::new("User", "bob", vec![
//!     Value::String("bob".to_string()),
//!     Value::String("Bob".to_string()),
//! ]));
//!
//! doc.root.insert("users".to_string(), Item::List(list));
//!
//! // Row order is preserved in Parquet file
//! let bytes = to_parquet_bytes(&doc).unwrap();
//! // alice is row 0, bob is row 1
//! ```
//!
//! ## Explicit Position Column
//!
//! For scenarios requiring explicit position tracking, add a dedicated column:
//!
//! ```rust
//! use hedl_core::{MatrixList, Node, Value};
//!
//! let schema = vec!["id".to_string(), "position".to_string(), "data".to_string()];
//! let mut list = MatrixList::new("Item", schema);
//!
//! for (i, data) in vec!["first", "second", "third"].iter().enumerate() {
//!     list.add_row(Node::new("Item", format!("item{}", i), vec![
//!         Value::String(format!("item{}", i)),
//!         Value::Int(i as i64),  // Explicit position
//!         Value::String(data.to_string()),
//!     ]));
//! }
//! ```
//!
//! See `POSITION_ENCODING.md` for detailed documentation.

use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, Float64Array, Int64Array, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, Encoding};
use parquet::file::properties::{WriterProperties, WriterVersion};

use hedl_core::{Document, HedlError, HedlErrorKind, Item, MatrixList, Node, Value};

/// Configuration for Parquet writing.
#[derive(Debug, Clone)]
pub struct ToParquetConfig {
    /// Compression algorithm to use.
    pub compression: Compression,
    /// Writer version.
    pub writer_version: WriterVersion,
    /// Encoding for string columns.
    pub string_encoding: Encoding,
}

impl Default for ToParquetConfig {
    fn default() -> Self {
        Self {
            compression: Compression::SNAPPY,
            writer_version: WriterVersion::PARQUET_2_0,
            string_encoding: Encoding::PLAIN,
        }
    }
}

/// Write a HEDL document to a Parquet file.
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
/// * `path` - Path to the output Parquet file
///
/// # Errors
///
/// Returns an error if:
/// - The document contains unsupported structures
/// - Parquet writing fails
/// - I/O operations fail
///
/// # Example
///
/// ```no_run
/// use hedl_core::Document;
/// use hedl_parquet::to_parquet;
/// use std::path::Path;
///
/// let doc = Document::new((1, 0));
/// to_parquet(&doc, Path::new("output.parquet")).unwrap();
/// ```
pub fn to_parquet(doc: &Document, path: &Path) -> Result<(), HedlError> {
    to_parquet_with_config(doc, path, &ToParquetConfig::default())
}

/// Write a HEDL document to a Parquet file with custom configuration.
pub fn to_parquet_with_config(
    doc: &Document,
    path: &Path,
    config: &ToParquetConfig,
) -> Result<(), HedlError> {
    let bytes = to_parquet_bytes_with_config(doc, config)?;

    std::fs::write(path, bytes).map_err(|e| {
        HedlError::io(format!("Failed to write Parquet file: {}", e))
    })
}

/// Convert a HEDL document to Parquet bytes.
///
/// # Example
///
/// ```
/// use hedl_core::Document;
/// use hedl_parquet::to_parquet_bytes;
///
/// let doc = Document::new((1, 0));
/// let bytes = to_parquet_bytes(&doc).unwrap();
/// ```
pub fn to_parquet_bytes(doc: &Document) -> Result<Vec<u8>, HedlError> {
    to_parquet_bytes_with_config(doc, &ToParquetConfig::default())
}

/// Convert a HEDL document to Parquet bytes with custom configuration.
pub fn to_parquet_bytes_with_config(
    doc: &Document,
    config: &ToParquetConfig,
) -> Result<Vec<u8>, HedlError> {
    // Pre-allocate buffer with estimated size based on typical Parquet overhead
    // Parquet header/footer + metadata typically requires 8-16 KB minimum
    let mut buffer = Vec::with_capacity(16 * 1024);

    // Convert the first matrix list found (Parquet supports one table per file)
    // Store the HEDL key in metadata for round-trip preservation
    for (key, item) in &doc.root {
        if let Item::List(matrix_list) = item {
            write_matrix_list_to_buffer(matrix_list, key, &mut buffer, config)?;
            return Ok(buffer);
        }
    }

    // If we have only key-value pairs (no matrix lists), create a metadata table
    let has_lists = doc.root.values().any(|item| matches!(item, Item::List(_)));
    if !has_lists && !doc.root.is_empty() {
        write_metadata_to_buffer(&doc.root, &mut buffer, config)?;
    }

    Ok(buffer)
}

/// Write a matrix list to the buffer as a Parquet table.
fn write_matrix_list_to_buffer(
    matrix_list: &MatrixList,
    hedl_key: &str,
    buffer: &mut Vec<u8>,
    config: &ToParquetConfig,
) -> Result<(), HedlError> {
    if matrix_list.rows.is_empty() {
        return Ok(());
    }

    // Build Arrow schema from HEDL schema with metadata
    let schema = build_schema_from_matrix_list(matrix_list, hedl_key)?;

    // Convert nodes to record batch
    let record_batch = build_record_batch_from_nodes(&matrix_list.rows, &schema)?;

    // Configure writer properties with metadata
    let mut props_builder = WriterProperties::builder()
        .set_compression(config.compression)
        .set_writer_version(config.writer_version);

    // Add metadata as key-value pairs
    props_builder = props_builder.set_key_value_metadata(Some(vec![
        parquet::file::metadata::KeyValue::new(
            "hedl:type_name".to_string(),
            matrix_list.type_name.clone(),
        ),
        parquet::file::metadata::KeyValue::new("hedl:key".to_string(), hedl_key.to_string()),
    ]));

    let props = props_builder.build();

    // Write to buffer
    let mut writer =
        ArrowWriter::try_new(buffer, Arc::clone(&schema), Some(props)).map_err(|e| {
            HedlError::io(format!("Failed to create Parquet writer: {}", e))
        })?;

    writer.write(&record_batch).map_err(|e| {
        HedlError::io(format!("Failed to write record batch: {}", e))
    })?;

    writer.close().map_err(|e| {
        HedlError::io(format!("Failed to close Parquet writer: {}", e))
    })?;

    Ok(())
}

/// Build Arrow schema from a matrix list.
fn build_schema_from_matrix_list(
    matrix_list: &MatrixList,
    hedl_key: &str,
) -> Result<Arc<Schema>, HedlError> {
    // Pre-allocate fields vector with exact capacity (one field per column)
    let mut fields = Vec::with_capacity(matrix_list.schema.len());

    // Per SPEC.md: MatrixList.schema includes all column names with ID first
    // Node.fields contains ALL values including ID (first column)
    for (idx, col_name) in matrix_list.schema.iter().enumerate() {
        // Infer type from first row's field value
        let data_type = if let Some(first_row) = matrix_list.rows.first() {
            if let Some(value) = first_row.fields.get(idx) {
                infer_arrow_type(value)
            } else {
                DataType::Utf8
            }
        } else {
            DataType::Utf8
        };

        // First column (ID) should be non-nullable
        let nullable = idx > 0;
        fields.push(Field::new(col_name, data_type, nullable));
    }

    // Add metadata with HEDL type name and key
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("hedl:type_name".to_string(), matrix_list.type_name.clone());
    metadata.insert("hedl:key".to_string(), hedl_key.to_string());

    Ok(Arc::new(Schema::new(fields).with_metadata(metadata)))
}

/// Infer Arrow data type from HEDL value.
fn infer_arrow_type(value: &Value) -> DataType {
    match value {
        Value::Null => DataType::Utf8,
        Value::Bool(_) => DataType::Boolean,
        Value::Int(_) => DataType::Int64,
        Value::Float(_) => DataType::Float64,
        Value::String(_) => DataType::Utf8,
        Value::Reference(_) => DataType::Utf8,
        Value::Expression(_) => DataType::Utf8,
        Value::Tensor(_) => DataType::Utf8, // Serialize tensors as strings
    }
}

/// Build a RecordBatch from nodes.
///
/// # Position Preservation
///
/// This function preserves row order by processing nodes sequentially:
/// - `nodes[i]` becomes Parquet row `i`
/// - Column arrays are built by iterating nodes in order
/// - No reordering or sorting occurs
///
/// This guarantees that row position is maintained from HEDL to Parquet.
fn build_record_batch_from_nodes(
    nodes: &[Node],
    schema: &Arc<Schema>,
) -> Result<RecordBatch, HedlError> {
    // Pre-allocate columns vector with exact capacity (one column per field)
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(schema.fields().len());

    // Per SPEC.md: Node.fields contains ALL values including ID (first column)
    // Process fields in order to preserve row position
    for (field_idx, field) in schema.fields().iter().enumerate() {
        let array: ArrayRef = build_array_for_field(nodes, field_idx, field.data_type())?;
        columns.push(array);
    }

    RecordBatch::try_new(Arc::clone(schema), columns).map_err(|e| {
        HedlError::new(
            HedlErrorKind::Syntax,
            format!("Failed to create record batch: {}", e),
            0,
        )
    })
}

/// Build an Arrow array for a specific field index across all nodes.
///
/// # Position Preservation
///
/// Processes nodes sequentially to maintain row order:
/// - Iterates `nodes` in order from index 0 to n-1
/// - Extracts field value at `field_idx` for each node
/// - Builds columnar array preserving node order
///
/// This ensures that array value at index `i` corresponds to `nodes[i]`.
fn build_array_for_field(
    nodes: &[Node],
    field_idx: usize,
    data_type: &DataType,
) -> Result<ArrayRef, HedlError> {
    match data_type {
        DataType::Boolean => {
            let values: Vec<Option<bool>> = nodes
                .iter()
                .map(|node| {
                    node.fields.get(field_idx).and_then(|v| match v {
                        Value::Bool(b) => Some(*b),
                        Value::Null => None,
                        _ => Some(false),
                    })
                })
                .collect();
            Ok(Arc::new(BooleanArray::from(values)))
        }
        DataType::Int64 => {
            let values: Vec<Option<i64>> = nodes
                .iter()
                .map(|node| {
                    node.fields.get(field_idx).and_then(|v| match v {
                        Value::Int(n) => Some(*n),
                        Value::Null => None,
                        _ => Some(0),
                    })
                })
                .collect();
            Ok(Arc::new(Int64Array::from(values)))
        }
        DataType::Float64 => {
            let values: Vec<Option<f64>> = nodes
                .iter()
                .map(|node| {
                    node.fields.get(field_idx).and_then(|v| match v {
                        Value::Float(f) => Some(*f),
                        Value::Int(n) => Some(*n as f64),
                        Value::Null => None,
                        _ => Some(0.0),
                    })
                })
                .collect();
            Ok(Arc::new(Float64Array::from(values)))
        }
        DataType::Utf8 => {
            // Pre-allocate StringBuilder with capacity for all rows
            // Average string length estimate: 32 bytes per string
            let mut builder = StringBuilder::with_capacity(nodes.len(), nodes.len() * 32);
            for node in nodes {
                if let Some(value) = node.fields.get(field_idx) {
                    match value {
                        Value::Null => builder.append_null(),
                        Value::String(s) => builder.append_value(s),
                        Value::Reference(r) => builder.append_value(r.to_ref_string()),
                        Value::Expression(e) => builder.append_value(format!("$({})", e)),
                        Value::Tensor(t) => {
                            // Serialize tensor as JSON-like string
                            builder.append_value(format!("{:?}", t.flatten()))
                        }
                        other => builder.append_value(other.to_string()),
                    }
                } else {
                    builder.append_null();
                }
            }
            Ok(Arc::new(builder.finish()))
        }
        _ => Err(HedlError::new(
            HedlErrorKind::Syntax,
            format!("Unsupported Arrow data type: {:?}", data_type),
            0,
        )),
    }
}

/// Write key-value metadata to buffer as a single-row table.
fn write_metadata_to_buffer(
    root: &std::collections::BTreeMap<String, Item>,
    buffer: &mut Vec<u8>,
    config: &ToParquetConfig,
) -> Result<(), HedlError> {
    // Build schema with key and value columns
    let schema = Arc::new(Schema::new(vec![
        Field::new("key", DataType::Utf8, false),
        Field::new("value", DataType::Utf8, true),
    ]));

    // Build arrays with pre-allocated capacity
    // Each metadata entry has a key and value string (estimate ~32 bytes each)
    let num_items = root.len();
    let mut key_builder = StringBuilder::with_capacity(num_items, num_items * 32);
    let mut value_builder = StringBuilder::with_capacity(num_items, num_items * 32);

    for (key, item) in root {
        key_builder.append_value(key);

        match item {
            Item::Scalar(v) => value_builder.append_value(v.to_string()),
            Item::Object(_) => value_builder.append_value("[object]"),
            Item::List(_) => value_builder.append_value("[list]"),
        }
    }

    let key_array = Arc::new(key_builder.finish()) as ArrayRef;
    let value_array = Arc::new(value_builder.finish()) as ArrayRef;

    let record_batch = RecordBatch::try_new(Arc::clone(&schema), vec![key_array, value_array])
        .map_err(|e| {
            HedlError::new(
                HedlErrorKind::Syntax,
                format!("Failed to create metadata record batch: {}", e),
                0,
            )
        })?;

    // Configure writer properties
    let props = WriterProperties::builder()
        .set_compression(config.compression)
        .set_writer_version(config.writer_version)
        .build();

    // Write to buffer
    let mut writer =
        ArrowWriter::try_new(buffer, Arc::clone(&schema), Some(props)).map_err(|e| {
            HedlError::io(format!("Failed to create metadata Parquet writer: {}", e))
        })?;

    writer.write(&record_batch).map_err(|e| {
        HedlError::io(format!("Failed to write metadata record batch: {}", e))
    })?;

    writer.close().map_err(|e| {
        HedlError::io(format!("Failed to close metadata Parquet writer: {}", e))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Document, MatrixList, Node, Value};

    #[test]
    fn test_to_parquet_bytes_empty_doc() {
        let doc = Document::new((1, 0));
        let result = to_parquet_bytes(&doc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_parquet_bytes_with_matrix_list() {
        let mut doc = Document::new((1, 0));
        let mut matrix_list = MatrixList::new(
            "User",
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );

        // Per SPEC.md: Node.fields contains ALL values including ID (first column)
        let node1 = Node::new(
            "User",
            "alice",
            vec![
                Value::String("alice".to_string()),
                Value::String("Alice".to_string()),
                Value::Int(30),
            ],
        );
        let node2 = Node::new(
            "User",
            "bob",
            vec![
                Value::String("bob".to_string()),
                Value::String("Bob".to_string()),
                Value::Int(25),
            ],
        );

        matrix_list.add_row(node1);
        matrix_list.add_row(node2);
        doc.root
            .insert("users".to_string(), Item::List(matrix_list));

        let result = to_parquet_bytes(&doc);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_to_parquet_bytes_with_metadata() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "version".to_string(),
            Item::Scalar(Value::String("1.0".to_string())),
        );
        doc.root
            .insert("count".to_string(), Item::Scalar(Value::Int(42)));

        let result = to_parquet_bytes(&doc);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }
}
