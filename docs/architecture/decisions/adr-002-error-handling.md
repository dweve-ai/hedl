# ADR-002: Error Handling Strategy

**Status**: Accepted

**Date**: 2025-01-06

**Context**: Core architecture decision

---

## Context

HEDL parsing involves multiple stages (lexing, parsing, validation, conversion) that can fail in various ways. We need a consistent error handling strategy that provides:

1. **Type Safety**: Compile-time error handling verification
2. **Context Preservation**: Full error context for debugging
3. **User-Friendly Messages**: Actionable error messages
4. **Position Tracking**: Line/column information for diagnostics
5. **Error Propagation**: Easy composition of fallible operations

## Decision Drivers

- Rust idiomatic error handling
- Parser error reporting best practices
- LSP diagnostic requirements
- CLI error message clarity
- Performance (minimal allocation overhead)

## Considered Options

### Option 1: Panic on Errors

```rust
pub fn parse(input: &str) -> Document {
    // Panic if parsing fails
    let doc = parse_internal(input).unwrap();
    doc
}
```

**Pros**:
- Simple API
- No Result handling

**Cons**:
- Cannot recover from errors
- Terrible user experience
- Not Rust idiomatic
- Crashes LSP/CLI

**Verdict**: ❌ Rejected

### Option 2: Error Codes (C-style)

```rust
pub fn parse(input: &str, doc: &mut Document) -> i32 {
    // Return 0 on success, error code on failure
    if let Err(e) = parse_internal(input) {
        return e.code();
    }
    0
}
```

**Pros**:
- FFI-friendly
- No allocations

**Cons**:
- No context preservation
- Not Rust idiomatic
- Requires out parameters

**Verdict**: ❌ Rejected for Rust API (✓ Used in FFI layer only)

### Option 3: Result<T, String>

```rust
pub fn parse(input: &str) -> Result<Document, String> {
    // Return error as string
    parse_internal(input)
        .map_err(|e| format!("Parse error: {}", e))
}
```

**Pros**:
- Type-safe
- Easy to use

**Cons**:
- No structured error information
- Cannot programmatically handle specific errors
- No position tracking

**Verdict**: ❌ Rejected

### Option 4: Result<T, HedlError> with thiserror (CHOSEN)

```rust
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{kind} at line {line}: {message}")]
pub struct HedlError {
    pub kind: HedlErrorKind,
    pub message: String,
    pub line: usize,
    pub column: Option<usize>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HedlErrorKind {
    Syntax,
    Version,
    Schema,
    Alias,
    Shape,
    Semantic,
    OrphanRow,
    Collision,
    Reference,
    Security,
    Conversion,
    IO,
}

pub type HedlResult<T> = Result<T, HedlError>;

pub fn parse(input: &[u8]) -> HedlResult<Document> {
    parse_internal(input)
}
```

**Pros**:
- Type-safe error handling
- Structured error information
- Position tracking for diagnostics
- `thiserror` derive reduces boilerplate
- Easy propagation with `?` operator
- Programmatic error inspection

**Cons**:
- More complex than simple strings
- Requires error type definitions

**Verdict**: ✅ **CHOSEN**

## Decision

**Chosen**: Result<T, HedlError> with thiserror

## Rationale

### Primary Reasons

1. **Type Safety**: Compiler enforces error handling
   ```rust
   let doc = parse(input)?;  // Compile error if not handled
   ```

2. **Structured Errors**: Error kind + message + line/column
   ```rust
   match parse(input) {
       Ok(doc) => { /* ... */ }
       Err(e) => match e.kind {
           HedlErrorKind::Syntax => { /* specific handling */ }
           HedlErrorKind::Reference => { /* ... */ }
           _ => { /* ... */ }
       }
   }
   ```

3. **Position Tracking**: Line/column information
   ```rust
   impl HedlError {
       pub fn with_column(mut self, column: usize) -> Self {
           self.column = Some(column);
           self
       }

       pub fn with_context(mut self, context: impl Into<String>) -> Self {
           self.context = Some(context.into());
           self
       }
   }
   ```

4. **Context Preservation**: Error context
   ```rust
   fn parse_value(input: &str, line: usize) -> HedlResult<Value> {
       let token = lex_value(input)
           .map_err(|e| {
               HedlError::syntax("Failed to lex value", line)
                   .with_context(format!("Input: {}", input))
           })?;
       // ...
   }
   ```

5. **User-Friendly Messages**: Actionable suggestions
   ```rust
   HedlError::syntax(
       "Unmatched quote at line 5\nHelp: Add closing quote or use triple-quote for multi-line",
       pos
   )
   ```

### Error Type Design

```rust
/// Main error type for all HEDL operations.
#[derive(Debug, Clone, Error)]
#[error("{kind} at line {line}: {message}")]
pub struct HedlError {
    pub kind: HedlErrorKind,
    pub message: String,
    pub line: usize,
    pub column: Option<usize>,
    pub context: Option<String>,
}

impl HedlError {
    pub fn new(kind: HedlErrorKind, message: impl Into<String>, line: usize) -> Self {
        Self {
            kind,
            message: message.into(),
            line,
            column: None,
            context: None,
        }
    }

    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn syntax(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Syntax, message, line)
    }

    pub fn schema(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Schema, message, line)
    }

    pub fn reference(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Reference, message, line)
    }

    pub fn security(message: impl Into<String>, line: usize) -> Self {
        Self::new(HedlErrorKind::Security, message, line)
    }

    pub fn conversion(message: impl Into<String>) -> Self {
        Self::new(HedlErrorKind::Conversion, message, 0)
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(HedlErrorKind::IO, message, 0)
    }
}
```

### Error Kind Enumeration

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HedlErrorKind {
    Syntax,       // Lexical or structural violation
    Version,      // Unsupported version
    Schema,       // Schema violation or mismatch
    Alias,        // Duplicate or invalid alias
    Shape,        // Wrong number of cells in row
    Semantic,     // Logical error
    OrphanRow,    // Child row without NEST rule
    Collision,    // Duplicate ID within type
    Reference,    // Unresolved reference
    Security,     // Security limit exceeded
    Conversion,   // Format conversion error
    IO,           // I/O error
}

impl fmt::Display for HedlErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax => write!(f, "SyntaxError"),
            Self::Version => write!(f, "VersionError"),
            Self::Schema => write!(f, "SchemaError"),
            Self::Alias => write!(f, "AliasError"),
            Self::Shape => write!(f, "ShapeError"),
            Self::Semantic => write!(f, "SemanticError"),
            Self::OrphanRow => write!(f, "OrphanRowError"),
            Self::Collision => write!(f, "CollisionError"),
            Self::Reference => write!(f, "ReferenceError"),
            Self::Security => write!(f, "SecurityError"),
            Self::Conversion => write!(f, "ConversionError"),
            Self::IO => write!(f, "IOError"),
        }
    }
}
```

## Implementation

### Parser Error Handling

```rust
pub fn parse(input: &[u8]) -> HedlResult<Document> {
    // 1. Preprocessing (UTF-8, line endings, limits)
    let preprocessed = preprocess(input, &limits)?;

    // 2. Parse header
    let (header, body_start_idx) = parse_header(&lines, &limits)?;

    // 3. Parse body (includes lexical analysis)
    let root = parse_body(&lines[body_start_idx..], &header, &limits, &mut registries)?;

    // 4. Reference resolution
    resolve_references(&doc, strict)?;

    Ok(doc)
}
```

### Error Propagation

```rust
fn parse_matrix_list(lines: &[&str], schema: &[String]) -> HedlResult<MatrixList> {
    let mut rows = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        // Propagate error with `?`
        let fields = parse_csv_row(line)?;

        // Add context
        if fields.len() != schema.len() {
            return Err(HedlError::schema(
                format!(
                    "Row {}: expected {} fields, got {}",
                    i + 1,
                    schema.len(),
                    fields.len()
                ),
                calculate_position(lines, i)
            ));
        }

        rows.push(create_node(fields));
    }

    Ok(MatrixList { rows, schema: schema.to_vec() })
}
```

### LSP Integration

```rust
use lsp_types::Diagnostic;

impl From<HedlError> for Diagnostic {
    fn from(e: HedlError) -> Self {
        let line = e.line.saturating_sub(1); // Convert to 0-based
        let col = e.column.unwrap_or(0).saturating_sub(1); // Convert to 0-based

        Diagnostic {
            range: lsp_types::Range {
                start: lsp_types::Position { line: line as u32, character: col as u32 },
                end: lsp_types::Position { line: line as u32, character: (col + 1) as u32 },
            },
            severity: Some(match e.kind {
                HedlErrorKind::Syntax => DiagnosticSeverity::ERROR,
                HedlErrorKind::Schema => DiagnosticSeverity::ERROR,
                HedlErrorKind::Reference => DiagnosticSeverity::WARNING,
                _ => DiagnosticSeverity::INFORMATION,
            }),
            message: e.message,
            source: Some("hedl".to_string()),
            ..Default::default()
        }
    }
}
```

### CLI Integration

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let content = std::fs::read_to_string(&args.file)?;

    match hedl::parse(content.as_bytes()) {
        Ok(doc) => {
            println!("Parsed successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e.message);
            eprintln!("  at line {}", e.line);

            if let Some(col) = e.column {
                eprintln!("  at column {}", col);
            }

            if let Some(ctx) = e.context {
                eprintln!("  context: {}", ctx);
            }

            std::process::exit(1);
        }
    }
}
```

## Consequences

### Positive

1. **Type Safety**: Compile-time error handling enforcement
2. **Rich Context**: Error kind, message, position, source chain
3. **Composability**: Easy to propagate with `?` operator
4. **Debugging**: Full error context for diagnostics
5. **Idiomatic**: Standard Rust error handling pattern

### Negative

1. **Verbosity**: More code than simple strings
2. **Learning Curve**: New contributors need to understand error types
3. **Allocation**: Error creation allocates (mitigated by infrequency)

### Mitigations

1. **thiserror**: Derive macro reduces boilerplate
2. **Documentation**: Error handling guide for contributors
3. **Performance**: Errors are rare, allocation overhead acceptable

## Alternatives Considered

### Custom Display vs thiserror

**Decision**: Use `thiserror`

**Rationale**:
- Less boilerplate
- Standard pattern
- Better maintainability

### Error Recovery vs. Fail Fast

**Decision**: Fail fast for parsing, optional recovery for linting

**Rationale**:
- Parsing: Clear failure semantics
- Linting: Can collect multiple diagnostics

```rust
// Parsing: fail fast
pub fn parse(input: &str) -> HedlResult<Document> {
    // First error stops parsing
}

// Linting: collect all diagnostics
pub fn lint(doc: &Document) -> Vec<Diagnostic> {
    // Collect all issues, don't stop on first
}
```

### Position Tracking: Byte Offset vs. Line/Column

**Decision**: Store line number directly, column optional

**Rationale**:
- Line number is tracked during parsing (1-based)
- Column is optional and added when available
- Context can be added via `with_context()` method
- Avoids byte offset to line/column conversion overhead

## References

- Rust Error Handling: https://doc.rust-lang.org/book/ch09-00-error-handling.html
- thiserror: https://docs.rs/thiserror/
- Error Handling in Parsers: https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html

## Review

This ADR should be reviewed if:
- User feedback indicates error messages are unclear
- Performance profiling shows error allocation overhead
- New error categories emerge

---

*Decision made: 2025-01-06*
*Last reviewed: 2026-01-06*
