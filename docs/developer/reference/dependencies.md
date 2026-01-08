# Dependency Reference

Complete dependency information for HEDL.

## Core Dependencies

### hedl-core

```toml
thiserror = "1.0"    # Error types
memchr = "2.7"       # SIMD string scanning
bumpalo = "3.16"     # Arena allocation
serde = { version = "1.0", optional = true }
```

### hedl-json

```toml
hedl-core = { workspace = true }
serde_json = "1.0"
thiserror = "1.0"
```

### hedl-yaml

```toml
hedl-core = { workspace = true }
serde_yaml = "0.9"
```

## Dependency Graph

```
hedl (facade)
├── hedl-core (core)
├── hedl-json
│   └── hedl-core
├── hedl-yaml
│   └── hedl-core
└── ...
```

## Version Policy

- **Major updates**: Reviewed for breaking changes
- **Minor updates**: Auto-update via Dependabot
- **Patch updates**: Auto-merge if CI passes

## Security Audits

```bash
cargo audit
cargo outdated
```

Run on every PR.

## Related

- [Module Guide](../module-guide.md)
- [Security Practices](../operations/security.md)
