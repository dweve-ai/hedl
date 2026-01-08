# hedl-test

Shared test fixtures and utilities for HEDL format converters.

## Installation

```toml
[dev-dependencies]
hedl-test = "1.0"
```

## Usage

```rust
use hedl_test::fixtures;

// Load test fixtures
let doc = fixtures::simple_document();
let nested = fixtures::nested_document();
let matrix = fixtures::matrix_document();

// Use in tests
#[test]
fn test_roundtrip() {
    let doc = fixtures::comprehensive_document();
    // ... test conversion roundtrip
}
```

## Available Fixtures

- `simple_document()` - Basic key-value pairs
- `nested_document()` - Nested object structures
- `matrix_document()` - Matrix lists with schemas
- `reference_document()` - Documents with references
- `comprehensive_document()` - All HEDL features combined

## File Fixtures

Located in `fixtures/` directory:
- `blog.hedl` - Blog post example
- `scalars.hedl` - All scalar types
- `tensors.hedl` - Tensor literals
- `references.hedl` - Reference examples

## License

Apache-2.0
