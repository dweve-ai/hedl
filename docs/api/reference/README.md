# API Reference

Comprehensive API reference documentation for all HEDL types, functions, and interfaces.

## Core Reference

- **[Core Types](core-types.md)** - Document, Value, Item, Node structures
- **[Parser API](parser-api.md)** - Parsing functions and options
- **[Serializer API](serializer-api.md)** - Serialization and format conversion
- **[Utility Functions](utility-functions.md)** - Helper functions and utilities

## Language-Specific References

### Rust
- [Rust API Reference](../rust-api.md)
- [hedl crate documentation](https://docs.rs/hedl)
- [Core types reference](core-types.md)

### C/C++ (FFI)
- [FFI API Reference](../ffi-api.md)
- [Thread safety details](../guides/thread-safety.md#ffi-thread-safety)
- [Memory management](../guides/memory-management.md#ffi-memory-management)

### JavaScript/TypeScript (WASM)
- [WASM API Reference](../wasm-api.md)
- [TypeScript definitions](https://www.npmjs.com/package/hedl-wasm?activeTab=code)
- [Browser integration](../tutorials/03-wasm-browser.md)

### MCP Server
- [MCP API Reference](../mcp-api.md)
- [Tool specifications](../mcp-api.md#available-tools)
- [Server usage](../tutorials/04-mcp-server.md)

### LSP Server
- [LSP API Reference](../lsp-api.md)
- [Editor integration](../lsp-api.md#editor-integration)

## Quick Reference

### Parsing

```rust
// Rust
use hedl::parse;
let doc = parse(input)?;

// C
HedlDocument* doc = NULL;
hedl_parse(input, -1, 0, &doc);

// JavaScript
import { parse } from 'hedl-wasm';
const doc = parse(input);
```

### Serialization

```rust
// To JSON
use hedl::to_json;
let json = to_json(&doc)?;

// To canonical HEDL
use hedl::canonicalize;
let canonical = canonicalize(&doc)?;
```

### Validation

```rust
// Quick validation
use hedl::validate;
validate(input)?;

// Detailed linting
use hedl::lint;
let diagnostics = lint(&doc);
```

## Type Hierarchy

```
Document
├── version: (u32, u32)
├── aliases: BTreeMap<String, String>
├── structs: BTreeMap<String, Vec<String>>
├── nests: BTreeMap<String, String>
└── root: BTreeMap<String, Item>

Item (enum)
├── Scalar(Value)
├── Object(BTreeMap<String, Item>)
└── List(MatrixList)

Value (enum)
├── Null
├── Bool(bool)
├── Int(i64)
├── Float(f64)
├── String(String)
├── Tensor(Tensor)
├── Reference(Reference)
└── Expression(Expression)

Node (for MatrixList rows)
├── type_name: String
├── id: String
├── fields: Vec<Value>
├── children: BTreeMap<String, Vec<Node>>
└── child_count: Option<usize>
```

## Function Index

### Parsing
- `parse()` - Parse HEDL string
- `parse_lenient()` - Parse with lenient reference handling
- `parse_with_limits()` - Parse with custom limits
- `validate()` - Validate HEDL syntax

### Serialization
- `to_json()` - Convert to JSON
- `from_json()` - Convert from JSON
- `canonicalize()` - Convert to canonical HEDL
- `to_yaml()` - Convert to YAML (feature-gated)
- `to_xml()` - Convert to XML (feature-gated)

### Validation
- `lint()` - Lint document
- `lint_with_config()` - Lint with custom rules

### Format Conversion
- `from_csv()` - Import from CSV (feature-gated)
- `to_parquet()` - Export to Parquet (feature-gated)
- `to_cypher()` - Export to Neo4j Cypher (feature-gated)

## Error Types

See [Errors Reference](../errors.md) for complete error documentation.

Common error types:
- `HedlError` - Main error type
- `HedlErrorKind` - Error classification
- `HedlResult<T>` - Result type alias

## Configuration Types

### ParseOptions
```rust
pub struct ParseOptions {
    pub limits: Limits,
    pub strict_refs: bool,
}
```

### Limits
```rust
pub struct Limits {
    pub max_file_size: usize,
    pub max_line_length: usize,
    pub max_indent_depth: usize,
    pub max_nodes: usize,
    pub max_aliases: usize,
    pub max_columns: usize,
    pub max_nest_depth: usize,
    pub max_block_string_size: usize,
    pub max_object_keys: usize,
    pub max_total_keys: usize,
}
```

## See Also

- [Getting Started](../getting-started.md) - API quickstart
- [Tutorials](../tutorials/README.md) - Step-by-step guides
- [Guides](../guides/README.md) - Best practices
- [Examples](../examples.md) - Code examples
