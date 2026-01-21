# Performance Architecture

## Performance Philosophy

HEDL is designed for high-performance data processing with multiple optimization layers:

1. **Algorithmic Optimization**: O(n) algorithms where possible
2. **Memory Efficiency**: Minimizes allocations during parsing (though AST uses owned Strings)
3. **Cache Optimization**: Data structures optimized for cache locality
4. **SIMD Utilization**: Vectorized hot paths
5. **Parallel Processing**: Multi-threaded where beneficial

## Performance Metrics

Based on benchmark suite (`cargo bench --bench parsing`, 2025-01-19):

**Parsing (release build):**

| Document Size | Latency | Throughput |
|---------------|---------|------------|
| 10 keys flat | ~20 µs | ~42 MiB/s |
| 50 keys flat | ~115 µs | ~34 MiB/s |
| 100 keys flat | ~229 µs | ~34 MiB/s |
| 500 keys flat | ~1.12 ms | ~35 MiB/s |
| Nested (5 parents, 2 children) | ~41 µs | ~49 MiB/s |

**JSON Conversion (release build, from `target/criterion/`):**

| Direction | Document | Latency |
|-----------|----------|---------|
| HEDL→JSON | users/10 | ~3.4 µs |
| HEDL→JSON | users/100 | ~54 µs |
| HEDL→JSON | users/1000 | ~576 µs |
| HEDL→JSON | products/1000 | ~645 µs |
| JSON→HEDL | users/10 | ~8.9 µs |
| JSON→HEDL | users/100 | ~84 µs |
| JSON→HEDL | users/1000 | ~851 µs |
| JSON→HEDL | products/1000 | ~819 µs |
| Roundtrip | blog/100 | ~105 µs |

**FFI Performance (1000 items):**

| Method | Latency | Overhead |
|--------|---------|----------|
| Native Rust | ~602 µs | baseline |
| FFI (C ABI) | ~587 µs | ~2.5% faster* |

*FFI can be faster due to different allocation patterns

**Streaming (array_streamer):**

| Items | Latency | Per-item |
|-------|---------|----------|
| 1,000 | ~523 µs | ~523 ns |
| 10,000 | ~5.28 ms | ~528 ns |

**Canonicalization:**

| Algorithm | Latency |
|-----------|---------|
| JSON RFC 8785 | ~664 ns |

**Cross-format Comparison:**

| Operation | Latency |
|-----------|---------|
| JSON parse via HEDL | ~369 µs |
| serde_json parse (direct) | ~180 µs |

Run `cargo bench -p hedl-bench` for full benchmark suite.

## Optimization Layers

```mermaid
graph TB
    subgraph "Layer 1: Algorithmic"
        ALG[Linear Algorithms<br/>O(n) complexity]
    end

    subgraph "Layer 2: Memory"
        MEM[Efficient Allocation<br/>Minimal Copying]
    end

    subgraph "Layer 3: CPU"
        CPU[SIMD<br/>Cache Optimization]
    end

    subgraph "Layer 4: Concurrency"
        CONC[Parallel Processing<br/>Async I/O]
    end

    ALG --> MEM --> CPU --> CONC

    style ALG fill:#e3f2fd
    style MEM fill:#fff3e0
    style CPU fill:#e8f5e9
    style CONC fill:#f3e5f5
```

## 1. Algorithmic Optimization

### Linear Complexity

All core operations are O(n) or better:

```rust
// Preprocessing: O(n) single pass
fn preprocess(input: &[u8]) -> impl Iterator<Item = &str> {
    std::str::from_utf8(input).unwrap().lines()
}

// Lexing: O(n) line-by-line processing
// Note: Actual implementation in lex/regions.rs is more complex,
// handling protected regions (quotes, expressions) to avoid
// stripping # inside strings.
fn strip_comment(line: &str) -> &str {
    // SIMD-optimized byte scanning with memchr
    if let Some(pos) = memchr::memchr(b'#', line.as_bytes()) {
        &line[..pos].trim_end()
    } else {
        line.trim_end()
    }
}

// Parsing: O(n) single pass
fn parse(input: &str) -> Result<Document> {
    // Single-pass recursive descent with security limits
}
```

### Efficient Data Structures

**BTreeMap vs HashMap**:
```rust
// Use BTreeMap for deterministic iteration (canonicalization)
pub struct Document {
    pub root: BTreeMap<String, Item>,  // O(log n) lookup
}

// Use HashMap for fast lookup (internal caches)
pub struct InferenceContext {
    alias_cache: HashMap<String, Value>,  // O(1) lookup
}
```

**Trade-off**: BTreeMap has O(log n) vs HashMap O(1), but provides:
- Deterministic ordering
- Better cache locality for small maps
- Lower memory overhead

### Index-Based Access

```rust
// Source tracking without re-allocating strings
pub struct Span {
    pub start: usize,
    pub end: usize,
}
```

## 2. Memory Optimization

HEDL prioritizes efficient memory usage through:

- **Input Buffering**: Parser operates on buffered input to minimize I/O overhead
- **Pre-allocation**: Uses `with_capacity` for vectors when sizes are known
- **In-place Processing**: Trimming and scanning without allocations where possible
- **FFI Optimization**: Zero-copy data transfer at FFI boundaries where possible

**Trade-off**: The current AST uses owned `String` types for safety and simplicity, which involves allocation. Zero-copy optimizations are focused on I/O and conversion boundaries.

### Memory Layout

**Structure of Arrays (SoA) for Matrices (Future Optimization)**:
```rust
// Current: Array of Structures with SmallVec optimization
pub struct Node {
    fields: SmallVec<[Value; 4]>,  // Stack-allocated for ≤4 fields
}

// Structure of Arrays (SoA) - cache friendly
pub struct NodeSoA {
    strings: Vec<Cow<'static, str>>,  // Contiguous strings
    numbers: Vec<f64>,                // Contiguous numbers
    bools: Vec<bool>,                 // Contiguous bools
}
```

**Potential application in**: Matrix list parsing for columnar access patterns

## 3. CPU Optimization

### SIMD Byte Searching

**memchr for Fast Scanning**:

The codebase uses `memchr` for SIMD-optimized byte searching. Example pattern:

```rust
use memchr::memchr_iter;

// Conceptual example - actual usage in preprocess.rs and lex/regions.rs
// uses memchr for newline detection and comment scanning
for pos in memchr_iter(b'\n', input) {
    // Process each line boundary
}
```

**Performance**: 5-10x faster than naive byte-by-byte scanning

### Compiler Optimizations

The Rust compiler applies automatic inlining based on heuristics. The codebase relies on the compiler's optimization passes rather than manual `#[inline]` hints:

```rust
// Small, frequently-called functions are auto-inlined by the compiler
pub fn is_valid_key_token(s: &str) -> bool {
    // Simple validation, compiler inlines automatically
}

pub fn calculate_indent(line: &str) -> usize {
    // Critical path, auto-inlined at -O2/release
}
```

**Note**: Manual `#[inline]` hints are avoided unless profiling demonstrates a measurable benefit. The compiler's LTO (Link-Time Optimization) handles cross-crate inlining in release builds.

### Branch Prediction

**Likely/Unlikely Paths**:
```rust
// Fast path (likely)
if likely(is_simple_value(token)) {
    return infer_simple_value(token);
}

// Slow path (unlikely)
if unlikely(is_complex_expression(token)) {
    return parse_expression(token);
}
```

**Note**: Rust doesn't have built-in `likely/unlikely`, but ordering helps branch predictor

### Loop Optimization

**Iterator Chains vs Manual Loops**:
```rust
// Iterator chain (compiler optimizes to SIMD)
let sum: f64 = values.iter().map(|v| v.as_f64()).sum();

// Manual loop (no SIMD)
let mut sum = 0.0;
for v in values {
    sum += v.as_f64();
}
```

**Prefer**: Iterator chains for auto-vectorization

## 4. Concurrency Optimization

### Parallel Processing

**Note**: The core HEDL library (`hedl-core`) does not use rayon for parallel processing. Rayon is used only in the benchmark suite (`hedl-bench`) for measuring parallel throughput.

**Benchmark Usage**:
```rust
// In hedl-bench (benchmarks only, not core library)
use rayon::prelude::*;

// Parallel benchmark computation
files.par_iter()
    .map(|file| {
        let content = std::fs::read_to_string(file).unwrap();
        let doc = parse(&content).unwrap();
        compute_stats(&doc)
    })
    .collect()
```

**For Applications**: If you need parallel HEDL processing, add rayon to your application and parallelize at the file level:
```rust
// Your application code
use rayon::prelude::*;

files.par_iter()
    .map(|path| hedl::parse(&std::fs::read(path)?))
    .collect()
```

**Scaling**: Near-linear speedup for independent files when parallelized at application level

### Async I/O

**Tokio for Async Operations**:
```rust
use tokio::io::AsyncBufReadExt;

pub async fn parse_async(reader: impl AsyncBufRead) -> Result<Document> {
    let mut lines = Vec::new();
    let mut reader = tokio::io::BufReader::new(reader);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await? {
            0 => break,
            _ => lines.push(line),
        }
    }

    parse_lines(&lines)
}
```

**Use Case**: Network streams, large files

### Lock-Free Data Structures

**Reference Counting for Shared Ownership**:
```rust
use std::sync::Arc;

// Thread-safe reference counting
pub struct SharedDocument {
    inner: Arc<Document>,
}

impl Clone for SharedDocument {
    fn clone(&self) -> Self {
        SharedDocument {
            inner: Arc::clone(&self.inner),  // Lock-free atomic increment
        }
    }
}
```

## 5. Caching Strategies

### LRU Cache (LSP)

```rust
use lru::LruCache;

pub struct DocumentCache {
    cache: LruCache<Url, Document>,
    max_size: usize,
}

impl DocumentCache {
    pub fn get_or_parse(&mut self, url: &Url, content: &str) -> Result<&Document> {
        if let Some(doc) = self.cache.get(url) {
            return Ok(doc);  // Cache hit
        }

        let doc = parse(content)?;
        self.cache.put(url.clone(), doc);  // Cache miss, parse and store
        Ok(self.cache.get(url).unwrap())
    }
}
```

**Impact**: 100x faster for repeated LSP requests

### Schema Cache (JSON)

```rust
use once_cell::sync::Lazy;

static SCHEMA_CACHE: Lazy<Mutex<HashMap<u64, Schema>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn get_schema(json: &str) -> Schema {
    let hash = calculate_hash(json);

    let cache = SCHEMA_CACHE.lock().unwrap();
    if let Some(schema) = cache.get(&hash) {
        return schema.clone();  // Cache hit
    }
    drop(cache);

    let schema = infer_schema(json);

    let mut cache = SCHEMA_CACHE.lock().unwrap();
    cache.insert(hash, schema.clone());

    schema
}
```

**Impact**: 10-20x faster for repeated JSON structures

## 6. Streaming Optimization

### Chunked Reading

```rust
pub struct ChunkedReader {
    buffer: Vec<u8>,
    chunk_size: usize,
    position: usize,
}

impl ChunkedReader {
    pub fn read_chunk(&mut self, reader: &mut impl Read) -> Result<&[u8]> {
        self.buffer.resize(self.chunk_size, 0);
        let n = reader.read(&mut self.buffer)?;
        self.buffer.truncate(n);
        Ok(&self.buffer)
    }
}
```

**Chunk Size**: 64KB for optimal I/O and cache performance

### Backpressure Control

```rust
pub async fn process_stream<R: AsyncRead + Unpin>(
    reader: R,
    handler: &mut impl StreamHandler,
) -> Result<()> {
    let mut parser = AsyncStreamingParser::new(reader).await?;

    while let Some(event) = parser.next_event().await? {
        // Apply backpressure by awaiting handler
        handler.handle_event(event).await?;
    }

    Ok(())
}
```

## Performance Monitoring

### Benchmark Suite

**30+ benchmarks** covering:
- Core operations (lexing, parsing, validation)
- Format conversions (JSON, YAML, XML, CSV, Parquet, Neo4j)
- Features (streaming, canonicalization, zero-copy)
- Integration (end-to-end, roundtrip, parallel)

**Run benchmarks**:
```bash
cargo bench --workspace
```

**View reports**:
```bash
open crates/hedl-bench/target/comprehensive_report.html
```

### Profiling

**CPU Profiling with perf**:
```bash
cargo build --release
perf record --call-graph=dwarf ./target/release/hedl-cli parse large.hedl
perf report
```

**Memory Profiling with heaptrack**:
```bash
heaptrack ./target/release/hedl-cli parse large.hedl
heaptrack_gui heaptrack.hedl-cli.*.gz
```

**Flamegraph**:
```bash
cargo install flamegraph
cargo flamegraph --bench parsing
```

## Performance Budget

Based on criterion benchmarks (2025-01-19, release build):

| Operation | Target | Measured |
|-----------|--------|----------|
| Parse 10 keys flat | <50 µs | ~20 µs |
| Parse 100 keys flat | <500 µs | ~229 µs |
| Parse 500 keys flat | <2 ms | ~1.12 ms |
| HEDL→JSON 1000 users | <1 ms | ~576 µs |
| JSON→HEDL 1000 users | <1 ms | ~851 µs |
| Stream 1000 items | <1 ms | ~523 µs |
| Canonicalize | <5 µs | ~664 ns |

Throughput: ~33-49 MiB/s for parsing (varies by document structure)

## Optimization Guidelines

### When to Optimize

1. **Measure First**: Always profile before optimizing
2. **Hot Path Focus**: Optimize the 20% that's called 80% of the time
3. **Algorithmic First**: O(n²) → O(n) beats micro-optimizations
4. **Memory Next**: Reduce allocations before SIMD
5. **SIMD Last**: Only after exhausting simpler optimizations

### What NOT to Optimize

- Cold paths (error handling, rare features)
- Already fast operations (<1% of runtime)
- Code clarity for negligible gains

### Benchmarking Discipline

**Before every optimization**:
1. Run baseline benchmark
2. Record metrics
3. Make change
4. Run benchmark again
5. Compare results

**Example**:
```bash
# Baseline
cargo bench --bench parsing -- --save-baseline before

# Make optimization
# ...

# Compare
cargo bench --bench parsing -- --baseline before
```

## See Also

- [Parsing Pipeline](parsing-pipeline.md) - Parser implementation details
- [Data Flow](data-flow.md) - Data transformation flow
- `target/criterion/` - Criterion benchmark results (generated locally)

---

