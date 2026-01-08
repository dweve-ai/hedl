# Rust Best Practices

This guide covers idiomatic Rust patterns and best practices for using HEDL effectively.

## Type Safety

### Using Newtype Pattern

Wrap HEDL types for domain-specific type safety:

```rust
use hedl::{Document, Value};

#[derive(Debug, Clone)]
pub struct UserId(String);

#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    name: String,
    email: String,
}

impl User {
    pub fn from_hedl_item(item: &Item) -> Result<Self, String> {
        match item {
            Item::Object(map) => {
                let id = map.get("id")
                    .and_then(|item| match item {
                        Item::Scalar(Value::String(s)) => Some(UserId(s.clone())),
                        _ => None,
                    })
                    .ok_or("Missing id")?;

                let name = map.get("name")
                    .and_then(|item| match item {
                        Item::Scalar(Value::String(s)) => Some(s.clone()),
                        _ => None,
                    })
                    .ok_or("Missing name")?;

                let email = map.get("email")
                    .and_then(|item| match item {
                        Item::Scalar(Value::String(s)) => Some(s.clone()),
                        _ => None,
                    })
                    .ok_or("Missing email")?;

                Ok(User { id, name, email })
            }
            _ => Err("Expected object".to_string())
        }
    }
}
```

### Phantom Types for Compile-Time Safety

```rust
use std::marker::PhantomData;

struct Validated;
struct Unvalidated;

pub struct HedlDoc<T> {
    doc: Document,
    _phantom: PhantomData<T>,
}

impl HedlDoc<Unvalidated> {
    pub fn parse(input: &str) -> Result<Self, HedlError> {
        Ok(HedlDoc {
            doc: hedl::parse(input)?,
            _phantom: PhantomData,
        })
    }

    pub fn validate(self) -> Result<HedlDoc<Validated>, HedlError> {
        // Validate using lint - check for errors
        use hedl::lint::{lint, Severity};
        let diagnostics = lint(&self.doc);
        if diagnostics.iter().any(|d| d.severity() == Severity::Error) {
            return Err(HedlError::new(
                hedl::HedlErrorKind::Semantic,
                "Validation failed with errors",
                0
            ));
        }
        Ok(HedlDoc {
            doc: self.doc,
            _phantom: PhantomData,
        })
    }
}

impl HedlDoc<Validated> {
    // Only validated documents can be serialized
    pub fn to_json(&self) -> Result<String, HedlError> {
        hedl::to_json(&self.doc)
    }
}

// Usage ensures validation before serialization
let doc = HedlDoc::parse(input)?;
let validated = doc.validate()?;
let json = validated.to_json()?;
```

## Error Handling

### Custom Error Types

```rust
use hedl::{HedlError, HedlErrorKind};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("HEDL parse error: {0}")]
    Parse(#[from] HedlError),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Entity not found: {entity_type}:{entity_id}")]
    NotFound {
        entity_type: String,
        entity_id: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type AppResult<T> = Result<T, AppError>;
```

### Error Context Chain

```rust
fn load_config(path: &str) -> AppResult<Config> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::Io(e))?;

    let doc = hedl::parse(&content)
        .map_err(|e| AppError::Parse(e))?;

    Config::from_document(&doc)
        .map_err(|e| AppError::Validation(format!("Failed to extract config: {}", e)))
}
```

### Pattern Matching on Error Kind

```rust
use hedl::parse;

match parse(input) {
    Ok(doc) => process_document(doc),
    Err(e) => {
        eprintln!("Parse error: {}", e);
        // For lenient parsing that ignores unresolved references
        match hedl::parse_lenient(input) {
            Ok(doc) => {
                eprintln!("Warning: Using lenient mode, some references may be unresolved");
                process_document(doc)
            }
            Err(e2) => handle_fatal_error(e2),
        }
    }
}
```

## Performance

### Parsing Optimization

#### Pre-allocate Structures

```rust
use hedl::ParseOptions;

// For known document sizes, set appropriate limits
let options = ParseOptions {
    strict_refs: true,
    ..Default::default()
};

let doc = hedl::parse_with_limits(input.as_bytes(), options)?;
```

#### Batch Parsing

```rust
use rayon::prelude::*;

fn parse_documents_parallel(inputs: &[String]) -> Vec<Result<Document, HedlError>> {
    inputs
        .par_iter()
        .map(|input| hedl::parse(input))
        .collect()
}
```

### Zero-Copy Where Possible

```rust
use hedl::Value;

// Avoid unnecessary cloning
fn extract_strings(item: &Item) -> Vec<&str> {
    let mut result = Vec::new();
    extract_strings_impl(item, &mut result);
    result
}

fn extract_strings_impl<'a>(item: &'a Item, result: &mut Vec<&'a str>) {
    match item {
        Item::Scalar(Value::String(s)) => result.push(s),
        Item::Object(map) => {
            for v in map.values() {
                extract_strings_impl(v, result);
            }
        }
        Item::List(matrix) => {
            for node in &matrix.rows {
                for field in &node.fields {
                    if let Value::String(s) = field {
                        result.push(s);
                    }
                }
            }
        }
        _ => {}
    }
}
```

### Serialization Efficiency

```rust
use hedl::{Document, HedlError};
use hedl::c14n::canonicalize;

// Reuse serialization for multiple documents
struct HedlSerializer;

impl HedlSerializer {
    pub fn new() -> Self {
        Self
    }

    pub fn serialize(&self, doc: &Document) -> Result<String, HedlError> {
        canonicalize(doc)
    }
}
```

### Memory Profiling

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use hedl::parse;

    #[bench]
    fn bench_parse_large_document(b: &mut test::Bencher) {
        let input = generate_large_hedl_document();

        b.iter(|| {
            let doc = parse(&input).unwrap();
            test::black_box(doc);
        });
    }

    #[bench]
    fn bench_to_json(b: &mut test::Bencher) {
        let input = generate_large_hedl_document();
        let doc = parse(&input).unwrap();

        b.iter(|| {
            let json = hedl::to_json(&doc).unwrap();
            test::black_box(json);
        });
    }
}
```

## Async Patterns

### Async File Loading

```rust
use tokio::fs;

async fn load_hedl_async(path: &str) -> AppResult<Document> {
    let content = fs::read_to_string(path).await?;
    let doc = tokio::task::spawn_blocking(move || {
        hedl::parse(&content)
    }).await??;

    Ok(doc)
}
```

### Streaming Parsing (Large Files)

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::fs::File;

async fn stream_parse_hedl(path: &str) -> AppResult<Document> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut content = String::new();

    while let Some(line) = lines.next_line().await? {
        content.push_str(&line);
        content.push('\n');
    }

    let doc = tokio::task::spawn_blocking(move || {
        hedl::parse(&content)
    }).await??;

    Ok(doc)
}
```

### Concurrent Processing

```rust
use tokio::sync::mpsc;

async fn process_hedl_files(paths: Vec<String>) -> Vec<AppResult<Document>> {
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn tasks for each file
    for path in paths {
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = load_hedl_async(&path).await;
            tx.send(result).await.ok();
        });
    }

    drop(tx); // Close sender

    // Collect results
    let mut results = Vec::new();
    while let Some(result) = rx.recv().await {
        results.push(result);
    }

    results
}
```

## Builder Patterns

### Document Builder

```rust
use hedl::{Document, Item, Node, Value};
use std::collections::BTreeMap;

pub struct DocumentBuilder {
    version: (u32, u32),
    structs: BTreeMap<String, Vec<String>>,
    aliases: BTreeMap<String, String>,
    nests: BTreeMap<String, String>,
    root: BTreeMap<String, Item>,
}

impl DocumentBuilder {
    pub fn new() -> Self {
        Self {
            version: (1, 0),
            structs: BTreeMap::new(),
            aliases: BTreeMap::new(),
            nests: BTreeMap::new(),
            root: BTreeMap::new(),
        }
    }

    pub fn version(mut self, major: u32, minor: u32) -> Self {
        self.version = (major, minor);
        self
    }

    pub fn add_struct(mut self, name: impl Into<String>, fields: Vec<String>) -> Self {
        self.structs.insert(name.into(), fields);
        self
    }

    pub fn add_scalar(mut self, key: impl Into<String>, value: Value) -> Self {
        self.root.insert(key.into(), Item::Scalar(value));
        self
    }

    pub fn add_object(mut self, key: impl Into<String>, obj: BTreeMap<String, Item>) -> Self {
        self.root.insert(key.into(), Item::Object(obj));
        self
    }

    pub fn build(self) -> Document {
        Document {
            version: self.version,
            structs: self.structs,
            aliases: self.aliases,
            nests: self.nests,
            root: self.root,
        }
    }
}

// Usage
let doc = DocumentBuilder::new()
    .version(1, 0)
    .add_struct("User", vec!["id".into(), "name".into()])
    .add_scalar("app_name", Value::String("MyApp".into()))
    .build();
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use hedl::{parse, to_json};

    #[test]
    fn test_parse_minimal() {
        let input = "%VERSION: 1.0\n---\n";
        let doc = parse(input).unwrap();
        assert_eq!(doc.version, (1, 0));
        assert_eq!(doc.root.len(), 0);
    }

    #[test]
    fn test_json_conversion() {
        let input = "%VERSION: 1.0\n---\nkey: value\nnum: 42";
        let doc = parse(input).unwrap();
        let json = hedl::json::to_json(&doc).unwrap();

        // Verify JSON output contains expected values
        assert!(json.contains("\"key\""));
        assert!(json.contains("\"value\""));
        assert!(json.contains("42"));
    }

    #[test]
    #[should_panic(expected = "Syntax error")]
    fn test_invalid_syntax() {
        parse("invalid hedl").unwrap();
    }
}
```

### Property-Based Testing

```rust
#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_parse_any_valid_key_value(
            key in "[a-z][a-z0-9_]*",
            value in ".*"
        ) {
            let input = format!("%VERSION: 1.0\n---\n{}: {}", key, value);
            let result = parse(&input);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_json_roundtrip_preserves_data(
            s in ".*",
            n in any::<i64>(),
            b in any::<bool>()
        ) {
            let doc = DocumentBuilder::new()
                .add_scalar("str", Value::String(s.clone()))
                .add_scalar("num", Value::Int(n))
                .add_scalar("bool", Value::Bool(b))
                .build();

            let json = hedl::json::to_json(&doc).unwrap();
            let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(&json_value["str"], &serde_json::Value::String(s));
            prop_assert_eq!(&json_value["num"], &serde_json::Value::Number(n.into()));
            prop_assert_eq!(&json_value["bool"], &serde_json::Value::Bool(b));
        }
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_load_from_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.hedl");

        fs::write(&file_path, "%VERSION: 1.0\n---\nkey: value").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        let doc = parse(&content).unwrap();

        assert_eq!(doc.root.len(), 1);
    }

    #[test]
    fn test_batch_processing() {
        let inputs = vec![
            "%VERSION: 1.0\n---\na: 1",
            "%VERSION: 1.0\n---\nb: 2",
            "%VERSION: 1.0\n---\nc: 3",
        ];

        let docs: Vec<_> = inputs
            .iter()
            .map(|input| parse(input).unwrap())
            .collect();

        assert_eq!(docs.len(), 3);
    }
}
```

## Token Efficiency

### Measuring Token Savings

```rust
use hedl_wasm::getStats; // Available in WASM binding

fn measure_token_efficiency(hedl_text: &str) {
    // Note: This example assumes WASM bindings
    // For pure Rust, implement token counting logic

    println!("HEDL document:");
    println!("{}", hedl_text);

    // Estimate tokens (simplified: ~4 chars per token)
    let hedl_tokens = hedl_text.len() / 4;

    let doc = hedl::parse(hedl_text).unwrap();
    let json = hedl::to_json(&doc).unwrap();
    let json_tokens = json.len() / 4;

    let savings = ((json_tokens - hedl_tokens) as f64 / json_tokens as f64) * 100.0;

    println!("HEDL tokens: {}", hedl_tokens);
    println!("JSON tokens: {}", json_tokens);
    println!("Token savings: {:.1}%", savings);
}
```

## Best Practices Summary

1. **Type Safety**: Use newtype pattern and phantom types
2. **Error Handling**: Custom error types with context chains
3. **Performance**: Pre-allocate, batch process, profile
4. **Async**: Use tokio for I/O, spawn_blocking for CPU work
5. **Testing**: Unit, property-based, and integration tests
6. **Documentation**: Rustdoc with examples and links

## See Also

- [Thread Safety Guide](thread-safety.md)
- [Memory Management Guide](memory-management.md)
- [Error Handling Guide](error-handling.md)
- [Core Types Reference](../reference/core-types.md)
