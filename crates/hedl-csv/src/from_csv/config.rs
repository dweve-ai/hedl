// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Configuration types for CSV import

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
/// - **Trade-off**: Higher limits allow larger datasets but increase `DoS` risk.
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

/// Default maximum number of columns to prevent column bomb attacks.
///
/// This limit prevents Denial-of-Service attacks from CSV files with excessive columns.
/// The default is 10,000 columns, which is generous but prevents abuse.
///
/// # Security Considerations
///
/// - **Column bomb**: Without a limit, attackers could provide CSV files with
///   hundreds of thousands of columns, causing memory exhaustion and slow processing.
/// - **Industry standards**: Excel limits to 16,384 columns, Google Sheets to 18,278.
/// - **Trade-off**: Higher limits allow wider datasets but increase `DoS` risk.
pub const DEFAULT_MAX_COLUMNS: usize = 10_000;

/// Default maximum cell size in bytes to prevent cell bomb attacks.
///
/// This limit prevents Denial-of-Service attacks from CSV files with enormous cells.
/// The default is 1MB per cell, which is reasonable for most legitimate use cases.
///
/// # Security Considerations
///
/// - **Cell bomb**: Without a limit, attackers could provide CSV files with
///   gigabyte-sized cells, causing memory exhaustion.
/// - **Cumulative effect**: Multiple large cells multiply the impact.
/// - **Trade-off**: Higher limits allow larger text fields but increase `DoS` risk.
pub const DEFAULT_MAX_CELL_SIZE: usize = 1_048_576; // 1MB

/// Default maximum total CSV size in bytes to prevent decompression bombs.
///
/// This limit prevents Denial-of-Service attacks from compressed CSV files that
/// decompress to enormous sizes. The default is 100MB.
///
/// # Security Considerations
///
/// - **Decompression bomb**: A 1MB gzipped file could decompress to 1GB+.
/// - **Memory exhaustion**: Prevents attackers from filling server memory.
/// - **Trade-off**: Higher limits allow larger datasets but increase `DoS` risk.
pub const DEFAULT_MAX_TOTAL_SIZE: usize = 104_857_600; // 100MB

/// Default maximum header size in bytes to prevent header bombs.
///
/// This limit prevents Denial-of-Service attacks from CSV files with enormous headers.
/// The default is 1MB for the total header size.
///
/// # Security Considerations
///
/// - **Header bomb**: Prevents attackers from using huge column names.
/// - **Per-column**: Also enforced per-column via `max_cell_size`.
/// - **Trade-off**: Higher limits allow longer column names but increase `DoS` risk.
pub const DEFAULT_MAX_HEADER_SIZE: usize = 1_048_576; // 1MB

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
    /// - **`DoS` Protection**: Prevents attackers from causing memory exhaustion
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

    /// Maximum number of columns allowed (default: 10,000).
    ///
    /// This security limit prevents "column bomb" attacks where malicious CSV files
    /// contain excessive columns that cause memory exhaustion and slow processing.
    ///
    /// # Security Impact
    ///
    /// - **`DoS` Protection**: Prevents attackers from creating CSVs with 50,000+ columns
    /// - **Memory Bound**: Limits worst-case memory usage for column metadata
    /// - **Industry Comparison**: Excel (16,384), Google Sheets (18,278), `PostgreSQL` (~1,600)
    /// - **Recommended Values**:
    ///   - Web uploads: 1,000 - 10,000 columns
    ///   - Internal processing: 10,000 - 50,000 columns
    ///   - Scientific data: Adjust based on requirements
    ///
    /// # Example
    ///
    /// ```
    /// # use hedl_csv::FromCsvConfig;
    /// // For processing wide scientific datasets
    /// let config = FromCsvConfig {
    ///     max_columns: 50_000,
    ///     ..Default::default()
    /// };
    /// ```
    pub max_columns: usize,

    /// Maximum size of a single cell in bytes (default: 1MB).
    ///
    /// This security limit prevents "cell bomb" attacks where malicious CSV files
    /// contain enormous individual cells that cause memory exhaustion.
    ///
    /// # Security Impact
    ///
    /// - **`DoS` Protection**: Prevents attackers from using 10MB+ cells
    /// - **Memory Bound**: Each cell is read into memory as a String
    /// - **Cumulative**: Multiple large cells multiply the impact
    /// - **Recommended Values**:
    ///   - Web uploads: 64KB - 1MB
    ///   - Internal processing: 1MB - 10MB
    ///   - Text-heavy data: Adjust based on requirements
    ///
    /// # Example
    ///
    /// ```
    /// # use hedl_csv::FromCsvConfig;
    /// // For processing long text fields (e.g., descriptions, comments)
    /// let config = FromCsvConfig {
    ///     max_cell_size: 5_242_880, // 5MB
    ///     ..Default::default()
    /// };
    /// ```
    pub max_cell_size: usize,

    /// Maximum total CSV size in bytes after decompression (default: 100MB).
    ///
    /// This security limit prevents "decompression bomb" attacks where compressed
    /// CSV files decompress to enormous sizes. A 1MB gzipped file could decompress
    /// to 1GB+, bypassing file size checks.
    ///
    /// # Security Impact
    ///
    /// - **`DoS` Protection**: Prevents decompression bombs
    /// - **Memory Bound**: Tracks total bytes read during parsing
    /// - **Transparent**: Works even if CSV library handles decompression
    /// - **Recommended Values**:
    ///   - Web uploads: 10MB - 100MB
    ///   - Internal processing: 100MB - 1GB
    ///   - Big data: Adjust based on available RAM
    ///
    /// # Example
    ///
    /// ```
    /// # use hedl_csv::FromCsvConfig;
    /// // For processing large datasets on high-memory servers
    /// let config = FromCsvConfig {
    ///     max_total_size: 1_073_741_824, // 1GB
    ///     ..Default::default()
    /// };
    /// ```
    pub max_total_size: usize,

    /// Maximum size of header row in bytes (default: 1MB).
    ///
    /// This security limit prevents "header bomb" attacks where malicious CSV files
    /// have enormous column names or excessive total header size.
    ///
    /// # Security Impact
    ///
    /// - **`DoS` Protection**: Prevents huge column names (e.g., 1MB per column)
    /// - **Memory Bound**: Limits memory for header parsing
    /// - **Combined with `max_columns`**: Total size = `column_count` × `avg_name_length`
    /// - **Recommended Values**:
    ///   - Web uploads: 64KB - 1MB
    ///   - Internal processing: 1MB - 10MB
    ///   - Verbose column naming: Adjust based on requirements
    ///
    /// # Example
    ///
    /// ```
    /// # use hedl_csv::FromCsvConfig;
    /// // For datasets with very descriptive column names
    /// let config = FromCsvConfig {
    ///     max_header_size: 5_242_880, // 5MB
    ///     ..Default::default()
    /// };
    /// ```
    pub max_header_size: usize,
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
            max_columns: DEFAULT_MAX_COLUMNS,
            max_cell_size: DEFAULT_MAX_CELL_SIZE,
            max_total_size: DEFAULT_MAX_TOTAL_SIZE,
            max_header_size: DEFAULT_MAX_HEADER_SIZE,
        }
    }
}

impl FromCsvConfig {
    /// Creates a config with NO security limits (use for trusted input only).
    ///
    /// # Security Warning
    ///
    /// This configuration disables ALL security limits. Only use this for:
    /// - Trusted internal data sources
    /// - Controlled batch processing environments
    /// - Known-good CSV files
    ///
    /// **DO NOT** use this for:
    /// - User uploads
    /// - Web service inputs
    /// - Untrusted data sources
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_csv::FromCsvConfig;
    /// // For internal batch processing with trusted data
    /// let config = FromCsvConfig::unlimited();
    /// ```
    #[must_use]
    pub fn unlimited() -> Self {
        Self {
            max_rows: usize::MAX,
            max_columns: usize::MAX,
            max_cell_size: usize::MAX,
            max_total_size: usize::MAX,
            max_header_size: usize::MAX,
            ..Default::default()
        }
    }

    /// Creates a config with strict limits for untrusted input.
    ///
    /// # Security
    ///
    /// This configuration provides stricter limits suitable for:
    /// - Web service uploads
    /// - User-submitted CSV files
    /// - Untrusted data sources
    /// - Rate-limited APIs
    ///
    /// # Limits
    ///
    /// - `max_rows`: 1,000,000 (same as default)
    /// - `max_columns`: 1,000 (stricter than default 10,000)
    /// - `max_cell_size`: 64KB (stricter than default 1MB)
    /// - `max_total_size`: 10MB (stricter than default 100MB)
    /// - `max_header_size`: 64KB (stricter than default 1MB)
    ///
    /// # Examples
    ///
    /// ```
    /// # use hedl_csv::FromCsvConfig;
    /// // For user uploads in a web service
    /// let config = FromCsvConfig::strict();
    /// ```
    #[must_use]
    pub fn strict() -> Self {
        Self {
            max_rows: 1_000_000,
            max_columns: 1_000,
            max_cell_size: 65_536,
            max_total_size: 10_485_760,
            max_header_size: 65_536,
            ..Default::default()
        }
    }
}
