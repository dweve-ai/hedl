# Testing Guide

Comprehensive guide to testing strategies, running tests, and writing new tests for HEDL.

## Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Test Organization](#test-organization)
3. [Running Tests](#running-tests)
4. [Unit Testing](#unit-testing)
5. [Integration Testing](#integration-testing)
6. [Property-Based Testing](#property-based-testing)
7. [Fuzz Testing](#fuzz-testing)
8. [Conformance Testing](#conformance-testing)
9. [Error Path Testing](#error-path-testing)
10. [Test Utilities](#test-utilities)

---

## Testing Philosophy

HEDL follows comprehensive testing practices:

### Core Principles

1. **Test-Driven Development (TDD)**: Write tests before implementation
2. **Comprehensive Coverage**: Test happy paths, edge cases, and error paths
3. **Fast Feedback**: Tests should run quickly
4. **Deterministic**: Tests must be reproducible
5. **Isolated**: Tests should not depend on each other
6. **Documented**: Tests serve as usage examples

### Testing Pyramid

```
         /\
        /  \  E2E Tests (few, slow)
       /____\
      /      \
     / Integ. \ Integration Tests (moderate)
    /__________\
   /            \
  /   Unit Tests \ Unit Tests (many, fast)
 /________________\
```

### Coverage Goals

- **Unit tests**: >90% coverage
- **Integration tests**: All module interactions
- **Edge cases**: All boundary conditions
- **Error paths**: All error conditions
- **Property tests**: Invariants and roundtrips

---

## Test Organization

### Directory Structure

```
hedl/
├── crates/
│   ├── hedl-core/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── parser.rs
│   │   └── tests/
│   │       ├── unit/           # Unit tests
│   │       ├── integration/    # Integration tests
│   │       └── property/       # Property-based tests
│   ├── hedl-json/
│   │   └── tests/
│   │       ├── conversion_tests.rs
│   │       └── property_tests.rs
│   └── ...
├── tests/
│   ├── conformance/            # Spec compliance tests
│   ├── integration/            # Cross-crate integration
│   └── regression/             # Regression tests
└── fixtures/
    ├── examples/               # Example files
    ├── conformance/            # Spec test cases
    └── fuzz/                   # Fuzz test corpus
```

### Test File Naming

```rust
// Unit tests (in src files)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_value() { }
}

// Integration tests (in tests/)
// tests/integration_tests.rs
#[test]
fn test_json_roundtrip() { }

// Property tests
// tests/property_tests.rs
proptest! {
    #[test]
    fn test_parse_any_valid_input(input in arb_hedl()) { }
}
```

---

## Running Tests

### Basic Commands

```bash
# All tests
cargo test --all

# Specific crate
cargo test -p hedl-core

# Specific test
cargo test test_parse_simple

# With output
cargo test -- --nocapture

# With backtrace
RUST_BACKTRACE=1 cargo test

# Single-threaded
cargo test -- --test-threads=1
```

### Test Filtering

```bash
# Only unit tests
cargo test --lib

# Only integration tests
cargo test --test '*'

# Only doc tests
cargo test --doc

# By name pattern
cargo test parse

# Ignored tests
cargo test -- --ignored

# All tests including ignored
cargo test -- --include-ignored
```

### Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --all --out Html

# Open report
open tarpaulin-report.html
```

### Continuous Testing

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-run tests on file changes
cargo watch -x test

# With clear screen
cargo watch -c -x test
```

---

## Unit Testing

Unit tests verify individual functions and methods.

### Basic Unit Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_key_token() {
        assert!(is_valid_key_token("name"));
        assert!(is_valid_key_token("user_id"));
        assert!(is_valid_key_token("_private"));

        assert!(!is_valid_key_token(""));
        assert!(!is_valid_key_token("123"));
        assert!(!is_valid_key_token("with-dash"));
    }
}
```

### Testing Errors

```rust
#[test]
fn test_parse_invalid_reference() {
    let result = parse_reference("invalid");
    assert!(result.is_err());

    match result {
        Err(LexError::InvalidReference { .. }) => {
            // Expected error
        }
        _ => panic!("Expected InvalidReference error"),
    }
}

// Using assert_matches
#[test]
fn test_parse_error_with_matches() {
    use assert_matches::assert_matches;

    let result = parse("invalid");
    assert_matches!(result, Err(HedlError { kind: HedlErrorKind::Syntax, .. }));
}
```

### Parameterized Tests

```rust
#[test]
fn test_value_inference() {
    let test_cases = vec![
        ("42", Value::Int(42)),
        ("3.14", Value::Float(3.14)),
        ("true", Value::Bool(true)),
        ("null", Value::Null),
        ("hello", Value::String("hello".to_string())),
    ];

    for (input, expected) in test_cases {
        assert_eq!(infer_value(input), expected);
    }
}

// Using rstest
use rstest::rstest;

#[rstest]
#[case("42", Value::Int(42))]
#[case("3.14", Value::Float(3.14))]
#[case("true", Value::Bool(true))]
fn test_infer_value(#[case] input: &str, #[case] expected: Value) {
    assert_eq!(infer_value(input), expected);
}
```

### Test Fixtures

```rust
// Create test fixture
fn sample_document() -> Document {
    let mut root = BTreeMap::new();
    root.insert("name".to_string(), Item::Scalar(Value::String("Alice".to_string())));
    root.insert("age".to_string(), Item::Scalar(Value::Int(30)));

    Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

#[test]
fn test_with_fixture() {
    let doc = sample_document();
    assert_eq!(doc.root.len(), 2);
}
```

---

## Integration Testing

Integration tests verify interactions between modules.

### Cross-Module Testing

```rust
// tests/integration_tests.rs
use hedl_core::parse;
use hedl_json::{hedl_to_json, json_to_hedl};

#[test]
fn test_parse_to_json_roundtrip() {
    let hedl_text = b"%VERSION: 1.0
---
name: Alice
age: 30
";

    // Parse HEDL
    let doc = parse(hedl_text).unwrap();

    // Convert to JSON
    let json = hedl_to_json(&doc).unwrap();

    // Parse JSON
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify
    assert_eq!(json_value["name"], "Alice");
    assert_eq!(json_value["age"], 30);
}
```

### Format Conversion Testing

```rust
use hedl_core::parse;
use hedl_json::{hedl_to_json, json_to_hedl};
use hedl_c14n::canonicalize;

#[test]
fn test_hedl_json_roundtrip() {
    let original = b"%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User
  | alice, Alice, 30
  | bob, Bob, 25
";

    // HEDL -> JSON -> HEDL
    let doc1 = parse(original).unwrap();
    let json = hedl_to_json(&doc1).unwrap();
    let doc2 = json_to_hedl(&json).unwrap();
    let hedl1 = canonicalize(&doc1).unwrap();
    let hedl2 = canonicalize(&doc2).unwrap();

    // Should be equivalent
    assert_eq!(hedl1, hedl2);
}
```

### Error Handling Integration

```rust
use hedl_core::{parse, HedlError, HedlErrorKind};

#[test]
fn test_error_propagation() {
    let invalid_hedl = b"%VERSION: 1.0
---
name: @User:missing
";

    let result = parse(invalid_hedl);
    assert!(result.is_err());

    match result {
        Err(err) if err.kind == HedlErrorKind::Reference => {
            assert!(err.to_string().contains("missing"));
        }
        _ => panic!("Expected reference error"),
    }
}
```

---

## Property-Based Testing

Property-based tests verify invariants across random inputs.

### Basic Property Test

```rust
use proptest::prelude::*;
use hedl_core::parse;

proptest! {
    #[test]
    fn test_parse_never_panics(s in ".*") {
        let _ = parse(s.as_bytes()); // Should not panic
    }

    #[test]
    fn test_valid_key_tokens(s in "[a-zA-Z_][a-zA-Z0-9_]*") {
        use hedl_core::lex::is_valid_key_token;
        assert!(is_valid_key_token(&s));
    }
}
```

### Roundtrip Properties

```rust
use hedl_core::parse;
use hedl_c14n::canonicalize;
use hedl_json::{hedl_to_json, json_to_hedl};

proptest! {
    #[test]
    fn test_canonicalize_roundtrip(doc in arb_document()) {
        let hedl = canonicalize(&doc)?;
        let parsed = parse(hedl.as_bytes())?;
        let hedl2 = canonicalize(&parsed)?;

        // Canonical form is stable
        prop_assert_eq!(hedl, hedl2);
    }

    #[test]
    fn test_json_roundtrip(doc in arb_document()) {
        let json = hedl_to_json(&doc)?;
        let parsed = json_to_hedl(&json)?;
        let json2 = hedl_to_json(&parsed)?;

        // JSON roundtrip preserves structure
        let v1: serde_json::Value = serde_json::from_str(&json)?;
        let v2: serde_json::Value = serde_json::from_str(&json2)?;
        prop_assert_eq!(v1, v2);
    }
}
```

### Custom Generators

```rust
use proptest::prelude::*;
use hedl_core::{Document, Value, Item};
use std::collections::BTreeMap;

// Generate valid HEDL documents
fn arb_document() -> impl Strategy<Value = Document> {
    prop::collection::btree_map("[a-zA-Z_][a-zA-Z0-9_]*", arb_item(), 0..10)
        .prop_map(|root| Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        })
}

fn arb_item() -> impl Strategy<Value = Item> {
    prop_oneof![
        arb_value().prop_map(Item::Scalar),
    ]
}

fn arb_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        "[a-zA-Z]+".prop_map(Value::String),
        any::<i64>().prop_map(Value::Int),
        any::<bool>().prop_map(Value::Bool),
        Just(Value::Null),
    ]
}
```

### Invariant Testing

```rust
proptest! {
    #[test]
    fn test_schema_validation_invariant(
        schema in arb_schema(),
        row in arb_csv_row()
    ) {
        let result = validate_row(&row, &schema);

        // Invariant: validation result must be deterministic
        let result2 = validate_row(&row, &schema);
        prop_assert_eq!(result.is_ok(), result2.is_ok());
    }
}
```

---

## Fuzz Testing

Fuzz testing finds bugs through random inputs.

### Setting Up Fuzzing

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Create fuzz target
cargo fuzz init

# List fuzz targets
cargo fuzz list

# Run fuzzer
cargo fuzz run parse

# With timeout
cargo fuzz run parse -- -max_total_time=60

# Check coverage
cargo fuzz coverage parse
```

### Fuzz Target Example

```rust
// fuzz/fuzz_targets/parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use hedl_core::parse;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse(s); // Should not panic
    }
});
```

### Structured Fuzzing

```rust
// fuzz/fuzz_targets/parse_structured.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct HedlInput {
    version: String,
    schemas: Vec<Schema>,
    data: Vec<Object>,
}

fuzz_target!(|input: HedlInput| {
    let hedl = generate_hedl(&input);
    let _ = parse(&hedl);
});
```

### Regression Testing from Corpus

```rust
#[test]
fn test_fuzz_corpus() {
    let corpus_dir = "fuzz/corpus/parse";

    for entry in std::fs::read_dir(corpus_dir).unwrap() {
        let path = entry.unwrap().path();
        let data = std::fs::read(&path).unwrap();

        if let Ok(s) = std::str::from_utf8(&data) {
            // Should not panic
            let _ = parse(s);
        }
    }
}
```

---

## Conformance Testing

Verify specification compliance.

### Conformance Test Suite

```rust
// tests/conformance/mod.rs
use serde::Deserialize;

#[derive(Deserialize)]
struct ConformanceTest {
    name: String,
    input: String,
    expected: ExpectedResult,
}

#[derive(Deserialize)]
enum ExpectedResult {
    Success { output: serde_json::Value },
    Error { error_type: String },
}

#[test]
fn test_conformance_suite() {
    let tests: Vec<ConformanceTest> =
        serde_json::from_str(include_str!("conformance.json")).unwrap();

    for test in tests {
        verify_conformance(&test);
    }
}

fn verify_conformance(test: &ConformanceTest) {
    let result = parse(&test.input);

    match &test.expected {
        ExpectedResult::Success { output } => {
            let doc = result.expect(&format!("Test '{}' should succeed", test.name));
            let json = to_json(&doc).unwrap();
            let actual: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(&actual, output, "Test '{}' output mismatch", test.name);
        }
        ExpectedResult::Error { error_type } => {
            assert!(
                result.is_err(),
                "Test '{}' should fail with {}",
                test.name,
                error_type
            );
        }
    }
}
```

---

## Error Path Testing

Test all error conditions.

### Error Coverage

```rust
use hedl_core::parse;

#[test]
fn test_parse_errors() {
    let error_cases: Vec<(&[u8], &str)> = vec![
        // Invalid reference
        (b"%VERSION: 1.0\n---\nname: @\n", "reference"),

        // Missing schema
        (b"%VERSION: 1.0\n---\nusers: @Unknown\n  | alice\n", "schema"),

        // Column mismatch
        (b"%VERSION: 1.0\n%STRUCT: User: [id]\n---\nusers: @User\n  | a, b\n", "column"),

        // Missing version
        (b"---\nname: Alice\n", "version"),
    ];

    for (input, expected_error_substring) in error_cases {
        let result = parse(input);
        assert!(
            result.is_err(),
            "Expected error for input containing: {}",
            expected_error_substring
        );

        let err = result.unwrap_err();
        let err_msg = err.to_string().to_lowercase();
        assert!(
            err_msg.contains(expected_error_substring),
            "Expected error containing '{}', got: {}",
            expected_error_substring,
            err
        );
    }
}
```

### Limit Validation

```rust
use hedl::{parse_with_limits, ParseOptions, Limits};

#[test]
fn test_max_depth_limit() {
    let mut input = String::from("%VERSION: 1.0\n---\n");
    for i in 0..200 {
        input.push_str(&format!("{}level{}: value\n", "  ".repeat(i), i));
    }

    let limits = Limits {
        max_indent_depth: 100,
        ..Default::default()
    };
    let options = ParseOptions {
        limits,
        ..Default::default()
    };

    let result = parse_with_limits(input.as_bytes(), options);
    assert!(result.is_err());
}

#[test]
fn test_max_total_keys_limit() {
    let mut input = String::from("%VERSION: 1.0\n---\n");
    for i in 0..10_001 {
        input.push_str(&format!("key{}: value\n", i));
    }

    let limits = Limits {
        max_total_keys: 10_000,
        ..Default::default()
    };
    let options = ParseOptions {
        limits,
        ..Default::default()
    };

    let result = parse_with_limits(input.as_bytes(), options);
    assert!(result.is_err());
}
```

---

## Test Utilities

Utilities for writing better tests.

### Test Utilities

```rust
// hedl-test provides pre-built fixtures and counting utilities
use hedl_test::{fixtures, count_nodes, count_references};

// Use pre-built fixtures for testing
let doc = fixtures::scalars();           // All scalar types
let doc = fixtures::user_list();         // MatrixList with users
let doc = fixtures::with_references();   // Cross-references
let doc = fixtures::comprehensive();     // Everything together

// Count utilities for assertions
let node_count = count_nodes(&doc);
let ref_count = count_references(&doc);
```

### Expression Utilities

```rust
// hedl-test provides expression helper functions
use hedl_test::{expr, expr_value};

// Create expression values for testing
let e = expr("now()");               // Create Expression
let v = expr_value("count + 1");     // Create Value::Expression
```

### Available Fixtures

```rust
// hedl-test provides pre-built fixtures
use hedl_test::{fixtures, fixtures_as_hedl};

// Access all fixtures
for (name, fixture_fn) in fixtures::all() {
    let doc = fixture_fn();
    // Test with the fixture document
}

// Specific fixtures
let doc = fixtures::scalars();           // All scalar value types
let doc = fixtures::user_list();         // MatrixList with 3 users
let doc = fixtures::with_nest();         // Nested relationships
let doc = fixtures::with_references();   // Cross-entity references
let doc = fixtures::comprehensive();     // Full feature coverage

// Get fixtures as HEDL text
for (name, hedl_text) in fixtures_as_hedl() {
    // Test with HEDL text representation
}
```

---

## Best Practices

### Test Naming

```rust
// Good: Descriptive, specific
#[test]
fn test_parse_simple_object_with_string_values() { }

// Bad: Vague, generic
#[test]
fn test1() { }
```

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod parsing {
        use super::*;

        #[test]
        fn test_simple_object() { }

        #[test]
        fn test_nested_object() { }
    }

    mod validation {
        use super::*;

        #[test]
        fn test_schema_validation() { }
    }
}
```

### Test Documentation

```rust
use hedl_core::parse;

/// Tests that parsing a simple object with string values
/// produces the expected AST structure.
///
/// This verifies:
/// - Key-value pairs are parsed correctly
/// - String inference works
/// - Attributes are stored in the correct order
#[test]
fn test_parse_simple_object() {
    // Arrange
    let input = b"%VERSION: 1.0
---
name: Alice
age: 30
";

    // Act
    let result = parse(input);

    // Assert
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.root.len(), 2);
}
```

---

**Next**: Learn about [Benchmarking](benchmarking.md) for performance testing
