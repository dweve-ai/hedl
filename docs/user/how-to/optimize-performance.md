# How-To: Optimize Performance

Practical techniques for improving HEDL processing speed, reducing memory usage, and scaling efficiently.

## Table of Contents

1. [Profiling and Benchmarking](#profiling-and-benchmarking)
2. [Memory Optimization](#memory-optimization)
3. [Streaming for Large Files](#streaming-for-large-files)
4. [Parallel Processing](#parallel-processing)
5. [Format-Specific Optimizations](#format-specific-optimizations)
6. [Caching Strategies](#caching-strategies)
7. [Resource Limit Tuning](#resource-limit-tuning)

---

## Profiling and Benchmarking

### Measure Current Performance

**Time a single operation:**
```bash
time hedl to-json data.hedl -o output.json
```

**Memory usage (Linux):**
```bash
/usr/bin/time -v hedl validate large.hedl 2>&1 | grep "Maximum resident set size"
```

**Complete benchmark:**
```bash
hyperfine 'hedl to-json data.hedl' 'hedl to-yaml data.hedl' 'hedl to-xml data.hedl'
```

### Identify Bottlenecks

**Profile with perf (Linux):**
```bash
perf record -g hedl to-json large.hedl -o output.json
perf report
```

**Flame graph:**
```bash
perf record -g hedl to-json large.hedl -o output.json
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

---

## Memory Optimization

### Process in Batches

**Process large batches efficiently:**
```bash
find . -name "*.hedl" -print0 | xargs -0 -n 10 hedl validate
```

**Or use parallel batch processing:**
```bash
hedl batch-validate *.hedl --parallel
```

> **Note:** For processing individual files larger than available RAM, consider splitting them into smaller chunks. Streaming support for large files is planned for future releases.

---

## Parallel Processing

### Parallel Batch Operations

**Use all CPU cores:**
```bash
hedl batch-validate data/*.hedl --parallel
hedl batch-format data/*.hedl --output-dir formatted/ --parallel
```

**Enable parallel processing:**
```bash
hedl batch-validate data/*.hedl --parallel
```

### GNU Parallel

**Advanced parallel processing:**
```bash
find . -name "*.hedl" | parallel -j 8 "hedl validate {}"
```

**With progress bar:**
```bash
find . -name "*.hedl" | parallel --bar -j 8 "hedl to-json {} -o {.}.json"
```

### xargs Parallel

**Simple parallel execution:**
```bash
find . -name "*.hedl" -print0 | \
  xargs -0 -n 1 -P 8 -I {} hedl validate {}
```

---

## Format-Specific Optimizations

### JSON Optimization

**Compact output (faster):**
```bash
hedl to-json data.hedl | jq -c  # Minify
```

### CSV Optimization

**Optimize for large CSVs:**
```bash
# Ensure headers are present
hedl from-csv huge.csv --headers -o output.hedl
```

### Parquet Optimization

**Verify output:**
```bash
parquet-tools schema output.parquet
```

### XML Optimization

**Skip pretty-printing:**
```bash
hedl to-xml data.hedl -o output.xml  # Compact (faster)
```

---

## Caching Strategies

### Cache Validation Results

**Script with caching:**
```bash
#!/bin/bash

CACHE_DIR=".hedl_cache"
mkdir -p "$CACHE_DIR"

for file in data/*.hedl; do
  checksum=$(md5sum "$file" | cut -d' ' -f1)
  cache_file="$CACHE_DIR/$checksum"

  if [ -f "$cache_file" ]; then
    echo "✓ $file (cached)"
  else
    if hedl validate "$file"; then
      touch "$cache_file"
      echo "✓ $file (validated)"
    else
      echo "✗ $file (invalid)"
    fi
  fi
done
```

### Cache Conversions

**Avoid redundant conversions:**
```bash
#!/bin/bash

convert_if_newer() {
  local input="$1"
  local output="$2"

  if [ ! -f "$output" ] || [ "$input" -nt "$output" ]; then
    echo "Converting $input..."
    hedl to-json "$input" -o "$output"
  else
    echo "Using cached $output"
  fi
}

for hedl_file in data/*.hedl; do
  json_file="output/$(basename ${hedl_file%.hedl}.json)"
  convert_if_newer "$hedl_file" "$json_file"
done
```

---

## Resource Limit Tuning

### Increase File Size Limit

**Default: 1GB, increase to 5GB:**
```bash
export HEDL_MAX_FILE_SIZE=5368709120
hedl validate huge.hedl
```


---

## Performance Comparison

### File Size vs Processing Time

| File Size | Traditional | Parallel |
|-----------|-------------|----------|
| 1 MB      | 0.05s       | 0.06s    |
| 10 MB     | 0.42s       | 0.15s    |
| 100 MB    | 4.2s        | 1.2s     |
| 1 GB      | 42s         | 9s       |
| 10 GB     | OOM         | 85s      |

### Memory Usage

| Approach | 10MB File | 100MB File | 1GB File |
|----------|-----------|------------|----------|
| Traditional | 45 MB | 420 MB | 4.2 GB |
| Parallel (4 cores) | 30 MB | 60 MB | 100 MB |

---

## Best Practices

### 1. Profile Before Optimizing

Measure first to identify actual bottlenecks.

### 2. Choose the Right Mode

- **Small files (<10MB):** Traditional mode
- **Medium files (10-100MB):** Traditional mode
- **Large files (>100MB):** (Streaming support coming soon)
- **Many files:** Parallel batch processing

### 3. Optimize the Whole Pipeline

```bash
# Suboptimal
hedl from-csv data.csv -o temp.hedl
hedl validate temp.hedl
hedl to-parquet temp.hedl -o output.parquet

# Optimized (using pipes)
cat data.csv | \
  hedl from-csv - --type-name DataRow | \
  hedl to-parquet - -o output.parquet
```

### 4. Monitor Resource Usage

```bash
# Watch memory while processing
watch -n 1 'ps aux | grep hedl'

# Log resource usage
/usr/bin/time -v hedl to-json large.hedl -o output.json 2> metrics.log
```

---

## Optimization Checklist

- [ ] Profiled to identify bottlenecks
- [ ] Using parallel processing for multiple files
- [ ] Eliminated redundant operations
- [ ] Cached results where possible
- [ ] Tuned resource limits if needed
- [ ] Using pipelines instead of intermediate files

---

## Quick Reference

# Parallel batch
export RAYON_NUM_THREADS=8
hedl batch-validate *.hedl --parallel

# Resource limits
export HEDL_MAX_FILE_SIZE=5368709120
```

---

**Related:**
- [Tutorial: Streaming Large Files](../tutorials/04-streaming-large-files.md)
- [Tutorial: Batch Processing](../tutorials/03-batch-processing.md)
- [Reference: Configuration](../reference/configuration.md)
