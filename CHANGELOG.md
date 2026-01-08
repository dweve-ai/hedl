# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-01-08

### Added

#### Core Framework
- **hedl-core**: Core parsing and serialization engine with deterministic parsing
  - Complete implementation of HEDL v1.0.0 specification
  - **lex**: Consolidated lexical analysis module (tokens, CSV rows, tensors)
  - Schema-defined positional matrices with typed columns
  - Document-wide identity system with global IDs
  - Graph relationship support through reference nodes
  - Implicit child list attachment via nesting rules
  - Scoped ditto operator for value repetition
  - Alias system for global constants and schema sharing
  - Tensor literal support for AI/ML workflows
  - Comprehensive error hierarchy with precise error reporting

#### Format Conversions
- **hedl-json**: Bidirectional JSON conversion
  - HEDL to JSON serialization with metadata preservation
  - JSON to HEDL deserialization with type inference
  - Support for nested structures and references

- **hedl-yaml**: Bidirectional YAML conversion
  - HEDL to YAML serialization
  - YAML to HEDL deserialization
  - Maintains structural fidelity

- **hedl-xml**: Bidirectional XML conversion
  - HEDL to XML serialization with configurable formatting
  - XML to HEDL deserialization
  - Attribute and element handling

- **hedl-csv**: Bidirectional CSV conversion
  - HEDL matrix lists to CSV export
  - CSV to HEDL matrix list import
  - Header row support and type mapping

- **hedl-parquet**: Bidirectional Parquet conversion
  - HEDL to Apache Parquet serialization
  - Parquet to HEDL deserialization
  - Arrow schema integration
  - Columnar storage optimization

- **hedl-toon**: TOON format export
  - HEDL to TOON (Token-Oriented Object Notation) serialization
  - Optimized for LLM context windows

#### Database Integration
- **hedl-neo4j**: Bidirectional Neo4j integration
  - Graph node and relationship extraction
  - Cypher CREATE statement generation
  - Neo4j record import to HEDL documents
  - Support for graph semantics and identity
  - Constraint and index generation

#### Tooling and Quality
- **hedl-c14n**: Canonicalization support
  - Deterministic document formatting
  - Canonical form generation for round-trip stability
  - Ditto optimization for token reduction
  - Whitespace normalization

- **hedl-lint**: Linting and best practices
  - Style consistency checking
  - Best practice enforcement
  - Warning and error reporting
  - JSON and text output formats

#### Developer Tools
- **hedl-cli**: Command-line interface
  - `validate`: HEDL file validation with strict mode
  - `format`: Canonical formatting with ditto optimization
  - `lint`: Best practices linting with configurable output
  - `inspect`: Debug inspection with verbose mode
  - `stats`: Size and token savings analysis
  - `to-json`/`from-json`: JSON conversion commands
  - `to-yaml`/`from-yaml`: YAML conversion commands
  - `to-xml`/`from-xml`: XML conversion commands
  - `to-csv`/`from-csv`: CSV conversion commands
  - `to-parquet`/`from-parquet`: Parquet conversion commands
  - `to-toon`: TOON conversion command

- **hedl-ffi**: Foreign Function Interface bindings
  - C/C++ API bindings
  - Memory-safe FFI layer
  - Cross-language integration support

- **hedl-wasm**: WebAssembly bindings
  - Browser and Node.js support
  - TypeScript definitions

- **hedl-lsp**: Language Server Protocol implementation
  - Syntax highlighting
  - Auto-completion
  - Diagnostics and validation
  - Go-to definition and find references

- **hedl-mcp**: Model Context Protocol server
  - AI/LLM integration
  - File reading and querying
  - Validation and optimization tools

- **hedl-stream**: Streaming parser
  - Process files larger than memory
  - Sync and async APIs
  - Event-based parsing

- **hedl-test**: Testing utilities
  - Conformance test suite
  - Test helpers and fixtures
  - Property-based testing support

#### Documentation
- **hedl**: Main library crate with comprehensive documentation
  - Complete API documentation
  - Usage examples and guides
  - Performance guidelines
  - Migration documentation

- Comprehensive specification (SPEC.md)
  - Formal grammar and parsing algorithms
  - Security considerations
  - Implementation requirements
  - Conformance and interoperability guidelines

- Architecture documentation
  - Component design and interactions
  - Performance characteristics
  - Extension and versioning strategy

- User guides and tutorials
  - Quick start guide
  - Format conversion examples
  - Graph semantics guide
  - Best practices documentation

### Technical Details

#### Language Support
- Minimum Rust version: 1.70
- Edition: 2021
- License: Apache-2.0

#### Performance
- Token-efficient representation optimized for LLM context windows
- Deterministic parsing with fail-fast error handling
- Zero-copy preprocessing with line offset tables
- First-byte dispatch for O(1) type inference
- Byte-based token validation for ASCII-only identifiers
- Efficient schema-based validation

#### Security
- Input validation for all parsers
- Denial-of-service protection
- Truncation detection
- Safe Unicode handling

#### Standards Compliance
- RFC 2119 conformance keywords
- Semantic versioning
- MIME type: `application/hedl`
- File extension: `.hedl`

[1.0.0]: https://github.com/dweve-ai/hedl/releases/tag/v1.0.0
