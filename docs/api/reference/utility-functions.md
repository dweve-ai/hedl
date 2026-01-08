# Utility Functions Reference

Helper functions and utilities for working with HEDL documents.

## Document Traversal

Note: Document traversal is available through `hedl-core` but not re-exported at the top level.
You can access these features by directly using `hedl_core::traverse` module.

**Example:**
```rust
use hedl_core::traverse::{traverse, DocumentVisitor, VisitorContext};

struct CountingVisitor {
    count: usize,
}

impl DocumentVisitor for CountingVisitor {
    fn visit_value(&mut self, value: &hedl_core::Value, _ctx: &VisitorContext) {
        self.count += 1;
    }
}

let mut visitor = CountingVisitor { count: 0 };
traverse(&doc, &mut visitor);
println!("Total values: {}", visitor.count);
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
assert_eq!(r2.type_name, Some("User".to_string()));
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
use hedl_core::lex::LexError;

let fields = parse_csv_row("alice, Alice Smith, alice@example.com")?;
assert_eq!(fields.len(), 3);
```

## Tensor Utilities

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

**Example:**
```rust
use hedl::lex::calculate_indent;

let info = calculate_indent("  key: value", 1)?.unwrap();
assert_eq!(info.level, 1);
assert_eq!(info.spaces, 2);
```

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
root.insert("name".to_string(), Item::Scalar(Value::String("Alice".to_string())));
root.insert("age".to_string(), Item::Scalar(Value::Int(30)));

let doc = Document {
    version: (1, 0),
    structs: BTreeMap::new(),
    aliases: BTreeMap::new(),
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
        structs: BTreeMap::new(),
        aliases: BTreeMap::new(),
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
