# hedl

The main HEDL library crate providing a unified API for working with the Hierarchical Entity Data Language.

## Installation

```toml
[dependencies]
hedl = "1.0"

# With optional format support
hedl = { version = "1.0", features = ["yaml", "xml", "csv"] }
```

## Features

- `yaml` - YAML format support
- `xml` - XML format support
- `csv` - CSV format support
- `parquet` - Apache Parquet support
- `neo4j` - Neo4j Cypher generation
- `toon` - TOON format for LLMs
- `serde` - Serde serialization support
- `all-formats` - Enable all format converters

## Usage

```rust
use hedl::{parse, to_json, from_json};

// Parse HEDL
let hedl = r#"
%VERSION: 1.0
---
name: Example
value: 42
"#;

let doc = parse(hedl.as_bytes())?;

// Convert to JSON
let json = to_json(&doc)?;
println!("{}", json);

// Parse from JSON
let doc2 = from_json(&json)?;
```

## Crate Structure

This crate re-exports functionality from:

- `hedl-core` - Core parsing and data model
- `hedl-c14n` - Canonicalization
- `hedl-json` - JSON conversion
- `hedl-lint` - Linting and validation

Optional re-exports (feature-gated):
- `hedl-yaml`, `hedl-xml`, `hedl-csv`, `hedl-parquet`, `hedl-neo4j`, `hedl-toon`

## License

Apache-2.0
