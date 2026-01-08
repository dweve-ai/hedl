# Getting Started with HEDL

This guide will walk you through installing HEDL, understanding the basics, and performing your first conversions.

## Table of Contents

1. [Installation](#installation)
2. [Your First HEDL Document](#your-first-hedl-document)
3. [Basic Commands](#basic-commands)
4. [Understanding HEDL Syntax](#understanding-hedl-syntax)
5. [Converting Between Formats](#converting-between-formats)
6. [Next Steps](#next-steps)

## Installation

### Prerequisites

- Rust 1.70 or later (for building from source)
- Git (for cloning the repository)

### Install from Source

```bash
# Clone the repository
git clone https://github.com/dweve-ai/hedl
cd hedl

# Build and install the CLI
cargo install --path crates/hedl-cli

# Verify installation
hedl --version
```

### Install via Cargo

```bash
# Install directly from crates.io
cargo install hedl-cli

# Verify installation
hedl --version
```

### Enable Shell Completions (Optional)

Generate completion scripts for your shell:

```bash
# Bash
hedl completion bash > ~/.local/share/bash-completion/completions/hedl

# Zsh
hedl completion zsh > ~/.zfunc/_hedl

# Fish
hedl completion fish > ~/.config/fish/completions/hedl.fish

# PowerShell
hedl completion powershell > $PROFILE
```

## Your First HEDL Document

Let's create a simple HEDL document to manage a list of books.

### Step 1: Create the File

Create a file named `books.hedl`:

```hedl
%VERSION: 1.0
%STRUCT: Book: [id, title, author, year]
---
books: @Book
  | b1, The Rust Programming Language, Steve Klabnik, 2018
  | b2, Programming Rust, Jim Blandy, 2021
  | b3, Rust in Action, Tim McNamara, 2021
```

### Step 2: Validate the File

Check that the syntax is correct:

```bash
hedl validate books.hedl
```

If everything is correct, you'll see:
```
✓ books.hedl is valid
```

### Step 3: Convert to JSON

Convert your HEDL document to JSON:

```bash
hedl to-json books.hedl --pretty
```

Output:
```json
{
  "books": [
    {
      "id": "b1",
      "title": "The Rust Programming Language",
      "author": "Steve Klabnik",
      "year": 2018
    },
    {
      "id": "b2",
      "title": "Programming Rust",
      "author": "Jim Blandy",
      "year": 2021
    },
    {
      "id": "b3",
      "title": "Rust in Action",
      "author": "Tim McNamara",
      "year": 2021
    }
  ]
}
```

### Step 4: Save the Output

Save the JSON to a file:

```bash
hedl to-json books.hedl --pretty -o books.json
```

## Basic Commands

Here are the essential commands you'll use regularly:

### Validation

Check if a HEDL file is syntactically correct:

```bash
hedl validate myfile.hedl
```

### Formatting

Format a HEDL file to canonical form (standardized spacing, ordering):

```bash
# Format and print to stdout
hedl format myfile.hedl

# Format and save to a new file
hedl format myfile.hedl -o formatted.hedl

# Format in-place (overwrite original)
hedl format myfile.hedl -o myfile.hedl
```

### Linting

Check for best practices and potential issues:

```bash
hedl lint myfile.hedl
```

### Inspection

View the internal structure (useful for debugging):

```bash
hedl inspect myfile.hedl
```

### Statistics

See how HEDL compares to other formats in terms of size and tokens:

```bash
hedl stats myfile.hedl
```

Example output:
```
Format Comparison for myfile.hedl:
  HEDL:    245 bytes,  62 tokens (baseline)
  JSON:    512 bytes, 156 tokens (+109%, +94 tokens)
  YAML:    398 bytes, 118 tokens (+62%, +56 tokens)
  XML:     687 bytes, 203 tokens (+180%, +141 tokens)
```

## Understanding HEDL Syntax

### Basic Structure

Every HEDL document starts with a version header:

```hedl
%VERSION: 1.0
---
```

### Simple Values

```hedl
%VERSION: 1.0
---
name: Alice
age: 30
active: true
score: 95.5
```

### Lists

```hedl
%VERSION: 1.0
---
colors: 3
  red
  green
  blue
```

### Typed Entities (Matrix Lists)

Matrix lists define a structure once and reuse it:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, bob@example.com
```

This is equivalent to:
```json
{
  "users": [
    {"id": "u1", "name": "Alice", "email": "alice@example.com"},
    {"id": "u2", "name": "Bob", "email": "bob@example.com"}
  ]
}
```

### Nested Structures

```hedl
%VERSION: 1.0
%STRUCT: Employee: [id, name]
---
company: @Company
  name: Acme Corp
  employees: @Employee
    | e1, Alice
    | e2, Bob
```

### References

Reference other entities using `@Type:id` syntax:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author, title]
---
users: @User
  | u1, Alice
  | u2, Bob

posts: @Post
  | p1, @User:u1, My First Post
```

### Ditto Operator

Repeat the previous value with `^`:

```hedl
%VERSION: 1.0
%STRUCT: Task: [id, status, priority]
---
tasks: @Task
  | t1, pending, high
  | t2, pending, ^      # priority = "high" (same as previous row)
  | t3, ^, low  # status = "pending", priority = "low"
```

## Converting Between Formats

### JSON Conversion

**HEDL to JSON:**
```bash
# Compact JSON
hedl to-json data.hedl

# Pretty-printed JSON
hedl to-json data.hedl --pretty

# Include HEDL metadata
hedl to-json data.hedl --metadata --pretty
```

**JSON to HEDL:**
```bash
# Basic conversion
hedl from-json data.json

# Save to file
hedl from-json data.json -o data.hedl
```

### YAML Conversion

**HEDL to YAML:**
```bash
hedl to-yaml data.hedl -o data.yaml
```

**YAML to HEDL:**
```bash
hedl from-yaml data.yaml -o data.hedl
```

### XML Conversion

**HEDL to XML:**
```bash
# Compact XML
hedl to-xml data.hedl

# Pretty-printed XML
hedl to-xml data.hedl --pretty
```

**XML to HEDL:**
```bash
hedl from-xml data.xml -o data.hedl
```

### CSV Conversion

**HEDL to CSV:**
```bash
# Convert structured data to CSV
hedl to-csv data.hedl
```

**CSV to HEDL:**
```bash
# Import CSV (first row is treated as headers containing column names)
# Must specify a type name with -t flag
hedl from-csv data.csv -t Record

# Specify custom type name for the matrix list
hedl from-csv data.csv -t User
```

### Parquet Conversion

**HEDL to Parquet:**
```bash
hedl to-parquet data.hedl -o data.parquet
```

**Parquet to HEDL:**
```bash
hedl from-parquet data.parquet -o data.hedl
```

## Practical Examples

### Example 1: Converting a JSON API Response

Suppose you have a JSON file from an API:

```bash
# Download JSON data
curl https://api.example.com/users > users.json

# Convert to HEDL (more compact for storage/LLM processing)
hedl from-json users.json -o users.hedl

# Validate the conversion
hedl validate users.hedl

# View statistics
hedl stats users.hedl
```

### Example 2: Processing CSV Data

```bash
# Import CSV data (first row is treated as headers, specify type name)
hedl from-csv customers.csv -t Customer -o customers.hedl

# Lint the data for issues
hedl lint customers.hedl

# Export to Parquet for analytics
hedl to-parquet customers.hedl -o customers.parquet
```

### Example 3: Batch Processing

```bash
# Validate all HEDL files in a directory
hedl batch-validate data/*.hedl --parallel

# Format all HEDL files
hedl batch-format data/*.hedl --output-dir formatted/ --parallel

# Convert all JSON files to HEDL
for file in *.json; do
  hedl from-json "$file" -o "${file%.json}.hedl"
done
```

### Example 4: Pipeline Processing

Use HEDL in Unix pipelines:

```bash
# Convert JSON to HEDL to YAML
hedl from-json data.json | hedl to-yaml - -o data.yaml

# Format and validate in one go
hedl format data.hedl | hedl validate -

# Get statistics for multiple files
for f in *.hedl; do echo "$f:"; hedl stats "$f"; done
```

## Configuration

### Environment Variables

**HEDL_MAX_FILE_SIZE**: Maximum file size to process (default: 1GB)

```bash
# Process larger files (5GB limit)
export HEDL_MAX_FILE_SIZE=5368709120

# Process files
hedl validate large_file.hedl
```

### Resource Limits

HEDL has built-in security limits to prevent DoS attacks:

- Maximum nesting depth: 50 levels
- Maximum string length: 1MB (lines), 10MB (blocks)
- Maximum file size: 1GB (configurable)

These limits can be customized when using HEDL as a library.

## Common Workflows

### Workflow 1: Data Validation Pipeline

```bash
#!/bin/bash
# validate_data.sh - Validate and format data files

for file in data/*.hedl; do
  echo "Processing $file..."

  # Validate
  if hedl validate "$file"; then
    # Format
    hedl format "$file" -o "${file%.hedl}_formatted.hedl"
    # Lint
    hedl lint "$file"
  else
    echo "Error: $file is invalid"
    exit 1
  fi
done
```

### Workflow 2: Format Conversion Service

```bash
#!/bin/bash
# convert_service.sh - Convert between formats

input_file="$1"
output_format="$2"

# Get input format from extension
input_ext="${input_file##*.}"

# Convert to HEDL first
case "$input_ext" in
  json) hedl from-json "$input_file" > temp.hedl ;;
  yaml) hedl from-yaml "$input_file" > temp.hedl ;;
  csv)  hedl from-csv "$input_file" -t Record > temp.hedl ;;
  *)    cp "$input_file" temp.hedl ;;
esac

# Convert to output format
case "$output_format" in
  json)    hedl to-json temp.hedl --pretty ;;
  yaml)    hedl to-yaml temp.hedl ;;
  xml)     hedl to-xml temp.hedl --pretty ;;
  parquet) hedl to-parquet temp.hedl -o output.parquet ;;
esac

rm temp.hedl
```

## Next Steps

Now that you understand the basics:

1. **Explore Examples**: Check out the [Examples](examples.md) guide for more use cases
2. **Learn All Commands**: Read the [CLI Guide](cli-guide.md) for comprehensive command reference
3. **Understand Formats**: See the [Formats Guide](formats.md) for detailed format conversion information
4. **Troubleshooting**: Visit [Troubleshooting](troubleshooting.md) if you encounter issues

## Quick Reference

### Most Used Commands

```bash
# Validation
hedl validate file.hedl

# Conversion
hedl to-json file.hedl --pretty
hedl from-json file.json -o file.hedl

# Formatting
hedl format file.hedl -o formatted.hedl

# Batch Processing
hedl batch-validate *.hedl --parallel
```

### Common Flags

- `-o, --output <FILE>`: Specify output file
- `--pretty`: Pretty-print output (JSON, XML)
- `--parallel`: Use parallel processing (batch commands)
- `--output-dir`: Output directory (batch-format)
- `--help`: Show help for any command

---

**Need help?** Check the [FAQ](faq.md) or [Troubleshooting](troubleshooting.md) guides!
