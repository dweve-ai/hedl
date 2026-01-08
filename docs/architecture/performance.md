# Performance Architecture

## Performance Philosophy

HEDL is designed for high-performance data processing with multiple optimization layers:

1. **Algorithmic Optimization**: O(n) algorithms where possible
| **Layer 2** | **Memory Efficiency** | Minimizes allocations during parsing (though AST uses owned Strings) |
3. **Cache Optimization**: Data structures optimized for cache locality
4. **SIMD Utilization**: Vectorized hot paths
5. **Parallel Processing**: Multi-threaded where beneficial

## Performance Metrics

Based on benchmark suite (`hedl-bench`):

| Operation | Throughput | Latency | Memory |
|-----------|------------|---------|--------|
| **Lexing** | 500+ MB/s | <1ms (small) | O(1) streaming |
| **Parsing** | 200+ MB/s | 1-5ms (small) | O(n) heap |
| **JSON Export** | 150+ MB/s | 2-10ms | O(n) |
| **CSV Parsing** | 400+ MB/s | <2ms | O(1) streaming |
| **Validation** | 300+ MB/s | <3ms | O(n) |

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
// Array of Structures (AoS) - cache unfriendly
pub struct NodeAoS {
    fields: Vec<Value>,  // Mixed types, poor locality
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
```rust
use memchr::memmem;

// Find all '@' for reference detection
pub fn find_references(input: &str) -> Vec<usize> {
    memmem::find_iter(input.as_bytes(), b"@")
        .collect()
}

// Find all '#' for comment detection
pub fn find_comments(input: &str) -> Vec<usize> {
    memmem::find_iter(input.as_bytes(), b"#")
        .collect()
}
```

**Performance**: 5-10x faster than naive byte-by-byte scanning

### Inline Hints

**Hot Path Functions**:
```rust
#[inline]
pub fn parse(input: &str) -> Result<Document> {
    // Frequently called, small function
}

#[inline]
pub fn is_valid_key_token(s: &str) -> bool {
    // Simple validation, called frequently
}

#[inline(always)]
pub fn calculate_indent(line: &str) -> usize {
    // Critical path, always inline
}
```

**Impact**: 5-10% improvement in small document parsing

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

**rayon for Data Parallelism**:
```rust
use rayon::prelude::*;

// Parallel stats computation
pub fn compute_stats_parallel(files: &[PathBuf]) -> Vec<Stats> {
    files.par_iter()
        .map(|file| {
            let content = std::fs::read_to_string(file).unwrap();
            let doc = parse(&content).unwrap();
            compute_stats(&doc)
        })
        .collect()
}
```

**Scaling**: Near-linear speedup for independent files

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

Target performance for typical operations:

| Operation | Size | Latency | Throughput | Memory |
|-----------|------|---------|------------|--------|
| Parse small doc | 1KB | <1ms | 1000+ docs/s | <10KB |
| Parse medium doc | 100KB | <50ms | 2000+ KB/s | <1MB |
| Parse large doc | 10MB | <5s | 2000+ KB/s | <50MB |
| JSON conversion | 100KB | <20ms | 5000+ KB/s | <500KB |
| Streaming parse | 1GB | <60s | 17+ MB/s | <10MB |

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
- [Benchmark Reports](../../crates/hedl-bench/target/) - Detailed performance data

---

*Last updated: 2026-01-06*
