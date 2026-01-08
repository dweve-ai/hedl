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

//! Convert HEDL documents to CSV format.

use crate::error::{CsvError, Result};
use hedl_core::{Document, MatrixList, Tensor, Value};
use std::io::Write;

/// Configuration for CSV output.
#[derive(Debug, Clone)]
pub struct ToCsvConfig {
    /// Field delimiter (default: ',')
    pub delimiter: u8,
    /// Include header row (default: true)
    pub include_headers: bool,
    /// Quote style for fields (default: necessary)
    pub quote_style: csv::QuoteStyle,
}

impl Default for ToCsvConfig {
    fn default() -> Self {
        Self {
            delimiter: b',',
            include_headers: true,
            quote_style: csv::QuoteStyle::Necessary,
        }
    }
}

/// Convert a HEDL document to CSV string.
///
/// # Example
/// ```no_run
/// use hedl_core::Document;
/// use hedl_csv::to_csv;
///
/// let doc = Document::new((1, 0));
/// let csv_string = to_csv(&doc).unwrap();
/// ```
pub fn to_csv(doc: &Document) -> Result<String> {
    to_csv_with_config(doc, ToCsvConfig::default())
}

/// Convert a specific matrix list from a HEDL document to CSV string.
///
/// Exports only the specified named list to CSV format, with configurable
/// header row and delimiter options. Nested children are skipped with a warning
/// (logged as part of error handling if strict mode desired).
///
/// # Arguments
///
/// * `doc` - The HEDL document
/// * `list_name` - The name of the matrix list to export (e.g., "people", "items")
///
/// # Returns
///
/// A CSV-formatted string containing the specified list data
///
/// # Errors
///
/// Returns `HedlError` if:
/// - The named list does not exist in the document
/// - The list is not a `MatrixList` (i.e., it's a scalar or object)
/// - CSV serialization fails
///
/// # Example
/// ```no_run
/// use hedl_core::Document;
/// use hedl_csv::to_csv_list;
///
/// let doc = Document::new((1, 0));
/// let csv_string = to_csv_list(&doc, "people").unwrap();
/// println!("{}", csv_string);
/// ```
pub fn to_csv_list(doc: &Document, list_name: &str) -> Result<String> {
    to_csv_list_with_config(doc, list_name, ToCsvConfig::default())
}

/// Convert a specific matrix list from a HEDL document to CSV string with custom configuration.
///
/// # Arguments
///
/// * `doc` - The HEDL document
/// * `list_name` - The name of the matrix list to export
/// * `config` - Custom CSV configuration (delimiter, headers, quote style)
///
/// # Example
/// ```no_run
/// use hedl_core::Document;
/// use hedl_csv::{to_csv_list_with_config, ToCsvConfig};
///
/// let doc = Document::new((1, 0));
/// let config = ToCsvConfig {
///     delimiter: b';',
///     include_headers: true,
///     ..Default::default()
/// };
/// let csv_string = to_csv_list_with_config(&doc, "people", config).unwrap();
/// ```
pub fn to_csv_list_with_config(
    doc: &Document,
    list_name: &str,
    config: ToCsvConfig,
) -> Result<String> {
    let estimated_size = estimate_list_csv_size(doc, list_name);
    let mut buffer = Vec::with_capacity(estimated_size);

    to_csv_list_writer_with_config(doc, list_name, &mut buffer, config)?;
    String::from_utf8(buffer).map_err(|_| {
        CsvError::InvalidUtf8 {
            context: "CSV output".to_string(),
        }
    })
}

/// Write a specific matrix list to CSV format using a writer.
///
/// # Arguments
///
/// * `doc` - The HEDL document
/// * `list_name` - The name of the matrix list to export
/// * `writer` - The output writer (file, buffer, etc.)
///
/// # Example
/// ```no_run
/// use hedl_core::Document;
/// use hedl_csv::to_csv_list_writer;
/// use std::fs::File;
///
/// let doc = Document::new((1, 0));
/// let file = File::create("output.csv").unwrap();
/// to_csv_list_writer(&doc, "people", file).unwrap();
/// ```
pub fn to_csv_list_writer<W: Write>(
    doc: &Document,
    list_name: &str,
    writer: W,
) -> Result<()> {
    to_csv_list_writer_with_config(doc, list_name, writer, ToCsvConfig::default())
}

/// Write a specific matrix list to CSV format with custom configuration.
///
/// # Arguments
///
/// * `doc` - The HEDL document
/// * `list_name` - The name of the matrix list to export
/// * `writer` - The output writer (file, buffer, etc.)
/// * `config` - Custom CSV configuration
pub fn to_csv_list_writer_with_config<W: Write>(
    doc: &Document,
    list_name: &str,
    writer: W,
    config: ToCsvConfig,
) -> Result<()> {
    // Find the specified list
    let matrix_list = find_matrix_list_by_name(doc, list_name)?;

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(config.delimiter)
        .quote_style(config.quote_style)
        .from_writer(writer);

    // Write header row if requested
    if config.include_headers {
        wtr.write_record(&matrix_list.schema).map_err(|e| {
            CsvError::Other(format!(
                "Failed to write CSV header for list '{}': {}",
                list_name, e
            ))
        })?;
    }

    // Write each row, skipping nested children
    for node in &matrix_list.rows {
        let record: Vec<String> = node.fields.iter().map(value_to_csv_string).collect();

        wtr.write_record(&record).map_err(|e| {
            CsvError::Other(format!(
                "Failed to write CSV record for id '{}' in list '{}': {}",
                node.id, list_name, e
            ))
        })?;

        // Note: Nested children in node.children are intentionally skipped.
        // If a caller needs to export nested data, they should export those lists separately.
    }

    wtr.flush().map_err(|e| {
        CsvError::Other(format!("Failed to flush CSV writer for list '{}': {}", list_name, e))
    })?;

    Ok(())
}

/// Convert a HEDL document to CSV string with custom configuration.
/// P1 OPTIMIZATION: Pre-allocate buffer capacity (1.1-1.2x speedup)
pub fn to_csv_with_config(doc: &Document, config: ToCsvConfig) -> Result<String> {
    // Estimate output size based on matrix list size
    // Approximate: rows * columns * 20 bytes/cell (conservative estimate)
    let estimated_size = estimate_csv_size(doc);
    let mut buffer = Vec::with_capacity(estimated_size);

    to_csv_writer_with_config(doc, &mut buffer, config)?;
    String::from_utf8(buffer).map_err(|_| {
        CsvError::InvalidUtf8 {
            context: "CSV output".to_string(),
        }
    })
}

/// Estimate CSV output size for pre-allocation
fn estimate_csv_size(doc: &Document) -> usize {
    let mut total = 0;

    // Scan for matrix lists and estimate size
    for item in doc.root.values() {
        if let Some(list) = item.as_list() {
            // Header row: column names + commas + newline
            let header_size = list.schema.iter().map(|s| s.len()).sum::<usize>()
                + list.schema.len()
                + 1;

            // Data rows: conservative estimate of 20 bytes per cell
            let row_count = list.rows.len();
            let col_count = list.schema.len();
            let data_size = row_count * col_count * 20;

            total += header_size + data_size;
        }
    }

    // Return at least 1KB, max estimated size
    total.max(1024)
}

/// Estimate CSV output size for a specific list
fn estimate_list_csv_size(doc: &Document, list_name: &str) -> usize {
    if let Some(item) = doc.root.get(list_name) {
        if let Some(list) = item.as_list() {
            // Header row: column names + commas + newline
            let header_size = list.schema.iter().map(|s| s.len()).sum::<usize>()
                + list.schema.len()
                + 1;

            // Data rows: conservative estimate of 20 bytes per cell
            let row_count = list.rows.len();
            let col_count = list.schema.len();
            let data_size = row_count * col_count * 20;

            return (header_size + data_size).max(1024);
        }
    }

    // Fallback to minimal size
    1024
}

/// Write a HEDL document to CSV format using a writer.
///
/// # Example
/// ```no_run
/// use hedl_core::Document;
/// use hedl_csv::to_csv_writer;
/// use std::fs::File;
///
/// let doc = Document::new((1, 0));
/// let file = File::create("output.csv").unwrap();
/// to_csv_writer(&doc, file).unwrap();
/// ```
pub fn to_csv_writer<W: Write>(doc: &Document, writer: W) -> Result<()> {
    to_csv_writer_with_config(doc, writer, ToCsvConfig::default())
}

/// Write a HEDL document to CSV format with custom configuration.
pub fn to_csv_writer_with_config<W: Write>(
    doc: &Document,
    writer: W,
    config: ToCsvConfig,
) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(config.delimiter)
        .quote_style(config.quote_style)
        .from_writer(writer);

    // Find the first matrix list in the document
    let matrix_list = find_first_matrix_list(doc)?;

    // Write header row if requested
    // Per SPEC.md: MatrixList.schema includes all column names with ID first
    if config.include_headers {
        wtr.write_record(&matrix_list.schema).map_err(|e| {
            CsvError::Other(format!("Failed to write CSV header: {}", e))
        })?;
    }

    // Write each row
    // Per SPEC.md: Node.fields contains ALL values including ID (first column)
    for node in &matrix_list.rows {
        let record: Vec<String> = node.fields.iter().map(value_to_csv_string).collect();

        wtr.write_record(&record).map_err(|e| {
            CsvError::Other(format!("Failed to write CSV record for id '{}': {}", node.id, e))
        })?;
    }

    wtr.flush().map_err(|e| {
        CsvError::Other(format!("Failed to flush CSV writer: {}", e))
    })?;

    Ok(())
}

/// Find the first matrix list in the document.
fn find_first_matrix_list(doc: &Document) -> Result<&MatrixList> {
    for item in doc.root.values() {
        if let Some(list) = item.as_list() {
            return Ok(list);
        }
    }

    Err(CsvError::NoLists)
}

/// Find a matrix list by name in the document.
fn find_matrix_list_by_name<'a>(doc: &'a Document, list_name: &str) -> Result<&'a MatrixList> {
    match doc.root.get(list_name) {
        Some(item) => match item.as_list() {
            Some(list) => Ok(list),
            None => Err(CsvError::NotAList {
                name: list_name.to_string(),
                actual_type: match item {
                        hedl_core::Item::Scalar(_) => "scalar",
                        hedl_core::Item::Object(_) => "object",
                        hedl_core::Item::List(_) => "list",
                    }.to_string(),
            }),
        },
        None => Err(CsvError::ListNotFound {
                name: list_name.to_string(),
                available: if doc.root.is_empty() {
                    "none".to_string()
                } else {
                    doc.root
                        .keys()
                        .map(|k| format!("'{}'", k))
                        .collect::<Vec<_>>()
                        .join(", ")
                },
            }),
    }
}

/// Convert a HEDL value to CSV string representation.
fn value_to_csv_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => {
            // Handle special float values
            if f.is_nan() {
                "NaN".to_string()
            } else if f.is_infinite() {
                if f.is_sign_positive() {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
            } else {
                f.to_string()
            }
        }
        Value::String(s) => s.clone(),
        Value::Reference(r) => r.to_ref_string(),
        Value::Tensor(t) => tensor_to_json_string(t),
        Value::Expression(e) => format!("$({})", e),
    }
}

/// Convert a tensor to JSON-like array string representation.
/// Examples: `[1,2,3]` or `[[1,2],[3,4]]`
fn tensor_to_json_string(tensor: &Tensor) -> String {
    match tensor {
        Tensor::Scalar(n) => {
            if n.fract() == 0.0 && n.abs() < i64::MAX as f64 {
                // Format as integer if it's a whole number
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Tensor::Array(items) => {
            let inner: Vec<String> = items.iter().map(tensor_to_json_string).collect();
            format!("[{}]", inner.join(","))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
    use hedl_core::lex::{Expression, Span};

    fn create_test_document() -> Document {
        let mut doc = Document::new((1, 0));

        // Per SPEC.md: MatrixList.schema includes all column names with ID first
        // Node.fields contains ALL values including ID (first column)
        let mut list = MatrixList::new(
            "Person",
            vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
                "active".to_string(),
            ],
        );

        list.add_row(Node::new(
            "Person",
            "1",
            vec![
                Value::String("1".to_string()),
                Value::String("Alice".to_string()),
                Value::Int(30),
                Value::Bool(true),
            ],
        ));

        list.add_row(Node::new(
            "Person",
            "2",
            vec![
                Value::String("2".to_string()),
                Value::String("Bob".to_string()),
                Value::Int(25),
                Value::Bool(false),
            ],
        ));

        doc.root.insert("people".to_string(), Item::List(list));
        doc
    }

    // ==================== ToCsvConfig tests ====================

    #[test]
    fn test_to_csv_config_default() {
        let config = ToCsvConfig::default();
        assert_eq!(config.delimiter, b',');
        assert!(config.include_headers);
        assert!(matches!(config.quote_style, csv::QuoteStyle::Necessary));
    }

    #[test]
    fn test_to_csv_config_debug() {
        let config = ToCsvConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("ToCsvConfig"));
        assert!(debug.contains("delimiter"));
        assert!(debug.contains("include_headers"));
        assert!(debug.contains("quote_style"));
    }

    #[test]
    fn test_to_csv_config_clone() {
        let config = ToCsvConfig {
            delimiter: b'\t',
            include_headers: false,
            quote_style: csv::QuoteStyle::Always,
        };
        let cloned = config.clone();
        assert_eq!(cloned.delimiter, b'\t');
        assert!(!cloned.include_headers);
    }

    #[test]
    fn test_to_csv_config_all_options() {
        let config = ToCsvConfig {
            delimiter: b';',
            include_headers: true,
            quote_style: csv::QuoteStyle::Always,
        };
        assert_eq!(config.delimiter, b';');
        assert!(config.include_headers);
    }

    // ==================== to_csv basic tests ====================

    #[test]
    fn test_to_csv_basic() {
        let doc = create_test_document();
        let csv = to_csv(&doc).unwrap();

        let expected = "id,name,age,active\n1,Alice,30,true\n2,Bob,25,false\n";
        assert_eq!(csv, expected);
    }

    #[test]
    fn test_to_csv_without_headers() {
        let doc = create_test_document();
        let config = ToCsvConfig {
            include_headers: false,
            ..Default::default()
        };
        let csv = to_csv_with_config(&doc, config).unwrap();

        let expected = "1,Alice,30,true\n2,Bob,25,false\n";
        assert_eq!(csv, expected);
    }

    #[test]
    fn test_to_csv_custom_delimiter() {
        let doc = create_test_document();
        let config = ToCsvConfig {
            delimiter: b'\t',
            ..Default::default()
        };
        let csv = to_csv_with_config(&doc, config).unwrap();

        let expected = "id\tname\tage\tactive\n1\tAlice\t30\ttrue\n2\tBob\t25\tfalse\n";
        assert_eq!(csv, expected);
    }

    #[test]
    fn test_to_csv_semicolon_delimiter() {
        let doc = create_test_document();
        let config = ToCsvConfig {
            delimiter: b';',
            ..Default::default()
        };
        let csv = to_csv_with_config(&doc, config).unwrap();

        assert!(csv.contains(";"));
        assert!(csv.contains("Alice"));
    }

    #[test]
    fn test_to_csv_empty_list() {
        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);
        doc.root.insert("people".to_string(), Item::List(list));

        let csv = to_csv(&doc).unwrap();
        assert_eq!(csv, "id,name\n");
    }

    #[test]
    fn test_to_csv_empty_list_no_headers() {
        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);
        doc.root.insert("people".to_string(), Item::List(list));

        let config = ToCsvConfig {
            include_headers: false,
            ..Default::default()
        };
        let csv = to_csv_with_config(&doc, config).unwrap();
        assert!(csv.is_empty());
    }

    // ==================== value_to_csv_string tests ====================

    #[test]
    fn test_value_to_csv_string_null() {
        assert_eq!(value_to_csv_string(&Value::Null), "");
    }

    #[test]
    fn test_value_to_csv_string_bool_true() {
        assert_eq!(value_to_csv_string(&Value::Bool(true)), "true");
    }

    #[test]
    fn test_value_to_csv_string_bool_false() {
        assert_eq!(value_to_csv_string(&Value::Bool(false)), "false");
    }

    #[test]
    fn test_value_to_csv_string_int_positive() {
        assert_eq!(value_to_csv_string(&Value::Int(42)), "42");
    }

    #[test]
    fn test_value_to_csv_string_int_negative() {
        assert_eq!(value_to_csv_string(&Value::Int(-100)), "-100");
    }

    #[test]
    fn test_value_to_csv_string_int_zero() {
        assert_eq!(value_to_csv_string(&Value::Int(0)), "0");
    }

    #[test]
    fn test_value_to_csv_string_int_large() {
        assert_eq!(
            value_to_csv_string(&Value::Int(i64::MAX)),
            i64::MAX.to_string()
        );
    }

    #[test]
    fn test_value_to_csv_string_float_positive() {
        assert_eq!(value_to_csv_string(&Value::Float(3.25)), "3.25");
    }

    #[test]
    fn test_value_to_csv_string_float_negative() {
        assert_eq!(value_to_csv_string(&Value::Float(-2.5)), "-2.5");
    }

    #[test]
    fn test_value_to_csv_string_float_zero() {
        assert_eq!(value_to_csv_string(&Value::Float(0.0)), "0");
    }

    #[test]
    fn test_value_to_csv_string_string() {
        assert_eq!(
            value_to_csv_string(&Value::String("hello".to_string())),
            "hello"
        );
    }

    #[test]
    fn test_value_to_csv_string_string_empty() {
        assert_eq!(value_to_csv_string(&Value::String("".to_string())), "");
    }

    #[test]
    fn test_value_to_csv_string_string_with_comma() {
        // The CSV library will quote this, but value_to_csv_string just returns the raw value
        assert_eq!(
            value_to_csv_string(&Value::String("hello, world".to_string())),
            "hello, world"
        );
    }

    #[test]
    fn test_value_to_csv_string_reference_local() {
        assert_eq!(
            value_to_csv_string(&Value::Reference(Reference::local("user1"))),
            "@user1"
        );
    }

    #[test]
    fn test_value_to_csv_string_reference_qualified() {
        assert_eq!(
            value_to_csv_string(&Value::Reference(Reference::qualified("User", "123"))),
            "@User:123"
        );
    }

    #[test]
    fn test_value_to_csv_string_expression_identifier() {
        let expr = Value::Expression(Expression::Identifier {
            name: "foo".to_string(),
            span: Span::default(),
        });
        assert_eq!(value_to_csv_string(&expr), "$(foo)");
    }

    #[test]
    fn test_value_to_csv_string_expression_call() {
        let expr = Value::Expression(Expression::Call {
            name: "add".to_string(),
            args: vec![
                Expression::Identifier {
                    name: "x".to_string(),
                    span: Span::default(),
                },
                Expression::Literal {
                    value: hedl_core::lex::ExprLiteral::Int(1),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        });
        assert_eq!(value_to_csv_string(&expr), "$(add(x, 1))");
    }

    // ==================== Special float values ====================

    #[test]
    fn test_special_float_nan() {
        assert_eq!(value_to_csv_string(&Value::Float(f64::NAN)), "NaN");
    }

    #[test]
    fn test_special_float_infinity() {
        assert_eq!(
            value_to_csv_string(&Value::Float(f64::INFINITY)),
            "Infinity"
        );
    }

    #[test]
    fn test_special_float_neg_infinity() {
        assert_eq!(
            value_to_csv_string(&Value::Float(f64::NEG_INFINITY)),
            "-Infinity"
        );
    }

    // ==================== Tensor tests ====================

    #[test]
    fn test_tensor_scalar_int() {
        let tensor = Tensor::Scalar(42.0);
        assert_eq!(tensor_to_json_string(&tensor), "42");
    }

    #[test]
    fn test_tensor_scalar_float() {
        let tensor = Tensor::Scalar(3.5);
        assert_eq!(tensor_to_json_string(&tensor), "3.5");
    }

    #[test]
    fn test_tensor_1d_array() {
        let tensor = Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
            Tensor::Scalar(3.0),
        ]);
        assert_eq!(tensor_to_json_string(&tensor), "[1,2,3]");
    }

    #[test]
    fn test_tensor_2d_array() {
        let tensor = Tensor::Array(vec![
            Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
            Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
        ]);
        assert_eq!(tensor_to_json_string(&tensor), "[[1,2],[3,4]]");
    }

    #[test]
    fn test_tensor_empty_array() {
        let tensor = Tensor::Array(vec![]);
        assert_eq!(tensor_to_json_string(&tensor), "[]");
    }

    #[test]
    fn test_value_to_csv_string_tensor() {
        let tensor = Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]);
        assert_eq!(value_to_csv_string(&Value::Tensor(tensor)), "[1,2]");
    }

    // ==================== Error cases ====================

    #[test]
    fn test_no_matrix_list_error() {
        let doc = Document::new((1, 0));
        let result = to_csv(&doc);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CsvError::NoLists | CsvError::NotAList { .. } | CsvError::ListNotFound { .. }));
    }

    #[test]
    fn test_no_matrix_list_with_scalar() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("value".to_string(), Item::Scalar(Value::Int(42)));

        let result = to_csv(&doc);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CsvError::NoLists | CsvError::NotAList { .. } | CsvError::ListNotFound { .. }));
    }

    // ==================== to_csv_writer tests ====================

    #[test]
    fn test_to_csv_writer_basic() {
        let doc = create_test_document();
        let mut buffer = Vec::new();
        to_csv_writer(&doc, &mut buffer).unwrap();

        let csv = String::from_utf8(buffer).unwrap();
        assert!(csv.contains("Alice"));
        assert!(csv.contains("Bob"));
    }

    #[test]
    fn test_to_csv_writer_with_config() {
        let doc = create_test_document();
        let config = ToCsvConfig {
            include_headers: false,
            ..Default::default()
        };
        let mut buffer = Vec::new();
        to_csv_writer_with_config(&doc, &mut buffer, config).unwrap();

        let csv = String::from_utf8(buffer).unwrap();
        assert!(!csv.contains("id,name"));
        assert!(csv.contains("Alice"));
    }

    // ==================== Quoting tests ====================

    #[test]
    fn test_quoting_with_comma() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);
        list.add_row(Node::new(
            "Item",
            "1",
            vec![
                Value::String("1".to_string()),
                Value::String("hello, world".to_string()),
            ],
        ));
        doc.root.insert("items".to_string(), Item::List(list));

        let csv = to_csv(&doc).unwrap();
        // The CSV library should quote fields with commas
        assert!(csv.contains("\"hello, world\""));
    }

    #[test]
    fn test_quoting_with_newline() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Item", vec!["id".to_string(), "text".to_string()]);
        list.add_row(Node::new(
            "Item",
            "1",
            vec![
                Value::String("1".to_string()),
                Value::String("line1\nline2".to_string()),
            ],
        ));
        doc.root.insert("items".to_string(), Item::List(list));

        let csv = to_csv(&doc).unwrap();
        // The CSV library should quote fields with newlines
        assert!(csv.contains("\"line1\nline2\""));
    }

    // ==================== to_csv_list tests ====================

    #[test]
    fn test_to_csv_list_basic() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new(
            "Person",
            vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
                "active".to_string(),
            ],
        );

        list.add_row(Node::new(
            "Person",
            "1",
            vec![
                Value::String("1".to_string()),
                Value::String("Alice".to_string()),
                Value::Int(30),
                Value::Bool(true),
            ],
        ));

        list.add_row(Node::new(
            "Person",
            "2",
            vec![
                Value::String("2".to_string()),
                Value::String("Bob".to_string()),
                Value::Int(25),
                Value::Bool(false),
            ],
        ));

        doc.root.insert("people".to_string(), Item::List(list));

        let csv = to_csv_list(&doc, "people").unwrap();
        let expected = "id,name,age,active\n1,Alice,30,true\n2,Bob,25,false\n";
        assert_eq!(csv, expected);
    }

    #[test]
    fn test_to_csv_list_selective_export() {
        let mut doc = Document::new((1, 0));

        // Add first list
        let mut people_list = MatrixList::new(
            "Person",
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );
        people_list.add_row(Node::new(
            "Person",
            "1",
            vec![
                Value::String("1".to_string()),
                Value::String("Alice".to_string()),
                Value::Int(30),
            ],
        ));
        doc.root
            .insert("people".to_string(), Item::List(people_list));

        // Add second list
        let mut items_list = MatrixList::new(
            "Item",
            vec!["id".to_string(), "name".to_string(), "price".to_string()],
        );
        items_list.add_row(Node::new(
            "Item",
            "101",
            vec![
                Value::String("101".to_string()),
                Value::String("Widget".to_string()),
                Value::Float(9.99),
            ],
        ));
        doc.root.insert("items".to_string(), Item::List(items_list));

        // Export only people
        let csv_people = to_csv_list(&doc, "people").unwrap();
        assert!(csv_people.contains("Alice"));
        assert!(!csv_people.contains("Widget"));

        // Export only items
        let csv_items = to_csv_list(&doc, "items").unwrap();
        assert!(csv_items.contains("Widget"));
        assert!(!csv_items.contains("Alice"));
    }

    #[test]
    fn test_to_csv_list_not_found() {
        let doc = Document::new((1, 0));
        let result = to_csv_list(&doc, "nonexistent");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CsvError::NoLists | CsvError::NotAList { .. } | CsvError::ListNotFound { .. }));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_to_csv_list_not_a_list() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("scalar".to_string(), Item::Scalar(Value::Int(42)));

        let result = to_csv_list(&doc, "scalar");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CsvError::NoLists | CsvError::NotAList { .. } | CsvError::ListNotFound { .. }));
        assert!(err.to_string().contains("not a matrix list"));
    }

    #[test]
    fn test_to_csv_list_without_headers() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);

        list.add_row(Node::new(
            "Person",
            "1",
            vec![Value::String("1".to_string()), Value::String("Alice".to_string())],
        ));

        doc.root.insert("people".to_string(), Item::List(list));

        let config = ToCsvConfig {
            include_headers: false,
            ..Default::default()
        };
        let csv = to_csv_list_with_config(&doc, "people", config).unwrap();

        let expected = "1,Alice\n";
        assert_eq!(csv, expected);
    }

    #[test]
    fn test_to_csv_list_custom_delimiter() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);

        list.add_row(Node::new(
            "Person",
            "1",
            vec![Value::String("1".to_string()), Value::String("Alice".to_string())],
        ));

        doc.root.insert("people".to_string(), Item::List(list));

        let config = ToCsvConfig {
            delimiter: b';',
            ..Default::default()
        };
        let csv = to_csv_list_with_config(&doc, "people", config).unwrap();

        let expected = "id;name\n1;Alice\n";
        assert_eq!(csv, expected);
    }

    #[test]
    fn test_to_csv_list_tab_delimiter() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);

        list.add_row(Node::new(
            "Person",
            "1",
            vec![Value::String("1".to_string()), Value::String("Alice".to_string())],
        ));

        doc.root.insert("people".to_string(), Item::List(list));

        let config = ToCsvConfig {
            delimiter: b'\t',
            ..Default::default()
        };
        let csv = to_csv_list_with_config(&doc, "people", config).unwrap();

        assert!(csv.contains("id\tname"));
        assert!(csv.contains("1\tAlice"));
    }

    #[test]
    fn test_to_csv_list_empty() {
        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);
        doc.root.insert("people".to_string(), Item::List(list));

        let csv = to_csv_list(&doc, "people").unwrap();
        let expected = "id,name\n";
        assert_eq!(csv, expected);
    }

    #[test]
    fn test_to_csv_list_empty_no_headers() {
        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);
        doc.root.insert("people".to_string(), Item::List(list));

        let config = ToCsvConfig {
            include_headers: false,
            ..Default::default()
        };
        let csv = to_csv_list_with_config(&doc, "people", config).unwrap();
        assert!(csv.is_empty());
    }

    #[test]
    fn test_to_csv_list_writer() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);

        list.add_row(Node::new(
            "Person",
            "1",
            vec![Value::String("1".to_string()), Value::String("Alice".to_string())],
        ));

        doc.root.insert("people".to_string(), Item::List(list));

        let mut buffer = Vec::new();
        to_csv_list_writer(&doc, "people", &mut buffer).unwrap();

        let csv = String::from_utf8(buffer).unwrap();
        assert!(csv.contains("Alice"));
    }

    #[test]
    fn test_to_csv_list_writer_with_config() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);

        list.add_row(Node::new(
            "Person",
            "1",
            vec![Value::String("1".to_string()), Value::String("Alice".to_string())],
        ));

        doc.root.insert("people".to_string(), Item::List(list));

        let config = ToCsvConfig {
            include_headers: false,
            ..Default::default()
        };
        let mut buffer = Vec::new();
        to_csv_list_writer_with_config(&doc, "people", &mut buffer, config).unwrap();

        let csv = String::from_utf8(buffer).unwrap();
        assert_eq!(csv, "1,Alice\n");
    }

    #[test]
    fn test_to_csv_list_with_all_value_types() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new(
            "Data",
            vec![
                "id".to_string(),
                "bool_val".to_string(),
                "int_val".to_string(),
                "float_val".to_string(),
                "string_val".to_string(),
                "null_val".to_string(),
                "ref_val".to_string(),
            ],
        );

        list.add_row(Node::new(
            "Data",
            "1",
            vec![
                Value::String("1".to_string()),
                Value::Bool(true),
                Value::Int(42),
                Value::Float(3.14),
                Value::String("hello".to_string()),
                Value::Null,
                Value::Reference(Reference::local("user1")),
            ],
        ));

        doc.root.insert("data".to_string(), Item::List(list));

        let csv = to_csv_list(&doc, "data").unwrap();
        assert!(csv.contains("true"));
        assert!(csv.contains("42"));
        assert!(csv.contains("3.14"));
        assert!(csv.contains("hello"));
        assert!(csv.contains("@user1"));
    }

    #[test]
    fn test_to_csv_list_with_nested_children_skipped() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Person", vec!["id".to_string(), "name".to_string()]);

        let mut person = Node::new(
            "Person",
            "1",
            vec![
                Value::String("1".to_string()),
                Value::String("Alice".to_string()),
            ],
        );

        // Add nested children (should be skipped in CSV export)
        let child = Node::new(
            "Address",
            "addr1",
            vec![Value::String("addr1".to_string()), Value::String("123 Main St".to_string())],
        );
        person.add_child("Address", child);

        list.add_row(person);
        doc.root.insert("people".to_string(), Item::List(list));

        // CSV should only contain the parent row, not the nested children
        let csv = to_csv_list(&doc, "people").unwrap();
        assert!(csv.contains("Alice"));
        assert!(!csv.contains("Address"));
        assert!(!csv.contains("123 Main St"));
    }

    #[test]
    fn test_to_csv_list_complex_quoting() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Item", vec!["id".to_string(), "description".to_string()]);

        list.add_row(Node::new(
            "Item",
            "1",
            vec![
                Value::String("1".to_string()),
                Value::String("Contains, comma and \"quotes\"".to_string()),
            ],
        ));

        doc.root.insert("items".to_string(), Item::List(list));

        let csv = to_csv_list(&doc, "items").unwrap();
        // CSV library should handle quoting
        assert!(csv.contains("comma"));
    }

    #[test]
    fn test_to_csv_list_multiple_lists_independent() {
        let mut doc = Document::new((1, 0));

        // First list with 2 rows
        let mut list1 = MatrixList::new("Type1", vec!["id".to_string(), "val".to_string()]);
        list1.add_row(Node::new(
            "Type1",
            "1",
            vec![Value::String("1".to_string()), Value::String("alpha".to_string())],
        ));
        list1.add_row(Node::new(
            "Type1",
            "2",
            vec![Value::String("2".to_string()), Value::String("bravo".to_string())],
        ));
        doc.root.insert("list1".to_string(), Item::List(list1));

        // Second list with 3 rows
        let mut list2 = MatrixList::new("Type2", vec!["id".to_string(), "val".to_string()]);
        list2.add_row(Node::new(
            "Type2",
            "1",
            vec![Value::String("1".to_string()), Value::String("x_ray".to_string())],
        ));
        list2.add_row(Node::new(
            "Type2",
            "2",
            vec![Value::String("2".to_string()), Value::String("yankee".to_string())],
        ));
        list2.add_row(Node::new(
            "Type2",
            "3",
            vec![Value::String("3".to_string()), Value::String("zulu".to_string())],
        ));
        doc.root.insert("list2".to_string(), Item::List(list2));

        // Export each list independently
        let csv1 = to_csv_list(&doc, "list1").unwrap();
        let csv2 = to_csv_list(&doc, "list2").unwrap();

        // List1 should have 2 data rows
        let lines1: Vec<&str> = csv1.lines().collect();
        assert_eq!(lines1.len(), 3); // header + 2 rows

        // List2 should have 3 data rows
        let lines2: Vec<&str> = csv2.lines().collect();
        assert_eq!(lines2.len(), 4); // header + 3 rows

        // Each should contain only its own data
        assert!(csv1.contains("alpha") && csv1.contains("bravo"));
        assert!(csv2.contains("x_ray") && csv2.contains("yankee") && csv2.contains("zulu"));
        assert!(!csv1.contains("x_ray"));
        assert!(!csv2.contains("alpha"));
    }

    #[test]
    fn test_to_csv_list_special_floats() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

        list.add_row(Node::new(
            "Data",
            "1",
            vec![Value::String("1".to_string()), Value::Float(f64::NAN)],
        ));

        list.add_row(Node::new(
            "Data",
            "2",
            vec![Value::String("2".to_string()), Value::Float(f64::INFINITY)],
        ));

        list.add_row(Node::new(
            "Data",
            "3",
            vec![
                Value::String("3".to_string()),
                Value::Float(f64::NEG_INFINITY),
            ],
        ));

        doc.root.insert("data".to_string(), Item::List(list));

        let csv = to_csv_list(&doc, "data").unwrap();
        assert!(csv.contains("NaN"));
        assert!(csv.contains("Infinity"));
        assert!(csv.contains("-Infinity"));
    }
}
