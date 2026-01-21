# API Design Guidelines

Principles for designing stable, user-friendly APIs in HEDL.

## Core Principles

1. **Minimize Surprises**: APIs should behave as users expect
2. **Consistent Naming**: Similar operations use similar names
3. **Type Safety**: Leverage Rust's type system
4. **Error Clarity**: Errors should be actionable
5. **Future-Proof**: Design for evolution

## Builder Pattern

Use for complex configuration:

```rust
// ParseOptions and ParseOptionsBuilder are in crates/hedl-core/src/parser.rs

pub struct ParseOptions {
    pub limits: Limits,
    pub reference_mode: ReferenceMode,
}

/// Reference resolution mode for controlling validation behavior.
pub enum ReferenceMode {
    /// Strict mode: Unresolved references cause errors (default).
    Strict,
    /// Lenient mode: Unresolved references are silently ignored.
    Lenient,
}

impl ParseOptions {
    pub fn builder() -> ParseOptionsBuilder {
        ParseOptionsBuilder::default()
    }
}

pub struct ParseOptionsBuilder {
    limits: Limits,
    reference_mode: ReferenceMode,
}

impl ParseOptionsBuilder {
    pub fn new() -> Self {
        Self {
            limits: Limits::default(),
            reference_mode: ReferenceMode::Strict,
        }
    }

    pub fn reference_mode(mut self, mode: ReferenceMode) -> Self {
        self.reference_mode = mode;
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.limits.max_indent_depth = depth;
        self
    }

    pub fn max_file_size(mut self, size: usize) -> Self {
        self.limits.max_file_size = size;
        self
    }

    pub fn max_nodes(mut self, length: usize) -> Self {
        self.limits.max_nodes = length;
        self
    }

    pub fn build(self) -> ParseOptions {
        ParseOptions {
            limits: self.limits,
            reference_mode: self.reference_mode,
        }
    }
}

// Usage
let options = ParseOptions::builder()
    .max_depth(100)
    .reference_mode(ReferenceMode::Strict)
    .build();

// Or with multiple settings:
let options = ParseOptions::builder()
    .max_depth(100)
    .max_nodes(50_000)
    .reference_mode(ReferenceMode::Lenient)
    .build();
```

## Error Handling

```rust
use hedl_core::{HedlError, HedlErrorKind, HedlResult};

// HedlError structure (from hedl-core src/error.rs)
// pub struct HedlError {
//     pub kind: HedlErrorKind,
//     pub message: String,
//     pub line: usize,
//     pub column: Option<usize>,
//     pub context: Option<String>,
// }

// Result type alias
pub type Result<T> = HedlResult<T>;

// Example usage
use hedl_core::Document;

pub fn parse_custom(input: &[u8]) -> Result<Document> {
    // Custom parsing logic
    if input.is_empty() {
        return Err(HedlError::syntax("Empty input", 0));
    }
    // ... actual parsing
    hedl_core::parse(input)
}
```

## Version Stability

- Major version: Breaking changes
- Minor version: New features (backward compatible)
- Patch version: Bug fixes

## Related

- [Code Style Guide](code-style.md)
- [Release Process](release-process.md)
