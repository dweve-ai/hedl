# Rust API Quickstart

This tutorial will guide you through using HEDL in Rust, from installation to building a complete application.

## Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- Basic Rust knowledge

## Installation

Add HEDL to your `Cargo.toml`:

```toml
[dependencies]
hedl = "1.0"

# Optional features
hedl = { version = "1.0", features = ["yaml", "xml", "csv", "parquet", "neo4j"] }
```

## Your First HEDL Program

Create a new Rust project:

```bash
cargo new hedl-quickstart
cd hedl-quickstart
```

Add this code to `src/main.rs`:

```rust
use hedl::{parse, to_json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a HEDL document
    let hedl_text = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
"#;

    // Parse the document
    let doc = parse(hedl_text)?;
    println!("Parsed {} items", doc.root.len());

    // Convert to JSON (uses default config)
    let json = to_json(&doc)?;
    println!("JSON:\n{}", json);

    Ok(())
}
```

Run it:

```bash
cargo run
```

Output:
```
Parsed 1 items
JSON:
{
  "users": [
    {
      "id": "alice",
      "name": "Alice Smith",
      "email": "alice@example.com"
    },
    {
      "id": "bob",
      "name": "Bob Jones",
      "email": "bob@example.com"
    }
  ]
}
```

## Core Operations

### Parsing

```rust
use hedl::parse;

let doc = parse(hedl_text)?;

// Access document metadata
println!("Version: {}.{}", doc.version.0, doc.version.1);
println!("Structs: {:?}", doc.structs.keys());
println!("Root items: {}", doc.root.len());
```

### With Parse Options

```rust
use hedl::{parse_with_limits, ParseOptions, Limits};

let options = ParseOptions {
    strict_refs: true,
    limits: Limits {
        max_indent_depth: 20,
        max_total_keys: 5000,
        ..Limits::default()
    },
};

let doc = parse_with_limits(hedl_text.as_bytes(), options)?;
```

### Validation

```rust
use hedl::{validate, lint};

// Quick validation
if let Err(e) = validate(hedl_text) {
    eprintln!("Validation error: {}", e);
}

// Detailed linting
let doc = parse(hedl_text)?;
let diagnostics = lint(&doc);

for d in diagnostics {
    if let Some(line) = d.line() {
        println!("[{:?}] Line {}: {}", d.severity(), line, d.message());
    } else {
        println!("[{:?}] {}", d.severity(), d.message());
    }
}
```

### Serialization

```rust
use hedl::{parse, to_json, canonicalize};

let doc = parse(hedl_text)?;

// To JSON
let json = to_json(&doc)?;

// To canonical HEDL
let canonical = canonicalize(&doc)?;
```

## Working with Data

### Accessing Values

```rust
use hedl::{parse, Item, Value};

let doc = parse(hedl_text)?;

// Iterate over root items
for (key, item) in &doc.root {
    match item {
        Item::Scalar(value) => {
            println!("Key: {}, Value: {:?}", key, value);
        }
        Item::List(matrix) => {
            println!("Matrix: {} (@{}), {} rows", key, matrix.type_name, matrix.rows.len());
        }
        Item::Object(map) => {
            println!("Nested: {}, {} children", key, map.len());
        }
    }
}
```

### Building Documents Programmatically

```rust
use hedl::{Document, Item, Value};
use std::collections::BTreeMap;

let mut doc = Document {
    version: (1, 0),
    structs: BTreeMap::new(),
    aliases: BTreeMap::new(),
    nests: BTreeMap::new(),
    root: BTreeMap::new(),
};

// Add a simple scalar
doc.root.insert("name".to_string(), Item::Scalar(Value::String("Test".to_string())));

// Add a nested object
let mut config = BTreeMap::new();
config.insert("debug".to_string(), Item::Scalar(Value::Bool(true)));
doc.root.insert("config".to_string(), Item::Object(config));

// Serialize to HEDL
let hedl_output = canonicalize(&doc)?;
println!("{}", hedl_output);
```

## Format Conversion

### JSON

```rust
use hedl::{from_json, to_json};
use hedl::json::{FromJsonConfig, ToJsonConfig};

// JSON to HEDL
let json_input = r#"{"name": "Alice", "age": 30}"#;
let doc = from_json(json_input)?;

// HEDL to JSON with config
let config = ToJsonConfig::default();
let json_output = hedl::json::to_json(&doc, &config)?;
```

### YAML (with feature)

```rust
#[cfg(feature = "yaml")]
use hedl::yaml::{from_yaml, to_yaml, FromYamlConfig, ToYamlConfig};

#[cfg(feature = "yaml")]
{
    let yaml_input = "name: Alice\nage: 30";
    let config_from = FromYamlConfig::default();
    let doc = from_yaml(yaml_input, &config_from)?;
    let config_to = ToYamlConfig::default();
    let yaml_output = to_yaml(&doc, &config_to)?;
}
```

### XML (with feature)

```rust
#[cfg(feature = "xml")]
use hedl::xml::{from_xml, to_xml, FromXmlConfig, ToXmlConfig};

#[cfg(feature = "xml")]
{
    let xml_input = "<root><name>Alice</name><age>30</age></root>";
    let config_from = FromXmlConfig::default();
    let doc = from_xml(xml_input, &config_from)?;
    let config_to = ToXmlConfig::default();
    let xml_output = to_xml(&doc, &config_to)?;
}
```

### Parquet (with feature)

```rust
#[cfg(feature = "parquet")]
use hedl::parquet::{to_parquet_bytes, from_parquet_bytes, ToParquetConfig};

#[cfg(feature = "parquet")]
{
    let doc = parse(hedl_text)?;
    let config = ToParquetConfig::default();
    let bytes = to_parquet_bytes(&doc, &config)?;

    // Write to file
    std::fs::write("output.parquet", bytes)?;

    // Read back
    let bytes = std::fs::read("output.parquet")?;
    let doc = from_parquet_bytes(&bytes)?;
}
```

## Error Handling

### Basic Error Handling

```rust
use hedl::{parse, HedlError};

match parse(hedl_text) {
    Ok(doc) => {
        println!("Success: {} items", doc.root.len());
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        eprintln!("  at line {}", e.line);
    }
}
```

### Pattern Matching on Error Kind

```rust
use hedl::{parse, parse_with_limits, ParseOptions, HedlError, HedlErrorKind};

match parse(hedl_text) {
    Ok(doc) => { /* use doc */ }
    Err(e) => {
        match e.kind {
            HedlErrorKind::Syntax => {
                eprintln!("Syntax error: {}", e);
            }
            HedlErrorKind::Reference => {
                eprintln!("Reference error: {}", e);
                // Try lenient parsing
                let options = ParseOptions {
                    strict_refs: false,
                    ..Default::default()
                };
                let doc = parse_with_limits(hedl_text.as_bytes(), options)?;
            }
            HedlErrorKind::Security => {
                eprintln!("Security limit exceeded: {}", e);
            }
            _ => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
```

### Error Handling

```rust
use hedl::{parse, HedlError};

// Parse with custom error handling
let result = parse(hedl_text).map_err(|e| {
    format!("Failed to parse user data: {}", e)
})?;

// Or chain errors
fn process_config(hedl_text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let doc = parse(hedl_text)?;
    // Process document...
    Ok(())
}
```

## Complete Example: User Management

```rust
use hedl::{parse, to_json, Document, Item, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse user database
    let hedl_text = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email, role]
%STRUCT: Permission: [resource, action]
---
users: @User
  | alice, Alice Smith, alice@example.com, admin
  | bob, Bob Jones, bob@example.com, user
  | charlie, Charlie Brown, charlie@example.com, moderator

permissions:
  alice: @Permission
    | users, write
    | posts, write
    | comments, moderate
  bob: @Permission
    | posts, read
    | comments, write
  charlie: @Permission
    | comments, moderate
"#;

    let doc = parse(hedl_text)?;

    // Extract users
    let mut users = HashMap::new();
    if let Some(Item::List(matrix)) = doc.root.get("users") {
        for node in &matrix.rows {
            if let [Value::String(id), Value::String(name), Value::String(email), Value::String(role)] = &node.fields[..] {
                users.insert(id.clone(), (name.clone(), email.clone(), role.clone()));
            }
        }
    }

    println!("Loaded {} users:", users.len());
    for (id, (name, email, role)) in &users {
        println!("  {}: {} <{}> [{}]", id, name, email, role);
    }

    // Convert to JSON for API response
    let json = to_json(&doc)?;
    println!("\nJSON representation:\n{}", json);

    Ok(())
}
```

## Performance Tips

### Pre-allocation

```rust
use hedl::{Document, Item};
use std::collections::BTreeMap;

let mut doc = Document {
    version: (1, 0),
    structs: BTreeMap::new(),
    aliases: BTreeMap::new(),
    nests: BTreeMap::new(),
    root: BTreeMap::new(),  // BTreeMap doesn't pre-allocate
};
```

### Reusing Parsers

For high-throughput scenarios, parse documents in parallel:

```rust
use hedl::parse;
use rayon::prelude::*;

let documents: Vec<String> = load_documents();

let parsed: Vec<_> = documents
    .par_iter()
    .filter_map(|text| parse(text).ok())
    .collect();
```

### Memory Limits

```rust
use hedl::{parse_with_limits, ParseOptions, Limits};

// For API responses (small documents)
let api_limits = Limits {
    max_indent_depth: 10,
    max_total_keys: 10000,
    ..Limits::default()
};

let options = ParseOptions {
    strict_refs: true,
    limits: api_limits,
};
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use hedl::{parse, to_json};

    #[test]
    fn test_parse_simple() {
        let input = "%VERSION: 1.0\n---\nkey: value";
        let doc = parse(input).unwrap();
        assert_eq!(doc.root.len(), 1);
    }

    #[test]
    fn test_json_roundtrip() {
        let input = "%VERSION: 1.0\n---\nkey: value";
        let doc = parse(input).unwrap();
        let json = to_json(&doc).unwrap();
        assert!(json.contains("\"key\""));
        assert!(json.contains("\"value\""));
    }

    #[test]
    fn test_invalid_syntax() {
        let input = "invalid";
        assert!(parse(input).is_err());
    }
}
```

## Next Steps

- **[FFI Integration](02-ffi-integration.md)** - Use HEDL from C/C++
- **[Rust Best Practices](../guides/rust-best-practices.md)** - Advanced patterns
- **[Core Types Reference](../reference/core-types.md)** - Detailed type documentation
- **[Examples](../examples.md)** - More complete examples

## Resources

- **[Rust API Reference](../rust-api.md)** - Complete Rust API documentation
- **[Error Handling Guide](../guides/error-handling.md)** - Error handling patterns
- **[GitHub Examples](https://github.com/dweve/hedl/tree/main/examples)** - Example code
