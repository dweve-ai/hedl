# How-To: Add Benchmarks

Create performance benchmarks for HEDL code.

## Goal

Add criterion benchmarks to track performance and detect regressions.

## Quick Start

```bash
# Run existing benchmarks
cargo bench -p hedl-bench

# Add new benchmark file
touch crates/hedl-bench/benches/my_feature.rs
```

## Structure

```
crates/hedl-bench/
├── benches/
│   ├── core/           # Core functionality
│   ├── formats/        # Format converters
│   ├── features/       # Specific features
│   └── integration/    # End-to-end
├── src/
│   ├── common/         # Shared utilities
│   └── generators.rs   # Test data generation
└── Cargo.toml
```

## Simple Benchmark

File: `crates/hedl-bench/benches/my_feature.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hedl_core::parse;

fn benchmark_simple_parse(c: &mut Criterion) {
    let input = b"%VERSION: 1.0\n---\nname: Alice\nage: 30";

    c.bench_function("parse_simple", |b| {
        b.iter(|| {
            parse(black_box(input)).unwrap()
        });
    });
}

criterion_group!(benches, benchmark_simple_parse);
criterion_main!(benches);
```

Run it:
```bash
cargo bench --bench my_feature
```

## Parametric Benchmarks

Compare performance across different inputs:

```rust
use criterion::{BenchmarkId, Criterion};

fn benchmark_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_by_size");

    for size in [10, 100, 1000, 10000].iter() {
        let input = generate_document(*size);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &input,
            |b, input| {
                b.iter(|| parse(black_box(input.as_bytes())));
            },
        );
    }

    group.finish();
}

fn generate_document(lines: usize) -> String {
    (0..lines)
        .map(|i| format!("key{}: value{}", i, i))
        .collect::<Vec<_>>()
        .join("\n")
}
```

## Comparison Benchmarks

Compare multiple implementations:

```rust
fn benchmark_string_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_parsing");

    let input = "  value  ";

    group.bench_function("trim_clone", |b| {
        b.iter(|| black_box(input.trim().to_string()));
    });

    group.bench_function("trim_borrow", |b| {
        b.iter(|| black_box(input.trim()));
    });

    group.finish();
}
```

## Complete Example

File: `crates/hedl-bench/benches/formats/json.rs` (example - TOML converter not in workspace)

```rust
use criterion::{
    black_box, criterion_group, criterion_main,
    BenchmarkId, Criterion, Throughput,
};
use hedl_core::parse;
use hedl_json::{to_json, from_json};

fn benchmark_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_conversion");

    // Different document complexities
    let test_cases = vec![
        ("simple", r#"%VERSION: 1.0
---
name: Alice
age: 30"#),
        ("nested", r#"%VERSION: 1.0
---
server:
  host: localhost
  database:
    url: postgres://localhost"#),
        ("large", generate_large_document(1000)),
    ];

    for (name, hedl) in test_cases {
        let doc = parse(hedl.as_bytes()).unwrap();

        // Benchmark HEDL → JSON
        group.bench_with_input(
            BenchmarkId::new("to_json", name),
            &doc,
            |b, doc| {
                b.iter(|| to_json(black_box(doc), &Default::default()));
            },
        );

        // Benchmark JSON → HEDL
        let json_str = to_json(&doc, &Default::default()).unwrap();
        group.throughput(Throughput::Bytes(json_str.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("from_json", name),
            &json_str,
            |b, json| {
                b.iter(|| from_json(black_box(json), &Default::default()));
            },
        );

        // Benchmark round-trip
        group.bench_with_input(
            BenchmarkId::new("round_trip", name),
            &doc,
            |b, doc| {
                b.iter(|| {
                    let json = to_json(black_box(doc), &Default::default()).unwrap();
                    from_json(&json, &Default::default()).unwrap()
                });
            },
        );
    }

    group.finish();
}

fn generate_large_document(nodes: usize) -> String {
    let mut hedl = String::from("%VERSION: 1.0\n---\n");
    for i in 0..nodes {
        hedl.push_str(&format!("node{}:\n  value: {}\n", i, i));
    }
    hedl
}

criterion_group!(benches, benchmark_conversion);
criterion_main!(benches);
```

## Custom Report Generation

See existing benchmarks in `crates/hedl-bench/benches/`

```rust
use std::fs;

fn save_report(results: &BenchmarkResults, path: &str) {
    let report = generate_markdown_report(results);
    fs::write(path, report).unwrap();
}

fn generate_markdown_report(results: &BenchmarkResults) -> String {
    format!(r#"# Performance Report

## Summary

| Benchmark | Time | Throughput |
|-----------|------|------------|
{}

## Details

{}
"#,
        results.table_rows(),
        results.detailed_analysis()
    )
}
```

## Related

- [Profile Performance](profile-performance.md)
- [How-To Index](README.md)
