# Benchmarking Guide

Comprehensive guide to performance testing, optimization, and benchmarking in HEDL.

## Table of Contents

1. [Benchmarking Philosophy](#benchmarking-philosophy)
2. [Running Benchmarks](#running-benchmarks)
3. [Benchmark Organization](#benchmark-organization)
4. [Writing Benchmarks](#writing-benchmarks)
5. [Performance Profiling](#performance-profiling)
6. [Optimization Workflow](#optimization-workflow)
7. [Regression Detection](#regression-detection)
8. [Benchmark Results](#benchmark-results)

---

## Benchmarking Philosophy

### Core Principles

1. **Measure First**: Profile before optimizing
2. **Realistic Workloads**: Benchmark real-world scenarios
3. **Statistical Rigor**: Use proper statistical methods
4. **Reproducibility**: Ensure consistent results
5. **Continuous Monitoring**: Track performance over time
6. **Regression Prevention**: Detect performance degradation early

### Performance Goals

| Operation | Target | Measured |
|-----------|--------|----------|
| Parse small document (< 1KB) | < 100 μs | ~50 μs |
| Parse medium document (10-100 KB) | < 10 ms | ~5 ms |
| Parse large document (1-10 MB) | < 1 s | ~500 ms |
| JSON conversion | < 2x parse time | ~1.5x |
| YAML conversion | < 3x parse time | ~2x |
| Canonicalization | < parse time | ~0.5x |

---

## Running Benchmarks

### Basic Commands

```bash
# All benchmarks
cargo bench --all

# Specific crate
cargo bench -p hedl-bench

# Specific benchmark
cargo bench --bench parsing

# With filter
cargo bench parsing -- simple

# Save baseline
cargo bench --bench parsing -- --save-baseline master

# Compare to baseline
cargo bench --bench parsing -- --baseline master

# Generate reports
cargo bench --bench parsing -- --plotting-backend gnuplot
```

### Environment Setup

For consistent results:

```bash
# Disable CPU frequency scaling
sudo cpupower frequency-set --governor performance

# Disable Turbo Boost
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Set CPU affinity
taskset -c 0 cargo bench

# Close other applications
# Ensure stable power supply
# Run multiple times for statistical validity
```

### CI Integration

```yaml
# .github/workflows/benchmarks.yml
name: Benchmarks

on:
  pull_request:
  push:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run benchmarks
        run: cargo bench --all -- --save-baseline pr-${{ github.event.number }}

      - name: Compare to main
        run: cargo bench --all -- --baseline main

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion
```

---

## Benchmark Organization

### Directory Structure

```
hedl-bench/
├── Cargo.toml
├── benches/
│   ├── core/
│   │   ├── lexer.rs           # Lexical analysis
│   │   ├── parsing.rs         # Document parsing
│   │   └── validation.rs      # Validation
│   ├── features/
│   │   ├── canonicalization.rs
│   │   ├── references.rs
│   │   ├── streaming.rs
│   │   └── zero_copy.rs
│   ├── formats/
│   │   ├── json.rs
│   │   ├── yaml.rs
│   │   ├── xml.rs
│   │   ├── csv.rs
│   │   └── parquet.rs
│   ├── bindings/
│   │   ├── ffi.rs
│   │   └── wasm.rs
│   ├── tools/
│   │   ├── linting.rs
│   │   ├── lsp.rs
│   │   └── mcp.rs
│   ├── integration/
│   │   ├── end_to_end.rs
│   │   ├── parallel.rs
│   │   └── comprehensive.rs
│   └── regression/
│       └── tracking.rs
├── baselines/
│   ├── current.json
│   └── main.json
└── target/
    └── criterion/              # Generated reports
```

### Benchmark Categories

**Core Benchmarks**:
- Lexer performance (tokenization)
- Parser performance (AST construction)
- Validation performance

**Format Benchmarks**:
- JSON conversion (to/from)
- YAML conversion
- XML conversion
- CSV conversion

**Feature Benchmarks**:
- Canonicalization
- Reference resolution
- Streaming parsing
- Zero-copy operations

**Scalability Benchmarks**:
- Small documents (< 1 KB)
- Medium documents (1-100 KB)
- Large documents (> 100 KB)
- Extremely large (> 1 MB)

---

## Writing Benchmarks

### Basic Benchmark

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hedl_core::parse;

fn bench_parse_simple(c: &mut Criterion) {
    let input = b"%VERSION: 1.0\n---\nname: Alice\nage: 30";

    c.bench_function("parse_simple", |b| {
        b.iter(|| {
            parse(black_box(input)).unwrap()
        })
    });
}

criterion_group!(benches, bench_parse_simple);
criterion_main!(benches);
```

### Parameterized Benchmarks

```rust
use criterion::{BenchmarkId, Criterion};
use hedl_core::parse;

fn bench_parse_varying_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_varying_size");

    for size in [10, 100, 1000, 10000].iter() {
        let input = generate_document(*size);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &input,
            |b, input| {
                b.iter(|| parse(black_box(input.as_bytes())).unwrap())
            },
        );
    }

    group.finish();
}

fn generate_document(lines: usize) -> String {
    let mut doc = String::from("%VERSION: 1.0\n---\n");
    for i in 0..lines {
        doc.push_str(&format!("key{}: value{}\n", i, i));
    }
    doc
}
```

### Throughput Benchmarks

```rust
use criterion::{BenchmarkId, Throughput, Criterion};
use hedl_core::parse;

fn bench_parse_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_throughput");

    for size in [1024, 10240, 102400].iter() {
        let input = generate_document_bytes(*size);

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &input,
            |b, input| {
                b.iter(|| parse(black_box(input)).unwrap())
            },
        );
    }

    group.finish();
}

fn generate_document_bytes(target_size: usize) -> Vec<u8> {
    let mut doc = String::from("%VERSION: 1.0\n---\n");
    let mut i = 0;
    while doc.len() < target_size {
        doc.push_str(&format!("key{}: value{}\n", i, i));
        i += 1;
    }
    doc.into_bytes()
}
```

### Comparison Benchmarks

```rust
use hedl_core::parse;
use std::fs;

fn bench_parse_vs_serde_json(c: &mut Criterion) {
    let hedl_input = fs::read("bindings/common/fixtures/sample_basic.hedl").unwrap();
    let json_input = fs::read_to_string("bindings/common/fixtures/sample_basic.json")
        .unwrap_or_else(|_| r#"{"name":"Alice","age":30}"#.to_string());

    let mut group = c.benchmark_group("parse_comparison");

    group.bench_function("hedl_parse", |b| {
        b.iter(|| parse(black_box(&hedl_input)).unwrap())
    });

    group.bench_function("json_parse", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(&json_input)).unwrap())
    });

    group.finish();
}
```

### Setup/Teardown

```rust
use criterion::BatchSize;
use hedl_core::parse;

fn bench_with_setup(c: &mut Criterion) {
    c.bench_function("parse_with_setup", |b| {
        b.iter_batched(
            || {
                // Setup: create input
                generate_document(1000).into_bytes()
            },
            |input| {
                // Benchmark: parse
                parse(&input).unwrap()
            },
            BatchSize::SmallInput,
        )
    });
}

fn generate_document(lines: usize) -> String {
    let mut doc = String::from("%VERSION: 1.0\n---\n");
    for i in 0..lines {
        doc.push_str(&format!("key{}: value{}\n", i, i));
    }
    doc
}
```

### Custom Measurement

```rust
use criterion::measurement::WallTime;
use hedl_core::parse;
use std::time::Duration;

fn bench_with_custom_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_time");

    // Increase sample size for more accurate results
    group.sample_size(1000);

    // Set warm-up time
    group.warm_up_time(Duration::from_secs(5));

    // Set measurement time
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("parse", |b| {
        let input = b"%VERSION: 1.0\n---\nname: Alice\nage: 30";
        b.iter(|| parse(black_box(input)).unwrap())
    });

    group.finish();
}
```

---

## Performance Profiling

### CPU Profiling with Flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bench parsing

# Open flamegraph.svg
firefox flamegraph.svg
```

### CPU Profiling with perf

```bash
# Record
cargo build --release --bench parsing
perf record -g target/release/deps/parsing-* --bench

# Report
perf report

# Annotate
perf annotate
```

### Memory Profiling with Valgrind

```bash
# Build
cargo build --release --bench parsing

# Run with massif
valgrind --tool=massif \
    target/release/deps/parsing-* --bench

# Visualize
ms_print massif.out.*
```

### Memory Profiling with Heaptrack

```bash
# Install heaptrack
sudo apt-get install heaptrack

# Profile
heaptrack target/release/deps/parsing-* --bench

# Analyze
heaptrack_gui heaptrack.parsing.*.gz
```

### Profiling with cargo-instruments (macOS)

```bash
# Install
cargo install cargo-instruments

# Time profiler
cargo instruments -t time --bench parsing

# Allocations
cargo instruments -t alloc --bench parsing

# Open in Instruments.app
open target/release/instruments/*.trace
```

---

## Optimization Workflow

### 1. Identify Bottleneck

```bash
# Profile to find hotspot
cargo flamegraph --bench parsing

# Look for:
# - Functions consuming most time
# - Unexpected allocations
# - Inefficient algorithms
```

### 2. Create Baseline

```bash
# Save current performance
cargo bench --bench parsing -- --save-baseline before
```

### 3. Implement Optimization

```rust
// Example: Optimize string allocation
// Before:
fn process(input: &str) -> String {
    let mut result = String::new();
    for line in input.lines() {
        result.push_str(line);  // Many allocations
        result.push('\n');
    }
    result
}

// After:
fn process(input: &str) -> String {
    let mut result = String::with_capacity(input.len());  // Pre-allocate
    for line in input.lines() {
        result.push_str(line);
        result.push('\n');
    }
    result
}
```

### 4. Measure Improvement

```bash
# Compare to baseline
cargo bench --bench parsing -- --baseline before

# Look for:
# - Percentage improvement
# - Statistical significance
# - Regression in other benchmarks
```

### 5. Verify Correctness

```bash
# Ensure tests still pass
cargo test --all

# Check for regressions
cargo bench --all -- --baseline main
```

---

## Regression Detection

### Automatic Regression Detection

```rust
// benches/regression/tracking.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_with_threshold(c: &mut Criterion) {
    c.bench_function("critical_path", |b| {
        let input = load_fixture("large.hedl");

        b.iter(|| {
            let result = parse(black_box(&input)).unwrap();
            black_box(result)
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        // Fail if performance degrades by > 10%
        .noise_threshold(0.10);
    targets = bench_with_threshold
}
criterion_main!(benches);
```

### CI Regression Checks

```bash
#!/bin/bash
# scripts/check_regression.sh

# Run benchmarks
cargo bench --bench parsing -- --save-baseline pr

# Compare to main
cargo bench --bench parsing -- --baseline main

# Extract results
CURRENT=$(jq '.mean.point_estimate' target/criterion/parse/pr/estimates.json)
BASELINE=$(jq '.mean.point_estimate' target/criterion/parse/main/estimates.json)

# Calculate percentage change
CHANGE=$(echo "scale=2; ($CURRENT - $BASELINE) / $BASELINE * 100" | bc)

# Fail if > 10% slower
if (( $(echo "$CHANGE > 10" | bc -l) )); then
    echo "Performance regression detected: ${CHANGE}%"
    exit 1
fi
```

---

## Benchmark Results

### Reading Criterion Output

```
parse_simple            time:   [48.234 µs 48.567 µs 48.912 µs]
                        change: [-2.5234% -1.2345% +0.3456%] (p = 0.13 > 0.05)
                        No change in performance detected.
```

**Interpretation**:
- **time**: Mean execution time with confidence interval
- **change**: Percentage change from baseline
- **p-value**: Statistical significance (< 0.05 = significant)

### Example Benchmark Results

```
Benchmark: parse
  Small (< 1KB)       50 μs     ████████████████████
  Medium (10KB)       5 ms      ████████████████████
  Large (1MB)         500 ms    ████████████████████

Benchmark: convert
  to_json             75 μs     ████████████████████████
  to_yaml             100 μs    ████████████████████████████
  to_xml              120 μs    ████████████████████████████████

Throughput:
  Parse               2 GB/s
  JSON convert        1.5 GB/s
  YAML convert        1 GB/s
```

### Performance Comparison

```
Format       Parse Time    Memory     Output Size
HEDL         50 μs         12 KB      1.0x
JSON         75 μs         18 KB      1.5x
YAML         100 μs        20 KB      2.0x
XML          120 μs        25 KB      3.0x
```

---

## Best Practices

### Benchmark Writing

1. **Use `black_box`**: Prevent compiler optimizations
   ```rust
   b.iter(|| parse(black_box(input)))
   ```

2. **Warm up properly**: Allow JIT compilation
   ```rust
   group.warm_up_time(Duration::from_secs(5));
   ```

3. **Sufficient samples**: Ensure statistical validity
   ```rust
   group.sample_size(1000);
   ```

4. **Realistic inputs**: Use representative data
   ```rust
   let input = load_real_world_fixture("production_data.hedl");
   ```

5. **Isolate benchmarks**: Avoid interference
   ```rust
   b.iter_batched(setup, benchmark, BatchSize::SmallInput)
   ```

### Performance Optimization

1. **Profile first**: Don't guess
2. **Optimize hotspots**: Focus on critical paths
3. **Measure impact**: Verify improvements
4. **Consider trade-offs**: Speed vs. memory vs. complexity
5. **Document optimizations**: Explain non-obvious code

### Continuous Monitoring

1. **Track over time**: Monitor trends
2. **Set thresholds**: Define acceptable degradation
3. **Automate checks**: CI integration
4. **Review regularly**: Scheduled performance reviews
5. **Document baselines**: Record expected performance

---

## Common Optimizations

### Reduce Allocations

```rust
// Before: Many allocations
fn parse_values(input: &str) -> Vec<Value> {
    input.split(',')
        .map(|s| parse_value(&s.trim().to_string()))  // Allocation
        .collect()
}

// After: Minimize allocations
fn parse_values(input: &str) -> Vec<Value> {
    input.split(',')
        .map(|s| parse_value(s.trim()))  // No allocation
        .collect()
}
```

### Pre-allocate

```rust
// Before: Incremental growth
let mut result = Vec::new();
for item in items {
    result.push(process(item));
}

// After: Pre-allocate
let mut result = Vec::with_capacity(items.len());
for item in items {
    result.push(process(item));
}
```

### Use String Slices

```rust
// Before: Owned strings
fn extract_key(line: String) -> String {
    line.split(':').next().unwrap().to_string()
}

// After: Borrowed slices
fn extract_key(line: &str) -> &str {
    line.split(':').next().unwrap()
}
```

### Cache Results

```rust
use std::collections::HashMap;

struct Parser {
    schema_cache: HashMap<String, Arc<Schema>>,
}

impl Parser {
    fn get_schema(&mut self, name: &str) -> Arc<Schema> {
        self.schema_cache.entry(name.to_string())
            .or_insert_with(|| Arc::new(load_schema(name)))
            .clone()
    }
}
```

### Use SIMD

```rust
#[cfg(target_arch = "x86_64")]
unsafe fn count_spaces_simd(input: &[u8]) -> usize {
    use std::arch::x86_64::*;

    let space = _mm_set1_epi8(b' ' as i8);
    let mut count = 0;
    let mut i = 0;

    while i + 16 <= input.len() {
        let chunk = _mm_loadu_si128(input.as_ptr().add(i) as *const __m128i);
        let cmp = _mm_cmpeq_epi8(chunk, space);
        count += _mm_popcnt_u32(_mm_movemask_epi8(cmp) as u32) as usize;
        i += 16;
    }

    count + input[i..].iter().filter(|&&b| b == b' ').count()
}
```

---

**Next**: Apply your knowledge by [Contributing](contributing.md) to HEDL
