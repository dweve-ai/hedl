# AST Design

Understanding HEDL's Abstract Syntax Tree structure and design principles.

## Core Types

```rust
// File: crates/hedl-core/src/document.rs

/// A parsed HEDL document with header directives and body content.
pub struct Document {
    pub version: (u32, u32),
    pub schema_versions: BTreeMap<String, crate::schema_version::SchemaVersion>,
    pub aliases: BTreeMap<String, String>,
    pub structs: BTreeMap<String, Vec<String>>,
    pub nests: BTreeMap<String, String>,
    pub root: BTreeMap<String, Item>,
}

/// An item in the document body (scalar, nested object, or matrix list).
pub enum Item {
    Scalar(Value),
    Object(BTreeMap<String, Item>),
    List(MatrixList),
}

/// A typed matrix list containing structured rows.
pub struct MatrixList {
    pub type_name: String,
    pub schema: Vec<String>,
    pub rows: Vec<Node>,
    pub count_hint: Option<usize>,
}

/// A row/entity in a matrix list.
pub struct Node {
    pub type_name: String,
    pub id: String,
    pub fields: SmallVec<[Value; 4]>,  // Stack-allocated for ≤4 fields
    pub children: Option<Box<BTreeMap<String, Vec<Node>>>>,  // Lazy allocation
    pub child_count: u16,  // Compact hint (u16 saves 6 bytes)
}

/// HEDL scalar values.
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Box<str>),           // Box<str> reduces enum size vs String
    Tensor(Box<Tensor>),        // Boxed to reduce enum size
    Reference(Reference),
    Expression(Box<Expression>), // Boxed to reduce enum size
}
```

## Design Principles

### 1. Hierarchical Structure

Items can contain nested objects, forming a tree:

```hedl
%VERSION: 1.0
---
user:           # Root key
  name: Alice   # Scalar value
  profile:      # Nested object
    bio: Dev    # Nested scalar
```

Maps to:
```rust
use std::collections::BTreeMap;

Document {
    version: (1, 0),
    aliases: BTreeMap::new(),
    structs: BTreeMap::new(),
    nests: BTreeMap::new(),
    root: {
        let mut root = BTreeMap::new();
        root.insert(
            "user".to_string(),
            Item::Object({
                let mut user = BTreeMap::new();
                user.insert("name".to_string(), Item::Scalar(Value::String("Alice".into())));
                user.insert("profile".to_string(), Item::Object({
                    let mut profile = BTreeMap::new();
                    profile.insert("bio".to_string(), Item::Scalar(Value::String("Dev".into())));
                    profile
                }));
                user
            })
        );
        root
    },
}
```

### 2. Typed Values

Values have explicit types to preserve semantics:

| HEDL | AST | Notes |
|------|-----|-------|
| `42` | `Value::Int(42)` | Integer |
| `3.14` | `Value::Float(3.14)` | Float |
| `true` | `Value::Bool(true)` | Boolean |
| `"text"` | `Value::String("text")` | String |
| `~` | `Value::Null` | Null |
| `@User:alice` | `Value::Reference(...)` | Reference |
| `[1, 2, 3]` | `Value::Tensor(...)` | Tensor |

### 3. Flexible Items

`Item` enum allows scalars, nested objects, and matrix lists:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice Smith
  | bob, Bob Jones

config:
  timeout: 30
```

```rust
use std::collections::BTreeMap;

Document {
    version: (1, 0),
    aliases: BTreeMap::new(),
    structs: {
        let mut structs = BTreeMap::new();
        structs.insert("User".to_string(), vec!["id".to_string(), "name".to_string()]);
        structs
    },
    nests: BTreeMap::new(),
    root: {
        let mut root = BTreeMap::new();
        root.insert("users".to_string(), Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                Node {
                    type_name: "User".into(),
                    id: "alice".into(),
                    fields: smallvec![Value::String("Alice Smith".into())],
                    children: None,
                    child_count: 0
                },
                Node {
                    type_name: "User".into(),
                    id: "bob".into(),
                    fields: smallvec![Value::String("Bob Jones".into())],
                    children: None,
                    child_count: 0
                },
            ],
            count_hint: None,
        }));
        root.insert("config".to_string(), Item::Object({
            let mut config = BTreeMap::new();
            config.insert("timeout".to_string(), Item::Scalar(Value::Int(30)));
            config
        }));
        root
    },
}
```

## Memory Layout

(Estimates for 64-bit systems)

```
Document
├─ version: (u32, u32) (8 bytes)
├─ aliases: BTreeMap<String, String> (24 bytes + heap)
├─ structs: BTreeMap<String, Vec<String>> (24 bytes + heap)
├─ nests: BTreeMap<String, String> (24 bytes + heap)
└─ root: BTreeMap<String, Item> (24 bytes + heap)

Item (enum, ~96-128 bytes)
├─ Scalar(Value)
├─ Object(BTreeMap<String, Item>)
└─ List(MatrixList)

Node (~72-88 bytes, optimized)
├─ type_name: String (24 bytes)
├─ id: String (24 bytes)
├─ fields: SmallVec<[Value; 4]> (32 bytes inline, stack-allocated for ≤4 fields)
├─ children: Option<Box<BTreeMap<...>>> (8 bytes, lazy heap allocation)
└─ child_count: u16 (2 bytes, compact hint)

Value (enum, ~32 bytes, optimized)
├─ Null, Bool, Int, Float (small)
├─ String: Box<str> (16 bytes + heap)
├─ Reference: (~32 bytes: Option<Box<str>> + Box<str>)
├─ Tensor: Box<Tensor> (8 bytes pointer + heap)
└─ Expression: Box<Expression> (8 bytes pointer + heap)
```

**Total for small document** (~10 items): ~2-4 KB

## Traversal Patterns

### Visitor Pattern

```rust
use hedl_core::traverse::{DocumentVisitor, VisitorContext};
use hedl_core::{Document, Value, MatrixList, Node};

pub trait DocumentVisitor {
    type Error;

    fn begin_document(&mut self, doc: &Document, ctx: &VisitorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_document(&mut self, doc: &Document, ctx: &VisitorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_scalar(&mut self, key: &str, value: &Value, ctx: &VisitorContext) -> Result<(), Self::Error>;

    fn begin_object(&mut self, key: &str, ctx: &VisitorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_object(&mut self, key: &str, ctx: &VisitorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn begin_list(&mut self, key: &str, list: &MatrixList, ctx: &VisitorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_list(&mut self, key: &str, list: &MatrixList, ctx: &VisitorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_node(&mut self, node: &Node, schema: &[String], ctx: &VisitorContext) -> Result<(), Self::Error>;
    
    fn begin_node_children(&mut self, node: &Node, ctx: &VisitorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_node_children(&mut self, node: &Node, ctx: &VisitorContext) -> Result<(), Self::Error> {
        Ok(())
    }
}
```

### Recursive Walking

```rust
use hedl_core::Item;

fn count_items(item: &Item) -> usize {
    match item {
        Item::Scalar(_) => 1,
        Item::Object(map) => 1 + map.values().map(count_items).sum::<usize>(),
        Item::List(list) => 1 + list.rows.len(),
    }
}
```

## Optimization Techniques

### 1. Pre-allocation

HEDL optimizes collection growth by pre-allocating capacity for `Vec` and `BTreeMap` when sizes are known or can be estimated from input metadata.

### 2. Ordered Iteration

Using `BTreeMap` for the root and objects ensures that document traversal and serialization are always deterministic, which is required for canonicalization.

## Type Definitions

### Reference

A reference to another entity in the document using the `@Type:id` syntax.

```rust
// File: crates/hedl-core/src/value.rs

pub struct Reference {
    /// Optional type qualifier (e.g., "User" in "@User:id").
    /// Boxed to reduce size when None (common case).
    pub type_name: Option<Box<str>>,
    /// The ID being referenced.
    /// Uses Box<str> for compact representation (16 bytes vs 24 for String).
    pub id: Box<str>,
}
```

**Key methods:**
- `local(id)` - Create a local reference (no type qualifier)
- `qualified(type_name, id)` - Create a qualified reference
- `unqualified(id)` - Alias for `local()`
- `to_ref_string()` - Format as a reference string (with @)

**Examples:**
```rust
let r1 = Reference::local("alice");
// r1: Reference { type_name: None, id: "alice" }

let r2 = Reference::qualified("User", "alice");
// r2: Reference { type_name: Some("User"), id: "alice" }

assert_eq!(r1.to_ref_string(), "@alice");
assert_eq!(r2.to_ref_string(), "@User:alice");
```

### Tensor

A multi-dimensional numerical array stored as nested vectors.

```rust
// File: crates/hedl-core/src/lex/tensor.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Tensor {
    /// A scalar number (integer or float).
    Scalar(f64),
    /// A nested array of tensors.
    Array(Vec<Tensor>),
}
```

**Key methods:**
- `shape()` - Returns dimensions as `Vec<usize>`
- `flatten()` - Converts to flat `Vec<f64>` in row-major order
- `is_integer()` - Checks if all values are integers
- `is_scalar()` / `is_array()` - Type checks
- `ndim()` - Number of dimensions
- `len()` / `is_empty()` - Element counts

**Examples:**
```rust
use hedl_core::lex::parse_tensor;

// Scalar
let t = parse_tensor("42").unwrap();
assert_eq!(t.shape(), vec![]);

// 1D vector
let t = parse_tensor("[1, 2, 3]").unwrap();
assert_eq!(t.shape(), vec![3]);
assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0]);

// 2D matrix
let t = parse_tensor("[[1, 2], [3, 4]]").unwrap();
assert_eq!(t.shape(), vec![2, 2]);
assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0, 4.0]);
```

**Security limits:**
- Max recursion depth: 100
- Max elements: 10,000,000
- Rejects NaN and Infinity values

### Expression

AST for `$(...)` expressions with function calls and field access.

```rust
// File: crates/hedl-core/src/lex/expression.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// A literal value: number, string, or boolean.
    Literal { value: ExprLiteral, span: Span },
    /// An identifier: `foo`, `bar_baz`.
    Identifier { name: String, span: Span },
    /// A function call: `func(arg1, arg2)`.
    Call {
        name: String,
        args: Vec<Expression>,
        span: Span,
    },
    /// Field access: `target.field`.
    Access {
        target: Box<Expression>,
        field: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprLiteral {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}
```

**Examples:**
```rust
use hedl_core::lex::parse_expression;

// Identifier
let expr = parse_expression("foo").unwrap();

// Function call
let expr = parse_expression("upper(name)").unwrap();

// Field access
let expr = parse_expression("user.name").unwrap();

// Chained access
let expr = parse_expression("user.profile.bio").unwrap();

// Nested calls
let expr = parse_expression("concat(upper(first), last)").unwrap();
```

### VisitorContext

Context provided during document traversal for format conversion.

```rust
// File: crates/hedl-core/src/traverse.rs

#[derive(Debug, Clone)]
pub struct VisitorContext<'a> {
    /// Current nesting depth (0 = root level).
    pub depth: usize,
    /// Path from root to current element (key names).
    pub path: Vec<&'a str>,
    /// Reference to the document being traversed.
    pub document: &'a Document,
    /// Schema for the current list (if within a list context).
    pub current_schema: Option<&'a [String]>,
}
```

**Methods:**
- `new(document)` - Create root context
- `child(key)` - Create child context with incremented depth
- `with_schema(schema)` - Add schema context for list traversal
- `path_string()` - Get current path as string (for error messages)

**Usage:**
```rust
use hedl_core::traverse::{DocumentVisitor, traverse, VisitorContext};

impl DocumentVisitor for MyConverter {
    type Error = String;

    fn visit_scalar(&mut self, key: &str, value: &Value, ctx: &VisitorContext)
        -> Result<(), Self::Error>
    {
        println!("At depth {}: {} = {:?}", ctx.depth, key, value);
        println!("Path: {}", ctx.path_string());
        Ok(())
    }
    // ... other methods
}
```

### SchemaVersion

Semantic versioning for schema evolution.

```rust
// File: crates/hedl-core/src/schema_version.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
```

**Methods:**
- `new(major, minor, patch)` - Create version
- `v1()` - Returns version 1.0.0
- `parse(s)` - Parse from string like "1.2.3"
- `is_compatible_with(other)` - Check backward compatibility
- `is_breaking_from(other)` - Check if major version changed

**Compatibility rules:**
- Same major version required
- Reader's minor >= writer's minor
- Patch versions don't affect compatibility

**Examples:**
```rust
use hedl_core::schema_version::SchemaVersion;

let v1_0 = SchemaVersion::new(1, 0, 0);
let v1_1 = SchemaVersion::new(1, 1, 0);
let v2_0 = SchemaVersion::new(2, 0, 0);

// v1.1 can read v1.0 data (backward compatible)
assert!(v1_1.is_compatible_with(&v1_0));

// v1.0 cannot read v1.1 data (missing new fields)
assert!(!v1_0.is_compatible_with(&v1_1));

// Different major versions are incompatible
assert!(!v2_0.is_compatible_with(&v1_0));

// Parse from string
let v = SchemaVersion::parse("1.2.3").unwrap();
assert_eq!(v.to_string(), "1.2.3");
```

## Related

- [Parser Architecture](parser-architecture.md)
- [Zero-Copy Optimizations](zero-copy-optimizations.md)
