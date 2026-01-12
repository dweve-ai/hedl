# Security Practices

Security guidelines for HEDL development and deployment.

---

## Security Limits

HEDL enforces 11 security limits to prevent DoS attacks:

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
    pub timeout: Option<Duration>,    // 30 seconds default (TODO: implement checking)
}
```

**Note**: Timeout field exists but parser implementation is TODO. Track at: TIMEOUT-IMPLEMENTATION issue.

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

See: [Fuzz Testing Guide](../testing-fuzz.md) for complete documentation.

---

## SPEC Conformance

64 conformance tests verify SPEC § 14 security requirements:

```bash
cargo test --package hedl-core conformance
```

See: [Conformance Guide](../testing-conformance.md) for details.

---

## Security Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Unsafe Code | 5.6% (20/359 files) | ✅ Excellent |
| Open CVEs | 0 | ✅ Pass |
| Fuzz Crashes | 0 open | ✅ Pass |
| SPEC Compliance | 100% (64/64 tests) | ✅ Pass |

---

## Security Disclosures

**IMPORTANT**: Report vulnerabilities privately.

- **Email**: security@dweve.com (private)
- **Do NOT**: Create public GitHub issues for vulnerabilities

---

## TODO: Critical Security Work

1. **Implement timeout checking in parser** (Issue: TIMEOUT-IMPLEMENTATION)
   - Timeout field exists in Limits struct
   - Parser implementation needs to check elapsed time
   - Requires tests and validation

2. **Enable CI/CD** (READY - `.github/workflows/ci.yml` created)
   - Automated testing on every commit
   - Security audit in CI
   - Coverage tracking

---

## Related

- [Fuzz Testing](../testing.md#fuzz-testing)
- [Input Validation](../concepts/error-handling.md)
