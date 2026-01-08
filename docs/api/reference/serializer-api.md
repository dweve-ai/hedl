# Serializer API Reference

Complete reference for HEDL serialization and format conversion functions.

## Core Serialization

### canonicalize

Convert document to canonical HEDL format.

```rust
pub fn canonicalize(doc: &Document) -> Result<String, HedlError>
```

**Parameters:**
- `doc`: Document to canonicalize

**Returns:**
- `Ok(String)`: Canonical HEDL string (deterministic, sorted keys)
- `Err(HedlError)`: Serialization error

**Example:**
```rust
use hedl::{parse, canonicalize};

let doc = parse("%VERSION: 1.0\n---\nb: 2\na: 1")?;
let canonical = canonicalize(&doc)?;
// Keys sorted: a before b
```

### canonicalize_with_config

Canonicalize with custom configuration.

```rust
pub fn canonicalize_with_config(
    doc: &Document,
    config: &CanonicalConfig
) -> Result<String, HedlError>
```

**Parameters:**
- `doc`: Document to canonicalize
- `config`: Canonicalization options

**Returns:**
- `Ok(String)`: Canonical HEDL string
- `Err(HedlError)`: Serialization error

**Example:**
```rust
use hedl::c14n::{canonicalize_with_config, CanonicalConfig, QuotingStrategy};

let config = CanonicalConfig::new()
    .with_quoting(QuotingStrategy::Minimal)
    .with_ditto(true)
    .with_sort_keys(true);

let canonical = canonicalize_with_config(&doc, &config)?;
```

## JSON Conversion

### to_json

Convert document to JSON.

```rust
pub fn to_json(doc: &Document) -> Result<String, HedlError>
```

**Parameters:**
- `doc`: Document to convert

**Returns:**
- `Ok(String)`: JSON string
- `Err(HedlError)`: Conversion error

**Example:**
```rust
use hedl::{parse, to_json};

let doc = parse("%VERSION: 1.0\n---\nkey: value")?;
let json = to_json(&doc)?;
// JSON output includes the key-value pair
```

### to_json_value

Convert to serde_json::Value.

```rust
pub fn to_json_value(
    doc: &Document,
    config: &ToJsonConfig
) -> Result<serde_json::Value, String>
```

**Example:**
```rust
use hedl::json::{to_json_value, ToJsonConfig};

let config = ToJsonConfig::default();
let json_value = to_json_value(&doc, &config).map_err(|e| format!("{}", e))?;
assert_eq!(json_value["key"], "value");
```

### from_json

Convert JSON to HEDL document.

```rust
pub fn from_json(json: &str) -> Result<Document, HedlError>
```

**Parameters:**
- `json`: JSON string

**Returns:**
- `Ok(Document)`: HEDL document
- `Err(HedlError)`: Conversion error

**Example:**
```rust
use hedl::from_json;

let json = r#"{"name": "Alice", "age": 30}"#;
let doc = from_json(json)?;
```

### from_json_value

Convert from serde_json::Value.

```rust
pub fn from_json_value(
    value: &serde_json::Value,
    config: &FromJsonConfig
) -> Result<Document, JsonConversionError>
```

**Parameters:**
- `value`: serde_json::Value to convert
- `config`: Conversion configuration

**Returns:**
- `Ok(Document)`: Converted HEDL document
- `Err(JsonConversionError)`: Conversion error

**Example:**
```rust
use hedl::json::{from_json_value, FromJsonConfig};
use serde_json::json;

let value = json!({"name": "Alice"});
let config = FromJsonConfig::default();
    let doc = from_json_value(&value, &config).map_err(|e| format!("{}", e))?;
```

### Partial JSON Parsing

Parse JSON with error recovery, collecting errors instead of failing immediately.

```rust
use hedl::json::{partial_parse_json, PartialConfig, ErrorTolerance};

let config = PartialConfig::builder()
    .tolerance(ErrorTolerance::CollectAll)
    .build();

let result = partial_parse_json(json_str, &config);

if result.is_complete() {
    // Success
    let doc = result.document.unwrap();
} else {
    // Handle errors
    for error in result.errors {
        println!("Error at {}: {}", error.location.path, error.error);
    }
    // Access partial document if available
    if let Some(doc) = result.document {
        println!("Recovered {} items", doc.root.len());
    }
}
```

## YAML Conversion (Feature-Gated)
### to_yaml

Convert document to YAML.

```rust
#[cfg(feature = "yaml")]
pub fn to_yaml(doc: &Document, config: &ToYamlConfig) -> Result<String, YamlError>
```

**Example:**
```rust
#[cfg(feature = "yaml")]
{
    use hedl::yaml::{to_yaml, YamlError};
    let config = hedl::yaml::ToYamlConfig::default();
    let yaml = to_yaml(&doc, &config)?;
}
```

### from_yaml

Convert YAML to HEDL document.

```rust
#[cfg(feature = "yaml")]
pub fn from_yaml(yaml: &str, config: &FromYamlConfig) -> Result<Document, YamlError>
```

## XML Conversion (Feature-Gated)

### to_xml

Convert document to XML.

```rust
#[cfg(feature = "xml")]
pub fn to_xml(doc: &Document, config: &ToXmlConfig) -> Result<String, String>
```

### from_xml

Convert XML to HEDL document.

```rust
#[cfg(feature = "xml")]
pub fn from_xml(xml: &str, config: &FromXmlConfig) -> Result<Document, String>
```

## CSV Conversion (Feature-Gated)

### to_csv

Export document to CSV.

```rust
#[cfg(feature = "csv")]
pub fn to_csv(doc: &Document) -> Result<String, CsvError>
```

### to_csv_with_config

Export with configuration.

```rust
#[cfg(feature = "csv")]
pub fn to_csv_with_config(
    doc: &Document,
    config: ToCsvConfig
) -> Result<String, CsvError>
```

### from_csv

Import CSV to HEDL document.

```rust
#[cfg(feature = "csv")]
pub fn from_csv(
    csv: &str,
    type_name: &str,
    schema: &[&str]
) -> Result<Document, CsvError>
```

### from_csv_with_config

Import with configuration.

```rust
#[cfg(feature = "csv")]
pub fn from_csv_with_config(
    csv: &str,
    type_name: &str,
    schema: &[&str],
    config: FromCsvConfig
) -> Result<Document, CsvError>
```

**Example:**
```rust
#[cfg(feature = "csv")]
{
    use hedl::csv_file::{from_csv_with_config, FromCsvConfig};

    let config = FromCsvConfig {
        has_headers: true,
        delimiter: b',',
        infer_schema: true,
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_data, "User", &["id", "name"], config)?;
}
```

## Parquet Conversion (Feature-Gated)

### to_parquet_bytes

Export to Parquet binary format.

```rust
#[cfg(feature = "parquet")]
pub fn to_parquet_bytes(doc: &Document, config: &ToParquetConfig) -> Result<Vec<u8>, ParquetError>
```

### from_parquet_bytes

Import from Parquet binary.

```rust
#[cfg(feature = "parquet")]
pub fn from_parquet_bytes(bytes: &[u8]) -> Result<Document, ParquetError>
```

**Parameters:**
- `bytes`: Parquet file bytes

**Returns:**
- `Ok(Document)`: Converted HEDL document
- `Err(ParquetError)`: Conversion error

**Example:**
```rust
#[cfg(feature = "parquet")]
{
    use hedl::parquet::{to_parquet_bytes, from_parquet_bytes, ToParquetConfig};
    use std::fs;

    // Export
    let config = ToParquetConfig::default();
    let bytes = to_parquet_bytes(&doc, &config)?;
    fs::write("data.parquet", bytes)?;

    // Import
    let bytes = fs::read("data.parquet")?;
    let doc = from_parquet_bytes(&bytes)?;
}
```

## Neo4j/Cypher Conversion (Feature-Gated)

### to_cypher

Export to Cypher CREATE statements.

```rust
#[cfg(feature = "neo4j")]
pub fn to_cypher(doc: &Document, config: &ToCypherConfig) -> Result<String, Neo4jError>
```

### to_cypher_statements

Export to structured Cypher statements.

```rust
#[cfg(feature = "neo4j")]
pub fn to_cypher_statements(doc: &Document) -> Result<Vec<CypherStatement>, Neo4jError>
```

**Example:**
```rust
#[cfg(feature = "neo4j")]
{
    use hedl::neo4j::{to_cypher, ToCypherConfig};

    let config = ToCypherConfig {
        batch_size: 1000,
        use_merge: true,
        ..Default::default()
    };

    let cypher = to_cypher(&doc, &config)?;
    // CREATE (n:User {id: 'alice', name: 'Alice'})...
}
```

## TOON Conversion (Feature-Gated)

### hedl_to_toon

Convert document to TOON string with default configuration.

```rust
#[cfg(feature = "toon")]
pub fn hedl_to_toon(doc: &Document) -> Result<String, ToonError>
```

### to_toon

Convert document to TOON with custom configuration.

```rust
#[cfg(feature = "toon")]
pub fn to_toon(
    doc: &Document,
    config: &ToToonConfig
) -> Result<String, ToonError>
```

**Parameters:**
- `doc`: Document to convert
- `config`: TOON configuration

**Returns:**
- `Ok(String)`: TOON string
- `Err(ToonError)`: Conversion error

**Example:**
```rust
#[cfg(feature = "toon")]
{
    use hedl::toon::{to_toon, ToToonConfig, Delimiter};

    let config = ToToonConfig::new()
        .with_indent(4)
        .with_delimiter(Delimiter::Tab);

    let toon = to_toon(&doc, &config)?;
}
```

## Configuration Types

### ToJsonConfig

```rust
pub struct ToJsonConfig {
    pub include_metadata: bool,    // Include HEDL metadata in output
    pub flatten_lists: bool,       // Flatten matrix lists to arrays
    pub include_children: bool,    // Include nested children
}
```

### FromJsonConfig

```rust
pub struct FromJsonConfig {
    pub default_type_name: String,       // Default type name for objects
    pub version: (u32, u32),             // HEDL version to use
    pub max_depth: Option<usize>,        // Maximum nesting depth
    pub max_array_size: Option<usize>,   // Maximum array size
    pub max_string_length: Option<usize>, // Maximum string length
    pub max_object_size: Option<usize>,  // Maximum object size
}
```

### CanonicalConfig

```rust
pub struct CanonicalConfig {
    pub quoting: QuotingStrategy,  // String quoting strategy
    pub use_ditto: bool,           // Use ditto optimization in matrix rows
    pub sort_keys: bool,           // Sort object keys alphabetically
    pub inline_schemas: bool,      // Use inline schemas vs header directives
}

pub enum QuotingStrategy {
    Minimal,                       // Quote only when necessary
    Always,                        // Always quote strings
}
```

## Streaming Serialization

For large documents, use streaming writers:

```rust
use hedl::c14n::CanonicalWriter;
use std::io::Write;

let mut buffer = Vec::new();
let mut writer = CanonicalWriter::new(&mut buffer);

writer.write_document(&doc)?;

let hedl_string = String::from_utf8(buffer)?;
```

## Performance Optimization

### Reusing Buffers

```rust
use std::io::Cursor;

struct Serializer {
    buffer: Vec<u8>,
}

impl Serializer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
        }
    }

    pub fn serialize(&mut self, doc: &Document) -> Result<String, HedlError> {
        self.buffer.clear();
        let mut writer = hedl::c14n::CanonicalWriter::new(&mut self.buffer);
        writer.write_document(doc)?;
        Ok(String::from_utf8(self.buffer.clone()).unwrap())
    }
}
```

### Parallel Conversion

```rust
use rayon::prelude::*;

let json_strings: Vec<Result<String, HedlError>> = documents
    .par_iter()
    .map(|doc| hedl::to_json(doc))
    .collect();
```

## Error Handling

```rust
use hedl::{to_json, HedlErrorKind};

match to_json(&doc) {
    Ok(json) => println!("{}", json),
    Err(e) => match e.kind {
        HedlErrorKind::Conversion => {
            eprintln!("Conversion error: {}", e.message);
        }
        _ => eprintln!("Unexpected error: {}", e.message),
    }
}
```

## See Also

- [Core Types](core-types.md) - Type definitions
- [Parser API](parser-api.md) - Parsing functions
- [Utility Functions](utility-functions.md) - Helper functions
- [Examples](../examples.md) - Code examples
