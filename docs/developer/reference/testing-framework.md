# Testing Framework Reference

HEDL's testing infrastructure and utilities.

## Test Organization

```
crates/hedl-core/
├── src/           # Unit tests via #[cfg(test)]
└── tests/
    ├── unit/
    ├── integration/
    ├── property/
    └── fuzz/
```

## Test Utilities (hedl-test)

```rust
use hedl_test::fixtures;

// Use pre-built fixtures
let doc = fixtures::scalars();
let doc = fixtures::user_list();

// Count nodes using utilities from hedl-test
use hedl_test::{count_nodes, count_references};
let node_count = count_nodes(&doc);
let ref_count = count_references(&doc);
```

## Property Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_never_panics(s in ".*") {
        let _ = parse(s.as_bytes());
    }
}
```

## Fuzz Testing

```bash
cd crates/hedl-core
cargo fuzz run parse
```

## Fixtures

Available via the `hedl_test::fixtures` module:

```rust
use hedl_test::fixtures;

// Pre-built fixtures - see fixtures::all() for complete list
let doc = fixtures::scalars();           // All scalar value types
let doc = fixtures::special_strings();   // Edge case strings
let doc = fixtures::references();        // Reference fixtures
let doc = fixtures::expressions();       // Expression fixtures
let doc = fixtures::tensors();           // Tensor fixtures
let doc = fixtures::named_values();      // Named value fixtures
let doc = fixtures::user_list();         // MatrixList with users
let doc = fixtures::mixed_type_list();   // Mixed types in list
let doc = fixtures::with_references();   // Cross-references between entities
let doc = fixtures::with_nest();         // Nested relationships (NEST)
let doc = fixtures::deep_nest();         // Deep nesting levels
let doc = fixtures::edge_cases();        // Edge case scenarios
let doc = fixtures::comprehensive();     // All features combined
let doc = fixtures::blog();              // Blog example with users/posts
let doc = fixtures::empty();             // Empty document
```

## Related

- [Testing Guide](../testing.md)
- [Tutorial: Writing Tests](../tutorials/04-writing-tests.md)
