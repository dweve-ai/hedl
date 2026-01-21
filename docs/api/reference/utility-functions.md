# Utility Functions Reference

Helper functions and utilities for working with HEDL documents.

## Document Traversal

Note: Document traversal is available through `hedl-core` but not re-exported at the top level.
You can access these features by directly using `hedl_core::traverse` module.

**Example:**
```rust
use hedl_core::traverse::{traverse, DocumentVisitor, VisitorContext};
use hedl_core::{Value, Node};

struct CountingVisitor {
    scalar_count: usize,
    node_count: usize,
}

impl DocumentVisitor for CountingVisitor {
    type Error = std::convert::Infallible;

    fn visit_scalar(
        &mut self,
        _key: &str,
        _value: &Value,
        _ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        self.scalar_count += 1;
        Ok(())
    }

    fn visit_node(
        &mut self,
        _node: &Node,
        _schema: &[String],
        _ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        self.node_count += 1;
        Ok(())
    }
}

let mut visitor = CountingVisitor { scalar_count: 0, node_count: 0 };
traverse(&doc, &mut visitor)?;
println!("Scalars: {}, Nodes: {}", visitor.scalar_count, visitor.node_count);
```

## Lexical Utilities

### is_valid_id_token

Check if string is valid ID token.

```rust
pub fn is_valid_id_token(s: &str) -> bool
```

**Example:**
```rust
use hedl::lex::is_valid_id_token;

assert!(is_valid_id_token("user123"));
assert!(!is_valid_id_token("123user")); // Can't start with digit
```

### is_valid_key_token

Check if string is valid key token.

```rust
pub fn is_valid_key_token(s: &str) -> bool
```

### is_valid_type_name

Check if string is valid type name.

```rust
pub fn is_valid_type_name(s: &str) -> bool
```

**Example:**
```rust
use hedl::lex::is_valid_type_name;

assert!(is_valid_type_name("User"));
assert!(is_valid_type_name("UserProfile"));
assert!(!is_valid_type_name("user")); // Must start with uppercase
```

### parse_reference

Parse reference string.

```rust
pub fn parse_reference(s: &str) -> Result<Reference, LexError>
```

**Example:**
```rust
use hedl::lex::parse_reference;

let r1 = parse_reference("@alice")?;
assert_eq!(r1.id, "alice");
assert_eq!(r1.type_name, None);

let r2 = parse_reference("@User:alice")?;
assert_eq!(r2.type_name.as_deref(), Some("User"));
assert_eq!(r2.id, "alice");
```

## CSV Utilities

### parse_csv_row

Parse CSV row into fields.

```rust
pub fn parse_csv_row(row: &str) -> Result<Vec<CsvField>, LexError>
```

**Example:**
```rust
use hedl::csv::parse_csv_row;
use hedl::lex::LexError;

let fields = parse_csv_row("alice, Alice Smith, alice@example.com")?;
assert_eq!(fields.len(), 3);
```

## Tensor Utilities

### is_tensor_literal

Quick check if string looks like a tensor literal.

```rust
pub fn is_tensor_literal(s: &str) -> bool
```

Fast check without full validation. Use `parse_tensor` for complete validation.

**Example:**
```rust
use hedl::lex::is_tensor_literal;

assert!(is_tensor_literal("[1, 2, 3]"));
assert!(is_tensor_literal("[[1, 2], [3, 4]]"));
assert!(!is_tensor_literal("hello"));
assert!(!is_tensor_literal("@reference"));
```

### parse_tensor

Parse tensor literal.

```rust
pub fn parse_tensor(s: &str) -> Result<Tensor, LexError>
```

**Example:**
```rust
use hedl::tensor::parse_tensor;
use hedl_core::lex::Tensor;

// 1D array [1, 2, 3, 4]
let t = parse_tensor("[1, 2, 3, 4]")?;
// Returns: Array(vec![Scalar(1.0), Scalar(2.0), Scalar(3.0), Scalar(4.0)])

// 2D array [[1, 2], [3, 4]]
let t2 = parse_tensor("[[1, 2], [3, 4]]")?;
// Returns: Array(vec![Array(vec![Scalar(1.0), Scalar(2.0)]), Array(vec![Scalar(3.0), Scalar(4.0)])])
```

## String Utilities

### strip_comment

Remove comments from line.

```rust
pub fn strip_comment(line: &str) -> &str
```

**Example:**
```rust
use hedl::lex::strip_comment;

let line = "key: value  # this is a comment";
assert_eq!(strip_comment(line), "key: value  ");
```

### calculate_indent

Calculate indentation info from a line.

```rust
pub fn calculate_indent(line: &str, line_num: u32) -> Result<Option<IndentInfo>, LexError>
```

**Parameters:**
- `line`: The line to analyze
- `line_num`: Line number (1-indexed) for error reporting

**Returns:**
- `Ok(None)` if the line is blank (only whitespace)
- `Ok(Some(IndentInfo))` with spaces and level information
- `Err` if indentation uses tabs or odd number of spaces

**Example:**
```rust
use hedl_core::lex::calculate_indent;

let info = calculate_indent("  key: value", 1)?.unwrap();
assert_eq!(info.level, 1);
assert_eq!(info.spaces, 2);
```

### validate_indent

Validate that indent level doesn't exceed maximum.

```rust
pub fn validate_indent(info: IndentInfo, max_depth: usize, line_num: u32) -> Result<(), LexError>
```

**Parameters:**
- `info`: Indentation information to validate
- `max_depth`: Maximum allowed indentation depth
- `line_num`: Line number (1-indexed) for error reporting

**Returns:**
- `Ok(())` if indent is within limits
- `Err(LexError::IndentTooDeep)` if indentation exceeds maximum

**Example:**
```rust
use hedl_core::lex::{calculate_indent, validate_indent};

let info = calculate_indent("    nested: value", 5)?.unwrap();
validate_indent(info, 10, 5)?; // OK, level 2 is within max of 10
```

## String Transformation Utilities

### singularize_and_capitalize

Convert pluralized snake_case to singular PascalCase.

```rust
pub fn singularize_and_capitalize(s: &str) -> String
```

Useful when converting from formats like JSON/XML/YAML where collection keys are often pluralized (e.g., "users", "posts") but HEDL struct types should be singular PascalCase (e.g., "User", "Post").

**Singularization Rules:**
- `-ies` suffix: `categories` → `category`
- `-es` suffix (after x/s/sh/ch): `boxes` → `box`, `classes` → `class`
- `-s` suffix: `users` → `user`
- Snake case conversion: `user_posts` → `UserPost`

**Example:**
```rust
use hedl::lex::singularize_and_capitalize;

assert_eq!(singularize_and_capitalize("users"), "User");
assert_eq!(singularize_and_capitalize("categories"), "Category");
assert_eq!(singularize_and_capitalize("user_posts"), "UserPost");
assert_eq!(singularize_and_capitalize("boxes"), "Box");
```

## Reference Parsing Utilities

### parse_reference_at

Parse reference with source position information.

```rust
pub fn parse_reference_at(s: &str, pos: SourcePos) -> Result<Reference, LexError>
```

Same as `parse_reference`, but allows specifying the source position for better error messages.

**Parameters:**
- `s`: Reference string (with or without leading `@`)
- `pos`: Source position for error reporting

**Example:**
```rust
use hedl_core::lex::{parse_reference_at, SourcePos};

let pos = SourcePos::new(42, 10);
let r = parse_reference_at("@User:user_1", pos)?;
assert_eq!(r.type_name.as_deref(), Some("User"));
assert_eq!(r.id, "user_1");
```

## Region Scanning Utilities

### scan_regions

Scan line for protected regions (quoted strings and expressions).

```rust
pub fn scan_regions(line: &str) -> Vec<Region>
```

Identifies regions where special characters like `#` and `,` lose their usual meaning. Returns regions for quoted strings (`"..."`) and expressions (`$(...)`).

**Returns:**
- Vector of `Region` structs with start/end positions and type

**Example:**
```rust
use hedl::lex::scan_regions;

let regions = scan_regions(r#"name: "John", age: $(years)"#);
// Returns 2 regions: one Quote region for "John", one Expression region for $(years)
```

## Expression Parsing Utilities

### parse_expression

Parse expression from content inside `$(...)`.

```rust
pub fn parse_expression(s: &str) -> Result<Expression, LexError>
```

Parses the expression grammar:
- Identifiers: `x`, `foo_bar`
- Literals: `42`, `3.5`, `"hello"`, `true`, `false`
- Function calls: `func(arg1, arg2)`
- Field access: `target.field`

**Example:**
```rust
use hedl_core::lex::parse_expression;

// Parse identifier
let expr = parse_expression("foo")?;

// Parse function call
let expr = parse_expression("concat(a, b)")?;

// Parse field access
let expr = parse_expression("user.name")?;
```

### parse_expression_token

Parse expression content from a `$(...)` token.

```rust
pub fn parse_expression_token(s: &str) -> Result<Expression, LexError>
```

Extracts content between `$(` and `)` and parses it. Handles nested parentheses and quotes.

**Example:**
```rust
use hedl_core::lex::parse_expression_token;

let expr = parse_expression_token("$(now())")?;
let expr = parse_expression_token("$(concat(\"a\", \"b\"))")?;
let expr = parse_expression_token("$(user.profile.name)")?;
```

## Type Inference Utilities

### Inference in hedl-core

Type inference functions are available in `hedl-core::lex` module but not re-exported by the `hedl` facade. For custom type inference, use `hedl-core` directly:

```rust
use hedl_core::lex::infer_value;
use std::collections::HashMap;

// Infer value from string in Key-Value context
// Follows the inference ladder (HEDL Spec Section 8.2)
let result = infer_value("42", None)?;  // Returns Value::Int(42)
```

For most use cases, type inference is handled automatically during parsing via the main `hedl::parse()` function. Custom inference is only needed for special scenarios.

**Note**: The `infer_value()` in `hedl-core` differs from the main `inference::infer_value()` in core (which uses `InferenceContext`). For detailed inference API, see hedl-core documentation.

## Reference Resolution

Note: Reference resolution is handled internally during parsing.
For custom reference resolution, use `hedl_core::reference` module functions.

**Example:**
```rust
use hedl::{parse, parse_lenient};

// References are resolved during parsing
let doc = parse(input)?;

// For lenient parsing (unresolved refs become null):
let doc = parse_lenient(input)?;
```

## Type Inference

Type inference is handled automatically during parsing. String values are parsed
according to HEDL syntax rules:

- `null` → `Value::Null`
- `true`/`false` → `Value::Bool`
- Integer literals → `Value::Int`
- Float literals → `Value::Float`
- `@id` or `@Type:id` → `Value::Reference`
- `[...]` → `Value::Tensor`
- `$(...)` → `Value::Expression`
- Everything else → `Value::String`

## Document Construction

Documents are typically constructed by parsing HEDL text or programmatically:

**Example:**
```rust
use hedl::{Document, Item, Value};
use std::collections::BTreeMap;

// Programmatic construction
let mut root = BTreeMap::new();
root.insert("name".to_string(), Item::Scalar(Value::String("Alice".into())));
root.insert("age".to_string(), Item::Scalar(Value::Int(30)));

let doc = Document {
    version: (1, 0),
    schema_versions: BTreeMap::new(),
    aliases: BTreeMap::new(),
    structs: BTreeMap::new(),
    nests: BTreeMap::new(),
    root,
};
```

## Comparison Utilities

Documents can be compared using Rust's standard `PartialEq` trait:

**Example:**
```rust
use hedl::parse;

let doc1 = parse(input1)?;
let doc2 = parse(input2)?;

if doc1 == doc2 {
    println!("Documents are equivalent");
}

// For canonical comparison, use canonicalization
use hedl::canonicalize;

let canonical1 = canonicalize(&doc1)?;
let canonical2 = canonicalize(&doc2)?;

if canonical1 == canonical2 {
    println!("Documents are canonically equivalent");
}
```

## Merging and Transformation

Document merging and transformation should be implemented by your application logic.
You can iterate over document fields and merge them as needed:

**Example:**
```rust
use hedl::{Document, Item};
use std::collections::BTreeMap;

fn merge_documents(docs: &[Document]) -> Document {
    let mut merged_root = BTreeMap::new();

    for doc in docs {
        for (key, item) in &doc.root {
            merged_root.insert(key.clone(), item.clone());
        }
    }

    Document {
        version: (1, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root: merged_root,
    }
}
```

## Validation Helpers

Validation is primarily handled through linting:

**Example:**
```rust
use hedl::{parse, lint};

let doc = parse(input)?;
let diagnostics = lint(&doc);

for diagnostic in diagnostics {
    println!("[{:?}] {}", diagnostic.severity(), diagnostic.message());
}
```

## Performance Utilities

Documents implement `Clone`, so you can use standard Rust cloning:

**Example:**
```rust
let doc = parse(input)?;
let cloned_doc = doc.clone();

// For size estimation, use std::mem
use std::mem;
let size_estimate = mem::size_of_val(&doc);
```

## Debugging Utilities

Use Rust's Debug trait for debugging:

**Example:**
```rust
use hedl::parse;

let doc = parse(input)?;

// Pretty-print for debugging
println!("{:#?}", doc);

// Or use canonicalization for readable output
use hedl_c14n::canonicalize;
let canonical = canonicalize(&doc)?;
println!("{}", canonical);
```

## See Also

- [Core Types](core-types.md) - Type definitions
- [Parser API](parser-api.md) - Parsing functions
- [Serializer API](serializer-api.md) - Serialization functions
- [Rust API Reference](../rust-api.md) - Complete API
