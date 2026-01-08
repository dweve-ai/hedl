# HEDL - Hierarchical Entity Data Language

**A high-performance data serialization format optimized for AI/ML applications**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)]()

---

## Overview

HEDL (Hierarchical Entity Data Language) is a text-based data serialization format that combines the **token efficiency of CSV-style tables** with the **semantic richness of hierarchical structures**. Designed specifically for AI/ML data representation, HEDL reduces token consumption by up to 60% compared to JSON while maintaining full expressiveness.

### Key Features

- **Token-Efficient**: CSV-style matrix lists reduce token consumption dramatically
- **Type-Safe**: Structured types with schema validation
- **Graph-Friendly**: First-class support for references and relationships
- **Multi-Format**: Seamless conversion between JSON, YAML, XML, CSV, Parquet, Neo4j Cypher
- **High Performance**: 30+ MB/s parsing throughput with zero-copy optimizations
- **Production-Ready**: Comprehensive FFI bindings, WASM support, LSP server, and MCP integration

---

## Quick Start

### Installation

Add HEDL to your `Cargo.toml`:

```toml
[dependencies]
hedl = "1.0.0"

# Optional format converters
hedl = { version = "1.0.0", features = ["all-formats"] }
```

### Basic Usage

```rust
use hedl::{parse, to_json, canonicalize, validate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a HEDL document
    let hedl_text = r#"
%VERSION: 1.0
%STRUCT: Product: [id, name, price, category]
---
# Product catalog
products: @Product
  | laptop, ThinkPad X1, 1299.99, electronics
  | mouse, Wireless Mouse, 29.99, accessories
  | keyboard, Mechanical Keyboard, 149.99, accessories

store_name: Tech Paradise
location: San Francisco
"#;

    // Parse the document
    let doc = parse(hedl_text)?;

    // Validate structure
    validate(hedl_text)?;

    // Convert to JSON
    let json = to_json(&doc)?;
    println!("{}", json);

    // Canonicalize (deterministic formatting)
    let canonical = canonicalize(&doc)?;
    println!("{}", canonical);

    Ok(())
}
```

---

## Format Examples

### HEDL Syntax

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email, age]
%STRUCT: Post: [id, author, title, tags]
---
# Users with type-scoped IDs
users: @User
  | alice, Alice Smith, alice@example.com, 28
  | bob, Bob Johnson, bob@example.com, 35

# Posts with references to users
posts: @Post
  | p1, @User:alice, First Post, [tech, rust]
  | p2, @User:bob, Another Post, [programming, web]

# Ditto operator for repeated values
events: @Event
  | e1, 2024-01-15, conference, San Francisco
  | e2, ^, workshop, ^
  | e3, 2024-01-16, meetup, ^
```

### Equivalent JSON

```json
{
  "users": [
    {"id": "alice", "name": "Alice Smith", "email": "alice@example.com", "age": 28},
    {"id": "bob", "name": "Bob Johnson", "email": "bob@example.com", "age": 35}
  ],
  "posts": [
    {"id": "p1", "author": {"@ref": "User:alice"}, "title": "First Post", "tags": ["tech", "rust"]},
    {"id": "p2", "author": {"@ref": "User:bob"}, "title": "Another Post", "tags": ["programming", "web"]}
  ],
  "events": [
    {"id": "e1", "date": "2024-01-15", "type": "conference", "location": "San Francisco"},
    {"id": "e2", "date": "2024-01-15", "type": "workshop", "location": "San Francisco"},
    {"id": "e3", "date": "2024-01-16", "type": "meetup", "location": "San Francisco"}
  ]
}
```

**Token Savings**: 342 tokens (HEDL) vs 523 tokens (JSON) = **35% reduction**

---

## Supported Formats

| Format | Read | Write | Streaming | Use Case |
|--------|------|-------|-----------|----------|
| **JSON** | ✅ | ✅ | ✅ | General-purpose interchange |
| **YAML** | ✅ | ✅ | ❌ | Configuration files |
| **XML** | ✅ | ✅ | ✅ | Legacy system integration |
| **CSV** | ✅ | ✅ | ❌ | Tabular data export |
| **Parquet** | ✅ | ✅ | ❌ | Analytics and data warehousing |
| **Neo4j Cypher** | ✅ | ✅ | ✅ | Graph database import |
| **TOON** | ❌ | ✅ | ❌ | Type-Object Notation |

---

## CLI Tool

Install the HEDL CLI for command-line operations:

```bash
cargo install hedl-cli
```

### Usage Examples

```bash
# Convert HEDL to JSON
hedl to-json input.hedl -o output.json

# Validate a HEDL document
hedl validate document.hedl

# Lint for best practices
hedl lint document.hedl

# Canonicalize (format deterministically)
hedl format document.hedl -o canonical.hedl

# Show document statistics
hedl stats document.hedl

# Batch process multiple files
hedl batch-validate data/*.hedl
```

---

## Language Bindings

### FFI (C/C++/Python/etc.)

```c
#include "hedl.h"

// Parse HEDL document
HedlDocument* doc = hedl_parse(hedl_text, text_len);

// Convert to JSON
char* json = hedl_to_json(doc);

// Cleanup
hedl_free_string(json);
hedl_free_document(doc);
```

### WebAssembly

```typescript
import init, { parse, toJson } from 'hedl-wasm';

await init();

const hedlText = `
%VERSION: 1.0
users: @User
  | alice, Alice, alice@example.com
`;

const doc = parse(hedlText);
const json = toJson(doc);
console.log(json);
```

### Language Server Protocol (LSP)

HEDL provides a full LSP server for editor integration with:

- Syntax highlighting
- Auto-completion
- Real-time validation
- Go-to-definition
- Hover documentation
- Code actions (quick fixes)

Configure your editor to use `hedl-lsp` for `.hedl` files.

### Model Context Protocol (MCP)

Integrate HEDL with AI/LLM workflows using the MCP server:

```bash
hedl-mcp --port 8080
```

Provides AI-optimized tools for:
- Document parsing and validation
- Format conversion
- Schema inference
- Query and transformation

---

## Performance

HEDL is engineered for high performance with extensive benchmarking:

| Operation | Throughput | Latency |
|-----------|------------|---------|
| **Parsing** | 30-45 MB/s | 19.6 µs (small), 265 µs (medium) |
| **JSON Conversion** | N/A | 83.5 µs (medium) |
| **YAML Conversion** | N/A | 400 µs (medium) |
| **CSV Conversion** | N/A | 22.3 µs (medium) |
| **Canonicalization** | N/A | 40.5 µs (medium) |
| **Linting** | N/A | 709 ns (medium) |

### Performance Characteristics

- **Linear Scaling**: Maintains performance with growing document size
- **Zero-Copy Parsing**: Minimal allocations for string data
- **SIMD Optimizations**: Vectorized string operations where applicable
- **Streaming Support**: Process multi-GB files with constant memory
- **Async API**: Non-blocking I/O for high-concurrency scenarios

See [benchmarks](crates/hedl-bench/target/comprehensive_report.md) for detailed performance analysis.

---

## Architecture

HEDL is organized as a modular workspace with 19 specialized crates:

### Core Parsing & Representation
- **hedl-core**: Core parser and data model
- **hedl**: Main library with unified API
- **hedl-stream**: Streaming parser for large files

### Format Converters
- **hedl-json**: JSON serialization/deserialization
- **hedl-yaml**: YAML conversion
- **hedl-xml**: XML conversion with streaming support
- **hedl-csv**: CSV file import/export
- **hedl-parquet**: Apache Parquet integration
- **hedl-neo4j**: Neo4j Cypher generation
- **hedl-toon**: Type-Object Notation output

### Tooling & Validation
- **hedl-c14n**: Canonicalization (deterministic formatting)
- **hedl-lint**: Linting and best practices
- **hedl-cli**: Command-line interface
- **hedl-test**: Testing utilities and fixtures

### Bindings & Integration
- **hedl-ffi**: C FFI bindings
- **hedl-wasm**: WebAssembly bindings
- **hedl-lsp**: Language Server Protocol implementation
- **hedl-mcp**: Model Context Protocol server

### Infrastructure
- **hedl-bench**: Comprehensive benchmarking suite

---

## Documentation

- [Language Specification](docs/SPECIFICATION.md) - Complete HEDL syntax reference
- [API Documentation](https://docs.rs/hedl) - Rust API docs
- [Format Conversion Guide](docs/CONVERSION.md) - Multi-format workflows
- [Performance Tuning](docs/PERFORMANCE.md) - Optimization strategies
- [CLI Reference](crates/hedl-cli/README.md) - Command-line tool usage
- [FFI Guide](crates/hedl-ffi/README.md) - C bindings integration
- [WASM Guide](crates/hedl-wasm/README.md) - Browser/Node.js usage

---

## Use Cases

### AI/ML Data Pipelines
- **Token Efficiency**: Reduce LLM token consumption by 35-60%
- **Structured Output**: Type-safe structured generation for LLMs
- **Graph Relationships**: Natural representation of knowledge graphs

### Data Engineering
- **ETL Workflows**: Convert between JSON, CSV, Parquet seamlessly
- **Schema Evolution**: Flexible schema with backward compatibility
- **Large-Scale Processing**: Stream multi-GB files efficiently

### Knowledge Graphs
- **Neo4j Import**: Direct Cypher generation for graph databases
- **Reference Resolution**: Built-in support for entity relationships
- **Type Namespacing**: Clean separation of entity types

### Configuration Management
- **Human-Readable**: More compact than JSON/YAML for tables
- **Validation**: Schema enforcement with struct definitions
- **Versioning**: Built-in version tracking and compatibility

---

## Contributing

We welcome contributions! HEDL is developed by Dweve B.V. and the open-source community.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/dweve-ai/hedl.git
cd hedl

# Build all crates
cargo build --all-features

# Run tests
cargo test --all-features

# Run benchmarks
cargo bench --all-features
```

### Testing Standards

HEDL maintains high quality standards:
- **Unit Tests**: Comprehensive coverage for all public APIs
- **Integration Tests**: Cross-crate workflow validation
- **Property Tests**: Fuzz testing with `proptest`
- **Benchmarks**: Performance regression detection
- **Security Tests**: Input validation and resource limits

### Code Review

All contributions go through:
1. Automated CI checks (format, lint, tests)
2. Benchmark regression analysis
3. Security review for external inputs
4. Documentation completeness check
5. Code review by maintainers

---

## License

Copyright © 2025 Dweve IP B.V. and individual contributors.

Licensed under the **Apache License, Version 2.0** (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at:

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

---

## Project Links

- **Homepage**: https://dweve.com
- **Repository**: https://github.com/dweve-ai/hedl
- **Documentation**: https://docs.rs/hedl
- **Crate**: https://crates.io/crates/hedl
- **Issues**: https://github.com/dweve-ai/hedl/issues

---

## Acknowledgments

HEDL builds upon the excellent work of the Rust ecosystem:

- **serde**: Serialization framework
- **quick-xml**: Fast XML parsing
- **parquet**: Apache Parquet implementation
- **criterion**: Benchmarking harness
- **tower-lsp**: LSP server framework

---

**Built with Rust** 🦀 | **Optimized for AI** 🤖 | **Production-Ready** ✅
