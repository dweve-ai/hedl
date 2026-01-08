# Module API Reference

Comprehensive API reference for all HEDL modules.

## hedl-core

### Parsing

```rust
pub fn parse(input: &[u8]) -> HedlResult<Document>

pub fn parse_with_limits(
    input: &[u8],
    options: ParseOptions
) -> HedlResult<Document>

pub struct ParseOptions {
    pub limits: Limits,
    pub strict_refs: bool,
}

impl Default for ParseOptions {
    fn default() -> Self { ... }
}
```

### Data Structures

```rust
pub struct Document {
    pub version: (u32, u32),
    pub aliases: BTreeMap<String, String>,
    pub structs: BTreeMap<String, Vec<String>>,
    pub nests: BTreeMap<String, String>,
    pub root: BTreeMap<String, Item>,
}

pub enum Item {
    Scalar(Value),
    Object(BTreeMap<String, Item>),
    List(MatrixList),
}

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

### Error Types

```rust
pub struct HedlError {
    pub kind: HedlErrorKind,
    pub message: String,
    pub line: usize,
    pub column: Option<usize>,
    pub context: Option<String>,
}

pub enum HedlErrorKind {
    Syntax, Version, Schema, Alias, Shape, Semantic,
    OrphanRow, Collision, Reference, Security, Conversion, IO,
}
```

## hedl-json

```rust
// Main conversion functions
pub fn to_json(doc: &Document, config: &ToJsonConfig) -> Result<String, String>;
pub fn from_json(json: &str, config: &FromJsonConfig) -> Result<Document, JsonConversionError>;

// Convenience functions with default configs
pub fn hedl_to_json(doc: &Document) -> Result<String, String>;
pub fn json_to_hedl(json: &str) -> Result<Document, String>;

// Convert to/from serde_json::Value
pub fn to_json_value(doc: &Document, config: &ToJsonConfig) -> Result<serde_json::Value, String>;
pub fn from_json_value(value: &serde_json::Value, config: &FromJsonConfig) -> Result<Document, JsonConversionError>;
pub fn from_json_value_owned(value: serde_json::Value, config: &FromJsonConfig) -> Result<Document, JsonConversionError>;
```

## For Complete API

See generated documentation:

```bash
cargo doc --all --no-deps --open
```

## Related

- [Module Guide](../module-guide.md)
- [Architecture](../architecture.md)
