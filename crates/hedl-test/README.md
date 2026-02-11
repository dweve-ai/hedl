# hedl-test

**Shared test fixtures and utilities for HEDL -comprehensive test data, builders, and error cases for consistent testing across all crates.**

Testing format converters requires representative documents. Building test cases by hand is tedious and error-prone. Common test scenarios should be reusable across crates. Edge cases need systematic coverage. Error conditions must be validated consistently.

`hedl-test` provides 15 pre-built fixtures covering all HEDL features, 4 builder types for programmatic document construction, comprehensive error fixtures (15 invalid HEDL, 15 invalid expressions, 8 semantic violations), and edge case generators (deep nesting, wide documents, long strings, many references).

## What's Implemented

Comprehensive test infrastructure:

1. **15 Pre-Built Fixtures**: Scalars, special strings, references, expressions, tensors, named values, user list, mixed type list, with references, with nest, deep nest, edge cases, comprehensive, blog, empty
2. **4 Builder Types**: DocumentBuilder, MatrixListBuilder, NodeBuilder, ValueBuilder
3. **Error Fixtures**: 15 invalid HEDL samples, 15 invalid expressions, 8 semantically invalid documents
4. **Edge Case Generators**: deeply_nested_document, wide_document, long_string_document, many_references_document
5. **Counting Utilities**: count_nodes, count_references
6. **Expression Helpers**: expr, try_expr, expr_value, try_expr_value with ExprError type

## Installation

```toml
[dev-dependencies]
hedl-test = { workspace = true }
```

Or specify version directly:

```toml
[dev-dependencies]
hedl-test = "2.0"
```

## Pre-Built Fixtures

### Basic Fixtures

```rust
use hedl_test::fixtures;

// Scalar types (all primitives)
let doc = fixtures::scalars();
// Contains: int, float, bool, null, string examples

// Special strings (escapes, Unicode, etc.)
let doc = fixtures::special_strings();
// Contains: quotes, escapes, newlines, Unicode

// References (qualified and local)
let doc = fixtures::references();
// Contains: local references (@id) and typed references (@Type:id)

// Tensors (matrix literals)
let doc = fixtures::tensors();
// Contains: 1D, 2D, 3D tensors and empty tensor
```

### Complex Fixtures

```rust
use hedl_test::fixtures;

// User list (simple matrix)
let doc = fixtures::user_list();
// 3 users: alice, bob, charlie with id, name, email

// Mixed type list
let doc = fixtures::mixed_type_list();
// Items with int, float, bool, null, and string fields

// With references
let doc = fixtures::with_references();
// Users and Posts with author references

// With NEST hierarchy
let doc = fixtures::with_nest();
// Users with nested posts (parent-child via NEST)

// Deep NEST hierarchy
let doc = fixtures::deep_nest();
// Organization → Department → Employee (3 levels)

// Blog (nested structure)
let doc = fixtures::blog();
// Complex blog platform: users, categories, tags, posts, comments, reactions, post_tags, followers
```

### Comprehensive Fixture

All HEDL features in one document:

```rust
use hedl_test::fixtures;

let doc = fixtures::comprehensive();
// Contains:
// - Scalar values (bool, string, int, float)
// - Expressions (function calls, identifiers, literals)
// - Tensors
// - Users list with nested posts (via NEST)
// - Comments and Tags lists
// - References between entities
// - Multiple STRUCT definitions
// - NEST directive
```

## Builder Types

### DocumentBuilder

Programmatically construct documents:

```rust
use hedl_test::fixtures::builders::DocumentBuilder;
use hedl_core::Value;

let doc = DocumentBuilder::new()
    .version(1, 0)
    .struct_def("User", vec!["id".to_string(), "name".to_string(), "email".to_string()])
    .alias("api_url", "https://api.example.com")
    .nest("User", "Post")
    .scalar("app_name", Value::String("MyApp".to_string().into()))
    .scalar("debug", Value::Bool(true))
    .build();
```

**Methods**:
- `version(major, minor)` - Set document version
- `struct_def(name, fields)` - Add struct definition
- `alias(alias, target)` - Add alias
- `nest(parent, child)` - Add NEST relationship
- `scalar(name, value)` - Add scalar value to root
- `list(name, list)` - Add MatrixList to root
- `item(name, item)` - Add Item to root
- `build()` - Build the Document

### MatrixListBuilder

Build typed entity lists:

```rust
use hedl_test::fixtures::builders::MatrixListBuilder;
use hedl_core::{Node, Value};

let list = MatrixListBuilder::new("User")
    .schema(vec!["id".to_string(), "name".to_string(), "age".to_string()])
    .row(Node::new("User", "alice", vec![
        Value::String("alice".to_string().into()),
        Value::String("Alice Smith".to_string().into()),
        Value::Int(30),
    ]))
    .row(Node::new("User", "bob", vec![
        Value::String("bob".to_string().into()),
        Value::String("Bob Jones".to_string().into()),
        Value::Int(25),
    ]))
    .build();
```

**Methods**:
- `schema(fields)` - Set schema (all field names)
- `field(field)` - Add single field to schema
- `row(node)` - Add a row (node)
- `rows(nodes)` - Add multiple rows
- `count_hint(count)` - Set count hint
- `build()` - Build the MatrixList

### NodeBuilder

Build individual entities:

```rust
use hedl_test::fixtures::builders::NodeBuilder;
use hedl_core::Value;

let node = NodeBuilder::new("User", "alice")
    .field(Value::String("alice".to_string().into()))
    .field(Value::String("Alice Smith".to_string().into()))
    .field(Value::Int(30))
    .build();
```

**Methods**:
- `field(value)` - Add a field value
- `fields(values)` - Add multiple field values
- `children(rel_name, nodes)` - Add child nodes under a relationship
- `child(rel_name, node)` - Add single child node
- `child_count(count)` - Set child count hint
- `build()` - Build the Node

### ValueBuilder

Static helper methods for creating values:

```rust
use hedl_test::fixtures::builders::ValueBuilder;

// Scalar values
let v = ValueBuilder::null();
let v = ValueBuilder::bool_val(true);
let v = ValueBuilder::int(42);
let v = ValueBuilder::float(3.14);
let v = ValueBuilder::string("Hello");

// References
let v = ValueBuilder::reference("User", "alice");  // Qualified reference
let v = ValueBuilder::local_ref("some_id");        // Local reference

// Tensors
let v = ValueBuilder::tensor_1d(vec![1.0, 2.0, 3.0]);
let v = ValueBuilder::tensor_2d(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
```

**Methods**:
- `null()` - Create null value
- `bool_val(value)` - Create boolean value
- `int(value)` - Create integer value
- `float(value)` - Create float value
- `string(value)` - Create string value
- `reference(type_name, id)` - Create qualified reference
- `local_ref(id)` - Create local reference
- `tensor_1d(values)` - Create 1D tensor
- `tensor_2d(rows)` - Create 2D tensor

## Error Fixtures

### Invalid HEDL Syntax

```rust
use hedl_test::fixtures::errors;

// Get all invalid HEDL samples as (name, hedl_text) pairs
let samples = errors::invalid_hedl_samples();
for (name, hedl_text) in samples {
    // Test parser with invalid input
    // Each should fail to parse
}
```

**Available samples** (15 total):
- `empty` - Empty document
- `whitespace_only` - Only whitespace
- `invalid_directive` - Invalid directive name
- `missing_separator` - Missing --- separator
- `invalid_version` - Invalid version number
- `malformed_struct` - Malformed STRUCT directive
- `unclosed_string` - Unclosed string literal
- `invalid_escape` - Invalid escape sequence
- `malformed_reference` - Malformed reference
- `invalid_tensor` - Malformed tensor syntax
- `mismatched_brackets` - Mismatched brackets
- `invalid_number` - Invalid number format
- `malformed_expression` - Malformed expression
- `invalid_identifier` - Invalid identifier
- `duplicate_directive` - Duplicate directive

### Invalid Expressions

```rust
use hedl_test::fixtures::errors;

// Get all invalid expression samples as (description, expr_string) pairs
let samples = errors::invalid_expression_samples();
for (desc, expr_str) in samples {
    // Test expression parser with invalid input
    // Each should fail to parse
}
```

**Available samples** (15 total):
- `empty` - Empty expression
- `whitespace_only` - Only whitespace
- `unclosed_paren` - Unclosed parenthesis
- `unclosed_string` - Unclosed string literal
- `invalid_chars` - Invalid characters
- `double_dot` - Double dot in path
- `trailing_dot` - Trailing dot
- `leading_dot` - Leading dot
- `mismatched_parens` - Mismatched parentheses
- `empty_call` - Empty function call
- `invalid_identifier` - Invalid identifier
- `special_chars_only` - Only special characters
- `nested_unclosed` - Nested unclosed parentheses
- `comma_without_args` - Comma without arguments
- `trailing_comma` - Trailing comma

### Semantic Errors

Valid syntax but semantic violations:

```rust
use hedl_test::fixtures::errors;

// Get all semantically invalid documents as (name, document) pairs
let docs = errors::semantically_invalid_docs();
for (name, doc) in docs {
    // Test validation logic with semantically invalid documents
    // Each should fail validation but parse correctly
}
```

**Available documents** (8 total):
- `undefined_struct` - MatrixList references undefined struct
- `undefined_nest` - NEST directive references non-existent child type
- `circular_nest` - Circular NEST references (TypeA → TypeB → TypeA)
- `dangling_reference` - Reference points to non-existent node
- `mismatched_schema` - MatrixList schema doesn't match struct definition
- `empty_type_name` - MatrixList has empty type name
- `duplicate_ids` - Multiple nodes with same ID in a list
- `invalid_alias` - Alias references non-existent item

## Edge Case Generators

### Deeply Nested Document

Test parser depth limits:

```rust
use hedl_test::fixtures::errors;

let doc = errors::deeply_nested_document(1000);
// Creates 1000 levels of nested children via NEST
```

**Use Case**: Verify recursion depth limits and stack safety

### Wide Document

Test handling of many entities:

```rust
use hedl_test::fixtures::errors;

let doc = errors::wide_document(1000);
// Creates a MatrixList with 1000 rows
```

**Use Case**: Verify large list handling and memory efficiency

### Long Strings

Test string length limits:

```rust
use hedl_test::fixtures::errors;

let doc = errors::long_string_document(1024 * 1024);
// Creates a document with a 1 MB string value
```

**Use Case**: Verify string buffer limits and allocation safety

### Many References

Test reference resolution performance:

```rust
use hedl_test::fixtures::errors;

let doc = errors::many_references_document(10_000);
// Creates 10K target nodes and 10K references
```

**Use Case**: Verify reference resolution performance and correctness

## Test Utilities

### Counting Utilities

```rust
use hedl_test::{count_nodes, count_references};

// Count total nodes (entities) in document
let count = count_nodes(&doc);

// Count total references in document
let ref_count = count_references(&doc);
```

**Available functions**:
- `count_nodes(&doc)` - Count all nodes in the document (including nested children)
- `count_references(&doc)` - Count all reference values in the document

### Expression Utilities

```rust
use hedl_test::{expr, try_expr, expr_value, try_expr_value, ExprError};

// Create Expression from string (panics on error)
let e = expr("42");
let e = expr("now()");
let e = expr("user.name");

// Create Expression safely (returns Result)
let result = try_expr("invalid!!!");
match result {
    Ok(e) => println!("Parsed: {:?}", e),
    Err(ExprError::ParseFailed { source, input }) => {
        println!("Failed to parse '{}': {}", input, source);
    }
    Err(ExprError::EmptyInput) => println!("Empty input"),
    Err(ExprError::Missing) => println!("Missing expression"),
}

// Create Value::Expression from string
let v = expr_value("42");        // Panics on error
let v = try_expr_value("42")?;   // Returns Result
```

**Available functions**:
- `expr(s)` - Parse expression string, panic on error
- `try_expr(s)` - Parse expression string, return Result
- `expr_value(s)` - Create Value::Expression, panic on error
- `try_expr_value(s)` - Create Value::Expression, return Result

**Error type**:
- `ExprError::EmptyInput` - Expression string is empty
- `ExprError::ParseFailed { source, input }` - Parse failed with lexical error
- `ExprError::Missing` - Expression is missing or null

## Quick Builder Helpers

The `fixtures::builders::quick` module provides convenient helper functions for common patterns:

```rust
use hedl_test::fixtures::builders::quick;

// Create document with simple scalar fields
let doc = quick::simple_scalars(vec![
    ("name", "Alice"),
    ("city", "NYC"),
]);

// Create document with simple user list
let doc = quick::simple_user_list(vec![
    ("alice", "Alice Smith", "alice@example.com"),
    ("bob", "Bob Jones", "bob@example.com"),
]);

// Create document with references
let doc = quick::with_references(
    vec![("alice", "Alice"), ("bob", "Bob")],  // users
    vec![("post1", "Title", "alice")],          // posts with author refs
);
```

## Usage Patterns

### Testing Parsers

```rust
use hedl_test::fixtures;

#[test]
fn test_parse_user_list() {
    let doc = fixtures::user_list();
    assert_eq!(doc.version, (1, 0));

    if let Some(hedl_core::Item::List(list)) = doc.root.get("users") {
        assert_eq!(list.rows.len(), 3);
        assert_eq!(list.type_name, "User");
    }
}
```

### Testing Format Converters

```rust
use hedl_test::fixtures;

#[test]
fn test_all_fixtures() {
    for (name, fixture_fn) in fixtures::all() {
        let doc = fixture_fn();

        // Test your converter with each fixture
        let json = to_json(&doc).unwrap();
        let back = from_json(&json).unwrap();

        // Verify roundtrip
        assert_eq!(doc, back, "Fixture {} failed roundtrip", name);
    }
}
```

### Testing Error Handling

```rust
use hedl_test::fixtures::errors;

#[test]
fn test_invalid_hedl_samples() {
    for (name, hedl_text) in errors::invalid_hedl_samples() {
        let result = parse(hedl_text);
        assert!(result.is_err(), "Sample {} should fail to parse", name);
    }
}

#[test]
fn test_semantic_errors() {
    for (name, doc) in errors::semantically_invalid_docs() {
        let result = validate(&doc);
        assert!(result.is_err(), "Document {} should fail validation", name);
    }
}
```

### Testing Edge Cases

```rust
use hedl_test::fixtures::errors;
use hedl_test::count_nodes;

#[test]
fn test_deeply_nested() {
    let doc = errors::deeply_nested_document(100);
    let count = count_nodes(&doc);

    // Verify deep nesting is handled correctly
    assert!(count > 0);
}

#[test]
fn test_many_references() {
    let doc = errors::many_references_document(1000);

    // Test reference resolution performance
    let result = resolve_references(&doc);
    assert!(result.is_ok());
}
```

## What This Crate Doesn't Do

**Property-Based Testing**: Fixtures are hand-crafted examples. For property-based testing, use `proptest` or `quickcheck` with custom generators.

**Performance Testing**: Fixtures designed for correctness testing. For performance, use `hedl-bench` with realistic workloads.

**Fuzz Testing**: No fuzzing infrastructure. For fuzz testing, use `cargo-fuzz` with custom fuzz targets.

**Test Data Generation**: Fixtures are static. For dynamic test data, write custom generators using builders.

## Dependencies

- `hedl-core` - Core HEDL types and parsing
- `hedl-c14n` - Canonicalization for HEDL documents
- `smallvec` - Optimized small vector implementation

## License

Apache-2.0
