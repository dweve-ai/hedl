# Monitoring and Metrics

Track performance and quality metrics for HEDL.

## Performance Metrics

### Benchmark Tracking

```bash
# Save baseline
cargo bench --all -- --save-baseline main

# Compare after changes
cargo bench --all -- --baseline main
```

### Key Metrics

- **Parse throughput**: MB/s for different document sizes
- **Memory usage**: Peak allocation per document
- **Latency**: p50, p95, p99 parse times

## Quality Metrics

### Code Coverage

```bash
cargo tarpaulin --all --out Html
```

Target: >90% line coverage

### Clippy Warnings

```bash
cargo clippy --all -- -D warnings
```

Target: Zero warnings

## Error Tracking

Monitor error rates in:
- Unit tests
- Integration tests
- Fuzz tests
- User reports

## Related

- [Benchmarking Guide](../benchmarking.md)
- [Testing Guide](../testing.md)
