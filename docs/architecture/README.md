# HEDL Architecture Documentation

> Comprehensive technical documentation for the HEDL ecosystem architecture

## Table of Contents

### Core Architecture
- [Module Dependencies](module-dependencies.md) - Crate dependency graph and relationships
- [Data Flow](data-flow.md) - How data moves through the system
- [Parsing Pipeline](parsing-pipeline.md) - Preprocess → Header → Body → References

### Extension & Integration
- [Format Adapters](format-adapters.md) - Architecture for format conversion
- [Extension Points](extension-points.md) - How to extend HEDL functionality
- [Performance Architecture](performance.md) - Performance design and optimizations

### Architectural Decisions
- [ADR-001: Workspace Structure](decisions/adr-001-workspace-structure.md) - Monorepo organization
- [ADR-002: Error Handling](decisions/adr-002-error-handling.md) - Error propagation strategy

## Quick Navigation

### By Concern
- **Parsing**: [Parsing Pipeline](parsing-pipeline.md), [Components](components/README.md)
- **Formats**: [Format Adapters](format-adapters.md), [Data Flow](data-flow.md)
- **Performance**: [Performance Architecture](performance.md), [Module Dependencies](module-dependencies.md)
- **Extension**: [Extension Points](extension-points.md), [Format Adapters](format-adapters.md)

### By Layer
- **Core Layer**: hedl-core, hedl, hedl-c14n
- **Format Layer**: hedl-json, hedl-yaml, hedl-xml, hedl-csv, hedl-toon, hedl-parquet, hedl-neo4j
- **Tools Layer**: hedl-cli, hedl-lint, hedl-lsp, hedl-mcp
- **Bindings Layer**: hedl-ffi, hedl-wasm
- **Support Layer**: hedl-stream, hedl-test, hedl-bench

## Architecture Principles

HEDL follows clean architecture principles with clear separation of concerns:

1. **Layered Design**: Core → Formats → Tools → Bindings
2. **Dependency Inversion**: Higher layers depend on core abstractions
3. **Single Responsibility**: Each crate has one clear purpose
4. **Efficient Memory Management**: Minimize allocations through internal zero-copy techniques and pre-allocation
5. **Security by Design**: Resource limits enforced at parse time
6. **Modular Format Support**: Optional feature-gated format converters

## Key Design Patterns

### Parser Architecture
- **Zero-Copy Line Splitting**: Efficient preprocessing with borrowed slices
- **Direct Document Construction**: Parse directly to Document structure (no intermediate AST)
- **Streaming Support**: Large file handling via hedl-stream

### Format Conversion
- **Adapter Pattern**: Each format has dedicated adapter crate
- **Bidirectional Conversion**: HEDL ↔ Format with symmetric APIs
- **Schema Preservation**: Type information maintained across conversions

### Error Handling
- **Result-Based**: Comprehensive `Result<T, HedlError>` throughout
- **Context Preservation**: Error chains maintain full context
- **User-Friendly Messages**: Actionable error messages with suggestions

## System Metrics

- **Total Crates**: 19 workspace members
- **Core Crates**: 3 (hedl-core, hedl, hedl-c14n)
- **Format Crates**: 7 (JSON, YAML, XML, CSV, TOON, Parquet, Neo4j)
- **Tool Crates**: 4 (CLI, Lint, LSP, MCP)
- **Binding Crates**: 2 (FFI, WASM)
- **Support Crates**: 3 (Stream, Test, Bench)

## Documentation Standards

All architecture documentation follows these conventions:

- **Mermaid Diagrams**: Visual architecture representations
- **Trade-off Discussions**: Explicit rationale for design decisions
- **Code Examples**: Concrete usage patterns
- **Performance Notes**: Complexity analysis and optimization notes
- **Security Considerations**: Resource limits and safety boundaries

## Related Documentation

- [User Guide](../../README.md) - Getting started with HEDL
- [API Reference](https://docs.rs/hedl) - Complete API documentation
- [Benchmark Reports](../../crates/hedl-bench/target/) - Performance analysis
- [Security Guide](../developer/operations/security.md) - Security considerations and best practices

---

*Architecture documentation maintained by the HEDL team*
*Last updated: 2026-01-06*
