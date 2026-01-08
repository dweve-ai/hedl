# hedl-toon

HEDL to TOON (Token-Oriented Object Notation) conversion.

## Installation

```toml
[dependencies]
hedl-toon = "1.0"
```

## Usage

```rust
use hedl_core::parse;
use hedl_toon::{hedl_to_toon, to_toon, ToToonConfig};

// HEDL to TOON with defaults
let doc = parse(hedl.as_bytes())?;
let toon = hedl_to_toon(&doc)?;

// With config
let config = ToToonConfig::builder()
    .delimiter(Delimiter::Comma)
    .build();
let toon = to_toon(&doc, &config)?;
```

## What is TOON?

TOON (Token-Oriented Object Notation) is a compact format optimized for LLM context windows:

- **70% smaller** than equivalent JSON
- **Minimal syntax** - reduces token overhead
- **LLM-friendly** - optimized for AI consumption

## Example

HEDL input:
```hedl
user:
  name: Alice
  age: 30
```

TOON output:
```
user(name:Alice,age:30)
```

## License

Apache-2.0
