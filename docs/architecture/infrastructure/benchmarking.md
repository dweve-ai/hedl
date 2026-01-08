# Benchmarking Infrastructure

> Performance measurement and regression tracking

## Overview

HEDL uses Criterion for benchmarking with comprehensive coverage of core functionality, format conversion, and integration scenarios.

## Benchmark Structure

```
hedl-bench/
├── benches/
│   ├── bindings/
│   │   ├── ffi.rs
│   │   ├── mod.rs
│   │   └── wasm.rs
│   ├── core/
│   │   ├── lexer.rs
│   │   ├── parsing.rs
│   │   └── validation.rs
│   ├── features/
│   │   ├── canonicalization.rs
│   │   ├── nesting.rs
│   │   ├── references.rs
│   │   ├── rows.rs
│   │   ├── streaming.rs
│   │   ├── tensors.rs
│   │   └── zero_copy.rs
│   ├── formats/
│   │   ├── csv.rs
│   │   ├── json.rs
│   │   ├── mod.rs
│   │   ├── neo4j_cypher.rs
│   │   ├── parquet.rs
│   │   ├── roundtrip.rs
│   │   ├── toon.rs
│   │   ├── xml.rs
│   │   └── yaml.rs
│   ├── integration/
│   │   ├── comprehensive.rs
│   │   ├── end_to_end.rs
│   │   └── parallel.rs
│   ├── regression/
│   │   └── tracking.rs
│   └── tools/
│       ├── linting.rs
│       ├── lsp.rs
│       ├── mcp.rs
│       └── mod.rs
└── src/
    ├── bin/
    │   └── accuracy.rs
    ├── core/
    ├── datasets.rs
    ├── error.rs
    ├── fixtures/
    ├── generators/
    ├── harness/
    ├── helpers/
    ├── legacy/
    ├── lib.rs
    ├── report.rs
    ├── reporters/
    └── token_counter.rs
```

## Benchmark Example

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hedl_core::parse;

fn benchmark_parse(c: &mut Criterion) {
    c.bench_function("parse_small", |b| {
        let input = "%VERSION: 1.0\n---\nkey: value";
        b.iter(|| {
            parse(black_box(input.as_bytes())).unwrap()
        });
    });

    c.bench_function("parse_large", |b| {
        let input = generate_large_input(10000);
        b.iter(|| {
            parse(black_box(input.as_bytes())).unwrap()
        });
    });
}

fn generate_large_input(size: usize) -> String {
    let mut result = "%VERSION: 1.0\n---\n".to_string();
    for i in 0..size {
        result.push_str(&format!("key{}: value{}\n", i, i));
    }
    result
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
```

## Performance Metrics

### Tracked Metrics

1. **Throughput**: Operations per second
2. **Latency**: Time per operation
3. **Memory**: Allocations and peak usage
4. **Regression**: Comparison to baseline

### Reporting

```bash
# Generate HTML reports
cargo bench --workspace

# View reports
open target/criterion/report/index.html
```

## Baseline Management

```bash
# Save current performance as baseline
cargo bench -- --save-baseline current

# Compare against baseline
cargo bench -- --baseline current
```

---

*Last updated: 2026-01-06*
