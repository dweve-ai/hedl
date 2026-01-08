# Getting Started with HEDL API

Welcome to the HEDL (Hierarchical Entity Data Language) API documentation. This guide will help you get started with using HEDL in your projects.

## Quick Start

### Installation

#### Rust
```toml
[dependencies]
hedl = "1.0"
```

#### C/C++ (via FFI)
Download the shared library from releases and include the header:
```c
#include "hedl.h"
```

#### JavaScript/TypeScript (via WASM)
```bash
npm install hedl-wasm
```

### First HEDL Document

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
```

## API Overview

HEDL provides multiple API layers:

1. **[Rust API](rust-api.md)** - Native Rust library with full type safety
2. **[FFI API](ffi-api.md)** - C-compatible interface for C/C++ integration
3. **[WASM API](wasm-api.md)** - Browser and Node.js integration
4. **[MCP API](mcp-api.md)** - Model Context Protocol server for AI/LLM systems
5. **[LSP API](lsp-api.md)** - Language Server Protocol for editor integration

## Basic Operations

### Parsing

#### Rust
```rust
use hedl::parse;

let doc = parse(hedl_text)?;
println!("Parsed {} items", doc.root.len());
```

#### C
```c
const char* input = "%VERSION: 1.0\n---\nkey: value";
HedlDocument* doc = NULL;
if (hedl_parse(input, -1, 0, &doc) == HEDL_OK) {
    // Use document
    hedl_free_document(doc);
}
```

#### JavaScript
```javascript
import { parse } from 'hedl-wasm';

const doc = parse(hedlText);
console.log(`Parsed ${doc.rootItemCount} items`);
```

### Serialization

#### Rust
```rust
use hedl::{to_json, canonicalize};

// Convert to JSON
let json = to_json(&doc)?;

// Canonical HEDL format
let canonical = canonicalize(&doc)?;
```

#### C
```c
char* json = NULL;
if (hedl_to_json(doc, 0, &json) == HEDL_OK) {  // 0 = no metadata
    printf("%s\n", json);
    hedl_free_string(json);
}
```

#### JavaScript
```javascript
import { parse, toJson, format } from 'hedl-wasm';

// Parse document
const doc = parse(hedlText);

// Convert HEDL string to JSON string
const json = toJson(hedlText, true);

// Or format HEDL string to canonical form
const canonical = format(hedlText, true);
```

### Validation

#### Rust
```rust
use hedl::{validate, lint};

// Basic validation
validate(hedl_text)?;

// Advanced linting
let doc = parse(hedl_text)?;
let diagnostics = lint(&doc);
for d in diagnostics {
    println!("{:?}: {}", d.severity(), d.message());
}
```

#### C
```c
HedlDiagnostics* diag = NULL;
if (hedl_lint(doc, &diag) == HEDL_OK) {
    int count = hedl_diagnostics_count(diag);
    for (int i = 0; i < count; i++) {
        char* msg = NULL;
        if (hedl_diagnostics_get(diag, i, &msg) == HEDL_OK) {
            printf("Diagnostic: %s\n", msg);
            hedl_free_string(msg);
        }
    }
    hedl_free_diagnostics(diag);
}
```

#### JavaScript
```javascript
import { validate, lint } from 'hedl-wasm';

const result = validate(hedlText);
if (!result.valid) {
    console.error(result.errors);
}

const diagnostics = lint(hedlText);
diagnostics.forEach(d => {
    console.log(`${d.severity}: ${d.message}`);
});
```

## Core Concepts

### Documents

A HEDL document consists of:
- **Header**: Version and structure definitions (`%VERSION`, `%STRUCT`)
- **Body**: Data organized as key-value pairs, lists, and nested objects

### Types

HEDL supports:
- **Primitives**: Strings, integers, floats, booleans, null
- **Collections**: Arrays (tensors) and objects
- **References**: `@Type:id` for graph relationships
- **Matrix Lists**: CSV-like tables with type annotations

### Error Handling

All APIs provide consistent error handling:

#### Rust
```rust
match parse(hedl_text) {
    Ok(doc) => { /* use doc */ },
    Err(e) => eprintln!("Parse error: {}", e),
}
```

#### C
```c
HedlDocument* doc = NULL;
if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
    const char* err = hedl_get_last_error();
    fprintf(stderr, "Error: %s\n", err);
    return 1;
}
```

#### JavaScript
```javascript
try {
    const doc = parse(hedlText);
} catch (error) {
    console.error(`Parse error: ${error.message}`);
}
```

## Performance Considerations

### Token Efficiency

HEDL is designed for AI/ML contexts with significant token savings:

```rust
use hedl_wasm::getStats;

let stats = getStats(hedlText);
println!("HEDL tokens: {}", stats.hedlTokens);
println!("JSON tokens: {}", stats.jsonTokens);
println!("Savings: {}%", stats.savingsPercent);
```

### Memory Management

#### Rust
Automatic memory management with RAII.

#### C/FFI
**Critical**: Always free allocated memory:
```c
// Strings
char* str = NULL;
hedl_to_json(doc, 0, &str);  // 0 = no metadata
hedl_free_string(str);  // REQUIRED

// Documents
HedlDocument* doc = NULL;
hedl_parse(input, -1, 0, &doc);
hedl_free_document(doc);  // REQUIRED

// Diagnostics
HedlDiagnostics* diag = NULL;
hedl_lint(doc, &diag);
hedl_free_diagnostics(diag);  // REQUIRED
```

#### JavaScript/WASM
Automatic garbage collection.

### Thread Safety

#### Rust
All types are `Send` and `Sync` where appropriate.

#### C/FFI
- Error messages: Thread-local storage (thread-safe)
- Documents: **NOT thread-safe** (one document per thread)

```c
// SAFE: Each thread has its own document
void* worker(void* arg) {
    HedlDocument* doc = NULL;
    hedl_parse(input, -1, 0, &doc);
    // ... use doc ...
    hedl_free_document(doc);
}

// UNSAFE: Sharing documents across threads
HedlDocument* doc = create_doc();
pthread_create(&t1, NULL, worker1, doc);  // DANGER!
pthread_create(&t2, NULL, worker2, doc);  // DANGER!
```

## Next Steps

- **Tutorials**: Step-by-step guides for common tasks
  - [Rust Quickstart](tutorials/01-rust-quickstart.md)
  - [FFI Integration](tutorials/02-ffi-integration.md)
  - [WASM Browser Integration](tutorials/03-wasm-browser.md)
  - [MCP Server Usage](tutorials/04-mcp-server.md)

- **Guides**: Best practices and patterns
  - [Rust Best Practices](guides/rust-best-practices.md)
  - [Thread Safety](guides/thread-safety.md)
  - [Memory Management](guides/memory-management.md)
  - [Error Handling](guides/error-handling.md)

- **Reference**: Detailed API documentation
  - [Core Types](reference/core-types.md)
  - [Parser API](reference/parser-api.md)
  - [Serializer API](reference/serializer-api.md)
  - [Utility Functions](reference/utility-functions.md)

- **Examples**: Real-world code samples
  - [Complete Examples](examples.md)

## Support

- **GitHub Issues**: https://github.com/dweve/hedl/issues
- **Documentation**: https://hedl.dev/docs
- **Examples**: See `examples/` directory in the repository

## License

HEDL is licensed under the Apache License 2.0. See LICENSE file for details.
