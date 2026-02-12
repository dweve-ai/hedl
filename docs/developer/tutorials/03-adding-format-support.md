# Tutorial 3: Adding Format Support

Learn how to add a new format converter to HEDL by implementing CSV support.

## Overview

In this tutorial, you'll create a complete format converter crate (`hedl-csv`) that converts between HEDL and CSV formats. This teaches you:

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

## The Feature: CSV Converter

We'll create `hedl-csv` with:
- `to_csv(&doc)` - Convert HEDL to CSV
- `from_csv(text)` - Convert CSV to HEDL
- Configuration options for conversion behavior
- Full test coverage

### Example Usage

```rust
use hedl_csv::{to_csv, from_csv};
use hedl_core::parse;

let hedl = parse(b"%V:2.0\n---\nusers:\n  alice:\n    email: alice@example.com\n  bob:\n    email: bob@example.com")?;
let csv = to_csv(&hedl)?;
// name,email
// alice,alice@example.com
// bob,bob@example.com

let back = from_csv(&csv)?;
// Converts back to HEDL structure
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
cat crates/hedl-json/src/from_json.rs
```

Note the patterns:
1. Error handling with custom error types (`JsonConversionError`)
2. Configuration via builder pattern (`ToJsonConfig`, `FromJsonConfig`)
3. Recursive traversal of document tree using visitor pattern
4. Support for schema inference and array handling

## Step 2: Create the Crate

Let's examine the existing CSV crate first:

```bash
# Check if hedl-csv already exists
ls -la crates/hedl-csv/

# If it exists, examine its structure
cat crates/hedl-csv/Cargo.toml
cat crates/hedl-csv/src/lib.rs
```

Note: The `hedl-csv` crate already exists in the workspace. For this tutorial, we'll demonstrate extending it with new features or creating a similar converter for another format (like MessagePack).

### Alternative: Create Your Own Format Crate

If you want to practice creating from scratch, follow this pattern:

```bash
# Create directory
mkdir -p crates/hedl-msgpack/src

# Create Cargo.toml
cat > crates/hedl-msgpack/Cargo.toml << 'EOF'
[package]
name = "hedl-msgpack"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
description = "MessagePack conversion for HEDL format"

[dependencies]
hedl-core.workspace = true
thiserror.workspace = true
rmp-serde = "1.1"

[dev-dependencies]
hedl-c14n.workspace = true

[[example]]
name = "basic_usage"
path = "examples/basic_usage.rs"
EOF
```

### Add to Workspace

Edit `Cargo.toml` and verify the new crate is listed in `members` array.

## Step 3: Define Error Types

Create `crates/hedl-msgpack/src/error.rs`:

```rust
use std::fmt;

/// Errors that can occur during MessagePack conversion
#[derive(Debug, thiserror::Error)]
pub enum MsgPackError {
    /// Error parsing MessagePack data
    #[error("MessagePack parse error: {0}")]
    Parse(String),

    /// Error serializing to MessagePack
    #[error("MessagePack serialization error: {0}")]
    Serialize(String),

    /// HEDL parsing error
    #[error("HEDL error: {0}")]
    Hedl(#[from] hedl_core::HedlError),

    /// Unsupported MessagePack feature
    #[error("Unsupported MessagePack feature: {message}")]
    Unsupported { message: String },

    /// Type conversion error
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
}

pub type Result<T> = std::result::Result<T, MsgPackError>;
```

## Step 4: Implement HEDL → JSON (Reference Implementation)

For this tutorial, we'll examine the actual JSON converter to understand the pattern. Create your own by following this structure:

Create `crates/hedl-msgpack/src/to_msgpack.rs`:

```rust
use hedl_core::{Document, Value, Item};
use crate::error::{Result, MsgPackError};

/// Convert HEDL document to MessagePack bytes
///
/// # Example
///
/// ```
/// use hedl_core::parse;
/// use hedl_msgpack::to_msgpack;
///
/// let doc = parse(b"%V:2.0\n---\nname: Alice\nage: 30").unwrap();
/// let bytes = to_msgpack(&doc).unwrap();
/// assert!(!bytes.is_empty());
/// ```
pub fn to_msgpack(doc: &Document) -> Result<Vec<u8>> {
    // Convert document root to value
    let mut map = std::collections::BTreeMap::new();

    for (key, item) in &doc.root {
        map.insert(key.clone(), item_to_value(item)?);
    }

    // Serialize to MessagePack
    rmp_serde::to_vec(&map)
        .map_err(|e| MsgPackError::Serialize(e.to_string()))
}

/// Convert HEDL Item to a serializable value
fn item_to_value(item: &Item) -> Result<serde_json::Value> {
    Ok(match item {
        Item::Scalar(value) => value_to_json(value)?,
        Item::Object(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.clone(), item_to_value(v)?);
            }
            serde_json::Value::Object(obj)
        }
        Item::List(matrix) => {
            // Convert matrix list to array of objects
            let arr: Vec<serde_json::Value> = matrix.rows.iter()
                .map(|node| {
                    let mut obj = serde_json::Map::new();
                    obj.insert("id".to_string(), serde_json::Value::String(node.id.clone()));
                    for (i, value) in node.fields.iter().enumerate() {
                        let key = matrix.schema.get(i).map(|s| s.as_str()).unwrap_or("field");
                        obj.insert(key.to_string(), value_to_json(value)?);
                    }
                    Ok(serde_json::Value::Object(obj))
                })
                .collect::<Result<_>>()?;
            serde_json::Value::Array(arr)
        }
    })
}

/// Convert HEDL value to JSON value
fn value_to_json(value: &Value) -> Result<serde_json::Value> {
    Ok(match value {
        Value::String(s) => serde_json::Value::String(s.to_string()),
        Value::Int(i) => serde_json::json!(i),
        Value::Float(f) => serde_json::json!(f),
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Null => serde_json::Value::Null,
        Value::Reference(r) => serde_json::Value::String(r.id.to_string()),
        Value::Tensor(_) => {
            return Err(MsgPackError::Unsupported {
                message: "Tensors require special handling".to_string(),
            });
        }
        Value::Expression(_) => {
            return Err(MsgPackError::Unsupported {
                message: "Expressions not supported in MessagePack".to_string(),
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
        let doc = parse(b"%V:2.0\n---\nname: Alice\nage: 30").unwrap();
        let bytes = to_msgpack(&doc).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_nested_objects() {
        let doc = parse(b"%V:2.0\n---\nserver:\n  host: localhost\n  port: 8080").unwrap();
        let bytes = to_msgpack(&doc).unwrap();
        assert!(!bytes.is_empty());
    }
}
```

## Step 5: Implement MessagePack → HEDL

Create `crates/hedl-msgpack/src/from_msgpack.rs`:

```rust
use hedl_core::{Document, Value, Item};
use std::collections::BTreeMap;
use crate::error::{Result, MsgPackError};

/// Convert MessagePack bytes to HEDL document
///
/// # Example
///
/// ```
/// use hedl_msgpack::{to_msgpack, from_msgpack};
/// use hedl_core::parse;
///
/// let doc = parse(b"%V:2.0\n---\nname: Alice").unwrap();
/// let bytes = to_msgpack(&doc).unwrap();
/// let back = from_msgpack(&bytes).unwrap();
/// // back contains the same data as doc
/// ```
pub fn from_msgpack(data: &[u8]) -> Result<Document> {
    let value: serde_json::Value = rmp_serde::from_slice(data)
        .map_err(|e| MsgPackError::Parse(e.to_string()))?;

    json_to_doc(value)
}

fn json_to_doc(value: serde_json::Value) -> Result<Document> {
    let mut root = BTreeMap::new();

    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            root.insert(key.clone(), json_to_item(val.clone())?);
        }
    }

    Ok(Document {
        version: (1, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    })
}

fn json_to_item(value: serde_json::Value) -> Result<Item> {
    Ok(match value {
        serde_json::Value::Object(map) => {
            let mut obj_map = BTreeMap::new();
            for (k, v) in map {
                obj_map.insert(k, json_to_item(v)?);
            }
            Item::Object(obj_map)
        }
        _ => Item::Scalar(json_to_value(value)?),
    })
}

fn json_to_value(value: serde_json::Value) -> Result<Value> {
    Ok(match value {
        serde_json::Value::String(s) => Value::String(s.into_boxed_str()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                return Err(MsgPackError::TypeMismatch {
                    expected: "Int or Float".to_string(),
                    actual: "Number".to_string(),
                });
            }
        }
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Array(_) => {
            return Err(MsgPackError::Unsupported {
                message: "Arrays require special handling".to_string(),
            });
        }
        serde_json::Value::Object(_) => {
            return Err(MsgPackError::Unsupported {
                message: "Nested objects should be handled separately".to_string(),
            });
        }
    })
}
```

## Step 6: Create Public API

Create `crates/hedl-msgpack/src/lib.rs`:

```rust
//! MessagePack conversion for HEDL documents
//!
//! Provides bidirectional conversion between HEDL and MessagePack formats.
//!
//! # Examples
//!
//! ```
//! use hedl_core::parse;
//! use hedl_msgpack::{to_msgpack, from_msgpack};
//!
//! // HEDL to MessagePack
//! let doc = parse(b"%V:2.0\n---\nname: Alice\nage: 30").unwrap();
//! let bytes = to_msgpack(&doc).unwrap();
//!
//! // MessagePack to HEDL
//! let back = from_msgpack(&bytes).unwrap();
//! ```

mod error;
mod from_msgpack;
mod to_msgpack;

pub use error::{MsgPackError, Result};
pub use from_msgpack::from_msgpack;
pub use to_msgpack::to_msgpack;
```

## Step 7: Write Comprehensive Tests

Create `crates/hedl-msgpack/tests/conversion_tests.rs`:

```rust
use hedl_core::parse;
use hedl_msgpack::{to_msgpack, from_msgpack};

#[test]
fn test_round_trip_simple() {
    let hedl = b"%V:2.0\n---\nname: Alice\nage: 30\nactive: true";
    let doc = parse(hedl).unwrap();
    let bytes = to_msgpack(&doc).unwrap();
    let back = from_msgpack(&bytes).unwrap();

    // Verify structure preserved
    assert!(back.root.contains_key("name"));
    assert!(back.root.contains_key("age"));
    assert!(back.root.contains_key("active"));
}

#[test]
fn test_nested_structures() {
    let hedl = b"%V:2.0\n---\ndatabase:\n  host: localhost\n  port: 5432\n  credentials:\n    user: admin\n    password: secret\n";
    let doc = parse(hedl).unwrap();
    let bytes = to_msgpack(&doc).unwrap();

    // Verify it doesn't panic during conversion
    let back = from_msgpack(&bytes).unwrap();
    assert!(back.root.contains_key("database"));
}

#[test]
fn test_types() {
    let hedl = b"%V:2.0\n---\nstring: hello\nint: 42\nbool: true\n";
    let doc = parse(hedl).unwrap();
    let bytes = to_msgpack(&doc).unwrap();

    let back = from_msgpack(&bytes).unwrap();
    // Verify types preserved
    use hedl_core::{Item, Value};
    assert!(matches!(back.root.get("int"), Some(Item::Scalar(Value::Int(42)))));
}
```

Create `crates/hedl-msgpack/tests/property_tests.rs`:

```rust
use hedl_msgpack::{to_msgpack, from_msgpack};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_round_trip_doesnt_panic(s in r"[a-z0-9]*") {
        // Should never panic, even with invalid input
        let _ = from_msgpack(s.as_bytes());
    }

    #[test]
    fn test_valid_docs_can_roundtrip(
        name in "[a-z]{1,20}",
        value in 1..100i64
    ) {
        let hedl = format!("---\n{}: {}", name, value);
        if let Ok(doc) = hedl_core::parse(hedl.as_bytes()) {
            let bytes = to_msgpack(&doc).unwrap();
            let result = from_msgpack(&bytes);
            assert!(result.is_ok());
        }
    }
}
```

## Step 8: Add Examples

Create `crates/hedl-msgpack/examples/basic_usage.rs`:

```rust
use hedl_core::parse;
use hedl_msgpack::{to_msgpack, from_msgpack};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HEDL-MessagePack Conversion Example ===\n");

    // Example configuration in HEDL
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
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

    // Convert to MessagePack
    let bytes = to_msgpack(&doc)?;
    println!("\nMessagePack Output (hex): {:x?}", &bytes[..20.min(bytes.len())]);

    // Convert back to HEDL
    let doc2 = from_msgpack(&bytes)?;
    let bytes2 = to_msgpack(&doc2)?;

    println!("\nRound-trip successful: {}", bytes == bytes2);

    Ok(())
}
```

Run it:
```bash
cargo run --example basic_usage -p hedl-msgpack
```

## Step 9: Add Benchmarks

Create `crates/hedl-bench/benches/formats/msgpack.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hedl_core::parse;
use hedl_msgpack::{to_msgpack, from_msgpack};

fn benchmark_msgpack(c: &mut Criterion) {
    let mut group = c.benchmark_group("msgpack_conversion");

    let small = parse(b"%V:2.0\n---\nname: Alice\nage: 30").unwrap();
    let medium = parse(b"%V:2.0\n---\nserver:\n  host: localhost\n  database:\n    url: postgres://localhost".as_bytes()).unwrap();

    group.bench_with_input(BenchmarkId::new("to_msgpack", "small"), &small, |b, doc| {
        b.iter(|| to_msgpack(black_box(doc)))
    });

    group.bench_with_input(BenchmarkId::new("to_msgpack", "medium"), &medium, |b, doc| {
        b.iter(|| to_msgpack(black_box(doc)))
    });

    group.bench_with_input(BenchmarkId::new("from_msgpack", "small"), &small, |b, doc| {
        let bytes = to_msgpack(doc).unwrap();
        b.iter(|| from_msgpack(black_box(&bytes)))
    });

    group.finish();
}

criterion_group!(benches, benchmark_msgpack);
criterion_main!(benches);
```

## Step 10: Build and Test

```bash
# Build the new crate
cargo build -p hedl-msgpack

# Run all tests
cargo test -p hedl-msgpack

# Run with verbose output
cargo test -p hedl-msgpack -- --nocapture

# Check for warnings
cargo clippy -p hedl-msgpack -- -D warnings

# Format code
cargo fmt -p hedl-msgpack

# Build documentation
cargo doc -p hedl-msgpack --open
```

## Step 11: Integrate with Main Crate

Edit `Cargo.toml` if hedl-msgpack is not already in members:

```toml
[workspace]
members = [
    # ... existing members ...
    "crates/hedl-msgpack",  # Verify it's included
]
```

## Step 12: Update Documentation

Add to `docs/developer/module-guide.md`:

```markdown
### hedl-msgpack

**Path**: `crates/hedl-msgpack/`
**Purpose**: MessagePack ↔ HEDL conversion
**Dependencies**: hedl-core, rmp-serde

#### Features

- Bidirectional MessagePack conversion
- Efficient binary format
- Full type preservation
- Fast serialization/deserialization

#### Example

\`\`\`rust
use hedl_msgpack::{to_msgpack, from_msgpack};

let doc = parse("server:\n  host: localhost")?;
let bytes = to_msgpack(&doc)?;
\`\`\`
```

## Step 13: Commit and Push

```bash
git add crates/hedl-msgpack
git add docs/developer/module-guide.md

git commit -m "feat(msgpack): Add MessagePack format converter

- Create hedl-msgpack crate for MessagePack conversion
- Implement bidirectional conversion (to_msgpack, from_msgpack)
- Handle nested structures and type preservation
- Add comprehensive unit and property tests
- Add example and benchmarks
- Update module documentation"

git push origin add-msgpack-support
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
