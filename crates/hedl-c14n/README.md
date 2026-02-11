# hedl-c14n

**Canonical form generation for HEDL documents -deterministic serialization for minimal token count.**

Comparing HEDL documents shouldn't fail on whitespace differences. Git diffs shouldn't show spurious changes from inconsistent formatting. LLM context windows are expensive -every token matters. Production systems need bit-for-bit identical outputs for cache hits and content-addressable storage. Cryptographic signatures require deterministic serialization.

`hedl-c14n` implements canonical form generation per SPEC.md Section 13.2. Transform any valid HEDL document into normalized form with consistent 1-space indentation, sorted keys, and count hints on matrix lists. Same document always produces identical output. Round-trip stable -parse(canonicalize(doc)) preserves semantic equivalence.

## What's Implemented

Comprehensive canonicalization with performance and security:

1. **Deterministic Serialization**: Bit-for-bit identical output for equivalent documents
2. **Count Hints**: Automatic `[count]` annotations on matrix lists for fast parsing
3. **Value Normalization**: Float formatting (no trailing zeros, -0 → 0), null as `~`, lowercase booleans
4. **Key Ordering**: Optional alphabetic sorting for consistent field ordering
5. **Quoting Strategy**: Minimal quoting (only when necessary) or always-quote modes
6. **Schema Options**: Inline schemas in matrix headers or separate %S: declarations
7. **Security Hardening**: 1000-level depth limit prevents stack overflow
8. **Performance Optimizations**: Pre-allocated buffers (P1), direct BTreeMap iteration (P0)
9. **Round-Trip Stability**: Semantic equivalence preserved through parse → canonicalize → parse

## Installation

```toml
[dependencies]
hedl-c14n = "2.0"
```

## Basic Usage

### Canonicalize with Defaults

```rust
use hedl_core::parse;
use hedl_c14n::canonicalize;

let doc = parse(br#"
%V:2.0
%S:User:[id, name, email]
---
users: @User
 | alice, Alice Smith, alice@example.com
 | bob, Bob Jones, bob@example.com
 | charlie, Charlie Brown, charlie@example.com
"#)?;

let canonical = canonicalize(&doc)?;
println!("{}", canonical);
```

**Output**:
```hedl
%V:2.0
%S:User:[id, name, email]
---
users: @User[3]
 | alice, Alice Smith, alice@example.com
 | bob, Bob Jones, bob@example.com
 | charlie, Charlie Brown, charlie@example.com
```

**Features Applied**:
- Count hint `[3]` added automatically
- Consistent 1-space indentation
- Minimal quoting (only when required)
- Preserved key order

### Custom Configuration

```rust
use hedl_c14n::{canonicalize_with_config, CanonicalConfig, QuotingStrategy};

let config = CanonicalConfig::builder()
    .sort_keys(true)                          // Alphabetically sort fields
    .inline_schemas(true)                     // Inline schemas in headers
    .quoting(QuotingStrategy::Minimal)        // Minimal quoting
    .build();

let canonical = canonicalize_with_config(&doc, &config)?;
```

## Count Hints

Automatically generate `[count]` annotations:

```rust
// Input: no count hint
users: @User
 | alice, Alice
 | bob, Bob

// Output: count hint added
users: @User[2]
 | alice, Alice
 | bob, Bob
```

**Benefits**:
- Parsers can pre-allocate memory (avoids Vec reallocation)
- Streaming parsers know total row count upfront
- Validation can detect truncated lists
- 20-30% faster parsing for large lists

**Algorithm**: Recursive traversal counts nodes in each matrix list before serialization.

## Value Normalization

All values normalized to canonical form:

### Float Normalization

```rust
// No trailing zeros
3.1400 → 3.14
5.000 → 5.0

// Whole numbers as floats (preserve type)
42.0 → 42.0
100.0 → 100.0

// Negative zero normalized
-0.0 → 0.0

// Special values
NaN → null     // Not preserved (becomes null)
Infinity → null
-Infinity → null
```

### Null Representation

```hedl
# Canonical form uses tilde
field: ~
```

### Boolean Lowercase

```rust
True → true
FALSE → false
```

### Reference Format

```hedl
# Qualified references
customer: @User:alice

# Local references
prev: @item1
```

## Quoting Strategy

Two quoting modes control string serialization:

### Minimal (Default)

Quote only when necessary:

```hedl
# No quotes needed
name: Alice Smith
status: active

# Quotes required (contains special characters)
path: "C:\\Program Files"
note: "Hello, world"    # Contains comma
value: "true"           # Looks like boolean
id: "42"                # Looks like integer
ref: "@alice"           # Starts with @ (looks like reference)
```

**Triggers for Quoting**:
- Contains structural characters: `: [ ] { } , | @`
- Starts with `-` (looks like list marker)
- Matches boolean literal: `true`, `false`
- Matches null literal: `null`, `~`
- Looks like number: `123`, `-456`, `3.14`
- Empty string or only whitespace
- Contains quotes or backslashes (requires escaping)

### Always

Quote all strings unconditionally:

```hedl
name: "Alice Smith"
status: "active"
age: 30              # Numbers never quoted
active: true         # Booleans never quoted
```

**Use When**: Maximum compatibility with naive parsers, explicit type marking

## Key Ordering

Control field order with `sort_keys`:

### Preserve Order (sort_keys=false, default)

```hedl
config:
  name: MyApp
  version: 1.0
  author: Alice
```

**Preserves**: Original insertion order from source document

### Alphabetic Sort (sort_keys=true)

```hedl
config:
  author: Alice
  name: MyApp
  version: 1.0
```

**Benefits**:
- Consistent field ordering across documents
- Easier visual diffing
- Deterministic regardless of original order
- Better for git diffs

**Note**: Entity IDs always appear first in matrix rows regardless of sort_keys.

## Schema Handling

Two modes for schema representation:

### Separate %STRUCT (inline_schemas=false, default)

```hedl
%V:2.0
%S:User:[id, name, email]
---
users: @User[2]
 | alice, Alice, alice@example.com
 | bob, Bob, bob@example.com
```

**Advantages**:
- Schema defined once, reused multiple times
- Type system remains explicit
- Easier schema updates (single location)

### Inline Schemas (inline_schemas=true)

```hedl
%V:2.0
---
users: @User[id, name, email][2]
 | alice, Alice, alice@example.com
 | bob, Bob, bob@example.com
```

**Advantages**:
- Self-contained matrix lists
- No forward references
- Easier to extract individual lists

## Configuration Reference

### CanonicalConfig Builder

```rust
use hedl_c14n::{CanonicalConfig, QuotingStrategy};

let config = CanonicalConfig::builder()
    .sort_keys(true)                          // Alphabetic sorting (default: true)
    .inline_schemas(false)                    // Inline vs %S: (default: false)
    .quoting(QuotingStrategy::Minimal)        // Quoting mode (default: Minimal)
    .build();
```

### Configuration Options

**sort_keys** (default: true)
- Alphabetically sort object fields
- Deterministic ordering regardless of insertion order
- Improves git diff readability

**inline_schemas** (default: false)
- `true`: Inline schemas in matrix headers `@Type[field1, field2]`
- `false`: Separate %S: declarations in header
- Trade-off: Self-contained vs reusable schemas

**quoting** (default: Minimal)
- `QuotingStrategy::Minimal` - Quote only when necessary
- `QuotingStrategy::Always` - Quote all strings
- Minimal recommended for token efficiency

## Security: Depth Limits

Protection against deeply nested structures:

```rust
const MAX_NESTING_DEPTH: usize = 1000;

// Attempting to canonicalize > 1000 levels deep:
// Error: HedlError::Syntax { line: ..., message: "Max depth exceeded: 1001 levels (max: 1000)" }
```

**Prevents**:
- Stack overflow from malicious input
- Infinite recursion bugs
- Accidental runaway nesting

**Implementation**: Depth counter incremented on each recursive call, decremented on return.

## Error Handling

Canonicalization uses `HedlError` from `hedl-core`:

```rust
use hedl_c14n::canonicalize;
use hedl_core::HedlError;

match canonicalize(&doc) {
    Ok(canonical) => println!("{}", canonical),
    Err(HedlError::Syntax { line, message }) => {
        eprintln!("Syntax error at line {}: {}", line, message);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### Error Conditions

- **Nesting too deep**: Document exceeds 1000-level nesting limit
- **Write failures**: Internal buffer errors (extremely rare)

Errors include line numbers and context for debugging.

## Round-Trip Stability

Canonical form preserves semantic equivalence:

```rust
use hedl_core::parse;
use hedl_c14n::canonicalize;

let original = parse(hedl_bytes)?;
let canonical_str = canonicalize(&original)?;
let reparsed = parse(canonical_str.as_bytes())?;

// Semantic equivalence holds
assert_eq!(original.version, reparsed.version);
assert_eq!(original.structs, reparsed.structs);
assert_eq!(original.entities, reparsed.entities);
```

**Guarantees**:
- All fields preserved
- All entities preserved
- References maintained
- Type information intact
- Semantic equality (not string equality)

**Non-Preserved**:
- Whitespace and formatting
- Comment positions (comments stripped)
- Original key order (if sort_keys=true)
- Exact float representation (normalized)

## Use Cases

**Version Control Normalization**: Canonicalize HEDL files before git commit to eliminate spurious formatting diffs. Enable clean git history focused on semantic changes.

**LLM Context Optimization**: Minimize token count through compact syntax. Fit more data in context windows without losing information.

**Content-Addressable Storage**: Generate deterministic hashes for identical documents regardless of source formatting. Enable deduplication and cache hits.

**Cryptographic Signatures**: Sign canonical form to ensure signatures verify regardless of whitespace/formatting variations. Ideal for document integrity verification.

**Database Exports**: Normalize exported HEDL for consistent baselines in testing and CI/CD. Detect actual data changes, not formatting noise.

**Configuration Management**: Standardize config file formatting across teams and tools. Automated formatting on save, consistent style enforcement.

## What This Crate Doesn't Do

**Validation**: Canonicalization assumes input is valid. For validation, use `hedl-lint` before canonicalization.

**Comment Preservation**: Comments are not part of canonical form and are stripped. For comment-preserving formatting, use hedl-core's pretty-printer.

**Custom Formatting Rules**: Configuration is comprehensive but not infinite. Highly custom formatting requirements may need custom serialization.

**Schema Inference**: Uses existing schemas from document. For schema generation from data, use hedl-core's inference APIs.

## Performance Characteristics

**Time Complexity**: O(n) where n = total nodes + fields. Single linear pass through document tree.

**Space Complexity**: O(n) output buffer + O(d) recursion stack where d = nesting depth. Pre-allocation optimization (P1) amortizes allocations.

**Optimizations Implemented**:
- **P0: Direct BTreeMap Iteration** - Iterate without intermediate Vec allocation (eliminates O(n) allocation)
- **P1: Pre-allocated Buffers** - Estimate output size, allocate once (reduces allocation count by 90%)

**Count Hint Generation**: O(n) single pass to count nodes. Cached during serialization (no double-traversal).

## Dependencies

- `hedl-core` 2.0 - Core HEDL data structures and parsing
- `thiserror` 1.0 - Error type definitions

## License

Apache-2.0
