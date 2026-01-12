# hedl-test

**Shared test fixtures and utilities for HEDL—comprehensive test data, builders, and error cases for consistent testing across all crates.**

Testing format converters requires representative documents. Building test cases by hand is tedious and error-prone. Common test scenarios should be reusable across crates. Edge cases need systematic coverage. Error conditions must be validated consistently.

`hedl-test` provides 15 pre-built fixtures covering all HEDL features, 4 builder types for programmatic document construction, comprehensive error fixtures (15 invalid HEDL, 15 invalid expressions, 8 semantic errors), edge case generators (deep nesting, wide documents, long strings), and 40+ test utilities for assertions and validation.

## What's Implemented

Comprehensive test infrastructure:

1. **15 Pre-Built Fixtures**: Scalars, references, nested structures, matrices, blog, errors, edge cases
2. **4 Builder Types**: DocumentBuilder, MatrixListBuilder, NodeBuilder, ValueBuilder
3. **Error Fixtures**: 15 invalid HEDL samples, 15 invalid expressions, 8 semantic violations
4. **Edge Case Generators**: Deeply nested (1000 levels), wide (1000 fields), long strings (1 MB)
5. **Test Utilities**: count_nodes, count_references, assert_roundtrip, validate_schema
6. **File Fixtures**: 10+ .hedl files in fixtures/ directory
7. **Expression Helpers**: expr(), try_expr() for testing expression evaluation
8. **Reference Helpers**: ref_qualified(), ref_local() for reference construction
9. **Schema Helpers**: struct_def(), nest_def(), alias_def() for header construction
10. **40+ Unit Tests**: Covering all utilities and builders

## Installation

```toml
[dev-dependencies]
hedl-test = "1.0"
```

## Pre-Built Fixtures

### Basic Fixtures

```rust
use hedl_test::fixtures::*;

// Scalar types (all primitives)
let doc = scalars();
// Contains: int, float, bool, null, string examples

// Special strings (escapes, Unicode, etc.)
let doc = special_strings();
// Contains: quotes, escapes, newlines, Unicode

// References (qualified and local)
let doc = references();
// Contains: @Type:id and @local_id examples

// Tensors (matrix literals)
let doc = tensors();
// Contains: 2D and 3D tensor examples
```

### Complex Fixtures

```rust
// User list (simple matrix)
let doc = user_list();
// 3 users: alice, bob, carol with id, name, age

// Blog (nested structure)
let doc = blog();
// Posts with nested comments and tags

// E-commerce (orders with items)
let doc = orders_with_items();
// Orders containing nested line items

// Graph (nodes with references)
let doc = graph();
// Nodes with outgoing edge references

// Nested hierarchy (deep nesting)
let doc = nested_hierarchy();
// Organization → Departments → Teams → Members
```

### Comprehensive Fixture

All HEDL features in one document:

```rust
let doc = comprehensive();
// Contains:
// - All scalar types
// - Qualified and local references
// - Matrix lists with schemas
// - Nested structures (3 levels deep)
// - Expressions
// - Aliases and substitutions
// - All header directives
```

## Builder Types

### DocumentBuilder

Programmatically construct documents:

```rust
use hedl_test::builders::DocumentBuilder;

let doc = DocumentBuilder::new()
    .version(1, 0)
    .struct_def("User", vec!["id", "name", "age"])
    .struct_def("Post", vec!["id", "author", "title"])
    .alias("api_url", "https://api.example.com")
    .nest("User", "Post")
    .field("app_name", "MyApp")
    .field("version", "1.0.0")
    .entity_list("users", "User")
    .build()?;
```

**Methods**:
- `version(major, minor)` - Set %VERSION
- `struct_def(name, fields)` - Add %STRUCT
- `alias(key, value)` - Add %ALIAS
- `nest(parent, child)` - Add %NEST
- `field(key, value)` - Add root field
- `entity_list(key, type_name)` - Start entity list

### MatrixListBuilder

Build typed entity lists:

```rust
use hedl_test::builders::MatrixListBuilder;

let list = MatrixListBuilder::new("User")
    .schema(vec!["id", "name", "age", "active"])
    .node()
        .id("alice")
        .field("Alice Smith")
        .field(30)
        .field(true)
        .build()
    .node()
        .id("bob")
        .field("Bob Jones")
        .field(25)
        .field(false)
        .build()
    .build()?;
```

**Methods**:
- `schema(fields)` - Set column names
- `node()` - Start new node
- `count_hint(n)` - Add count annotation
- `build()` - Finalize list

### NodeBuilder

Build individual entities:

```rust
use hedl_test::builders::NodeBuilder;

let node = NodeBuilder::new("User")
    .id("alice")
    .field("Alice Smith")
    .field(30)
    .field(true)
    .reference_field("manager", "User", "bob")
    .build()?;
```

**Methods**:
- `id(value)` - Set entity ID
- `field(value)` - Add field value
- `reference_field(name, type, id)` - Add reference
- `null_field()` - Add null value
- `build()` - Finalize node

### ValueBuilder

Build complex values:

```rust
use hedl_test::builders::ValueBuilder;

let value = ValueBuilder::string("Hello")
    .or()
    .int(42)
    .or()
    .reference("User", "alice")
    .build()?;
```

**Methods**:
- `string(s)`, `int(i)`, `float(f)`, `bool(b)`, `null()`
- `reference(type, id)` - Qualified reference
- `local_ref(id)` - Local reference
- `expression(expr)` - Expression value

## Error Fixtures

### Invalid HEDL Syntax

```rust
use hedl_test::errors::*;

// Missing version
let hedl = invalid_missing_version();

// Invalid header syntax
let hedl = invalid_struct_syntax();

// Shape mismatch
let hedl = invalid_shape_mismatch();

// Orphan row
let hedl = invalid_orphan_row();

// Invalid indentation
let hedl = invalid_indentation();

// All 15 error cases
let cases = all_invalid_hedl_cases();
```

**Error Categories**:
- Missing/invalid headers
- Syntax errors
- Schema violations
- Reference errors
- Indentation problems

### Invalid Expressions

```rust
// Syntax error
let expr = invalid_expr_syntax();        // "$(1 +"

// Undefined variable
let expr = invalid_expr_undefined();     // "$(undefined_var)"

// Type mismatch
let expr = invalid_expr_type();          // "$(\"string\" + 42)"

// Division by zero
let expr = invalid_expr_div_zero();      // "$(10 / 0)"

// All 15 expression errors
let cases = all_invalid_expr_cases();
```

### Semantic Errors

Valid syntax but semantic violations:

```rust
// Unresolved reference
let hedl = semantic_unresolved_ref();    // References non-existent entity

// Circular reference
let hedl = semantic_circular_ref();      // A → B → C → A

// Type mismatch
let hedl = semantic_type_mismatch();     // Wrong schema applied

// Duplicate ID
let hedl = semantic_duplicate_id();      // Same ID used twice

// All 8 semantic errors
let cases = all_semantic_error_cases();
```

## Edge Case Generators

### Deeply Nested Document

Test parser depth limits:

```rust
use hedl_test::generators::deeply_nested_document;

let doc = deeply_nested_document(1000);
// 1000 levels of nesting: obj1.obj2.obj3...obj1000
```

**Use Case**: Verify MAX_NESTING_DEPTH enforcement

### Wide Document

Test field count limits:

```rust
let doc = wide_document(1000);
// Single object with 1000 fields
```

**Use Case**: Verify large schema handling

### Long Strings

Test string length limits:

```rust
let doc = long_string_document(1024 * 1024);
// 1 MB string value
```

**Use Case**: Verify MAX_STRING_LENGTH enforcement

### Many Entities

Test large entity lists:

```rust
let doc = many_entities_document(100_000);
// 100K entities in single list
```

**Use Case**: Verify streaming parser constant memory

## Test Utilities

### Counting Utilities

```rust
use hedl_test::utils::*;

// Count total entities
let count = count_nodes(&doc);

// Count references
let ref_count = count_references(&doc);

// Count specific type
let user_count = count_type(&doc, "User");

// Count depth
let max_depth = measure_max_depth(&doc);
```

### Validation Utilities

```rust
// Validate schema consistency
assert_schema_valid(&doc, "User", &["id", "name", "age"])?;

// Validate reference resolution
assert_all_refs_resolve(&doc)?;

// Validate no duplicate IDs
assert_no_duplicate_ids(&doc)?;
```

### Roundtrip Utilities

```rust
// Test format roundtrip
assert_roundtrip_json(&doc)?;
assert_roundtrip_yaml(&doc)?;
assert_roundtrip_xml(&doc)?;

// Generic roundtrip with custom convert functions
assert_roundtrip(&doc, to_fmt, from_fmt)?;
```

### Expression Utilities

```rust
// Evaluate expression (panics on error)
let value = expr("$(1 + 2)", &doc);
assert_eq!(value, Value::Int(3));

// Try evaluate (returns Result)
let result = try_expr("$(undefined)", &doc);
assert!(result.is_err());
```

## File Fixtures

Located in `fixtures/` directory:

```rust
use hedl_test::load_fixture;

// Load from fixtures/ dir
let doc = load_fixture("blog.hedl")?;
let doc = load_fixture("scalars.hedl")?;
let doc = load_fixture("tensors.hedl")?;
```

**Available Files**:
- `blog.hedl` - Blog with posts and comments
- `scalars.hedl` - All scalar types
- `special_strings.hedl` - String escapes and Unicode
- `references.hedl` - Reference examples
- `tensors.hedl` - Matrix literals
- `user_list.hedl` - Simple user matrix
- `nested.hedl` - Deep nesting
- `graph.hedl` - Graph with references
- `orders.hedl` - E-commerce orders
- `analytics.hedl` - Time series data

## Usage Patterns

### Testing Parsers

```rust
use hedl_test::fixtures::user_list;

#[test]
fn test_parse_user_list() {
    let doc = user_list();
    assert_eq!(doc.version, (1, 0));
    assert_eq!(doc.entities.len(), 1);
    assert_eq!(doc.entities["User"].len(), 3);
}
```

### Testing Format Converters

```rust
use hedl_test::{fixtures::comprehensive, utils::assert_roundtrip_json};

#[test]
fn test_json_roundtrip() {
    let doc = comprehensive();
    assert_roundtrip_json(&doc).unwrap();
}
```

### Testing Error Handling

```rust
use hedl_test::errors::invalid_shape_mismatch;

#[test]
fn test_shape_mismatch_error() {
    let hedl = invalid_shape_mismatch();
    let result = parse(hedl.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, HedlError::ShapeMismatch { .. }));
}
```

### Testing Edge Cases

```rust
use hedl_test::generators::deeply_nested_document;

#[test]
fn test_max_depth() {
    let doc = deeply_nested_document(1001);
    let result = validate(&doc);
    assert!(result.is_err());
    // Should fail depth limit (1000)
}
```

## What This Crate Doesn't Do

**Property-Based Testing**: Fixtures are hand-crafted examples. For property-based testing, use `proptest` or `quickcheck` with custom generators.

**Performance Testing**: Fixtures designed for correctness testing. For performance, use `hedl-bench` with realistic workloads.

**Fuzz Testing**: No fuzzing infrastructure. For fuzz testing, use `cargo-fuzz` with custom fuzz targets.

**Test Data Generation**: Fixtures are static. For dynamic test data, write custom generators using builders.

## Dependencies

- `hedl-core` 1.0 - Core HEDL implementation

## License

Apache-2.0
