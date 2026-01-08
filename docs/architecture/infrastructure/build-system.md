# Build System

> Cargo workspace configuration and build optimization

## Overview

HEDL uses a Cargo workspace with 19 crates, optimized for parallel compilation and modular development.

## Workspace Structure

```toml
[workspace]
resolver = "2"
members = [
    "crates/hedl",
    "crates/hedl-core",
    "crates/hedl-c14n",
    "crates/hedl-json",
    "crates/hedl-yaml",
    "crates/hedl-xml",
    "crates/hedl-csv",
    "crates/hedl-toon",
    "crates/hedl-parquet",
    "crates/hedl-neo4j",
    "crates/hedl-lint",
    "crates/hedl-cli",
    "crates/hedl-ffi",
    "crates/hedl-test",
    "crates/hedl-mcp",
    "crates/hedl-lsp",
    "crates/hedl-wasm",
    "crates/hedl-stream",
    "crates/hedl-bench",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
license = "Apache-2.0"
rust-version = "1.70"

[workspace.dependencies]
# Internal dependencies
hedl-core = { version = "1.0.0", path = "crates/hedl-core" }

# External dependencies
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
```

## Build Profiles

### Development Profile

```toml
[profile.dev]
opt-level = 0
debug = true
```

**Characteristics**:
- Fast compilation
- Full debug info
- No optimizations

### Release Profile

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
debug = false
```

**Characteristics**:
- Maximum runtime performance
- Link-time optimization
- Single codegen unit for better inlining

### Benchmark Profile

```toml
[profile.bench]
inherits = "release"
```

**Characteristics**:
- Same as release
- Ensures benchmarks reflect production performance

## Build Optimization

### Parallel Compilation

Cargo automatically parallelizes independent crate builds:

```
Format crates compile in parallel:
  hedl-json ─┐
  hedl-yaml ─┼─> Parallel execution
  hedl-xml ──┘
```

### Incremental Compilation

```toml
[profile.dev]
incremental = true  # Faster rebuilds
```

### Dependency Caching

```bash
# Cache dependencies for CI
cargo fetch  # Download all dependencies
```

## Build Scripts

### Custom Build Configuration

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=src/");

    // Platform-specific configuration
    #[cfg(target_arch = "x86_64")]
    println!("cargo:rustc-cfg=simd_support");
}
```

## Related Documentation

- [Module Dependencies](../module-dependencies.md)
- [Architecture Overview](../README.md)

---

*Last updated: 2026-01-06*
