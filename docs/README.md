# HEDL Documentation

Welcome to the **HEDL** (Hierarchical Entity Data Language) documentation. HEDL is a comprehensive Rust-based data format ecosystem providing high-performance parsing, serialization, and transformation across multiple data formats.

## Quick Navigation

| I am a... | Start here |
|-----------|------------|
| **User** | [User Guide](user/) - Learn to convert data between formats |
| **Developer** | [Developer Guide](developer/) - Contribute to HEDL |
| **API Integrator** | [API Reference](api/) - Integrate HEDL into your project |
| **Architect** | [Architecture Docs](architecture/) - Understand system design |

## Supported Formats

HEDL supports seamless conversion between these formats:

| Format | Read | Write | Streaming |
|--------|------|-------|-----------|
| JSON | Yes | Yes | Yes |
| YAML | Yes | Yes | Yes |
| XML | Yes | Yes | Yes |
| CSV | Yes | Yes | Yes |
| Parquet | Yes | Yes | - |
| Neo4j Cypher | Yes | Yes | - |
| TOON | - | Yes | - |

## Getting Started

### Installation

```bash
# Add to Cargo.toml
cargo add hedl

# Or install the CLI
cargo install hedl-cli
```

### Quick Example

```rust
use hedl::{from_json, parse, to_json};

// Parse HEDL
let doc = parse("%VERSION: 1.0\n---\nname: HEDL\nversion: 1.0")?;

// Convert to JSON
let json = to_json(&doc)?;

// Parse JSON
let doc = from_json(r#"{"name": "HEDL", "version": "1.0"}"#)?;
```

### CLI Usage

```bash
# Convert JSON to HEDL
hedl from-json input.json -o output.hedl

# Validate and format HEDL document
hedl validate document.hedl
hedl format document.hedl

# Get statistics
hedl stats document.hedl --tokens
```

## Documentation Structure

```
docs/
├── README.md              # You are here
├── user/                 # End-user documentation
│   ├── getting-started.md
│   ├── formats.md
│   ├── cli-guide.md
│   └── examples.md
├── developer/            # Developer documentation
│   ├── architecture.md
│   ├── contributing.md
│   └── testing.md
├── api/                  # API reference
│   ├── rust-api.md
│   ├── ffi-api.md
│   └── wasm-api.md
└── architecture/         # System architecture
    └── decisions/
```

## Key Features

- **High Performance**: Optimized parsing with SIMD acceleration
- **Memory-Efficient Streaming**: Process large files efficiently with minimal memory
- **Type Safety**: Strong Rust type system with comprehensive error handling
- **Multiple Bindings**: Native Rust, C/FFI, WebAssembly, MCP, LSP
- **Extensible**: Clean architecture for adding new formats
- **Well Tested**: Comprehensive test suite with property-based and fuzz testing

## Crate Overview

### Core
- **hedl-core**: Core parsing engine, lexer, AST, validation
- **hedl**: Main library facade and public API
- **hedl-c14n**: Document canonicalization

### Format Adapters
- **hedl-json**: JSON serialization/deserialization
- **hedl-yaml**: YAML conversion
- **hedl-xml**: XML transformation
- **hedl-csv**: CSV parsing/generation
- **hedl-toon**: TOON format support
- **hedl-parquet**: Parquet columnar format
- **hedl-neo4j**: Neo4j Cypher generation

### Tools
- **hedl-cli**: Command-line interface
- **hedl-lint**: Code linting and static analysis
- **hedl-lsp**: Language Server Protocol support
- **hedl-mcp**: Model Context Protocol server

### Bindings
- **hedl-ffi**: C API bindings
- **hedl-wasm**: WebAssembly bindings

### Support
- **hedl-stream**: Streaming parser for large files
- **hedl-test**: Testing infrastructure
- **hedl-bench**: Performance benchmarks

## Performance

HEDL is designed for high performance. See our [benchmark results](architecture/performance.md) for detailed metrics.

Key highlights:
- CSV parsing: High throughput for tabular data
- JSON serialization: Optimized for common cases
- Zero-copy streaming: Minimal allocations for large files
- Parallel processing: Scalable for multi-core systems

## Need Help?

- [FAQ](user/faq.md) - Frequently asked questions
- [Troubleshooting](user/troubleshooting.md) - Common issues and solutions
- [GitHub Issues](https://github.com/dweve-ai/hedl/issues) - Report bugs or request features
- [Discussions](https://github.com/dweve-ai/hedl/discussions) - Community support

## License

HEDL is licensed under the [Apache License 2.0](https://github.com/dweve-ai/hedl/blob/master/LICENSE).

---

*Documentation generated with comprehensive codebase analysis*
