# Security Practices

Security guidelines for HEDL development and deployment.

---

## Security Hardening

HEDL includes comprehensive security hardening across all crates:

### hedl-mcp

| Feature | Description |
|---------|-------------|
| OAuth2 Authentication | Full OAuth2 flow for MCP server authentication |
| API Key Authentication | Simple API key auth for internal services |
| Resource Limits | Configurable limits to prevent resource exhaustion |
| Path Traversal Prevention | Secured `resources/read` endpoint against path traversal attacks |

### hedl-neo4j

| Feature | Description |
|---------|-------------|
| Cypher Injection Prevention | Comprehensive input sanitization and parameterized queries |
| Max Nodes Enforcement | Resource limits properly enforced to prevent DoS attacks |
| Reference ID Validation | Validation for reference IDs before Cypher generation |

### hedl-xml

| Feature | Description |
|---------|-------------|
| Entity Injection Protection | Secured against XML entity injection attacks |
| XXE Prevention | Disabled external entity processing by default |

### hedl-csv

| Feature | Description |
|---------|-------------|
| DoS Protection | Added limits for row/column counts and field sizes |

### hedl-stream

| Feature | Description |
|---------|-------------|
| Line Length Limits | Enforced maximum line length to prevent memory exhaustion |

### hedl-core

| Feature | Description |
|---------|-------------|
| Unsafe Code Audit | Comprehensive review and documentation of all unsafe code |
| Max Total IDs Limit | Added security limit for total IDs in document |

### hedl-ffi

| Feature | Description |
|---------|-------------|
| Thread Safety Audit | Verified and documented thread safety guarantees |
| Null Pointer Safety | Added comprehensive null pointer validation |

### hedl-bench

| Feature | Description |
|---------|-------------|
| Path Traversal Prevention | Secured baseline loader against path traversal |

---

## Security Limits

HEDL enforces 12 security limits to prevent DoS attacks:

```rust
pub struct Limits {
    pub max_file_size: usize,         // 1 GB default
    pub max_line_length: usize,       // 1 MB default
    pub max_indent_depth: usize,      // 50 default
    pub max_nodes: usize,             // 10 million default
    pub max_aliases: usize,           // 10,000 default
    pub max_columns: usize,           // 100 default
    pub max_nest_depth: usize,        // 100 default
    pub max_block_string_size: usize, // 10 MB default
    pub max_object_keys: usize,       // 10,000 per object
    pub max_total_keys: usize,        // 10 million total
    pub max_total_ids: usize,         // 10 million total
    pub timeout: Option<Duration>,    // 30 seconds default
}
```

---

## Dependency Auditing

```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit

# Fix vulnerabilities
cargo update
```

**CI Integration**: Now automated via `.github/workflows/ci.yml` (security job).

---

## Fuzz Testing

HEDL has 15 fuzz targets across 4 crates.

```bash
# Run specific fuzz target
cd crates/hedl-core
cargo +nightly fuzz run fuzz_parse -- -max_total_time=300

# Run all targets
for target in $(cargo +nightly fuzz list); do
    cargo +nightly fuzz run $target -- -max_total_time=300
done
```

See fuzz README files in each crate for complete documentation:
- `crates/hedl-core/fuzz/README.md`
- `crates/hedl-cli/fuzz/README.md`
- `crates/hedl-stream/fuzz/README.md`
- `crates/hedl-csv/fuzz/README.md`

---

## SPEC Conformance

Conformance tests verify SPEC § 14 security requirements:

```bash
cargo test --package hedl-core conformance
```

See: [Conformance Guide](../testing-conformance.md) for details.

---

## Security Metrics

Run these commands to check current security metrics:

```bash
# Count files with unsafe code
grep -rln "unsafe" crates --include="*.rs" | wc -l

# Check for CVEs in dependencies
cargo audit

# Run conformance tests
cargo test --package hedl-core --test conformance_tests
```

---

## Security Disclosures

**IMPORTANT**: Report vulnerabilities privately.

- **Email**: security@dweve.com (private)
- **Do NOT**: Create public GitHub issues for vulnerabilities

---

## Ongoing Security Work

1. **Timeout Implementation**
   - Timeout field exists in Limits struct
   - Parser checks elapsed time during long operations
   - Requires additional coverage testing

2. **CI/CD Security Pipeline** (ACTIVE)
   - Automated testing on every commit
   - Security audit via `cargo audit` in CI
   - Dependency scanning
   - Coverage tracking

3. **Continuous Monitoring**
   - Regular dependency audits
   - Fuzz testing campaigns
   - Security metric tracking

---

## Related

- [Fuzz Testing](../testing.md#fuzz-testing)
- [Input Validation](../concepts/error-handling.md)
- [CHANGELOG v1.2.0 Security](../../../CHANGELOG.md) for complete security fixes
