# hedl-bench

**Comprehensive benchmark suite for HEDL - measure parsing performance, conversion overhead, and regression detection across all crates.**

Performance regressions slip into production. Optimization claims need quantitative validation. Comparing HEDL to alternatives requires rigorous measurement. Benchmarking shouldn't require custom harnesses for each test case. Tracking performance over time reveals degradation trends before they impact users.

`hedl-bench` provides 27+ benchmarks across 6 phases covering parsing, canonicalization, format conversion, validation, streaming, and tools. Includes dataset generators for realistic workloads (users, products, orders, analytics, graphs). Performance baselines with 4-level regression detection. Token efficiency comparison using tiktoken-rs. Memory profiling via Linux /proc/self/statm. Multiple export formats (Console, JSON, Markdown, HTML) for CI/CD integration.

## What's Implemented

Production-grade benchmarking infrastructure:

1. **27+ Benchmarks**: Parsing, canonicalization, conversion (6 formats), validation, streaming, LSP, MCP, lint
2. **6 Benchmark Phases**: Core, Canonicalization, Conversion, Validation, Streaming, Tools
3. **Dataset Generators**: Users, products, orders, analytics, graphs (configurable sizes)
4. **Performance Baselines**: Regression detection with 4 severity levels (0-5%, 5-10%, 10-20%, >20%)
5. **Token Efficiency**: HEDL vs JSON/YAML/TOON comparison using tiktoken-rs cl100k_base
6. **Memory Profiling**: Linux /proc/self/statm integration for memory usage measurement
7. **Statistical Analysis**: Mean, std_dev, min, max, percentiles (p50, p90, p95, p99)
8. **Export Formats**: Console (default), JSON (CI/CD), Markdown (docs), HTML (reports)
9. **Iteration Control**: Configurable warmup + measurement iterations
10. **Benchmark Isolation**: Each benchmark runs in isolated process for accurate measurement

## Installation

```bash
# Run all benchmarks
cargo bench -p hedl-bench

# Run specific benchmark
cargo bench -p hedl-bench -- parse_users

# Run benchmark category
cargo bench -p hedl-bench -- conversion
```

## Benchmark Categories

### Core Parsing (Phase 1)

Measure fundamental parsing performance:

```bash
cargo bench -p hedl-bench -- core
```

**Benchmarks**:
- `parse_small` - 100-line document with basic key-value pairs (5 KB)
- `parse_medium` - 1000-line document with users list (50 KB)
- `parse_large` - 10,000-line document with nested structures (500 KB)
- `parse_users_1k` - 1000 user entities with matrix format
- `parse_blog` - Blog post with comments and tags
- `parse_scalars` - All scalar types exhaustively

**Metrics**:
- Throughput (MB/s)
- Latency (µs)
- Allocations (count)

### Canonicalization (Phase 2)

Measure normalization performance:

```bash
cargo bench -p hedl-bench -- canon
```

**Benchmarks**:
- `canon_basic` - Simple document canonicalization
- `canon_sorted` - Sorted keys with alphabetic ordering
- `canon_large` - 10K entity canonicalization

**Configurations Tested**:
- `sort_keys: true` vs `false`
- `inline_schemas: true` vs `false`

### Format Conversion (Phase 3)

Measure bidirectional conversion overhead:

```bash
cargo bench -p hedl-bench -- conversion
```

**Benchmarks**:
- `hedl_to_json` + `json_to_hedl` (roundtrip)
- `hedl_to_yaml` + `yaml_to_hedl` (roundtrip)
- `hedl_to_xml` + `xml_to_hedl` (roundtrip)
- `hedl_to_csv` + `csv_to_hedl` (single list)
- `hedl_to_parquet` + `parquet_to_hedl` (columnar)
- `hedl_to_toon` + `toon_to_hedl` (LLM-optimized)

**Comparison Metrics**:
- Conversion time (µs)
- Output size (bytes)
- Roundtrip fidelity (pass/fail)
- Token count (tiktoken cl100k_base)

### Validation (Phase 4)

Measure validation and linting performance:

```bash
cargo bench -p hedl-bench -- validation
```

**Benchmarks**:
- `validate_basic` - Basic syntax validation
- `validate_strict` - Strict mode with reference checking
- `lint_all_rules` - All 5 lint rules enabled
- `lint_large_doc` - Linting 10K entity document

**Rules Benchmarked**:
- id-naming
- unused-schema
- empty-list
- unqualified-kv-ref
- unused-alias

### Streaming (Phase 5)

Measure streaming parser performance:

```bash
cargo bench -p hedl-bench -- streaming
```

**Benchmarks**:
- `stream_small` - 1K entities streaming
- `stream_medium` - 10K entities streaming
- `stream_large` - 100K entities streaming

**Memory Measurement**:
- Peak memory (bytes)
- Memory per entity (bytes)
- Constant memory verification (should be O(nesting_depth))

### Tools (Phase 6)

Measure LSP and MCP tool performance:

```bash
cargo bench -p hedl-bench -- tools
```

**Benchmarks**:
- `lsp_completion` - Completion request latency
- `lsp_diagnostics` - Diagnostic generation
- `lsp_hover` - Hover info retrieval
- `mcp_query` - MCP query tool
- `mcp_validate` - MCP validation tool

**Metrics**:
- Request latency (ms)
- Debounce effectiveness
- Cache hit rate

## Dataset Generators

Pre-built generators for realistic workloads:

### generate_users(count)

Generate user entities with realistic data:

```rust
use hedl_bench::generate_users;

let doc = generate_users(1000);
// 1000 users with id, name, email, role, created_at fields
```

**Fields**: id, name, email, role, created_at
**Size**: ~80 bytes per user
**Use Case**: Basic entity list benchmarking

### generate_products(count)

Generate product catalog:

```rust
let doc = generate_products(500);
// 500 products with id, name, price, category, stock, description
```

**Fields**: id, name, price, category, stock, description
**Size**: ~100 bytes per product
**Use Case**: E-commerce workloads

### generate_orders(count)

Generate orders with nested items:

```rust
let doc = generate_orders(100);
// 100 orders, each with 1-5 items (nested structure)
```

**Fields**: Order(id, customer, status, total) + Item(sku, name, quantity, price)
**Size**: ~200 bytes per order + 50 bytes per item
**Use Case**: Nested structure benchmarking

### generate_blog(posts, comments_per_post)

Generate blog with posts and comments:

```rust
let doc = generate_blog(50, 20);
// 50 blog posts, each with 20 comments
```

**Structure**: Author(id, name, email) + Post(id, title, author, published_at) + Comment(id, author, content, created_at)
**Use Case**: Hierarchical content benchmarking

### generate_analytics(count)

Generate analytics time series:

```rust
let doc = generate_analytics(10000);
// 10K time series data points
```

**Fields**: id, timestamp, name, value, tags
**Use Case**: Large dataset performance

### generate_graph(nodes, edges_per_node)

Generate graph data with references:

```rust
let doc = generate_graph(1000, 5);
// 1000 nodes, average 5 edges each (5000 total references)
```

**Structure**: Node(id, label) with reference fields to other nodes
**Use Case**: Reference resolution performance

## Performance Baselines

Regression detection with 4 severity levels:

```bash
# Save baseline
cargo bench -p hedl-bench -- --save-baseline main

# Compare against baseline
cargo bench -p hedl-bench -- --baseline main
```

**Regression Severity**:
- **None** (0-4%): No regression detected
- **Minor** (5-14%): Minor regression, monitor if consistent
- **Moderate** (15-49%): Moderate regression, requires investigation
- **Severe** (50%+): Severe regression, blocks merge

**CI/CD Integration**:
```yaml
- name: Benchmark Regression Check
  run: |
    cargo bench -p hedl-bench -- --baseline main --format json > bench_results.json
    python scripts/check_regression.py bench_results.json
```

## Token Efficiency Comparison

Compare HEDL token efficiency against alternatives:

```bash
cargo bench -p hedl-bench -- token_efficiency
```

**Benchmarks**:
- `tokens_hedl_vs_json` - HEDL matrix vs JSON array-of-objects
- `tokens_hedl_vs_yaml` - HEDL vs YAML equivalent
- `tokens_hedl_vs_toon` - HEDL vs TOON format

**Tokenizer**: tiktoken-rs with cl100k_base encoding (GPT-3.5/4/Claude)

**Example Results**:
```
hedl_users_1k:        2,847 tokens
json_users_1k:        3,156 tokens  (+10.9%)
yaml_users_1k:        3,423 tokens  (+20.2%)
toon_users_1k:        2,951 tokens  (+3.7%)
```

## Memory Profiling

Linux-specific memory measurement:

```bash
cargo bench -p hedl-bench -- memory_profile
```

**Metrics** (from /proc/self/statm):
- VmSize: Virtual memory size
- VmRSS: Resident set size (actual RAM)
- VmData: Data segment size
- VmStk: Stack size

**Benchmarks**:
- `memory_parse_1mb` - Memory usage parsing 1 MB document
- `memory_stream_10mb` - Streaming 10 MB with constant memory
- `memory_conversion` - Memory overhead of format conversions

**Verification**: Streaming benchmarks verify O(nesting_depth) memory, not O(document_size)

## Statistical Analysis

Benchmarks collect comprehensive performance metrics:

```rust
// Performance result for a single benchmark run
pub struct PerfResult {
    pub name: String,
    pub iterations: u64,
    pub total_time_ns: u64,
    pub throughput_bytes: Option<u64>,
    pub avg_time_ns: Option<u64>,
    pub throughput_mbs: Option<f64>,
}

// Statistical analysis from multiple measurements
pub struct Statistics {
    pub mean: Duration,
    pub std_dev: Duration,
    pub min: Duration,
    pub max: Duration,
    pub median: Duration,
}

// Percentile measurements for baseline tracking
pub struct Percentiles {
    pub p50: u64,  // Median
    pub p95: u64,
    pub p99: u64,
}
```

**Console Output**:
```
parse_users_1k
  avg_time:   1.234 ms
  throughput: 40.5 MB/s
  iterations: 100
```

## Export Formats

### Console (Default)

Human-readable terminal output:

```bash
cargo bench -p hedl-bench
```

### JSON (CI/CD)

Machine-readable for automation:

```bash
cargo bench -p hedl-bench -- --format json > results.json
```

**Schema**:
```json
{
  "benchmarks": [
    {
      "name": "parse_users_1k",
      "mean_us": 1234.5,
      "std_dev_us": 45.2,
      "p95_us": 1315.0,
      "throughput_mbs": 40.5
    }
  ]
}
```

### Markdown (Documentation)

Generate performance documentation:

```bash
cargo bench -p hedl-bench -- --format markdown > PERFORMANCE.md
```

**Output**: Formatted tables with comparisons

### HTML (Reports)

Generate visual reports:

```bash
cargo bench -p hedl-bench -- --format html > report.html
```

**Features**: Sortable tables, charts, trend graphs

## Iteration Control

Configure warmup and measurement iterations:

```bash
# 10 warmup, 100 measurement
cargo bench -p hedl-bench -- --warmup 10 --iterations 100

# Quick run (less accurate)
cargo bench -p hedl-bench -- --quick

# Thorough run (more accurate)
cargo bench -p hedl-bench -- --thorough
```

**Defaults**:
- Warmup: 100ms duration
- Iterations: Size-dependent (1000 for small, 100 for medium, 10 for large datasets)
- Baseline path: `baselines/current.json`
- Export formats: Console and JSON

## Use Cases

**Performance Regression Detection**: Run benchmarks in CI/CD to catch performance regressions before merge. Fail builds on moderate (>15%) or severe (>50%) degradation.

**Optimization Validation**: Quantify performance improvements from optimizations. Verify claims with rigorous measurement.

**Capacity Planning**: Use throughput metrics to estimate server requirements. Plan infrastructure scaling based on measured performance.

**Format Comparison**: Compare HEDL performance against JSON/YAML/XML for migration decisions. Measure conversion overhead for hybrid systems.

**Token Efficiency Analysis**: Quantify LLM context window savings from HEDL. Optimize token usage for GPT-3.5/4/Claude applications.

**Memory Budget Verification**: Confirm streaming parser maintains constant memory. Verify O(1) memory claims with profiling data.

## What This Crate Doesn't Do

**Production Profiling**: Benchmarks run in controlled environment. For production profiling, use actual workloads with real data.

**Flamegraphs**: No flamegraph generation. Use `cargo flamegraph` or `perf` for detailed profiling.

**Custom Workloads**: Generators cover common cases. For domain-specific workloads, write custom dataset generators.

**Distributed Benchmarking**: Single-machine benchmarks only. For distributed system testing, use external frameworks.

## Dependencies

- `hedl-core` 2.0 - Core implementation
- All format crates (json, yaml, xml, csv, parquet, toon, neo4j)
- `hedl-stream` 2.0 - Streaming parser
- `hedl-lint` 2.0 - Linting
- `hedl-c14n` 2.0 - Canonicalization
- `criterion` 0.5 - Benchmarking framework
- `tiktoken-rs` 0.6 - Token counting

## License

Apache-2.0
