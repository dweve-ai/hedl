# hedl-toon

**Bidirectional HEDL ↔ TOON conversion with TOON v3.0 compliance—optimized for LLM context windows.**

TOON (Token-Oriented Object Notation) was designed for LLM efficiency. Every token counts. Every byte costs money. But comprehensive accuracy testing with GPT-5.1, Mistral Large, and DeepSeek v3.2 reveals: **HEDL outperforms TOON for LLM comprehension** while using 10% fewer tokens.

`hedl-toon` provides bidirectional conversion between HEDL and TOON formats. Convert legacy TOON data to HEDL for better accuracy. Export HEDL to TOON when backward compatibility is required. Full TOON v3.0 specification compliance with tabular and expanded formats, three delimiter options, and comprehensive pluralization support.

## LLM Accuracy: HEDL vs TOON

Comprehensive accuracy testing across 3 major LLM providers shows HEDL consistently outperforms TOON:

| Provider | HEDL Accuracy | TOON Accuracy | HEDL Advantage |
|----------|---------------|---------------|----------------|
| GPT-5.1 | 71.8% | 68.2% | **+3.6 points** |
| Mistral Large | 51.8% | 45.1% | **+6.7 points** |
| DeepSeek v3.2 | 63.1% | 63.1% | Tie |
| **Average** | **62.2%** | **58.8%** | **+3.4 points** |

**Token Efficiency**: HEDL uses 10.3% fewer tokens than TOON while achieving higher accuracy.

**Accuracy per 1K tokens**: HEDL 0.23 vs TOON 0.21 (+9.5% efficiency)

**Conclusion**: HEDL is the superior format for LLM applications. Use `hedl-toon` only for backward compatibility with existing TOON-based systems.

## Installation

```toml
[dependencies]
hedl-toon = "1.1"
```

## What's Implemented

Full TOON v3.0 specification compliance:

1. **Bidirectional Conversion**: HEDL ↔ TOON with roundtrip fidelity
2. **Dual Format Support**: Tabular format for primitives, expanded format for complex structures
3. **Three Delimiters**: Comma, Tab, Pipe with proper TOON v3.0 bracket notation
4. **Float Normalization**: NaN/Infinity → null, -0 → 0, no trailing zeros, whole numbers as integers
5. **Reference Preservation**: Maintains `@Type:id` format without JSON corruption
6. **Comprehensive Pluralization**: 30+ irregular forms (child→children, person→people, etc.)
7. **Security Hardening**: 100-level depth limit, escape sequences, injection prevention
8. **Auto-Indentation Detection**: Parser auto-detects indent width from input
9. **Configuration API**: Builder pattern for flexible customization
10. **Comprehensive Error Handling**: 8 error types with line number tracking

## Bidirectional Conversion

### HEDL → TOON: Export for Legacy Systems

Convert HEDL documents to TOON when backward compatibility is required:

```rust
use hedl_toon::{hedl_to_toon, to_toon, ToToonConfig, Delimiter};

let doc = hedl_core::parse(br#"
%STRUCT: User: [id, name, age]
---
users: @User
  | alice, Alice Smith, 30
  | bob, Bob Jones, 25
"#)?;

// Default configuration (2-space indent, comma delimiter)
let toon = hedl_to_toon(&doc)?;
```

Generated TOON (tabular format for primitive arrays):

```toon
users[2]{id,name,age}:
  alice,Alice Smith,30
  bob,Bob Jones,25
```

### Custom Configuration

```rust
let config = ToToonConfig::builder()
    .indent(4)                     // 4-space indentation
    .delimiter(Delimiter::Tab)      // Tab-separated values
    .build();

let toon = to_toon(&doc, &config)?;
```

Generated TOON with tabs:

```toon
users[2	]{id	name	age}:
    alice	Alice Smith	30
    bob	Bob Jones	25
```

### TOON → HEDL: Import Legacy Data

Parse existing TOON files into HEDL's structured format:

```rust
use hedl_toon::{toon_to_hedl, from_toon};

let toon = r#"app_name: MyApp
version: 1.0
users[2]{id,name}:
  alice,Alice
  bob,Bob
"#;

// Auto-detects indentation
let doc = toon_to_hedl(toon)?;
// Or equivalently:
let doc = from_toon(toon)?;

// Now use HEDL's structured API for querying, validation, transformation
```

## TOON Format Specification

### Tabular Format (Primitives Only)

For arrays containing only primitive values (null, bool, int, float, string, references) with no nested children:

```toon
users[3]{id,name,age,active}:
  alice,Alice Smith,30,true
  bob,Bob Jones,25,false
  carol,Carol White,35,true
```

**Requirements**:
- ALL values must be primitives
- NO nested children allowed
- Compact, efficient for tabular data

### Expanded Format (Complex/Nested)

For arrays with complex values or nested children:

```toon
orders[2]:
  - id: ord1
    customer: @User:alice
    total: 149.99
    items[2]{product,quantity}:
      widget,5
      gadget,3
  - id: ord2
    customer: @User:bob
    total: 89.99
    items[1]{product,quantity}:
      doohickey,10
```

**Features**:
- Each item starts with `- ` marker (dash + space)
- Supports nested fields and children
- Child arrays use pluralized names (item → items)
- Can mix tabular and expanded formats at different levels

## Delimiter Options

Three delimiter types with TOON v3.0 bracket notation:

### Comma (Default)

```toon
users[2]{id,name}:
  alice,Alice
  bob,Bob
```

### Tab

```toon
users[2	]{id	name}:
  alice	Alice
  bob	Bob
```

**Note**: Tab character appears in brackets: `[count\t]` and between field names: `{field1\tfield2}`

### Pipe

```toon
users[2|]{id|name}:
  alice|Alice
  bob|Bob
```

**Use Cases**:
- Comma: Default, human-readable
- Tab: When data contains commas (addresses, descriptions)
- Pipe: When data contains both commas and tabs

## Float Normalization (TOON v3.0)

All float values are normalized according to TOON v3.0 specification:

```rust
// Special values → null
NaN        → null
Infinity   → null
-Infinity  → null

// Sign normalization
-0         → 0

// Whole numbers → integer format
42.0       → 42
100.0      → 100

// No trailing zeros
3.1400     → 3.14
5.000      → 5

// No exponent notation
1.5e10     → 15000000000
```

## String Quoting and Escaping

Strings are automatically quoted when they contain special characters or could be ambiguous:

### Quoting Triggers

```toon
# Empty or whitespace
name: ""
desc: " leading space"

# Boolean/null literals
status: "true"      # Quoted to distinguish from boolean true
value: "null"       # Quoted to distinguish from null

# Numeric-like strings
id: "123"           # Quoted to distinguish from integer 123
code: "-456"        # Quoted to distinguish from negative integer

# Structural characters
path: "config:value"     # Contains ':'
data: "[test]"           # Contains '[' and ']'

# Contains active delimiter
note: "hello, world"     # Contains comma (when using Comma delimiter)

# Special markers
ref: "@example"          # Starts with '@' (looks like reference)
item: "- test"           # Starts with '-' (looks like list marker)
```

### Escape Sequences

```toon
# Backslash and quotes
path: "C:\\Program Files\\App"
text: "He said \"hello\""

# Newlines and whitespace
multiline: "Line 1\nLine 2\nLine 3"
tabs: "Column1\tColumn2\tColumn3"
cr: "Old Mac\rLine"
```

## Reference Handling

References are preserved as primitive string values (prevents corruption through JSON conversion):

### Qualified References

```rust
// HEDL: Value::Reference(Reference::qualified("User", "alice"))
// TOON: @User:alice

customer: @User:alice
author: @Person:john_doe
```

### Local References

```rust
// HEDL: Value::Reference(Reference::local("item1"))
// TOON: @item1

prev: @item1
next: @item2
```

## Pluralization

Child node arrays use pluralized field names. Comprehensive support for 30+ irregular forms:

### Common Irregulars

```toon
# People
child → children
person → people
man → men
woman → women

# Body parts
foot → feet
tooth → teeth
goose → geese

# Animals
mouse → mice
ox → oxen
sheep → sheep (unchanged)

# Scientific terms
phenomenon → phenomena
criterion → criteria
datum → data
analysis → analyses

# Classical plurals
cactus → cacti
fungus → fungi
nucleus → nuclei
radius → radii
```

### Case Preservation

```toon
# Lowercase
child → children

# Capitalized
Child → Children

# Uppercase
CHILD → CHILDREN
```

### Regular Plurals (Fallback)

```toon
user → users
product → products
order → orders
```

## Security: Depth Limit Protection

Prevents stack overflow from deeply nested structures:

```rust
const MAX_NESTING_DEPTH: usize = 100;

// Attempting to convert/parse > 100 levels deep:
// Error: MaxDepthExceeded { depth: 101, max: 100 }
```

**Protection against**:
- Malicious deeply nested input
- Accidental infinite recursion
- Stack overflow attacks

## Configuration Reference

### ToToonConfig

```rust
use hedl_toon::{ToToonConfig, Delimiter};

let config = ToToonConfig {
    indent: 2,                  // Spaces per indentation level (default: 2)
    delimiter: Delimiter::Comma, // Field delimiter (default: Comma)
};

// Or use builder pattern
let config = ToToonConfig::builder()
    .indent(4)
    .delimiter(Delimiter::Tab)
    .build();
```

**Validation**: Indent must be ≥1 (returns `InvalidIndent` error otherwise)

### FromToonConfig

```rust
use hedl_toon::FromToonConfig;

let config = FromToonConfig {
    indent_width: 0,  // 0 = auto-detect (default), or specify explicit width
};
```

**Auto-Detection**: Parser detects indent width from first indented line, validates consistency throughout document.

## Error Handling

Comprehensive error types with line number tracking:

```rust
use hedl_toon::ToonError;

match toon_to_hedl(toon) {
    Ok(doc) => { /* success */ }
    Err(ToonError::MaxDepthExceeded { depth, max }) => {
        eprintln!("Nesting too deep: {} levels (max: {})", depth, max);
    }
    Err(ToonError::ParseError { line, message }) => {
        eprintln!("Parse error at line {}: {}", line, message);
    }
    Err(ToonError::SchemaMismatch { type_name, expected, actual }) => {
        eprintln!("Schema mismatch in {}: expected {} fields, found {}",
            type_name, expected, actual);
    }
    Err(ToonError::IndentationError { line, message }) => {
        eprintln!("Indentation error at line {}: {}", line, message);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### Error Types

- `MaxDepthExceeded` - Nesting > 100 levels
- `InvalidIndent` - Indent must be ≥1
- `SchemaMismatch` - Field count doesn't match schema
- `ParseError` - Invalid TOON syntax
- `UnexpectedEof` - Premature end of input
- `InvalidArrayHeader` - Malformed array header
- `InvalidValue` - Invalid value at location
- `IndentationError` - Inconsistent indentation

## Format Selection Logic

The encoder automatically chooses the optimal format:

1. **Empty array** → Simple header: `users[0]:`
2. **All primitives + no children** → Tabular format (compact)
3. **Any complex value OR has children** → Expanded format (flexible)

```rust
// Primitives only → Tabular
users[2]{id,name}:
  alice,Alice
  bob,Bob

// Has children → Expanded
teams[1]:
  - id: eng
    name: Engineering
    members[2]{id,name}:
      alice,Alice
      bob,Bob
```

## Use Cases

**Legacy TOON Migration**: Convert existing TOON data to HEDL for improved LLM accuracy (+3.4 points average) and 10% token efficiency gain.

**Backward Compatibility**: Export HEDL to TOON when integrating with systems that only consume TOON format.

**Format Comparison**: Evaluate TOON vs HEDL for your specific LLM workloads using bidirectional conversion for A/B testing.

**Multi-Format Pipelines**: Read TOON from legacy sources, convert to HEDL for processing, combine with JSON APIs (`hedl-json`), export to various formats.

## What This Crate Doesn't Do

**Schema Preservation**: TOON has no schema concept (like CSV and JSON). HEDL's `%STRUCT`, `%NEST`, `%ALIAS` declarations are lost in TOON export. If you need schemas, keep HEDL source files or redefine schemas after import.

**Type Inference**: TOON → HEDL conversion uses basic type detection (numbers, booleans, nulls) but can't infer complex semantic types. Schema information must come from external sources or HEDL struct definitions.

**Validation**: Converts formats faithfully, doesn't validate data against business rules. For schema validation, use `hedl-lint` on HEDL documents.

**Complex Expression Preservation**: While references (`@Type:id`) are preserved, computed expressions (`$(expr)`) are converted to strings in TOON and back to expressions in HEDL. Complex expression semantics may be lost if not properly formatted.

## Performance Characteristics

**Conversion**: HEDL → TOON is O(n) time and space where n = total nodes. TOON → HEDL is O(n) parsing with depth-limited recursion.

**Memory**: Linear with document size. No significant overhead beyond output buffer allocation.

**String Operations**: Optimized with pre-allocation for common cases. Quoting decisions made in single pass.

**Pluralization**: O(1) lookup via HashMap (lazy-initialized on first use).

## Dependencies

- `hedl-core` 1.0 - HEDL parsing and data model
- `thiserror` 1.0 - Error type definitions

## License

Apache-2.0
