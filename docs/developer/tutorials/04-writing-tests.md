# Tutorial 4: Writing Effective Tests

Master the art of writing comprehensive tests for HEDL.

## Overview

This tutorial covers HEDL's testing practices:
- Unit testing patterns
- Integration testing strategies
- Property-based testing with proptest
- Fuzz testing setup
- Error path testing
- Test organization

**Time**: ~45 minutes

## Prerequisites

- Completed previous tutorials
- Understanding of Rust testing basics
- Familiarity with the HEDL codebase

## HEDL Testing Philosophy

### Test Pyramid

```
        E2E (Few, Slow)
       /                \
      /   Integration    \
     /   (Some, Medium)   \
    /__Unit (Many, Fast)__\
```

### Coverage Goals

- **Unit Tests**: >90% line coverage
- **Branch Coverage**: All conditionals tested
- **Error Paths**: Every error condition
- **Edge Cases**: Boundary values, empty inputs, max limits

## Part 1: Unit Testing Patterns

### Example: Testing a Parser Function

File: `crates/hedl-core/src/lex.rs`

```rust
use hedl_core::lex::{LexError, Reference, is_valid_type_name, is_valid_id_token};

/// Parse a reference token (@Type:id or @id)
pub fn parse_reference(s: &str) -> Result<Reference, LexError> {
    let s = s.strip_prefix('@').unwrap_or(s);

    if let Some((type_part, id_part)) = s.split_once(':') {
        // Qualified reference
        if !is_valid_type_name(type_part) {
            return Err(LexError::InvalidReference {
                message: format!("invalid type name: {}", type_part),
                pos: SourcePos::default(),
            });
        }
        if !is_valid_id_token(id_part) {
            return Err(LexError::InvalidReference {
                message: format!("invalid ID: {}", id_part),
                pos: SourcePos::default(),
            });
        }
        Ok(Reference {
            type_name: Some(type_part.to_string()),
            id: id_part.to_string(),
        })
    } else {
        // Local reference
        if !is_valid_id_token(s) {
            return Err(LexError::InvalidReference {
                message: format!("invalid ID: {}", s),
                pos: SourcePos::default(),
            });
        }
        Ok(Reference {
            type_name: None,
            id: s.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Happy path: Simple reference
    #[test]
    fn test_parse_reference_simple() {
        let ref_ = parse_reference("@user123").unwrap();
        assert_eq!(ref_.id, "user123");
        assert_eq!(ref_.type_name, None);
    }

    // Happy path: Qualified reference
    #[test]
    fn test_parse_reference_qualified() {
        let ref_ = parse_reference("@User:alice").unwrap();
        assert_eq!(ref_.id, "alice");
        assert_eq!(ref_.type_name, Some("User".to_string()));
    }

    // Edge case: Minimum valid
    #[test]
    fn test_parse_reference_min() {
        let ref_ = parse_reference("@a").unwrap();
        assert_eq!(ref_.id, "a");
    }

    // Edge case: Maximum length
    #[test]
    fn test_parse_reference_long() {
        let long_id = "a".repeat(1000);
        let input = format!("@{}", long_id);
        let ref_ = parse_reference(&input).unwrap();
        assert_eq!(ref_.id, long_id);
    }

    // Error path: Invalid type name
    #[test]
    fn test_parse_reference_invalid_type() {
        let result = parse_reference("@123Invalid:alice");
        assert!(matches!(result, Err(LexError::InvalidReference { .. })));
    }

    // Error path: Invalid ID
    #[test]
    fn test_parse_reference_invalid_id() {
        let result = parse_reference("@invalid id with spaces");
        assert!(matches!(result, Err(LexError::InvalidReference { .. })));
    }

    // Edge case: Empty after @
    #[test]
    fn test_parse_reference_empty() {
        let result = parse_reference("@");
        assert!(result.is_err());
    }

    // Edge case: Only colon
    #[test]
    fn test_parse_reference_only_colon() {
        let result = parse_reference("@:");
        assert!(result.is_err());
    }

    // Special characters
    #[test]
    fn test_parse_reference_with_underscore() {
        let ref_ = parse_reference("@user_123").unwrap();
        assert_eq!(ref_.id, "user_123");
    }

    // Multiple colons (only first is separator)
    #[test]
    fn test_parse_reference_multiple_colons() {
        // Invalid: IDs can't contain colons
        let result = parse_reference("@User:id:extra");
        assert!(result.is_err());
    }
}
```

### Testing Checklist

For each function, test:

- [ ] **Happy path**: Typical valid inputs
- [ ] **Edge cases**: Empty, minimum, maximum, boundary values
- [ ] **Error paths**: All error conditions
- [ ] **Special values**: Null, zero, negative, Unicode
- [ ] **State transitions**: Before/after state changes
- [ ] **Invariants**: Properties that must always hold

## Part 2: Integration Testing

File: `crates/hedl-json/tests/round_trip_tests.rs`

```rust
use hedl_core::parse;
use hedl_json::{to_json, from_json};
use hedl_c14n::canonicalize;

#[test]
fn test_round_trip_simple_document() {
    let hedl = r#"%VERSION: 1.0
---
name: Alice
age: 30
active: true
"#;

    // HEDL → JSON → HEDL → JSON
    let doc1 = parse(hedl.as_bytes()).unwrap();
    let json1 = to_json(&doc1).unwrap();
    let doc2 = from_json(&json1).unwrap();
    let json2 = to_json(&doc2).unwrap();

    // Should be identical
    assert_eq!(json1, json2);
}

#[test]
fn test_round_trip_preserves_types() {
    let hedl = b"%VERSION: 1.0\n---\nint: 42\nfloat: 3.14\nbool: true\nstring: hello\nnull: ~\n";

    let doc1 = parse(hedl).unwrap();
    let json = to_json(&doc1).unwrap();
    let doc2 = from_json(&json).unwrap();

    // Verify types preserved
    use hedl_core::{Item, Value};
    let root = &doc2.root;
    assert!(matches!(root.get("int"), Some(Item::Scalar(Value::Int(42)))));
    assert!(matches!(root.get("bool"), Some(Item::Scalar(Value::Bool(true)))));
    assert!(matches!(root.get("null"), Some(Item::Scalar(Value::Null))));
}

#[test]
fn test_cross_format_conversion() {
    let hedl = b"%VERSION: 1.0\n---\nuser:\n  name: Alice\n  email: alice@example.com\n";

    let doc = parse(hedl).unwrap();

    // Convert to all formats
    let json = hedl_json::to_json(&doc).unwrap();
    let yaml = hedl_yaml::to_yaml(&doc).unwrap();
    let xml = hedl_xml::to_xml(&doc).unwrap();

    // Convert back and verify structure preserved
    let from_json = hedl_json::from_json(&json).unwrap();
    let from_yaml = hedl_yaml::from_yaml(&yaml).unwrap();
    let from_xml = hedl_xml::from_xml(&xml).unwrap();

    // Canonicalize for comparison
    let c1 = canonicalize(&from_json).unwrap();
    let c2 = canonicalize(&from_yaml).unwrap();
    let c3 = canonicalize(&from_xml).unwrap();

    assert_eq!(c1, c2);
    assert_eq!(c2, c3);
}
```

## Part 3: Property-Based Testing

File: `crates/hedl-core/tests/property_tests.rs`

```rust
use proptest::prelude::*;
use hedl_core::parse;
use hedl_c14n::canonicalize;

// Strategy: Generate valid HEDL identifiers
fn arb_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}"
}

// Strategy: Generate valid HEDL values
fn arb_value() -> impl Strategy<Value = String> {
    prop_oneof![
        // Strings
        "\"[a-zA-Z0-9 ]{0,50}\"",
        // Integers
        any::<i64>().prop_map(|i| i.to_string()),
        // Floats
        any::<f64>().prop_map(|f| f.to_string()).prop_filter("finite", |s| {
            !s.contains("inf") && !s.contains("nan")
        }),
        // Booleans
        prop_oneof!["true", "false"],
        // Null
        Just("~".to_string()),
    ]
}

proptest! {
    // Property: Parsing never panics
    #[test]
    fn test_parse_never_panics(s in ".*") {
        let _ = parse(s.as_bytes());
    }

    // Property: Valid identifiers always parse
    #[test]
    fn test_valid_identifier_parses(id in arb_identifier()) {
        let hedl = format!("%VERSION: 1.0\n---\n{}: value", id);
        let result = parse(hedl.as_bytes());
        assert!(result.is_ok(), "Failed to parse valid identifier: {}", id);
    }

    // Property: Round-trip is idempotent
    #[test]
    fn test_round_trip_idempotent(
        key in arb_identifier(),
        value in arb_value()
    ) {
        let hedl1 = format!("%VERSION: 1.0\n---\n{}: {}", key, value);

        if let Ok(doc1) = parse(hedl1.as_bytes()) {
            if let Ok(canon1) = hedl_c14n::canonicalize(&doc1) {
                if let Ok(doc2) = parse(canon1.as_bytes()) {
                    if let Ok(canon2) = hedl_c14n::canonicalize(&doc2) {
                        // Second canonicalization should be identical
                        assert_eq!(canon1, canon2, "Round-trip not idempotent");
                    }
                }
            }
        }
    }

    // Property: Attribute count invariant
    #[test]
    fn test_attribute_count_invariant(
        count in 0usize..10,
        keys in prop::collection::vec(arb_identifier(), 0..10)
    ) {
        let mut hedl = String::from("%VERSION: 1.0\n---\n");
        for (i, key) in keys.iter().enumerate().take(count) {
            hedl.push_str(&format!("{}: value{}\n", key, i));
        }

        if let Ok(doc) = parse(hedl.as_bytes()) {
            // Number of items should match (accounting for duplicates)
            assert!(doc.root.len() <= count);
        }
    }
}

// Custom arbitrary document generator
fn arb_document() -> impl Strategy<Value = String> {
    (0..5usize).prop_flat_map(|depth| {
        arb_document_with_depth(depth)
    })
}

fn arb_document_with_depth(max_depth: usize) -> BoxedStrategy<String> {
    if max_depth == 0 {
        // Leaf: just attributes
        prop::collection::vec(
            (arb_identifier(), arb_value()),
            0..5
        )
        .prop_map(|attrs| {
            let mut hedl = String::from("%VERSION: 1.0\n---\n");
            hedl.push_str(&attrs.into_iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n"));
            hedl
        })
        .boxed()
    } else {
        // Recursively nest
        (
            prop::collection::vec((arb_identifier(), arb_value()), 0..3),
            prop::collection::vec(
                (arb_identifier(), arb_document_with_depth(max_depth - 1)),
                0..3
            )
        )
        .prop_map(|(attrs, children)| {
            let mut hedl = String::from("%VERSION: 1.0\n---\n");
            for (k, v) in attrs {
                hedl.push_str(&format!("{}: {}\n", k, v));
            }
            for (k, child) in children {
                hedl.push_str(&format!("{}:\n", k));
                // Skip the %VERSION and --- from nested child
                let child_lines: Vec<&str> = child.lines().skip(2).collect();
                for line in child_lines {
                    hedl.push_str(&format!(" {}\n", line));
                }
            }
            hedl
        })
        .boxed()
    }
}
```

## Part 4: Fuzz Testing

Setup fuzzing for `hedl-core`:

```bash
cd crates/hedl-core
cargo fuzz init
cargo fuzz add parse_fuzz
```

File: `crates/hedl-core/fuzz/fuzz_targets/parse_fuzz.rs`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use hedl_core::parse;

fuzz_target!(|data: &[u8]| {
    // Should never panic, even with arbitrary input
    let _ = parse(data);
});
```

Run fuzzing:
```bash
cargo fuzz run parse_fuzz -- -max_len=10000 -runs=100000
```

## Part 5: Error Path Testing

File: `crates/hedl-core/tests/error_paths.rs`

```rust
use hedl_core::{parse, HedlError, HedlErrorKind};

#[test]
fn test_error_max_depth_exceeded() {
    // Create deeply nested structure beyond limit
    let mut hedl = String::from("%VERSION: 1.0\n---\nroot:\n");
    for i in 0..150 {
        hedl.push_str(&"  ".repeat(i + 1));
        hedl.push_str(&format!("level{}:\n", i));
    }

    let result = parse(hedl.as_bytes());
    assert!(result.is_err());
    // Expect depth limit error from parser
}

#[test]
fn test_error_invalid_utf8() {
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    let result = parse(&invalid_utf8);
    assert!(result.is_err());
}

#[test]
fn test_error_unclosed_quote() {
    let hedl = b"%VERSION: 1.0\n---\nname: \"Alice";
    let result = parse(hedl);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("quote"));
}

#[test]
fn test_error_schema_mismatch() {
    let hedl = b"%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice Smith, extra_field\n";
    let result = parse(hedl);
    assert!(result.is_err());
}
```

## Part 6: Test Organization

### Directory Structure

```
crates/hedl-core/
├── src/
│   └── lib.rs          # Unit tests via #[cfg(test)]
├── tests/
│   ├── unit/           # Focused unit tests
│   │   ├── lexer.rs
│   │   └── parser.rs
│   ├── integration/    # Cross-module tests
│   │   ├── round_trip.rs
│   │   └── conversion.rs
│   ├── property/       # Property-based tests
│   │   └── invariants.rs
│   └── error_paths/    # Error condition tests
│       └── limits.rs
└── fuzz/
    └── fuzz_targets/
        ├── parse.rs
        └── lex.rs
```

### Test Utilities

File: `crates/hedl-test/src/lib.rs`

```rust
//! Test utilities for HEDL

use hedl_core::Document;
use hedl_test::{count_nodes, count_references, fixtures, expr, expr_value};

// Pre-built fixtures for common test cases
let doc = fixtures::scalars();           // All scalar types
let doc = fixtures::user_list();         // MatrixList with users
let doc = fixtures::with_references();   // Cross-references
let doc = fixtures::comprehensive();     // Everything together

// Counting utilities
let node_count = count_nodes(&doc);
let ref_count = count_references(&doc);

// Expression helpers
let e = expr("now()");                   // Create expression
let v = expr_value("count + 1");         // Create Value::Expression

// Access all fixtures
for (name, fixture_fn) in fixtures::all() {
    let doc = fixture_fn();
    // Test with fixture
}
```

## Part 7: Test Data Management

### Fixtures

```
fixtures/
├── examples/
│   ├── simple.hedl
│   ├── nested.hedl
│   └── complex.hedl
├── conformance/
│   ├── spec_001.hedl
│   └── spec_002.hedl
└── fuzz/
    └── corpus/
        ├── seed1.hedl
        └── seed2.hedl
```

### Golden Files

For snapshot testing:

```rust
#[test]
fn test_json_output_format() {
    let doc = parse(b"%VERSION: 1.0\n---\nname: Alice\nage: 30\n").unwrap();
    let json = to_json(&doc).unwrap();

    // Compare against golden file
    let expected = std::fs::read_to_string("tests/golden/simple.json")
        .expect("Missing golden file");

    assert_eq!(json.trim(), expected.trim());
}
```

## Part 8: Coverage Analysis

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --all --out Html --output-dir coverage/

# Open report
firefox coverage/index.html

# Coverage for specific crate
cargo tarpaulin -p hedl-core --out Lcov

# Fail build if coverage drops below threshold
cargo tarpaulin --all --fail-under 90
```

## Best Practices

### 1. Test Naming

```rust
// Pattern: test_<function>_<scenario>_<expected>
#[test]
fn test_parse_reference_simple_succeeds() { }

#[test]
fn test_parse_reference_missing_at_fails() { }

#[test]
fn test_parse_reference_empty_returns_error() { }
```

### 2. Arrange-Act-Assert

```rust
#[test]
fn test_example() {
    // Arrange: Set up test data
    let input = "test data";
    let expected = "expected result";

    // Act: Execute the code under test
    let actual = function_under_test(input);

    // Assert: Verify results
    assert_eq!(actual, expected);
}
```

### 3. One Assertion Per Test

```rust
// ❌ Bad: Multiple unrelated assertions
#[test]
fn test_everything() {
    assert_eq!(parse("a"), Ok(...));
    assert_eq!(parse("b"), Ok(...));
    assert_eq!(parse("c"), Ok(...));
}

// ✅ Good: Focused tests
#[test]
fn test_parse_a() {
    assert_eq!(parse("a"), Ok(...));
}

#[test]
fn test_parse_b() {
    assert_eq!(parse("b"), Ok(...));
}
```

### 4. Descriptive Failure Messages

```rust
assert_eq!(
    actual,
    expected,
    "Parsing '{}' produced unexpected result. Expected {:?}, got {:?}",
    input, expected, actual
);
```

## Next Steps

You now know how to write comprehensive tests for HEDL. Continue to:

- [How-To Guides](../how-to/README.md) for specific testing scenarios
- [Testing Guide](../testing.md) for advanced topics
- [Benchmarking Guide](../benchmarking.md) for performance testing

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [proptest Documentation](https://docs.rs/proptest/)
- [cargo-fuzz Guide](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [HEDL Test Examples](/crates/hedl-core/tests/)

---

**Excellent!** You've mastered HEDL testing practices.
