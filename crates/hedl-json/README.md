# hedl-json

Bidirectional JSON conversion for HEDL documents.

## Installation

```toml
[dependencies]
hedl-json = "1.0"
```

## Usage

```rust
use hedl_core::parse;
use hedl_json::{to_json, from_json, ToJsonConfig};

// HEDL to JSON
let doc = parse(hedl.as_bytes())?;
let json = to_json(&doc)?;

// JSON to HEDL
let doc = from_json(&json)?;

// With config
let config = ToJsonConfig::builder()
    .pretty(true)
    .include_metadata(false)
    .build();
let json = hedl_json::to_json_with_config(&doc, &config)?;
```

## Features

- **Bidirectional conversion** - HEDL to JSON and JSON to HEDL
- **Metadata preservation** - Optionally include type information
- **Streaming support** - Process large files efficiently
- **JSONPath queries** - Query HEDL documents with JSONPath
- **Schema generation** - Generate JSON Schema from HEDL

## License

Apache-2.0
