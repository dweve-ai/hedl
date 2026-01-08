# hedl-csv

Bidirectional CSV conversion for HEDL matrix lists.

## Installation

```toml
[dependencies]
hedl-csv = "1.0"
```

## Usage

```rust
use hedl_core::parse;
use hedl_csv::{to_csv, from_csv};

// HEDL matrix list to CSV
let doc = parse(hedl.as_bytes())?;
let csv = to_csv(&doc, "users")?;  // Export "users" matrix

// CSV to HEDL
let doc = from_csv(&csv, "User")?;  // Import as "User" type
```

## Features

- **Matrix list export** - Convert HEDL matrix lists to CSV
- **CSV import** - Parse CSV into HEDL matrix lists
- **Header support** - Automatic schema detection from headers
- **Type inference** - Infer column types from data

## License

Apache-2.0
