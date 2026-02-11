# How to Add Benchmarks: Measuring What Matters

You wrote a new feature. Tests pass. Code looks clean. But is it fast? Will it stay fast? Without benchmarks, you cannot know.

Performance is a feature that can regress silently. A seemingly innocent change makes parsing twice as slow. A new allocation in a hot loop destroys throughput. Without measurements, these regressions slip into production, discovered only when users complain.

Benchmarks prevent this. They establish baselines, detect regressions, and prove optimizations work. This guide teaches you to create benchmarks that catch problems before they reach users.

---

## Goal

Add criterion benchmarks that measure performance and detect regressions.

## Prerequisites

- Understanding of the code you want to benchmark
- Basic Rust knowledge
- The `cargo bench` command working

---

## The Benchmark Crate Structure

HEDL benchmarks live in `crates/hedl-bench`:

```
crates/hedl-bench/
├── Cargo.toml              # Benchmark dependencies
├── benches/
│   ├── core/               # Core parsing benchmarks
│   │   └── parsing.rs
│   ├── formats/            # Format conversion benchmarks
│   │   ├── json.rs
│   │   └── yaml.rs
│   ├── features/           # Feature-specific benchmarks
│   │   └── references.rs
│   └── integration/        # End-to-end benchmarks
│       └── pipeline.rs
└── src/
    ├── lib.rs              # Shared utilities
    └── generators.rs       # Test data generation
```

Each benchmark file becomes a separate binary. Running `cargo bench --bench parsing` runs only the parsing benchmarks.

---

## Creating a Simple Benchmark

### Step 1: Create the Benchmark File

Create `crates/hedl-bench/benches/features/my_feature.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hedl_core::parse;

/// Benchmark parsing a simple document
fn benchmark_simple_parse(c: &mut Criterion) {
    let input = br#"%V:2.0
%NULL:~
%QUOTE:"
---
name: Alice
age: 30
active: true
"#;

    c.bench_function("parse_simple", |b| {
        b.iter(|| {
            parse(black_box(input)).unwrap()
        });
    });
}

criterion_group!(benches, benchmark_simple_parse);
criterion_main!(benches);
```

### Step 2: Register the Benchmark

Add to `crates/hedl-bench/Cargo.toml`:

```toml
[[bench]]
name = "my_feature"
harness = false
path = "benches/features/my_feature.rs"
```

### Step 3: Run the Benchmark

```bash
cd crates/hedl-bench
cargo bench --bench my_feature
```

Output:

```
my_feature/parse_simple
                        time:   [12.5 µs 12.7 µs 12.9 µs]
```

---

## Creating Parametric Benchmarks

Compare performance across different input sizes or types:

```rust
use criterion::{BenchmarkId, Criterion, Throughput};
use hedl_core::parse;

fn benchmark_parse_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_by_size");

    for size in [10, 100, 1_000, 10_000] {
        let input = generate_document(size);
        let bytes = input.len() as u64;

        // Report throughput in bytes/second
        group.throughput(Throughput::Bytes(bytes));

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &input,
            |b, input| {
                b.iter(|| parse(black_box(input.as_bytes())).unwrap());
            },
        );
    }

    group.finish();
}

fn generate_document(lines: usize) -> String {
    let mut doc = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n");
    for i in 0..lines {
        doc.push_str(&format!("key{}: value{}\n", i, i));
    }
    doc
}

criterion_group!(benches, benchmark_parse_by_size);
criterion_main!(benches);
```

Output shows scaling behavior:

```
parse_by_size/10       time:   [1.2 µs]   throughput: [120 MiB/s]
parse_by_size/100      time:   [8.5 µs]   throughput: [140 MiB/s]
parse_by_size/1000     time:   [75 µs]    throughput: [160 MiB/s]
parse_by_size/10000    time:   [720 µs]   throughput: [166 MiB/s]
```

---

## Comparing Implementations

Benchmark alternative approaches to find the fastest:

```rust
use criterion::Criterion;

fn benchmark_string_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_handling");

    let input = "  some value with whitespace  ";

    // Method 1: Clone after trim
    group.bench_function("trim_clone", |b| {
        b.iter(|| {
            let result: String = black_box(input).trim().to_string();
            black_box(result)
        });
    });

    // Method 2: Borrow and measure later if clone needed
    group.bench_function("trim_borrow", |b| {
        b.iter(|| {
            let result: &str = black_box(input).trim();
            black_box(result)
        });
    });

    // Method 3: Strip prefix/suffix pattern
    group.bench_function("strip_pattern", |b| {
        b.iter(|| {
            let result = black_box(input)
                .trim_start()
                .trim_end();
            black_box(result)
        });
    });

    group.finish();
}
```

Results reveal which approach is fastest:

```
string_handling/trim_clone   time:   [45.2 ns]
string_handling/trim_borrow  time:   [3.1 ns]    # 14x faster!
string_handling/strip_pattern time:  [3.3 ns]
```

---

## Benchmarking Format Conversion

Test round-trip conversion performance:

```rust
use criterion::{BenchmarkId, Criterion, Throughput};
use hedl_core::parse;
use hedl_json::{to_json, from_json, JsonOptions};

fn benchmark_json_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_conversion");

    let test_cases = vec![
        ("simple", generate_simple()),
        ("nested", generate_nested()),
        ("matrix", generate_matrix(100)),
    ];

    for (name, hedl) in test_cases {
        let doc = parse(hedl.as_bytes()).unwrap();
        let json_opts = JsonOptions::default();

        // Benchmark HEDL to JSON
        group.bench_with_input(
            BenchmarkId::new("to_json", name),
            &doc,
            |b, doc| {
                b.iter(|| to_json(black_box(doc), &json_opts).unwrap());
            },
        );

        // Benchmark JSON to HEDL
        let json_str = to_json(&doc, &json_opts).unwrap();
        group.throughput(Throughput::Bytes(json_str.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("from_json", name),
            &json_str,
            |b, json| {
                b.iter(|| from_json(black_box(json), &json_opts).unwrap());
            },
        );

        // Benchmark round-trip
        group.bench_with_input(
            BenchmarkId::new("round_trip", name),
            &hedl,
            |b, hedl| {
                b.iter(|| {
                    let doc = parse(black_box(hedl.as_bytes())).unwrap();
                    let json = to_json(&doc, &json_opts).unwrap();
                    from_json(&json, &json_opts).unwrap()
                });
            },
        );
    }

    group.finish();
}

fn generate_simple() -> String {
    r#"%V:2.0
%NULL:~
%QUOTE:"
---
name: Alice
age: 30
"#.to_string()
}

fn generate_nested() -> String {
    r#"%V:2.0
%NULL:~
%QUOTE:"
---
server:
 host: localhost
 port: 8080
 database:
  url: postgres://localhost
  pool_size: 10
"#.to_string()
}

fn generate_matrix(rows: usize) -> String {
    let mut doc = String::from(r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users: @User
"#);
    for i in 0..rows {
        doc.push_str(&format!(" |u{},User{},user{}@example.com\n", i, i, i));
    }
    doc
}
```

---

## Benchmark Configuration

### Adjust Measurement Time

For fast operations, increase measurement time for accuracy:

```rust
use std::time::Duration;

fn configure_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("precise_measurements");

    // More measurement time for stable results
    group.measurement_time(Duration::from_secs(15));

    // Longer warm-up to fill caches
    group.warm_up_time(Duration::from_secs(5));

    // More samples for statistical significance
    group.sample_size(200);

    // ... benchmarks ...

    group.finish();
}
```

### Prevent Optimization with `black_box`

The compiler aggressively optimizes away dead code. Use `black_box` to prevent this:

```rust
// Without black_box: compiler might not run parse at all
b.iter(|| parse(input));

// With black_box: compiler cannot optimize away
b.iter(|| parse(black_box(input)));

// Also black_box the result if needed
b.iter(|| {
    let result = parse(black_box(input));
    black_box(result)
});
```

### Set Significance Level

Configure how sensitive regression detection is:

```rust
group.significance_level(0.05);  // 5% significance (default)
group.noise_threshold(0.02);      // 2% changes are noise (default: 1%)
```

---

## Running and Interpreting Benchmarks

### Basic Run

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench my_feature

# Run specific function within benchmark
cargo bench --bench my_feature -- parse_simple
```

### Save Baselines

```bash
# Save current results as "before"
cargo bench --bench my_feature -- --save-baseline before

# Make changes...

# Compare against "before"
cargo bench --bench my_feature -- --baseline before
```

### Interpret Results

```
parse_simple           time:   [12.5 µs 12.7 µs 12.9 µs]
                       change: [-15.2% -12.8% -10.4%] (p < 0.001)
                       Performance improved significantly!
```

The three times are:
- Lower bound of 95% confidence interval
- Point estimate (most likely value)
- Upper bound of 95% confidence interval

The change shows:
- Percentage change from baseline
- p-value for statistical significance

Interpretation:
- `Performance improved significantly!`: Statistically significant improvement
- `Performance regressed significantly!`: Statistically significant regression
- `No change in performance detected`: Within noise threshold

---

## Generating Reports

### HTML Reports

Criterion generates detailed HTML reports:

```bash
cargo bench --bench my_feature

# Open report
open target/criterion/report/index.html
```

Reports include:
- Time distributions
- Comparison plots
- Regression analysis
- Historical trends

### Custom Reports

Generate markdown or JSON reports:

```rust
use std::fs;

fn generate_report(results: &[BenchmarkResult]) -> String {
    let mut report = String::from("# Performance Report\n\n");
    report.push_str("| Benchmark | Time | Throughput |\n");
    report.push_str("|-----------|------|------------|\n");

    for result in results {
        report.push_str(&format!(
            "| {} | {} | {} |\n",
            result.name,
            format_time(result.time_ns),
            format_throughput(result.throughput),
        ));
    }

    report
}

fn format_time(ns: f64) -> String {
    if ns < 1_000.0 {
        format!("{:.1} ns", ns)
    } else if ns < 1_000_000.0 {
        format!("{:.1} µs", ns / 1_000.0)
    } else {
        format!("{:.1} ms", ns / 1_000_000.0)
    }
}
```

---

## CI Integration

### GitHub Actions Workflow

```yaml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache
        uses: Swatinem/rust-cache@v2

      - name: Run benchmarks
        run: cargo bench --bench parsing -- --save-baseline pr

      - name: Compare to main
        if: github.event_name == 'pull_request'
        run: |
          git fetch origin main
          git checkout origin/main
          cargo bench --bench parsing -- --baseline pr --noplot 2>&1 | tee bench.txt

      - name: Check for regressions
        if: github.event_name == 'pull_request'
        run: |
          if grep -q "Performance has regressed" bench.txt; then
            echo "Performance regression detected!"
            exit 1
          fi
```

### Performance Budgets

Fail CI if performance degrades:

```rust
#[test]
fn performance_budget_parsing() {
    let input = generate_document(10_000);

    let start = std::time::Instant::now();
    let _doc = hedl_core::parse(input.as_bytes()).unwrap();
    let duration = start.elapsed();

    assert!(
        duration < std::time::Duration::from_millis(100),
        "Parse budget exceeded: {:?}ms > 100ms",
        duration.as_millis()
    );
}

#[test]
fn performance_budget_json_conversion() {
    let input = generate_document(1_000);
    let doc = hedl_core::parse(input.as_bytes()).unwrap();

    let start = std::time::Instant::now();
    let _json = hedl_json::to_json(&doc, &Default::default()).unwrap();
    let duration = start.elapsed();

    assert!(
        duration < std::time::Duration::from_millis(10),
        "JSON conversion budget exceeded: {:?}ms > 10ms",
        duration.as_millis()
    );
}
```

---

## Best Practices

### 1. Benchmark Real Workloads

Use inputs that represent actual usage:

```rust
// Good: realistic document structure
let input = include_str!("../../fixtures/real_config.hedl");

// Less good: synthetic data that may not match real patterns
let input = (0..1000).map(|i| format!("k{}: v{}", i, i)).collect();
```

### 2. Test Multiple Sizes

Performance often scales differently across sizes:

```rust
for size in [10, 100, 1_000, 10_000, 100_000] {
    group.bench_with_input(
        BenchmarkId::new("parse", size),
        &generate_document(size),
        |b, input| { /* ... */ },
    );
}
```

### 3. Separate Setup from Measurement

Do not include setup in measured time:

```rust
// Bad: setup included in measurement
b.iter(|| {
    let input = generate_large_input();  // Setup
    parse(&input)  // Actual work
});

// Good: setup outside measurement
let input = generate_large_input();
b.iter(|| parse(black_box(&input)));
```

### 4. Document What You Measure

Make benchmark purpose clear:

```rust
/// Measures parsing throughput for documents with deep nesting.
///
/// Deep nesting stresses the parser's recursion handling.
/// Expected: O(n) time where n is total nodes.
fn benchmark_deep_nesting(c: &mut Criterion) {
    // ...
}
```

---

## Verification

Ensure benchmarks work correctly:

```bash
# Run benchmarks (should complete without errors)
cargo bench --bench my_feature

# Run with verbose output
cargo bench --bench my_feature -- --verbose

# Check benchmark compiles with warnings
cargo clippy -p hedl-bench

# Verify baseline saving works
cargo bench --bench my_feature -- --save-baseline test
```

---

## Related Documentation

- **[Profile Performance](profile-performance.md)**: Find what to optimize
- **[Benchmarking Guide](../benchmarking.md)**: Comprehensive benchmarking documentation
- **[CI/CD](../operations/ci-cd.md)**: Automated benchmark tracking
