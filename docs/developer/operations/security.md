# Security Practices

Security guidelines for HEDL development.

## Dependency Auditing

```bash
# Install
cargo install cargo-audit

# Run audit
cargo audit

# Fix vulnerabilities
cargo update
```

Run on every PR via CI.

## Input Validation

Always enforce limits:

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
}
```

## Fuzz Testing

```bash
cd crates/hedl-core
cargo fuzz run parse -- -max_len=10000
```

## Security Disclosures

Report vulnerabilities to: security@dweve.ai

Do not create public issues for security vulnerabilities.

## Related

- [Fuzz Testing](../testing.md#fuzz-testing)
- [Input Validation](../concepts/error-handling.md)
