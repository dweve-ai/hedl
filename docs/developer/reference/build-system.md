# Build System Reference

HEDL's Cargo workspace and build configuration.

## Workspace Structure

```toml
[workspace]
members = [
    "crates/hedl",
    "crates/hedl-core",
    "crates/hedl-json",
    # ... all 19 crates
]

[workspace.package]
version = "1.0.0"
edition = "2021"
license = "Apache-2.0"
repository = "https://github.com/dweve-ai/hedl"
```

## Feature Flags

### hedl (main crate)

```toml
[features]
default = []
json = ["hedl-json"]
yaml = ["hedl-yaml"]
xml = ["hedl-xml"]
csv = ["hedl-csv"]
all-formats = ["yaml", "xml", "csv", "parquet", "neo4j", "toon"]
```

## Build Profiles

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.dev]
opt-level = 0
debug = true
```

## Cross-Compilation

```bash
# Linux → Windows
cargo build --target x86_64-pc-windows-gnu

# macOS → Linux
cargo build --target x86_64-unknown-linux-gnu
```

## Related

- [Getting Started](../getting-started.md)
- [Module Guide](../module-guide.md)
