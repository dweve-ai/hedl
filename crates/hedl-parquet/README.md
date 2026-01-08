# hedl-parquet

Bidirectional Apache Parquet conversion for HEDL documents.

## Installation

```toml
[dependencies]
hedl-parquet = "1.0"
```

## Usage

```rust
use hedl_core::parse;
use hedl_parquet::{to_parquet, from_parquet};
use std::fs::File;

// HEDL to Parquet
let doc = parse(hedl.as_bytes())?;
let file = File::create("output.parquet")?;
to_parquet(&doc, file)?;

// Parquet to HEDL
let file = File::open("input.parquet")?;
let doc = from_parquet(file)?;
```

## Features

- **Bidirectional conversion** - HEDL to Parquet and Parquet to HEDL
- **Arrow integration** - Uses Apache Arrow for columnar storage
- **Efficient storage** - Columnar format optimized for analytics
- **Schema preservation** - Maintains HEDL type information

## License

Apache-2.0
