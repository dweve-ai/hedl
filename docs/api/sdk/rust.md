# Rust SDK Documentation

Complete SDK documentation for using HEDL in Rust applications.

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
hedl = "1.2"

# With all features
hedl = { version = "1.2", features = ["all-formats"] }

# Selective features
hedl = { version = "1.2", features = ["yaml", "xml", "csv", "parquet", "neo4j"] }
```

## Quick Start

```rust
use hedl::{parse, to_json, canonicalize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse HEDL
    let doc = parse("%VERSION: 1.0\n---\nkey: value")?;

    // Convert to JSON
    let json = to_json(&doc)?;
    println!("JSON: {}", json);

    // Canonicalize
    let canonical = canonicalize(&doc)?;
    println!("Canonical: {}", canonical);

    Ok(())
}
```

## Core Modules

### hedl (root)
- `parse()` - Parse HEDL documents
- `parse_lenient()` - Parse with lenient reference handling
- `to_json()` - Convert to JSON
- `from_json()` - Convert from JSON to HEDL
- `canonicalize()` - Canonical serialization
- `validate()` - Validate HEDL syntax
- `lint()` - Lint for best practices
- `lint_with_config()` - Lint with custom configuration

#### Constants
- `SUPPORTED_VERSION: (u32, u32)` - HEDL format version (1, 0)
- `VERSION: &str` - Library version from Cargo.toml

### hedl::lex
- Lexical analysis utilities
- Token validation
- Reference parsing
- CSV row parsing
- Tensor literal parsing

### hedl::c14n
- Canonicalization
- Deterministic formatting
- Custom configuration

### hedl::json
- JSON conversion
- Schema generation
- Bidirectional conversion

### hedl::yaml (feature = "yaml")
- YAML conversion
- YAML parsing
- YAML serialization

### hedl::xml (feature = "xml")
- XML conversion
- Schema validation
- XML parsing/serialization

### hedl::csv_file (feature = "csv")
- CSV file import/export
- Schema inference
- Custom delimiters

### hedl::parquet (feature = "parquet")
- Parquet export/import
- Efficient binary format
- Schema preservation

### hedl::neo4j (feature = "neo4j")
- Cypher generation
- Graph import/export
- Neo4j integration

## API Reference

See [Rust API Reference](../rust-api.md) for complete API documentation.

### Core Types

#### Value Type Checking

```rust
use hedl::Value;

let value = Value::Int(42);

// Type checking
if value.is_null() {
    println!("Value is null");
}
if value.is_reference() {
    println!("Value is a reference");
}

// Type extraction with Option<T>
let int_val: Option<i64> = value.as_int();
let float_val: Option<f64> = value.as_float();
let bool_val: Option<bool> = value.as_bool();
let str_val: Option<&str> = value.as_str();
let ref_val: Option<&Reference> = value.as_reference();

// Safe extraction
match value.as_int() {
    Some(n) => println!("Integer: {}", n),
    None => println!("Not an integer"),
}
```

#### Node Helper Methods

```rust
use hedl::{parse, Item};

let doc = parse(r#"
%VERSION: 1.0
%STRUCT: User: [id,name,email]
---
users: @User
  | alice, Alice, alice@example.com
"#)?;

for (key, item) in &doc.root {
    if let Item::List(matrix_list) = item {
        for node in &matrix_list.rows {
            // Get field by index (0-based, after the ID column)
            if let Some(name) = node.get_field(0) {
                println!("Name: {}", name);
            }
            if let Some(email) = node.get_field(1) {
                println!("Email: {}", email);
            }
        }
    }
}
```

### Builder Patterns

#### ParseOptions Builder

```rust
use hedl::{parse_with_limits, ParseOptions, ReferenceMode};

// Build custom parse options
let options = ParseOptions::builder()
    .max_depth(100)
    .max_array_length(5000)
    .max_file_size(10 * 1024 * 1024)  // 10MB
    .reference_mode(ReferenceMode::Lenient)
    .build();

let doc = parse_with_limits(input.as_bytes(), options)?;
```

#### CanonicalConfig Builder

```rust
use hedl::c14n::{CanonicalConfig, QuotingStrategy, canonicalize_with_config};

// Build custom canonicalization config
let config = CanonicalConfig::builder()
    .use_ditto(true)
    .sort_keys(true)
    .quoting(QuotingStrategy::Minimal)
    .inline_schemas(false)
    .build();

let canonical = canonicalize_with_config(&doc, &config)?;
```

#### LintConfig

```rust
use hedl::lint::{lint_with_config, LintConfig, Severity};

// Configure linting behavior
let mut config = LintConfig::default();
config.min_severity = Severity::Warning;
config.max_diagnostics = 100;
config.disable_rule("duplicate-keys");
config.escalate_rule("missing-version", Severity::Error);

let diagnostics = lint_with_config(&doc, config);
for diagnostic in diagnostics {
    println!("{}: {} ({})", diagnostic.severity, diagnostic.message, diagnostic.rule_id);
}
```

### Bidirectional JSON Conversion

```rust
use hedl::{from_json, to_json};

// JSON to HEDL
let json = r#"{"users": [{"id": "alice", "name": "Alice"}]}"#;
let doc = from_json(json)?;

// HEDL to JSON
let json_output = to_json(&doc)?;
println!("{}", json_output);

// With custom configuration
use hedl::json::{ToJsonConfig, FromJsonConfig};

let to_config = ToJsonConfig::default();
let json = hedl::json::to_json(&doc, &to_config)?;

let from_config = FromJsonConfig::default();
let doc = hedl::json::from_json(json_str, &from_config)?;
```

## Common Patterns

### Error Handling

```rust
use hedl::{parse, HedlError, HedlErrorKind};

match parse(input) {
    Ok(doc) => {
        // Process document
    }
    Err(e) => match e.kind {
        HedlErrorKind::Syntax => {
            eprintln!("Syntax error at line {}: {}", e.line, e.message);
        }
        HedlErrorKind::Reference => {
            // Try lenient parsing
            let doc = hedl::parse_lenient(input)?;
        }
        _ => {
            return Err(e.into());
        }
    }
}
```

### Type-Safe Extraction

```rust
use hedl::{Document, Value, Item};

fn extract_users(doc: &Document) -> Vec<User> {
    let mut users = Vec::new();

    for (key, item) in &doc.root {
        if let Item::List(matrix_list) = item {
            if key == "users" {
                for node in &matrix_list.rows {
                    // Access fields by index (aligned with schema columns)
                    // Schema: [id, name, email] -> fields[0]=name, fields[1]=email
                    // Note: node.id contains the first column value
                    if let (Some(name), Some(email)) = (
                        node.fields.get(0).and_then(|v| v.as_str()),
                        node.fields.get(1).and_then(|v| v.as_str()),
                    ) {
                        users.push(User {
                            id: node.id.clone(),
                            name: name.to_string(),
                            email: email.to_string(),
                        });
                    }
                }
            }
        }
    }

    users
}
```

### Custom Serialization

```rust
use hedl::c14n::{CanonicalConfig, QuotingStrategy, canonicalize_with_config};

// Using builder pattern
let config = CanonicalConfig::builder()
    .quoting(QuotingStrategy::Minimal)
    .use_ditto(true)
    .sort_keys(true)
    .build();

let canonical = canonicalize_with_config(&doc, &config)?;
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use hedl::{parse, to_json};

    #[test]
    fn test_roundtrip() {
        let input = "%VERSION: 1.0\n---\nkey: value";
        let doc = parse(input).unwrap();
        let json = to_json(&doc).unwrap();
        assert!(json.contains("\"key\""));
    }
}
```

## Performance Tips

- Use `parse_with_limits()` to set resource constraints
- Reuse parser state for batch processing
- Use `rayon` for parallel document processing
- Profile with `cargo bench` and `cargo flamegraph`

## Examples

See [Rust Quickstart Tutorial](../tutorials/01-rust-quickstart.md) and [Examples](../examples.md).

## Cargo Features

- `all-formats`: All format conversion features enabled
- `yaml`: YAML conversion support
- `xml`: XML conversion support
- `csv`: CSV file support
- `parquet`: Parquet format support
- `neo4j`: Neo4j/Cypher support
- `toon`: TOON format support

## See Also

- [Rust API Reference](../rust-api.md)
- [Rust Best Practices](../guides/rust-best-practices.md)
- [Core Types](../reference/core-types.md)
- [docs.rs/hedl](https://docs.rs/hedl)
