// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Main CSV to HEDL conversion logic

use crate::error::{CsvError, Result};
use crate::from_csv::config::FromCsvConfig;
use crate::from_csv::parsing::{parse_csv_value, parse_csv_value_with_type};
use crate::from_csv::schema_inference::{infer_column_types, ColumnType};
use crate::from_csv::validation::{validate_cell, validate_headers, CsvSizeTracker};
use hedl_core::{Document, Item, MatrixList, Node};
use std::io::Read;

/// Parse CSV string into a HEDL document with default configuration.
///
/// This is a convenience wrapper around `from_csv_with_config` using default settings.
pub fn from_csv(csv: &str, type_name: &str, schema: &[&str]) -> Result<Document> {
    from_csv_with_config(csv, type_name, schema, FromCsvConfig::default())
}

/// Parse CSV string into a HEDL document with custom configuration.
///
/// This function provides full control over CSV parsing behavior through `FromCsvConfig`.
///
/// # Arguments
///
/// * `csv` - The CSV string to parse
/// * `type_name` - The HEDL type name for rows
/// * `schema` - Column names excluding the 'id' column
/// * `config` - Configuration controlling delimiter, headers, trimming, and row limits
///
/// # Examples
///
/// ## Tab-Separated Values (TSV)
///
/// ```
/// use hedl_csv::{from_csv_with_config, FromCsvConfig};
///
/// let tsv_data = "id\tname\tage\n1\tAlice\t30";
/// let config = FromCsvConfig {
///     delimiter: b'\t',
///     ..Default::default()
/// };
/// let doc = from_csv_with_config(tsv_data, "Person", &["name", "age"], config).unwrap();
/// ```
///
/// ## Custom Row Limit
///
/// ```
/// use hedl_csv::{from_csv_with_config, FromCsvConfig};
///
/// let config = FromCsvConfig {
///     max_rows: 10_000_000, // Allow up to 10M rows
///     ..Default::default()
/// };
/// let csv_data = "id,value\n1,test";
/// let doc = from_csv_with_config(csv_data, "Item", &["value"], config).unwrap();
/// ```
///
/// ## Disable Whitespace Trimming
///
/// ```
/// use hedl_csv::{from_csv_with_config, FromCsvConfig};
/// use hedl_core::Value;
///
/// let csv_data = "id,name\n1,  Alice  ";
/// let config = FromCsvConfig {
///     trim: false,
///     ..Default::default()
/// };
/// let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();
///
/// let list = doc.get("persons").unwrap().as_list().unwrap();
/// assert_eq!(list.rows[0].fields[1], Value::String("  Alice  ".to_string().into()));
/// ```
///
/// # See Also
///
/// - `from_csv` - Convenience function with default configuration
/// - `from_csv_reader_with_config` - For streaming from files/network
pub fn from_csv_with_config(
    csv: &str,
    type_name: &str,
    schema: &[&str],
    config: FromCsvConfig,
) -> Result<Document> {
    from_csv_reader_with_config(csv.as_bytes(), type_name, schema, config)
}

/// Parse CSV from a reader into a HEDL document with default configuration.
///
/// This function is useful for processing CSV files or network streams without
/// loading the entire content into memory first.
///
/// # Arguments
///
/// * `reader` - Any type implementing `Read` (e.g., `File`, `TcpStream`, `&[u8]`)
/// * `type_name` - The HEDL type name for rows
/// * `schema` - Column names excluding the 'id' column
///
/// # Examples
///
/// ## Reading from a File
///
/// ```no_run
/// use hedl_csv::from_csv_reader;
/// use std::fs::File;
///
/// let file = File::open("data.csv").unwrap();
/// let doc = from_csv_reader(file, "Person", &["name", "age"]).unwrap();
/// ```
///
/// ## Reading from a Byte Slice
///
/// ```
/// use hedl_csv::from_csv_reader;
///
/// let csv_bytes = b"id,name\n1,Alice";
/// let doc = from_csv_reader(&csv_bytes[..], "Person", &["name"]).unwrap();
/// ```
///
/// ## Reading from Standard Input
///
/// ```no_run
/// use hedl_csv::from_csv_reader;
/// use std::io;
///
/// let stdin = io::stdin();
/// let doc = from_csv_reader(stdin.lock(), "Record", &["field1", "field2"]).unwrap();
/// ```
///
/// # Performance
///
/// This function uses streaming I/O to minimize memory usage. The CSV data is
/// processed row-by-row without buffering the entire file.
///
/// # See Also
///
/// - `from_csv_reader_with_config` - For custom delimiters and limits
/// - `from_csv` - For parsing CSV strings
pub fn from_csv_reader<R: Read>(reader: R, type_name: &str, schema: &[&str]) -> Result<Document> {
    from_csv_reader_with_config(reader, type_name, schema, FromCsvConfig::default())
}

/// Converts a CSV file to a HEDL Document with custom configuration.
///
/// This is the main conversion function that handles CSV parsing with
/// configurable options for delimiters, headers, trimming, and limits.
pub fn from_csv_reader_with_config<R: Read>(
    reader: R,
    type_name: &str,
    schema: &[&str],
    config: FromCsvConfig,
) -> Result<Document> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(config.delimiter)
        .has_headers(config.has_headers)
        .trim(if config.trim {
            csv::Trim::All
        } else {
            csv::Trim::None
        })
        .from_reader(reader);

    let mut doc = Document::new((2, 0));

    // Create schema with 'id' column
    let mut full_schema = vec!["id".to_string()];
    full_schema.extend(schema.iter().map(|s| (*s).to_string()));

    // Register the struct type
    doc.structs
        .insert(type_name.to_string(), full_schema.clone());

    // Create matrix list
    let mut matrix_list = MatrixList::new(type_name, full_schema.clone());

    // VALIDATE HEADERS if has_headers is enabled
    let headers = csv_reader.headers().map_err(|e| CsvError::ParseError {
        line: 0,
        message: e.to_string(),
    })?;

    validate_headers(headers, &config)?;

    // Initialize size tracker
    let mut size_tracker = CsvSizeTracker::new(config.max_total_size);

    // Track header size
    let header_size: usize = headers.iter().map(str::len).sum();
    size_tracker.bytes_read += header_size;

    // If schema inference is enabled, collect records first
    let _inferred_types = if config.infer_schema {
        // Collect records for sampling
        let mut all_records = Vec::new();
        for (record_idx, result) in csv_reader.records().enumerate() {
            // Security: Limit row count to prevent memory exhaustion
            if record_idx >= config.max_rows {
                return Err(CsvError::SecurityLimit {
                    limit: config.max_rows,
                    actual: record_idx + 1,
                });
            }

            let record = result.map_err(|e| CsvError::ParseError {
                line: record_idx + 1,
                message: e.to_string(),
            })?;

            if record.is_empty() {
                continue;
            }

            // VALIDATE TOTAL SIZE
            size_tracker.track_record(&record)?;

            // VALIDATE EACH CELL
            for (col_idx, cell) in record.iter().enumerate() {
                validate_cell(cell, record_idx + 1, col_idx, &config)?;
            }

            // Convert StringRecord to Vec<String>
            let row: Vec<String> = record
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            all_records.push(row);
        }

        // Infer column types from sampled records
        let types = infer_column_types(&all_records, config.sample_rows);

        // Process all records with inferred types
        for (record_idx, row) in all_records.iter().enumerate() {
            // First column is the ID
            let id = row
                .first()
                .ok_or_else(|| CsvError::MissingColumn("id".to_string()))?;

            if id.is_empty() {
                return Err(CsvError::EmptyId {
                    row: record_idx + 1,
                });
            }

            // Parse ALL fields (including ID) with inferred types
            let mut fields = Vec::new();
            for (field_idx, field) in row.iter().enumerate() {
                let col_type = types.get(field_idx).copied().unwrap_or(ColumnType::String);
                let value = parse_csv_value_with_type(field, col_type).map_err(|e| {
                    e.with_context(format!(
                        "in column '{}' at line {}",
                        full_schema.get(field_idx).unwrap_or(&"unknown".to_string()),
                        record_idx + 1
                    ))
                })?;
                fields.push(value);
            }

            // Check field count matches full schema (including ID)
            if fields.len() != full_schema.len() {
                return Err(CsvError::WidthMismatch {
                    expected: full_schema.len(),
                    actual: fields.len(),
                    row: record_idx + 1,
                });
            }

            let node = Node::new(type_name, id, fields);
            matrix_list.add_row(node);
        }

        types
    } else {
        // Standard parsing without schema inference
        for (record_idx, result) in csv_reader.records().enumerate() {
            // Security: Limit row count to prevent memory exhaustion
            if record_idx >= config.max_rows {
                return Err(CsvError::SecurityLimit {
                    limit: config.max_rows,
                    actual: record_idx + 1,
                });
            }

            let record = result.map_err(|e| CsvError::ParseError {
                line: record_idx + 1,
                message: e.to_string(),
            })?;

            if record.is_empty() {
                continue;
            }

            // VALIDATE TOTAL SIZE
            size_tracker.track_record(&record)?;

            // VALIDATE EACH CELL
            for (col_idx, cell) in record.iter().enumerate() {
                validate_cell(cell, record_idx + 1, col_idx, &config)?;
            }

            // First column is the ID
            let id = record
                .get(0)
                .ok_or_else(|| CsvError::MissingColumn("id".to_string()))?;

            if id.is_empty() {
                return Err(CsvError::EmptyId {
                    row: record_idx + 1,
                });
            }

            // Parse ALL fields (including ID) per SPEC
            let mut fields = Vec::new();
            for (field_idx, field) in record.iter().enumerate() {
                let value = parse_csv_value(field).map_err(|e| {
                    e.with_context(format!(
                        "in column '{}' at line {}",
                        full_schema.get(field_idx).unwrap_or(&"unknown".to_string()),
                        record_idx + 1
                    ))
                })?;
                fields.push(value);
            }

            // Check field count matches full schema (including ID)
            if fields.len() != full_schema.len() {
                return Err(CsvError::WidthMismatch {
                    expected: full_schema.len(),
                    actual: fields.len(),
                    row: record_idx + 1,
                });
            }

            let node = Node::new(type_name, id, fields);
            matrix_list.add_row(node);
        }

        Vec::new()
    };

    // Add matrix list to document with custom or default key
    let list_key = config
        .list_key
        .unwrap_or_else(|| format!("{}s", type_name.to_lowercase()));

    doc.root.insert(list_key, Item::List(matrix_list));

    Ok(doc)
}

// Note: parse_csv_value function is in parsing.rs module
