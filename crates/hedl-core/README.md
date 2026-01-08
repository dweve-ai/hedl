# hedl-core

Core parsing engine and data model for the HEDL format.

## Installation

```toml
[dependencies]
hedl-core = "1.0"
```

## Usage

```rust
use hedl_core::{parse, Document, Value};

let hedl = r#"
%VERSION: 1.0
---
user:
  name: Alice
  age: 30
"#;

let doc = parse(hedl.as_bytes())?;

// Access values
if let Some(user) = doc.get("user") {
    if let Some(obj) = user.as_object() {
        println!("Name: {:?}", obj.get("name"));
    }
}
```

## Features

- **Deterministic parsing** with fail-fast error handling
- **Zero-copy preprocessing** with line offset tables
- **Schema-defined matrices** with typed columns
- **Reference system** for graph relationships
- **Tensor literals** for AI/ML workflows
- **Ditto operator** for value repetition

## Data Model

- `Document` - Root container with header and content
- `Node` - Key-value entries in an object
- `Value` - Scalar, Object, or MatrixList
- `MatrixList` - Typed rows with schema
- `Reference` - Links between entities

## License

Apache-2.0
