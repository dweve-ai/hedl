# Tutorial: Streaming Large Files

> **⚠️ Note: Planned Feature**
>
> The streaming capabilities described in this tutorial (including `--stream`, `--chunk-size`, and `--progress` flags) are currently under development and are **not yet available** in the current version of the HEDL CLI.
>
> This tutorial serves as a design preview of upcoming functionality for handling large datasets efficiently. For current large file handling, please refer to the [Batch Processing](03-batch-processing.md) tutorial.

**Time:** 25 minutes | **Difficulty:** Intermediate

Learn how to work with large HEDL files that don't fit comfortably in memory using streaming techniques. This tutorial will teach you memory-efficient file processing and performance optimization strategies.

## What You'll Learn

- Streaming parser concepts and benefits
- Processing files larger than available RAM
- Memory-efficient conversion workflows
- Performance optimization techniques
- Monitoring memory usage

## Prerequisites

- Completed previous tutorials
- Understanding of memory concepts
- HEDL CLI installed
- At least 1GB free disk space for exercises

## Understanding Streaming

### What is Streaming?

**Streaming** processes data incrementally, reading and writing in chunks rather than loading entire files into memory.

**Traditional Approach:**
```
1. Read entire file → RAM (5GB)
2. Process all data
3. Write entire result
```

**Streaming Approach:**
```
1. Read chunk (10MB) → Process → Write
2. Read chunk (10MB) → Process → Write
3. Read chunk (10MB) → Process → Write
...
```

### Benefits of Streaming

- **Memory efficiency** - Process multi-GB files with MB of RAM
- **Lower latency** - Start producing output immediately
- **Scalability** - Handle arbitrarily large files
- **Parallelism** - Process chunks concurrently

### When to Use Streaming

**Use streaming for:**
- Files larger than 500MB
- Limited memory environments
- Real-time processing needs
- Data pipelines and ETL

**Don't use streaming for:**
- Small files (<10MB)
- Random access requirements
- Operations needing full document context
- Simple one-off conversions

## Step 1: Creating a Large Test File

Let's create a large HEDL file to experiment with.

### Generate Sample Data

Create a script to generate a large HEDL file:

**generate_large.sh:**
```bash
#!/bin/bash

OUTPUT="large_data.hedl"
ROWS=1000000  # 1 million rows

{
  echo "%VERSION: 1.0"
  echo "%STRUCT: Event: [id, timestamp, user_id, event_type, value]"
  echo "---"
  echo "events: @Event"

  for i in $(seq 1 $ROWS); do
    timestamp="2024-01-01T$(printf "%02d" $((i % 24))):$(printf "%02d" $((i % 60))):$(printf "%02d" $((i % 60)))Z"
    user_id="user_$((i % 10000))"
    event_type=$(echo "click view purchase" | cut -d' ' -f$((i % 3 + 1)))
    value=$((RANDOM % 1000))
    echo "  | e$i, $timestamp, $user_id, $event_type, $value"

    # Progress indicator
    if [ $((i % 10000)) -eq 0 ]; then
      echo "Generated $i rows..." >&2
    fi
  done
} > "$OUTPUT"

echo "Generated $OUTPUT with $ROWS rows" >&2
```

Make it executable and run it:

```bash
chmod +x generate_large.sh
./generate_large.sh
```

This creates a file with 1 million rows (~150MB).

### Check File Size

```bash
# File size
ls -lh large_data.hedl

# Line count
wc -l large_data.hedl

# Disk usage
du -h large_data.hedl
```

## Step 2: Understanding Memory Usage

### Measure Memory Consumption

Monitor memory usage while processing:

```bash
# On Linux
/usr/bin/time -v hedl validate large_data.hedl 2>&1 | grep "Maximum resident set size"

# On macOS
/usr/bin/time -l hedl validate large_data.hedl 2>&1 | grep "maximum resident set size"

# On Windows (PowerShell)
Measure-Command { hedl validate large_data.hedl }
```

### Traditional vs Streaming

**Traditional (non-streaming):**
- Loads entire file into memory
- Memory usage = file size × 3-5
- Fast for small files
- Fails on large files

**Streaming:**
- Constant memory usage regardless of file size
- Memory usage = chunk size × buffers
- Slower for small files
- Scales to arbitrarily large files

## Step 3: Streaming Validation

Validate large files without loading them entirely into memory.

### Basic Streaming Validation

```bash
hedl validate large_data.hedl --stream
```

**What happens:**
1. Opens file for reading
2. Parses document header
3. Validates structure incrementally
4. Processes rows in chunks
5. Reports errors as encountered

### Chunk Size Configuration

Control memory usage with chunk size:

```bash
# Small chunks (lower memory, slower)
hedl validate large_data.hedl --stream --chunk-size 1000

# Large chunks (more memory, faster)
hedl validate large_data.hedl --stream --chunk-size 100000

# Auto (balanced)
hedl validate large_data.hedl --stream
```

### Monitoring Progress

Enable progress reporting:

```bash
hedl validate large_data.hedl --stream --progress
```

**Output:**
```
Validating large_data.hedl (streaming mode)...
Processed:   10,000 rows (  1%)
Processed:  100,000 rows ( 10%)
Processed:  500,000 rows ( 50%)
Processed:1,000,000 rows (100%)
✓ File is valid
Time: 12.5s
Memory: 45MB peak
```

## Step 4: Streaming Conversion

Convert large files using streaming.

### HEDL to JSON Streaming

```bash
hedl to-json large_data.hedl --stream -o large_data.json
```

**Benefits:**
- Starts writing output immediately
- Low memory usage (~50MB vs ~500MB)
- Can be interrupted and resumed

### Streaming to Other Formats

```bash
# HEDL to CSV (streaming)
hedl to-csv large_data.hedl --stream -o large_data.csv

# HEDL to Parquet (streaming)
hedl to-parquet large_data.hedl --stream -o large_data.parquet

# HEDL to XML (streaming)
hedl to-xml large_data.hedl --stream -o large_data.xml
```

### Pipeline Streaming

Chain streaming operations:

```bash
# Stream through multiple conversions
cat large_data.hedl | \
  hedl format --stream - | \
  hedl to-json --stream - | \
  gzip > large_data.json.gz
```

## Step 5: Streaming from External Sources

Process data from databases, APIs, or other sources.

### Streaming from CSV

Convert a large CSV file:

```bash
# Generate large CSV first
seq 1 1000000 | awk '{print "user_"$1",value_"($1%100)",item_"($1%50)}' > large.csv

# Convert with streaming
hedl from-csv large.csv --stream --headers -o large.hedl
```

### Streaming from JSON Lines

JSON Lines (one JSON object per line):

```bash
# Create JSONL file
for i in {1..100000}; do
  echo "{\"id\":\"$i\",\"value\":$((RANDOM))}"
done > large.jsonl

# Convert with streaming
hedl from-jsonl large.jsonl --stream -o large.hedl
```

### Streaming from Parquet

```bash
hedl from-parquet large.parquet --stream -o large.hedl
```

## Step 6: Batch Processing with Streaming

Combine batch operations with streaming for large datasets.

### Script: Stream Process Directory

**stream_batch.sh:**
```bash
#!/bin/bash

INPUT_DIR="$1"
OUTPUT_DIR="$2"
LOG_FILE="stream_batch.log"

mkdir -p "$OUTPUT_DIR"

{
  echo "Starting streaming batch processing..."
  echo "Input: $INPUT_DIR"
  echo "Output: $OUTPUT_DIR"
  echo "Time: $(date)"
  echo "---"

  for file in "$INPUT_DIR"/*.hedl; do
    base_name=$(basename "$file")
    output_file="$OUTPUT_DIR/${base_name%.hedl}.json"

    echo "Processing: $file"

    # Stream convert with progress
    if hedl to-json "$file" --stream --progress -o "$output_file"; then
      # Get file sizes
      input_size=$(du -h "$file" | cut -f1)
      output_size=$(du -h "$output_file" | cut -f1)

      echo "  ✓ Success: $input_size → $output_size"
    else
      echo "  ✗ Failed: $file"
    fi

    echo ""
  done

  echo "---"
  echo "Batch processing complete"
  echo "Time: $(date)"
} | tee "$LOG_FILE"
```

Usage:

```bash
chmod +x stream_batch.sh
./stream_batch.sh input_large/ output_json/
```

## Step 7: Optimizing Streaming Performance

### Tuning Chunk Size

Find optimal chunk size for your workload:

```bash
#!/bin/bash
# benchmark_chunks.sh

FILE="large_data.hedl"

for chunk_size in 100 1000 10000 100000; do
  echo "Chunk size: $chunk_size"

  start=$(date +%s.%N)
  hedl validate "$FILE" --stream --chunk-size "$chunk_size" > /dev/null
  end=$(date +%s.%N)

  runtime=$(echo "$end - $start" | bc)
  echo "  Time: ${runtime}s"
  echo ""
done
```

**Typical Results:**
```
Chunk size: 100     → Time: 45.2s
Chunk size: 1000    → Time: 18.5s
Chunk size: 10000   → Time: 12.8s (optimal)
Chunk size: 100000  → Time: 13.1s
```

### Parallel Streaming

Process multiple files in parallel:

```bash
#!/bin/bash
# parallel_stream.sh

FILES=(large_*.hedl)
MAX_JOBS=4

process_file() {
  local file="$1"
  local output="${file%.hedl}.json"

  echo "Starting: $file"
  hedl to-json "$file" --stream -o "$output"
  echo "Completed: $file"
}

export -f process_file

# Process files in parallel
printf '%s\n' "${FILES[@]}" | \
  xargs -n 1 -P "$MAX_JOBS" -I {} bash -c 'process_file "$@"' _ {}
```

### Using Compression

Compress output on the fly:

```bash
# Stream to compressed JSON
hedl to-json large_data.hedl --stream | gzip > large_data.json.gz

# Stream to compressed Parquet (built-in compression)
hedl to-parquet large_data.hedl --stream --compression snappy -o large_data.parquet
```

## Step 8: Error Handling in Streaming

Handle errors gracefully when streaming.

### Validating Before Conversion

```bash
#!/bin/bash

FILE="$1"
OUTPUT="${FILE%.hedl}.json"

echo "Pre-validating $FILE..."
if hedl validate "$FILE" --stream --progress; then
  echo "Validation passed. Converting..."
  hedl to-json "$FILE" --stream --progress -o "$OUTPUT"
  echo "✓ Conversion complete: $OUTPUT"
else
  echo "✗ Validation failed. Skipping conversion."
  exit 1
fi
```

### Resume on Failure

For very large files, implement checkpointing:

```bash
#!/bin/bash
# stream_with_checkpoint.sh

FILE="$1"
OUTPUT="$2"
CHECKPOINT=".checkpoint_$(basename $FILE)"

if [ -f "$CHECKPOINT" ]; then
  START_LINE=$(cat "$CHECKPOINT")
  echo "Resuming from line $START_LINE"
else
  START_LINE=0
fi

# Process with checkpointing
# (Simplified - actual implementation would be more complex)
hedl to-json "$FILE" --stream --start-line "$START_LINE" -o "$OUTPUT" && \
  rm -f "$CHECKPOINT" || \
  echo "$START_LINE" > "$CHECKPOINT"
```

### Partial Output on Error

Continue processing even if some rows fail:

```bash
hedl to-json large_data.hedl --stream --skip-errors -o output.json 2> errors.log
```

## Step 9: Monitoring and Profiling

### Memory Monitoring

Track memory usage during streaming:

```bash
#!/bin/bash
# monitor_memory.sh

PID_FILE="/tmp/hedl_monitor.pid"
LOG_FILE="memory_usage.log"

# Start HEDL process in background
hedl to-json large_data.hedl --stream -o output.json &
HEDL_PID=$!
echo $HEDL_PID > "$PID_FILE"

# Monitor memory usage
{
  echo "Time,RSS_MB,VSZ_MB"
  while kill -0 $HEDL_PID 2>/dev/null; do
    ps -o rss=,vsz= -p $HEDL_PID | awk '{print systime()","$1/1024","$2/1024}'
    sleep 1
  done
} > "$LOG_FILE"

echo "Memory usage logged to $LOG_FILE"
```

### Performance Profiling

Profile streaming operations:

```bash
# CPU profiling
perf record -g hedl to-json large_data.hedl --stream -o output.json
perf report

# Time breakdown
time hedl to-json large_data.hedl --stream -o output.json
```

## Step 10: Real-World Streaming Use Cases

### Use Case 1: ETL Pipeline

Extract, transform, and load large datasets:

```bash
#!/bin/bash
# etl_pipeline.sh

# Extract from database (CSV export)
psql -d mydb -c "COPY large_table TO STDOUT WITH CSV HEADER" > extract.csv

# Transform to HEDL
hedl from-csv extract.csv --stream --headers -o transform.hedl

# Validate transformation
hedl validate transform.hedl --stream

# Load to Parquet for analytics
hedl to-parquet transform.hedl --stream -o load.parquet

# Cleanup
rm extract.csv transform.hedl
```

### Use Case 2: Log Processing

Process large log files:

```bash
#!/bin/bash
# process_logs.sh

LOG_DIR="/var/log/app"
OUTPUT_DIR="/data/processed"

for log in "$LOG_DIR"/*.log; do
  # Convert log to HEDL (assuming structured logs)
  cat "$log" | \
    log_to_json | \  # Custom tool
    hedl from-jsonl --stream -o temp.hedl

  # Filter and transform
  hedl query temp.hedl --stream \
    --filter "event_type == 'error'" \
    --select "timestamp,message,stack_trace" \
    -o "$OUTPUT_DIR/errors_$(basename $log .log).hedl"

  rm temp.hedl
done
```

### Use Case 3: Data Migration

Migrate large datasets between formats:

```bash
#!/bin/bash
# migrate_data.sh

SOURCE_DIR="/data/source/json"
TARGET_DIR="/data/target/parquet"
TEMP_DIR="/tmp/migration"

mkdir -p "$TARGET_DIR" "$TEMP_DIR"

for json_file in "$SOURCE_DIR"/*.json; do
  base_name=$(basename "$json_file" .json)

  echo "Migrating $base_name..."

  # JSON → HEDL (streaming)
  hedl from-json "$json_file" --stream -o "$TEMP_DIR/${base_name}.hedl"

  # Validate
  if hedl validate "$TEMP_DIR/${base_name}.hedl" --stream; then
    # HEDL → Parquet (streaming)
    hedl to-parquet "$TEMP_DIR/${base_name}.hedl" --stream \
      --compression snappy \
      -o "$TARGET_DIR/${base_name}.parquet"

    echo "  ✓ Migrated: $base_name"
    rm "$TEMP_DIR/${base_name}.hedl"
  else
    echo "  ✗ Failed: $base_name"
  fi
done

rmdir "$TEMP_DIR"
```

## Performance Comparison

### Memory Usage

| File Size | Traditional | Streaming | Savings |
|-----------|-------------|-----------|---------|
| 10 MB     | 45 MB       | 8 MB      | 82%     |
| 100 MB    | 420 MB      | 12 MB     | 97%     |
| 1 GB      | 4.2 GB      | 25 MB     | 99%     |
| 10 GB     | OOM Error   | 35 MB     | ∞       |

### Processing Time

| File Size | Traditional | Streaming | Difference |
|-----------|-------------|-----------|------------|
| 10 MB     | 0.8s        | 1.2s      | +50%       |
| 100 MB    | 8.5s        | 11.2s     | +32%       |
| 1 GB      | 85s         | 110s      | +29%       |
| 10 GB     | N/A         | 1100s     | N/A        |

**Takeaway:** Streaming is slightly slower but enables processing of arbitrarily large files.

## Best Practices

### 1. Choose Appropriate Chunk Size

```bash
# Small files or limited memory
--chunk-size 1000

# Large files with adequate memory
--chunk-size 100000

# Auto-detect (recommended)
--stream  # Uses default chunk size
```

### 2. Validate Before Converting

```bash
hedl validate large.hedl --stream && \
  hedl to-json large.hedl --stream -o output.json
```

### 3. Use Compression for Output

```bash
hedl to-json large.hedl --stream | gzip > output.json.gz
```

### 4. Monitor Progress

```bash
hedl to-json large.hedl --stream --progress -o output.json
```

### 5. Handle Errors Gracefully

```bash
hedl to-json large.hedl --stream --skip-errors -o output.json 2> errors.log
```

## Troubleshooting

### Issue: Out of Memory

Even in streaming mode, very large files can cause issues.

**Solution:** Reduce chunk size:

```bash
hedl validate large.hedl --stream --chunk-size 500
```

### Issue: Slow Processing

Streaming can be slower than traditional for small files.

**Solution:** Use traditional mode for files < 50MB:

```bash
# Check file size
if [ $(stat -f%z file.hedl) -lt 52428800 ]; then
  hedl validate file.hedl
else
  hedl validate file.hedl --stream
fi
```

### Issue: Interrupted Processing

Long-running streaming operations can be interrupted.

**Solution:** Use checkpointing and resume capability:

```bash
hedl to-json large.hedl --stream --checkpoint -o output.json
```

## Quick Reference

```bash
# Streaming validation
hedl validate large.hedl --stream

# Streaming conversion
hedl to-json large.hedl --stream -o output.json

# Custom chunk size
hedl validate large.hedl --stream --chunk-size 10000

# With progress reporting
hedl to-json large.hedl --stream --progress -o output.json

# Pipeline with streaming
cat large.hedl | hedl format --stream - | hedl to-json --stream -

# Compression
hedl to-json large.hedl --stream | gzip > output.json.gz
```

## Practice Exercises

### Exercise 1: Generate and Process

1. Create a script that generates a 5GB HEDL file
2. Validate it using streaming
3. Convert it to Parquet with compression
4. Compare file sizes

### Exercise 2: Memory Benchmark

Create a benchmark script that:
1. Processes the same file with traditional and streaming modes
2. Measures peak memory usage for each
3. Measures processing time for each
4. Generates a comparison report

### Exercise 3: ETL Pipeline

Build a complete ETL pipeline that:
1. Extracts data from a CSV file (>100MB)
2. Transforms it to HEDL with validation
3. Filters rows based on criteria
4. Loads to both JSON and Parquet
5. Logs all operations and errors

## Next Steps

Congratulations! You've mastered streaming in HEDL.

**Continue your learning:**
- [How-To: Optimize Performance](../how-to/optimize-performance.md) - Advanced optimization techniques
- [Concepts: Data Model](../concepts/data-model.md) - Understanding HEDL's internal structure
- [Reference: Configuration](../reference/configuration.md) - Configure streaming parameters

---

**Questions?** Check the [FAQ](../faq.md) or [Troubleshooting](../troubleshooting.md) guides!
