# How-To: Profile Performance

Identify and analyze performance bottlenecks in HEDL.

## Goal

Find where your code spends time and optimize hot paths.

## Tools Overview

| Tool | Purpose | Use Case |
|------|---------|----------|
| criterion | Microbenchmarks | Compare function performance |
| flamegraph | CPU profiling | Find hot functions |
| valgrind | Memory profiling | Find allocations |
| perf | System profiling | Linux performance analysis |

## Quick Start: Criterion Benchmarks

```bash
# Run existing benchmarks
cargo bench -p hedl-bench --bench parsing

# View results
open target/criterion/report/index.html
```

## Method 1: Flamegraph (CPU Profiling)

### Install

```bash
cargo install flamegraph
```

### Profile Binary

```bash
# Profile the CLI
cargo build --release --bin hedl
sudo flamegraph target/release/hedl parse large_file.hedl

# Opens flamegraph.svg in browser
firefox flamegraph.svg
```

### Profile Benchmark

```bash
cd crates/hedl-bench
cargo flamegraph --bench parsing -- --bench
```

### Interpret Flamegraph

- **Width**: Time spent (wider = more time)
- **Color**: Stack depth (red = hot, cold = base)
- **Click**: Zoom into function

**Example**:
```
main (100%)
  └─ parse (80%)
      └─ parse_node (60%)
          └─ parse_value (40%)  ← OPTIMIZE THIS
              └─ allocate (35%)
```

## Method 2: Criterion Detailed Analysis

### Create Benchmark

File: `crates/hedl-bench/benches/my_optimization.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hedl_core::parse;

fn benchmark_parse_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_by_size");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let input = generate_hedl_of_size(*size);

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

fn generate_hedl_of_size(n: usize) -> String {
    (0..n)
        .map(|i| format!("key{}: value{}", i, i))
        .collect::<Vec<_>>()
        .join("\n")
}

criterion_group!(benches, benchmark_parse_sizes);
criterion_main!(benches);
```

### Run with Statistics

```bash
cargo bench --bench my_optimization -- --save-baseline before

# Make optimization changes...

cargo bench --bench my_optimization -- --baseline before
```

Output shows:
```
parse_by_size/100      time:   [12.5 µs 12.7 µs 12.9 µs]
                      change: [-15.2% -12.8% -10.4%] (p < 0.001)
                      Performance improved!
```

## Method 3: Memory Profiling with Valgrind

### Install

```bash
# Linux
sudo apt install valgrind

# macOS
brew install valgrind
```

### Profile Allocations

```bash
# Build with debug symbols
cargo build --release

# Run with massif (heap profiler)
valgrind --tool=massif \
    --massif-out-file=massif.out \
    target/release/hedl parse test.hedl

# Visualize
ms_print massif.out | less
```

### Find Allocation Hot Spots

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAlloc;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

#[test]
fn test_allocation_count() {
    ALLOCATED.store(0, Ordering::SeqCst);

    let doc = parse(b"key: value").unwrap();

    let allocated = ALLOCATED.load(Ordering::SeqCst);
    println!("Allocated {} bytes", allocated);

    // Set budget
    assert!(allocated < 1024, "Allocated too much: {}", allocated);
}
```

## Method 4: Linux perf

```bash
# Record performance data
cargo build --release
perf record --call-graph dwarf target/release/hedl parse large.hedl

# Generate report
perf report

# Generate flamegraph from perf data
perf script | stackcollapse-perf.pl | flamegraph.pl > perf.svg
```

## Analyzing Results

### Identify Bottlenecks

1. **Hot Functions** (>5% of time):
   ```
   parse_value: 35%        ← Optimize this
   allocate_string: 20%    ← Reduce allocations
   hash_map_insert: 15%    ← Consider FxHashMap
   ```

2. **Allocation Patterns**:
   ```
   String::from: 1,000 calls  ← Use string slices?
   Vec::push: 10,000 calls    ← Pre-allocate with capacity?
   ```

### Common Optimizations

**Replace allocations with borrowing**:
```rust
// Before (allocates)
pub fn parse_key(input: &str) -> String {
    input.trim().to_string()
}

// After (zero-copy)
pub fn parse_key(input: &str) -> &str {
    input.trim()
}
```

**Pre-allocate collections**:
```rust
// Before
let mut items = Vec::new();
for i in 0..1000 {
    items.push(i);
}

// After
let mut items = Vec::with_capacity(1000);
for i in 0..1000 {
    items.push(i);
}
```

**Use faster hash function**:
```rust
// Before
use std::collections::HashMap;

// After (faster for small keys)
use rustc_hash::FxHashMap as HashMap;
```

## Regression Detection

### Setup Baseline

```bash
# Save current performance
cargo bench --bench parsing -- --save-baseline main

# After changes
cargo bench --bench parsing -- --baseline main
```

### Automated Checks

File: `.github/workflows/performance.yml`

```yaml
name: Performance Regression

on: [pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run benchmarks
        run: cargo bench --bench parsing -- --save-baseline pr

      - name: Compare with main
        run: |
          git checkout main
          cargo bench --bench parsing -- --baseline pr
```

## Example: Optimizing parse_value

### Before

```rust
pub fn parse_value(input: &str) -> Result<Value, HedlError> {
    let trimmed = input.trim().to_string();  // Allocation 1

    if trimmed.starts_with('"') {
        let unquoted = trimmed[1..trimmed.len()-1].to_string();  // Allocation 2
        Ok(Value::String(unquoted))
    } else if trimmed == "true" || trimmed == "false" {
        Ok(Value::Bool(trimmed == "true"))
    } else {
        trimmed.parse::<i64>()  // Allocation 3 (on error path)
            .map(Value::Int)
            .or_else(|_| trimmed.parse::<f64>().map(Value::Float))
            .map_err(|_| HedlError::syntax("Invalid value", 0))
    }
}
```

**Profile**: 100 µs per call, 3 allocations

### After

```rust
pub fn parse_value(input: &str) -> Result<Value, HedlError> {
    let trimmed = input.trim();  // No allocation

    if let Some(unquoted) = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        Ok(Value::String(unquoted.to_string()))  // Only allocate when needed
    } else if trimmed == "true" {
        Ok(Value::Bool(true))
    } else if trimmed == "false" {
        Ok(Value::Bool(false))
    } else if let Ok(i) = trimmed.parse::<i64>() {
        Ok(Value::Int(i))
    } else if let Ok(f) = trimmed.parse::<f64>() {
        Ok(Value::Float(f))
    } else {
        Err(HedlError::syntax(format!("Invalid value: {}", trimmed), 0))
    }
}
```

**Profile**: 45 µs per call (-55%), 1 allocation

## Benchmarking Best Practices

### 1. Use black_box

```rust
use criterion::black_box;

// Prevents compiler from optimizing away the call
b.iter(|| parse(black_box(input)));
```

### 2. Warm Up

```rust
group.measurement_time(Duration::from_secs(10));
group.warm_up_time(Duration::from_secs(3));
```

### 3. Statistical Significance

```rust
group.significance_level(0.05)
    .sample_size(100);
```

## Verification

Confirm optimization worked:

```bash
# Run benchmark suite
cargo bench --all

# Check no regression
cargo test --all --release

# Measure specific improvement
cargo bench --bench parsing -- parse_value --baseline before
```

Expected output:
```
parse_value            time:   [45.2 µs 46.1 µs 47.0 µs]
                      change: [-56.8% -54.1% -51.3%] (p < 0.001)
                      Performance improved significantly!
```

## Related

- [Add Benchmarks](add-benchmarks.md)
- [Performance Concepts](../concepts/zero-copy-optimizations.md)
- [How-To Index](README.md)
