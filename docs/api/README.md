# HEDL API Documentation

**Comprehensive reference for HEDL APIs across Rust, C/FFI, WebAssembly, and language services.**

---

## Overview

HEDL provides multiple API surfaces to support different use cases and programming environments:

- **[Rust API](rust-api.md)** - Native Rust library for high-performance HEDL processing
- **[FFI/C API](ffi-api.md)** - C-compatible bindings for integration with C, Python, Go, and other languages
- **[WASM/JavaScript API](wasm-api.md)** - WebAssembly bindings for browser and Node.js environments
- **[MCP Server API](mcp-api.md)** - Model Context Protocol server for AI/LLM integration
- **[LSP API](lsp-api.md)** - Language Server Protocol for IDE integration
- **[Examples](examples.md)** - Code examples across all APIs

---

## Quick Navigation

### By Use Case

| Use Case | API | Quick Link |
|----------|-----|------------|
| Native Rust application | Rust | [Getting Started](rust-api.md#quick-start) |
| Python/Go/C integration | FFI | [FFI Quick Start](ffi-api.md#quick-start) |
| Browser/JavaScript | WASM | [WASM Quick Start](wasm-api.md#quick-start) |
| AI/LLM integration | MCP | [MCP Tools](mcp-api.md#available-tools) |
| IDE/editor integration | LSP | [LSP Features](lsp-api.md#features) |

### By Operation

| Operation | Rust | FFI | WASM |
|-----------|------|-----|------|
| **Parse HEDL** | `hedl::parse()` | `hedl_parse()` | `parse()` |
| **Convert to JSON** | `hedl::to_json()` | `hedl_to_json()` | `toJson()` |
| **Convert from JSON** | `hedl::from_json()` | `hedl_from_json()` | `fromJson()` |
| **Validate** | `hedl::validate()` | `hedl_validate()` | `validate()` |
| **Canonicalize** | `hedl::canonicalize()` | `hedl_canonicalize()` | `format()` |
| **Lint** | `hedl::lint()` | `hedl_lint()` | N/A |

---

## Core Concepts

### Document Structure

HEDL documents consist of:

1. **Header**: Version, schemas, aliases, nests
2. **Body**: Entities, key-value pairs, hierarchies

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
```

### Type System

| Type | Syntax | Rust Type | Description |
|------|--------|-----------|-------------|
| Null | `null` | `Value::Null` | Absence of value |
| Boolean | `true`, `false` | `Value::Bool(bool)` | Boolean value |
| Integer | `42`, `-10` | `Value::Int(i64)` | 64-bit signed integer |
| Float | `3.14`, `1.5e-10` | `Value::Float(f64)` | 64-bit floating point |
| String | `"hello"`, `bare` | `Value::String(String)` | UTF-8 text |
| Reference | `@User:alice` | `Value::Reference` | Entity reference |
| Tensor | `[1, 2, 3]` | `Value::Tensor` | Multi-dimensional array |
| Expression | `$(now())` | `Value::Expression` | Deferred computation |

### Feature Support Matrix

| Feature | Rust | FFI | WASM | MCP | LSP |
|---------|------|-----|------|-----|-----|
| Parsing | ✓ | ✓ | ✓ | ✓ | ✓ |
| JSON conversion | ✓ | ✓ | ✓ | ✓ | - |
| YAML conversion | ✓ | ✓ | - | ✓ | - |
| XML conversion | ✓ | ✓ | - | ✓ | - |
| CSV conversion | ✓ | ✓ | - | ✓ | - |
| Parquet conversion | ✓ | ✓ | - | ✓ | - |
| Neo4j/Cypher | ✓ | ✓ | - | ✓ | - |
| TOON conversion | ✓ | - | - | ✓ | - |
| Validation | ✓ | ✓ | ✓ | ✓ | ✓ |
| Linting | ✓ | ✓ | ✓ | ✓ | ✓ |
| Canonicalization | ✓ | ✓ | ✓ | ✓ | ✓ |
| Streaming | ✓ | - | - | ✓ | - |
| Autocomplete | - | - | - | - | ✓ |
| Hover info | - | - | - | - | ✓ |
| Go to definition | - | - | - | - | ✓ |

---

## Installation

### Rust

```toml
[dependencies]
hedl = "1.0"

# Optional features
hedl = { version = "1.0", features = ["yaml", "xml", "csv", "parquet", "neo4j"] }
```

### C/FFI

Download the pre-built library or build from source:

```bash
cargo build --release --package hedl-ffi
```

Link against `libhedl_ffi.so` (Linux), `hedl_ffi.dylib` (macOS), or `hedl_ffi.dll` (Windows).

### WebAssembly/JavaScript

```bash
npm install hedl-wasm
```

Or use from CDN:

```html
<script type="module">
  import init, { parse, toJson } from 'https://unpkg.com/hedl-wasm';
  await init();
</script>
```

### MCP Server

```bash
# Install globally
cargo install hedl-mcp

# Or run from source
cargo run --bin hedl-mcp
```

### LSP Server

```bash
# Install globally
cargo install hedl-lsp

# Or run from source
cargo run --bin hedl-lsp
```

---

## Performance Characteristics

### Memory Usage

| API | Typical Memory Overhead | Maximum Document Size |
|-----|------------------------|----------------------|
| Rust | ~2-3x input size | Limited by available RAM |
| FFI | ~2-3x input size | Configurable (default: unlimited) |
| WASM | ~3-4x input size | Configurable (default: 500 MB) |
| MCP | ~2-3x input size + cache | Configurable per file |
| LSP | ~3-5x input size + cache | Configurable (default: 500 MB) |

### Parsing Speed

Approximate throughput on modern hardware:

- **Simple documents**: 50-100 MB/s
- **Complex nested structures**: 10-30 MB/s
- **Matrix lists**: 100-200 MB/s

### Token Savings

HEDL typically achieves:

- **40-60% token reduction** vs JSON for structured data
- **50-70% reduction** for tabular/matrix data
- **30-50% reduction** for nested hierarchies

---

## Thread Safety

### Rust API

All parsing and conversion functions are thread-safe and can be called from multiple threads concurrently. Document objects (`Document`) are not thread-safe and should not be shared between threads without synchronization.

### FFI API

- **Error messages** are stored in thread-local storage (TLS), making error handling thread-safe
- **Document handles** (`HedlDocument*`) are NOT thread-safe and must not be shared between threads
- Each thread maintains independent error state

### WASM API

Single-threaded by default (JavaScript runtime limitation). Use Web Workers for concurrent processing.

---

## Error Handling

All APIs use explicit error handling:

**Rust**: `Result<T, HedlError>`
```rust
match hedl::parse(input) {
    Ok(doc) => { /* success */ }
    Err(e) => eprintln!("Error: {}", e)
}
```

**FFI**: Integer error codes + `hedl_get_last_error()`
```c
if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
    fprintf(stderr, "Error: %s\n", hedl_get_last_error());
}
```

**WASM**: JavaScript `Error` objects
```javascript
try {
    const doc = parse(hedl);
} catch (e) {
    console.error(e.message);
}
```

---

## Version Compatibility

All APIs support HEDL format version **1.0**.

### API Stability

- **Rust API**: Semver guarantees (1.0+ = stable)
- **FFI API**: C ABI stability guaranteed
- **WASM API**: JavaScript API follows semver
- **MCP API**: Protocol stability (backward compatible)
- **LSP API**: LSP 3.17 specification compliance

---

## Next Steps

1. **Choose your API** based on your use case
2. **Read the specific API documentation** for detailed function signatures
3. **Check the examples** for code samples in your language
4. **Join the community** for support and discussions

### API Documentation

- [Rust API Reference](rust-api.md)
- [FFI/C API Reference](ffi-api.md)
- [WASM/JavaScript API Reference](wasm-api.md)
- [MCP Server API Reference](mcp-api.md)
- [LSP API Reference](lsp-api.md)
- [Cross-Language Examples](examples.md)

---

**Last updated**: 2025-01-06
**HEDL Version**: 1.0
**License**: Apache 2.0
