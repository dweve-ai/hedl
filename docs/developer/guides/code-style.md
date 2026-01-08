# Code Style Guide

Coding standards and conventions for HEDL development.

## Rust Naming Conventions

### Functions and Variables

```rust
// ✅ Good: snake_case
fn parse_document(input: &str) -> Result<Document> { }
let user_count = 42;

// ❌ Bad: camelCase or PascalCase
fn ParseDocument(input: &str) -> Result<Document> { }
let UserCount = 42;
```

### Types

```rust
// ✅ Good: PascalCase
struct DocumentParser { }
enum ParseError { }
trait DocumentVisitor { }

// ❌ Bad: snake_case
struct document_parser { }
```

### Constants

```rust
// ✅ Good: SCREAMING_SNAKE_CASE
const MAX_DEPTH: usize = 100;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// ❌ Bad: lowercase or PascalCase
const max_depth: usize = 100;
const MaxDepth: usize = 100;
```

### Modules

```rust
// ✅ Good: snake_case
mod parser;
mod error_handling;

// ❌ Bad: PascalCase or kebab-case
mod Parser;
mod error-handling;
```

## Documentation Standards

### Public Functions

```rust
/// Parses a HEDL document from UTF-8 bytes.
///
/// This function performs complete parsing including header directives,
/// body parsing, and reference resolution.
///
/// # Arguments
///
/// * `input` - UTF-8 encoded HEDL document
///
/// # Returns
///
/// Parsed `Document` on success.
///
/// # Errors
///
/// Returns `HedlError` if:
/// - Input is not valid UTF-8
/// - Syntax errors are found
/// - Resource limits are exceeded
///
/// # Examples
///
/// ```
/// use hedl_core::parse;
///
/// let input = b"%VERSION: 1.0\n---\nname: Alice";
/// let doc = parse(input)?;
/// assert_eq!(doc.root.len(), 1);
/// # Ok::<(), hedl_core::HedlError>(())
/// ```
pub fn parse(input: &[u8]) -> Result<Document, HedlError> {
    // Implementation
}
```

### Modules

```rust
//! Parser module for HEDL documents.
//!
//! This module provides the core parsing functionality, converting
//! HEDL text into an Abstract Syntax Tree (AST).
//!
//! # Architecture
//!
//! Parsing happens in multiple stages:
//! 1. Preprocessing: Line splitting and indentation analysis
//! 2. Header parsing: Directive processing
//! 3. Body parsing: Recursive descent through document structure
//! 4. Reference resolution: Two-pass ID collection and resolution
//!
//! # Example
//!
//! ```
//! use hedl_core::parser::parse_with_options;
//! # use hedl_core::ParseOptions;
//!
//! let options = ParseOptions::default();
//! let doc = parse_with_options(b"key: value", &options)?;
//! # Ok::<(), hedl_core::HedlError>(())
//! ```
```

## Code Organization

### File Structure

```rust
// 1. License header
// Dweve HEDL - Hierarchical Entity Data Language
// Copyright (c) 2025...

// 2. Module documentation
//! Module description

// 3. Imports (grouped and sorted)
use std::collections::HashMap;
use std::fmt;

use thiserror::Error;

use crate::error::HedlError;
use crate::value::Value;

// 4. Constants
const MAX_DEPTH: usize = 100;

// 5. Type definitions
pub struct Parser { }

// 6. Implementations
impl Parser { }

// 7. Tests
#[cfg(test)]
mod tests { }
```

### Import Grouping

```rust
// Standard library
use std::collections::HashMap;
use std::io::{self, Write};

// External crates
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Internal crates (workspace)
use hedl_core::{Document, parse};

// Current crate
use crate::error::JsonError;
use crate::config::ToJsonConfig;
```

## Error Handling

### Use Result for Fallible Operations

```rust
// ✅ Good: Return Result
pub fn parse(input: &[u8]) -> Result<Document, HedlError> {
    if input.is_empty() {
        return Err(HedlError::syntax("Input is empty", 0));
    }
    // ...
}

// ❌ Bad: Panic on error
pub fn parse(input: &[u8]) -> Document {
    assert!(!input.is_empty(), "Input is empty!");
    // ...
}
```

### Propagate Errors with ?

```rust
// ✅ Good: Use ? operator
pub fn process(input: &str) -> Result<Value> {
    let trimmed = validate_input(input)?;
    let parsed = parse_value(trimmed)?;
    Ok(parsed)
}

// ❌ Bad: Unwrap (panics)
pub fn process(input: &str) -> Value {
    let trimmed = validate_input(input).unwrap();
    let parsed = parse_value(trimmed).unwrap();
    parsed
}
```

## Testing Conventions

### Test Naming

```rust
#[test]
fn test_parse_simple_document_succeeds() {
    // Arrange
    let input = b"name: Alice";

    // Act
    let result = parse(input);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_parse_empty_input_returns_error() {
    let result = parse(b"");
    assert!(result.is_err());
}
```

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod parse_function {
        use super::*;

        #[test]
        fn succeeds_with_valid_input() { }

        #[test]
        fn fails_with_invalid_utf8() { }
    }

    mod validation {
        use super::*;

        #[test]
        fn rejects_duplicate_ids() { }
    }
}
```

## Performance Considerations

### Avoid Unnecessary Clones

```rust
// ✅ Good: Borrow when possible
pub fn get_attribute(&self, key: &str) -> Option<&Value> {
    self.attributes.get(key)
}

// ❌ Bad: Clone unnecessarily
pub fn get_attribute(&self, key: &str) -> Option<Value> {
    self.attributes.get(key).cloned()
}
```

### Pre-allocate Collections

```rust
// ✅ Good: Pre-allocate if size known
let mut items = Vec::with_capacity(expected_size);

// ❌ Bad: Let Vec grow dynamically
let mut items = Vec::new();
```

## Clippy Warnings

All code must pass clippy with no warnings:

```bash
cargo clippy --all -- -D warnings
```

Common clippy fixes:

```rust
// Before (clippy warning)
if let Some(x) = option {
    x
} else {
    default
}

// After (idiomatic)
option.unwrap_or(default)

// Before (clippy warning)
match result {
    Ok(v) => Some(v),
    Err(_) => None,
}

// After (idiomatic)
result.ok()
```

## Formatting

Use `rustfmt` for all code:

```bash
cargo fmt --all
```

Custom `rustfmt.toml`:

```toml
max_width = 100
tab_spaces = 4
edition = "2021"
```

## Related

- [API Design Guidelines](api-design.md)
- [Documentation Guide](documentation-guide.md)
- [Contributing Guide](../contributing.md)
