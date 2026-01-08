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

//! Convert CSV files to HEDL documents.

use crate::error::{CsvError, Result};
use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_core::lex::parse_expression_token;
use hedl_core::lex::parse_tensor;
use std::io::Read;

/// Default maximum number of rows to prevent memory exhaustion.
///
/// This limit prevents Denial-of-Service attacks from maliciously large CSV files.
/// The default is 1 million rows, which allows processing reasonably large datasets
/// while preventing unbounded memory allocation.
///
/// # Security Considerations
///
/// - **Memory exhaustion**: Without a limit, attackers could provide CSV files with
///   billions of rows, causing the application to allocate excessive memory and crash.
/// - **Configurable**: The limit can be adjusted via `FromCsvConfig::max_rows` based on
///   deployment context and available resources.
/// - **Trade-off**: Higher limits allow larger datasets but increase DoS risk.
///
/// # Examples
///
/// ```
/// # use hedl_csv::FromCsvConfig;
/// // Use default 1M row limit
/// let config = FromCsvConfig::default();
/// assert_eq!(config.max_rows, 1_000_000);
///
/// // Increase limit for large dataset processing
/// let config = FromCsvConfig {
///     max_rows: 10_000_000, // 10 million rows
///     ..Default::default()
/// };
/// ```
pub const DEFAULT_MAX_ROWS: usize = 1_000_000;

/// Configuration for CSV parsing.
///
/// This structure controls all aspects of CSV parsing behavior, including delimiters,
/// headers, whitespace handling, security limits, and custom list naming.
///
/// # Examples
///
/// ## Default Configuration
///
/// ```
/// # use hedl_csv::FromCsvConfig;
/// let config = FromCsvConfig::default();
/// assert_eq!(config.delimiter, b',');
/// assert!(config.has_headers);
/// assert!(config.trim);
/// assert_eq!(config.max_rows, 1_000_000);
/// assert_eq!(config.list_key, None);
/// ```
///
/// ## Tab-Delimited without Headers
///
/// ```
/// # use hedl_csv::FromCsvConfig;
/// let config = FromCsvConfig {
///     delimiter: b'\t',
///     has_headers: false,
///     ..Default::default()
/// };
/// ```
///
/// ## Custom Row Limit for Large Datasets
///
/// ```
/// # use hedl_csv::FromCsvConfig;
/// let config = FromCsvConfig {
///     max_rows: 10_000_000, // Allow up to 10M rows
///     ..Default::default()
/// };
/// ```
///
/// ## Disable Whitespace Trimming
///
/// ```
/// # use hedl_csv::FromCsvConfig;
/// let config = FromCsvConfig {
///     trim: false,
///     ..Default::default()
/// };
/// ```
///
/// ## Enable Schema Inference
///
/// ```
/// # use hedl_csv::FromCsvConfig;
/// let config = FromCsvConfig {
///     infer_schema: true,
///     sample_rows: 200, // Sample first 200 rows
///     ..Default::default()
/// };
/// ```
///
/// ## Custom List Key for Irregular Plurals
///
/// ```
/// # use hedl_csv::FromCsvConfig;
/// // For "Person" type, use "people" instead of default "persons"
/// let config = FromCsvConfig {
///     list_key: Some("people".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FromCsvConfig {
    /// Field delimiter character (default: `,`).
    ///
    /// Common alternatives:
    /// - `b'\t'` - Tab-separated values (TSV)
    /// - `b';'` - Semicolon-separated (common in European locales)
    /// - `b'|'` - Pipe-separated
    pub delimiter: u8,

    /// Whether the first row contains column headers (default: `true`).
    ///
    /// When `true`, the first row is interpreted as column names and not included
    /// in the data. When `false`, all rows are treated as data.
    pub has_headers: bool,

    /// Whether to trim leading/trailing whitespace from fields (default: `true`).
    ///
    /// When `true`, fields like `"  value  "` become `"value"`. This is generally
    /// recommended to handle inconsistently formatted CSV files.
    pub trim: bool,

    /// Maximum number of rows to parse (default: 1,000,000).
    ///
    /// This security limit prevents memory exhaustion from maliciously large CSV files.
    /// Processing stops with an error if more rows are encountered.
    ///
    /// # Security Impact
    ///
    /// - **DoS Protection**: Prevents attackers from causing memory exhaustion
    /// - **Memory Bound**: Limits worst-case memory usage to approximately
    ///   `max_rows × avg_row_size × columns`
    /// - **Recommended Values**:
    ///   - Small deployments: 100,000 - 1,000,000 rows
    ///   - Large deployments: 1,000,000 - 10,000,000 rows
    ///   - Batch processing: Adjust based on available RAM
    ///
    /// # Example
    ///
    /// ```
    /// # use hedl_csv::FromCsvConfig;
    /// // For processing very large datasets on a high-memory server
    /// let config = FromCsvConfig {
    ///     max_rows: 50_000_000,
    ///     ..Default::default()
    /// };
    /// ```
    pub max_rows: usize,

    /// Whether to automatically infer column types from data (default: `false`).
    ///
    /// When `true`, the parser samples the first `sample_rows` to determine the
    /// most specific type for each column. When `false`, uses standard per-value
    /// type inference.
    ///
    /// # Type Inference Hierarchy (most to least specific)
    ///
    /// 1. **Null**: All values are empty/null
    /// 2. **Bool**: All values are "true" or "false"
    /// 3. **Int**: All values parse as integers
    /// 4. **Float**: All values parse as floats
    /// 5. **String**: Fallback for all other cases
    ///
    /// # Example
    ///
    /// ```
    /// # use hedl_csv::FromCsvConfig;
    /// let config = FromCsvConfig {
    ///     infer_schema: true,
    ///     sample_rows: 100,
    ///     ..Default::default()
    /// };
    /// ```
    pub infer_schema: bool,

    /// Number of rows to sample for schema inference (default: 100).
    ///
    /// Only used when `infer_schema` is `true`. Larger sample sizes provide
    /// more accurate type detection but slower initial processing.
    ///
    /// # Trade-offs
    ///
    /// - **Small (10-50)**: Fast inference, may miss edge cases
    /// - **Medium (100-500)**: Balanced accuracy and performance
    /// - **Large (1000+)**: High accuracy, slower for large datasets
    pub sample_rows: usize,

    /// Custom key name for the matrix list in the document (default: `None`).
    ///
    /// When `None`, the list key is automatically generated by adding 's' to the
    /// lowercased type name (e.g., "Person" → "persons"). When `Some`, uses the
    /// specified custom key instead.
    ///
    /// # Use Cases
    ///
    /// - **Irregular Plurals**: "Person" → "people" instead of "persons"
    /// - **Collective Nouns**: "Data" → "dataset" instead of "datas"
    /// - **Custom Naming**: Any non-standard naming convention
    /// - **Case-Sensitive Keys**: Preserve specific casing requirements
    ///
    /// # Examples
    ///
    /// ## Irregular Plural
    ///
    /// ```
    /// # use hedl_csv::{from_csv_with_config, FromCsvConfig};
    /// let csv = "id,name\n1,Alice\n";
    /// let config = FromCsvConfig {
    ///     list_key: Some("people".to_string()),
    ///     ..Default::default()
    /// };
    /// let doc = from_csv_with_config(csv, "Person", &["name"], config).unwrap();
    /// assert!(doc.get("people").is_some()); // Uses custom plural
    /// assert!(doc.get("persons").is_none()); // Default plural not used
    /// ```
    ///
    /// ## Collective Noun
    ///
    /// ```
    /// # use hedl_csv::{from_csv_with_config, FromCsvConfig};
    /// let csv = "id,value\n1,42\n";
    /// let config = FromCsvConfig {
    ///     list_key: Some("dataset".to_string()),
    ///     ..Default::default()
    /// };
    /// let doc = from_csv_with_config(csv, "Data", &["value"], config).unwrap();
    /// assert!(doc.get("dataset").is_some());
    /// ```
    ///
    /// ## Case-Sensitive Key
    ///
    /// ```
    /// # use hedl_csv::{from_csv_with_config, FromCsvConfig};
    /// let csv = "id,value\n1,test\n";
    /// let config = FromCsvConfig {
    ///     list_key: Some("MyCustomList".to_string()),
    ///     ..Default::default()
    /// };
    /// let doc = from_csv_with_config(csv, "Item", &["value"], config).unwrap();
    /// assert!(doc.get("MyCustomList").is_some());
    /// ```
    pub list_key: Option<String>,
}

impl Default for FromCsvConfig {
    fn default() -> Self {
        Self {
            delimiter: b',',
            has_headers: true,
            trim: true,
            max_rows: DEFAULT_MAX_ROWS,
            infer_schema: false,
            sample_rows: 100,
            list_key: None,
        }
    }
}

/// Parse CSV string into a HEDL document with default configuration.
///
/// This is the primary entry point for CSV parsing. It uses sensible defaults:
/// - Comma delimiter
/// - Headers expected in first row
/// - Whitespace trimming enabled
/// - 1 million row limit for security
///
/// # Arguments
///
/// * `csv` - The CSV string to parse
/// * `type_name` - The HEDL type name for rows (e.g., "Person")
/// * `schema` - Column names excluding the 'id' column (which is always first)
///
/// # Returns
///
/// A `Document` containing a single matrix list with the parsed data, or an error
/// if parsing fails.
///
/// # Errors
///
/// Returns `HedlError` in the following cases:
///
/// - `Syntax`: Malformed CSV records or invalid UTF-8
/// - `Schema`: Missing ID column or field count mismatch
/// - `Semantic`: Empty ID field
/// - `Security`: Row count exceeds maximum (default 1M rows)
///
/// # Type Inference
///
/// Values are automatically inferred from CSV text:
///
/// - Empty string or `~` → `Value::Null`
/// - `true`/`false` → `Value::Bool`
/// - Integer pattern → `Value::Int` (e.g., "42", "-123")
/// - Float pattern → `Value::Float` (e.g., "3.14", "1.5e10")
/// - Special floats: `NaN`, `Infinity`, `-Infinity`
/// - `@id` or `@Type:id` → `Value::Reference`
/// - `$(expr)` → `Value::Expression`
/// - `[1,2,3]` → `Value::Tensor`
/// - Otherwise → `Value::String`
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use hedl_csv::from_csv;
/// use hedl_core::Value;
///
/// let csv_data = "id,name,age\n1,Alice,30\n2,Bob,25";
/// let doc = from_csv(csv_data, "Person", &["name", "age"]).unwrap();
///
/// // Access the parsed data
/// let list = doc.get("persons").unwrap().as_list().unwrap();
/// assert_eq!(list.rows.len(), 2);
/// assert_eq!(list.rows[0].id, "1");
/// ```
///
/// ## Mixed Type Inference
///
/// ```
/// use hedl_csv::from_csv;
/// use hedl_core::Value;
///
/// let csv_data = "id,value\n1,42\n2,3.14\n3,true\n4,hello";
/// let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
///
/// let list = doc.get("items").unwrap().as_list().unwrap();
/// assert!(matches!(list.rows[0].fields[1], Value::Int(42)));
/// assert!(matches!(list.rows[1].fields[1], Value::Float(f) if (f - 3.14).abs() < 0.001));
/// assert!(matches!(list.rows[2].fields[1], Value::Bool(true)));
/// assert!(matches!(list.rows[3].fields[1], Value::String(_)));
/// ```
///
/// ## References
///
/// ```
/// use hedl_csv::from_csv;
///
/// let csv_data = "id,owner\n1,@user1\n2,@User:alice";
/// let doc = from_csv(csv_data, "Item", &["owner"]).unwrap();
///
/// let list = doc.get("items").unwrap().as_list().unwrap();
/// let ref1 = list.rows[0].fields[1].as_reference().unwrap();
/// assert_eq!(ref1.id, "user1");
/// assert_eq!(ref1.type_name, None); // Local reference
///
/// let ref2 = list.rows[1].fields[1].as_reference().unwrap();
/// assert_eq!(ref2.id, "alice");
/// assert_eq!(ref2.type_name, Some("User".to_string())); // Qualified reference
/// ```
///
/// # Performance
///
/// - **Streaming**: Processes CSV row-by-row to minimize memory usage
/// - **Memory bound**: O(rows × columns) space complexity
/// - **Time complexity**: O(rows × columns) with efficient parsing
///
/// For very large files, consider using `from_csv_reader` for file I/O or
/// increasing `max_rows` via `from_csv_with_config`.
///
/// # See Also
///
/// - `from_csv_with_config` - For custom delimiters, row limits, etc.
/// - `from_csv_reader` - For parsing from files or network streams
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
/// assert_eq!(list.rows[0].fields[1], Value::String("  Alice  ".to_string()));
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
pub fn from_csv_reader<R: Read>(
    reader: R,
    type_name: &str,
    schema: &[&str],
) -> Result<Document> {
    from_csv_reader_with_config(reader, type_name, schema, FromCsvConfig::default())
}

/// Inferred column type from sampling CSV data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColumnType {
    /// All sampled values are null/empty
    Null,
    /// All sampled values are "true" or "false"
    Bool,
    /// All sampled values parse as integers
    Int,
    /// All sampled values parse as floats (but not all as integers)
    Float,
    /// Default fallback for mixed or string data
    String,
}

/// Infer the type of a single column from sampled values.
///
/// # Type Inference Rules
///
/// The function examines non-null values and determines the most specific type:
///
/// 1. If all values are null → `ColumnType::Null`
/// 2. If all values are "true"/"false" → `ColumnType::Bool`
/// 3. If all values parse as i64 → `ColumnType::Int`
/// 4. If all values parse as f64 → `ColumnType::Float`
/// 5. Otherwise → `ColumnType::String`
///
/// # Arguments
///
/// * `values` - Iterator over string values from a column
///
/// # Examples
///
/// ```text
/// let values = vec!["1", "2", "3"];
/// let col_type = infer_column_type(values.iter().map(|s| s.as_str()));
/// assert_eq!(col_type, ColumnType::Int);
/// ```
fn infer_column_type<'a, I>(values: I) -> ColumnType
where
    I: Iterator<Item = &'a str>,
{
    let mut all_null = true;
    let mut all_bool = true;
    let mut all_int = true;
    let mut all_float = true;

    for value in values {
        let trimmed = value.trim();

        // Skip null values (don't affect type inference)
        if trimmed.is_empty() || trimmed == "~" || trimmed == "null" {
            continue;
        }

        all_null = false;

        // Check bool
        if trimmed != "true" && trimmed != "false" {
            all_bool = false;
        }

        // Check int
        if trimmed.parse::<i64>().is_err() {
            all_int = false;
        }

        // Check float
        if trimmed.parse::<f64>().is_err() {
            all_float = false;
        }

        // Early exit if we know it's a string
        if !all_bool && !all_int && !all_float {
            return ColumnType::String;
        }
    }

    // Determine type based on inference (most specific to least)
    if all_null {
        ColumnType::Null
    } else if all_bool {
        ColumnType::Bool
    } else if all_int {
        ColumnType::Int
    } else if all_float {
        ColumnType::Float
    } else {
        ColumnType::String
    }
}

/// Infer types for all columns by sampling CSV records.
///
/// # Arguments
///
/// * `records` - Slice of CSV records (each record is a Vec<String>)
/// * `sample_size` - Maximum number of records to sample
///
/// # Returns
///
/// A vector of `ColumnType` for each column in the CSV.
///
/// # Examples
///
/// ```text
/// let records = vec![
///     vec!["1".to_string(), "Alice".to_string(), "30".to_string()],
///     vec!["2".to_string(), "Bob".to_string(), "25".to_string()],
/// ];
/// let types = infer_column_types(&records, 100);
/// assert_eq!(types, vec![ColumnType::Int, ColumnType::String, ColumnType::Int]);
/// ```
fn infer_column_types(records: &[Vec<String>], sample_size: usize) -> Vec<ColumnType> {
    if records.is_empty() {
        return Vec::new();
    }

    let num_columns = records[0].len();
    let sample_count = sample_size.min(records.len());

    (0..num_columns)
        .map(|col_idx| {
            let column_values = records
                .iter()
                .take(sample_count)
                .filter_map(|row| row.get(col_idx).map(|s| s.as_str()));

            infer_column_type(column_values)
        })
        .collect()
}

/// Parse a CSV value using a specific inferred type.
///
/// This function forces type conversion based on the inferred schema,
/// falling back to string on conversion failure.
///
/// # Arguments
///
/// * `field` - The string value to parse
/// * `col_type` - The inferred column type
///
/// # Returns
///
/// A HEDL `Value` of the specified type, or `Value::String` if conversion fails.
fn parse_csv_value_with_type(field: &str, col_type: ColumnType) -> Result<Value> {
    let trimmed = field.trim();

    // Always handle null values regardless of inferred type
    if trimmed.is_empty() || trimmed == "~" {
        return Ok(Value::Null);
    }

    match col_type {
        ColumnType::Null => Ok(Value::Null),
        ColumnType::Bool => {
            if trimmed == "true" {
                Ok(Value::Bool(true))
            } else if trimmed == "false" {
                Ok(Value::Bool(false))
            } else {
                // Fallback to string if not a valid bool
                Ok(Value::String(field.to_string()))
            }
        }
        ColumnType::Int => {
            if let Ok(n) = trimmed.parse::<i64>() {
                Ok(Value::Int(n))
            } else {
                // Fallback to string if not a valid int
                Ok(Value::String(field.to_string()))
            }
        }
        ColumnType::Float => {
            if let Ok(f) = trimmed.parse::<f64>() {
                Ok(Value::Float(f))
            } else {
                // Fallback to string if not a valid float
                Ok(Value::String(field.to_string()))
            }
        }
        ColumnType::String => {
            // Use the original parse_csv_value for full type detection
            // (handles references, expressions, tensors, etc.)
            parse_csv_value(field)
        }
    }
}

/// Parse CSV from a reader into a HEDL document with custom configuration.
///
/// This is the most flexible CSV parsing function, supporting both custom I/O sources
/// and custom parsing configuration.
///
/// # Arguments
///
/// * `reader` - Any type implementing `Read`
/// * `type_name` - The HEDL type name for rows
/// * `schema` - Column names excluding the 'id' column
/// * `config` - Configuration controlling all parsing behavior
///
/// # Examples
///
/// ## Large File with Custom Limit
///
/// ```no_run
/// use hedl_csv::{from_csv_reader_with_config, FromCsvConfig};
/// use std::fs::File;
///
/// let file = File::open("large_dataset.csv").unwrap();
/// let config = FromCsvConfig {
///     max_rows: 50_000_000, // 50M rows for high-memory server
///     ..Default::default()
/// };
/// let doc = from_csv_reader_with_config(file, "Record", &["value"], config).unwrap();
/// ```
///
/// ## TSV from Network Stream
///
/// ```no_run
/// use hedl_csv::{from_csv_reader_with_config, FromCsvConfig};
/// use std::net::TcpStream;
///
/// let stream = TcpStream::connect("example.com:8080").unwrap();
/// let config = FromCsvConfig {
///     delimiter: b'\t',
///     ..Default::default()
/// };
/// let doc = from_csv_reader_with_config(stream, "Data", &["col1", "col2"], config).unwrap();
/// ```
///
/// # Implementation Details
///
/// The function performs the following steps:
///
/// 1. Creates a CSV reader with the specified configuration
/// 2. Initializes a new HEDL document with version (1, 0)
/// 3. Constructs the full schema (ID column + provided columns)
/// 4. Registers the struct type in the document
/// 5. Iterates through CSV records:
///    - Checks row count against `max_rows` security limit
///    - Parses each field using type inference
///    - Validates field count matches schema
///    - Creates `Node` instances and adds to matrix list
/// 6. Inserts the completed matrix list into the document
///
/// # See Also
///
/// - `from_csv_with_config` - For parsing CSV strings
/// - `FromCsvConfig` - Configuration options documentation
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

    let mut doc = Document::new((1, 0));

    // Create schema with 'id' column
    let mut full_schema = vec!["id".to_string()];
    full_schema.extend(schema.iter().map(|s| s.to_string()));

    // Register the struct type
    doc.structs
        .insert(type_name.to_string(), full_schema.clone());

    // Create matrix list
    let mut matrix_list = MatrixList::new(type_name, full_schema.clone());

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

            let record = result.map_err(|e| {
                CsvError::ParseError {
                line: record_idx + 1,
                message: e.to_string(),
            }
            })?;

            if record.is_empty() {
                continue;
            }

            // Convert StringRecord to Vec<String>
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            all_records.push(row);
        }

        // Infer column types from sampled records
        let types = infer_column_types(&all_records, config.sample_rows);

        // Process all records with inferred types
        for (record_idx, row) in all_records.iter().enumerate() {
            // First column is the ID
            let id = row.first().ok_or_else(|| {
                CsvError::MissingColumn("id".to_string())
            })?;

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

            let record = result.map_err(|e| {
                CsvError::ParseError {
                line: record_idx + 1,
                message: e.to_string(),
            }
            })?;

            if record.is_empty() {
                continue;
            }

            // First column is the ID
            let id = record.get(0).ok_or_else(|| {
                CsvError::MissingColumn("id".to_string())
            })?;

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

/// Parse a CSV field value into a HEDL Value.
///
/// Type inference rules:
/// - Empty string → Null
/// - "true" or "false" → Bool
/// - Integer pattern → Int
/// - Float pattern → Float
/// - Reference pattern (@...) → Reference
/// - Expression pattern $(...) → Expression
/// - Otherwise → String
fn parse_csv_value(field: &str) -> Result<Value> {
    let trimmed = field.trim();

    // Empty or null
    if trimmed.is_empty() || trimmed == "~" {
        return Ok(Value::Null);
    }

    // Boolean
    if trimmed == "true" {
        return Ok(Value::Bool(true));
    }
    if trimmed == "false" {
        return Ok(Value::Bool(false));
    }

    // Special float values
    match trimmed {
        "NaN" => return Ok(Value::Float(f64::NAN)),
        "Infinity" => return Ok(Value::Float(f64::INFINITY)),
        "-Infinity" => return Ok(Value::Float(f64::NEG_INFINITY)),
        _ => {}
    }

    // Reference
    if trimmed.starts_with('@') {
        return parse_reference(trimmed);
    }

    // Expression
    if trimmed.starts_with("$(") && trimmed.ends_with(')') {
        let expr = parse_expression_token(trimmed).map_err(|e| {
            CsvError::ParseError {
                line: 0,
                message: format!("Invalid expression: {}", e),
            }
        })?;
        return Ok(Value::Expression(expr));
    }

    // Try integer
    if let Ok(n) = trimmed.parse::<i64>() {
        return Ok(Value::Int(n));
    }

    // Try float
    if let Ok(f) = trimmed.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    // Tensor literal (starts with '[' and ends with ']')
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        if let Ok(tensor) = parse_tensor(trimmed) {
            return Ok(Value::Tensor(tensor));
        }
        // If parsing fails, fall through to string
    }

    // Default to string
    Ok(Value::String(field.to_string()))
}

/// Parse a reference string (e.g., "@user1" or "@User:user1").
fn parse_reference(s: &str) -> Result<Value> {
    let without_at = &s[1..];

    if let Some(colon_pos) = without_at.find(':') {
        // Qualified reference: @Type:id
        let type_name = &without_at[..colon_pos];
        let id = &without_at[colon_pos + 1..];

        if type_name.is_empty() || id.is_empty() {
            return Err(CsvError::ParseError {
            line: 0,
            message: format!("Invalid reference format: {}", s),
        });
        }

        Ok(Value::Reference(hedl_core::Reference::qualified(
            type_name, id,
        )))
    } else {
        // Local reference: @id
        if without_at.is_empty() {
            return Err(CsvError::ParseError {
            line: 0,
            message: "Empty reference ID".to_string(),
        });
        }

        Ok(Value::Reference(hedl_core::Reference::local(without_at)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::lex::Tensor;
    use hedl_test::expr_value;

    // ==================== FromCsvConfig tests ====================

    #[test]
    fn test_from_csv_config_default() {
        let config = FromCsvConfig::default();
        assert_eq!(config.delimiter, b',');
        assert!(config.has_headers);
        assert!(config.trim);
        assert_eq!(config.max_rows, DEFAULT_MAX_ROWS);
    }

    #[test]
    fn test_from_csv_config_debug() {
        let config = FromCsvConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("FromCsvConfig"));
        assert!(debug.contains("delimiter"));
        assert!(debug.contains("has_headers"));
        assert!(debug.contains("trim"));
    }

    #[test]
    fn test_from_csv_config_clone() {
        let config = FromCsvConfig {
            delimiter: b'\t',
            has_headers: false,
            trim: false,
            max_rows: 500_000,
            infer_schema: false,
            sample_rows: 100,
            list_key: None,
        };
        let cloned = config.clone();
        assert_eq!(cloned.delimiter, b'\t');
        assert!(!cloned.has_headers);
        assert!(!cloned.trim);
        assert_eq!(cloned.max_rows, 500_000);
        assert!(!cloned.infer_schema);
        assert_eq!(cloned.sample_rows, 100);
        assert_eq!(cloned.list_key, None);
    }

    #[test]
    fn test_from_csv_config_all_options() {
        let config = FromCsvConfig {
            delimiter: b';',
            has_headers: true,
            trim: true,
            max_rows: 2_000_000,
            infer_schema: true,
            sample_rows: 200,
            list_key: Some("custom".to_string()),
        };
        assert_eq!(config.delimiter, b';');
        assert!(config.has_headers);
        assert!(config.trim);
        assert_eq!(config.max_rows, 2_000_000);
        assert!(config.infer_schema);
        assert_eq!(config.sample_rows, 200);
        assert_eq!(config.list_key, Some("custom".to_string()));
    }

    #[test]
    fn test_max_rows_limit_enforcement() {
        // Create CSV with exactly max_rows + 1 rows
        let mut csv_data = String::from("id,value\n");
        let max_rows = 100;
        for i in 0..=max_rows {
            csv_data.push_str(&format!("{},test{}\n", i, i));
        }

        let config = FromCsvConfig {
            max_rows,
            infer_schema: false,
            sample_rows: 100,
            ..Default::default()
        };

        let result = from_csv_with_config(&csv_data, "Item", &["value"], config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CsvError::SecurityLimit { .. }));
        assert!(err.to_string().contains("Security limit"));
        assert!(err.to_string().contains(&max_rows.to_string()));
    }

    #[test]
    fn test_max_rows_limit_not_exceeded() {
        // Create CSV with exactly max_rows rows
        let mut csv_data = String::from("id,value\n");
        let max_rows = 100;
        for i in 0..(max_rows - 1) {
            csv_data.push_str(&format!("{},test{}\n", i, i));
        }

        let config = FromCsvConfig {
            max_rows,
            infer_schema: false,
            sample_rows: 100,
            ..Default::default()
        };

        let result = from_csv_with_config(&csv_data, "Item", &["value"], config);
        assert!(result.is_ok());
        let doc = result.unwrap();
        let list = doc.get("items").unwrap().as_list().unwrap();
        assert_eq!(list.rows.len(), max_rows - 1);
    }

    // ==================== from_csv basic tests ====================

    #[test]
    fn test_from_csv_basic() {
        let csv_data = "id,name,age,active\n1,Alice,30,true\n2,Bob,25,false\n";
        let doc = from_csv(csv_data, "Person", &["name", "age", "active"]).unwrap();

        // Check document structure
        assert_eq!(doc.version, (1, 0));

        // Check schema registration
        let schema = doc.get_schema("Person").unwrap();
        assert_eq!(schema, &["id", "name", "age", "active"]);

        // Check matrix list
        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.type_name, "Person");
        assert_eq!(list.rows.len(), 2);

        // Check first row
        let row1 = &list.rows[0];
        assert_eq!(row1.id, "1");
        assert_eq!(row1.fields.len(), schema.len()); // schema includes ID
        assert_eq!(row1.fields[0], Value::Int(1)); // ID field
        assert_eq!(row1.fields[1], Value::String("Alice".to_string()));
        assert_eq!(row1.fields[2], Value::Int(30));
        assert_eq!(row1.fields[3], Value::Bool(true));

        // Check second row
        let row2 = &list.rows[1];
        assert_eq!(row2.id, "2");
        assert_eq!(row2.fields.len(), schema.len()); // schema includes ID
        assert_eq!(row2.fields[0], Value::Int(2)); // ID field
        assert_eq!(row2.fields[1], Value::String("Bob".to_string()));
        assert_eq!(row2.fields[2], Value::Int(25));
        assert_eq!(row2.fields[3], Value::Bool(false));
    }

    #[test]
    fn test_from_csv_without_headers() {
        let csv_data = "1,Alice,30\n2,Bob,25\n";
        let config = FromCsvConfig {
            has_headers: false,
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name", "age"], config).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 2);
    }

    #[test]
    fn test_from_csv_custom_delimiter() {
        let csv_data = "id\tname\tage\n1\tAlice\t30\n2\tBob\t25\n";
        let config = FromCsvConfig {
            delimiter: b'\t',
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name", "age"], config).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 2);
    }

    #[test]
    fn test_from_csv_semicolon_delimiter() {
        let csv_data = "id;name;age\n1;Alice;30\n";
        let config = FromCsvConfig {
            delimiter: b';',
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name", "age"], config).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.rows[0].fields[1], Value::String("Alice".to_string()));
    }

    #[test]
    fn test_from_csv_empty_file() {
        let csv_data = "id,name\n";
        let doc = from_csv(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert!(list.rows.is_empty());
    }

    #[test]
    fn test_from_csv_single_row() {
        let csv_data = "id,name\n1,Alice\n";
        let doc = from_csv(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 1);
    }

    // ==================== parse_csv_value tests ====================

    #[test]
    fn test_parse_csv_value_null_empty() {
        assert_eq!(parse_csv_value("").unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_csv_value_null_tilde() {
        assert_eq!(parse_csv_value("~").unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_csv_value_null_whitespace() {
        assert_eq!(parse_csv_value("   ").unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_csv_value_bool_true() {
        assert_eq!(parse_csv_value("true").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_parse_csv_value_bool_false() {
        assert_eq!(parse_csv_value("false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_parse_csv_value_int_positive() {
        assert_eq!(parse_csv_value("42").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_parse_csv_value_int_negative() {
        assert_eq!(parse_csv_value("-123").unwrap(), Value::Int(-123));
    }

    #[test]
    fn test_parse_csv_value_int_zero() {
        assert_eq!(parse_csv_value("0").unwrap(), Value::Int(0));
    }

    #[test]
    fn test_parse_csv_value_int_large() {
        assert_eq!(
            parse_csv_value("9223372036854775807").unwrap(),
            Value::Int(i64::MAX)
        );
    }

    #[test]
    fn test_parse_csv_value_float_positive() {
        assert_eq!(parse_csv_value("3.25").unwrap(), Value::Float(3.25));
    }

    #[test]
    fn test_parse_csv_value_float_negative() {
        assert_eq!(parse_csv_value("-2.5").unwrap(), Value::Float(-2.5));
    }

    #[test]
    fn test_parse_csv_value_float_zero() {
        assert_eq!(parse_csv_value("0.0").unwrap(), Value::Float(0.0));
    }

    #[test]
    fn test_parse_csv_value_float_scientific() {
        let val = parse_csv_value("1.5e10").unwrap();
        if let Value::Float(f) = val {
            assert!((f - 1.5e10).abs() < 1e5);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_parse_csv_value_string() {
        assert_eq!(
            parse_csv_value("hello").unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_parse_csv_value_string_with_spaces() {
        assert_eq!(
            parse_csv_value("  hello world  ").unwrap(),
            Value::String("  hello world  ".to_string())
        );
    }

    #[test]
    fn test_parse_csv_value_string_numeric_looking() {
        // Strings that look like numbers but have leading zeros
        assert_eq!(
            parse_csv_value("007").unwrap(),
            Value::Int(7) // Parsed as int
        );
    }

    // ==================== Special float values ====================

    #[test]
    fn test_parse_csv_value_nan() {
        let nan = parse_csv_value("NaN").unwrap();
        assert!(matches!(nan, Value::Float(f) if f.is_nan()));
    }

    #[test]
    fn test_parse_csv_value_infinity() {
        let inf = parse_csv_value("Infinity").unwrap();
        assert_eq!(inf, Value::Float(f64::INFINITY));
    }

    #[test]
    fn test_parse_csv_value_neg_infinity() {
        let neg_inf = parse_csv_value("-Infinity").unwrap();
        assert_eq!(neg_inf, Value::Float(f64::NEG_INFINITY));
    }

    // ==================== Reference tests ====================

    #[test]
    fn test_parse_csv_value_reference_local() {
        let ref_val = parse_csv_value("@user1").unwrap();
        if let Value::Reference(r) = ref_val {
            assert_eq!(r.id, "user1");
            assert_eq!(r.type_name, None);
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_parse_csv_value_reference_qualified() {
        let ref_val = parse_csv_value("@User:user1").unwrap();
        if let Value::Reference(r) = ref_val {
            assert_eq!(r.id, "user1");
            assert_eq!(r.type_name, Some("User".to_string()));
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_parse_csv_value_reference_with_dashes() {
        let ref_val = parse_csv_value("@my-item-123").unwrap();
        if let Value::Reference(r) = ref_val {
            assert_eq!(r.id, "my-item-123");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_parse_reference_empty_error() {
        let result = parse_reference("@");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Empty reference ID"));
    }

    #[test]
    fn test_parse_reference_empty_type_error() {
        let result = parse_reference("@:id");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid reference format"));
    }

    #[test]
    fn test_parse_reference_empty_id_error() {
        let result = parse_reference("@Type:");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid reference format"));
    }

    // ==================== Expression tests ====================

    #[test]
    fn test_parse_csv_value_expression_identifier() {
        let expr = parse_csv_value("$(foo)").unwrap();
        assert_eq!(expr, expr_value("foo"));
    }

    #[test]
    fn test_parse_csv_value_expression_call() {
        let expr = parse_csv_value("$(add(x, y))").unwrap();
        assert_eq!(expr, expr_value("add(x, y)"));
    }

    #[test]
    fn test_parse_csv_value_expression_nested() {
        let expr = parse_csv_value("$(outer(inner(x)))").unwrap();
        if let Value::Expression(e) = expr {
            assert_eq!(e.to_string(), "outer(inner(x))");
        } else {
            panic!("Expected expression");
        }
    }

    // ==================== Tensor tests ====================

    #[test]
    fn test_parse_csv_value_tensor_1d() {
        let val = parse_csv_value("[1, 2, 3]").unwrap();
        if let Value::Tensor(Tensor::Array(arr)) = val {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected tensor array");
        }
    }

    #[test]
    fn test_parse_csv_value_tensor_2d() {
        let val = parse_csv_value("[[1, 2], [3, 4]]").unwrap();
        if let Value::Tensor(Tensor::Array(outer)) = val {
            assert_eq!(outer.len(), 2);
            if let Tensor::Array(inner) = &outer[0] {
                assert_eq!(inner.len(), 2);
            } else {
                panic!("Expected nested array");
            }
        } else {
            panic!("Expected tensor array");
        }
    }

    #[test]
    fn test_parse_csv_value_tensor_empty_is_string() {
        // Empty tensors are not valid in HEDL (must have at least one element)
        // So "[]" falls through to being treated as a string
        let val = parse_csv_value("[]").unwrap();
        assert_eq!(val, Value::String("[]".to_string()));
    }

    // ==================== Error cases ====================

    #[test]
    fn test_empty_id_error() {
        let csv_data = "id,name\n,Alice\n";
        let result = from_csv(csv_data, "Person", &["name"]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CsvError::EmptyId { .. }));
    }

    #[test]
    fn test_mismatched_field_count() {
        let csv_data = "id,name,age\n1,Alice\n";
        let result = from_csv(csv_data, "Person", &["name", "age"]);
        assert!(result.is_err());
        // CSV parser returns Syntax error for malformed records
        assert!(matches!(result.unwrap_err(), CsvError::ParseError { .. }));
    }

    // ==================== Whitespace handling ====================

    #[test]
    fn test_whitespace_trimming_enabled() {
        let csv_data = "id,name,age\n1,  Alice  ,  30  \n";
        let doc = from_csv(csv_data, "Person", &["name", "age"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        let row = &list.rows[0];

        assert_eq!(row.fields[0], Value::Int(1)); // ID field
        assert_eq!(row.fields[1], Value::String("Alice".to_string()));
        assert_eq!(row.fields[2], Value::Int(30));
    }

    #[test]
    fn test_whitespace_trimming_disabled() {
        let csv_data = "id,name\n1,  Alice  \n";
        let config = FromCsvConfig {
            trim: false,
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        // With trim disabled, whitespace is preserved
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("  Alice  ".to_string())
        );
    }

    // ==================== from_csv_reader tests ====================

    #[test]
    fn test_from_csv_reader_basic() {
        let csv_data = "id,name\n1,Alice\n".as_bytes();
        let doc = from_csv_reader(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 1);
    }

    #[test]
    fn test_from_csv_reader_with_config() {
        let csv_data = "1\tAlice\n".as_bytes();
        let config = FromCsvConfig {
            delimiter: b'\t',
            has_headers: false,
            trim: true,
            ..Default::default()
        };
        let doc = from_csv_reader_with_config(csv_data, "Person", &["name"], config).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows.len(), 1);
    }

    // ==================== Type naming tests ====================

    #[test]
    fn test_type_naming_singularization() {
        let csv_data = "id,name\n1,Alice\n";
        let doc = from_csv(csv_data, "User", &["name"]).unwrap();

        // Matrix list should use "users" as key (lowercase + pluralized)
        let item = doc.get("users").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.type_name, "User");
    }

    // ==================== Quoted fields ====================

    #[test]
    fn test_quoted_fields() {
        let csv_data = "id,name,bio\n1,Alice,\"Hello, World\"\n";
        let doc = from_csv(csv_data, "Person", &["name", "bio"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(
            list.rows[0].fields[2],
            Value::String("Hello, World".to_string())
        );
    }

    #[test]
    fn test_quoted_fields_with_newline() {
        let csv_data = "id,name,bio\n1,Alice,\"Line 1\nLine 2\"\n";
        let doc = from_csv(csv_data, "Person", &["name", "bio"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(
            list.rows[0].fields[2],
            Value::String("Line 1\nLine 2".to_string())
        );
    }

    #[test]
    fn test_quoted_fields_with_quotes() {
        let csv_data = "id,name\n1,\"Alice \"\"Bob\"\" Smith\"\n";
        let doc = from_csv(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("Alice \"Bob\" Smith".to_string())
        );
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_unicode_values() {
        let csv_data = "id,name\n1,héllo 世界\n";
        let doc = from_csv(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(
            list.rows[0].fields[1],
            Value::String("héllo 世界".to_string())
        );
    }

    #[test]
    fn test_string_id() {
        let csv_data = "id,name\nabc,Alice\n";
        let doc = from_csv(csv_data, "Person", &["name"]).unwrap();

        let item = doc.get("persons").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.rows[0].id, "abc");
        assert_eq!(list.rows[0].fields[0], Value::String("abc".to_string()));
    }

    #[test]
    fn test_many_columns() {
        let csv_data = "id,a,b,c,d,e\n1,2,3,4,5,6\n";
        let doc = from_csv(csv_data, "Item", &["a", "b", "c", "d", "e"]).unwrap();

        let item = doc.get("items").unwrap();
        let list = item.as_list().unwrap();
        assert_eq!(list.schema.len(), 6); // id + 5 columns
        assert_eq!(list.rows[0].fields.len(), 6);
    }

    // ==================== Custom list_key tests ====================

    #[test]
    fn test_custom_list_key_basic() {
        let csv_data = "id,name\n1,Alice\n";
        let config = FromCsvConfig {
            list_key: Some("people".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();

        // Custom plural should exist
        assert!(doc.get("people").is_some());
        // Default plural should not exist
        assert!(doc.get("persons").is_none());

        let list = doc.get("people").unwrap().as_list().unwrap();
        assert_eq!(list.type_name, "Person");
        assert_eq!(list.rows.len(), 1);
    }

    #[test]
    fn test_custom_list_key_irregular_plurals() {
        // Test common irregular plurals
        let test_cases = vec![
            ("Person", "people"),
            ("Child", "children"),
            ("Tooth", "teeth"),
            ("Foot", "feet"),
            ("Mouse", "mice"),
            ("Goose", "geese"),
            ("Man", "men"),
            ("Woman", "women"),
            ("Ox", "oxen"),
            ("Datum", "data"),
        ];

        for (type_name, plural) in test_cases {
            let csv_data = format!("id,value\n1,test\n");
            let config = FromCsvConfig {
                list_key: Some(plural.to_string()),
                ..Default::default()
            };
            let doc = from_csv_with_config(&csv_data, type_name, &["value"], config).unwrap();

            assert!(
                doc.get(plural).is_some(),
                "Failed to find {} for type {}",
                plural,
                type_name
            );
        }
    }

    #[test]
    fn test_custom_list_key_collective_nouns() {
        let csv_data = "id,value\n1,42\n";

        // Test collective nouns
        let test_cases = vec![
            ("Data", "dataset"),
            ("Information", "info_collection"),
            ("Equipment", "gear"),
            ("Furniture", "furnishings"),
        ];

        for (type_name, collective) in test_cases {
            let config = FromCsvConfig {
                list_key: Some(collective.to_string()),
                ..Default::default()
            };
            let doc = from_csv_with_config(&csv_data, type_name, &["value"], config).unwrap();

            assert!(
                doc.get(collective).is_some(),
                "Failed to find {} for type {}",
                collective,
                type_name
            );
        }
    }

    #[test]
    fn test_custom_list_key_case_sensitive() {
        let csv_data = "id,value\n1,test\n";
        let config = FromCsvConfig {
            list_key: Some("MyCustomList".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(&csv_data, "Item", &["value"], config).unwrap();

        // Exact case should exist
        assert!(doc.get("MyCustomList").is_some());
        // Different case should not exist
        assert!(doc.get("mycustomlist").is_none());
        assert!(doc.get("items").is_none());
    }

    #[test]
    fn test_custom_list_key_empty_string() {
        // Empty string is technically allowed as a key
        let csv_data = "id,value\n1,test\n";
        let config = FromCsvConfig {
            list_key: Some("".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(&csv_data, "Item", &["value"], config).unwrap();

        assert!(doc.get("").is_some());
    }

    #[test]
    fn test_custom_list_key_with_special_chars() {
        let csv_data = "id,value\n1,test\n";
        let config = FromCsvConfig {
            list_key: Some("my-custom_list.v2".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(&csv_data, "Item", &["value"], config).unwrap();

        assert!(doc.get("my-custom_list.v2").is_some());
    }

    #[test]
    fn test_custom_list_key_unicode() {
        let csv_data = "id,value\n1,test\n";
        let config = FromCsvConfig {
            list_key: Some("人々".to_string()), // Japanese for "people"
            ..Default::default()
        };
        let doc = from_csv_with_config(&csv_data, "Person", &["value"], config).unwrap();

        assert!(doc.get("人々").is_some());
    }

    #[test]
    fn test_custom_list_key_with_schema_inference() {
        let csv_data = "id,value\n1,42\n2,43\n3,44\n";
        let config = FromCsvConfig {
            list_key: Some("people".to_string()),
            infer_schema: true,
            sample_rows: 10,
            ..Default::default()
        };
        let doc = from_csv_with_config(&csv_data, "Person", &["value"], config).unwrap();

        assert!(doc.get("people").is_some());
        let list = doc.get("people").unwrap().as_list().unwrap();
        assert_eq!(list.rows.len(), 3);
        // Schema inference should still work
        assert_eq!(list.rows[0].fields[1], Value::Int(42));
    }

    #[test]
    fn test_custom_list_key_none_uses_default() {
        let csv_data = "id,name\n1,Alice\n";
        let config = FromCsvConfig {
            list_key: None,
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();

        // Should use default pluralization
        assert!(doc.get("persons").is_some());
        assert!(doc.get("people").is_none());
    }

    #[test]
    fn test_custom_list_key_default_config() {
        let csv_data = "id,name\n1,Alice\n";
        let doc = from_csv(csv_data, "User", &["name"]).unwrap();

        // Default should use simple pluralization
        assert!(doc.get("users").is_some());
    }

    #[test]
    fn test_custom_list_key_preserves_type_name() {
        let csv_data = "id,name\n1,Alice\n";
        let config = FromCsvConfig {
            list_key: Some("people".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();

        let list = doc.get("people").unwrap().as_list().unwrap();
        // Type name should still be "Person", not "people"
        assert_eq!(list.type_name, "Person");
    }

    #[test]
    fn test_custom_list_key_with_multiple_types() {
        // This test ensures each call can have its own list_key
        let csv1 = "id,name\n1,Alice\n";
        let config1 = FromCsvConfig {
            list_key: Some("people".to_string()),
            ..Default::default()
        };
        let doc1 = from_csv_with_config(csv1, "Person", &["name"], config1).unwrap();

        let csv2 = "id,name\n1,Fluffy\n";
        let config2 = FromCsvConfig {
            list_key: Some("mice".to_string()),
            ..Default::default()
        };
        let doc2 = from_csv_with_config(csv2, "Mouse", &["name"], config2).unwrap();

        assert!(doc1.get("people").is_some());
        assert!(doc1.get("persons").is_none());

        assert!(doc2.get("mice").is_some());
        assert!(doc2.get("mouses").is_none());
    }

    #[test]
    fn test_custom_list_key_numbers_in_name() {
        let csv_data = "id,value\n1,test\n";
        let config = FromCsvConfig {
            list_key: Some("items_v2".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(&csv_data, "Item", &["value"], config).unwrap();

        assert!(doc.get("items_v2").is_some());
    }

    #[test]
    fn test_custom_list_key_round_trip_compatibility() {
        // Ensure custom list keys work with to_csv_list
        let csv_data = "id,name\n1,Alice\n2,Bob\n";
        let config = FromCsvConfig {
            list_key: Some("people".to_string()),
            ..Default::default()
        };
        let doc = from_csv_with_config(csv_data, "Person", &["name"], config).unwrap();

        // Export the list using the custom key
        use crate::to_csv_list;
        let exported_csv = to_csv_list(&doc, "people").unwrap();
        assert!(exported_csv.contains("Alice"));
        assert!(exported_csv.contains("Bob"));

        // Should not be accessible via default key
        assert!(to_csv_list(&doc, "persons").is_err());
    }

    #[test]
    fn test_from_csv_config_clone_with_list_key() {
        let config = FromCsvConfig {
            delimiter: b',',
            has_headers: true,
            trim: true,
            max_rows: 1000,
            infer_schema: false,
            sample_rows: 50,
            list_key: Some("people".to_string()),
        };
        let cloned = config.clone();
        assert_eq!(cloned.list_key, Some("people".to_string()));
    }

    #[test]
    fn test_from_csv_config_debug_with_list_key() {
        let config = FromCsvConfig {
            list_key: Some("people".to_string()),
            ..Default::default()
        };
        let debug = format!("{:?}", config);
        assert!(debug.contains("list_key"));
        assert!(debug.contains("people"));
    }
}
