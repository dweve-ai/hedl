# System Design Documentation

> Comprehensive system design documentation for HEDL architecture

## Overview

This section covers the high-level system design patterns and architectural decisions that shape the HEDL ecosystem. The design follows clean architecture principles with clear separation of concerns and modular composition.

## Table of Contents

### Core Design Patterns
- [Layered Architecture](layered-architecture.md) - Multi-tier separation of concerns
- [Module Structure](module-structure.md) - Workspace organization and crate boundaries
- [Dependency Injection](dependency-injection.md) - Configuration and trait-based abstractions
- [Plugin Architecture](plugin-architecture.md) - Extension points and format adapters

## Design Principles

### 1. Layered Architecture

HEDL follows a strict layered design where dependencies flow inward:

```
┌─────────────────────────────────────┐
│   Application Layer (CLI, LSP)     │
├─────────────────────────────────────┤
│   Bindings Layer (FFI, WASM)       │
├─────────────────────────────────────┤
│   Format Layer (JSON, YAML, etc)   │
├─────────────────────────────────────┤
│   Core Layer (Parser, AST)         │
└─────────────────────────────────────┘
```

Each layer:
- Depends only on layers below it
- Provides stable abstractions to layers above
- Can be tested independently

### 2. Single Responsibility Principle

Each crate has one clear purpose:
- **hedl-core**: Parsing and AST representation
- **hedl-json**: JSON conversion (and only JSON)
- **hedl-cli**: Command-line interface
- **hedl-lsp**: Language server protocol

This enables:
- Independent evolution
- Clear ownership
- Focused testing
- Optional dependencies

### 3. Dependency Inversion

Higher-level modules depend on the core abstractions provided by `hedl-core`:

```rust
// Format crates depend on hedl-core's data model
use hedl_core::{Document, Item, Value};

pub fn to_format(doc: &Document, config: &Config) -> Result<String, Error> {
    // Implementation uses Document visitor or direct traversal
}
```

### 4. Open/Closed Principle

The system is:
- **Open for extension**: New format adapters can be added as separate crates
- **Closed for modification**: Core parser and data model are stable

This enables:
- Third-party format adapters
- Custom tooling integration
- Independent evolution of format support

## Key Design Patterns

### Facade Pattern

The `hedl` crate acts as a unified facade:

```rust
// Users interact with simple facade
use hedl::{parse, to_json, canonicalize};

// Facade delegates to specialized crates
pub fn parse(input: &str) -> Result<Document, HedlError> {
    hedl_core::parse(input.as_bytes())
}

pub fn to_json(doc: &Document) -> Result<String, HedlError> {
    hedl_json::to_json(doc, &ToJsonConfig::default())
        .map_err(|e| HedlError::syntax(format!("JSON conversion error: {}", e), 0))
}
```

**Benefits**:
- Simple API for common cases
- Hides implementation complexity
- Easy to learn and use
- Can evolve independently

### Adapter Pattern

Format converters use the adapter pattern:

```rust
// Each format adapts Document to format-specific representation
pub struct JsonAdapter;
impl JsonAdapter {
    pub fn to_json(&self, doc: &Document) -> Result<String> {
        // Convert Document AST to JSON AST
        // Serialize JSON AST to string
    }
}
```

### Builder Pattern

Configuration uses the builder pattern:

```rust
let config = ToJsonConfig::builder()
    .indent(4)
    .sort_keys(true)
    .schema_generation(true)
    .build();
```

### Strategy Pattern

Validation and linting use strategies:

```rust
pub trait LintRule {
    fn check(&self, doc: &Document) -> Vec<Diagnostic>;
}

pub struct LintEngine {
    rules: Vec<Box<dyn LintRule>>,
}
```

## Architectural Constraints

### Performance Constraints

1. **Efficient Parsing**: Minimize allocations during parsing (owned `String` values)
2. **Arena Allocation**: Single allocation for AST nodes
3. **SIMD Optimization**: Use `memchr` for byte searching
4. **Streaming Support**: Handle files larger than memory

### Security Constraints

1. **Resource Limits**: All parsers enforce configurable limits
2. **Safe Rust**: Minimize unsafe code, document invariants
3. **Input Validation**: Validate all external input
4. **DoS Protection**: Prevent resource exhaustion attacks

### Maintainability Constraints

1. **Single Responsibility**: Each crate has one clear purpose
2. **Comprehensive Testing**: Unit, integration, property, fuzz tests
3. **Documentation**: All public APIs documented with examples
4. **Consistent Patterns**: Similar crates follow same structure

## Cross-Cutting Concerns

### Error Handling

All crates use consistent error handling:

```rust
pub type HedlResult<T> = Result<T, HedlError>;

pub struct HedlError {
    pub kind: HedlErrorKind,
    pub message: String,
    pub line: usize,
    pub column: Option<usize>,
    pub context: Option<String>,
}
```

### Logging and Diagnostics

Structured logging for observability:

```rust
use tracing::{info, warn, error, debug};

debug!("Parsing document of size {}", input.len());
info!("Successfully parsed {} nodes", doc.nodes().len());
warn!("Large document detected: {} KB", size_kb);
error!("Parse failed: {}", err);
```

### Configuration Management

Hierarchical configuration:

```rust
// System defaults
const DEFAULT_MAX_FILE_SIZE: usize = 1_000_000_000; // 1GB

// User configuration
pub struct ParseOptions {
    pub max_file_size: usize,
    pub max_total_keys: usize,
    // ...
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            max_total_keys: 10_000_000,
        }
    }
}
```

## System Boundaries

### Internal Boundaries

- **Core ↔ Formats**: Stable AST representation
- **Core ↔ Tools**: Public API only
- **Formats ↔ Tools**: Optional feature gates

### External Boundaries

- **FFI Boundary**: C-compatible ABI
- **WASM Boundary**: JavaScript interop
- **Network Boundary**: MCP protocol (JSON-RPC)

## Design Trade-offs

### Modularity vs. Complexity

**Decision**: Favor modularity
- **Pro**: Clear boundaries, optional dependencies, parallel builds
- **Con**: More files, complex workspace management
- **Mitigation**: Consistent patterns, comprehensive docs

### Performance vs. Safety

**Decision**: Safety first, optimize hot paths
- **Pro**: Memory safe, no undefined behavior
- **Con**: Some performance overhead vs. C
- **Mitigation**: SIMD, arena allocation, zero-copy where safe

### Flexibility vs. Simplicity

**Decision**: Simple core, flexible extensions
- **Pro**: Easy to learn, stable core, extensible
- **Con**: More abstraction layers
- **Mitigation**: Facade pattern, clear examples

## Related Documentation

- [Layered Architecture](layered-architecture.md) - Detailed layer design
- [Module Structure](module-structure.md) - Crate organization
- [Dependency Injection](dependency-injection.md) - DI patterns
- [Plugin Architecture](plugin-architecture.md) - Extension system
- [ADR-001: Workspace Structure](../decisions/adr-001-workspace-structure.md) - Rationale

## Future Directions

### Planned Enhancements

1. **Async Core**: Full async/await support throughout
2. **Incremental Parsing**: Fine-grained updates for LSP
3. **Query DSL**: SQL-like queries over HEDL documents
4. **Network Protocol**: Distributed HEDL streaming

### Under Consideration

1. **Distributed Consensus**: Raft-based HEDL replication
2. **Query Optimization**: Index-based query planning
3. **Schema Evolution**: Version migration support
4. **Transactional Updates**: ACID guarantees for mutations

---

*Last updated: 2026-01-06*
