# hedl-csv

**HEDL's CSV integration -bidirectional conversion with flexible configuration, type inference, and security limits.**

CSV is the universal data export format. Your spreadsheets export it. Your databases dump it. Your analytics tools import it. Your data science workflows depend on it. But CSV lacks types, schemas, and structure. Every field is a string. Every export is a guessing game.

`hedl-csv` bridges HEDL's structured matrix lists with CSV's simplicity. Parse CSV files into typed HEDL documents with automatic type inference. Export HEDL matrix lists to CSV for compatibility with spreadsheets, databases, and legacy ETL tools. Configure delimiters, handle irregular plurals, enforce security limits.

Part of the **HEDL format family** alongside `hedl-json`, `hedl-yaml`, `hedl-xml`, and `hedl-parquet` -bringing HEDL's structure to every ecosystem you work in.

## What's Implemented

Bidirectional conversion with comprehensive configuration:

1. **CSV → HEDL Parsing**: Parse CSV files into HEDL matrix lists with automatic type inference
2. **HEDL → CSV Export**: Export HEDL matrix lists to CSV for spreadsheets and databases
3. **Type Inference**: Automatic detection of null, bool, int, float, string, references, expressions, tensors
4. **Schema Inference**: Optional column-level type inference from sampled rows
5. **Flexible Configuration**: Custom delimiters (comma, tab, semicolon, pipe), headers, trimming, quote styles
6. **Custom List Keys**: Support for irregular plurals and custom naming conventions
7. **Security Limits**: Configurable row limits to prevent memory exhaustion (default: 1M rows)
8. **Streaming I/O**: Process files larger than available RAM with row-by-row processing
9. **Selective Export**: Export specific matrix lists from multi-list documents
10. **Comprehensive Error Handling**: Detailed errors with line numbers and context

## Installation

```toml
[dependencies]
hedl-csv = "2.0"
```

## Bidirectional Conversion

### CSV → HEDL: Parse Tabular Data

Convert CSV files into HEDL's typed matrix list structures:

```rust
use hedl_csv::from_csv;

// Parse CSV with automatic type inference
let csv = r#"id,name,age,active
alice,Alice Smith,30,true
bob,Bob Jones,25,false
carol,Carol White,35,true"#;

// Default configuration (comma delimiter, headers, trimming)
// Note: schema parameter excludes the 'id' column
let doc = from_csv(csv, "User", &["name", "age", "active"])?;

// Resulting HEDL structure:
// users: @User[id, name, age, active]
//   | alice, Alice Smith, 30, true
//   | bob, Bob Jones, 25, false
//   | carol, Carol White, 35, true
```

**Type Inference**: CSV fields are automatically inferred as null, bool, int, float, or string based on content. `"30"` → Int(30), `"true"` → Bool(true), `"Alice Smith"` → String("Alice Smith").

### Custom Configuration

Fine-tune parsing with `FromCsvConfig`:

```rust
use hedl_csv::{from_csv_with_config, FromCsvConfig};

let tsv_data = "id\tname\tage\n1\tAlice\t30\n2\tBob\t25";

let config = FromCsvConfig {
    delimiter: b'\t',          // Tab-separated values
    has_headers: true,         // First row contains headers
    trim: true,                // Trim whitespace
    max_rows: 100_000,         // Security limit (100K rows)
    infer_schema: true,        // Column-level type inference
    sample_rows: 50,           // Sample 50 rows for schema
    list_key: Some("people".to_string()), // Custom key (irregular plural)
    max_columns: 10_000,       // Maximum columns (default)
    max_cell_size: 1_048_576,  // Maximum cell size (1MB, default)
    max_total_size: 104_857_600, // Maximum total size (100MB, default)
    max_header_size: 1_048_576,  // Maximum header size (1MB, default)
};

let doc = from_csv_with_config(tsv_data, "Person", &["name", "age"], config)?;
// List key is "people" instead of default "persons"
```

### Custom List Keys: Irregular Plurals

Support for irregular plurals and custom naming conventions:

```rust
use hedl_csv::{from_csv, from_csv_with_config, FromCsvConfig};

let csv = "id,name,age\n1,Alice,30\n2,Bob,25";

// Default pluralization: adds 's' to lowercased type name
let doc = from_csv(csv, "User", &["name", "age"])?;  // List key: "users"

// Custom key for irregular plurals
let config = FromCsvConfig {
    list_key: Some("people".to_string()),
    ..Default::default()
};
let doc = from_csv_with_config(csv, "Person", &["name", "age"], config)?;  // List key: "people"

// Other irregular plurals
let config = FromCsvConfig { list_key: Some("children".to_string()), ..Default::default() };
let doc = from_csv_with_config(csv, "Child", &["name", "age"], config)?;

let config = FromCsvConfig { list_key: Some("mice".to_string()), ..Default::default() };
let doc = from_csv_with_config(csv, "Mouse", &["name", "age"], config)?;
```

### Streaming Large CSV Files

Process files larger than available RAM with row-by-row streaming:

```rust
use hedl_csv::{from_csv_reader_with_config, FromCsvConfig};
use std::fs::File;

// Open large CSV file (e.g., 10 GB database export)
let file = File::open("massive_export.csv")?;

let config = FromCsvConfig {
    max_rows: 10_000_000,  // 10M row limit
    ..Default::default()
};

// Streams row-by-row without loading entire file into memory
let doc = from_csv_reader_with_config(file, "Transaction", &["amount", "date", "status"], config)?;
```

**Memory Usage**: O(1) per row. A 10 GB CSV uses the same memory as a 10 MB CSV -only the current row and output buffer are in memory.

### HEDL → CSV: Export for Analysis

Export HEDL matrix lists to CSV for spreadsheets, databases, or legacy tools:

```rust
use hedl_csv::{to_csv, ToCsvConfig};

let doc = hedl_core::parse(br#"
%S:Product:[id, name, price, stock]
---
products: @Product
 | p1, Widget, 19.99, 100
 | p2, Gadget, 29.99, 50
 | p3, Doohickey, 9.99, 200
"#)?;

// Export to CSV (default config: comma delimiter, headers included)
let csv = to_csv(&doc)?;
```

Generated CSV:

```csv
id,name,price,stock
p1,Widget,19.99,100
p2,Gadget,29.99,50
p3,Doohickey,9.99,200
```

### Custom CSV Output Configuration

```rust
use hedl_csv::{to_csv_with_config, ToCsvConfig};
use csv::QuoteStyle;

let doc = hedl_core::parse(br#"
%S:Product:[id, name, price, stock]
---
products: @Product
 | p1, Widget, 19.99, 100
 | p2, Gadget, 29.99, 50
"#)?;

let config = ToCsvConfig {
    delimiter: b';',                  // Semicolon delimiter
    include_headers: false,            // No header row
    quote_style: QuoteStyle::Always,   // Always quote fields
};

let csv = to_csv_with_config(&doc, config)?;
```

### Selective List Export

Export only specific matrix lists from multi-list documents:

```rust
use hedl_csv::to_csv_list;

let doc = hedl_core::parse(br#"
users: @User[id, name]
 | alice, Alice
 | bob, Bob

products: @Product[id, name, price]
 | p1, Widget, 19.99
 | p2, Gadget, 29.99
"#)?;

// Export only the products list
let products_csv = to_csv_list(&doc, "products")?;
```

## Type Inference

CSV values are inferred in this hierarchical order:

### 1. Null Values

```csv
id,value,description
1,,    # → Value::Null
2,~,   # → Value::Null (explicit null)
3,"",  # → Value::Null (empty string after trim)
```

### 2. Boolean Values

```csv
id,active
1,true   # → Value::Bool(true)
2,false  # → Value::Bool(false)
```

### 3. Integer Values

```csv
id,count
1,42      # → Value::Int(42)
2,-123    # → Value::Int(-123)
```

### 4. Float Values

```csv
id,price,special
1,19.99,       # → Value::Float(19.99)
2,1.5e10,      # → Value::Float(1.5e10)
3,NaN,         # → Value::Float(NaN)
4,Infinity,    # → Value::Float(∞)
5,-Infinity,   # → Value::Float(-∞)
```

### 5. Reference Values

```csv
id,owner
1,@alice        # → Value::Reference(local("alice"))
2,@User:bob     # → Value::Reference(qualified("User", "bob"))
```

### 6. Expression Values

```csv
id,computed
1,$(revenue * 0.1)      # → Value::Expression("revenue * 0.1")
2,$(price + tax)        # → Value::Expression("price + tax")
```

### 7. Tensor/Array Values

```csv
id,coordinates,matrix
1,"[1, 2, 3]","[[1,2],[3,4]]"  # → Value::Tensor
```

### 8. String Values (Fallback)

Everything else becomes a string:

```csv
id,name,description
1,Alice Smith,Regular text content  # → Value::String
```

## Schema Inference

For automatic column-level type detection:

```rust
use hedl_csv::{from_csv_with_config, FromCsvConfig};

let csv = "id,count,score,active\n1,42,95.5,true\n2,87,88.3,false";

let config = FromCsvConfig {
    infer_schema: true,   // Enable schema inference
    sample_rows: 100,     // Sample first 100 rows
    ..Default::default()
};

let doc = from_csv_with_config(csv, "Record", &["count", "score", "active"], config)?;
```

**How it works**:
1. Collects first N rows (sample_rows)
2. Determines most specific type for each column that accommodates all samples
3. Re-processes all rows with inferred types
4. Falls back to per-value inference if schema inference fails for specific cells

**Type hierarchy**: Null → Bool → Int → Float → String (most general)

## Security Limits: DoS Protection

`hedl-csv` enforces row count limits to prevent memory exhaustion from malicious or unexpectedly large CSV files:

### Default Limit: 1,000,000 Rows

```rust
use hedl_csv::{from_csv, FromCsvConfig};

let csv = "id,value\n1,42\n2,87";

// Default configuration has 1M row limit
let doc = from_csv(csv, "Record", &["value"])?;

// Parsing stops with SecurityLimit error if exceeded:
// Error: SecurityLimit { limit: 1000000, actual: 1000001 }
```

### Custom Limits for Different Contexts

**Small Deployments** (limited RAM):

```rust
let config = FromCsvConfig {
    max_rows: 100_000,  // 100K rows
    ..Default::default()
};
```

**Large Deployments** (dedicated data processing):

```rust
let config = FromCsvConfig {
    max_rows: 10_000_000,  // 10M rows
    ..Default::default()
};
```

**Trusted Internal Data** (no limit):

```rust
let config = FromCsvConfig {
    max_rows: usize::MAX,  // No practical limit
    ..Default::default()
};
```

## Error Handling

Comprehensive error types with context:

```rust
use hedl_csv::{from_csv, CsvError};

let csv = "id,name,age\n1,Alice,30";

match from_csv(csv, "User", &["name", "age"]) {
    Ok(doc) => { /* process document */ }
    Err(CsvError::ParseError { line, message }) => {
        eprintln!("CSV parse error at line {}: {}", line, message);
    }
    Err(CsvError::TypeMismatch { column, expected, value }) => {
        eprintln!("Type mismatch in column '{}': expected {}, found '{}'",
            column, expected, value);
    }
    Err(CsvError::WidthMismatch { expected, actual, row }) => {
        eprintln!("Row {} has {} columns, expected {}", row, actual, expected);
    }
    Err(CsvError::SecurityLimit { limit, actual }) => {
        eprintln!("Row limit exceeded: {} rows (limit: {})", actual, limit);
    }
    Err(CsvError::EmptyId { row }) => {
        eprintln!("Empty ID field at row {}", row);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### Contextual Error Messages

Add context to errors for better debugging:

```rust
use hedl_csv::from_csv;

let csv = "id,name,age\n1,Alice,30";

let doc = from_csv(csv, "User", &["name", "age"])
    .map_err(|e| format!("Error importing user data: {}", e))?;
// Error: "Error importing user data: ..."
```

## Configuration Reference

### FromCsvConfig

```rust
use hedl_csv::FromCsvConfig;

let config = FromCsvConfig {
    delimiter: b',',             // Field delimiter (default: comma)
    has_headers: true,           // First row contains column names (default: true)
    trim: true,                  // Trim whitespace from fields (default: true)
    max_rows: 1_000_000,         // Maximum rows to process (default: 1M)
    infer_schema: false,         // Infer column types from samples (default: false)
    sample_rows: 100,            // Rows to sample for schema inference (default: 100)
    list_key: None,              // Custom list key (default: type_name.to_lowercase() + "s")
    max_columns: 10_000,         // Maximum columns allowed (default: 10K)
    max_cell_size: 1_048_576,    // Maximum cell size in bytes (default: 1MB)
    max_total_size: 104_857_600, // Maximum total size in bytes (default: 100MB)
    max_header_size: 1_048_576,  // Maximum header size in bytes (default: 1MB)
};
```

**Common Delimiters**:
- `b','` - Comma (CSV)
- `b'\t'` - Tab (TSV)
- `b';'` - Semicolon (European CSV)
- `b'|'` - Pipe (database exports)

**Security Limits** (prevent DoS attacks):
- `max_rows` - Prevents unbounded memory allocation from huge datasets
- `max_columns` - Prevents column bomb attacks (default: 10K columns)
- `max_cell_size` - Prevents cell bomb attacks with gigantic fields (default: 1MB per cell)
- `max_total_size` - Prevents decompression bomb attacks (default: 100MB total)
- `max_header_size` - Prevents header bomb with enormous column names (default: 1MB header)

**Convenience Methods**:
```rust
use hedl_csv::FromCsvConfig;

// For trusted internal data (no limits)
let config = FromCsvConfig::unlimited();

// For untrusted user input (stricter limits)
let config = FromCsvConfig::strict();
```

### ToCsvConfig

```rust
use hedl_csv::ToCsvConfig;
use csv::QuoteStyle;

let config = ToCsvConfig {
    delimiter: b',',                    // Field delimiter (default: comma)
    include_headers: true,              // Include column names (default: true)
    quote_style: QuoteStyle::Necessary, // Quote when needed (default)
};
```

**Quote Styles**:
- `QuoteStyle::Necessary` - Only quote fields containing delimiters, quotes, or newlines
- `QuoteStyle::Always` - Quote all fields
- `QuoteStyle::Never` - Never quote (may produce invalid CSV)
- `QuoteStyle::NonNumeric` - Quote non-numeric fields

## Use Cases

**Database Export/Import**: Export database query results to CSV, parse with type inference, transform with HEDL's structured API, reimport to database.

**Spreadsheet Integration**: Parse Excel/Google Sheets CSV exports into typed HEDL structures. Export HEDL data to CSV for analysts who prefer spreadsheets.

**Data Pipeline Integration**: Convert CSV logs and exports to HEDL for structured querying. Combine with JSON APIs (`hedl-json`) and XML feeds (`hedl-xml`) in unified workflows.

**ML Feature Engineering**: Parse CSV datasets with type inference, compute derived features with HEDL expressions, export to CSV for training.

**ETL Workflows**: Read CSV from legacy systems, validate and transform with HEDL, export to modern formats (JSON, Parquet) or back to CSV for compatibility.

**Report Generation**: Query databases, aggregate in HEDL, export to CSV for Excel pivot tables and charts.

## What This Crate Doesn't Do

**Schema Preservation**: CSV has no schema concept. HEDL's `%STRUCT`, `%NEST`, `%ALIAS` declarations are lost in CSV export. If you need schemas, use HEDL source files or define validation rules with `hedl-lint`.

**Nested Data**: CSV is flat. HEDL matrix lists with nested children (via `%NEST`) are flattened -only the parent list fields are exported, nested children are skipped.

**Complex Types**: CSV represents everything as strings. Type inference helps but can't handle arbitrary complex types. Use JSON or Parquet for rich nested structures.

**Multi-List Export**: `to_csv()` exports only the first matrix list found. For documents with multiple lists, use `to_csv_list(doc, "specific_list")` for selective export.

## Performance Characteristics

**Parsing**: Streaming row-by-row processing provides O(1) memory per row. Throughput: ~50-100 MB/s depending on column count and type inference complexity.

**Schema Inference**: When enabled, collects all rows into memory first, then re-processes. Memory: O(rows × columns). Time overhead: +10-20% for sampling and re-processing.

**Export**: Buffer pre-allocation provides 1.1-1.2x speedup. Estimated capacity: rows × columns × 20 bytes/cell. Throughput: ~100-150 MB/s.

**Type Inference**: Per-value inference is O(1) per field. Hierarchical type checking (null → bool → int → float → string) averages 3-4 checks per field.

Detailed performance benchmarks are available in the HEDL repository benchmark suite.

## Dependencies

- `hedl-core` 2.0 - HEDL parsing and data model
- `csv` 1.3 - High-performance CSV parsing and writing
- `thiserror` 1.0 - Error type definitions

## License

Apache-2.0
