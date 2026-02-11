# Error Handling and Codes

## Overview

HEDL provides comprehensive error handling across all API surfaces with consistent error types, codes, and messages.

## Error Categories

### Parse Errors

Errors that occur during document parsing.

#### `Syntax` Error

**Description**: Invalid HEDL syntax

**Common Causes**:
- Missing version directive
- Invalid header syntax
- Malformed key-value pairs
- Unclosed strings or brackets

**Example**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
key value  # Error: Missing colon
```

**Rust**:
```rust
use hedl::{parse, HedlError, HedlErrorKind};

match parse(input) {
    Err(HedlError { kind: HedlErrorKind::Syntax, .. }) => {
        eprintln!("Syntax error in HEDL document");
    }
    _ => {}
}
```

#### `Reference` Error

**Description**: Unresolved or invalid reference (in strict mode)

**Common Causes**:
- Reference to non-existent entity
- Circular references (in some contexts)
- Invalid reference syntax

**Example**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
user:@User:alice  # Error: User:alice not found
```

**Solution**: Use lenient parsing or define the referenced entity:
```rust
use hedl::{parse_with_limits, ParseOptions};

// Use lenient_refs() to ignore unresolved references
let opts = ParseOptions::builder().lenient_refs().build();
let doc = parse_with_limits(input.as_bytes(), opts)?;
```

#### `Schema` Error

**Description**: Invalid struct definition or usage (field count mismatch)

**Common Causes**:
- Undefined struct type
- Field count mismatch
- Duplicate struct definition

**Example**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
 |alice,Alice Smith, extra_field  # Error: Expected 2 fields, got 3
```

### Validation and Security Errors

Errors from linting, security limits, and best practices.

#### `Security` Error

**Description**: Document exceeds configured resource limits.

**Common Causes**:
- Too many nesting levels
- Too many total keys (DoS protection)
- Large block strings

#### `Conversion` Error

**Description**: Failed to convert between HEDL and another format (JSON, YAML, etc.).

### Error Handling by API

### Rust API

#### Result Type
```rust
pub type HedlResult<T> = Result<T, HedlError>;
```

#### Error Structure
```rust
pub struct HedlError {
    pub kind: HedlErrorKind,
    pub message: String,
    pub line: usize,
    pub column: Option<usize>,
    pub context: Option<String>,
}

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

#### Pattern Matching
```rust
use hedl::{parse, HedlError, HedlErrorKind};

match parse(input) {
    Ok(doc) => {
        println!("Success: {} items", doc.root.len());
    }
    Err(HedlError { kind, message, line }) => {
        match kind {
            HedlErrorKind::Syntax => {
                eprintln!("Syntax error at line {}: {}", line, message);
            }
            HedlErrorKind::Reference => {
                eprintln!("Reference error: {}", message);
            }
            HedlErrorKind::Security => {
                eprintln!("Document too large: {}", message);
            }
            _ => {
                eprintln!("Error: {}", message);
            }
        }
    }
}
```

#### Error Chaining
```rust
use hedl::HedlResultExt;

let result = parse(input)
    .context("Failed to parse HEDL document")
    .context("Processing user input");
```

### FFI API (C/C++)

#### Error Codes
```c
#define HEDL_OK              0      // Success
#define HEDL_ERR_NULL_PTR   -1      // Null pointer argument
#define HEDL_ERR_INVALID_UTF8 -2    // Invalid UTF-8
#define HEDL_ERR_PARSE      -3      // Parse error
#define HEDL_ERR_CANONICALIZE -4    // Canonicalization error
#define HEDL_ERR_JSON       -5      // JSON conversion error
#define HEDL_ERR_ALLOC      -6      // Memory allocation failed
#define HEDL_ERR_YAML       -7      // YAML conversion error
#define HEDL_ERR_XML        -8      // XML conversion error
#define HEDL_ERR_CSV        -9      // CSV conversion error
#define HEDL_ERR_PARQUET    -10     // Parquet conversion error
#define HEDL_ERR_LINT       -11     // Linting error
```

> **Note**: Additional internal error codes (`HEDL_ERR_NEO4J` through `HEDL_ERR_INVALID_HANDLE`, codes -12 to -16) are defined in the Rust implementation but may not be exposed in the public C header. Check your header version for availability.

#### Error Handling
```c
HedlDocument* doc = NULL;
HedlErrorCode code = hedl_parse(input, -1, 0, &doc);

if (code != HEDL_OK) {
    const char* error_msg = hedl_get_last_error();
    fprintf(stderr, "Error %d: %s\n", code, error_msg);
    return 1;
}

// Use document
hedl_free_document(doc);
```

#### Thread-Safe Error Handling
```c
// Each thread has its own error state
void* worker(void* arg) {
    HedlDocument* doc = NULL;

    if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
        // Get error for THIS thread only
        const char* err = hedl_get_last_error_threadsafe();
        fprintf(stderr, "Parse error in thread: %s\n", err);
        return NULL;
    }

    hedl_free_document(doc);
    return (void*)1;
}
```

### WASM API (JavaScript/TypeScript)

All HEDL WASM functions throw standard JavaScript `Error` objects on failure. Parse errors include line information in the error message.

#### Try-Catch Handling
```typescript
import { parse } from 'hedl-wasm';

try {
    const doc = parse(hedlText);
    console.log(`Parsed ${doc.root.length} items`);
} catch (error) {
    if (error instanceof Error) {
        // Parse errors include line info in message:
        // "Parse error at line 5: unexpected token"
        const lineMatch = error.message.match(/line (\d+)/i);
        if (lineMatch) {
            console.error(`Error at line ${lineMatch[1]}: ${error.message}`);
        } else {
            console.error(`Error: ${error.message}`);
        }
    } else {
        console.error(`Unexpected error: ${error}`);
    }
}
```

#### Validation Results
```typescript
import { validate } from 'hedl-wasm';

const result = validate(hedlText);

if (!result.valid) {
    result.errors.forEach(error => {
        console.error(`Line ${error.line}: ${error.message}`);
    });
}
```

### MCP API

#### JSON-RPC Error Format
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": {
      "kind": "Syntax",
      "line": 5,
      "column": 12,
      "details": "Expected colon after key"
    }
  },
  "id": 1
}
```

#### Standard Error Codes
- `-32700`: Parse error (Invalid JSON-RPC)
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error
- `-32001`: Unauthorized (reserved, not used)
- `-32002`: File operation error
- `-32005`: Rate limit exceeded

## Best Practices

### Input Validation

Always validate input before processing:

```rust
use hedl::{parse, validate};

// Quick validation
if let Err(e) = validate(input) {
    return Err(format!("Invalid HEDL: {}", e));
}

// Full parsing with error handling
let doc = parse(input).map_err(|e| {
    format!("Parse failed at line {}: {}", e.line, e.message)
})?;
```

### Resource Limits

Set appropriate limits for your use case:

```rust
use hedl::{parse_with_limits, ParseOptions, Limits};

// For small documents (API responses)
let limits = Limits {
    max_indent_depth: 10,
    max_object_keys: 1000,
    max_total_keys: 10000,
    max_block_string_size: 100_000,
    ..Limits::default()
};

// For large documents (data files)
let limits = Limits {
    max_indent_depth: 64,
    max_object_keys: 100_000,
    max_total_keys: 1_000_000,
    max_block_string_size: 10_000_000,
    ..Limits::default()
};

let options = ParseOptions { reference_mode: ReferenceMode::Strict, limits };
let doc = parse_with_limits(input.as_bytes(), options)?;
```

### Error Recovery

Implement graceful error recovery:

```rust
use hedl::{parse, parse_with_limits, ParseOptions, HedlErrorKind};

// Try strict parsing first
match parse(input) {
    Ok(doc) => Ok(doc),
    Err(e) if matches!(e.kind, HedlErrorKind::Reference) => {
        // Fall back to lenient parsing for reference errors
        eprintln!("Warning: Using lenient mode due to: {}", e.message);
        let opts = ParseOptions::builder().lenient_refs().build();
        parse_with_limits(input.as_bytes(), opts)
    }
    Err(e) => Err(e),
}
```

### Logging

Log errors for debugging and monitoring:

```rust
use hedl::parse;
use tracing::{error, warn};

match parse(input) {
    Ok(doc) => {
        // Success
    }
    Err(e) => {
        error!(
            kind = ?e.kind,
            line = e.line,
            message = %e.message,
            "Failed to parse HEDL document"
        );
        return Err(e);
    }
}
```

## See Also

- [Getting Started](getting-started.md) - Basic API usage
- [Rust API Reference](rust-api.md) - Detailed Rust documentation
- [FFI/C API Reference](ffi-api.md) - C-compatible bindings
