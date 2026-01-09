# hedl-toon

Bidirectional HEDL to TOON (Token-Oriented Object Notation) conversion.

## Installation

```toml
[dependencies]
hedl-toon = "1.0"
```

## Usage

### HEDL to TOON

```rust
use hedl_core::parse;
use hedl_toon::{hedl_to_toon, to_toon, ToToonConfig, Delimiter};

// HEDL to TOON with defaults
let doc = parse(hedl.as_bytes())?;
let toon = hedl_to_toon(&doc)?;

// With custom config
let config = ToToonConfig::builder()
    .indent(4)
    .delimiter(Delimiter::Tab)
    .build();
let toon = to_toon(&doc, &config)?;
```

### TOON to HEDL

```rust
use hedl_toon::{toon_to_hedl, from_toon};

// TOON to HEDL Document
let toon = r#"name: MyApp
version: 1
users[2]{id,name}:
  u1,Alice
  u2,Bob
"#;
let doc = toon_to_hedl(toon)?;
// Or equivalently:
let doc = from_toon(toon)?;
```

## What is TOON?

TOON (Token-Oriented Object Notation) is a compact format optimized for LLM context windows:

- **70% smaller** than equivalent JSON
- **Minimal syntax** - reduces token overhead
- **LLM-friendly** - optimized for AI consumption

### TOON Formats

**Tabular Format** (for primitive arrays):
```
users[2]{id,name,age}:
  u1,Alice,30
  u2,Bob,25
```

**Expanded Format** (for complex/nested structures):
```
orders[1]:
  - id: ord1
    customer: @User:u1
    total: 99.99
```

## Example

HEDL input:
```hedl
%STRUCT: User: [id, name, age]
---
users: @User
  | u1, Alice, 30
  | u2, Bob, 25
```

TOON output:
```
users[2]{id,name,age}:
  u1,Alice,30
  u2,Bob,25
```

## Delimiters

Three delimiter types are supported:

- `Delimiter::Comma` - Default, human-readable
- `Delimiter::Tab` - For TSV-like data
- `Delimiter::Pipe` - When data contains commas

## Features

- **Bidirectional conversion** - HEDL to TOON and TOON to HEDL
- **Reference preservation** - Maintains `@Type:id` references
- **Format optimization** - Automatically selects tabular vs expanded format
- **Security** - Depth limit protection against stack overflow

## License

Apache-2.0
