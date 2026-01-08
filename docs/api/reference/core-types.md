# Core Types Reference

Complete reference for HEDL core data types and structures.

## Document

The root type representing a complete HEDL document.

### Type Definition

```rust
pub struct Document {
    pub version: (u32, u32),
    pub aliases: BTreeMap<String, String>,
    pub structs: BTreeMap<String, Vec<String>>,
    pub nests: BTreeMap<String, String>,
    pub root: BTreeMap<String, Item>,
}
```

### Fields

- **version**: `(u32, u32)` - HEDL format version (major, minor)
- **aliases**: Alias definitions mapping alias names to string values
- **structs**: Struct type definitions mapping struct names to field lists
- **nests**: NEST relationships mapping parent type to child type
- **root**: Top-level items in the document body, keyed by name

### Example

```rust
use hedl::parse;

let doc = parse("%VERSION: 1.0\n---\nkey: value")?;
assert_eq!(doc.version, (1, 0));
assert!(doc.root.contains_key("key"));
```

## Value

Represents scalar values in HEDL.

### Type Definition

```rust
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

### Variants

#### Null
Represents a null/nil value (`~` in HEDL).

```rust
let v = Value::Null;
```

#### Bool(bool)
Boolean true/false.

```rust
let v = Value::Bool(true);
```

#### Int(i64)
64-bit signed integer.

```rust
let v = Value::Int(42);
```

#### Float(f64)
64-bit floating point.

```rust
let v = Value::Float(3.14);
```

#### String(String)
UTF-8 string.

```rust
let v = Value::String("hello".to_string());
```

#### Tensor(Tensor)
Multi-dimensional numerical array.

```rust
use hedl_core::Tensor;

let v = Value::Tensor(Tensor::Array(vec![
    Tensor::Scalar(1.0),
    Tensor::Scalar(2.0),
    Tensor::Scalar(3.0),
]));
```

#### Reference(Reference)
Reference to another entity.

```rust
let v = Value::Reference(Reference {
    type_name: Some("User".to_string()),
    id: "alice".to_string(),
});
```

#### Expression(Expression)
Deferred computation expression (`$(...)` syntax).

```rust
// Expressions are parsed from $(...) syntax
let v = Value::Expression(expr);
```

### Methods

```rust
impl Value {
    pub fn is_null(&self) -> bool;
    pub fn is_reference(&self) -> bool;
    pub fn as_str(&self) -> Option<&str>;
    pub fn as_int(&self) -> Option<i64>;
    pub fn as_float(&self) -> Option<f64>;
    pub fn as_bool(&self) -> Option<bool>;
    pub fn as_reference(&self) -> Option<&Reference>;
    pub fn as_expression(&self) -> Option<&Expression>;
}
```

## Item

Body items in a HEDL document.

### Type Definition

```rust
pub enum Item {
    Scalar(Value),
    Object(BTreeMap<String, Item>),
    List(MatrixList),
}
```

### Variants

#### Scalar(Value)
A scalar value.

```rust
let item = Item::Scalar(Value::String("hello".to_string()));
```

#### Object(BTreeMap<String, Item>)
A nested object with key-value pairs.

```rust
let mut map = BTreeMap::new();
map.insert("name".into(), Item::Scalar(Value::String("Alice".into())));
let item = Item::Object(map);
```

#### List(MatrixList)
A typed matrix list.

```rust
let item = Item::List(matrix_list);
```

### Methods

```rust
impl Item {
    pub fn as_scalar(&self) -> Option<&Value>;
    pub fn as_object(&self) -> Option<&BTreeMap<String, Item>>;
    pub fn as_list(&self) -> Option<&MatrixList>;
}
```

## Node

A row/node in a matrix list.

### Type Definition

```rust
pub struct Node {
    pub type_name: String,
    pub id: String,
    pub fields: Vec<Value>,
    pub children: BTreeMap<String, Vec<Node>>,
    pub child_count: Option<usize>,
}
```

### Fields

- **type_name**: The struct type name (from schema)
- **id**: The node's ID (first column value)
- **fields**: Field values aligned with schema columns
- **children**: Child nodes grouped by type (from NEST relationships)
- **child_count**: Optional count hint for LLM comprehension

### Example

```rust
let node = Node::new("User", "alice", vec![
    Value::String("Alice Smith".to_string()),
    Value::String("alice@example.com".to_string()),
]);
```

### Methods

```rust
impl Node {
    pub fn new(type_name: impl Into<String>, id: impl Into<String>, fields: Vec<Value>) -> Self;
    pub fn get_field(&self, index: usize) -> Option<&Value>;
    pub fn add_child(&mut self, child_type: impl Into<String>, child: Node);
    pub fn set_child_count(&mut self, count: usize);
}
```

## MatrixList

A typed matrix list with schema.

### Type Definition

```rust
pub struct MatrixList {
    pub type_name: String,
    pub schema: Vec<String>,
    pub rows: Vec<Node>,
    pub count_hint: Option<usize>,
}
```

### Fields

- **type_name**: The struct type name
- **schema**: Column names from the struct definition
- **rows**: Data rows as Node instances
- **count_hint**: Optional count hint for LLM comprehension

### Example

```rust
let list = MatrixList::new("User", vec!["id".into(), "name".into(), "email".into()]);
```

### Methods

```rust
impl MatrixList {
    pub fn new(type_name: impl Into<String>, schema: Vec<String>) -> Self;
    pub fn with_rows(type_name: impl Into<String>, schema: Vec<String>, rows: Vec<Node>) -> Self;
    pub fn add_row(&mut self, node: Node);
    pub fn column_count(&self) -> usize;
}
```

## Reference

Entity reference type.

### Type Definition

```rust
pub struct Reference {
    pub type_name: Option<String>,
    pub id: String,
}
```

### Fields

- **type_name**: Optional type qualifier (e.g., "User" in `@User:alice`)
- **id**: Entity identifier

### Examples

```rust
// @alice (local reference)
Reference {
    type_name: None,
    id: "alice".to_string(),
}

// @User:alice (qualified reference)
Reference {
    type_name: Some("User".to_string()),
    id: "alice".to_string(),
}
```

### Methods

```rust
impl Reference {
    pub fn local(id: impl Into<String>) -> Self;
    pub fn qualified(type_name: impl Into<String>, id: impl Into<String>) -> Self;
    pub fn to_ref_string(&self) -> String;
}
```

## Tensor

Multi-dimensional numerical array (recursive enum).

### Type Definition

```rust
pub enum Tensor {
    Scalar(f64),
    Array(Vec<Tensor>),
}
```

### Variants

- **Scalar(f64)**: A single numeric value
- **Array(Vec<Tensor>)**: A nested array of tensors

### Example

```rust
// 2x2 matrix [[1, 2], [3, 4]]
let tensor = Tensor::Array(vec![
    Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
    Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
]);

// Simple 1D array [1, 2, 3]
let array = Tensor::Array(vec![
    Tensor::Scalar(1.0),
    Tensor::Scalar(2.0),
    Tensor::Scalar(3.0),
]);
```

## ParseOptions

Parser configuration options.

### Type Definition

```rust
pub struct ParseOptions {
    pub limits: Limits,
    pub strict_refs: bool,
}
```

### Fields

- **limits**: Resource limits for parsing
- **strict_refs**: Require all references to be resolved (default: true)

### Default

```rust
impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            strict_refs: true,
        }
    }
}
```

## Limits

Resource limits for secure parsing.

### Type Definition

```rust
pub struct Limits {
    pub max_file_size: usize,
    pub max_line_length: usize,
    pub max_indent_depth: usize,
    pub max_nodes: usize,
    pub max_aliases: usize,
    pub max_columns: usize,
    pub max_nest_depth: usize,
    pub max_block_string_size: usize,
    pub max_object_keys: usize,
    pub max_total_keys: usize,
}
```

### Fields

- **max_file_size**: Maximum file size in bytes (default: 1GB)
- **max_line_length**: Maximum line length in bytes (default: 1MB)
- **max_indent_depth**: Maximum indent depth (default: 50)
- **max_nodes**: Maximum number of nodes (default: 10M)
- **max_aliases**: Maximum number of aliases (default: 10k)
- **max_columns**: Maximum columns per schema (default: 100)
- **max_nest_depth**: Maximum NEST hierarchy depth (default: 100)
- **max_block_string_size**: Maximum block string size in bytes (default: 10MB)
- **max_object_keys**: Maximum keys per object (default: 10k)
- **max_total_keys**: Maximum total keys across all objects (default: 10M)

### Example

```rust
use hedl_core::Limits;

let limits = Limits {
    max_indent_depth: 20,
    max_nodes: 100_000,
    ..Limits::default()
};
```

### Methods

```rust
impl Limits {
    pub fn unlimited() -> Self;  // No restrictions (for testing)
}
```

## Error Types

See [Errors Reference](../errors.md) for complete error documentation.

### HedlError

```rust
pub struct HedlError {
    pub kind: HedlErrorKind,
    pub message: String,
    pub line: usize,
    pub column: Option<usize>,
    pub context: Option<String>,
}
```

### HedlErrorKind

```rust
pub enum HedlErrorKind {
    Syntax,      // Lexical or structural violation
    Version,     // Unsupported version
    Schema,      // Schema violation or mismatch
    Alias,       // Duplicate or invalid alias
    Shape,       // Wrong number of cells in row
    Semantic,    // Logical error (ditto in ID, etc.)
    OrphanRow,   // Child row without NEST rule
    Collision,   // Duplicate ID within type
    Reference,   // Unresolved reference in strict mode
    Security,    // Security limit exceeded
    Conversion,  // Format conversion error
    IO,          // I/O error
}
```

## Type Aliases

```rust
pub type HedlResult<T> = Result<T, HedlError>;
```

## See Also

- [Parser API](parser-api.md) - Parsing functions
- [Serializer API](serializer-api.md) - Serialization functions
- [Utility Functions](utility-functions.md) - Helper functions
- [Rust API Reference](../rust-api.md) - Complete Rust API
