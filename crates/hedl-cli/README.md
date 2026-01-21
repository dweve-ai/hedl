# hedl-cli

**Complete HEDL toolkit—validation, formatting, linting, inspection, conversion, and batch processing with parallel execution.**

You need to validate HEDL files, convert between formats, analyze structure, or process hundreds of files in parallel. `hedl-cli` provides 21 commands covering the entire HEDL workflow: core operations (validate, format, lint, inspect, stats), bidirectional conversion to 6 formats (JSON, YAML, XML, CSV, Parquet, TOON), batch processing with automatic parallelization, and shell completion generation.

This is the official command-line interface for the HEDL ecosystem. Whether you're validating configuration files, converting database exports, analyzing token efficiency, or processing directories of HEDL documents—`hedl-cli` provides the tools you need.

## What's Implemented

Complete command-line toolkit with 21 commands across 4 categories:

1. **Core Commands (5)**: Validate, format, lint, inspect, stats
2. **Format Conversion (12)**: Bidirectional conversion for JSON, YAML, XML, CSV, Parquet, TOON
3. **Batch Processing (3)**: Parallel validation, formatting, linting with progress tracking
4. **Utilities (1)**: Shell completion generation for 5 shells
5. **High-Performance Architecture**: Parallel processing, colored output, structured errors
6. **Security Features**: File size limits (1 GB default), input validation, safe error propagation
7. **Flexible Output**: File or stdout, JSON or text, pretty or compact formatting

## Installation

```bash
# From source
cargo install hedl-cli

# Or build locally
cd crates/hedl-cli
cargo build --release
```

Binary location: `target/release/hedl`

## Core Commands

### validate - Syntax and Structure Validation

Validate HEDL files with optional strict reference checking:

```bash
# Basic validation
hedl validate config.hedl

# Strict mode (all references must resolve)
hedl validate --strict api_schema.hedl
```

**Output**:
```
✓ config.hedl
  Version: 1.0
  Structs: 5
  Aliases: 2
  Nests: 3
```

**Options**:
- `--strict` - Enforce all entity references must resolve to defined entities

**Exit Codes**: 0 (valid), 1 (parse errors or validation failures)

### format - Canonical Formatting

Normalize HEDL files to canonical form with optional optimizations:

```bash
# Format to stdout
hedl format data.hedl

# Format to file
hedl format data.hedl -o formatted.hedl

# Check if already canonical (no changes)
hedl format --check config.hedl

# Disable ditto optimization (keep repeated values explicit)
hedl format --ditto=false data.hedl

# Add count hints to all matrix lists
hedl format --with-counts users.hedl
```

**Options**:
- `-o, --output <FILE>` - Write to file instead of stdout
- `--check` - Only check if canonical, don't write output
- `--ditto` - Enable ditto operator optimization for repeated values (default: enabled)
- `--with-counts` - Recursively add count hints to matrix lists

**Exit Codes**: 0 (success or already canonical), 1 (parse error or check failed)

### lint - Best Practices Checking

Check HEDL files against best practices with configurable severity:

```bash
# Text output with colors
hedl lint schema.hedl

# JSON output for programmatic processing
hedl lint --format json config.hedl

# Treat warnings as errors
hedl lint --warn-error critical.hedl
```

**Output** (text format with colors):
```
Warning [unused-alias]: Alias 'old_api' is defined but never used
  at line 15

Suggestion [add-count-hints]: Matrix list 'users' is missing count hint
  at line 42

Found 1 warning, 1 suggestion
```

**Output** (JSON format):
```json
{
  "issues": [
    {
      "severity": "warning",
      "rule": "unused-alias",
      "message": "Alias 'old_api' is defined but never used",
      "line": 15
    }
  ],
  "summary": {
    "errors": 0,
    "warnings": 1,
    "suggestions": 1
  }
}
```

**Options**:
- `-f, --format <text|json>` - Output format (default: text with colors)
- `-W, --warn-error` - Treat warnings as errors

**Exit Codes**: 0 (no issues), 1 (has errors or warnings when --warn-error enabled)

### inspect - Structure Visualization

Display HEDL file structure as an interactive tree:

```bash
# Basic tree view
hedl inspect data.hedl

# Verbose mode (show field values and row data)
hedl inspect -v schema.hedl
```

**Output** (tree format):
```
Document (1.0)
├─ Schemas (3)
│  ├─ User [id, name, email, created_at]
│  ├─ Post [id, author, title, content]
│  └─ Comment [id, post, author, text]
├─ Aliases (2)
│  ├─ $api_url = "https://api.example.com"
│  └─ $version = "2.1.0"
├─ Nests (1)
│  └─ Post > Comment
└─ Data
   ├─ users: @User (125 entities)
   ├─ posts: @Post (48 entities)
   └─ comments: @Comment (312 entities)
```

**Verbose Output** (shows actual data):
```
└─ users: @User (3 entities)
   ├─ alice [Alice Smith, alice@example.com, 2024-01-15]
   ├─ bob [Bob Jones, bob@example.com, 2024-02-20]
   └─ carol [Carol White, carol@example.com, 2024-03-10]
```

**Options**:
- `-v, --verbose` - Show detailed field values and row data

### stats - Format Comparison Analysis

Compare HEDL file size and token counts vs JSON, YAML, XML:

```bash
# Byte counts only
hedl stats data.hedl

# Include LLM token estimates
hedl stats --tokens config.hedl
```

**Output**:
```
Format Comparison for 'data.hedl':

File Sizes:
  HEDL:         2,458 bytes
  JSON (compact): 3,841 bytes  (+56.3%)
  JSON (pretty):  5,219 bytes  (+112.3%)
  YAML:         4,105 bytes  (+67.0%)
  XML:          6,732 bytes  (+173.9%)

Token Estimates (LLM ~4 chars/token):
  HEDL:         615 tokens
  JSON (compact): 960 tokens   (+56.1%)
  JSON (pretty):  1,305 tokens (+112.2%)
  YAML:         1,026 tokens  (+66.8%)
  XML:          1,683 tokens  (+173.7%)

Conclusion: HEDL saves 345 tokens (36%) vs JSON compact, 690 tokens (53%) vs JSON pretty
```

**Options**:
- `--tokens` - Include LLM token count estimates (~4 chars/token heuristic)

**Performance**: All format conversions run in parallel using Rayon for maximum throughput.

## Format Conversion Commands

Bidirectional conversion between HEDL and 6 popular formats.

### JSON Conversion

```bash
# HEDL → JSON (compact)
hedl to-json data.hedl -o output.json

# HEDL → JSON (pretty-printed)
hedl to-json --pretty data.hedl

# HEDL → JSON (with metadata)
hedl to-json --metadata schema.hedl

# JSON → HEDL
hedl from-json input.json -o output.hedl
```

**to-json Options**:
- `-o, --output <FILE>` - Write to file
- `--pretty` - Pretty-print with indentation
- `--metadata` - Include HEDL version and schema information

### YAML Conversion

```bash
# HEDL → YAML
hedl to-yaml config.hedl -o config.yml

# YAML → HEDL
hedl from-yaml config.yml -o config.hedl
```

### XML Conversion

```bash
# HEDL → XML (compact)
hedl to-xml data.hedl -o output.xml

# HEDL → XML (pretty-printed)
hedl to-xml --pretty data.hedl

# XML → HEDL
hedl from-xml input.xml -o output.hedl
```

**to-xml Options**:
- `--pretty` - Pretty-print with indentation

### CSV Conversion

```bash
# HEDL → CSV (includes headers by default)
hedl to-csv users.hedl -o users.csv

# CSV → HEDL (specify entity type name)
hedl from-csv --type-name User input.csv -o users.hedl
```

**to-csv Options**:
- `--headers` - Include column headers (default: true)

**from-csv Options**:
- `--type-name <NAME>` - Entity type name (default: "Row")

### Parquet Conversion

```bash
# HEDL → Parquet (columnar format)
hedl to-parquet data.hedl --output output.parquet

# Parquet → HEDL
hedl from-parquet input.parquet -o data.hedl
```

Note: `--output` is required (not optional like other commands) because Parquet uses binary columnar format.

### TOON Conversion

```bash
# HEDL → TOON
hedl to-toon data.hedl -o output.toon

# TOON → HEDL
hedl from-toon input.toon -o data.hedl
```

TOON (Token-Oriented Object Notation) is optimized for LLM efficiency but accuracy testing shows HEDL achieves higher comprehension (+3.4 points average) with 10% fewer tokens.

## Batch Processing Commands

Process multiple files in parallel with automatic parallelization and progress tracking.

### batch-validate - Parallel Validation

```bash
# Validate all .hedl files in directory
hedl batch-validate data/*.hedl

# Strict mode for all files
hedl batch-validate --strict schemas/*.hedl

# Verbose progress tracking
hedl batch-validate -v configs/*.hedl

# Force parallel processing
hedl batch-validate -p data/*.hedl

# Use streaming mode for large files (constant memory)
hedl batch-validate --streaming large-files/*.hedl

# Automatically use streaming for files > 100MB
hedl batch-validate --auto-streaming mixed-files/*.hedl

# Limit processing to 5000 files
hedl batch-validate --max-files 5000 huge-directory/*.hedl
```

**Options**:
- `--strict` - Enforce reference resolution for all files
- `-v, --verbose` - Detailed progress output
- `-p, --parallel` - Force parallel processing (default: auto-detect based on file count)
- `--streaming` - Use streaming mode for memory-efficient processing (constant memory, ideal for files >100MB)
- `--auto-streaming` - Automatically use streaming for large files (>100MB) and standard mode for smaller files
- `--max-files <N>` - Maximum number of files to process (default: 10,000, set to 0 for unlimited)

**Output**:
```
Validating 127 files...
Progress: [========================================] 127/127 (100%)
Completed in 2.3s (55 files/sec)

Results:
  Valid: 125 files
  Failed: 2 files
    - data/broken.hedl: Parse error at line 42: unexpected token
    - schemas/old.hedl: Unresolved reference @User:nonexistent
```

**Performance**: Automatic parallelization when file count ≥ 10 (configurable), ~3-5x speedup on multi-core systems.

### batch-format - Parallel Formatting

```bash
# Format all files in-place
hedl batch-format data/*.hedl

# Format to output directory
hedl batch-format configs/*.hedl --output-dir formatted/

# Format with ditto optimization
hedl batch-format --ditto data/*.hedl

# Add count hints to all files
hedl batch-format --with-counts schemas/*.hedl

# Limit processing to 5000 files
hedl batch-format --max-files 5000 huge-directory/*.hedl
```

**Options**:
- `--output-dir <DIR>` - Write formatted files to directory (preserves relative paths)
- `--ditto` - Enable ditto optimization (default: true)
- `--with-counts` - Add count hints to matrix lists
- `-v, --verbose` - Detailed progress output
- `-p, --parallel` - Force parallel processing
- `--max-files <N>` - Maximum number of files to process (default: 10,000, set to 0 for unlimited)

**Output**:
```
Formatting 89 files...
Progress: [========================================] 89/89 (100%)
Completed in 1.8s (49 files/sec)

Results:
  Formatted: 87 files
  Unchanged: 0 files (already canonical)
  Failed: 2 files
    - data/corrupt.hedl: Parse error at line 15
```

### batch-lint - Parallel Linting

```bash
# Lint all files with aggregated results
hedl batch-lint data/*.hedl

# Treat warnings as errors
hedl batch-lint --warn-error schemas/*.hedl

# Verbose per-file results
hedl batch-lint -v configs/*.hedl

# Limit processing to 5000 files
hedl batch-lint --max-files 5000 huge-directory/*.hedl
```

**Options**:
- `--warn-error` - Treat warnings as errors
- `-v, --verbose` - Show issues for each file
- `-p, --parallel` - Force parallel processing
- `--max-files <N>` - Maximum number of files to process (default: 10,000, set to 0 for unlimited)

**Output**:
```
Linting 64 files...
Progress: [========================================] 64/64 (100%)
Completed in 1.1s (58 files/sec)

Aggregated Results:
  Errors: 3 across 2 files
  Warnings: 12 across 8 files
  Suggestions: 25 across 19 files

Top Issues:
  - unused-alias (8 occurrences)
  - add-count-hints (7 occurrences)
  - unresolved-reference (3 occurrences)

Failed Files:
  - schemas/old.hedl: 2 errors, 3 warnings
  - configs/broken.hedl: 1 error
```

## Shell Completion

Generate shell completion scripts for interactive usage:

```bash
# Generate for current shell
hedl completion bash > ~/.hedl-completion.bash
hedl completion zsh > ~/.hedl-completion.zsh
hedl completion fish > ~/.config/fish/completions/hedl.fish

# Supported shells
hedl completion bash      # Bash
hedl completion zsh       # Zsh
hedl completion fish      # Fish
hedl completion powershell # PowerShell
hedl completion elvish    # Elvish
```

**Installation** (bash example):
```bash
# Add to ~/.bashrc
source ~/.hedl-completion.bash
```

After installation, tab completion works for all commands, subcommands, and options:
```bash
hedl <TAB>           # Shows all commands
hedl batch-<TAB>     # Shows batch-validate, batch-format, batch-lint
hedl validate --<TAB> # Shows --strict option
```

## Security Features

### File Size Limits

Prevents memory exhaustion from malicious or unexpected large files:

```bash
# Default: 1 GB limit
hedl validate huge_file.hedl
# Error: File size (1.2 GB) exceeds limit (1 GB)

# Configure via environment variable
export HEDL_MAX_FILE_SIZE=2147483648  # 2 GB
hedl validate huge_file.hedl
```

**Default Limit**: 1,073,741,824 bytes (1 GB)

### Input Validation

- **Type Names** (CSV conversion): Alphanumeric characters and underscores only
- **Path Safety**: All file operations validated before processing
- **Error Boundaries**: Continues batch processing on individual file errors

### Error Context

All errors include file paths and detailed context:

```
Error: Failed to parse 'data/broken.hedl'
  Parse error at line 42, column 15:
    unexpected token ']', expected field name

  Context:
    40 | users: @User[id, name]
    41 |   | alice, Alice Smith
    42 |   | bob, Bob Jones]
       |                     ^ here
```

## Architecture Features

### Parallel Processing

**BatchProcessor System**:
- Configurable parallelization threshold (default: 10 files)
- Automatic thread pool sizing
- Progress tracking with atomic counters (lock-free)
- Error resilience (collects all failures, continues processing)

**Performance**: ~3-5x speedup on multi-core systems for batch operations.

### Count Hints System

Recursively adds count hints to matrix lists and nested children:

```hedl
# Before formatting with --with-counts
users: @User[id, name]
  | alice, Alice
  | bob, Bob

# After formatting
users[2]: @User[id, name]
  | alice, Alice
  | bob, Bob
```

**Behavior**: Overwrites existing hints with actual counts from parsed document.

### Output Handling

- **Colored Console**: Uses `colored` crate for syntax highlighting and progress
- **Flexible Destinations**: File path or stdout (respects --output/-o)
- **Format Options**: JSON, text, compact, pretty-printed
- **Error Separation**: Errors always printed to stderr, output to stdout

### Error Types

Comprehensive error handling with 19 error variants:

- **Io** - File I/O errors with path context
- **FileTooLarge** - Size limit exceeded (configurable)
- **IoTimeout** - I/O operation timeout
- **Parse** - HEDL syntax errors with line/column
- **Canonicalization** - Canonicalization failures
- **JsonConversion** - JSON conversion errors
- **JsonFormat** - JSON serialization/deserialization errors
- **YamlConversion** - YAML conversion errors
- **XmlConversion** - XML conversion errors
- **CsvConversion** - CSV conversion errors
- **ParquetConversion** - Parquet conversion errors
- **LintErrors** - Linting errors found
- **NotCanonical** - File is not in canonical form
- **InvalidInput** - Input validation failures (type names, paths)
- **ThreadPoolError** - Parallel processing thread pool creation failure
- **GlobPattern** - Invalid glob pattern syntax
- **NoFilesMatched** - No files matched the provided patterns
- **DirectoryTraversal** - Directory traversal failures
- **ResourceExhaustion** - System resource exhaustion (file handles, memory)

All errors implement `std::error::Error`, `Display`, and `Clone` for detailed messages and parallel error handling.

## Use Cases

**Configuration Management**: Validate and lint HEDL configuration files in CI/CD pipelines, format for canonical diffs, convert to JSON/YAML for runtime.

**Data Pipeline Integration**: Convert CSV exports to HEDL for structured processing, validate schemas, transform data, export to Parquet for analytics.

**Schema Development**: Write HEDL schemas with instant validation feedback, lint for best practices, inspect structure, compare token efficiency vs JSON.

**Batch Processing**: Process directories of HEDL files in parallel (validation, formatting, linting), aggregate results, identify issues across large codebases.

**LLM Context Optimization**: Analyze token counts with `stats` command, convert JSON to HEDL for 40-60% token savings, validate compressed output.

**Database Export/Import**: Export databases to CSV, convert to HEDL with type inference, validate structure, transform with matrix operations, import to Neo4j via Cypher.

## What This Crate Doesn't Do

**Interactive Editing**: Not a REPL or interactive editor—use `hedl-lsp` with your favorite editor (VS Code, Neovim, Emacs) for interactive development.

**Language Server**: LSP functionality is in `hedl-lsp` crate—this CLI focuses on batch operations and one-off conversions.

**MCP Server**: Model Context Protocol server is in `hedl-mcp` crate—this CLI is for human-driven workflows and automation scripts.

**Data Transformation**: Provides format conversion and validation, not arbitrary data transformations—use HEDL's matrix query capabilities or convert to SQL/Cypher for complex transformations.

## Performance Characteristics

**Command Performance**:
- **validate**: O(n) parsing, ~100-200 MB/s throughput
- **format**: O(n) parse + canonicalization, ~50-100 MB/s
- **lint**: O(n) parse + validation rules, ~80-150 MB/s
- **stats**: Parallel format conversions, ~50-100 MB/s per format

**Batch Processing**: ~3-5x speedup with parallel execution on multi-core systems.

**Memory**: O(document_size) per file—loads entire document for parsing. For streaming large files (>100 MB), use `hedl-stream` crate directly.

Detailed performance benchmarks are available in the HEDL repository benchmark suite.

## Dependencies

- `hedl-core` 1.2 - HEDL parsing and data model
- `hedl-c14n` 1.2 - Canonicalization
- `hedl-lint` 1.2 - Best practices linting
- `hedl-json`, `hedl-yaml`, `hedl-xml`, `hedl-csv`, `hedl-parquet`, `hedl-toon` 1.2 - Format conversion
- `clap` 4.4 - CLI argument parsing
- `clap_complete` - Shell completion generation
- `colored` - Terminal coloring
- `rayon` - Parallel processing
- `serde_json` - JSON output formatting
- `thiserror` - Error type definitions

## License

Apache-2.0
