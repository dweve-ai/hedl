# hedl-core

The parsing engine for HEDL (Hierarchical Entity Data Language).

## Why HEDL exists

Consider a dataset with 50,000 user records. In JSON, each object repeats the same keys: `{"id": ..., "name": ..., "email": ...}`. The parser processes those keys fifty thousand times, and roughly 40% of your file is redundant key names.

HEDL separates structure from data. You declare the schema once in the header, then write rows of values. The parser knows the column names upfront, which means less parsing work and smaller files. For entity-heavy datasets (users, products, events), this matters.

## What this crate does

`hedl-core` parses HEDL text into typed Rust values. It handles the complete syntax: headers, matrix rows, nested objects, entity references, and tensor literals.

```rust
use hedl_core::{parse, Document};

let hedl = r#"
%S:User:[id, name, email]
---
users: @User
 | alice, Alice Smith, alice@example.com
 | bob, Bob Jones, bob@example.com
"#;

let doc = parse(hedl.as_bytes())?;
```

Values come out typed. A reference like `@User:alice` parses to `Value::Reference`, not a string. A list like `[1, 2, 3]` parses to `Value::Tensor`. Pattern matching gives you compile-time guarantees about what you're handling.

## Schema-aware matrices

HEDL's core feature is typed tabular data. You define schemas in the header, then use them to structure your data:

```rust
let hedl = r#"
%S:Post:[id, author, title]
%S:Comment:[id, post, author, text]
---
posts: @Post
 | p1, @User:alice, First Post
 | p2, @User:bob, Second Post

comments: @Comment
 | c1, @Post:p1, @User:bob, Great post!
 | c2, @Post:p1, @User:alice, Thanks!
"#;

let doc = parse(hedl.as_bytes())?;

// Each row maps to the schema columns
// References like @Post:p1 are parsed as Reference values, not strings
```

The schema association (`posts: @Post`) tells the parser that each row follows the `Post` schema. Fields are accessed by column name, not by parsing key-value pairs repeatedly.

## Entity references

References are first-class values. When you write `@User:alice`, the parser produces a `Reference` with the entity type ("User") and identifier ("alice") already separated. Building graphs or resolving relationships doesn't require string parsing at runtime.

## Tensor literals

Arrays parse directly to `Value::Tensor`:

```rust
let hedl = r#"
---
metrics: @Metric[id, values, percentiles]
 | m1, 1250, [850, 1100, 1400, 2100, 3500]
 | m2, 890, [620, 780, 920, 1200, 1800]
"#;

let doc = parse(hedl.as_bytes())?;
// The [850, 1100, ...] values are Tensor types, ready for numerical work
```

## Data model

After parsing:

**`Document`** holds the header metadata and a map of named entries.

**`Item`** represents a body entry: a scalar value, a nested object, or a matrix list.

**`MatrixList`** contains schema information and rows of data.

**`Node`** represents one row with typed fields and optional child nodes.

**`Value`** is a tagged enum: Null, Bool, Int, Float, String, Tensor, Reference, Expression.

**`Reference`** holds an entity type and identifier, parsed from `@Type:id` syntax.

## Error handling

Parse errors include location and context:

```rust
use hedl_core::{parse, HedlError, HedlErrorKind};

match parse(data) {
    Ok(doc) => { /* use the document */ },
    Err(e) => {
        eprintln!("Error at line {}: {}", e.line, e.message);
        match e.kind {
            HedlErrorKind::Reference => { /* unresolved reference */ },
            HedlErrorKind::Schema => { /* schema mismatch */ },
            HedlErrorKind::Syntax => { /* malformed input */ },
            _ => { /* other issues */ }
        }
    }
}
```

The parser never panics on invalid input. All failures return structured errors.

## When to use this crate

For most tasks, use higher-level tools: the `hedl` facade crate for a convenient API, `hedl-cli` for command-line operations, or `hedl-lsp` for editor integration.

Use `hedl-core` directly when building custom tooling (linters, converters, analyzers), embedding HEDL parsing in a larger application, or when you need direct access to the parsed AST without abstraction layers.

## Safety

The crate uses minimal unsafe code in two internal modules: string interning and arena allocation. The public API is safe Rust. All errors return `Result` values with diagnostics.

Report security issues privately before public disclosure.

## License

Apache-2.0
