# hedl-yaml

Bidirectional YAML conversion for HEDL documents.

## Installation

```toml
[dependencies]
hedl-yaml = "1.0"
```

## Usage

```rust
use hedl_core::parse;
use hedl_yaml::{to_yaml, from_yaml};

// HEDL to YAML
let doc = parse(hedl.as_bytes())?;
let yaml = to_yaml(&doc)?;

// YAML to HEDL
let doc = from_yaml(&yaml)?;
```

## Features

- **Bidirectional conversion** - HEDL to YAML and YAML to HEDL
- **Structure preservation** - Maintains nested hierarchies
- **Type inference** - Automatic type detection from YAML

## License

Apache-2.0
