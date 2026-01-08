# CLI Commands Reference

Complete reference for all HEDL command-line interface commands.

## Command Overview

| Command | Purpose |
|---------|---------|
| `validate` | Validate HEDL syntax |
| `format` | Format to canonical form |
| `lint` | Check best practices |
| `inspect` | Show internal structure |
| `stats` | Compare format efficiency |
| `to-json` | Convert to JSON |
| `from-json` | Convert from JSON |
| `to-yaml` | Convert to YAML |
| `from-yaml` | Convert from YAML |
| `to-xml` | Convert to XML |
| `from-xml` | Convert from XML |
| `to-csv` | Convert to CSV |
| `from-csv` | Convert from CSV |
| `to-parquet` | Convert to Parquet |
| `from-parquet` | Convert from Parquet |
| `to-toon` | Convert to TOON format |
| `batch-validate` | Validate multiple files |
| `batch-format` | Format multiple files |
| `batch-lint` | Lint multiple files |
| `completion` | Generate shell completions |

---

## Core Commands

### `validate`

Validate HEDL file syntax and structure.

**Syntax:**
```bash
hedl validate [OPTIONS] <FILE>
```

**Parameters:**
- `<FILE>` - HEDL file to validate

**Options:**
- `-s, --strict` - Strict mode (fail on any error)

**Exit Codes:**
- `0` - File is valid
- `1` - File is invalid or error occurred

**Examples:**
```bash
# Basic validation
hedl validate data.hedl

# Strict mode
hedl validate data.hedl --strict
```

---

### `format`

Format HEDL file to canonical form.

**Syntax:**
```bash
hedl format [OPTIONS] <FILE>
```

**Parameters:**
- `<FILE>` - HEDL file to format

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)
- `-c, --check` - Check only (exit 1 if not canonical)
- `--ditto` - Use ditto optimization (default: true)
- `--with-counts` - Automatically add count hints to all matrix lists

**Examples:**
```bash
# Format to stdout
hedl format data.hedl

# Format to file
hedl format data.hedl -o formatted.hedl

# Check if already canonical
hedl format data.hedl --check

# Disable ditto optimization
hedl format data.hedl --ditto false

# Add count hints
hedl format data.hedl --with-counts -o output.hedl
```

---

### `lint`

Check HEDL file for best practices and potential issues.

**Syntax:**
```bash
hedl lint [OPTIONS] <FILE>
```

**Parameters:**
- `<FILE>` - HEDL file to lint

**Options:**
- `-f, --format <FORMAT>` - Output format: `text` or `json` (default: text)
- `-W, --warn-error` - Treat warnings as errors

**Examples:**
```bash
# Basic linting
hedl lint data.hedl

# JSON output
hedl lint data.hedl --format json

# Treat warnings as errors
hedl lint data.hedl -W

---

### `inspect`

Show internal structure of HEDL document (debug output).

**Syntax:**
```bash
hedl inspect [OPTIONS] <FILE>
```

**Parameters:**
- `<FILE>` - HEDL file to inspect

**Options:**
- `-v, --verbose` - Show detailed internal structure

**Examples:**
```bash
# Basic inspection
hedl inspect data.hedl

# Verbose output
hedl inspect data.hedl --verbose
```

---

### `stats`

Compare HEDL file size/tokens to other formats.

**Syntax:**
```bash
hedl stats [OPTIONS] <FILE>
```

**Parameters:**
- `<FILE>` - HEDL file to analyze

**Options:**
- `-t, --tokens` - Show estimated token counts for LLM context

**Examples:**
```bash
# Basic stats
hedl stats data.hedl

# With token estimates
hedl stats data.hedl --tokens
```

**Output:**
```
Format Comparison:
  HEDL:    245 bytes,  62 tokens (baseline)
  JSON:    512 bytes, 156 tokens (+109%)
  YAML:    398 bytes, 118 tokens (+62%)
  XML:     687 bytes, 203 tokens (+180%)
```

---

## Conversion Commands

### `to-json`

Convert HEDL to JSON.

**Syntax:**
```bash
hedl to-json [OPTIONS] <FILE>
```

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)
- `-p, --pretty` - Pretty-print JSON
- `--metadata` - Include HEDL metadata in JSON

**Examples:**
```bash
# Compact JSON to stdout
hedl to-json data.hedl

# Pretty JSON to file
hedl to-json data.hedl --pretty -o output.json

# With metadata
hedl to-json data.hedl --metadata --pretty
```

---

### `from-json`

Convert JSON to HEDL.

**Syntax:**
```bash
hedl from-json [OPTIONS] <FILE>
```

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)

**Examples:**
```bash
hedl from-json data.json -o data.hedl
```

---

### `to-yaml`

Convert HEDL to YAML.

**Syntax:**
```bash
hedl to-yaml [OPTIONS] <FILE>
```

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)

**Examples:**
```bash
hedl to-yaml data.hedl -o output.yaml
```

---

### `from-yaml`

Convert YAML to HEDL.

**Syntax:**
```bash
hedl from-yaml [OPTIONS] <FILE>
```

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)

**Examples:**
```bash
hedl from-yaml config.yaml -o config.hedl
```

---

### `to-xml`

Convert HEDL to XML.

**Syntax:**
```bash
hedl to-xml [OPTIONS] <FILE>
```

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)
- `-p, --pretty` - Pretty-print XML

**Examples:**
```bash
hedl to-xml data.hedl --pretty -o output.xml
```

---

### `from-xml`

Convert XML to HEDL.

**Syntax:**
```bash
hedl from-xml [OPTIONS] <FILE>
```

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)

**Examples:**
```bash
hedl from-xml data.xml -o data.hedl
```

---

### `to-csv`

Convert HEDL to CSV.

**Syntax:**
```bash
hedl to-csv [OPTIONS] <FILE>
```

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)
- `--headers` - Include header row (default: true)

**Examples:**
```bash
hedl to-csv data.hedl -o output.csv
```

---

### `from-csv`

Convert CSV to HEDL.

**Syntax:**
```bash
hedl from-csv [OPTIONS] <FILE>
```

**Parameters:**
- `<FILE>` - Input CSV file

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)
- `-t, --type-name <NAME>` - Type name for the matrix list (default: "Row")

**Note:** The first row of the CSV file is always treated as the header row containing column names.

**Examples:**
```bash
hedl from-csv data.csv -o data.hedl
hedl from-csv users.csv --type-name User -o users.hedl
```

---

### `to-parquet`

Convert HEDL to Apache Parquet.

**Syntax:**
```bash
hedl to-parquet [OPTIONS] <FILE>
```

**Options:**
- `-o, --output <FILE>` - Output Parquet file (required)

**Examples:**
```bash
hedl to-parquet data.hedl -o output.parquet
```

---

### `from-parquet`

Convert Parquet to HEDL.

**Syntax:**
```bash
hedl from-parquet [OPTIONS] <FILE>
```

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)

**Examples:**
```bash
hedl from-parquet data.parquet -o data.hedl
```

---

### `to-toon`

Convert HEDL to TOON (Type-Object Notation) format.

**Syntax:**
```bash
hedl to-toon [OPTIONS] <FILE>
```

**Parameters:**
- `<FILE>` - Input HEDL file

**Options:**
- `-o, --output <FILE>` - Output file (default: stdout)

**Examples:**
```bash
hedl to-toon data.hedl -o output.toon
```

---

## Batch Commands

### `batch-validate`

Validate multiple HEDL files.

**Syntax:**
```bash
hedl batch-validate [OPTIONS] <FILES>...
```

**Parameters:**
- `<FILES>...` - Input file paths (supports glob patterns)

**Options:**
- `-s, --strict` - Strict mode (fail on any error)
- `-p, --parallel` - Enable parallel processing
- `-v, --verbose` - Show verbose progress

**Examples:**
```bash
hedl batch-validate data/*.hedl
hedl batch-validate **/*.hedl --parallel
hedl batch-validate data/*.hedl --strict --verbose
```

---

### `batch-format`

Format multiple HEDL files.

**Syntax:**
```bash
hedl batch-format [OPTIONS] <FILES>...
```

**Parameters:**
- `<FILES>...` - Input file paths (supports glob patterns)

**Options:**
- `-o, --output-dir <DIR>` - Output directory for formatted files
- `-c, --check` - Check only (exit 1 if not canonical)
- `--ditto` - Use ditto optimization (default: true)
- `--with-counts` - Automatically add count hints
- `-p, --parallel` - Force parallel processing
- `-v, --verbose` - Show verbose progress

**Examples:**
```bash
hedl batch-format data/*.hedl --output-dir formatted/
hedl batch-format data/*.hedl --check --parallel
hedl batch-format data/*.hedl --with-counts -o formatted/
```

---

### `batch-lint`

Lint multiple HEDL files.

**Syntax:**
```bash
hedl batch-lint [OPTIONS] <FILES>...
```

**Parameters:**
- `<FILES>...` - Input file paths (supports glob patterns)

**Options:**
- `-W, --warn-error` - Treat warnings as errors
- `-p, --parallel` - Force parallel processing
- `-v, --verbose` - Show verbose progress

**Examples:**
```bash
hedl batch-lint data/*.hedl
hedl batch-lint **/*.hedl --parallel --verbose
hedl batch-lint data/*.hedl -W
```

---

## Utility Commands

### `completion`

Generate shell completion scripts.

**Syntax:**
```bash
hedl completion <SHELL>
```

**Parameters:**
- `<SHELL>` - Shell type: `bash`, `zsh`, `fish`, `powershell`

**Options:**
- `-i, --install` - Print installation instructions instead of generating script

**Examples:**
```bash
# Bash
hedl completion bash > ~/.local/share/bash-completion/completions/hedl

# Zsh
hedl completion zsh > ~/.zfunc/_hedl

# Fish
hedl completion fish > ~/.config/fish/completions/hedl.fish
```

---

## Global Options

These options work with all commands:

- `--help` - Show help
- `--version` - Show version

---

**Related:**
- [Configuration](configuration.md) - Environment variables
- [File Formats](file-formats.md) - Format details
