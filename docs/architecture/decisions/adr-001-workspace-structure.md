# ADR-001: Workspace Structure

**Status**: Accepted

**Date**: 2025-01-06

**Context**: Initial architecture decision

---

## Context

HEDL requires multiple components for parsing, format conversion, tooling, and language bindings. We need to decide how to organize the codebase: monolithic single crate vs. modular workspace structure.

## Decision Drivers

1. **Modularity**: Ability to use components independently
2. **Compilation Time**: Parallel compilation and incremental builds
3. **Dependency Management**: Optional features and minimal dependencies
4. **Maintenance**: Clear boundaries and responsibilities
5. **Distribution**: Selective publishing of crates

## Considered Options

### Option 1: Monolithic Single Crate

**Structure**:
```
hedl/
  src/
    core/
    json/
    yaml/
    xml/
    ...
```

**Pros**:
- Simple dependency management
- Single version number
- Easy cross-module refactoring
- Single point of documentation

**Cons**:
- All features compiled together
- Longer compilation times
- Large dependency tree even for simple usage
- No selective publishing
- Tight coupling between components

### Option 2: Workspace with Multiple Crates (CHOSEN)

**Structure**:
```
hedl/
  crates/
    hedl-core/       # Core parser
    hedl/            # Public facade
    hedl-json/       # JSON converter
    hedl-yaml/       # YAML converter
    ...
```

**Pros**:
- Modular compilation (parallel builds)
- Clear separation of concerns
- Optional dependencies via features
- Selective publishing
- Smaller binary sizes for minimal usage
- Independent versioning possible (future)

**Cons**:
- More complex workspace management
- Version synchronization overhead
- Cross-crate refactoring more complex
- Multiple Cargo.toml files

### Option 3: Hybrid Approach

**Structure**: Core as separate crate, everything else in hedl crate

**Pros**:
- Balance of modularity and simplicity

**Cons**:
- Still large hedl crate
- Limited modularity benefits

## Decision

**Chosen**: Option 2 - Workspace with Multiple Crates

## Rationale

### Primary Reasons

1. **Optional Dependencies**: Users can include only what they need
   ```toml
   # Minimal: just core parsing and JSON
   hedl = "1.0"

   # All formats
   hedl = { version = "1.0", features = ["all-formats"] }
   ```

2. **Parallel Compilation**: Cargo can compile independent crates in parallel
   - Format crates (json, yaml, xml) compile in parallel
   - Tools (cli, lsp, mcp) compile in parallel
   - 30-50% faster clean builds on multi-core machines

3. **Clear Boundaries**: Each crate has single responsibility
   - `hedl-core`: Parsing only
   - `hedl-json`: JSON conversion only
   - `hedl-cli`: CLI tool only

4. **Selective Publishing**: Can publish subsets
   - Core parser stable → publish `hedl-core`
   - Experimental format → unpublished crate
   - Internal tools → workspace-only

5. **Binary Size**: Smaller binaries for focused usage
   - CLI with only JSON: ~500KB
   - CLI with all formats: ~2MB
   - Users choose their tradeoff

### Supporting Benefits

- **Testing Isolation**: Test each crate independently
- **Documentation Clarity**: Separate docs per crate
- **Dependency Minimization**: Core has minimal deps (3 crates)
- **Future Evolution**: Can split further if needed

## Implementation

### Workspace Organization

```toml
[workspace]
resolver = "2"
members = [
    "crates/hedl-core",      # Core layer
    "crates/hedl",           # Facade
    "crates/hedl-c14n",      # Canonicalization
    "crates/hedl-json",      # Format converters
    "crates/hedl-yaml",
    "crates/hedl-xml",
    "crates/hedl-csv",
    "crates/hedl-toon",
    "crates/hedl-parquet",
    "crates/hedl-neo4j",
    "crates/hedl-cli",       # Tools
    "crates/hedl-lint",
    "crates/hedl-lsp",
    "crates/hedl-mcp",
    "crates/hedl-ffi",       # Bindings
    "crates/hedl-wasm",
    "crates/hedl-stream",    # Support
    "crates/hedl-test",
    "crates/hedl-bench",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
```

### Dependency Pattern

All format crates follow this pattern:
```toml
[package]
name = "hedl-json"
version.workspace = true

[dependencies]
hedl-core.workspace = true
serde_json = "1.0"
```

### Feature Gates in Facade

```toml
# crates/hedl/Cargo.toml
[dependencies]
hedl-core.workspace = true
hedl-json.workspace = true  # Always included

hedl-yaml = { workspace = true, optional = true }
hedl-xml = { workspace = true, optional = true }
hedl-csv = { workspace = true, optional = true }

[features]
default = []
yaml = ["dep:hedl-yaml"]
xml = ["dep:hedl-xml"]
csv = ["dep:hedl-csv"]
all-formats = ["yaml", "xml", "csv", "parquet", "neo4j", "toon"]
```

## Consequences

### Positive

1. **Compilation Performance**: 30-50% faster parallel builds
2. **Binary Size**: 60-80% smaller for focused usage
3. **Modularity**: Clear boundaries, easy to understand
4. **Optional Features**: Users choose what to include
5. **Testing**: Isolated test suites per crate

### Negative

1. **Complexity**: More files and configuration
2. **Versioning**: Must keep versions in sync
3. **Refactoring**: Cross-crate changes more complex
4. **Learning Curve**: New contributors need to understand structure

### Mitigations

1. **Workspace.package**: Share version/edition across crates
2. **Workspace.dependencies**: Share dependency versions
3. **Clear Documentation**: Architecture docs (this ADR)
4. **Consistent Patterns**: All format crates follow same structure

## Alternatives Considered

### Virtual Workspace vs. Root Package

**Decision**: Root package is `hedl` (facade)

**Rationale**:
- Most users want `hedl` crate
- Provides unified API
- Can re-export from sub-crates

### Granularity of Crates

**Decision**: One crate per format, tool, or binding

**Alternatives**:
- Single `hedl-formats` crate → rejected (less modular)
- Per-feature subcrates (e.g., `hedl-json-schema`) → rejected (too granular)

## References

- Cargo Workspace documentation: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Rust API Guidelines on modularity: https://rust-lang.github.io/api-guidelines/

## Review

This ADR should be reviewed if:
- Performance profiling shows workspace overhead
- User feedback indicates confusion about structure
- New organization patterns emerge in Rust ecosystem

---

*Decision made: 2025-01-06*
*Last reviewed: 2026-01-06*
