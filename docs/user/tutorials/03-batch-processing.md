# Tutorial: Batch Processing

**Time:** 20 minutes | **Difficulty:** Intermediate

Learn how to process multiple HEDL files efficiently using batch operations, parallel processing, and shell scripting. This tutorial will help you automate repetitive tasks and work with large numbers of files.

## What You'll Learn

- Batch validation, formatting, and linting
- Parallel processing for performance
- Shell scripting with HEDL
- Error handling in batch workflows
- Progress reporting and logging

## Prerequisites

- Completed [Tutorial 2: CLI Basics](02-cli-basics.md)
- Basic shell scripting knowledge
- HEDL CLI installed

## Understanding Batch Operations

Batch operations allow you to:
- Process hundreds or thousands of files efficiently
- Automate validation and formatting workflows
- Run conversions on entire directories
- Parallelize work across CPU cores
- Generate reports across multiple files

## Step 1: Setting Up Sample Data

Create a sample directory structure with multiple HEDL files:

```bash
mkdir -p batch_demo/{data,output,logs}
cd batch_demo
```

Create several HEDL files:

**data/customers.hedl:**
```hedl
%VERSION: 1.0
%STRUCT: Customer: [id, name, email, country]
---
customers: @Customer
  | c1, Alice Johnson, alice@example.com, USA
  | c2, Bob Smith, bob@example.com, Canada
  | c3, Carol White, carol@example.com, UK
```

**data/products.hedl:**
```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, category, price]
---
products: @Product
  | p1, Laptop, Electronics, 999.99
  | p2, Mouse, Accessories, 29.99
  | p3, Keyboard, Accessories, 79.99
  | p4, Monitor, Electronics, 349.99
```

**data/orders.hedl:**
```hedl
%VERSION: 1.0
%STRUCT: Order: [id, customer_id, product_id, quantity, total]
---
orders: @Order
  | o1, c1, p1, 1, 999.99
  | o2, c1, p2, 2, 59.98
  | o3, c2, p3, 1, 79.99
  | o4, c3, p1, 1, 999.99
  | o5, c3, p4, 2, 699.98
```

## Step 2: Basic Batch Validation

The `batch-validate` command validates multiple files at once.

### Simple Batch Validation

```bash
hedl batch-validate data/*.hedl
```

**Output:**
```
Validating 3 files...
✓ data/customers.hedl (valid)
✓ data/products.hedl (valid)
✓ data/orders.hedl (valid)

Summary: 3/3 files valid (100%)
```

### What Happened?

- HEDL found all `.hedl` files in the `data/` directory
- Validated each file sequentially
- Reported results for each file
- Showed a summary at the end

### Using Glob Patterns

The shell expands glob patterns into a list of files before passing them to the HEDL CLI:

```bash
# All .hedl files in current directory (shell expands)
hedl batch-validate *.hedl

# Specific files
hedl batch-validate data/customers.hedl data/products.hedl

# Multiple patterns
hedl batch-validate data/*.hedl archive/*.hedl
```

**Note:** Do NOT quote the glob patterns - let the shell expand them. The CLI receives the expanded file list.

## Step 3: Parallel Processing

Use `--parallel` to process files concurrently:

```bash
hedl batch-validate data/*.hedl --parallel
```

**Output:**
```
Validating 3 files in parallel (4 threads)...
✓ data/products.hedl (valid)
✓ data/customers.hedl (valid)
✓ data/orders.hedl (valid)

Summary: 3/3 files valid (100%)
Time: 0.15s (vs 0.42s sequential)
Speedup: 2.8x
```

### When to Use Parallel Processing

**Use parallel processing when:**
- Processing many files (10+)
- Files are large (>100 KB)
- CPU is not maxed out
- Files are independent

**Don't use parallel processing when:**
- Few files (<5)
- Files are tiny (<10 KB)
- Limited CPU or memory
- Processing order matters

### Controlling Parallelism

Control the number of threads using the `RAYON_NUM_THREADS` environment variable:

```bash
# Use 8 threads
RAYON_NUM_THREADS=8 hedl batch-validate data/*.hedl --parallel

# Auto-detect (uses CPU core count by default)
hedl batch-validate data/*.hedl --parallel
```

## Step 4: Batch Formatting

The `batch-format` command formats multiple files.

### Basic Batch Formatting

```bash
hedl batch-format data/*.hedl
```

By default, this processes files and reports success/failure, but does not write changes unless an output directory is specified.

### Formatting to a Different Directory

Save formatted files to an output directory:

```bash
hedl batch-format data/*.hedl --output-dir output/
```

**Output:**
```
Formatting 3 files...
✓ data/customers.hedl → output/customers.hedl
✓ data/products.hedl → output/products.hedl
✓ data/orders.hedl → output/orders.hedl

Summary: 3 files formatted
```

**Note:** The output directory is required. Original files are not modified.

### Parallel Formatting

Combine with `--parallel` for speed:

```bash
hedl batch-format data/*.hedl --output-dir output/ --parallel
```

## Step 5: Batch Linting

Check multiple files for best practices:

```bash
hedl batch-lint data/*.hedl
```

**Output:**
```
Linting 3 files...

data/customers.hedl: No issues found ✓

data/products.hedl: No issues found ✓

data/orders.hedl:
  Line 3: Values look like references but aren't using @Type:id syntax
  Line 4: Values look like references but aren't using @Type:id syntax
  Line 5: Values look like references but aren't using @Type:id syntax

Summary: 1 file with warnings, 2 files clean
```

### Parallel Linting

```bash
hedl batch-lint data/*.hedl --parallel
```

## Step 6: Building a Batch Processing Script

Create a comprehensive batch processing script.

**batch_process.sh:**
```bash
#!/bin/bash

# Configuration
INPUT_DIR="data"
OUTPUT_DIR="output"
LOG_FILE="logs/batch_$(date +%Y%m%d_%H%M%S).log"
ERROR_COUNT=0

# Create output directory
mkdir -p "$OUTPUT_DIR"
mkdir -p "logs"

# Log function
log() {
  echo "[$(date +%Y-%m-%d\ %H:%M:%S)] $1" | tee -a "$LOG_FILE"
}

log "Starting batch processing..."

# Step 1: Validate all files
log "Step 1: Validating files..."
if hedl batch-validate $INPUT_DIR/*.hedl --parallel | tee -a "$LOG_FILE"; then
  log "✓ All files valid"
else
  log "✗ Validation failed"
  ERROR_COUNT=$((ERROR_COUNT + 1))
fi

# Step 2: Format files
log "Step 2: Formatting files..."
if hedl batch-format $INPUT_DIR/*.hedl --output-dir "$OUTPUT_DIR" --parallel | tee -a "$LOG_FILE"; then
  log "✓ Files formatted"
else
  log "✗ Formatting failed"
  ERROR_COUNT=$((ERROR_COUNT + 1))
fi

# Step 3: Lint formatted files
log "Step 3: Linting formatted files..."
hedl batch-lint $OUTPUT_DIR/*.hedl --parallel | tee -a "$LOG_FILE"

# Step 4: Validate formatted files
log "Step 4: Validating formatted files..."
if hedl batch-validate $OUTPUT_DIR/*.hedl --parallel | tee -a "$LOG_FILE"; then
  log "✓ Formatted files valid"
else
  log "✗ Formatted files validation failed"
  ERROR_COUNT=$((ERROR_COUNT + 1))
fi

# Summary
log "Processing complete. Errors: $ERROR_COUNT"

if [ $ERROR_COUNT -eq 0 ]; then
  log "✓ All steps successful!"
  exit 0
else
  log "✗ $ERROR_COUNT step(s) failed. Check log: $LOG_FILE"
  exit 1
fi
```

Make it executable:

```bash
chmod +x batch_process.sh
```

Run it:

```bash
./batch_process.sh
```

## Step 7: Error Handling in Batch Operations

Handle errors gracefully in batch scripts.

### Collecting Failed Files

```bash
#!/bin/bash

FAILED_FILES=()

for file in data/*.hedl; do
  if ! hedl validate "$file" 2>/dev/null; then
    FAILED_FILES+=("$file")
  fi
done

if [ ${#FAILED_FILES[@]} -gt 0 ]; then
  echo "Failed files:"
  printf '%s\n' "${FAILED_FILES[@]}"
else
  echo "All files valid!"
fi
```

### Continue on Error

Process all files even if some fail:

```bash
#!/bin/bash

SUCCESS=0
FAILED=0

for file in data/*.hedl; do
  if hedl validate "$file"; then
    hedl to-json "$file" -o "output/$(basename ${file%.hedl}.json)"
    SUCCESS=$((SUCCESS + 1))
  else
    echo "Skipping $file due to validation error"
    FAILED=$((FAILED + 1))
  fi
done

echo "Processed: $SUCCESS successful, $FAILED failed"
```

### Fail Fast

Stop on first error:

```bash
#!/bin/bash
set -e  # Exit on any error

for file in data/*.hedl; do
  hedl validate "$file"
  hedl to-json "$file" -o "output/$(basename ${file%.hedl}.json)"
done

echo "All files processed successfully"
```

## Step 8: Progress Reporting

Add progress indicators for long-running operations.

### Simple Progress Counter

```bash
#!/bin/bash

FILES=(data/*.hedl)
TOTAL=${#FILES[@]}
CURRENT=0

for file in "${FILES[@]}"; do
  CURRENT=$((CURRENT + 1))
  echo "[$CURRENT/$TOTAL] Processing $file..."
  hedl format "$file" -o "output/$(basename $file)"
done
```

### Progress Bar

Using `pv` (pipe viewer):

```bash
#!/bin/bash

FILES=(data/*.hedl)
TOTAL=${#FILES[@]}

echo "${FILES[@]}" | tr ' ' '\n' | pv -l -s "$TOTAL" | while read file; do
  hedl format "$file" -o "output/$(basename $file)"
done
```

## Step 9: Converting Entire Directories

Convert all files from one format to another.

### JSON to HEDL

```bash
#!/bin/bash
# json_to_hedl.sh

INPUT_DIR="json_files"
OUTPUT_DIR="hedl_files"

mkdir -p "$OUTPUT_DIR"

for json_file in "$INPUT_DIR"/*.json; do
  base_name=$(basename "$json_file" .json)
  hedl from-json "$json_file" -o "$OUTPUT_DIR/${base_name}.hedl"
  echo "✓ Converted $json_file"
done
```

### HEDL to Multiple Formats

```bash
#!/bin/bash
# hedl_to_all.sh

INPUT_FILE="$1"
BASE_NAME=$(basename "$INPUT_FILE" .hedl)
OUTPUT_DIR="output"

mkdir -p "$OUTPUT_DIR"

# Convert to all supported formats
hedl to-json "$INPUT_FILE" --pretty -o "$OUTPUT_DIR/${BASE_NAME}.json"
hedl to-yaml "$INPUT_FILE" -o "$OUTPUT_DIR/${BASE_NAME}.yaml"
hedl to-xml "$INPUT_FILE" --pretty -o "$OUTPUT_DIR/${BASE_NAME}.xml"
hedl to-csv "$INPUT_FILE" -o "$OUTPUT_DIR/${BASE_NAME}.csv"

echo "✓ Converted $INPUT_FILE to all formats"
```

## Step 10: Generating Reports

Create summary reports for batch operations.

### Validation Report

```bash
#!/bin/bash
# validation_report.sh

REPORT_FILE="validation_report.txt"

{
  echo "HEDL Validation Report"
  echo "Generated: $(date)"
  echo "================================"
  echo ""

  VALID=0
  INVALID=0

  for file in data/*.hedl; do
    if hedl validate "$file" 2>&1 | grep -q "valid"; then
      echo "✓ $file - VALID"
      VALID=$((VALID + 1))
    else
      echo "✗ $file - INVALID"
      hedl validate "$file" 2>&1 | sed 's/^/  /'
      INVALID=$((INVALID + 1))
    fi
  done

  echo ""
  echo "================================"
  echo "Summary:"
  echo "  Valid files: $VALID"
  echo "  Invalid files: $INVALID"
  echo "  Total files: $((VALID + INVALID))"
  echo "  Success rate: $(( VALID * 100 / (VALID + INVALID) ))%"
} | tee "$REPORT_FILE"
```

### Statistics Report

```bash
#!/bin/bash
# stats_report.sh

REPORT_FILE="stats_report.csv"

echo "File,HEDL Bytes,HEDL Tokens,JSON Tokens,Savings %" > "$REPORT_FILE"

for file in data/*.hedl; do
  # Get stats (simplified - actual parsing would be more complex)
  stats=$(hedl stats "$file")
  # Parse stats output and append to CSV
  # (Implementation would parse the actual stats output)
  echo "$(basename $file),..." >> "$REPORT_FILE"
done

echo "Report saved to $REPORT_FILE"
```

## Best Practices

### 1. Always Validate First

```bash
# Bad: Format without validating
hedl batch-format data/*.hedl --output-dir output/

# Good: Validate then format
hedl batch-validate data/*.hedl && hedl batch-format data/*.hedl --output-dir output/
```

### 2. Use Version Control

Before batch operations:

```bash
# Commit current state
git add data/
git commit -m "Before batch formatting"

# Run batch operation
hedl batch-format data/*.hedl --output-dir formatted/

# Review changes (compare directories)
diff -r data/ formatted/

# If good, replace originals (manual step required)
# cp formatted/*.hedl data/
```

### 3. Test on a Subset First

```bash
# Test on one file
hedl validate data/customers.hedl

# Test on a few files
hedl batch-validate data/customer*.hedl

# Run on all files
hedl batch-validate data/*.hedl --parallel
```

### 4. Use Meaningful Log Files

```bash
LOG_FILE="logs/batch_$(date +%Y%m%d_%H%M%S)_$(whoami).log"
hedl batch-validate data/*.hedl --parallel | tee "$LOG_FILE"
```

### 5. Handle Large File Sets

For thousands of files:

```bash
# Process in chunks
find data -name "*.hedl" -print0 | xargs -0 -n 100 -P 4 hedl validate
```

## Practical Use Cases

### Use Case 1: Daily Data Validation

```bash
#!/bin/bash
# daily_validation.sh - Run via cron

hedl batch-validate /data/incoming/*.hedl --parallel > /logs/validation_$(date +%Y%m%d).log

if [ $? -eq 0 ]; then
  # Move valid files to processing directory
  mv /data/incoming/*.hedl /data/processing/
else
  # Alert on validation failure
  mail -s "HEDL Validation Failed" admin@example.com < /logs/validation_$(date +%Y%m%d).log
fi
```

### Use Case 2: Data Migration

```bash
#!/bin/bash
# migrate_to_hedl.sh

# Convert all JSON files in archive to HEDL
for json in archive/**/*.json; do
  hedl_path="${json%.json}.hedl"
  hedl_dir=$(dirname "$hedl_path")

  mkdir -p "$hedl_dir"
  hedl from-json "$json" -o "$hedl_path"

  # Verify conversion
  if hedl validate "$hedl_path"; then
    echo "✓ Migrated $json"
  else
    echo "✗ Failed to migrate $json"
  fi
done
```

### Use Case 3: Pre-Deployment Check

```bash
#!/bin/bash
# pre_deploy.sh

echo "Running pre-deployment checks..."

# Validate all HEDL configs
if ! hedl batch-validate config/*.hedl --parallel; then
  echo "✗ Configuration validation failed"
  exit 1
fi

# Format check (don't modify, just verify)
for file in config/*.hedl; do
  if ! diff <(cat "$file") <(hedl format "$file"); then
    echo "✗ $file is not canonically formatted"
    exit 1
  fi
done

echo "✓ All pre-deployment checks passed"
```

## Troubleshooting

### Issue: Glob Pattern Not Matching

```bash
# CORRECT - shell expands pattern (do NOT quote)
hedl batch-validate data/*.hedl

# WRONG - shell doesn't expand quoted patterns, passes literal string
hedl batch-validate "data/*.hedl"
```

### Issue: Parallel Processing Slower

Parallel processing has overhead. For small files:

```bash
# If parallel is slower, use sequential
hedl batch-validate data/*.hedl
```

### Issue: Out of Memory

Process files in batches:

```bash
find data -name "*.hedl" | xargs -n 10 hedl batch-validate
```

## Quick Reference

```bash
# Batch validation
hedl batch-validate *.hedl
hedl batch-validate **/*.hedl --parallel

# Batch formatting
hedl batch-format *.hedl --output-dir output/
hedl batch-format *.hedl --output-dir output/ --parallel

# Batch linting
hedl batch-lint *.hedl --parallel

# With error handling
hedl batch-validate *.hedl || echo "Some files invalid"

# Progress tracking
FILES=(data/*.hedl); for i in "${!FILES[@]}"; do
  echo "$((i+1))/${#FILES[@]}: ${FILES[i]}"
  hedl validate "${FILES[i]}"
done
```

## Practice Exercises

### Exercise 1: Validation Pipeline

Create a script that:
1. Finds all `.hedl` files recursively
2. Validates them in parallel
3. Generates a CSV report with results
4. Sends email if any files are invalid

### Exercise 2: Format Standardization

Write a script to:
1. Format all HEDL files in a project
2. Check if formatting changed any files
3. Create a git commit with changes
4. Generate a summary of what was reformatted

### Exercise 3: Conversion Service

Build a directory watcher that:
1. Monitors a directory for new JSON files
2. Automatically converts them to HEDL
3. Validates the conversion
4. Moves files to appropriate directories based on validation result

## Next Steps

You've mastered batch processing! Continue your learning:

- [Tutorial 4: Streaming Large Files](04-streaming-large-files.md) - Handle files too large for memory
- [How-To: Optimize Performance](../how-to/optimize-performance.md) - Speed up processing
- [Reference: CLI Commands](../reference/cli-commands.md) - Complete command reference

---

**Questions?** Check the [FAQ](../faq.md) or [Troubleshooting](../troubleshooting.md) guides!
