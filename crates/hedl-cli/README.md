# hedl-cli

**The complete command-line toolkit for HEDL.**

HEDL files are more compact than JSON, more readable than YAML, and more structured than CSV. But a file format is only as good as the tools that surround it. `hedl-cli` gives you everything you need to work with HEDL at scale: validation, formatting, linting, structure inspection, format conversion, and batch processing with parallel execution.

Whether you're validating schemas in CI, converting database exports, analyzing token efficiency for LLM contexts, or processing thousands of configuration files, this is your tool.

## Installation

```bash
# From source
cargo install hedl-cli

# Or build locally
cd crates/hedl-cli
cargo build --release
```

The binary is named `hedl` and lives at `target/release/hedl`.

## Core Commands

### validate

Check that your HEDL files are syntactically correct and structurally sound.

```bash
hedl validate config.hedl
hedl validate --strict api_schema.hedl
```

The `--strict` flag ensures all entity references resolve to defined entities. Without it, forward references are allowed. Exit code 0 means valid; exit code 1 means something's wrong.

Output looks like:
```
✓ config.hedl
  Version: 1.0
  Structs: 5
  Aliases: 2
  Nests: 3
```

### format

Normalize HEDL files to canonical form.

```bash
hedl format data.hedl                    # Output to stdout
hedl format data.hedl -o formatted.hedl  # Output to file
hedl format --check config.hedl          # Check without writing
hedl format --with-counts users.hedl     # Add count hints to matrix lists
```

The `--check` flag is perfect for CI: it exits with code 1 if the file isn't already canonical, without modifying anything. The `--with-counts` flag recursively adds count hints to all matrix lists, showing you exactly how many rows each contains.

### lint

Check your files against best practices.

```bash
hedl lint schema.hedl
hedl lint --format json config.hedl
hedl lint --warn-error critical.hedl
```

Text output with colors:
```
Warning [unused-alias]: Alias 'old_api' is defined but never used
  at line 15

Suggestion [add-count-hints]: Matrix list 'users' is missing count hint
  at line 42

Found 1 warning, 1 suggestion
```

JSON output for programmatic processing:
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

Use `--warn-error` to fail the build on warnings, not just errors.

### inspect

Visualize the structure of a HEDL document as a tree.

```bash
hedl inspect data.hedl
hedl inspect -v schema.hedl  # Verbose: show field values
```

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

With `-v`, you get the actual data:
```
└─ users: @User (3 entities)
   ├─ alice [Alice Smith, alice@example.com, 2024-01-15]
   ├─ bob [Bob Jones, bob@example.com, 2024-02-20]
   └─ carol [Carol White, carol@example.com, 2024-03-10]
```

### stats

Compare HEDL file sizes and token counts against JSON, YAML, and XML.

```bash
hedl stats data.hedl
hedl stats --tokens config.hedl  # Include LLM token estimates
```

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

All format conversions run in parallel for maximum throughput.

## Format Conversion

Bidirectional conversion between HEDL and six popular formats: JSON, YAML, XML, CSV, Parquet, and TOON.

### JSON

```bash
hedl to-json data.hedl -o output.json
hedl to-json --pretty data.hedl          # Pretty-printed
hedl to-json --metadata schema.hedl      # Include HEDL version and schema info
hedl from-json input.json -o output.hedl
```

### YAML

```bash
hedl to-yaml config.hedl -o config.yml
hedl from-yaml config.yml -o config.hedl
```

### XML

```bash
hedl to-xml data.hedl -o output.xml
hedl to-xml --pretty data.hedl
hedl from-xml input.xml -o output.hedl
```

### CSV

```bash
hedl to-csv users.hedl -o users.csv
hedl from-csv --type-name User input.csv -o users.hedl
```

The `--type-name` flag specifies the entity type name when converting from CSV (defaults to "Row").

### Parquet

```bash
hedl to-parquet data.hedl --output output.parquet
hedl from-parquet input.parquet -o data.hedl
```

Parquet uses binary columnar format, so `--output` is required.

### TOON

```bash
hedl to-toon data.hedl -o output.toon
hedl from-toon input.toon -o data.hedl
```

TOON (Token-Oriented Object Notation) was designed for LLM efficiency, but accuracy testing shows HEDL achieves higher comprehension (+12.2pp average) with 7% fewer tokens.

## Batch Processing

Process hundreds or thousands of files in parallel with automatic parallelization and progress tracking.

### batch-validate

```bash
hedl batch-validate data/*.hedl
hedl batch-validate --strict schemas/*.hedl
hedl batch-validate -v configs/*.hedl              # Verbose progress
hedl batch-validate --streaming large-files/*.hedl # Constant memory for huge files
hedl batch-validate --auto-streaming mixed/*.hedl  # Auto-detect when to stream
hedl batch-validate --max-files 5000 huge-dir/*.hedl
```

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

Parallelization kicks in automatically when processing 10+ files. Expect 3-5x speedup on multi-core systems.

### batch-format

```bash
hedl batch-format data/*.hedl
hedl batch-format configs/*.hedl --output-dir formatted/
hedl batch-format --with-counts schemas/*.hedl
```

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

### batch-lint

```bash
hedl batch-lint data/*.hedl
hedl batch-lint --warn-error schemas/*.hedl
hedl batch-lint -v configs/*.hedl
```

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

Generate completion scripts for your shell.

```bash
hedl completion bash > ~/.hedl-completion.bash
hedl completion zsh > ~/.hedl-completion.zsh
hedl completion fish > ~/.config/fish/completions/hedl.fish
hedl completion powershell
hedl completion elvish
```

Add `source ~/.hedl-completion.bash` to your `.bashrc`, then enjoy tab completion for all commands, subcommands, and options.

```bash
hedl <TAB>           # Shows all commands
hedl batch-<TAB>     # Shows batch-validate, batch-format, batch-lint
hedl validate --<TAB> # Shows --strict option
```

## Security

### File Size Limits

By default, `hedl-cli` refuses to process files larger than 1 GB to prevent memory exhaustion. Configure this via environment variable:

```bash
export HEDL_MAX_FILE_SIZE=2147483648  # 2 GB
```

### Input Validation

Type names (for CSV conversion) accept only alphanumeric characters and underscores. All file paths are validated before processing. Batch operations continue processing other files when individual files fail.

### Error Context

Errors include detailed context to help you fix problems quickly:

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

## Performance

Validation runs at 100-200 MB/s. Formatting at 50-100 MB/s. Linting at 80-150 MB/s. Batch operations with parallelization see 3-5x speedup on multi-core systems.

Memory usage is O(document_size) per file since documents are fully parsed. For streaming large files (>100 MB), use the `--streaming` flag or the `hedl-stream` crate directly.

## License

Apache-2.0
