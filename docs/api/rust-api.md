# Rust API Reference

**Native Rust library for high-performance HEDL processing**

---

## Quick Start

```rust
use hedl::{parse, canonicalize, to_json, validate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a HEDL document
    let hedl = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
    "#;

    let doc = parse(hedl)?;

    // Convert to JSON
    let json = to_json(&doc)?;

    // Canonicalize
    let canonical = canonicalize(&doc)?;

    // Validate
    validate(hedl)?;

    Ok(())
}
```

---

## Core Functions

### Parsing

#### `parse`

Parse a HEDL document from a string.

```rust
pub fn parse(input: &str) -> Result<Document, HedlError>
```

**Parameters**:
- `input`: HEDL document as UTF-8 string

**Returns**: `Result<Document, HedlError>`

**Example**:
```rust
let doc = hedl::parse("%VERSION: 1.0\n---\nkey: value")?;
assert_eq!(doc.version, (1, 0));
```

**Performance**: Optimized with `#[inline]` hint for 5-10% improvement in small document scenarios.

---

#### `parse_lenient`

Parse with lenient reference handling (unresolved references become `null`).

```rust
pub fn parse_lenient(input: &str) -> Result<Document, HedlError>
```

**Example**:
```rust
// This will succeed even with invalid references
let doc = hedl::parse_lenient("%VERSION: 1.0\n---\nuser: @InvalidRef")?;
```

---

#### `parse_with_limits`

Parse with custom resource limits and options.

```rust
pub fn parse_with_limits(
    input: &[u8],
    options: ParseOptions
) -> Result<Document, HedlError>
```

**Example**:
```rust
use hedl::{parse_with_limits, ParseOptions, Limits};

let options = ParseOptions {
    strict_refs: false,
    limits: Limits {
        max_indent_depth: 100,
        ..Limits::default()
    },
};

let doc = parse_with_limits(input.as_bytes(), options)?;
```

---

### Validation

#### `validate`

Validate HEDL input without fully parsing.

```rust
pub fn validate(input: &str) -> Result<(), HedlError>
```

**Returns**: `Ok(())` if valid, `Err(HedlError)` with details if invalid.

**Example**:
```rust
match hedl::validate(hedl_input) {
    Ok(()) => println!("Valid HEDL"),
    Err(e) => eprintln!("Invalid: {} at line {}", e.message, e.line),
}
```

---

#### `lint`

Check document for best practices and potential issues.

```rust
pub fn lint(doc: &Document) -> Vec<lint::Diagnostic>
```

**Returns**: List of diagnostics (errors, warnings, hints)

**Example**:
```rust
let doc = hedl::parse(input)?;
let diagnostics = hedl::lint(&doc);

for d in diagnostics {
    println!("[{}] {}: {}", d.severity(), d.rule_id(), d.message());
}
```

---

### Canonicalization

#### `canonicalize`

Convert document to canonical (deterministic) form.

```rust
pub fn canonicalize(doc: &Document) -> Result<String, HedlError>
```

**Features**:
- Sorted keys for deterministic output
- Ditto operator optimization
- Consistent whitespace
- Suitable for hashing and diffing

**Example**:
```rust
let doc = hedl::parse("%VERSION: 1.0\n---\nz: 3\na: 1")?;
let canonical = hedl::canonicalize(&doc)?;
// Keys are sorted: a appears before z
```

---

### Format Conversion

#### `to_json`

Convert HEDL document to JSON.

```rust
pub fn to_json(doc: &Document) -> Result<String, HedlError>
```

**Example**:
```rust
let doc = hedl::parse(hedl_input)?;
let json = hedl::to_json(&doc)?;
println!("{}", json);
```

---

#### `from_json`

Convert JSON to HEDL document.

```rust
pub fn from_json(json: &str) -> Result<Document, HedlError>
```

**Example**:
```rust
let json = r#"{"users": [{"id": "alice", "name": "Alice"}]}"#;
let doc = hedl::from_json(json)?;
```

---

## Data Types

### `Document`

Represents a parsed HEDL document.

```rust
pub struct Document {
    pub version: (u32, u32),
    pub aliases: BTreeMap<String, String>,
    pub structs: BTreeMap<String, Vec<String>>,
    pub nests: BTreeMap<String, String>,
    pub root: BTreeMap<String, Item>,
}
```

**Fields**:
- `version`: HEDL format version (e.g., `(1, 0)`)
- `aliases`: Alias definitions mapping alias names to string values
- `structs`: Schema definitions mapping type names to field lists
- `nests`: Parent-child type relationships (parent type → child type)
- `root`: Top-level items in the document body

---

### `Item`

Represents an item in the document body.

```rust
pub enum Item {
    Scalar(Value),
    Object(BTreeMap<String, Item>),
    List(MatrixList),
}
```

**Variants**:
- `Scalar`: Single value
- `Object`: Nested object/map
- `List`: Matrix list of typed entities

---

### `Value`

Represents a HEDL scalar value.

```rust
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Tensor(Tensor),
    Reference(Reference),
    Expression(Expression),
}
```

---

### `Reference`

Represents an entity reference.

```rust
pub struct Reference {
    pub type_name: Option<String>,
    pub id: String,
}
```

**Examples**:
- `@alice` → `Reference { type_name: None, id: "alice" }`
- `@User:alice` → `Reference { type_name: Some("User"), id: "alice" }`

---

### `Node`

Represents an entity/row in a matrix list.

```rust
pub struct Node {
    pub type_name: String,
    pub id: String,
    pub fields: Vec<Value>,
    pub children: BTreeMap<String, Vec<Node>>,
    pub child_count: Option<usize>,
}
```

**Fields**:
- `type_name`: Entity type (from schema)
- `id`: Entity identifier (first column value)
- `fields`: Field values (parallel to schema columns)
- `children`: Nested child entities by type (from NEST relationships)
- `child_count`: Optional count hint for LLM comprehension

---

### `MatrixList`

Represents a typed matrix list.

```rust
pub struct MatrixList {
    pub type_name: String,
    pub schema: Vec<String>,
    pub rows: Vec<Node>,
    pub count_hint: Option<usize>,
}
```

**Fields**:
- `type_name`: The struct type name
- `schema`: Column names from the struct definition
- `rows`: Data rows as Node instances
- `count_hint`: Optional count hint for LLM comprehension

---

## Advanced Modules

### `hedl::c14n`

Canonicalization with custom configuration.

```rust
use hedl::c14n::{canonicalize_with_config, CanonicalConfig, QuotingStrategy};

let config = CanonicalConfig::new()
    .with_quoting(QuotingStrategy::Minimal)
    .with_ditto(true)
    .with_sort_keys(true);

let canonical = canonicalize_with_config(&doc, &config)?;
```

---

### `hedl::json`

JSON conversion with configuration.

```rust
use hedl::json::{to_json_value, ToJsonConfig};

let config = ToJsonConfig::default();
let json_value = to_json_value(&doc, &config).map_err(|e| format!("{}", e))?;
```

---

### `hedl::lint`

Linting with custom rules.

```rust
use hedl::lint::{lint_with_config, LintConfig};

let mut config = LintConfig::default();
config.enable_rule("unused-alias");
config.set_rule_error("duplicate-id");

let diagnostics = lint_with_config(&doc, config);
```

---

### `hedl::lex`

Low-level lexical utilities.

```rust
use hedl::lex::{parse_reference, is_valid_id_token, is_valid_type_name, scan_regions};

// Parse reference
let ref_token = parse_reference("@User:alice")?;

// Validate tokens
let is_valid = is_valid_id_token("alice_123");

// Scan document regions for IDE integration
let regions = scan_regions(hedl_text);
```

### `hedl::tensor`

Tensor literal parsing.

```rust
use hedl::tensor::parse_tensor;

let tensor = parse_tensor("[1, 2, 3]")?;
```

### `hedl::csv`

CSV row parsing.

```rust
use hedl::csv::parse_csv_row;

let fields = parse_csv_row("alice, Alice Smith, alice@example.com")?;
```

---

## Feature-Gated Modules

### YAML Conversion (`feature = "yaml"`)

```rust
use hedl::yaml::{to_yaml, from_yaml, ToYamlConfig, FromYamlConfig};

let config_to = ToYamlConfig::default();
let yaml = to_yaml(&doc, &config_to)?;
let config_from = FromYamlConfig::default();
let doc = from_yaml(&yaml, &config_from)?;
```

---

### XML Conversion (`feature = "xml"`)

```rust
use hedl::xml::{to_xml, from_xml, ToXmlConfig, FromXmlConfig};

let config_to = ToXmlConfig::default();
let xml = to_xml(&doc, &config_to)?;
let config_from = FromXmlConfig::default();
let doc = from_xml(&xml, &config_from)?;
```

---

### CSV File Conversion (`feature = "csv"`)

```rust
use hedl::csv_file::{to_csv, from_csv, ToCsvConfig, FromCsvConfig};

let config_to = ToCsvConfig::default();
let csv = to_csv(&doc, &config_to)?;
let config_from = FromCsvConfig::default();
let doc = from_csv(&csv, &config_from)?;
```

---

### Parquet Conversion (`feature = "parquet"`)

```rust
use hedl::parquet::{to_parquet_bytes, from_parquet_bytes, ToParquetConfig};

let config = ToParquetConfig::default();
let bytes = to_parquet_bytes(&doc, &config)?;
let doc = from_parquet_bytes(&bytes)?;
```

---

### Neo4j/Cypher Export (`feature = "neo4j"`)

```rust
use hedl::neo4j::{to_cypher, ToCypherConfig};

let config = ToCypherConfig::default();
let cypher = to_cypher(&doc, &config)?;
// Returns Cypher statements for Neo4j graph database
```

---

### TOON Conversion (`feature = "toon"`)

```rust
use hedl::toon::{hedl_to_toon, to_toon, ToToonConfig, Delimiter};

// Simple conversion
let toon = hedl_to_toon(&doc)?;

// Custom configuration
let config = ToToonConfig::new()
    .with_indent(4)
    .with_delimiter(Delimiter::Tab);
let toon = to_toon(&doc, &config)?;
```

---

## Error Handling

### `HedlError`

Main error type for all HEDL operations.

```rust
pub struct HedlError {
    pub kind: HedlErrorKind,
    pub message: String,
    pub line: usize,
    pub column: Option<usize>,
    pub context: Option<String>,
}
```

**Example**:
```rust
match hedl::parse(input) {
    Ok(doc) => { /* success */ }
    Err(e) => {
        eprintln!("Error: {} (kind: {:?}) at line {}",
                  e.message, e.kind, e.line);
    }
}
```

---

### `HedlErrorKind`

Error category enumeration.

```rust
pub enum HedlErrorKind {
    Syntax,      // Lexical or structural violation
    Version,     // Unsupported version
    Schema,      // Schema violation or mismatch
    Alias,       // Duplicate or invalid alias
    Shape,       // Wrong number of cells in row
    Semantic,    // Logical error
    OrphanRow,   // Child row without NEST rule
    Collision,   // Duplicate ID within type
    Reference,   // Unresolved reference
    Security,    // Security limit exceeded
    Conversion,  // Format conversion error
    IO,          // I/O error
}
```

---

## Performance Notes

### Inline Hints

Critical hot-path functions are annotated with `#[inline]`:
- `parse()`: 5-10% improvement for small documents
- `canonicalize()`: 5-10% improvement in serialization
- `to_json()`: 5-10% improvement in format conversion

---

### Memory Optimization

- **Efficient parsing**: Minimizes allocations during parsing (uses owned `String` in AST for safety)
- **Efficient tensor storage**: Flat `Vec<f64>` with shape metadata
- **Reference counting**: `Arc` for shared structures

---

### Benchmarking

Use the `hedl-bench` crate for performance testing:

```bash
cargo bench --package hedl-bench
```

---

## Thread Safety

All parsing and conversion functions are **thread-safe** and can be called concurrently from multiple threads.

**Document objects** (`Document`, `Node`, etc.) are **not thread-safe**. Use `Arc<Mutex<Document>>` for shared access or `Send` for transfer between threads.

```rust
use std::sync::{Arc, Mutex};

let doc = Arc::new(Mutex::new(hedl::parse(input)?));

// Clone for thread
let doc_clone = Arc::clone(&doc);
std::thread::spawn(move || {
    let d = doc_clone.lock().unwrap();
    println!("Version: {}.{}", d.version.0, d.version.1);
});
```

---

## Constants

```rust
/// HEDL format version supported by this library
pub const SUPPORTED_VERSION: (u32, u32) = (1, 0);

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

---

## Best Practices

### 1. Use `parse_lenient` for User Input

```rust
// For user-provided data with potentially invalid references
let doc = hedl::parse_lenient(user_input)?;
```

### 2. Validate Before Canonicalization

```rust
hedl::validate(input)?;
let doc = hedl::parse(input)?;
let canonical = hedl::canonicalize(&doc)?;
```

### 3. Configure Limits for Untrusted Input

```rust
use hedl::{parse_with_limits, ParseOptions, Limits};

let options = ParseOptions {
    strict_refs: true,
    limits: Limits {
        max_indent_depth: 50,
        max_total_keys: 100_000,
        ..Limits::default()
    },
};

let doc = parse_with_limits(untrusted_input.as_bytes(), options)?;
```

### 4. Use Feature Flags to Minimize Dependencies

```toml
[dependencies]
hedl = { version = "1.0", default-features = false }
```

---

## Examples

See the [examples directory](examples.md) for comprehensive code samples.

Quick examples:
- [Quick Start](../../crates/hedl/examples/quick_start.rs)
- [Advanced Features](../../crates/hedl/examples/advanced_features.rs)

---

**Next**: [FFI/C API Reference](ffi-api.md)
