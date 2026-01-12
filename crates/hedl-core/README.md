# hedl-core

**The parsing engine that makes HEDL work.**

Every format needs a foundation. JSON has parsers that handle `{}` and `[]`. YAML wrestles with indentation. XML navigates angle brackets. HEDL's foundation is `hedl-core` - a parser designed from scratch for AI-era data: typed matrices, entity references, and tensor literals built into the format itself.

## Why hedl-core Exists

When you're processing 100GB datasets for ML training, or serving thousands of API requests per second, parser performance isn't academic—it's operational. `hedl-core` was built for this: deterministic parsing with fail-fast error handling, zero-copy preprocessing for minimal allocations, and a data model that maps directly to how AI systems think about data.

## What Makes It Different

**Schema-Aware Matrices**: Most formats see tabular data as "arrays of objects." HEDL sees them as typed matrices with column definitions—the difference between `[{}, {}, {}]` and `@User[id,name,email]`. The parser understands this natively.

**Entity References**: When you write `@User:alice`, the parser knows it's a reference, not a string. Graph relationships are first-class

 citizens, not workarounds.

**Zero-Copy Where It Counts**: Line offset tables and careful memory layout mean the parser allocates only what it needs. Less allocation pressure means more predictable performance.

**Fail-Fast Philosophy**: Invalid syntax? You get an error at parse time with the exact line number. No silent failures, no "undefined" cascading through your system.

## Installation

```toml
[dependencies]
hedl-core = "1.0"
```

## Usage

Parse HEDL documents into a structured data model:

```rust
use hedl_core::{parse, Document, Value};

let hedl = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email, created]
---
users: @User
  | alice, Alice Smith, alice@example.com, 2024-01-15
  | bob, Bob Jones, bob@example.com, 2024-02-20
"#;

let doc = parse(hedl.as_bytes())?;

// The data model preserves structure
if let Some(users) = doc.get("users") {
    if let Some(matrix) = users.as_matrix_list() {
        println!("Schema: {:?}", matrix.schema());
        println!("Rows: {}", matrix.rows().len());
    }
}
```

Work with references and relationships:

```rust
let hedl = r#"
%VERSION: 1.0
%STRUCT: Post: [id, author, title]
%STRUCT: Comment: [id, post, author, text]
---
posts: @Post
  | p1, @User:alice, First Post
  | p2, @User:bob, Second Post

comments: @Comment
  | c1, @Post:p1, @User:bob, Great post!
  | c2, @Post:p1, @User:alice, Thanks!
"#;

let doc = parse(hedl.as_bytes())?;

// References are parsed as Reference values, not strings
// Traverse the graph structure naturally
```

Handle tensors for ML workflows:

```rust
let hedl = r#"
%VERSION: 1.0
---
metrics: @Metric[id, values, percentiles]
  | m1, 1250, [850, 1100, 1400, 2100, 3500]
  | m2, 890, [620, 780, 920, 1200, 1800]
"#;

let doc = parse(hedl.as_bytes())?;

// Tensor literals are native Value::Array types
// No string parsing needed
```

## Data Model

The parsed document exposes a clean, typed API:

- **`Document`** - Root container with version header and content map
- **`Node`** - Key-value entries in objects (like JSON object properties)
- **`Value`** - Tagged enum: Scalar (string/number/bool/null), Object (nested), Array (tensor), Reference, MatrixList
- **`MatrixList`** - Schema-defined tabular data with typed columns
- **`Reference`** - Typed entity pointers (`@Type:id` syntax)

## Features

**Deterministic Parsing**: Same input always produces the same AST. No locale-dependent behavior, no timestamp drift, no platform quirks.

**Zero-Copy Preprocessing**: Line offset tables let the parser jump to any line without scanning the entire file. Parse the schema header, skip to the data section—no wasted work.

**Type Safety**: The data model uses Rust enums, not stringly-typed polymorphism. `Value::Reference` is distinct from `Value::Scalar`. Pattern matching gives you compile-time guarantees.

**Error Messages That Help**: Parse errors include line numbers, column positions, and context. No cryptic "unexpected token" messages.

**Extensible Design**: The core parser handles syntax. Higher-level crates add semantic validation, reference resolution, and format conversion.

## What hedl-core Doesn't Do

It's a parser, not a Swiss Army knife:

- **No validation**: Schema compliance checking is in `hedl-lint`
- **No resolution**: Cross-reference verification is in `hedl-core`'s validation layer
- **No conversion**: Format translation is in `hedl-json`, `hedl-yaml`, etc.
- **No I/O**: Streaming and network protocols are in `hedl-stream`

This separation means you can parse a 10GB file without loading validation rules, or validate a document without network-dependent resolution.

## Performance Characteristics

- **Memory**: Proportional to document structure, not text size. A 1MB file with 1000 rows allocates for 1000 matrix entries, not 1,000,000 characters.
- **Speed**: Parsing is I/O-bound for most documents. The bottleneck is reading bytes, not processing them.
- **Allocation**: Zero-copy line tables and string interning reduce heap pressure.

## When to Use hedl-core Directly

Most developers use higher-level crates (`hedl` facade crate, `hedl-cli` tool, `hedl-lsp` editor support). You'd use `hedl-core` directly when:

- **Building custom tooling**: Format converters, linters, analysis tools
- **Embedding HEDL parsing**: In a larger application that needs low-level control
- **Performance-critical paths**: When you need zero-abstraction parsing

If you're just converting formats or validating files, use `hedl-cli`. If you need autocomplete in your editor, use `hedl-lsp`. If you're building the next HEDL tool, start here.

## License

Apache-2.0 — Use it, fork it, build on it.
