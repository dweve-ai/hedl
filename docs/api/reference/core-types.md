# Core Types Reference

Complete reference for HEDL core data types and structures.

## Document

The root type representing a complete HEDL document.

### Type Definition

```rust
pub struct Document {
    pub version: (u32, u32),
    pub schema_versions: BTreeMap<String, SchemaVersion>,
    pub aliases: BTreeMap<String, String>,
    pub structs: BTreeMap<String, Vec<String>>,
    pub nests: BTreeMap<String, String>,
    pub root: BTreeMap<String, Item>,
}
```

### Fields

- **version**: `(u32, u32)` - HEDL format version (major, minor)
- **schema_versions**: Schema version metadata for typed structures
- **aliases**: Alias definitions mapping alias names to string values
- **structs**: Struct type definitions mapping struct names to field lists
- **nests**: NEST relationships mapping parent type to child type
- **root**: Top-level items in the document body, keyed by name

### Methods

```rust
impl Document {
    /// Create a new empty document with the specified version.
    pub fn new(version: (u32, u32)) -> Self;

    /// Get an item from the root by key.
    pub fn get(&self, key: &str) -> Option<&Item>;

    /// Get a struct schema by type name.
    pub fn get_schema(&self, type_name: &str) -> Option<&Vec<String>>;

    /// Get the child type for a parent type (from NEST).
    pub fn get_child_type(&self, parent_type: &str) -> Option<&String>;

    /// Expand an alias key to its value.
    pub fn expand_alias(&self, key: &str) -> Option<&String>;

    /// Get schema version for a type.
    pub fn get_schema_version(&self, type_name: &str) -> Option<SchemaVersion>;

    /// Set schema version for a type.
    pub fn set_schema_version(&mut self, type_name: String, version: SchemaVersion);
}
```

### Example

```rust
use hedl::parse;

let doc = parse("%V:2.0\n---\nkey: value")?;
assert_eq!(doc.version, (1, 0));
assert!(doc.root.contains_key("key"));

// Query methods
let item = doc.get("key");
let schema = doc.get_schema("User");
let child_type = doc.get_child_type("User");
let alias_value = doc.expand_alias("active");
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
    String(Box<str>),
    Tensor(Box<Tensor>),
    Reference(Reference),
    Expression(Box<Expression>),
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

#### String(Box<str>)
UTF-8 string (boxed to reduce enum size).

```rust
let v = Value::String("hello".into());
```

#### Tensor(Box<Tensor>)
Multi-dimensional numerical array (boxed to reduce enum size).

```rust
use hedl_core::lex::Tensor;

let v = Value::Tensor(Box::new(Tensor::Array(vec![
    Tensor::Scalar(1.0),
    Tensor::Scalar(2.0),
    Tensor::Scalar(3.0),
])));
```

#### Reference(Reference)
Reference to another entity.

```rust
let v = Value::Reference(Reference::qualified("User", "alice"));
// Or for unqualified references:
let v = Value::Reference(Reference::local("alice"));
```

#### Expression(Box\<Expression\>)
Deferred computation expression (`$(...)` syntax).

```rust
// Expressions are parsed from $(...) syntax
use hedl_core::lex::parse_expression_token;

let expr = parse_expression_token("$(now())")?;
let v = Value::Expression(Box::new(expr));
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
let item = Item::Scalar(Value::String("hello".into()));
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
    pub fields: SmallVec<[Value; 4]>,  // Stack-allocated for ≤4 fields
    pub children: Option<Box<BTreeMap<String, Vec<Node>>>>,  // Lazy allocation
    pub child_count: u16,  // Compact hint (u16 saves 6 bytes)
}
```

### Fields

- **type_name**: The struct type name (from schema)
- **id**: The node's ID (first column value)
- **fields**: Field values aligned with schema columns (SmallVec optimizes for ≤4 fields)
- **children**: Child nodes grouped by type (lazy Box allocation when present)
- **child_count**: Count hint for LLM comprehension (u16, 0 means no hint)

### Example

```rust
let node = Node::new("User", "alice", vec![
    Value::String("Alice Smith".into()),
    Value::String("alice@example.com".into()),
]);
```

### Methods

```rust
impl Node {
    /// Create a new node. Fields are converted to SmallVec internally.
    pub fn new(type_name: impl Into<String>, id: impl Into<String>, fields: Vec<Value>) -> Self;

    /// Create a node with a child count hint.
    pub fn with_child_count(
        type_name: impl Into<String>,
        id: impl Into<String>,
        fields: Vec<Value>,
        child_count: usize,
    ) -> Self;

    /// Get a field value by column index.
    pub fn get_field(&self, index: usize) -> Option<&Value>;

    /// Get the child count hint, if provided.
    pub fn get_child_count(&self) -> Option<usize>;

    /// Set the child count hint. Saturates at u16::MAX (65,535).
    pub fn set_child_count(&mut self, count: usize);

    /// Get children map (if any exist).
    pub fn children(&self) -> Option<&BTreeMap<String, Vec<Node>>>;

    /// Get mutable children map (if any exist).
    pub fn children_mut(&mut self) -> Option<&mut BTreeMap<String, Vec<Node>>>;

    /// Add a child node.
    pub fn add_child(&mut self, child_type: impl Into<String>, child: Node);
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
    pub fn with_count_hint(type_name: impl Into<String>, schema: Vec<String>, count_hint: usize) -> Self;
    pub fn add_row(&mut self, node: Node);
    pub fn column_count(&self) -> usize;
}
```

## Reference

Entity reference type.

### Type Definition

```rust
pub struct Reference {
    pub type_name: Option<Box<str>>,
    pub id: Box<str>,
}
```

### Fields

- **type_name**: Optional type qualifier (e.g., "User" in `@User:alice`), boxed for memory efficiency
- **id**: Entity identifier, boxed for memory efficiency

### Examples

```rust
// @alice (local reference)
Reference {
    type_name: None,
    id: "alice".into(),
}

// @User:alice (qualified reference)
Reference {
    type_name: Some("User".into()),
    id: "alice".into(),
}
```

### Methods

```rust
impl Reference {
    /// Create a local (unqualified) reference.
    pub fn local(id: impl Into<String>) -> Self;

    /// Create a qualified reference with type name.
    pub fn qualified(type_name: impl Into<String>, id: impl Into<String>) -> Self;

    /// Alias for local() - create an unqualified reference.
    pub fn unqualified(id: impl Into<String>) -> Self;

    /// Format as a reference string with @ prefix.
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

### Methods

```rust
impl Tensor {
    /// Returns true if this tensor contains only integers (no decimal points).
    pub fn is_integer(&self) -> bool;

    /// Returns the shape of the tensor as a vector of dimensions.
    pub fn shape(&self) -> Vec<usize>;

    /// Flattens the tensor into a 1D vector of f64 values in row-major order.
    pub fn flatten(&self) -> Vec<f64>;

    /// Returns true if this is a scalar value.
    pub fn is_scalar(&self) -> bool;

    /// Returns true if this is an array.
    pub fn is_array(&self) -> bool;

    /// Returns the number of dimensions (0 for scalar).
    pub fn ndim(&self) -> usize;

    /// Returns the total number of elements.
    pub fn len(&self) -> usize;

    /// Returns true if the tensor has no elements.
    pub fn is_empty(&self) -> bool;
}
```

### Example

```rust
use hedl_core::lex::Tensor;

// 2x2 matrix [[1, 2], [3, 4]]
let tensor = Tensor::Array(vec![
    Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
    Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
]);
assert_eq!(tensor.shape(), vec![2, 2]);
assert_eq!(tensor.flatten(), vec![1.0, 2.0, 3.0, 4.0]);
assert_eq!(tensor.ndim(), 2);
assert_eq!(tensor.len(), 4);
assert!(tensor.is_integer());

// Simple 1D array [1, 2, 3]
let array = Tensor::Array(vec![
    Tensor::Scalar(1.0),
    Tensor::Scalar(2.0),
    Tensor::Scalar(3.0),
]);
assert_eq!(array.shape(), vec![3]);
assert!(array.is_array());
assert_eq!(array.ndim(), 1);

// Scalar
let scalar = Tensor::Scalar(42.0);
assert_eq!(scalar.shape(), vec![]);
assert!(scalar.is_scalar());
assert_eq!(scalar.ndim(), 0);
```

## ParseOptions

Parser configuration options (defined in `hedl_core::parser`).

### Type Definition

```rust
pub struct ParseOptions {
    pub limits: Limits,
    pub reference_mode: ReferenceMode,
}

pub enum ReferenceMode {
    Strict,   // Unresolved references cause errors (default)
    Lenient,  // Unresolved references are silently ignored
}
```

### Fields

- **limits**: Resource limits for parsing (see `Limits` below)
- **reference_mode**: How to handle unresolved references (default: `ReferenceMode::Strict`)
  - `Strict`: Errors on unresolved references (default)
  - `Lenient`: Ignores unresolved references
  - Note: Ambiguous references always error regardless of mode

### Methods

```rust
impl ParseOptions {
    /// Create a new builder for ParseOptions.
    pub fn builder() -> ParseOptionsBuilder;
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            reference_mode: ReferenceMode::Strict,
        }
    }
}
```

### Example

```rust
use hedl_core::ParseOptions;
use hedl_core::reference::ReferenceMode;

// Using defaults
let opts = ParseOptions::default();

// Using builder
let opts = ParseOptions::builder()
    .max_depth(100)
    .build();

// Manual construction
let opts = ParseOptions {
    limits: Limits::default(),
    reference_mode: ReferenceMode::Lenient,
};
```

## Limits

Resource limits for secure parsing.

### Type Definition

```rust
use std::time::Duration;

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
    pub max_total_ids: usize,
    pub timeout: Option<Duration>,
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
- **max_total_ids**: Maximum total IDs across all types (default: 10M)
- **timeout**: Maximum parsing duration (default: 30 seconds, None disables timeout)

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
    /// Create limits with no restrictions (for testing).
    ///
    /// All numeric limits are set to usize::MAX and timeout is disabled.
    /// WARNING: Only use for testing trusted input - provides no DoS protection.
    pub fn unlimited() -> Self;
}

impl Default for Limits {
    fn default() -> Self;
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
    Semantic,    // Logical error (null in ID, etc.)
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
