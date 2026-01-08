# HEDL CLI Reference Guide

Complete reference for all HEDL command-line interface commands, options, and usage patterns.

## Table of Contents

1. [Overview](#overview)
2. [Global Options](#global-options)
3. [Core Commands](#core-commands)
4. [Conversion Commands](#conversion-commands)
5. [Batch Commands](#batch-commands)
6. [Utility Commands](#utility-commands)
7. [Exit Codes](#exit-codes)
8. [Environment Variables](#environment-variables)
9. [Advanced Usage](#advanced-usage)

## Overview

The HEDL CLI provides a comprehensive toolkit for working with HEDL files:

```bash
hedl <COMMAND> [OPTIONS] <ARGS>
```

### Quick Command Reference

```bash
# Core operations
hedl validate <file>          # Validate HEDL syntax
hedl format <file>            # Format to canonical form
hedl lint <file>              # Check best practices
hedl inspect <file>           # Show internal structure
hedl stats <file>             # Compare format sizes

# Format conversion
hedl to-json <file>           # Convert to JSON
hedl from-json <file>         # Convert from JSON
hedl to-yaml <file>           # Convert to YAML
hedl from-yaml <file>         # Convert from YAML
hedl to-xml <file>            # Convert to XML
hedl from-xml <file>          # Convert from XML
hedl to-csv <file>            # Convert to CSV
hedl from-csv <file>          # Convert from CSV
hedl to-parquet <file>        # Convert to Parquet
hedl from-parquet <file>      # Convert from Parquet
hedl to-toon <file>           # Convert to TOON

# Batch operations
hedl batch-validate <files>   # Validate multiple files
hedl batch-format <files>     # Format multiple files
hedl batch-lint <files>       # Lint multiple files

# Utilities
hedl completion <shell>       # Generate shell completions
```

## Global Options

These options work with all commands:

### Help and Version

```bash
# Show help
hedl --help
hedl <command> --help

# Show version
hedl --version
```

### Common Patterns

All commands support these common patterns:

```bash
# Read from file
hedl <command> file.hedl

# Read from stdin
hedl <command> -
cat file.hedl | hedl <command> -

# Write to stdout (default)
hedl <command> file.hedl

# Write to file
hedl <command> file.hedl -o output.hedl
hedl <command> file.hedl --output output.hedl
```

## Core Commands

### validate

Validate HEDL file syntax and structure.

**Usage:**
```bash
hedl validate [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to HEDL file

**Options:**
- `-s, --strict`: Strict mode (fail on any error)

**Examples:**
```bash
# Validate a file
hedl validate data.hedl

# Strict validation
hedl validate data.hedl --strict

# Validate in a script
if hedl validate data.hedl; then
  echo "Valid!"
else
  echo "Invalid!" >&2
  exit 1
fi
```

**Exit Codes:**
- `0`: File is valid
- `1`: File is invalid or error occurred

**Output:**
```
✓ data.hedl is valid
```

Or on error:
```
Error: Parse error at line 5, column 12: unexpected token
```

---

### format

Format HEDL file to canonical form (standardized spacing, ordering).

**Usage:**
```bash
hedl format [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to HEDL file (use `-` for stdin)

**Options:**
- `-o, --output <FILE>`: Output file path (default: stdout)
- `-c, --check`: Check only (exit 1 if not canonical)
- `--ditto`: Use ditto optimization (default: true)
- `--with-counts`: Automatically add count hints to all matrix lists

**Examples:**
```bash
# Format to stdout
hedl format messy.hedl

# Format to new file
hedl format messy.hedl -o clean.hedl

# Format to output directory (batch)
hedl batch-format *.hedl --output-dir clean/

# Format from stdin
cat messy.hedl | hedl format - > clean.hedl

# Check if file is canonical (CI/CD)
hedl format data.hedl --check

# Format with count hints
hedl format data.hedl --with-counts -o optimized.hedl

# Disable ditto optimization
hedl format data.hedl --ditto false
```

**What It Does:**
- Standardizes indentation (2 spaces)
- Removes trailing whitespace
- Normalizes line endings
- Orders fields consistently
- Preserves semantic meaning
- Optionally applies ditto (^) optimization for repeated values
- Optionally adds count hints to matrix lists

**Before:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1,   Alice
    | u2, Bob
```

**After:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
  | u2, Bob
```

---

### lint

Check HEDL file for best practices and potential issues.

**Usage:**
```bash
hedl lint [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to HEDL file (use `-` for stdin)

**Options:**
- `-f, --format <FORMAT>`: Output format: `text` or `json` (default: text)
- `-W, --warn-error`: Treat warnings as errors

**Examples:**
```bash
# Lint a file
hedl lint data.hedl

# Lint from stdin
cat data.hedl | hedl lint -

# Output as JSON (for CI/CD integration)
hedl lint data.hedl --format json

# Fail on warnings (strict mode)
hedl lint data.hedl --warn-error
```

**Checks:**
- Unused entity types (`unused-schema`)
- Inconsistent naming conventions (`id-naming`)
- Empty matrix lists (`empty-list`)
- Unqualified references in Key-Value pairs (`unqualified-kv-ref`)
- Suspicious patterns

**Output:**
```
Linting data.hedl...

Warning (line 5): Schema 'Product' is defined but never used
Warning (line 12): ID 'a' is very short, consider a more descriptive name
Info (line 20): Matrix list 'users' is empty

3 issues found (0 errors, 2 warnings, 1 info)
```

---

### inspect

Display the internal parsed structure (useful for debugging).

**Usage:**
```bash
hedl inspect [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to HEDL file (use `-` for stdin)

**Options:**
- `-v, --verbose`: Show detailed internal structure

**Examples:**
```bash
# Inspect structure
hedl inspect data.hedl

# Inspect from stdin
echo '%VERSION: 1.0\n---\nname: Alice' | hedl inspect -

# Show verbose internal details
hedl inspect data.hedl --verbose
```

**Output:**
```
Document {
  version: "1.0",
  root: Object {
    fields: [
      Field {
        key: "name",
        value: String("Alice")
      }
    ]
  }
}
```

---

### stats

Show size and token statistics comparing HEDL to other formats.

**Usage:**
```bash
hedl stats [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to HEDL file (use `-` for stdin)

**Options:**
- `-t, --tokens`: Show estimated token counts for LLM context

**Examples:**
```bash
# Show statistics
hedl stats data.hedl

# Show with token counts
hedl stats data.hedl --tokens

# Compare multiple files
for f in *.hedl; do echo "$f:"; hedl stats "$f" --tokens; echo; done
```

**Output:**
```
Format Comparison for data.hedl:

  HEDL:      245 bytes,    62 tokens (baseline)
  JSON:      512 bytes,   156 tokens (+109%, +94 tokens)
  YAML:      398 bytes,   118 tokens (+62%, +56 tokens)
  XML:       687 bytes,   203 tokens (+180%, +141 tokens)
  CSV:       189 bytes,    48 tokens (-23%, -14 tokens)
  Parquet:    98 bytes,   N/A        (-60%)

Token Efficiency:
  - 60% fewer tokens than JSON
  - 47% fewer tokens than YAML
  - 69% fewer tokens than XML
  - CSV not applicable (tabular format)

Recommendations:
  ✓ HEDL is optimal for LLM context windows
  ✓ Consider Parquet for analytics (smallest binary)
  ✓ Use CSV for simple tabular exports
```

## Conversion Commands

### to-json

Convert HEDL to JSON format.

**Usage:**
```bash
hedl to-json [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to HEDL file (use `-` for stdin)

**Options:**
- `-o, --output <FILE>`: Output file path (default: stdout)
- `--pretty`: Pretty-print with indentation
- `--metadata`: Include HEDL metadata in output

**Examples:**
```bash
# Compact JSON to stdout
hedl to-json data.hedl

# Pretty-printed JSON
hedl to-json data.hedl --pretty

# Include metadata
hedl to-json data.hedl --metadata --pretty

# Save to file
hedl to-json data.hedl --pretty -o output.json

# Pipeline usage
hedl to-json data.hedl | jq '.users[] | select(.age > 30)'
```

---

### from-json

Convert JSON to HEDL format.

**Usage:**
```bash
hedl from-json [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to JSON file (use `-` for stdin)

**Options:**
- `-o, --output <FILE>`: Output file path (default: stdout)

**Examples:**
```bash
# Basic conversion
hedl from-json data.json

# Save to file
hedl from-json data.json -o data.hedl

# Pipeline usage
curl https://api.example.com/users | hedl from-json - > users.hedl
```

---

### to-yaml / from-yaml

Convert between HEDL and YAML formats.

**Usage:**
```bash
hedl to-yaml [OPTIONS] <FILE>
hedl from-yaml [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to file (use `-` for stdin)

**Options:**
- `-o, --output <FILE>`: Output file path (default: stdout)

**Examples:**
```bash
# HEDL to YAML
hedl to-yaml config.hedl -o config.yaml

# YAML to HEDL
hedl from-yaml config.yaml -o config.hedl

# Pipeline
hedl from-json data.json | hedl to-yaml - > data.yaml
```

---

### to-xml / from-xml

Convert between HEDL and XML formats.

**Usage:**
```bash
hedl to-xml [OPTIONS] <FILE>
hedl from-xml [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to file (use `-` for stdin)

**Options (to-xml):**
- `-o, --output <FILE>`: Output file path (default: stdout)
- `-p, --pretty`: Pretty-print with indentation

**Options (from-xml):**
- `-o, --output <FILE>`: Output file path (default: stdout)

**Examples:**
```bash
# HEDL to XML (compact)
hedl to-xml data.hedl

# HEDL to XML (pretty-printed)
hedl to-xml data.hedl --pretty -o output.xml

# XML to HEDL
hedl from-xml data.xml -o data.hedl
```

---

### to-csv / from-csv

Convert between HEDL and CSV formats.

**Usage:**
```bash
hedl to-csv [OPTIONS] <FILE>
hedl from-csv [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to file (use `-` for stdin)

**Options (to-csv):**
- `-o, --output <FILE>`: Output file path (default: stdout)
- `--headers`: Include header row (default: true)

**Options (from-csv):**
- `-o, --output <FILE>`: Output file path (default: stdout)
- `-t, --type-name <NAME>`: Type name for the matrix list (default: `Row`)

**Examples:**
```bash
# HEDL to CSV
hedl to-csv data.hedl -o output.csv

# HEDL to CSV without headers
hedl to-csv data.hedl --headers false -o output.csv

# CSV to HEDL
hedl from-csv data.csv -o data.hedl

# CSV to HEDL (custom type name)
hedl from-csv users.csv --type-name User -o users.hedl
```

---

### to-parquet / from-parquet

Convert between HEDL and Apache Parquet formats.

**Usage:**
```bash
hedl to-parquet <FILE> -o <OUTPUT>
hedl from-parquet [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to file

**Options (to-parquet):**
- `-o, --output <FILE>`: Output Parquet file path (required)

**Options (from-parquet):**
- `-o, --output <FILE>`: Output file path (default: stdout)

**Examples:**
```bash
# HEDL to Parquet
hedl to-parquet data.hedl -o output.parquet

# Parquet to HEDL (stdout)
hedl from-parquet data.parquet

# Parquet to HEDL (file)
hedl from-parquet data.parquet -o data.hedl

# Pipeline: CSV → HEDL → Parquet
hedl from-csv data.csv | hedl to-parquet - -o data.parquet
```

**Note:** Parquet output requires a file path (cannot write to stdout).

### to-toon

Convert HEDL to TOON (Token-Oriented Object Notation) format.

**Usage:**
```bash
hedl to-toon [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>`: Path to HEDL file (use `-` for stdin)

**Options:**
- `-o, --output <FILE>`: Output file path (default: stdout)

**Examples:**
```bash
# HEDL to TOON
hedl to-toon data.hedl

# Save to file
hedl to-toon data.hedl -o data.toon

# Pipeline
hedl from-json data.json | hedl to-toon -
```

## Batch Commands

Process multiple files in parallel.

### batch-validate

Validate multiple HEDL files.

**Usage:**
```bash
hedl batch-validate [OPTIONS] <FILES>...
```

**Arguments:**
- `<FILES>...`: Input file paths (supports glob patterns)

**Options:**
- `-s, --strict`: Strict mode (fail on any error)
- `-p, --parallel`: Force parallel processing
- `-v, --verbose`: Show verbose progress

**Examples:**
```bash
# Validate all HEDL files in current directory
hedl batch-validate *.hedl

# Validate recursively (shell glob)
hedl batch-validate data/*.hedl config/*.hedl

# With strict mode
hedl batch-validate *.hedl --strict

# Verbose output
hedl batch-validate *.hedl --verbose
```

**Output:**
```
Validating 15 files...

✓ data/users.hedl
✓ data/products.hedl
✗ data/orders.hedl: Parse error at line 23
✓ config/settings.hedl

Results: 14 valid, 1 invalid
```

**Exit Code:**
- `0`: All files valid
- `1`: One or more files invalid

---

### batch-format

Format multiple HEDL files.

**Usage:**
```bash
hedl batch-format [OPTIONS] <FILES>...
```

**Arguments:**
- `<FILES>...`: Input file paths (supports glob patterns)

**Options:**
- `-o, --output-dir <DIR>`: Output directory for formatted files
- `-c, --check`: Check only (exit 1 if not canonical)
- `--ditto`: Use ditto optimization (default: true)
- `--with-counts`: Automatically add count hints to all matrix lists
- `-p, --parallel`: Force parallel processing
- `-v, --verbose`: Show verbose progress

**Examples:**
```bash
# Check if files are canonical
hedl batch-format *.hedl --check

# Format to output directory
hedl batch-format src/*.hedl --output-dir formatted/

# Format with verbose output
hedl batch-format *.hedl --output-dir out/ --verbose

# Format with count hints
hedl batch-format *.hedl --output-dir out/ --with-counts
```

**Safety:**
- Use `--check` to verify formatting without modifying
- Use `--output-dir` to write to a separate directory

---

### batch-lint

Lint multiple HEDL files.

**Usage:**
```bash
hedl batch-lint [OPTIONS] <FILES>...
```

**Arguments:**
- `<FILES>...`: Input file paths (supports glob patterns)

**Options:**
- `-W, --warn-error`: Treat warnings as errors
- `-p, --parallel`: Force parallel processing
- `-v, --verbose`: Show verbose progress

**Examples:**
```bash
# Lint all HEDL files
hedl batch-lint *.hedl

# Lint with verbose output
hedl batch-lint *.hedl --verbose

# Treat warnings as errors (CI/CD)
hedl batch-lint *.hedl --warn-error
```

**Output:**
```
Linting 10 files...

data/users.hedl: 3 issues (0 errors, 2 warnings, 1 info)
data/products.hedl: ✓ No issues
config/settings.hedl: 1 issue (0 errors, 1 warning, 0 info)

Summary: 4 total issues across 2 files
```

## Utility Commands

### completion

Generate shell completion scripts.

**Usage:**
```bash
hedl completion [OPTIONS] <SHELL>
```

**Arguments:**
- `<SHELL>`: Shell type (`bash`, `zsh`, `fish`, `powershell`, `elvish`)

**Options:**
- `-i, --install`: Print installation instructions instead of generating script

**Examples:**
```bash
# Show installation instructions
hedl completion bash --install

# Bash
hedl completion bash > ~/.local/share/bash-completion/completions/hedl
source ~/.bashrc

# Zsh
hedl completion zsh > ~/.zfunc/_hedl
# Add to ~/.zshrc: fpath=(~/.zfunc $fpath)

# Fish
hedl completion fish > ~/.config/fish/completions/hedl.fish

# PowerShell
hedl completion powershell >> $PROFILE

# Elvish
hedl completion elvish > ~/.elvish/lib/hedl.elv
```

**Features:**
- Command name completion
- Option completion
- File path completion
- Context-aware suggestions

## Exit Codes

All HEDL commands use standard exit codes:

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error (parse error, invalid input, I/O error) |

**Usage in scripts:**
```bash
#!/bin/bash
set -e  # Exit on any error

hedl validate data.hedl || {
  echo "Validation failed!" >&2
  exit 1
}

hedl to-json data.hedl -o output.json
echo "Success!"
```

## Environment Variables

### HEDL_MAX_FILE_SIZE

Maximum file size to process (in bytes).

**Default:** `1073741824` (1 GB)

**Usage:**
```bash
# Allow processing of 5GB files
export HEDL_MAX_FILE_SIZE=5368709120

# Process large file
hedl validate large_file.hedl

# One-time override
HEDL_MAX_FILE_SIZE=10737418240 hedl validate huge_file.hedl
```

**Security Note:** This limit prevents out-of-memory (OOM) attacks. Set carefully based on available RAM.

## Advanced Usage

### Pipeline Processing

Chain HEDL commands with Unix pipes:

```bash
# Validate, format, and convert in one pipeline
cat data.hedl | hedl validate - && \
  hedl format - | \
  hedl to-json - --pretty > output.json

# Multi-format conversion
hedl from-csv data.csv --headers | \
  hedl format - | \
  hedl to-parquet - -o data.parquet

# Process API response
curl -s https://api.example.com/data | \
  hedl from-json - | \
  hedl lint - | \
  hedl to-yaml - > config.yaml
```

### Batch Processing Scripts

**Validate all files before processing:**
```bash
#!/bin/bash
set -e

echo "Validating all HEDL files..."
if ! hedl batch-validate data/*.hedl; then
  echo "Validation failed!" >&2
  exit 1
fi

echo "Converting to JSON..."
for f in data/*.hedl; do
  hedl to-json "$f" --pretty -o "json/${f%.hedl}.json"
done

echo "Done!"
```

**Multi-format export:**
```bash
#!/bin/bash
# export_all.sh - Export HEDL to multiple formats

INPUT="$1"
BASE="${INPUT%.hedl}"

hedl to-json "$INPUT" --pretty -o "${BASE}.json"
hedl to-yaml "$INPUT" -o "${BASE}.yaml"
hedl to-xml "$INPUT" --pretty -o "${BASE}.xml"
hedl to-csv "$INPUT" -o "${BASE}.csv"
hedl to-parquet "$INPUT" -o "${BASE}.parquet"

echo "Exported $INPUT to 5 formats"
```

### Error Handling

**Robust error handling:**
```bash
#!/bin/bash

convert_file() {
  local input="$1"
  local output="$2"

  if [ ! -f "$input" ]; then
    echo "Error: File not found: $input" >&2
    return 1
  fi

  if ! hedl validate "$input" 2>/dev/null; then
    echo "Error: Invalid HEDL: $input" >&2
    return 1
  fi

  if ! hedl to-json "$input" --pretty -o "$output"; then
    echo "Error: Conversion failed: $input -> $output" >&2
    return 1
  fi

  echo "Success: $input -> $output"
  return 0
}

# Process files
for file in *.hedl; do
  convert_file "$file" "${file%.hedl}.json" || true
done
```

### Performance Optimization

**Parallel processing:**
```bash
# Use GNU parallel for maximum throughput
ls *.json | parallel -j 8 'hedl from-json {} -o {.}.hedl'

# Batch commands use parallelism by default
hedl batch-format *.hedl --output-dir formatted/  # Already parallel

# Control parallelism via RAYON_NUM_THREADS
export RAYON_NUM_THREADS=4
hedl batch-validate "**/*.hedl"
```

**Memory optimization:**
```bash
# Process large files with streaming (future feature)
# Current: Adjust max file size
export HEDL_MAX_FILE_SIZE=5368709120  # 5GB

# Split large files before processing
split -l 100000 huge.csv chunk_
for chunk in chunk_*; do
  hedl from-csv "$chunk" --headers >> combined.hedl
done
```

### Integration Examples

**Git Pre-commit Hook:**
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Validating HEDL files..."
if ! hedl batch-validate data/*.hedl --parallel; then
  echo "❌ Commit rejected: Invalid HEDL files" >&2
  exit 1
fi

echo "✅ All HEDL files valid"
```

**CI/CD Pipeline (GitHub Actions):**
```yaml
name: Validate HEDL

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install HEDL
        run: cargo install hedl-cli
      - name: Validate
        run: hedl batch-validate **/*.hedl
      - name: Lint
        run: hedl batch-lint **/*.hedl
```

**Data Processing Pipeline:**
```bash
#!/bin/bash
# ETL pipeline: CSV → HEDL → Validation → Parquet

set -euo pipefail

INPUT_DIR="input"
OUTPUT_DIR="output"
HEDL_DIR="temp/hedl"

mkdir -p "$HEDL_DIR" "$OUTPUT_DIR"

# Extract: CSV to HEDL
echo "Converting CSV to HEDL..."
for csv in "$INPUT_DIR"/*.csv; do
  base=$(basename "$csv" .csv)
  hedl from-csv "$csv" --headers -o "$HEDL_DIR/${base}.hedl"
done

# Transform: Validate and lint
echo "Validating HEDL files..."
hedl batch-validate $HEDL_DIR/*.hedl
hedl batch-lint $HEDL_DIR/*.hedl

# Load: Convert to Parquet
echo "Converting to Parquet..."
for hedl in "$HEDL_DIR"/*.hedl; do
  base=$(basename "$hedl" .hedl)
  hedl to-parquet "$hedl" -o "$OUTPUT_DIR/${base}.parquet"
done

echo "Pipeline complete!"
```

---

**Need help?** Use `hedl <command> --help` for detailed command information, or check the [Troubleshooting Guide](troubleshooting.md) for common issues.
