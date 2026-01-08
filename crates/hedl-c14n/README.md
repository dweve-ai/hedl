# hedl-c14n

Canonicalization and pretty-printing for HEDL documents.

## Installation

```toml
[dependencies]
hedl-c14n = "1.0"
```

## Usage

```rust
use hedl_core::parse;
use hedl_c14n::{canonicalize, Config};

let doc = parse(hedl.as_bytes())?;

// Canonicalize with defaults
let canonical = canonicalize(&doc)?;

// With custom config
let config = Config::builder()
    .indent_size(2)
    .use_ditto(true)
    .build();
let canonical = hedl_c14n::canonicalize_with_config(&doc, &config)?;
```

## Features

- **Deterministic output** - Same document always produces identical output
- **Ditto optimization** - Reduces token count by using `"` for repeated values
- **Configurable formatting** - Control indentation and style
- **Round-trip stable** - `parse(canonicalize(doc)) == doc`

## License

Apache-2.0
