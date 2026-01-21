# Rust API Reference

**Native Rust library for high-performance HEDL processing**

---

## Quick Start

```rust
use hedl::{parse, canonicalize, to_json, validate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a HEDL document
    let hedl = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
    "#;

    let doc = parse(hedl)?;

    // Convert to JSON
    let json = to_json(&doc)?;

    // Canonicalize
    let canonical = canonicalize(&doc)?;

    // Validate
    validate(hedl)?;

    Ok(())
}
```

---

## Core Functions

### Parsing

#### `parse`

Parse a HEDL document from a string.

```rust
pub fn parse(input: &str) -> Result<Document, HedlError>
```

**Parameters**:
- `input`: HEDL document as UTF-8 string

**Returns**: `Result<Document, HedlError>`

**Example**:
```rust
let doc = hedl::parse("%VERSION: 1.0\n---\nkey: value")?;
assert_eq!(doc.version, (1, 0));
```

**Performance**: Optimized with `#[inline]` hint for 5-10% improvement in small document scenarios.

---

#### `parse_lenient`

Parse with lenient reference handling (unresolved references become `null`).

```rust
pub fn parse_lenient(input: &str) -> Result<Document, HedlError>
```

**Example**:
```rust
// This will succeed even with invalid references
let doc = hedl::parse_lenient("%VERSION: 1.0\n---\nuser: @InvalidRef")?;
```

---

#### `parse_with_limits`

Parse with custom resource limits and options.

```rust
pub fn parse_with_limits(
    input: &[u8],
    options: ParseOptions
) -> Result<Document, HedlError>
```

**Example**:
```rust
use hedl::{parse_with_limits, ParseOptions, Limits, ReferenceMode};

let options = ParseOptions {
    reference_mode: ReferenceMode::Lenient,
    limits: Limits {
        max_indent_depth: 100,
        ..Limits::default()
    },
    ..Default::default()
};

let doc = parse_with_limits(input.as_bytes(), options)?;
```

**ReferenceMode Options**:
- `ReferenceMode::Strict` - Error on unresolved references (default)
- `ReferenceMode::Lenient` - Convert unresolved references to `null`

---

### Validation

#### `validate`

Validate HEDL input without fully parsing.

```rust
pub fn validate(input: &str) -> Result<(), HedlError>
```

**Returns**: `Ok(())` if valid, `Err(HedlError)` with details if invalid.

**Example**:
```rust
match hedl::validate(hedl_input) {
    Ok(()) => println!("Valid HEDL"),
    Err(e) => eprintln!("Invalid: {} at line {}", e.message, e.line),
}
```

---

#### `lint`

Check document for best practices and potential issues.

```rust
pub fn lint(doc: &Document) -> Vec<lint::Diagnostic>
```

**Returns**: List of diagnostics (errors, warnings, hints)

**Example**:
```rust
let doc = hedl::parse(input)?;
let diagnostics = hedl::lint(&doc);

for d in diagnostics {
    println!("[{}] {}: {}", d.severity(), d.rule_id(), d.message());
}
```

---

### Canonicalization

#### `canonicalize`

Convert document to canonical (deterministic) form.

```rust
pub fn canonicalize(doc: &Document) -> Result<String, HedlError>
```

**Features**:
- Sorted keys for deterministic output
- Ditto operator optimization
- Consistent whitespace
- Suitable for hashing and diffing

**Example**:
```rust
let doc = hedl::parse("%VERSION: 1.0\n---\nz: 3\na: 1")?;
let canonical = hedl::canonicalize(&doc)?;
// Keys are sorted: a appears before z
```

---

### Format Conversion

#### `to_json`

Convert HEDL document to JSON.

```rust
pub fn to_json(doc: &Document) -> Result<String, HedlError>
```

**Example**:
```rust
let doc = hedl::parse(hedl_input)?;
let json = hedl::to_json(&doc)?;
println!("{}", json);
```

---

#### `from_json`

Convert JSON to HEDL document.

```rust
pub fn from_json(json: &str) -> Result<Document, HedlError>
```

**Example**:
```rust
let json = r#"{"users": [{"id": "alice", "name": "Alice"}]}"#;
let doc = hedl::from_json(json)?;
```

---

## Data Types

### `Document`

Represents a parsed HEDL document.

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

**Fields**:
- `version`: HEDL format version (e.g., `(1, 0)`)
- `schema_versions`: Schema evolution metadata for versioned types
- `aliases`: Alias definitions mapping alias names to string values
- `structs`: Schema definitions mapping type names to field lists
- `nests`: Parent-child type relationships (parent type → child type)
- `root`: Top-level items in the document body

---

### `Item`

Represents an item in the document body.

```rust
pub enum Item {
    Scalar(Value),
    Object(BTreeMap<String, Item>),
    List(MatrixList),
}
```

**Variants**:
- `Scalar`: Single value
- `Object`: Nested object/map
- `List`: Matrix list of typed entities

---

### `Value`

Represents a HEDL scalar value.

```rust
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Box<str>),           // Box<str> reduces enum size
    Tensor(Box<Tensor>),        // Boxed to reduce enum size
    Reference(Reference),
    Expression(Box<Expression>), // Boxed to reduce enum size
}
```

---

### `Reference`

Represents an entity reference.

```rust
pub struct Reference {
    pub type_name: Option<Box<str>>,
    pub id: Box<str>,
}
```

**Examples**:
- `@alice` → `Reference { type_name: None, id: "alice".into() }`
- `@User:alice` → `Reference { type_name: Some("User".into()), id: "alice".into() }`

---

### `Node`

Represents an entity/row in a matrix list.

```rust
pub struct Node {
    pub type_name: String,
    pub id: String,
    pub fields: SmallVec<[Value; 4]>,  // Stack-allocated for ≤4 fields
    pub children: Option<Box<BTreeMap<String, Vec<Node>>>>,  // Lazy allocation
    pub child_count: u16,  // Compact hint (0 = no hint)
}
```

**Fields**:
- `type_name`: Entity type (from schema)
- `id`: Entity identifier (first column value)
- `fields`: Field values (SmallVec for ≤4 fields stack allocation)
- `children`: Nested child entities by type (lazy Box allocation)
- `child_count`: Count hint for LLM comprehension (u16, 0 means no hint)

---

### `MatrixList`

Represents a typed matrix list.

```rust
pub struct MatrixList {
    pub type_name: String,
    pub schema: Vec<String>,
    pub rows: Vec<Node>,
    pub count_hint: Option<usize>,
}
```

**Fields**:
- `type_name`: The struct type name
- `schema`: Column names from the struct definition
- `rows`: Data rows as Node instances
- `count_hint`: Optional count hint for LLM comprehension

---

## Error Handling Extensions

### `HedlResultExt`

Extension trait for adding context to `Result<T, HedlError>` values.

```rust
pub trait HedlResultExt<T> {
    fn context<C>(self, context: C) -> Result<T, HedlError>
    where
        C: fmt::Display;

    fn with_context<C, F>(self, f: F) -> Result<T, HedlError>
    where
        C: fmt::Display,
        F: FnOnce() -> C;

    fn map_err_to_hedl<F>(self, f: F) -> Result<T, HedlError>
    where
        F: FnOnce(Self::ErrorType) -> HedlError;
}
```

**Methods**:

- `context()`: Add context to an error (eager evaluation)
- `with_context()`: Add context to an error using a closure (lazy evaluation)
- `map_err_to_hedl()`: Convert foreign error types to `HedlError`

**Example**:
```rust
use hedl::{parse, HedlResultExt};

fn load_config(path: &str) -> Result<hedl::Document, hedl::HedlError> {
    let content = std::fs::read_to_string(path)
        .map_err_to_hedl(|e| hedl::HedlError::io(format!("Failed to read {}: {}", path, e)))?;

    parse(&content)
        .context(format!("while parsing config file {}", path))
}
```

**Lazy Context for Performance**:
```rust
use hedl::{parse, HedlResultExt};

fn process_document(id: u64, content: &str) -> Result<(), hedl::HedlError> {
    let doc = parse(content)
        .with_context(|| format!("processing document {} ({} bytes)", id, content.len()))?;

    // Process the document...
    Ok(())
}
```

**Context Chaining**:
```rust
use hedl::{parse, HedlResultExt};

fn validate_user_data(user_id: &str, data: &str) -> Result<(), hedl::HedlError> {
    let doc = parse(data)
        .context("failed to parse user data")
        .context(format!("for user {}", user_id))?;

    // Validation logic...
    Ok(())
}
```

---

## Value Coercion

Value coercion functions are available from the `hedl_core` crate.

### Coercion Functions

Convert values between types with configurable strictness.

#### `coerce`

Coerce a value to a target type using specified mode.

```rust
pub fn coerce(
    value: Value,
    expected: &ExpectedType,
    mode: CoercionMode
) -> CoercionResult
```

**Example**:
```rust
use hedl_core::{coerce, Value, ExpectedType, CoercionMode};

let value = Value::String("42".into());
let result = coerce(value, &ExpectedType::Int, CoercionMode::Lenient)?;
// Result: Value::Int(42)
```

---

#### `coerce_with_config`

Coerce a value with custom configuration (more control than `coerce`).

```rust
pub fn coerce_with_config(
    value: Value,
    expected: &ExpectedType,
    config: &CoercionConfig
) -> CoercionResult
```

**Example**:
```rust
use hedl_core::{coerce_with_config, CoercionConfig, CoercionLevel, Value, ExpectedType};

let config = CoercionConfig {
    level: CoercionLevel::Permissive,
    allow_string_to_number: true,
    allow_lossy_float_to_int: true,
    ..Default::default()
};

let value = Value::String("3.14".into());
let result = coerce_with_config(value, &ExpectedType::Int, &config)?;
// With allow_lossy_float_to_int: truncates to Value::Int(3)
```

---

### `CoercionConfig`

Configuration for value coercion with fine-grained control.

```rust
pub struct CoercionConfig {
    pub level: CoercionLevel,
    pub allow_string_to_number: bool,
    pub allow_lossy_float_to_int: bool,
    pub bool_true_values: Vec<String>,
    pub bool_false_values: Vec<String>,
    pub null_as_default: bool,
}
```

**Fields**:
- `level`: Strictness level (None, Strict, Standard, Permissive)
- `allow_string_to_number`: Allow "42" → 42 (only applies when level is Standard or Permissive)
- `allow_lossy_float_to_int`: Allow 3.14 → 3 truncation (only applies when level is Permissive)
- `bool_true_values`: Custom true values for boolean coercion (default: ["true", "yes", "1"])
- `bool_false_values`: Custom false values for boolean coercion (default: ["false", "no", "0"])
- `null_as_default`: Treat null as type-specific default (0, 0.0, false, "" - only applies when level is Permissive)

**Helper Methods**:
- `CoercionConfig::none()` - No coercion allowed
- `CoercionConfig::strict()` - Only safe, obvious conversions
- `CoercionConfig::standard()` - Common conversions (default)

---

### `CoercionLevel`

Controls strictness of type coercion.

```rust
pub enum CoercionLevel {
    None,        // No coercion allowed
    Strict,      // Only safe, obvious conversions
    Standard,    // Common conversions (default)
    Permissive,  // Aggressive conversions with potential data loss
}
```

---

### `CoercionMode`

Simple mode for basic coercion strictness.

```rust
pub enum CoercionMode {
    Strict,   // Conservative coercion
    Lenient,  // More permissive coercion
}
```

---

## Visitor System

### Additional Visitor Types

Beyond the basic `DocumentVisitor` trait (documented in the traverse section), several specialized visitor types are available:

#### `Visitor` Trait

Immutable visitor for document traversal.

```rust
pub trait Visitor {
    fn visit_document(&mut self, doc: &Document) -> VisitDecision;
    fn visit_item(&mut self, key: &str, item: &Item) -> VisitDecision;
    fn visit_value(&mut self, value: &Value) -> VisitDecision;
    fn visit_node(&mut self, node: &Node) -> VisitDecision;
}
```

---

#### `VisitorMut` Trait

Mutable visitor for document transformation.

```rust
pub trait VisitorMut {
    fn visit_document_mut(&mut self, doc: &mut Document) -> VisitDecision;
    fn visit_item_mut(&mut self, key: &str, item: &mut Item) -> VisitDecision;
    fn visit_value_mut(&mut self, value: &mut Value) -> VisitDecision;
    fn visit_node_mut(&mut self, node: &mut Node) -> VisitDecision;
}
```

---

#### `FallibleVisitor` Trait

Visitor with error handling.

```rust
pub trait FallibleVisitor {
    type Error;

    fn visit_document(&mut self, doc: &Document) -> Result<VisitDecision, Self::Error>;
    fn visit_item(&mut self, key: &str, item: &Item) -> Result<VisitDecision, Self::Error>;
    fn visit_value(&mut self, value: &Value) -> Result<VisitDecision, Self::Error>;
    fn visit_node(&mut self, node: &Node) -> Result<VisitDecision, Self::Error>;
}
```

---

#### `Transformer` Trait

Transform values during traversal.

```rust
pub trait Transformer {
    fn transform_value(&mut self, value: Value) -> Value;
    fn transform_node(&mut self, node: Node) -> Node;
}
```

---

### Visitor Utilities

#### Built-in Visitor Implementations

```rust
// Collect all references in a document
use hedl_core::visitor::{ReferenceCollector, traverse};

let mut collector = ReferenceCollector::new();
traverse(&doc, &mut collector)?;
println!("Found {} references", collector.references.len());
```

```rust
// Collect all nodes in a document
use hedl_core::visitor::{NodeCollector, traverse};

let mut collector = NodeCollector::new();
traverse(&doc, &mut collector)?;
println!("Found {} nodes", collector.nodes.len());
```

```rust
// Count traversal depth
use hedl_core::visitor::{DepthCounter, traverse};

let mut counter = DepthCounter::new();
traverse(&doc, &mut counter)?;
println!("Max depth: {}", counter.max_depth);
```

```rust
// Collect paths to all items
use hedl_core::visitor::{PathCollector, traverse};

let mut collector = PathCollector::new();
traverse(&doc, &mut collector)?;
for path in &collector.paths {
    println!("Path: {}", path);
}
```

---

#### Traversal Functions

```rust
// Immutable traversal
pub fn traverse<V: Visitor>(doc: &Document, visitor: &mut V) -> TraversalResult;

// Mutable traversal
pub fn traverse_mut<V: VisitorMut>(doc: &mut Document, visitor: &mut V) -> TraversalResult;

// Fallible traversal
pub fn traverse_fallible<V: FallibleVisitor>(doc: &Document, visitor: &mut V) -> Result<(), V::Error>;

// Transform document
pub fn transform<T: Transformer>(doc: Document, transformer: &mut T) -> Document;
```

---

#### `VisitDecision`

Control flow for visitor traversal.

```rust
pub enum VisitDecision {
    Continue,   // Continue traversal normally
    Skip,       // Skip children of current node
    Stop,       // Stop traversal completely
}
```

---

#### `TraversalConfig`

Configuration for traversal behavior.

```rust
pub struct TraversalConfig {
    pub mode: TraversalMode,
    pub order: TraversalOrder,
    pub max_depth: Option<usize>,
}
```

**Fields**:
- `mode`: Depth-first or breadth-first traversal
- `order`: Pre-order or post-order traversal
- `max_depth`: Optional depth limit

---

#### `TraversalMode`

Traversal strategy.

```rust
pub enum TraversalMode {
    DepthFirst,
    BreadthFirst,
}
```

---

#### `TraversalOrder`

Visit order for traversal.

```rust
pub enum TraversalOrder {
    PreOrder,   // Visit parent before children
    PostOrder,  // Visit children before parent
}
```

---

#### `PathSegment`

Represents a segment in a document path.

```rust
pub enum PathSegment {
    Key(String),
    Index(usize),
    Field(String),
}
```

---

#### `TraversalStats`

Statistics collected during traversal.

```rust
pub struct TraversalStats {
    pub nodes_visited: usize,
    pub values_visited: usize,
    pub max_depth: usize,
    pub total_items: usize,
}
```

---

## Advanced Modules

### `hedl::c14n`

Canonicalization with custom configuration.

```rust
use hedl::c14n::{canonicalize_with_config, CanonicalConfig, QuotingStrategy};

let config = CanonicalConfig::new()
    .with_quoting(QuotingStrategy::Minimal)
    .with_ditto(true)
    .with_sort_keys(true);

let canonical = canonicalize_with_config(&doc, &config)?;
```

---

### `hedl::json`

JSON conversion with configuration.

```rust
use hedl::json::{to_json_value, ToJsonConfig};

let config = ToJsonConfig::default();
let json_value = to_json_value(&doc, &config).map_err(|e| format!("{}", e))?;
```

---

### `hedl::lint`

Linting and best practices validation for HEDL documents.

```rust
use hedl::lint::{
    lint, lint_with_config, LintConfig, LintRunner, LintRule,
    Diagnostic, DiagnosticKind, Severity, RuleConfig,
};

// Simple linting with defaults
let doc = hedl::parse(input)?;
let diagnostics = lint(&doc);

for diagnostic in &diagnostics {
    println!(
        "[{}] {}: {}",
        diagnostic.rule_id(),
        diagnostic.severity(),
        diagnostic.message()
    );

    if let Some(suggestion) = diagnostic.suggestion() {
        println!("  Suggestion: {}", suggestion);
    }
}

// Custom configuration
let mut config = LintConfig::default();
config.min_severity = Severity::Warning;  // Only report warnings and errors
config.disable_rule("empty-list");         // Disable specific rule

let diagnostics = lint_with_config(&doc, config);
```

**Core Types**:

| Type | Description |
|------|-------------|
| `LintRule` | Trait for implementing custom lint rules |
| `Diagnostic` | Lint result with severity, message, and suggestion |
| `DiagnosticKind` | Type of lint issue found |
| `LintConfig` | Configuration for rule enablement and thresholds |
| `LintRunner` | Orchestrates rule execution |
| `Severity` | Hint, Warning, Error |
| `RuleConfig` | Per-rule configuration (enabled, error escalation) |

**Built-in Rules**:
- `IdNamingRule` (`id-naming`) - Checks ID naming conventions
- `UnusedSchemaRule` (`unused-schema`) - Warns about unused %STRUCT definitions
- `EmptyListRule` (`empty-list`) - Warns about empty matrix lists
- `UnqualifiedKvReferenceRule` (`unqualified-kv-ref`) - Warns about unqualified references in KV context

**Custom Rules**:

```rust
use hedl::lint::{LintRule, Diagnostic, DiagnosticKind};
use hedl_core::Document;

struct MaxFieldsRule {
    max_fields: usize,
}

impl LintRule for MaxFieldsRule {
    fn id(&self) -> &str { "max-fields" }
    fn description(&self) -> &str { "Check for excessive fields per node" }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];
        // Check each matrix list for nodes with too many fields
        for item in doc.root.values() {
            if let hedl_core::Item::List(list) = item {
                for row in &list.rows {
                    if row.fields.len() > self.max_fields {
                        diagnostics.push(Diagnostic::warning(
                            DiagnosticKind::Custom("max-fields".into()),
                            format!("Node '{}' has {} fields, max is {}",
                                row.id, row.fields.len(), self.max_fields),
                            "max-fields",
                        ));
                    }
                }
            }
        }
        diagnostics
    }
}
```

---

### `hedl_core::traverse` (Legacy Visitor System)

Document traversal system for analysis. This legacy system is available via `hedl_core` crate directly. For new code, prefer the modern visitor system documented above.

```rust
use hedl_core::{traverse, DocumentVisitor, VisitorContext};
use hedl_core::{Document, Value, Node, MatrixList};

// Implement DocumentVisitor for custom analysis
struct NodeCounter {
    count: usize,
}

impl DocumentVisitor for NodeCounter {
    type Error = std::convert::Infallible;

    fn visit_node(&mut self, _node: &Node, _schema: &[String], _ctx: &VisitorContext) -> Result<(), Self::Error> {
        self.count += 1;
        Ok(())
    }
}

let doc = hedl::parse(input)?;
let mut counter = NodeCounter { count: 0 };
let _ = traverse(&doc, &mut counter)?;
println!("Found {} nodes", counter.count);
```

**DocumentVisitor Trait**:

```rust
pub trait DocumentVisitor {
    type Error;

    fn begin_document(&mut self, doc: &Document, ctx: &VisitorContext) -> Result<(), Self::Error> { Ok(()) }
    fn end_document(&mut self, doc: &Document, ctx: &VisitorContext) -> Result<(), Self::Error> { Ok(()) }
    fn visit_scalar(&mut self, key: &str, value: &Value, ctx: &VisitorContext) -> Result<(), Self::Error> { Ok(()) }
    fn begin_object(&mut self, key: &str, ctx: &VisitorContext) -> Result<(), Self::Error> { Ok(()) }
    fn end_object(&mut self, key: &str, ctx: &VisitorContext) -> Result<(), Self::Error> { Ok(()) }
    fn begin_list(&mut self, key: &str, list: &MatrixList, ctx: &VisitorContext) -> Result<(), Self::Error> { Ok(()) }
    fn end_list(&mut self, key: &str, list: &MatrixList, ctx: &VisitorContext) -> Result<(), Self::Error> { Ok(()) }
    fn visit_node(&mut self, node: &Node, schema: &[String], ctx: &VisitorContext) -> Result<(), Self::Error> { Ok(()) }
    fn begin_node_children(&mut self, node: &Node, ctx: &VisitorContext) -> Result<(), Self::Error> { Ok(()) }
    fn end_node_children(&mut self, node: &Node, ctx: &VisitorContext) -> Result<(), Self::Error> { Ok(()) }
}
```

**VisitorContext**:

```rust
pub struct VisitorContext<'a> {
    pub depth: usize,               // Current nesting depth
    pub path: Vec<&'a str>,         // Path to current location (borrowed strings)
    pub document: &'a Document,     // Reference to document
    pub current_schema: Option<&'a [String]>,  // Current list schema
}
```

**Modern Traversal System** (Recommended):

The `hedl_core` crate provides a modern traversal system with `Visitor`, `VisitorMut`, and `FallibleVisitor` traits (available via `use hedl_core::visitor::*`).

```rust
use hedl_core::visitor::{traverse, Visitor, TraversalResult};

// Generic traversal with modern Visitor trait
pub fn traverse<V: Visitor>(doc: &Document, visitor: &mut V) -> TraversalResult;
```

**Legacy Traversal System**:

For backward compatibility, `hedl_core` also provides the legacy `DocumentVisitor`-based system with `StatsCollector`:

```rust
use hedl_core::{traverse, StatsCollector};

let mut stats = StatsCollector::new();
let _ = traverse(&doc, &mut stats)?;
println!("Total nodes: {}", stats.node_count);
println!("Total values: {}", stats.value_count);
```

---

### `hedl::lex`

Low-level lexical utilities.

```rust
use hedl::lex::{parse_reference, is_valid_id_token, is_valid_type_name, scan_regions};

// Parse reference
let ref_token = parse_reference("@User:alice")?;

// Validate tokens
let is_valid = is_valid_id_token("alice_123");

// Scan document regions for IDE integration
let regions = scan_regions(hedl_text);
```

### `hedl::tensor`

Tensor literal parsing.

```rust
use hedl::tensor::parse_tensor;

let tensor = parse_tensor("[1, 2, 3]")?;
```

### `hedl::csv`

CSV row parsing.

```rust
use hedl::csv::parse_csv_row;

let fields = parse_csv_row("alice, Alice Smith, alice@example.com")?;
```

---

## Feature-Gated Modules

### YAML Conversion (`feature = "yaml"`)

```rust
use hedl::yaml::{to_yaml, from_yaml, ToYamlConfig, FromYamlConfig};

let config_to = ToYamlConfig::default();
let yaml = to_yaml(&doc, &config_to)?;
let config_from = FromYamlConfig::default();
let doc = from_yaml(&yaml, &config_from)?;
```

---

### XML Conversion (`feature = "xml"`)

```rust
use hedl::xml::{to_xml, from_xml, ToXmlConfig, FromXmlConfig};

let config_to = ToXmlConfig::default();
let xml = to_xml(&doc, &config_to)?;
let config_from = FromXmlConfig::default();
let doc = from_xml(&xml, &config_from)?;
```

---

### CSV File Conversion (`feature = "csv"`)

```rust
use hedl::csv_file::{to_csv, from_csv, ToCsvConfig, FromCsvConfig};

let config_to = ToCsvConfig::default();
let csv = to_csv(&doc, &config_to)?;
let config_from = FromCsvConfig::default();
let doc = from_csv(&csv, &config_from)?;
```

---

### Parquet Conversion (`feature = "parquet"`)

```rust
use hedl::parquet::{to_parquet_bytes, from_parquet_bytes, ToParquetConfig};

let config = ToParquetConfig::default();
let bytes = to_parquet_bytes(&doc, &config)?;
let doc = from_parquet_bytes(&bytes)?;
```

---

### Neo4j/Cypher Export (`feature = "neo4j"`)

```rust
use hedl::neo4j::{to_cypher, ToCypherConfig};

let config = ToCypherConfig::default();
let cypher = to_cypher(&doc, &config)?;
// Returns Cypher statements for Neo4j graph database
```

---

### TOON Conversion (`feature = "toon"`)

```rust
use hedl::toon::{hedl_to_toon, to_toon, ToToonConfig, Delimiter};

// Simple conversion
let toon = hedl_to_toon(&doc)?;

// Custom configuration
let config = ToToonConfig::new()
    .with_indent(4)
    .with_delimiter(Delimiter::Tab);
let toon = to_toon(&doc, &config)?;
```

---

## Error Handling

### `HedlError`

Main error type for all HEDL operations.

```rust
pub struct HedlError {
    pub kind: HedlErrorKind,
    pub message: String,
    pub line: usize,
    pub column: Option<usize>,
    pub context: Option<String>,
}
```

**Example**:
```rust
match hedl::parse(input) {
    Ok(doc) => { /* success */ }
    Err(e) => {
        eprintln!("Error: {} (kind: {:?}) at line {}",
                  e.message, e.kind, e.line);
    }
}
```

---

### `HedlErrorKind`

Error category enumeration.

```rust
pub enum HedlErrorKind {
    Syntax,      // Lexical or structural violation
    Version,     // Unsupported version
    Schema,      // Schema violation or mismatch
    Alias,       // Duplicate or invalid alias
    Shape,       // Wrong number of cells in row
    Semantic,    // Logical error
    OrphanRow,   // Child row without NEST rule
    Collision,   // Duplicate ID within type
    Reference,   // Unresolved reference
    Security,    // Security limit exceeded
    Conversion,  // Format conversion error
    IO,          // I/O error
}
```

---

## Performance Notes

### Inline Hints

Critical hot-path functions are annotated with `#[inline]`:
- `parse()`: 5-10% improvement for small documents
- `canonicalize()`: 5-10% improvement in serialization
- `to_json()`: 5-10% improvement in format conversion

---

### Memory Optimization

- **Efficient parsing**: Minimizes allocations during parsing (uses owned `String` in AST for safety)
- **Efficient tensor storage**: Flat `Vec<f64>` with shape metadata
- **Reference counting**: `Arc` for shared structures

---

### Benchmarking

Use the `hedl-bench` crate for performance testing:

```bash
cargo bench --package hedl-bench
```

---

## Thread Safety

All parsing and conversion functions are **thread-safe** and can be called concurrently from multiple threads.

**Document objects** (`Document`, `Node`, etc.) are **not thread-safe**. Use `Arc<Mutex<Document>>` for shared access or `Send` for transfer between threads.

```rust
use std::sync::{Arc, Mutex};

let doc = Arc::new(Mutex::new(hedl::parse(input)?));

// Clone for thread
let doc_clone = Arc::clone(&doc);
std::thread::spawn(move || {
    let d = doc_clone.lock().unwrap();
    println!("Version: {}.{}", d.version.0, d.version.1);
});
```

---

## Constants

```rust
/// HEDL format version supported by this library
pub const SUPPORTED_VERSION: (u32, u32) = (1, 0);

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

---

## Best Practices

### 1. Use `parse_lenient` for User Input

```rust
// For user-provided data with potentially invalid references
let doc = hedl::parse_lenient(user_input)?;
```

### 2. Validate Before Canonicalization

```rust
hedl::validate(input)?;
let doc = hedl::parse(input)?;
let canonical = hedl::canonicalize(&doc)?;
```

### 3. Configure Limits for Untrusted Input

```rust
use hedl::{parse_with_limits, ParseOptions, Limits, ReferenceMode};

let options = ParseOptions {
    reference_mode: ReferenceMode::Strict,
    limits: Limits {
        max_indent_depth: 50,
        max_total_keys: 100_000,
        max_total_ids: 50_000,
        ..Limits::default()
    },
    ..Default::default()
};

let doc = parse_with_limits(untrusted_input.as_bytes(), options)?;
```

### 4. Use Feature Flags to Minimize Dependencies

```toml
[dependencies]
hedl = { version = "1.2", default-features = false }
```

---

## Examples

See the [examples directory](examples.md) for comprehensive code samples.

Quick examples:
- [Quick Start](../../crates/hedl/examples/quick_start.rs)
- [Advanced Features](../../crates/hedl/examples/advanced_features.rs)

---

**Next**: [FFI/C API Reference](ffi-api.md)
