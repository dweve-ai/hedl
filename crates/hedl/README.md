# hedl

**The unified HEDL API -parse, validate, convert, and canonicalize with a single, ergonomic interface.**

Production systems need simple APIs. Importing a dozen crates for basic operations creates cognitive overhead. Feature flags should work intuitively. Errors need rich context without boilerplate. The most common operations should be frictionless.

`hedl` is the facade crate providing the complete HEDL experience through 7 core functions and comprehensive error handling utilities. Zero default dependencies -only pay for features you enable. Full re-exports from 13 specialized crates beneath a unified API. Designed for developers who want results, not dependency management.

## What's Implemented

Unified API with zero-overhead ergonomics:

1. **7 Core Functions**: parse, parse_lenient, canonicalize, to_json, from_json, lint, validate
2. **8 Feature Flags**: serde, yaml, xml, csv, parquet, neo4j, toon, all-formats
3. **Zero Default Dependencies**: Core parsing only -enable formats as needed
4. **Error Context Extensions**: HedlResultExt trait with .context() and .with_context()
5. **Full Re-Exports**: 10 internal crates exposed through unified namespace
6. **Type Aliases**: Convenient Result<T> = std::result::Result<T, HedlError>
7. **299 Integration Tests**: Comprehensive validation including error context propagation
8. **Minimal Overhead**: < 1% compared to using crates directly
9. **Feature Composition**: Mix and match formats without conflicts
10. **Ergonomic Imports**: use hedl::*; gives you everything you need

## Installation

### Minimal (Core Only)

Parse, validate, and work with HEDL documents:

```toml
[dependencies]
hedl = "2.0"
```

**What you get**: Parsing, canonicalization, linting, validation
**What you don't get**: Format conversion (JSON/YAML/XML/etc.)

### With Specific Formats

Enable only the formats you need:

```toml
[dependencies]
hedl = { version = "2.0", features = ["yaml", "xml"] }
```

### All Features

Get everything (recommended for applications):

```toml
[dependencies]
hedl = { version = "2.0", features = ["all-formats"] }
```

**Includes**: JSON, YAML, XML, CSV, Parquet, Neo4j, TOON, Serde

## Core API

### parse(input: &str) -> Result<Document>

Parse HEDL document with strict validation:

```rust
use hedl::parse;

let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, age]
---
users: @User
 |alice, Alice Smith, 30
 |bob, Bob Jones, 25
"#;

let doc = parse(hedl)?;
println!("Version: {}.{}", doc.version.0, doc.version.1);
```

**Behavior**:
- Strict parsing (all errors fatal)
- Schema validation
- Reference checking
- UTF-8 validation

**Use When**: Production parsing, CI/CD validation, strict correctness required

### parse_lenient(bytes: &[u8]) -> Result<Document>

Parse with relaxed validation:

```rust
let doc = parse_lenient(hedl_bytes)?;
// Tolerates some malformations:
// - Missing schemas (inferred from data)
// - Unresolved references (preserved as strings)
// - Inconsistent indentation (best-effort parse)
```

**Use When**: Parsing user-generated content, migration from other formats, exploratory analysis

### validate(bytes: &[u8]) -> Result<()>

Validate document without constructing AST:

```rust
use hedl::validate;

if validate(hedl_bytes).is_ok() {
    println!("Valid HEDL document");
}
```

**Performance**: Faster than `parse()` when you only need validation (no AST construction)

**Use Cases**: CI/CD checks, pre-commit hooks, batch validation

### canonicalize(doc: &Document) -> Result<String>

Convert to canonical form:

```rust
use hedl::{parse, canonicalize};

let doc = parse(hedl_bytes)?;
let canonical = canonicalize(&doc)?;
// Same document always produces identical output
// Perfect for version control and content-addressable storage
```

**Output Characteristics**:
- Deterministic formatting (2-space indentation)
- Count hints on matrix lists
- Normalized floats (no trailing zeros)
- Sorted keys (optional)

**Use When**: Git commits, content hashing, cache keys, comparing documents

### lint(doc: &Document) -> Vec<Diagnostic>

Run lint checks for best practices:

```rust
use hedl::{parse, lint};

let doc = parse(hedl_bytes)?;
let diagnostics = lint(&doc);

for diag in diagnostics {
    println!("[{}] Line {}: {}",
        diag.severity, diag.line, diag.message);
}
```

**Rules Checked** (default):
- id-naming: ID naming conventions
- unused-schema: Unused %STRUCT definitions
- empty-list: Empty matrix lists
- unqualified-kv-ref: Unqualified references in key-value context

**Use When**: Code review automation, CI/CD quality gates, pre-commit hooks

### to_json(doc: &Document) -> Result<String>

Convert to JSON (requires "json" feature, enabled by default):

```rust
use hedl::{parse, to_json};

let doc = parse(hedl_bytes)?;
let json = to_json(&doc)?;
// Use with existing JSON APIs
```

**Configuration**: Use `hedl_json::to_json_with_config()` for custom options (pretty-printing, type preservation)

### from_json(json: &str) -> Result<Document>

Convert from JSON to HEDL:

```rust
use hedl::from_json;

let json = r#"{"users": [{"id": "alice", "name": "Alice"}]}"#;
let doc = from_json(json)?;
// Now use HEDL's structured API
```

**Type Inference**: Automatically detects arrays-of-objects → matrix lists

## Error Handling with Context

### The Problem

Standard Rust errors lose context during propagation:

```rust
fn load_config() -> Result<Config> {
    let data = std::fs::read("config.hedl")?;
    let doc = parse(&data)?;
    // Error: "Parse error at line 42: unexpected token"
    // Missing: WHICH file? WHAT operation?
}
```

### The Solution: HedlResultExt

Add context at error boundaries:

```rust
use hedl::{parse, HedlResultExt};

fn load_config(path: &str) -> Result<Config> {
    let data = std::fs::read(path)
        .context("Failed to read config file")?;

    let doc = parse(&data)
        .with_context(|| format!("Failed to parse config: {}", path))?;

    Ok(/* ... */)
}
```

**Output**:
```
Failed to parse config: config.hedl
  caused by: Parse error at line 42: unexpected token ':'
```

### HedlResultExt Methods

#### .context(msg: &str)

Add static context:

```rust
parse(data).context("Failed to parse HEDL document")?;
```

#### .with_context<F: FnOnce() -> String>(f: F)

Add computed context (lazily evaluated):

```rust
parse(data)
    .with_context(|| format!("Failed to parse file: {}", filename))?;
```

**Benefits**:
- Lazy: Closure only evaluated on error (zero cost on success path)
- Flexible: Capture variables, format strings, complex logic

#### .map_err_to_hedl()

Convert external errors to HedlError:

```rust
std::fs::read("file.hedl")
    .map_err_to_hedl()
    .context("Failed to read input")?;
```

**Use Cases**:
- Wrapping std::io::Error
- Converting serde errors
- Unifying error types

## Feature Flags

### serde

Enable Serde serialization for Document type:

```toml
hedl = { version = "2.0", features = ["serde"] }
```

**Provides**: Serialize/Deserialize impls for Document, Value, Reference

**Use Cases**: Storing parsed documents in databases, IPC, caching

### yaml

Enable YAML format conversion:

```toml
hedl = { version = "2.0", features = ["yaml"] }
```

**Adds**:
- `hedl::to_yaml(doc) -> Result<String>`
- `hedl::from_yaml(yaml) -> Result<Document>`

**Dependencies**: serde_yaml 0.9

### xml

Enable XML format conversion:

```toml
hedl = { version = "2.0", features = ["xml"] }
```

**Adds**:
- `hedl::to_xml(doc) -> Result<String>`
- `hedl::from_xml(xml) -> Result<Document>`

**Dependencies**: quick-xml 0.31

### csv

Enable CSV format conversion:

```toml
hedl = { version = "2.0", features = ["csv"] }
```

**Adds**:
- `hedl::to_csv(doc) -> Result<String>` (exports first entity list)
- `hedl::from_csv(csv, type_name) -> Result<Document>`

**Dependencies**: csv 1.3

### parquet

Enable Apache Parquet columnar format:

```toml
hedl = { version = "2.0", features = ["parquet"] }
```

**Adds**:
- `hedl::to_parquet(doc, writer) -> Result<()>`
- `hedl::from_parquet(reader) -> Result<Document>`

**Dependencies**: arrow 57.0, parquet 57.0

### neo4j

Enable Neo4j Cypher generation:

```toml
hedl = { version = "2.0", features = ["neo4j"] }
```

**Adds**:
- `hedl::to_cypher(doc) -> Result<String>`
- Cypher CREATE/MERGE statements for Neo4j import

**Dependencies**: None (pure Cypher generation)

### toon

Enable TOON format (token-optimized for LLMs):

```toml
hedl = { version = "2.0", features = ["toon"] }
```

**Adds**:
- `hedl::to_toon(doc) -> Result<String>`
- `hedl::from_toon(toon) -> Result<Document>`

**Use Case**: Maximum token efficiency for LLM contexts

### all-formats

Enable all format converters:

```toml
hedl = { version = "2.0", features = ["all-formats"] }
```

**Equivalent to**: `["serde", "yaml", "xml", "csv", "parquet", "neo4j", "toon"]`

**Recommended for**: Applications (not libraries)

## Re-Exported Modules

Full access to specialized crates through unified namespace:

```rust
use hedl::{
    // Core types
    Document, Value, Reference, HedlError,

    // Sub-modules (explicit opt-in)
    core,      // hedl-core (parsing, AST)
    c14n,      // hedl-c14n (canonicalization)
    json,      // hedl-json (always available)
    lint,      // hedl-lint (linting)

    // Feature-gated modules
    yaml,      // hedl-yaml (requires "yaml")
    xml,       // hedl-xml (requires "xml")
    csv,       // hedl-csv (requires "csv")
    parquet,   // hedl-parquet (requires "parquet")
    neo4j,     // hedl-neo4j (requires "neo4j")
    toon,      // hedl-toon (requires "toon")
};
```

**Use When**: Need advanced configuration beyond core API (e.g., `hedl::json::JsonConfig` for custom JSON options)

## Common Patterns

### Parse and Convert

```rust
use hedl::{parse, to_json};

let doc = parse(hedl_bytes)?;
let json = to_json(&doc)?;
// Send JSON to REST API
```

### Validate Before Processing

```rust
use hedl::{validate, parse};

if validate(hedl_bytes).is_err() {
    return Err("Invalid HEDL document");
}

let doc = parse(hedl_bytes)?; // Safe to unwrap
```

### Multi-Format Pipeline

```rust
use hedl::{from_json, to_yaml, to_xml};

let doc = from_json(json_str)?;
let yaml = to_yaml(&doc)?;
let xml = to_xml(&doc)?;
```

### Error Context Chaining

```rust
use hedl::{parse, HedlResultExt};

fn process_file(path: &str) -> Result<()> {
    let data = std::fs::read(path)
        .context("Failed to read input file")?;

    let doc = parse(&data)
        .with_context(|| format!("Failed to parse: {}", path))?;

    let canonical = canonicalize(&doc)
        .context("Failed to canonicalize document")?;

    std::fs::write(format!("{}.canonical", path), canonical)
        .context("Failed to write canonical output")?;

    Ok(())
}
```

## Type Aliases

Convenient Result type with HedlError:

```rust
pub type Result<T> = std::result::Result<T, HedlError>;

// Use in function signatures
fn my_function() -> Result<Document> {
    parse(input)?
}
```

## Performance Characteristics

**Zero Overhead**: Facade adds < 1% overhead vs using crates directly

**Feature Gating**: Unused features don't bloat binary (tree-shaking works)

**Compile Time**: Minimal -most work delegated to specialized crates

**Binary Size**: Core-only build ~200 KB, all-formats ~800 KB (compressed)

## Use Cases

**Application Development**: Use `features = ["all-formats"]` for maximum flexibility without managing individual crate versions.

**Library Development**: Minimize dependencies -only enable formats your library needs. Let downstream users add more formats via their own dependencies.

**CLI Tools**: Import hedl with all-formats for comprehensive format support in user-facing tools.

**Embedded Systems**: Use core-only (no features) for minimal binary size. Add formats selectively based on constraints.

**Microservices**: Enable only formats you consume/produce (e.g., JSON for REST APIs, Parquet for analytics export).

## What This Crate Doesn't Do

**Streaming Parsing**: Use `hedl-stream` directly for memory-efficient streaming of multi-GB files.

**LSP/MCP Servers**: Use `hedl-lsp` or `hedl-mcp` directly for server applications.

**C/WASM Bindings**: Use `hedl-ffi` or `hedl-wasm` directly for non-Rust integrations.

**Benchmarking**: Use `hedl-bench` directly for performance measurement and regression detection.

**Testing Utilities**: Use `hedl-test` directly for shared test fixtures and builders.

## Dependencies

### Core (Zero Features)

- `hedl-core` 2.0 - Parsing and AST
- `hedl-c14n` 1.0 - Canonicalization
- `hedl-json` 1.0 - JSON (always included)
- `hedl-lint` 1.0 - Linting
- `thiserror` 1.0 - Error types

### Optional (Feature-Gated)

- `hedl-yaml` 1.0 - YAML conversion (feature: yaml)
- `hedl-xml` 1.0 - XML conversion (feature: xml)
- `hedl-csv` 1.0 - CSV conversion (feature: csv)
- `hedl-parquet` 1.0 - Parquet conversion (feature: parquet)
- `hedl-neo4j` 1.0 - Neo4j Cypher (feature: neo4j)
- `hedl-toon` 1.0 - TOON format (feature: toon)
- `serde` 1.0 - Serialization (feature: serde)

## License

Apache-2.0
