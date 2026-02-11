# AST Design: The Shape of Data

When you parse a HEDL document, what do you get? Not the original text. Not a stream of tokens. You get an Abstract Syntax Tree: a data structure that captures the meaning of the document in a form programs can work with.

The AST is where text becomes data. Every design decision here ripples through the entire system. The types you choose, the relationships you model, the trade-offs you make: they all shape what's possible and what's efficient.

This document explains HEDL's AST design: what the types are, why they're structured this way, and how to work with them effectively.

```
╔═══════════════════════════════════════════════════════════════════╗
║                    FROM TEXT TO TREE                              ║
╠═══════════════════════════════════════════════════════════════════╣
║                                                                   ║
║   Text:                          AST:                            ║
║                                                                   ║
║   user:                          Document                        ║
║    name: Alice                    └── root                       ║
║    profile:                           └── "user" → Object        ║
║     bio: Developer                        ├── "name" → Scalar    ║
║                                           │     └── String("Alice")
║                                           └── "profile" → Object ║
║                                                 └── "bio" → Scalar║
║                                                       └── String("Developer")
║                                                                   ║
║   Same information, different representation.                    ║
║   Text is for humans. AST is for programs.                       ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
```

---

## The Core Types

### Document: The Root of Everything

Every parsed HEDL document becomes a `Document`:

```rust
pub struct Document {
    /// Version tuple: (1, 3) means v2.0
    pub version: (u32, u32),

    /// Schema version overrides from %SV headers
    pub schema_versions: BTreeMap<String, SchemaVersion>,

    /// Alias definitions from %A headers
    pub aliases: BTreeMap<String, String>,

    /// Schema definitions from %S headers
    pub structs: BTreeMap<String, Vec<String>>,

    /// Nesting relationships from %N headers
    pub nests: BTreeMap<String, String>,

    /// The actual content: key-value pairs at the root level
    pub root: BTreeMap<String, Item>,
}
```

Notice the `BTreeMap` everywhere. Not `HashMap`. Why?

```
┌─────────────────────────────────────────────────────────────────┐
│                    WHY BTREEMAP?                                │
│                                                                 │
│  HashMap:                                                       │
│  ├── O(1) lookup (faster)                                      │
│  ├── Random iteration order                                    │
│  └── Non-deterministic: same data, different order each run    │
│                                                                 │
│  BTreeMap:                                                      │
│  ├── O(log n) lookup (still fast enough)                       │
│  ├── Sorted iteration order                                    │
│  └── Deterministic: same data, same order, always             │
│                                                                 │
│  HEDL needs determinism for:                                    │
│  • Canonicalization (same AST = same output)                   │
│  • Testing (predictable iteration)                             │
│  • Diffing (meaningful comparisons)                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Item: The Three Faces of Data

An `Item` represents a value in the document body. It can be three things:

```rust
pub enum Item {
    /// A single value: number, string, boolean, etc.
    Scalar(Value),

    /// A nested object with key-value pairs
    Object(BTreeMap<String, Item>),

    /// A typed matrix list with structured rows
    List(MatrixList),
}
```

This three-way split reflects HEDL's data model:

```
┌─────────────────────────────────────────────────────────────────┐
│                    THE THREE ITEM TYPES                         │
│                                                                 │
│  SCALAR                                                         │
│  └── A single value                                            │
│      name: Alice                                                │
│      count: 42                                                  │
│      active: true                                               │
│                                                                 │
│  OBJECT                                                         │
│  └── A container of key-value pairs                            │
│      user:                                                      │
│       name: Alice                                               │
│       age: 30                                                   │
│                                                                 │
│  LIST (MatrixList)                                              │
│  └── A typed collection of structured rows                     │
│      users:@User                                                │
│       |alice,Alice,alice@example.com                           │
│       |bob,Bob,bob@example.com                                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Value: The Atomic Types

At the leaves of the tree, you find `Value`:

```rust
pub enum Value {
    Null,                        // ~
    Bool(bool),                  // true, false
    Int(i64),                    // 42, -17
    Float(f64),                  // 3.14, 1.5e10
    String(Box<str>),            // "hello", unquoted
    Tensor(Box<Tensor>),         // [1,2,3], [[1,2],[3,4]]
    Reference(Reference),        // @User:alice, @bob
    Expression(Box<Expression>), // $(func(arg))
}
```

Why `Box<str>` instead of `String`? Why `Box<Tensor>` instead of `Tensor`?

```
┌─────────────────────────────────────────────────────────────────┐
│                    ENUM SIZE OPTIMIZATION                       │
│                                                                 │
│  The size of an enum is the size of its largest variant.       │
│                                                                 │
│  Without boxing:                                                │
│  enum Value {                                                   │
│      Null,                   // 0 bytes payload                │
│      Int(i64),               // 8 bytes payload                │
│      String(String),         // 24 bytes payload (ptr+len+cap) │
│      Tensor(Tensor),         // 48+ bytes payload              │
│  }                                                              │
│  Total: 48+ bytes for EVERY Value, even simple ones            │
│                                                                 │
│  With boxing:                                                   │
│  enum Value {                                                   │
│      Null,                   // 0 bytes payload                │
│      Int(i64),               // 8 bytes payload                │
│      String(Box<str>),       // 16 bytes payload (ptr+len)     │
│      Tensor(Box<Tensor>),    // 8 bytes payload (ptr only)     │
│  }                                                              │
│  Total: ~32 bytes per Value                                    │
│                                                                 │
│  For a document with 1 million values, that's ~16 MB saved.    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### MatrixList: Structured Collections

Matrix lists are HEDL's power feature. They represent typed, structured data efficiently:

```rust
pub struct MatrixList {
    /// The type name (e.g., "User")
    pub type_name: String,

    /// Column names from the schema
    pub schema: Vec<String>,

    /// The actual rows/entities
    pub rows: Vec<Node>,

    /// Optional hint for row count (for pre-allocation)
    pub count_hint: Option<usize>,
}
```

### Node: Rows with Identity

Each row in a matrix list is a `Node`:

```rust
pub struct Node {
    /// The type this node belongs to
    pub type_name: String,

    /// The unique identifier (first column)
    pub id: String,

    /// Field values (second column onward)
    pub fields: SmallVec<[Value; 4]>,

    /// Children, if this node has nested rows (via %N)
    pub children: Option<Box<BTreeMap<String, Vec<Node>>>>,

    /// Count of children (compact hint)
    pub child_count: u16,
}
```

Notice `SmallVec<[Value; 4]>`. Most rows have 1-4 fields. SmallVec stores up to 4 values inline (on the stack), avoiding heap allocation for the common case.

---

## Mapping HEDL to AST

Let's see how a complete document maps to the AST:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
%A:active=true
---
users:@User
 |alice,Alice,alice@example.com
 |bob,Bob,bob@example.com

config:
 timeout: 30
 enabled: $active
```

Becomes:

```rust
Document {
    version: (1, 3),
    schema_versions: {},
    aliases: {
        "active" => "true"
    },
    structs: {
        "User" => ["id", "name", "email"]
    },
    nests: {},
    root: {
        "users" => Item::List(MatrixList {
            type_name: "User",
            schema: ["id", "name", "email"],
            rows: [
                Node {
                    type_name: "User",
                    id: "alice",
                    fields: [
                        Value::String("Alice"),
                        Value::String("alice@example.com")
                    ],
                    children: None,
                    child_count: 0
                },
                Node {
                    type_name: "User",
                    id: "bob",
                    fields: [
                        Value::String("Bob"),
                        Value::String("bob@example.com")
                    ],
                    children: None,
                    child_count: 0
                }
            ],
            count_hint: None
        }),
        "config" => Item::Object({
            "timeout" => Item::Scalar(Value::Int(30)),
            "enabled" => Item::Scalar(Value::Bool(true))  // Alias expanded
        })
    }
}
```

---

## Memory Layout

Understanding memory layout helps when optimizing for large documents:

```
┌─────────────────────────────────────────────────────────────────┐
│                    MEMORY LAYOUT                                │
│                                                                 │
│  Document (~128 bytes + heap)                                   │
│  ├── version: 8 bytes                                          │
│  ├── schema_versions: 24 bytes + heap                          │
│  ├── aliases: 24 bytes + heap                                  │
│  ├── structs: 24 bytes + heap                                  │
│  ├── nests: 24 bytes + heap                                    │
│  └── root: 24 bytes + heap                                     │
│                                                                 │
│  Item (~32-128 bytes)                                           │
│  ├── Scalar: 32 bytes (discriminant + Value)                   │
│  ├── Object: 32 bytes (discriminant + BTreeMap header)         │
│  └── List: ~96 bytes (discriminant + MatrixList)               │
│                                                                 │
│  Node (~72-88 bytes)                                            │
│  ├── type_name: 24 bytes                                       │
│  ├── id: 24 bytes                                              │
│  ├── fields: 32 bytes (SmallVec inline storage)                │
│  ├── children: 8 bytes (Option<Box<...>>)                      │
│  └── child_count: 2 bytes                                      │
│                                                                 │
│  Value (~32 bytes)                                              │
│  ├── Null/Bool: ~1 byte payload                                │
│  ├── Int: 8 bytes payload                                      │
│  ├── Float: 8 bytes payload                                    │
│  ├── String: 16 bytes (Box<str>)                               │
│  ├── Tensor: 8 bytes (Box pointer)                             │
│  ├── Reference: ~32 bytes                                      │
│  └── Expression: 8 bytes (Box pointer)                         │
│                                                                 │
│  Small document (10 items): ~2-4 KB                            │
│  Medium document (1000 items): ~200-400 KB                     │
│  Large document (100,000 items): ~20-40 MB                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Traversing the AST

### The Visitor Pattern

HEDL provides a visitor pattern for walking documents:

```rust
pub trait DocumentVisitor {
    type Error;

    /// Called at the start of traversal
    fn begin_document(&mut self, doc: &Document, ctx: &VisitorContext)
        -> Result<(), Self::Error> { Ok(()) }

    /// Called at the end of traversal
    fn end_document(&mut self, doc: &Document, ctx: &VisitorContext)
        -> Result<(), Self::Error> { Ok(()) }

    /// Called for each scalar value
    fn visit_scalar(&mut self, key: &str, value: &Value, ctx: &VisitorContext)
        -> Result<(), Self::Error>;

    /// Called when entering a nested object
    fn begin_object(&mut self, key: &str, ctx: &VisitorContext)
        -> Result<(), Self::Error> { Ok(()) }

    /// Called when leaving a nested object
    fn end_object(&mut self, key: &str, ctx: &VisitorContext)
        -> Result<(), Self::Error> { Ok(()) }

    /// Called when entering a matrix list
    fn begin_list(&mut self, key: &str, list: &MatrixList, ctx: &VisitorContext)
        -> Result<(), Self::Error> { Ok(()) }

    /// Called for each row in a matrix list
    fn visit_node(&mut self, node: &Node, schema: &[String], ctx: &VisitorContext)
        -> Result<(), Self::Error>;

    /// Called when leaving a matrix list
    fn end_list(&mut self, key: &str, list: &MatrixList, ctx: &VisitorContext)
        -> Result<(), Self::Error> { Ok(()) }
}
```

Example: counting all values in a document:

```rust
struct ValueCounter {
    count: usize,
}

impl DocumentVisitor for ValueCounter {
    type Error = std::convert::Infallible;

    fn visit_scalar(&mut self, _key: &str, _value: &Value, _ctx: &VisitorContext)
        -> Result<(), Self::Error>
    {
        self.count += 1;
        Ok(())
    }

    fn visit_node(&mut self, node: &Node, _schema: &[String], _ctx: &VisitorContext)
        -> Result<(), Self::Error>
    {
        self.count += 1 + node.fields.len();  // ID + fields
        Ok(())
    }
}

// Use it
let mut counter = ValueCounter { count: 0 };
traverse(&doc, &mut counter)?;
println!("Total values: {}", counter.count);
```

### Direct Recursion

For simple tasks, direct recursion is often cleaner:

```rust
fn count_items(item: &Item) -> usize {
    match item {
        Item::Scalar(_) => 1,
        Item::Object(map) => 1 + map.values().map(count_items).sum::<usize>(),
        Item::List(list) => 1 + list.rows.len(),
    }
}

fn find_all_references(item: &Item) -> Vec<&Reference> {
    let mut refs = Vec::new();
    collect_refs(item, &mut refs);
    refs
}

fn collect_refs<'a>(item: &'a Item, refs: &mut Vec<&'a Reference>) {
    match item {
        Item::Scalar(Value::Reference(r)) => refs.push(r),
        Item::Scalar(_) => {}
        Item::Object(map) => {
            for child in map.values() {
                collect_refs(child, refs);
            }
        }
        Item::List(list) => {
            for node in &list.rows {
                for field in &node.fields {
                    if let Value::Reference(r) = field {
                        refs.push(r);
                    }
                }
            }
        }
    }
}
```

---

## Special Types in Detail

### Reference: Links Between Entities

```rust
pub struct Reference {
    /// Optional type qualifier (e.g., "User" in "@User:id")
    pub type_name: Option<Box<str>>,

    /// The ID being referenced
    pub id: Box<str>,
}
```

References can be qualified (`@User:alice`) or unqualified (`@alice`):

```rust
// Unqualified: searches based on context
let r1 = Reference::local("alice");
assert_eq!(r1.to_ref_string(), "@alice");

// Qualified: explicit type
let r2 = Reference::qualified("User", "alice");
assert_eq!(r2.to_ref_string(), "@User:alice");
```

### Tensor: Multi-dimensional Arrays

```rust
pub enum Tensor {
    /// A single number
    Scalar(f64),

    /// A nested array of tensors
    Array(Vec<Tensor>),
}
```

Tensors represent numerical data of any dimension:

```rust
// Scalar
let t = parse_tensor("42")?;
assert_eq!(t.shape(), vec![]);  // Empty shape = scalar

// 1D vector
let t = parse_tensor("[1,2,3]")?;
assert_eq!(t.shape(), vec![3]);
assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0]);

// 2D matrix
let t = parse_tensor("[[1,2],[3,4]]")?;
assert_eq!(t.shape(), vec![2, 2]);
assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0, 4.0]);

// 3D tensor
let t = parse_tensor("[[[1,2],[3,4]],[[5,6],[7,8]]]")?;
assert_eq!(t.shape(), vec![2, 2, 2]);
```

### Expression: Computed Values

```rust
pub enum Expression {
    /// A literal value
    Literal { value: ExprLiteral, span: Span },

    /// An identifier
    Identifier { name: String, span: Span },

    /// A function call
    Call { name: String, args: Vec<Expression>, span: Span },

    /// Field access
    Access { target: Box<Expression>, field: String, span: Span },
}
```

Expressions represent computed values in `$(...)` syntax:

```rust
// Simple identifier
let e = parse_expression("name")?;

// Function call
let e = parse_expression("upper(name)")?;

// Field access
let e = parse_expression("user.name")?;

// Chained
let e = parse_expression("user.profile.bio")?;

// Nested calls
let e = parse_expression("concat(upper(first), \" \", last)")?;
```

---

## Design Decisions

### Why Owned Strings?

The AST owns its string data. This makes the API simpler (no lifetime parameters) and allows the AST to outlive the input buffer. See [Memory Optimization](zero-copy-optimizations.md) for the full discussion.

### Why SmallVec for Fields?

Most rows have few fields. SmallVec avoids heap allocation for the common case while gracefully handling rows with many fields.

### Why Option<Box<...>> for Children?

Most nodes don't have children. Using `Option<Box<_>>` means childless nodes only pay 8 bytes (a null pointer), not the full size of an empty BTreeMap.

### Why BTreeMap Everywhere?

Determinism. HEDL documents serialize to identical output regardless of insertion order. This is required for canonicalization and makes testing reliable.

---

## Working with the AST

### Reading Values

```rust
// Get a scalar value
if let Some(Item::Scalar(Value::String(name))) = doc.root.get("name") {
    println!("Name: {}", name);
}

// Get a nested value
if let Some(Item::Object(user)) = doc.root.get("user") {
    if let Some(Item::Scalar(Value::Int(age))) = user.get("age") {
        println!("Age: {}", age);
    }
}

// Get a row from a matrix
if let Some(Item::List(users)) = doc.root.get("users") {
    for node in &users.rows {
        println!("User {} has {} fields", node.id, node.fields.len());
    }
}
```

### Modifying the AST

```rust
// Add a new key
doc.root.insert(
    "new_key".to_string(),
    Item::Scalar(Value::String("new_value".into()))
);

// Modify a value
if let Some(Item::Scalar(ref mut val)) = doc.root.get_mut("count") {
    *val = Value::Int(42);
}

// Add a row to a matrix
if let Some(Item::List(ref mut users)) = doc.root.get_mut("users") {
    users.rows.push(Node {
        type_name: "User".to_string(),
        id: "charlie".to_string(),
        fields: smallvec![
            Value::String("Charlie".into()),
            Value::String("charlie@example.com".into())
        ],
        children: None,
        child_count: 0,
    });
}
```

---

## The AST's Role

The AST is the bridge between text and meaning. It's where:

- The parser deposits its work
- Format converters find their source data
- Validators check for problems
- Tools navigate and transform documents

Understanding the AST is understanding HEDL at its core. Every other system builds on these types, these relationships, these design decisions.

---

## Related Documentation

- [Parser Architecture](parser-architecture.md): How the AST gets built
- [Memory Optimization](zero-copy-optimizations.md): Why the AST is designed this way
- [Error Handling](error-handling.md): Errors during AST construction
