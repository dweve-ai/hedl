# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2026-01-20

### Breaking Changes

- **hedl-core**: `resolve_references()` now accepts `ReferenceMode` enum instead of `bool`
  - `ParseOptions::strict_refs` renamed to `reference_mode`
  - Migration: `strict_refs: true` → `reference_mode: ReferenceMode::Strict`
  - Migration: `strict_refs: false` → `reference_mode: ReferenceMode::Lenient`

### Security

- **Authentication & Authorization** (hedl-mcp): OAuth2, API key, JWT authentication; role-based access control
- **Injection Prevention**: Cypher injection protection (hedl-neo4j), XXE/entity injection prevention (hedl-xml)
- **DoS Protection**: Resource limits across hedl-mcp, hedl-csv, hedl-stream, hedl-core, hedl-bench
- **Memory Safety** (hedl-ffi): Thread safety audit, null pointer validation, memory leak fixes
- **Unsafe Code Audit** (hedl-core): Comprehensive review and documentation of all unsafe blocks

### Fixed

- **Integer Overflow**: hedl-json number parsing, hedl-neo4j batch size calculation
- **Memory Management** (hedl-ffi): Leak detection, const correctness, null pointer handling
- **Panic Handling** (hedl-wasm): Proper error conversion from Rust panics
- **XML Handling**: Attribute validation, namespace preservation (SOAP/RSS/SVG), whitespace normalization
- **YAML Handling**: Duplicate key detection, empty documents, anchor/alias resolution
- **Streaming** (hedl-stream): Async cancellation cleanup, backpressure, incomplete EOF reads
- **Canonicalization** (hedl-c14n): Namespace handling, deterministic attribute ordering
- **Misc**: hedl-parquet null ID corruption, hedl-csv quote escaping, hedl-lint severity escalation

### Performance

- **hedl-core**: Arena allocation (bumpalo), SIMD scanning (memchr, 4-20x faster), parallel parsing (rayon, 2-4x throughput)
- **hedl-neo4j**: Async operations, adaptive batch sizing, transaction batching, result streaming
- **hedl-parquet**: Column pruning, projection/predicate pushdown, async I/O, dictionary encoding
- **hedl-json**: Array optimization, streaming parser
- **hedl-yaml**: Alias optimization, faster serialization
- **hedl-xml**: Streaming parser for large files
- **hedl-wasm**: Custom optimization passes, reduced bundle size, faster startup
- **hedl-lint**: Single-pass traversal, optimized rule checks

### Changed

- **hedl-core**: Removed unused Header fields, deduplicated timeout checks, improved ParseContext
- **hedl-neo4j**: Replaced magic strings with constants, extracted common UNWIND/sanitization logic, split to_cypher module
- **hedl-cli**: Configurable batch file limit, fixed thread pool config, deduplicated completion code
- **hedl-lint**: Removed unused DiagnosticKind variants, improved visitor API
- **hedl-yaml**: Improved error messages with source locations
- **hedl-lsp**: Added rename refactoring support

### Added

#### hedl-core
- **Validation Framework**: `Rule` trait, `Diagnostic` with metadata/source locations/auto-fix suggestions, `RuleRegistry`, `ValidationRunner`
  - Built-in rules: DuplicateKeyRule, InvalidReferenceRule, TypeMismatchRule, UnusedReferenceRule
  - Severity levels: Error, Warning, Info, Hint
- **Visitor Pattern API**: `Visitor`, `VisitorMut`, `Transformer`, `FallibleVisitor` traits
  - `VisitDecision` enum, `TraversalConfig` (pre/post-order, DFS/BFS), `VisitorContext`
  - Utility visitors: DepthCounter, FindNode, NodeCollector, PathCollector, ReferenceCollector, TypeCounter
- **Arena Allocation**: String interner, arena-backed vectors for reduced allocation overhead
- **Parallel Parsing**: Rayon-based multi-threaded parsing with configurable parallelism
- **Type System**: Type coercion module, schema versioning with migration support
- **Enhanced Lexer**: Directive/expression/row parsing improvements, span tracking

#### hedl-mcp
- **Authentication System**: OAuth2, API key, JWT, session management, RBAC, crypto (Argon2, HMAC)
- **Batch Processing**: Parallel execution, progress tracking, cancellation, error aggregation
- **Resource Limits**: Memory limits, rate limiting, connection pooling

#### hedl-neo4j
- **Async Client**: Non-blocking operations with tokio
- **Batch Operations**: Configurable sizes, transaction grouping, progress callbacks
- **Security**: Unicode normalization, input validation, safe math utilities
- **Size Estimation**: Memory usage prediction

#### hedl-lint
- **Auto-Fix System**: Fix applicator, conflict detection, diff preview, verification, statistics

#### hedl-parquet
- **Async I/O**: Non-blocking file operations with tokio
- **Configuration Module**: Compression, row group sizing, encoding options
- **Predicate Pushdown**: Filter pushdown for reduced I/O

#### hedl-json
- **String Cache**: String interning for memory efficiency
- **Validation Module**: Schema-aware validation, type checking

#### hedl-yaml
- **Anchor Module**: Anchor tracking, alias resolution, circular reference detection
- **Snippet Module**: Error context snippets
- **YAML Scanner**: Low-level tokenization

#### hedl-xml
- **Security Module**: XXE prevention, entity expansion limits

#### hedl-stream
- **Buffer Configuration**: Tunable buffer parameters and pooling
- **Compression**: gzip, zstd support

#### hedl-ffi
- **Async Operations**: Callback-based async API
- **Reentrancy Protection**: Deadlock prevention, recursive call detection

#### hedl-cli
- **File Discovery**: Glob pattern matching, recursive traversal

#### hedl-lsp
- **Rename Support**: Symbol renaming across files with preview

#### Fuzz Targets
- hedl-core: fuzz_parse, fuzz_limits, fuzz_nest_depth, fuzz_references
- hedl-cli: fuzz_format

#### Examples (16 new)
- hedl-core: custom_limits
- hedl-json: array_performance, streaming_demo, test_unicode_escape
- hedl-mcp: batch_operations
- hedl-neo4j: constants_usage, streaming_example
- hedl-stream: async_batch_processing, async_cancellation, async_concurrent_files, async_stream_trait
- hedl-csv: csv_conversion
- hedl-toon: pluralization_demo
- hedl-xml: basic_conversion
- hedl-yaml: benchmark_serialization
- hedl-bench: regression_check

### Language Bindings

- **C**: New `HEDL_ERR_NEO4J` error code; fixed example memory management
- **C#**: Parquet overflow protection (>2GB); improved error messages; NUL byte documentation
- **Go**: `contentToFFI()` for embedded NUL handling; Parquet overflow protection; UTF-8 length fix
- **Node.js**: `Buffer.byteLength()` for UTF-8; BigInt overflow checks; improved error messages
- **PHP/Python/Ruby**: Convenience methods and error handling improvements

### Testing

- Comprehensive test coverage across all 19 crates
- Test categories: validation rules, async operations, security, edge cases, roundtrips, property-based tests, fuzz tests

### Documentation

- **Formal Specification**: docs/spec/syntax.md (EBNF grammar), docs/spec/semantics.md (type system, references)
- **Unsafe Code Guidelines**: Safety invariants, review checklist, FFI best practices
- **README Verification**: All 19 crate READMEs verified; fixed version mismatches in hedl-xml, hedl-mcp, hedl-ffi, hedl-csv, hedl-lint, hedl-parquet, hedl-bench, hedl-cli
- **Link Fixes**: Fixed 4 broken internal documentation links

### Infrastructure

- **CI/CD**: GitHub Actions workflows (ci.yml, scheduled.yml)
- **Security**: Added .cargo/audit.toml
- **Cleanup**: Consolidated deprecated files to .deprecated/; removed obsolete configs

## [1.1.0] - 2026-01-10

### Changed
- Fixed inter-crate dependency issues for workspace builds
- Added README.md files to all crates

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

[1.2.0]: https://github.com/dweve-ai/hedl/releases/tag/v1.2.0
[1.1.0]: https://github.com/dweve-ai/hedl/releases/tag/v1.1.0
[1.0.0]: https://github.com/dweve-ai/hedl/releases/tag/v1.0.0
