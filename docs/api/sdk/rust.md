# Rust SDK Documentation

Complete SDK documentation for using HEDL in Rust applications.

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
hedl = "1.0"

# With all features
hedl = { version = "1.0", features = ["all-formats"] }

# Selective features
hedl = { version = "1.0", features = ["yaml", "xml", "csv", "parquet", "neo4j"] }
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
- `to_json()` - Convert to JSON
- `canonicalize()` - Canonical serialization
- `validate()` - Validate HEDL syntax
- `lint()` - Lint for best practices

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
                for row in &matrix_list.rows {
                    if let Some(user) = User::from_row(row) {
                        users.push(user);
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

let config = CanonicalConfig::new()
    .with_quoting(QuotingStrategy::Minimal)
    .with_ditto(true)
    .with_sort_keys(true);

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

- `full`: All features enabled
- `yaml`: YAML conversion support
- `xml`: XML conversion support
- `csv`: CSV file support
- `parquet`: Parquet format support
- `neo4j`: Neo4j/Cypher support

## See Also

- [Rust API Reference](../rust-api.md)
- [Rust Best Practices](../guides/rust-best-practices.md)
- [Core Types](../reference/core-types.md)
- [docs.rs/hedl](https://docs.rs/hedl)
