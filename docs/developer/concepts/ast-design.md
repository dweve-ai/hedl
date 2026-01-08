# AST Design

Understanding HEDL's Abstract Syntax Tree structure and design principles.

## Core Types

```rust
// File: crates/hedl-core/src/document.rs

/// A parsed HEDL document with header directives and body content.
pub struct Document {
    pub version: (u32, u32),
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
    pub fields: Vec<Value>,
    pub children: BTreeMap<String, Vec<Node>>,
    pub child_count: Option<usize>,
}

/// HEDL scalar values.
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
                    type_name: "User".to_string(),
                    id: "alice".to_string(),
                    fields: vec![Value::String("Alice Smith".to_string())],
                    children: BTreeMap::new(),
                    child_count: None
                },
                Node {
                    type_name: "User".to_string(),
                    id: "bob".to_string(),
                    fields: vec![Value::String("Bob Jones".to_string())],
                    children: BTreeMap::new(),
                    child_count: None
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

Node (~128-160 bytes)
├─ type_name: String (24 bytes)
├─ id: String (24 bytes)
├─ fields: Vec<Value> (24 bytes + heap)
├─ children: BTreeMap<String, Vec<Node>> (24 bytes + heap)
└─ child_count: Option<usize> (16 bytes)

Value (enum, ~64-80 bytes)
├─ Null, Bool, Int, Float (small)
├─ String: (24 bytes + heap)
├─ Reference: (56 bytes + heap)
└─ Tensor, Expression (recursive/boxed)
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

## Related

- [Parser Architecture](parser-architecture.md)
- [Zero-Copy Optimizations](zero-copy-optimizations.md)
