# Error Handling

HEDL's approach to errors, validation, and user-friendly error messages.

## Error Philosophy

1. **Fail Fast**: Detect errors early in the pipeline
2. **Actionable Messages**: Tell users how to fix the problem
3. **Source Context**: Show where the error occurred
4. **Type Safety**: Use Rust's type system to prevent errors

## Error Types

```rust
// File: crates/hedl-core/src/error.rs

/// Main error type for all HEDL operations.
pub struct HedlError {
    pub kind: HedlErrorKind,
    pub message: String,
    pub line: usize,
    pub column: Option<usize>,
    pub context: Option<String>,
}

/// Error category enumeration.
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
```

## Error Location Tracking

Every error includes line information:

```rust
// Note: The HedlError type doesn't have a with_snippet() method in the current implementation.
// Error formatting with snippets would be done manually or in client code.
// The HedlError type provides:
//
// impl HedlError {
//     pub fn new(kind: HedlErrorKind, message: impl Into<String>, line: usize) -> Self
//     pub fn with_column(mut self, column: usize) -> Self
//     pub fn with_context(mut self, context: impl Into<String>) -> Self
//
//     // Convenience constructors for each error kind:
//     pub fn syntax(message: impl Into<String>, line: usize) -> Self
//     pub fn schema(message: impl Into<String>, line: usize) -> Self
//     pub fn reference(message: impl Into<String>, line: usize) -> Self
//     // ... and others
// }
//
// Example of formatting an error with snippet (in client code):
pub fn format_error_with_snippet(err: &HedlError, input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();

    if err.line > 0 && err.line <= lines.len() {
        let line_content = lines[err.line - 1];

        let mut output = format!(
            "{} at line {}: {}\n  {}\n",
            err.kind,
            err.line,
            err.message,
            line_content,
        );

        // Add column indicator if available
        if let Some(col) = err.column {
            let spaces = " ".repeat(col + 2);  // +2 for "  " prefix
            output.push_str(&format!("{}^\n", spaces));
        }

        output
    } else {
        format!("{}: {}", err.kind, err.message)
    }
}
```

Example output:
```
Parse error at line 5: unexpected token
  profile:
    bio: "Developer
         ^
Error: unclosed string quote
```

## Error Recovery

### Lenient Parsing

For documents with potential reference issues:

```rust
use hedl_core::{parse, parse_with_limits, ParseOptions, HedlErrorKind};

// Try strict parsing first, fall back to lenient
let doc = match parse(input) {
    Ok(doc) => doc,
    Err(e) if matches!(e.kind, HedlErrorKind::Reference) => {
        // Lenient mode: use non-strict reference validation
        let options = ParseOptions::builder()
            .strict_refs(false)
            .build();
        parse_with_limits(input, options)?
    }
    Err(e) => return Err(e),
};
```

### Error Pattern Matching

```rust
use hedl_core::{parse, HedlError, HedlErrorKind};

match parse(input) {
    Ok(doc) => {
        println!("Parsed {} root items", doc.root.len());
    }
    Err(e) => {
        match e.kind {
            HedlErrorKind::Syntax => {
                eprintln!("Syntax error at line {}: {}", e.line, e.message);
            }
            HedlErrorKind::Schema => {
                eprintln!("Schema error at line {}: {}", e.line, e.message);
            }
            HedlErrorKind::Reference => {
                eprintln!("Reference error: {}", e.message);
            }
            HedlErrorKind::Security => {
                eprintln!("Security limit exceeded: {}", e.message);
            }
            _ => {
                eprintln!("Error at line {}: {}", e.line, e.message);
            }
        }
    }
}
```

## User-Friendly Messages

### Bad Error Message

```
Error: parse failed
```

### Good Error Message

```
Parse error at line 12, column 8: unexpected token ']'

Expected one of:
  - value (string, number, boolean, null)
  - reference (@Type:id)
  - tensor ([...])

Help: Arrays in HEDL use square brackets. Did you mean [value1, value2]?

   users: @User
     | alice, Alice
     | bob, Bob
   ]
   ^
```

### Implementation

```rust
// Note: HedlError doesn't have an enhanced_message() method built-in.
// Here's an example of how to create enhanced error messages in client code:

pub fn enhanced_error_message(err: &HedlError, input: &str) -> String {
    let mut msg = format!("{:?} error at line {}: {}", err.kind, err.line, err.message);

    // Add context
    let lines: Vec<&str> = input.lines().collect();
    if err.line > 0 && err.line <= lines.len() {
        msg.push_str(&format!("\n\n  {}\n", lines[err.line - 1]));
    }

    // Add suggestions based on error kind
    let suggestion = match err.kind {
        HedlErrorKind::Syntax if err.message.contains("unclosed") => {
            Some("Add a closing quote (\") to the string")
        }
        HedlErrorKind::Reference => {
            Some("Ensure the referenced ID exists or define the entity")
        }
        HedlErrorKind::Schema => {
            Some("Check that the struct is defined with %STRUCT directive")
        }
        HedlErrorKind::Shape => {
            Some("Ensure the row has the correct number of fields for the struct")
        }
        _ => None,
    };

    if let Some(s) = suggestion {
        msg.push_str(&format!("\nHelp: {}", s));
    }

    msg
}
```

## Validation vs Parsing Errors

### Parse-Time Errors

Detected during parsing:
- Syntax errors (malformed HEDL)
- UTF-8 encoding issues
- Indentation errors

### Validation Errors

Detected after parsing:
- Dangling references
- Schema mismatches
- Duplicate IDs

```rust
use hedl_core::{Document, HedlResult};

pub fn parse_and_validate(input: &[u8]) -> HedlResult<Document> {
    // Phase 1: Parse
    let doc = hedl_core::parse(input)?;

    // Phase 2: Validate
    validate_document(&doc)?;

    Ok(doc)
}

fn validate_document(doc: &Document) -> HedlResult<()> {
    validate_references(doc)?;
    validate_schemas(doc)?;
    validate_ids(doc)?;
    Ok(())
}
```

## Related

- [Parser Architecture](parser-architecture.md)
- [Debug Parser](../how-to/debug-parser.md)
