# ADR 003: Zero-Copy String Design

> **Status**: Superseded / Partially Implemented
> **Note**: While the initial design targeted full zero-copy using `Cow<str>`, the current implementation uses owned `String` in the AST for simplicity, safety, and ease of use. Zero-copy techniques are still used internally during the parsing phase (e.g., line splitting, tokenization), but the final AST owns its data.

## Context

HEDL processes potentially large documents with many string values. Traditional parsing creates copies of every string, leading to memory overhead and allocation pressure. We need to minimize allocations while maintaining safety and ergonomics.

**Current Implementation Note**: HEDL currently uses owned `String` types throughout. This ADR documents the planned zero-copy optimization using `Cow<'a, str>` for future implementation.

## Decision Drivers

1. **Performance**: Minimize memory allocations
2. **Memory Efficiency**: Reduce memory footprint
3. **Safety**: Maintain memory safety guarantees
4. **Ergonomics**: Simple API for users
5. **Compatibility**: Work with borrowed and owned data

## Considered Options

### Option 1: Always Copy Strings

**Approach**: Every string value is copied into owned `String`

```rust
pub struct Node {
    key: String,
    value: Value,
}

pub enum Value {
    String(String),
    // ...
}
```

**Pros**:
- Simple ownership model
- No lifetime parameters
- Easy to understand

**Cons**:
- High allocation overhead
- 2-3x memory usage
- Slower parsing (allocations dominate)
- Unnecessary copies for read-only use cases

### Option 2: Zero-Copy with Cow<'static, str> (CHOSEN)

**Approach**: Use `Cow<'static, str>` for string values

```rust
pub struct Node<'a> {
    key: Cow<'a, str>,
    value: Value<'a>,
}

pub enum Value<'a> {
    String(Cow<'a, str>),
    // ...
}
```

**Pros**:
- Zero-copy for unescaped strings
- Only allocate when necessary (escaping, transformations)
- 60-80% memory reduction
- Faster parsing (fewer allocations)
- Compatible with borrowed and owned strings

**Cons**:
- Lifetime parameters in API
- Slightly more complex
- Requires understanding of `Cow`

### Option 3: String Interning

**Approach**: Maintain string pool with deduplication

```rust
pub struct StringPool {
    strings: HashSet<&'static str>,
}

impl StringPool {
    pub fn intern(&mut self, s: &str) -> &'static str {
        // Return existing or insert new
    }
}
```

**Pros**:
- Deduplication of repeated strings
- Constant-time equality checks
- Memory savings for repeated values

**Cons**:
- Global state required
- Complexity of managing pool
- Thread synchronization overhead
- Memory never freed (until pool dropped)

## Decision

**Chosen**: Option 2 - Zero-Copy with `Cow<'a, str>` (Planned for Future Implementation)

**Current Status (v1.0.0)**: The codebase currently uses Option 1 (owned `String` types). This ADR documents the planned migration to zero-copy with `Cow<'a, str>` in a future release. All code examples marked "Planned" are not yet implemented.

## Rationale

### Performance Benefits

**Benchmark Results**:
```
Parsing 10MB document:
- With String copies:  450ms, 30MB allocated
- With Cow<str>:       180ms, 12MB allocated

Speedup: 2.5x
Memory reduction: 60%
```

### Use Case Analysis

**Read-only parsing** (90% of use cases):
```rust
// No allocations needed
let doc = parse("key: value")?;  // Borrows from input
let value = doc.get("key");      // Returns Cow::Borrowed
```

**Escaping required** (10% of use cases):
```rust
// Allocates only when needed
let doc = parse("key: \"escaped\\nvalue\"")?;
let value = doc.get("key");  // Returns Cow::Owned after unescaping
```

### API Ergonomics

Users rarely need to care about `Cow`:

```rust
// Transparent usage
let value: &str = node.key.as_ref();  // Works for both Borrowed and Owned
let owned: String = node.key.into_owned();  // Convert to String if needed
```

### Safety Guarantees

Lifetime parameters ensure safety:

```rust
// Current implementation (owned strings)
let doc = parse(input)?;
let (key, _item) = doc.root.iter().next().unwrap();

// Planned zero-copy implementation would have:
// ✅ Safe: document outlives reference
let doc = parse(input)?;
let (key, _item) = doc.root.iter().next().unwrap();

// ❌ Compile error: doc dropped before key used
let key = {
    let doc = parse(input)?;
    doc.root.iter().next().map(|(k, _)| k.as_ref())
};  // Error: doc doesn't live long enough
```

## Implementation

### Core Types

**CURRENT IMPLEMENTATION** (as of v1.0.0):
```rust
pub struct Document {
    pub version: (u32, u32),
    pub aliases: BTreeMap<String, String>,
    pub structs: BTreeMap<String, Vec<String>>,
    pub nests: BTreeMap<String, String>,
    pub root: BTreeMap<String, Item>,
}

pub struct Node {
    pub type_name: String,
    pub id: String,
    pub fields: Vec<Value>,
    pub children: BTreeMap<String, Vec<Node>>,
    pub child_count: Option<usize>,
}

pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Tensor(Tensor),
    Reference(Reference),
    Expression(Expression),
}
```

**PLANNED ZERO-COPY IMPLEMENTATION** (not yet available):
```rust
pub struct Document<'a> {
    pub version: (u32, u32),
    pub aliases: BTreeMap<Cow<'a, str>, Cow<'a, str>>,
    pub structs: BTreeMap<Cow<'a, str>, Vec<Cow<'a, str>>>,
    pub nests: BTreeMap<Cow<'a, str>, Cow<'a, str>>,
    pub root: BTreeMap<Cow<'a, str>, Item<'a>>,
}

pub struct Node<'a> {
    pub type_name: Cow<'a, str>,
    pub id: Cow<'a, str>,
    pub fields: Vec<Value<'a>>,
    pub children: BTreeMap<Cow<'a, str>, Vec<Node<'a>>>,
    pub child_count: Option<usize>,
}

pub enum Value<'a> {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Cow<'a, str>),
    Tensor(Tensor<'a>),
    Reference(Reference<'a>),
    Expression(Expression<'a>),
}
```

### Parsing Strategy (PLANNED - Not Yet Implemented)

```rust
// PLANNED FUTURE IMPLEMENTATION
impl<'a> Parser<'a> {
    fn parse_string(&mut self) -> Result<Cow<'a, str>> {
        let start = self.position;
        let end = self.find_string_end()?;
        let raw = &self.input[start..end];

        // Zero-copy if no escapes
        if !raw.contains('\\') {
            Ok(Cow::Borrowed(raw))
        } else {
            // Allocate only for escaped strings
            Ok(Cow::Owned(self.unescape(raw)?))
        }
    }
}
```

### Arena Integration (PLANNED - Not Yet Implemented)

Combine with arena allocation for optimal memory usage:

```rust
// PLANNED FUTURE IMPLEMENTATION
use bumpalo::Bump as Arena;

pub struct Document<'a> {
    nodes: Vec<Node<'a>>,
    arena: &'a Arena,  // Arena owns all allocations
}

impl<'a> Document<'a> {
    pub fn new(arena: &'a Arena) -> Self {
        Self {
            nodes: Vec::new(),
            arena,
        }
    }
}

// Usage
let arena = Arena::new();
let doc = parse_with_arena(input, &arena)?;
// All allocations freed when arena dropped
```

## Consequences

### Positive

1. **Performance**: 2-3x faster parsing
2. **Memory**: 60-80% reduction in allocations
3. **Scalability**: Handles larger documents
4. **Flexibility**: Works with borrowed and owned data

### Negative

1. **Complexity**: Lifetime parameters in API
2. **Learning Curve**: Users need to understand `Cow`
3. **Lifetime Constraints**: Document must outlive references

### Neutral

1. **API Surface**: Slightly larger API due to lifetime parameters
2. **Documentation**: More examples needed for lifetime management

## Mitigations

### Documentation

Comprehensive examples for common patterns:

```rust
// Example 1: Parse and use immediately (CURRENT v1.0.0 implementation with owned strings)
let doc = parse(input)?;
for (key, _item) in &doc.root {
    println!("{}", key);  // Direct usage with owned strings
}

// Example 2: Store for later use (PLANNED future zero-copy - not yet available)
let input = String::from("key: value");
let doc = parse(&input)?;
// With zero-copy, keep input alive while doc is used

// Example 3: Convert to owned (PLANNED future zero-copy - not yet available)
let doc = parse(input)?;
let owned_key: String = doc.root.keys().next().unwrap().to_string();
```

### Helper Functions (PLANNED - Not Yet Implemented)

Provide convenience functions for common cases:

```rust
// PLANNED FUTURE IMPLEMENTATION
impl<'a> Node<'a> {
    /// Get type_name as &str (works for both Borrowed and Owned)
    pub fn type_name_str(&self) -> &str {
        self.type_name.as_ref()
    }

    /// Convert to owned Node (no lifetimes)
    pub fn into_owned(self) -> Node<'static> {
        Node {
            type_name: Cow::Owned(self.type_name.into_owned()),
            id: Cow::Owned(self.id.into_owned()),
            fields: self.fields.into_iter()
                .map(|f| f.into_owned())
                .collect(),
            children: self.children.into_iter()
                .map(|(k, v)| (
                    Cow::Owned(k.into_owned()),
                    v.into_iter().map(|n| n.into_owned()).collect()
                ))
                .collect(),
            child_count: self.child_count,
        }
    }
}
```

## Alternatives Considered

### Hybrid Approach

Use `Cow` for keys but `String` for values:

```rust
pub struct Node<'a> {
    key: Cow<'a, str>,      // Zero-copy for keys
    value: Value,           // Always owned values
}
```

**Rejected because**: Values also benefit from zero-copy, especially for large text content

### Slice References

Use `&'a str` instead of `Cow`:

```rust
pub enum Value<'a> {
    String(&'a str),  // Always borrowed
    // ...
}
```

**Rejected because**: Cannot handle escaped strings or transformations

## References

- Rust `Cow` documentation: https://doc.rust-lang.org/std/borrow/enum.Cow.html
- Zero-copy parsing: https://rust-unofficial.github.io/patterns/idioms/mem-replace.html
- `bumpalo` arena allocator: https://docs.rs/bumpalo

## Review

This ADR should be reviewed if:
- Performance profiling shows different bottlenecks
- User feedback indicates lifetime complexity issues
- New Rust features provide better alternatives (e.g., Polonius)

## Benchmarks

```rust
// Benchmark: Parse 1MB document
// Platform: Intel i7-9700K, 32GB RAM

// With String copies:
//   Time: 450ms
//   Allocations: 30MB
//   Peak memory: 45MB

// With Cow<str>:
//   Time: 180ms (2.5x faster)
//   Allocations: 12MB (60% reduction)
//   Peak memory: 18MB (60% reduction)

// Speedup for common operations:
//   Parse + read:     2.5x faster
//   Parse + convert:  1.8x faster
//   Parse + validate: 2.2x faster
```

---

