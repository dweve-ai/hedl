# HEDL API Documentation

You've decided to integrate HEDL into your application. Excellent choice.

Whether you're building a Rust service, embedding HEDL in a Python pipeline, running conversions in a browser, or adding HEDL support to an AI agent, there's an API designed for your use case.

This guide will help you pick the right integration path and get you writing code fast.

---

## Which API Should You Use?

Let's match your situation to the right API:

### "I'm building a Rust application"

You want the **native Rust API**. It's the fastest, most feature-complete, and most idiomatic option.

```rust
use hedl::{parse, to_json};

let doc = parse(hedl_text)?;
let json = to_json(&doc)?;
```

**Start here**: [Rust API Guide](rust-api.md)

### "I need to call HEDL from Python, C, C++, or Go"

You want the **FFI (Foreign Function Interface) API**. It exposes a C-compatible ABI that works with any language that can call C functions.

```c
#include "hedl.h"

HedlDocument* doc = NULL;
hedl_parse(hedl_text, -1, 1, &doc);

char* json = NULL;
hedl_to_json(doc, 0, &json);
```

**Start here**: [FFI/C API Guide](ffi-api.md)

### "I'm running in a browser or Node.js"

You want the **WebAssembly API**. Same core engine, compiled to WASM for JavaScript environments.

```javascript
import init, { parse, toJson } from 'hedl-wasm';

await init();
const doc = parse(hedlText);
const json = toJson(doc);
```

**Start here**: [WASM/JavaScript API Guide](wasm-api.md)

### "I'm building an AI agent that needs HEDL tools"

You want the **MCP (Model Context Protocol) Server**. It exposes HEDL operations as tools that Claude and other MCP-compatible agents can call.

```json
{
  "mcpServers": {
    "hedl": {
      "command": "hedl-mcp",
      "args": ["--auto-convert"]
    }
  }
}
```

**Start here**: [MCP Server API Guide](mcp-api.md)

### "I want editor integration (VS Code, Neovim, Emacs)"

You want the **LSP (Language Server Protocol) Server**. It provides syntax highlighting, error checking, autocomplete, and more.

**Start here**: [LSP API Guide](lsp-api.md)

---

## Quick Reference: Operations Across APIs

Here's what you can do and how to do it in each API:

### Parsing and Conversion

| Operation | Rust | FFI (C) | WASM (JS) |
|-----------|------|---------|-----------|
| **Parse HEDL** | `hedl::parse()` | `hedl_parse()` | `parse()` |
| **To JSON** | `hedl::to_json()` | `hedl_to_json()` | `toJson()` |
| **From JSON** | `hedl::from_json()` | `hedl_from_json()` | `fromJson()` |
| **To YAML** | `hedl::yaml::to_yaml()` | `hedl_to_yaml()` | `toYaml()` |
| **From YAML** | `hedl::yaml::from_yaml()` | `hedl_from_yaml()` | `fromYaml()` |
| **To XML** | `hedl::xml::to_xml()` | `hedl_to_xml()` | `toXml()` |
| **From XML** | `hedl::xml::from_xml()` | `hedl_from_xml()` | `fromXml()` |
| **To CSV** | `hedl::csv_file::to_csv()` | `hedl_to_csv()` | `toCsv()` |
| **From CSV** | `hedl::csv_file::from_csv()` | `hedl_from_csv()` | `fromCsv()` |
| **To TOON** | `hedl::toon::to_toon()` | `hedl_to_toon()` | `toToon()` |
| **From TOON** | `hedl::toon::from_toon()` | `hedl_from_toon()` | `fromToon()` |

### Utilities

| Operation | Rust | FFI (C) | WASM (JS) |
|-----------|------|---------|-----------|
| **Validate** | `hedl::validate()` | `hedl_validate()` | `validate()` |
| **Canonicalize** | `hedl::canonicalize()` | `hedl_canonicalize()` | `format()` |
| **Lint** | `hedl::lint()` | `hedl_lint()` | `lint()` |
| **Token count** | `hedl::token_count()` | `hedl_token_count()` | `tokenCount()` |

---

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
hedl = "2.0"
```

Want specific format support? Use feature flags:

```toml
[dependencies]
hedl = { version = "2.0", features = ["yaml", "xml", "csv"] }

# Or get everything:
hedl = { version = "2.0", features = ["all-formats"] }
```

Available features:
| Feature | What it enables |
|---------|-----------------|
| `json` | JSON conversion (always enabled) |
| `yaml` | YAML conversion |
| `xml` | XML conversion |
| `csv` | CSV conversion |
| `parquet` | Apache Parquet conversion |
| `neo4j` | Neo4j Cypher generation |
| `toon` | TOON format conversion |
| `all-formats` | All of the above |

### C/FFI

**Option 1: Pre-built binaries**

Download from the [releases page](https://github.com/dweve-ai/hedl/releases):
- `libhedl_ffi.so` (Linux)
- `libhedl_ffi.dylib` (macOS)
- `hedl_ffi.dll` + `hedl_ffi.lib` (Windows)

**Option 2: Build from source**

```bash
git clone https://github.com/dweve-ai/hedl.git
cd hedl
cargo build --release --package hedl-ffi
# Output in target/release/
```

**Header file**: `bindings/c/hedl.h`

### JavaScript/TypeScript (WASM)

**npm/yarn:**

```bash
npm install hedl-wasm
# or
yarn add hedl-wasm
```

**CDN (for browsers):**

```html
<script type="module">
  import init, { parse, toJson } from 'https://unpkg.com/hedl-wasm';
  await init();
  // Ready to use
</script>
```

**Deno:**

```typescript
import init, { parse, toJson } from "https://esm.sh/hedl-wasm";
await init();
```

### MCP Server

```bash
# Install globally
cargo install hedl-mcp

# Verify
hedl-mcp --version
```

Then add to your MCP configuration (e.g., `claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "hedl": {
      "command": "hedl-mcp"
    }
  }
}
```

### LSP Server

```bash
# Install globally
cargo install hedl-lsp

# Verify
hedl-lsp --version
```

Then configure your editor. See [LSP API Guide](lsp-api.md) for editor-specific instructions.

---

## Your First Integration

Let's write some actual code. Here's "Hello, HEDL" in each API:

### Rust

```rust
use hedl::{parse, to_json, from_json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a HEDL document
    let hedl_input = r#"
%V:2.0
%NULL:~
%QUOTE:"
---
greeting: Hello, World!
count: 42
active: true
"#;

    let doc = parse(hedl_input)?;

    // Convert to JSON
    let json = to_json(&doc)?;
    println!("As JSON: {}", json);
    // Output: {"greeting":"Hello, World!","count":42,"active":true}

    // Convert JSON back to HEDL
    let json_input = r#"{"name": "Alice", "age": 30}"#;
    let hedl_output = from_json(json_input)?;
    println!("As HEDL:\n{}", hedl_output);

    Ok(())
}
```

### C

```c
#include <stdio.h>
#include "hedl.h"

int main() {
    const char* hedl_input =
        "%V:2.0\n"
        "%NULL:~\n"
        "%QUOTE:\"\n"
        "---\n"
        "greeting: Hello, World!\n"
        "count: 42\n"
        "active: true\n";

    // Parse
    HedlDocument* doc = NULL;
    if (hedl_parse(hedl_input, -1, 1, &doc) != HEDL_OK) {
        fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
        return 1;
    }

    // Convert to JSON
    char* json = NULL;
    if (hedl_to_json(doc, 0, &json) != HEDL_OK) {
        fprintf(stderr, "Conversion error: %s\n", hedl_get_last_error());
        hedl_document_free(doc);
        return 1;
    }

    printf("As JSON: %s\n", json);

    // Clean up
    hedl_string_free(json);
    hedl_document_free(doc);
    return 0;
}
```

### JavaScript

```javascript
import init, { parse, toJson, fromJson } from 'hedl-wasm';

async function main() {
    await init();

    const hedlInput = `
%V:2.0
%NULL:~
%QUOTE:"
---
greeting: Hello, World!
count: 42
active: true
`;

    // Parse and convert to JSON
    const doc = parse(hedlInput);
    const json = toJson(doc);
    console.log('As JSON:', json);

    // Convert JSON to HEDL
    const jsonInput = '{"name": "Alice", "age": 30}';
    const hedlOutput = fromJson(jsonInput);
    console.log('As HEDL:', hedlOutput);
}

main();
```

### Python (via FFI)

```python
from ctypes import *

# Load the library
hedl = cdll.LoadLibrary("libhedl_ffi.so")

# Set up function signatures
hedl.hedl_parse.argtypes = [c_char_p, c_long, c_int, POINTER(c_void_p)]
hedl.hedl_parse.restype = c_int
hedl.hedl_to_json.argtypes = [c_void_p, c_int, POINTER(c_char_p)]
hedl.hedl_to_json.restype = c_int

hedl_input = b"""
%V:2.0
%NULL:~
%QUOTE:"
---
greeting: Hello, World!
count: 42
active: true
"""

# Parse
doc = c_void_p()
result = hedl.hedl_parse(hedl_input, -1, 1, byref(doc))
if result != 0:
    print("Parse error")
    exit(1)

# Convert to JSON
json_output = c_char_p()
result = hedl.hedl_to_json(doc, 0, byref(json_output))
if result == 0:
    print("As JSON:", json_output.value.decode())

# Clean up
hedl.hedl_string_free(json_output)
hedl.hedl_document_free(doc)
```

---

## Understanding the Type System

HEDL has a rich type system that maps cleanly to each API:

| HEDL Type | Example | Rust | C/FFI | JavaScript |
|-----------|---------|------|-------|------------|
| Null | `~` | `Value::Null` | `HEDL_VALUE_NULL` | `null` |
| Boolean | `true`, `false` | `Value::Bool(bool)` | `int` (0/1) | `boolean` |
| Integer | `42`, `-10` | `Value::Int(i64)` | `int64_t` | `number` |
| Float | `3.14`, `1.5e-10` | `Value::Float(f64)` | `double` | `number` |
| String | `hello`, `"quoted"` | `Value::String(Box<str>)` | `char*` | `string` |
| Reference | `@User:alice` | `Value::Reference { type_name, id }` | `HedlReference*` | `{ type: string, id: string }` |
| Tensor | `[1, 2, 3]` | `Value::Tensor(Vec<...>)` | `HedlTensor*` | `number[]` |
| List | `(a, b, c)` | `Value::List(Vec<Value>)` | `HedlList*` | `any[]` |

### Type Coercion

When converting between formats, HEDL preserves types as accurately as possible:

| HEDL → JSON | Result |
|-------------|--------|
| Integer `42` | JSON number `42` |
| Float `3.14` | JSON number `3.14` |
| Boolean `true` | JSON boolean `true` |
| Null `~` | JSON `null` |
| String `hello` | JSON string `"hello"` |
| Tensor `[1, 2, 3]` | JSON array `[1, 2, 3]` |
| List `(a, b, c)` | JSON array `["a", "b", "c"]` |
| Reference `@User:alice` | JSON string `"@User:alice"` or expanded object (configurable) |

---

## Feature Support Matrix

Not every API supports every feature. Here's what's available where:

| Feature | Rust | FFI | WASM | MCP | LSP |
|---------|:----:|:---:|:----:|:---:|:---:|
| Parsing | ✓ | ✓ | ✓ | ✓ | ✓ |
| JSON conversion | ✓ | ✓ | ✓ | ✓ | |
| YAML conversion | ✓ | ✓ | ✓ | ✓ | |
| XML conversion | ✓ | ✓ | ✓ | ✓ | |
| CSV conversion | ✓ | ✓ | ✓ | ✓ | |
| Parquet conversion | ✓ | ✓ | | ✓ | |
| Neo4j/Cypher | ✓ | ✓ | | ✓ | |
| TOON conversion | ✓ | ✓ | ✓ | ✓ | |
| Validation | ✓ | ✓ | ✓ | ✓ | ✓ |
| Linting | ✓ | ✓ | ✓ | ✓ | |
| Canonicalization | ✓ | ✓ | ✓ | ✓ | ✓ |
| Streaming | ✓ | | | ✓ | |
| Autocomplete | | | | | ✓ |
| Hover info | | | | | ✓ |
| Go to definition | | | | | ✓ |
| Find references | | | | | ✓ |

---

## Error Handling

Each API has its own error handling idiom:

### Rust: `Result<T, HedlError>`

```rust
use hedl::{parse, HedlError};

fn process(input: &str) -> Result<String, HedlError> {
    let doc = parse(input)?;  // ? propagates errors
    let json = hedl::to_json(&doc)?;
    Ok(json)
}

// Or handle explicitly:
match parse(input) {
    Ok(doc) => println!("Parsed {} nodes", doc.root.len()),
    Err(e) => {
        eprintln!("Error at line {}: {}", e.line().unwrap_or(0), e);
    }
}
```

### FFI: Integer codes + `hedl_get_last_error()`

```c
HedlDocument* doc = NULL;
int result = hedl_parse(input, -1, 1, &doc);

if (result != HEDL_OK) {
    // Get the error message
    const char* error = hedl_get_last_error();
    fprintf(stderr, "Error: %s\n", error);

    // Error codes:
    // HEDL_OK = 0
    // HEDL_ERR_PARSE = 1
    // HEDL_ERR_INVALID_INPUT = 2
    // HEDL_ERR_MEMORY = 3
    // etc.
}
```

### WASM: JavaScript exceptions

```javascript
try {
    const doc = parse(hedlInput);
    const json = toJson(doc);
} catch (error) {
    // error.message contains the HEDL error message
    console.error('HEDL error:', error.message);

    // For parse errors, includes line number:
    // "Parse error at line 5: unexpected token"
}
```

---

## Thread Safety

### Rust

Parsing and conversion functions are thread-safe. You can call them from multiple threads simultaneously.

```rust
use std::thread;
use hedl::parse;

let handles: Vec<_> = (0..4)
    .map(|_| {
        thread::spawn(|| {
            parse("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value")
        })
    })
    .collect();

for handle in handles {
    let result = handle.join().unwrap();
    assert!(result.is_ok());
}
```

`Document` objects are NOT `Sync`. If you need to share a document between threads, wrap it in `Arc<Mutex<Document>>` or clone it.

### FFI

- **Parsing/conversion functions**: Thread-safe (each call is independent)
- **Error messages**: Thread-local storage (each thread has its own error state)
- **Document handles**: NOT thread-safe (don't share between threads)

### WASM

JavaScript is single-threaded by default. For concurrent processing, use Web Workers:

```javascript
// main.js
const worker = new Worker('hedl-worker.js');
worker.postMessage({ hedl: hedlInput });
worker.onmessage = (e) => console.log('Result:', e.data.json);

// hedl-worker.js
import init, { parse, toJson } from 'hedl-wasm';

self.onmessage = async (e) => {
    await init();
    const doc = parse(e.data.hedl);
    const json = toJson(doc);
    self.postMessage({ json });
};
```

---

## Performance Characteristics

### Parsing Speed

Benchmarked on typical documents (Intel i7, release build):

| Document Size | Parse Time | Throughput |
|---------------|------------|------------|
| Tiny (< 1 KB) | ~37 µs | ~27 MB/s |
| Small (1-10 KB) | ~396 µs | ~25 MB/s |
| Medium (10-100 KB) | ~12 ms | ~8 MB/s |
| Large (> 100 KB) | Use streaming | Variable |

### Memory Usage

| API | Memory Overhead | Notes |
|-----|-----------------|-------|
| Rust | ~2-3x input size | AST owns strings |
| FFI | ~2-3x input size | Same as Rust |
| WASM | ~3-4x input size | JS object overhead |

### Token Savings (the reason you're here)

| Data Type | JSON Tokens | HEDL Tokens | Savings |
|-----------|-------------|-------------|---------|
| Flat records | 100 | 44 | 56% |
| Nested objects | 100 | 50 | 50% |
| Matrix data | 100 | 35 | 65% |

---

## Documentation Map

### Getting Started

| Guide | What you'll learn |
|-------|-------------------|
| [Getting Started](getting-started.md) | Quick integration overview |
| [Rust Quickstart](tutorials/01-rust-quickstart.md) | First Rust integration |
| [FFI Integration](tutorials/02-ffi-integration.md) | C/Python integration |
| [WASM in Browser](tutorials/03-wasm-browser.md) | Browser integration |
| [MCP Server](tutorials/04-mcp-server.md) | AI agent integration |

### API References

| Guide | What's inside |
|-------|---------------|
| [Rust API](rust-api.md) | Complete Rust API reference |
| [FFI/C API](ffi-api.md) | C function signatures and usage |
| [WASM API](wasm-api.md) | JavaScript/TypeScript API |
| [MCP API](mcp-api.md) | MCP tools and resources |
| [LSP API](lsp-api.md) | LSP capabilities and configuration |

### Deep Dives

| Guide | What you'll learn |
|-------|-------------------|
| [Error Handling](guides/error-handling.md) | Comprehensive error handling |
| [Memory Management](guides/memory-management.md) | FFI memory patterns |
| [Thread Safety](guides/thread-safety.md) | Concurrent usage patterns |
| [Rust Best Practices](guides/rust-best-practices.md) | Idiomatic Rust usage |

### SDK Documentation

| SDK | What's inside |
|-----|---------------|
| [Rust SDK](sdk/rust.md) | Rust crate documentation |
| [JavaScript SDK](sdk/javascript.md) | npm package documentation |
| [Python SDK](sdk/python.md) | Python bindings documentation |
| [C/C++ SDK](sdk/c-cpp.md) | C header and usage |

### Reference

| Document | What's inside |
|----------|---------------|
| [Core Types](reference/core-types.md) | Type definitions |
| [Parser API](reference/parser-api.md) | Parser internals |
| [Serializer API](reference/serializer-api.md) | Conversion internals |
| [Utility Functions](reference/utility-functions.md) | Helper functions |
| [Errors Reference](errors.md) | All error types |

---

## What's Next?

Pick your path:

**Ready to integrate?**
→ Jump to your API: [Rust](rust-api.md) | [FFI](ffi-api.md) | [WASM](wasm-api.md) | [MCP](mcp-api.md) | [LSP](lsp-api.md)

**Want working code first?**
→ [Examples](examples.md) (all languages, all APIs)

**Need to understand the data model?**
→ [Core Types Reference](reference/core-types.md)

---

<p align="center">
  <em>Pick an API, write some code, save some tokens.</em>
</p>
