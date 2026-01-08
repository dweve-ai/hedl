# Tutorial 3: Adding Format Support

Learn how to add a new format converter to HEDL by implementing TOML support.

## Overview

In this tutorial, you'll create a complete format converter crate (`hedl-toml`) that converts between HEDL and TOML formats. This teaches you:

- How to structure a converter crate
- Bidirectional format conversion patterns
- Integration with the HEDL workspace
- Comprehensive testing strategies
- Performance benchmarking

**Time**: ~60 minutes

## Prerequisites

- Completed [Tutorial 2: Adding Your First Feature](02-first-feature.md)
- Understanding of TOML format basics
- Familiarity with `serde` (Rust serialization library)

## The Feature: TOML Converter

We'll create `hedl-toml` with:
- `to_toml(&doc)` - Convert HEDL to TOML
- `from_toml(text)` - Convert TOML to HEDL
- Configuration options for conversion behavior
- Full test coverage

### Example Usage

```rust
use hedl_toml::{to_toml, from_toml};
use hedl_core::parse;

let hedl = parse(b"%VERSION: 1.0\n---\nserver:\n  host: localhost\n  port: 8080")?;
let toml = to_toml(&hedl)?;
// [server]
// host = "localhost"
// port = 8080

let back = from_toml(&toml)?;
assert_eq!(hedl, back); // Round-trip works!
```

## Step 1: Study Existing Converters

### Examine hedl-json Structure

```bash
cd hedl
tree crates/hedl-json/ -L 2
```

Structure:
```
crates/hedl-json/
├── Cargo.toml          # Dependencies and metadata
├── src/
│   ├── lib.rs          # Public API and docs
│   ├── from_json.rs    # JSON → HEDL
│   ├── to_json.rs      # HEDL → JSON
│   ├── config.rs       # Configuration types
│   └── error.rs        # Error types
├── tests/
│   ├── conversion_tests.rs
│   └── property_tests.rs
└── examples/
    └── basic_usage.rs
```

### Read the JSON Converter

```bash
cat crates/hedl-json/src/lib.rs
cat crates/hedl-json/src/to_json.rs
```

Note the patterns:
1. Configuration via builder pattern
2. Error handling with custom error types
3. Recursive traversal of document tree
4. Schema inference for arrays

## Step 2: Create the Crate

```bash
# Create directory
mkdir -p crates/hedl-toml/src

# Create Cargo.toml
cat > crates/hedl-toml/Cargo.toml << 'EOF'
[package]
name = "hedl-toml"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
description = "TOML conversion for HEDL format"

[dependencies]
hedl-core.workspace = true
thiserror.workspace = true
toml = "0.8"
serde_json = "1.0"

[dev-dependencies]
hedl-c14n.workspace = true

[[example]]
name = "basic_usage"
path = "examples/basic_usage.rs"
EOF
```

### Add to Workspace

Edit `/home/marc/dev/projects/hedl/Cargo.toml`:

```toml
[workspace]
members = [
    # ... existing members ...
    "crates/hedl-toml",  # Add this line
]
```

## Step 3: Define Error Types

Create `crates/hedl-toml/src/error.rs`:

```rust
use std::fmt;

/// Errors that can occur during TOML conversion
#[derive(Debug, thiserror::Error)]
pub enum TomlError {
    /// Error parsing TOML text
    #[error("TOML parse error: {0}")]
    Parse(String),

    /// Error serializing to TOML
    #[error("TOML serialization error: {0}")]
    Serialize(String),

    /// HEDL parsing error
    #[error("HEDL error: {0}")]
    Hedl(#[from] hedl_core::HedlError),

    /// Unsupported TOML feature
    #[error("Unsupported TOML feature: {message}")]
    Unsupported { message: String },

    /// Type conversion error
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
}

pub type Result<T> = std::result::Result<T, TomlError>;
```

## Step 4: Implement HEDL → TOML

Create `crates/hedl-toml/src/to_toml.rs`:

```rust
use hedl_core::{Document, Node, Value, Item};
use toml::Table;
use crate::error::{Result, TomlError};

/// Convert HEDL document to TOML string
///
/// # Example
///
/// ```
/// use hedl_core::parse;
/// use hedl_toml::to_toml;
///
/// let doc = parse(b"%VERSION: 1.0\n---\nserver:\n  host: localhost\n  port: 8080").unwrap();
/// let toml = to_toml(&doc).unwrap();
/// assert!(toml.contains("[server]"));
/// ```
pub fn to_toml(doc: &Document) -> Result<String> {
    let table = node_to_table(&doc.root)?;
    toml::to_string_pretty(&table)
        .map_err(|e| TomlError::Serialize(e.to_string()))
}

/// Convert HEDL document root to a TOML table
fn doc_to_table(doc: &Document) -> Result<Table> {
    let mut table = Table::new();

    // Convert root items
    for (key, item) in &doc.root {
        table.insert(key.clone(), item_to_toml(item)?);
    }

    Ok(table)
}

/// Convert HEDL Item to TOML value
fn item_to_toml(item: &Item) -> Result<toml::Value> {
    Ok(match item {
        Item::Scalar(value) => value_to_toml(value)?,
        Item::Object(map) => {
            let mut table = Table::new();
            for (k, v) in map {
                table.insert(k.clone(), item_to_toml(v)?);
            }
            toml::Value::Table(table)
        }
        Item::List(matrix) => {
            // Matrix lists convert to arrays of tables
            let arr: Vec<toml::Value> = matrix.rows.iter()
                .map(|node| {
                    let mut table = Table::new();
                    table.insert("id".to_string(), toml::Value::String(node.id.clone()));
                    for (i, value) in node.fields.iter().enumerate() {
                        let key = matrix.schema.get(i).map(|s| s.as_str()).unwrap_or("field");
                        table.insert(key.to_string(), value_to_toml(value)?);
                    }
                    Ok(toml::Value::Table(table))
                })
                .collect::<Result<_>>()?;
            toml::Value::Array(arr)
        }
    })
}

/// Convert HEDL value to TOML value
fn value_to_toml(value: &Value) -> Result<toml::Value> {
    Ok(match value {
        Value::String(s) => toml::Value::String(s.clone()),
        Value::Int(i) => toml::Value::Integer(*i),
        Value::Float(f) => toml::Value::Float(*f),
        Value::Bool(b) => toml::Value::Boolean(*b),
        Value::Null => {
            // TOML doesn't have null, use empty string
            toml::Value::String(String::new())
        }
        Value::Reference(r) => {
            // Convert reference to string ID
            toml::Value::String(r.id.clone())
        }
        Value::Tensor(t) => {
            // Convert tensor to array
            let values: Vec<toml::Value> = t
                .values
                .iter()
                .map(|v| value_to_toml(v))
                .collect::<Result<_>>()?;
            toml::Value::Array(values)
        }
        Value::Expression(_) => {
            return Err(TomlError::Unsupported {
                message: "Expressions not supported in TOML".to_string(),
            });
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::parse;

    #[test]
    fn test_simple_conversion() {
        let doc = parse(b"%VERSION: 1.0\n---\nname: Alice\nage: 30").unwrap();
        let toml = to_toml(&doc).unwrap();
        assert!(toml.contains("name = \"Alice\""));
        assert!(toml.contains("age = 30"));
    }

    #[test]
    fn test_nested_tables() {
        let doc = parse(b"%VERSION: 1.0\n---\nserver:\n  host: localhost\n  port: 8080").unwrap();
        let toml = to_toml(&doc).unwrap();
        assert!(toml.contains("[server]"));
        assert!(toml.contains("host = \"localhost\""));
    }
}
```

## Step 5: Implement TOML → HEDL

Create `crates/hedl-toml/src/from_toml.rs`:

```rust
use hedl_core::{Document, Value, Item};
use toml::Table;
use std::collections::BTreeMap;
use crate::error::{Result, TomlError};

/// Convert TOML string to HEDL document
///
/// # Example
///
/// ```
/// use hedl_toml::from_toml;
///
/// let toml = "[server]\nhost = \"localhost\"\nport = 8080";
/// let doc = from_toml(toml).unwrap();
/// assert_eq!(doc.version, (1, 0));
/// ```
pub fn from_toml(toml_text: &str) -> Result<Document> {
    let table: Table = toml::from_str(toml_text)
        .map_err(|e| TomlError::Parse(e.to_string()))?;

    table_to_doc(table)
}

fn table_to_doc(table: Table) -> Result<Document> {
    let mut root = BTreeMap::new();

    for (key, value) in table {
        root.insert(key, toml_to_item(value)?);
    }

    Ok(Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    })
}

fn toml_to_item(value: toml::Value) -> Result<Item> {
    Ok(match value {
        toml::Value::Table(table) => {
            let mut map = BTreeMap::new();
            for (k, v) in table {
                map.insert(k, toml_to_item(v)?);
            }
            Item::Object(map)
        }
        _ => Item::Scalar(toml_to_value(value)?),
    })
}

fn toml_to_value(value: toml::Value) -> Result<Value> {
    Ok(match value {
        toml::Value::String(s) => Value::String(s),
        toml::Value::Integer(i) => Value::Int(i),
        toml::Value::Float(f) => Value::Float(f),
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Array(arr) => {
            // For now, convert arrays to string representation
            // A more complete implementation would use Tensor type
            let json_str = serde_json::to_string(&arr)
                .map_err(|e| TomlError::Serialize(e.to_string()))?;
            Value::String(json_str)
        }
        toml::Value::Table(_) => {
            return Err(TomlError::Unsupported {
                message: "Nested tables should be handled separately".to_string(),
            });
        }
        toml::Value::Datetime(dt) => {
            Value::String(dt.to_string())
        }
    })
}
```

## Step 6: Create Public API

Create `crates/hedl-toml/src/lib.rs`:

```rust
//! TOML conversion for HEDL documents
//!
//! Provides bidirectional conversion between HEDL and TOML formats.
//!
//! # Examples
//!
//! ```
//! use hedl_core::parse;
//! use hedl_toml::{to_toml, from_toml};
//!
//! // HEDL to TOML
//! let doc = parse(b"%VERSION: 1.0\n---\nserver:\n  host: localhost\n  port: 8080").unwrap();
//! let toml = to_toml(&doc).unwrap();
//!
//! // TOML to HEDL
//! let back = from_toml(&toml).unwrap();
//! ```

mod error;
mod from_toml;
mod to_toml;

pub use error::{TomlError, Result};
pub use from_toml::from_toml;
pub use to_toml::to_toml;
```

## Step 7: Write Comprehensive Tests

Create `crates/hedl-toml/tests/conversion_tests.rs`:

```rust
use hedl_core::parse;
use hedl_toml::{to_toml, from_toml};

#[test]
fn test_round_trip_simple() {
    let hedl = b"%VERSION: 1.0\n---\nname: Alice\nage: 30\nactive: true";
    let doc = parse(hedl).unwrap();
    let toml = to_toml(&doc).unwrap();
    let back = from_toml(&toml).unwrap();

    // Verify structure preserved
    assert!(back.root.contains_key("name"));
    assert!(back.root.contains_key("age"));
    assert!(back.root.contains_key("active"));
}

#[test]
fn test_nested_structures() {
    let hedl = b"%VERSION: 1.0\n---\ndatabase:\n  host: localhost\n  port: 5432\n  credentials:\n    user: admin\n    password: secret\n";
    let doc = parse(hedl).unwrap();
    let toml = to_toml(&doc).unwrap();

    assert!(toml.contains("[database]"));
    assert!(toml.contains("[database.credentials]"));
}

#[test]
fn test_arrays() {
    let hedl = b"%VERSION: 1.0\n---\nports: [8080, 8081, 8082]\n";
    let doc = parse(hedl).unwrap();
    let toml = to_toml(&doc).unwrap();

    assert!(toml.contains("ports = [8080, 8081, 8082]"));
}

#[test]
fn test_types() {
    let hedl = b"%VERSION: 1.0\n---\nstring: hello\nint: 42\nfloat: 3.14\nbool: true\n";
    let doc = parse(hedl).unwrap();
    let toml = to_toml(&doc).unwrap();

    let back = from_toml(&toml).unwrap();
    // Verify types preserved
    use hedl_core::{Item, Value};
    assert!(matches!(back.root.get("int"), Some(Item::Scalar(Value::Int(42)))));
}
```

Create `crates/hedl-toml/tests/property_tests.rs`:

```rust
use hedl_toml::{to_toml, from_toml};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_round_trip_doesnt_panic(s in ".*") {
        // Should never panic, even with invalid input
        let _ = from_toml(&s);
    }

    #[test]
    fn test_valid_toml_round_trips(
        name in "[a-z]+",
        value in 1..100i64
    ) {
        let toml = format!("{} = {}", name, value);
        if let Ok(doc) = from_toml(&toml) {
            let result = to_toml(&doc);
            assert!(result.is_ok());
        }
    }
}
```

## Step 8: Add Examples

Create `crates/hedl-toml/examples/basic_usage.rs`:

```rust
use hedl_core::parse;
use hedl_toml::{to_toml, from_toml};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HEDL-TOML Conversion Example ===\n");

    // Example configuration in HEDL
    let hedl = r#"%VERSION: 1.0
---
server:
  host: localhost
  port: 8080
  max_connections: 100

database:
  url: postgresql://localhost/mydb
  pool_size: 20
"#;

    println!("HEDL Input:");
    println!("{}", hedl);

    // Parse HEDL
    let doc = parse(hedl.as_bytes())?;

    // Convert to TOML
    let toml = to_toml(&doc)?;
    println!("\nTOML Output:");
    println!("{}", toml);

    // Convert back to HEDL
    let doc2 = from_toml(&toml)?;
    let toml2 = to_toml(&doc2)?;

    println!("\nRound-trip successful: {}", toml == toml2);

    Ok(())
}
```

Run it:
```bash
cargo run --example basic_usage -p hedl-toml
```

## Step 9: Add Benchmarks

Create `crates/hedl-bench/benches/formats/toml.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hedl_core::parse;
use hedl_toml::{to_toml, from_toml};

fn benchmark_to_toml(c: &mut Criterion) {
    let mut group = c.benchmark_group("toml_conversion");

    let small = parse(b"%VERSION: 1.0\n---\nname: Alice\nage: 30").unwrap();
    let medium = parse(b"%VERSION: 1.0\n---\nserver:\n  host: localhost\n  database:\n    url: postgres://localhost".as_bytes()).unwrap();

    group.bench_with_input(BenchmarkId::new("to_toml", "small"), &small, |b, doc| {
        b.iter(|| to_toml(black_box(doc)))
    });

    group.bench_with_input(BenchmarkId::new("to_toml", "medium"), &medium, |b, doc| {
        b.iter(|| to_toml(black_box(doc)))
    });

    group.finish();
}

criterion_group!(benches, benchmark_to_toml);
criterion_main!(benches);
```

## Step 10: Build and Test

```bash
# Build the new crate
cargo build -p hedl-toml

# Run all tests
cargo test -p hedl-toml

# Run with verbose output
cargo test -p hedl-toml -- --nocapture

# Check for warnings
cargo clippy -p hedl-toml -- -D warnings

# Format code
cargo fmt -p hedl-toml

# Build documentation
cargo doc -p hedl-toml --open
```

## Step 11: Integrate with Main Crate

Edit `crates/hedl/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
hedl-toml = { workspace = true, optional = true }

[features]
default = []
toml = ["hedl-toml"]
all = ["json", "yaml", "xml", "csv", "toml"]  # Add toml
```

Edit `crates/hedl/src/lib.rs`:

```rust
#[cfg(feature = "toml")]
pub use hedl_toml as toml;
```

Now users can:
```rust
use hedl::toml::{to_toml, from_toml};
```

## Step 12: Update Documentation

Add to `docs/developer/module-guide.md`:

```markdown
### hedl-toml

**Path**: `crates/hedl-toml/`
**Purpose**: TOML ↔ HEDL conversion
**Dependencies**: hedl-core, toml

#### Features

- Bidirectional TOML conversion
- Nested table support
- Array handling
- Type preservation
- TOML datetime support

#### Example

\`\`\`rust
use hedl_toml::{to_toml, from_toml};

let doc = parse("server:\n  host: localhost")?;
let toml = to_toml(&doc)?;
\`\`\`
```

## Step 13: Commit and Push

```bash
git add crates/hedl-toml
git add crates/hedl/Cargo.toml
git add crates/hedl/src/lib.rs
git add docs/developer/module-guide.md

git commit -m "feat(toml): Add TOML format converter

- Create hedl-toml crate for TOML conversion
- Implement bidirectional conversion (to_toml, from_toml)
- Handle nested tables and arrays
- Add comprehensive unit and property tests
- Add example and benchmarks
- Integrate with main hedl crate via feature flag"

git push origin add-toml-support
```

## Best Practices Learned

### 1. Follow Existing Patterns

Study similar crates (`hedl-json`, `hedl-yaml`) and match their structure.

### 2. Comprehensive Testing

- Unit tests for each function
- Integration tests for full conversions
- Property tests for fuzzing
- Round-trip tests for correctness

### 3. Error Handling

- Custom error types with `thiserror`
- Meaningful error messages
- Proper error propagation

### 4. Documentation

- Module-level docs in `lib.rs`
- Function docs with examples
- Examples directory for users
- Integration with main docs

### 5. Performance

- Add benchmarks early
- Profile before optimizing
- Compare with reference implementations

## Common Challenges

### Challenge 1: Type Mapping

HEDL and TOML have different type systems.

**Solution**: Document the mapping explicitly:
```rust
// HEDL Null → TOML empty string
// HEDL Reference → TOML string (ID only)
// HEDL Expression → Error (unsupported)
```

### Challenge 2: Round-Trip Fidelity

Not all conversions preserve perfect round-trip.

**Solution**:
- Document known limitations
- Add tests showing acceptable changes
- Provide configuration for strict mode

### Challenge 3: Integration Testing

Need to test with other crates.

**Solution**:
- Use `dev-dependencies` for test-only crates
- Create integration test directory
- Test realistic workflows

## Next Steps

You now know how to:
- Create a new crate in the workspace
- Implement format conversion
- Write comprehensive tests
- Integrate with the main library

Try these challenges:
1. Add configuration options (e.g., `ToTomlConfig`)
2. Optimize for large documents
3. Add streaming support
4. Implement another format (e.g., MessagePack)

**Next Tutorial**: [Writing Effective Tests](04-writing-tests.md)

## Additional Resources

- [TOML Specification](https://toml.io/)
- [toml-rs Documentation](https://docs.rs/toml/)
- [hedl-json API](../../api/sdk/rust.md)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

---

**Congratulations!** You've created a complete format converter crate.
