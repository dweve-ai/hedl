# Documentation Guide

How to write clear, helpful documentation for HEDL.

## Documentation Types

### 1. API Documentation (rustdoc)

```rust
/// Parses HEDL document from UTF-8 bytes.
///
/// # Examples
///
/// ```
/// use hedl_core::parse;
///
/// let input = b"%VERSION: 1.0\n---\nkey: value";
/// let doc = parse(input)?;
/// assert_eq!(doc.version, (1, 0));
/// # Ok::<(), hedl_core::HedlError>(())
/// ```
pub fn parse(input: &[u8]) -> Result<Document, HedlError> {
    // ...
}
```

### 2. Module Documentation

```rust
//! JSON conversion module.
//!
//! Provides bidirectional conversion between HEDL and JSON.
```

### 3. User Guides

Written in markdown in `/docs/` directory.

### 4. Examples

In `/examples/` directory, runnable with `cargo run --example`.

## Writing Good Docs

### Be Concise

```rust
/// Parses HEDL from bytes.
```

Not:

```rust
/// This function takes a byte slice representing a HEDL document
/// and attempts to parse it into a Document structure...
```

### Include Examples

Every public function should have at least one example.

### Document Errors

```rust
/// # Errors
///
/// Returns `HedlError` if:
/// - Input is not valid UTF-8
/// - Syntax errors found
```

## Related

- [Code Style Guide](code-style.md)
- [Contributing Guide](../contributing.md)
