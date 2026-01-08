# HEDL Developer Documentation

Welcome to the HEDL (Hierarchical Entity Data Language) developer documentation. This comprehensive guide will help you understand, contribute to, and build upon the HEDL implementation.

## Quick Navigation

### Getting Started
- **[Getting Started Guide](getting-started.md)** - Set up your development environment and build HEDL
- **[Contributing Guide](contributing.md)** - Learn how to contribute to HEDL

### Architecture & Design
- **[Architecture Overview](architecture.md)** - High-level system architecture and design patterns
- **[Module Guide](module-guide.md)** - Detailed overview of all 19 crates
- **[Internals](internals.md)** - Deep dive into parsing, AST, and core concepts

### Development & Testing
- **[Testing Guide](testing.md)** - Testing strategies, running tests, and writing new tests
- **[Benchmarking Guide](benchmarking.md)** - Performance testing and optimization

## What is HEDL?

HEDL (Hierarchical Entity Data Language) is a text-based data serialization format optimized for AI/ML workflows. It combines:

- **Token efficiency**: Minimal structural overhead for LLM context windows
- **Type safety**: Schema-defined structures with compile-time validation
- **Graph semantics**: Built-in identity and reference system
- **Developer ergonomics**: Human-readable, easy to write and maintain

### Key Features

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email, role]
%STRUCT: Post: [id, title, author, tags]
---
users: @User
  | alice, Alice Smith, alice@example.com, admin
  | bob, Bob Jones, bob@example.com, user

posts: @Post
  | post1, "Hello World", @User:alice, [tech, intro]
  | post2, "HEDL Guide", @User:alice, [tech, tutorial]
```

## Repository Structure

The HEDL project is organized as a Rust workspace with 19 specialized crates:

```
hedl/
├── crates/
│   ├── hedl/              # Unified API facade
│   ├── hedl-core/         # Core parser and data model
│   ├── hedl-c14n/         # Canonicalization
│   ├── hedl-json/         # JSON conversion (always available)
│   ├── hedl-yaml/         # YAML conversion (feature-gated)
│   ├── hedl-xml/          # XML conversion (feature-gated)
│   ├── hedl-csv/          # CSV file conversion (feature-gated)
│   ├── hedl-toon/         # TOON format output
│   ├── hedl-parquet/      # Parquet conversion (feature-gated)
│   ├── hedl-neo4j/        # Neo4j Cypher generation (feature-gated)
│   ├── hedl-lint/         # Linting
│   ├── hedl-cli/          # Command-line tool
│   ├── hedl-ffi/          # C FFI bindings
│   ├── hedl-wasm/         # WebAssembly bindings
│   ├── hedl-lsp/          # Language Server Protocol
│   ├── hedl-mcp/          # Model Context Protocol server
│   ├── hedl-stream/       # Streaming parser (sync + async)
│   ├── hedl-test/         # Test utilities and fixtures
│   └── hedl-bench/        # Performance benchmarks
├── docs/                  # Documentation
├── bindings/              # Language bindings
└── tests/                 # Integration tests
```

## Core Concepts

### Document Model

HEDL documents consist of two main sections:

1. **Header** - Directives for configuration and schema definitions
2. **Body** - Actual data in hierarchical structure

```rust
use hedl::parse;

let doc = parse("%VERSION: 1.0\n---\nkey: value")?;
// doc contains the parsed Document with version, structs, and root
```

### AST (Abstract Syntax Tree)

The core data structures:

- `Document` - Top-level container with `version`, `structs`, `aliases`, `nests`, and `root`
- `Item` - Body item enum: `Scalar`, `Object`, or `List`
- `Node` - A row in a matrix list with `type_name`, `id`, `fields`, and `children`
- `MatrixList` - Typed list with `type_name`, `schema`, and `rows`
- `Value` - Scalar values: `Null`, `Bool`, `Int`, `Float`, `String`, `Tensor`, `Reference`, `Expression`
- `HedlError` - Unified error type with kind, message, and line number

### Type System

HEDL supports:

- **Scalar types**: String, Integer, Float, Boolean, Null
- **Complex types**: Tensors (multi-dimensional arrays), Expressions
- **References**: `@id` or `@Type:id` for graph relationships
- **Schema types**: User-defined structs with ordered columns

### Parsing Pipeline

```
Input Text
    ↓
Lexical Analysis (tokenization, indentation)
    ↓
Header Parsing (directives, schemas)
    ↓
Body Parsing (objects, lists, values)
    ↓
Reference Resolution (link IDs)
    ↓
Validation (type checking, constraints)
    ↓
Document (AST)
```

## Development Workflow

### Quick Start

```bash
# Clone and build
git clone https://github.com/dweve-ai/hedl.git
cd hedl
cargo build

# Build with all features
cargo build --all-features

# Run tests
cargo test

# Run benchmarks
cargo bench

# Build documentation
cargo doc --workspace --all-features --no-deps --open
```

### Common Tasks

| Task | Command |
|------|---------|
| Build all crates | `cargo build` |
| Build with all features | `cargo build --all-features` |
| Run all tests | `cargo test` |
| Run specific crate tests | `cargo test -p hedl-core` |
| Run benchmarks | `cargo bench` |
| Check formatting | `cargo fmt -- --check` |
| Run linter | `cargo clippy --workspace --all-targets --all-features` |
| Generate docs | `cargo doc --workspace --all-features --no-deps` |

## Key Design Principles

### 1. Zero-Copy Parsing
Where possible, HEDL uses string slices (`&str`) instead of owned strings to minimize allocations.

### 2. Error Handling
All errors use the `thiserror` crate for comprehensive error messages with source locations.

### 3. Type Safety
Extensive use of Rust's type system to prevent invalid states at compile time.

### 4. Performance
Optimized for both small documents (low latency) and large documents (high throughput).

### 5. Modularity
Each crate has a single, well-defined responsibility with minimal dependencies.

## API Stability

HEDL follows semantic versioning:

- **Core API** (`hedl`, `hedl-core`): Stable, breaking changes only in major versions
- **Format converters** (`hedl-json`, `hedl-yaml`, etc.): Stable within major version
- **Tool integrations** (`hedl-lsp`, `hedl-mcp`): May evolve more rapidly
- **Bindings** (`hedl-ffi`, `hedl-wasm`): ABI-stable within major version

## Resources

### Documentation
- [HEDL Specification](../../SPEC.md) - Formal language specification
- [Contributing Guidelines](../../CONTRIBUTING.md) - How to contribute
- [Security Policy](../../SECURITY.md) - Security considerations
- [Changelog](../../CHANGELOG.md) - Version history

### External Links
- [GitHub Repository](https://github.com/dweve-ai/hedl)
- [Issue Tracker](https://github.com/dweve-ai/hedl/issues)
- [Discussions](https://github.com/dweve-ai/hedl/discussions)

### Community
- Report bugs via [GitHub Issues](https://github.com/dweve-ai/hedl/issues)
- Ask questions in [Discussions](https://github.com/dweve-ai/hedl/discussions)
- Contribute via [Pull Requests](https://github.com/dweve-ai/hedl/pulls)

## License

HEDL is licensed under the Apache License 2.0. See [LICENSE](../../LICENSE) for details.

---

**Next Steps**:
- New to HEDL development? Start with the [Getting Started Guide](getting-started.md)
- Want to understand the architecture? Read the [Architecture Overview](architecture.md)
- Ready to contribute? Check the [Contributing Guide](contributing.md)
