# Tutorial: CLI Basics

**Time:** 15 minutes | **Difficulty:** Beginner

Master the essential HEDL command-line tools and learn how to use them effectively in your daily workflow. This tutorial builds on what you learned in the first tutorial and introduces you to the full power of the HEDL CLI.

## What You'll Learn

- Core CLI commands and their options
- Reading from stdin and writing to stdout
- Using pipes and command chains
- Common workflow patterns
- Command shortcuts and best practices

## Prerequisites

- Completed [Tutorial 1: Your First Conversion](01-first-conversion.md)
- HEDL CLI installed
- Basic shell/terminal knowledge

## The HEDL CLI Philosophy

The HEDL CLI follows Unix philosophy:
- **Do one thing well** - Each command has a focused purpose
- **Work together** - Commands can be chained in pipelines
- **Text streams** - Input and output are text, enabling composition
- **Sensible defaults** - Common operations are simple

## Core Commands Overview

Let's explore the five essential commands you'll use every day:

```bash
validate    # Check syntax and structure
format      # Standardize formatting
lint        # Check for issues and best practices
inspect     # View internal structure
stats       # Compare format efficiency
```

## Step 1: Setting Up Sample Data

Create a sample HEDL file named `employees.hedl`:

```hedl
%VERSION: 1.0
%STRUCT: Employee: [id, name, department, salary, hired_date]
---
employees: @Employee
  | e1, Alice Johnson, Engineering, 95000, 2022-01-15
  | e2, Bob Smith, Engineering, 87000, 2022-03-20
  | e3, Carol White, Marketing, 72000, 2021-11-10
  | e4, David Brown, Sales, 68000, 2023-02-01
  | e5, Eve Davis, Engineering, 102000, 2020-06-15
  | e6, Frank Miller, Marketing, 71000, 2023-05-22
```

## Step 2: Validation Deep Dive

The `validate` command checks your HEDL file for errors.

### Basic Validation

```bash
hedl validate employees.hedl
```

**Output:**
```
✓ employees.hedl is valid
```

### What Validation Checks

Validation ensures:
1. **Syntax correctness** - Proper HEDL syntax
2. **Structure consistency** - Row lengths match column definitions
3. **Type compatibility** - Values match expected types
4. **Reference integrity** - All references point to existing entities
5. **Indentation rules** - Proper 2-space indentation

### Testing Validation

Let's create an invalid file to see validation in action. Create `invalid.hedl`:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob
```

Note: Row `u2` is missing the email field.

```bash
hedl validate invalid.hedl
```

**Output:**
```
✗ invalid.hedl is invalid
Error on line 4: Expected 3 values but found 2
  Row: u2 "Bob"
  Expected: id, name, email
```

**Key point:** Validation gives you clear error messages with line numbers and explanations.

### Validating from Stdin

You can validate content directly without a file:

```bash
echo '%VERSION: 1.0
---
name "Test"' | hedl validate -
```

The `-` tells HEDL to read from stdin instead of a file.

## Step 3: Formatting for Consistency

The `format` command applies HEDL's canonical formatting rules.

### Basic Formatting

```bash
hedl format employees.hedl
```

This outputs the formatted version to stdout. To save it:

```bash
hedl format employees.hedl -o employees_formatted.hedl
```

### In-Place Formatting

Format a file and overwrite it:

```bash
hedl format employees.hedl -o employees.hedl
```

**Warning:** This overwrites the original file. Use version control or backups!

### What Formatting Does

Formatting ensures:
- **Consistent indentation** - Exactly 2 spaces
- **Standardized spacing** - After colons, around values
- **Deterministic ordering** - Predictable field order
- **Canonical representation** - Same data always formatted identically

### Why Canonical Formatting Matters

Canonical formatting is crucial for:
- **Version control** - Reduces meaningless diffs
- **Reproducibility** - Same input always produces same output
- **Validation** - Easier to spot structural issues
- **Code review** - Consistent style across team

### Formatting in Pipelines

Combine format with other commands:

```bash
# Format and then validate
hedl format messy.hedl | hedl validate -

# Format and convert to JSON
hedl format data.hedl | hedl to-json - --pretty
```

## Step 4: Linting for Quality

The `lint` command checks for issues and suggests improvements.

```bash
hedl lint employees.hedl
```

### What Linting Checks

Linting identifies:
1. **Inefficient patterns** - Places where ditto could be used
2. **Naming conventions** - Inconsistent ID or field naming
3. **Data quality** - Suspicious patterns (duplicate values, etc.)
4. **Best practices** - HEDL usage recommendations
5. **Optimization opportunities** - More efficient representations

### Example Lint Issues

Create `needs_lint.hedl`:

```hedl
%VERSION: 1.0
%STRUCT: Task: [id, status, priority, assignee]
---
tasks: @Task
  | t1, pending, high, Alice
  | t2, pending, high, Alice
  | t3, pending, medium, Bob
  | t4, done, low, Alice
```

```bash
hedl lint needs_lint.hedl
```

**Possible output:**
```
needs_lint.hedl:
  Line 4: Consider using ditto (^) for repeated 'pending' value
  Line 4: Consider using ditto (^) for repeated 'high' value
  Line 4: Consider using ditto (^) for repeated 'Alice' value
  Line 5: Consider using ditto (^) for repeated 'pending' value

Suggestions:
  - Use the ditto operator to reduce redundancy
  - This would save 18 tokens
```

### Fixing Lint Issues

Here's the improved version using ditto:

```hedl
%VERSION: 1.0
%STRUCT: Task: [id, status, priority, assignee]
---
tasks: @Task
  | t1, pending, high, Alice
  | t2, ^, ^, ^
  | t3, ^, medium, Bob
  | t4, done, low, Alice
```

## Step 5: Inspecting Internal Structure

The `inspect` command shows you how HEDL interprets your document.

```bash
hedl inspect employees.hedl
```

**Output:**
```
Document Structure:
  Version: 1.0
  Entities: 1

Entity: employees
  Type: Employee
  Columns: [id, name, department, salary, hired_date]
  Row count: 6
  Rows:
    [0] e1: ["Alice Johnson", "Engineering", 95000, "2022-01-15"]
    [1] e2: ["Bob Smith", "Engineering", 87000, "2022-03-20"]
    [2] e3: ["Carol White", "Marketing", 72000, "2021-11-10"]
    [3] e4: ["David Brown", "Sales", 68000, "2023-02-01"]
    [4] e5: ["Eve Davis", "Engineering", 102000, "2020-06-15"]
    [5] e6: ["Frank Miller", "Marketing", 71000, "2023-05-22"]
```

### When to Use Inspect

Use `inspect` to:
- **Debug parsing issues** - See how HEDL interprets your data
- **Verify structure** - Confirm the data model matches expectations
- **Learn HEDL** - Understand how syntax maps to structure
- **Troubleshoot conversions** - See what data will be converted

## Step 6: Comparing Format Efficiency

The `stats` command compares HEDL to other formats.

```bash
hedl stats employees.hedl
```

**Output:**
```
HEDL Size Comparison
====================

Input: employees.hedl

Bytes:
  Format               Size     Savings          %
  -------------------- ---------- ------------ ----------
  HEDL                      312
  JSON (minified)           758        +446      58.9%
  JSON (pretty)             892        +580      65.0%
  YAML                      562        +250      44.5%
  XML (minified)           1024        +712      69.5%
  XML (pretty)             1156        +844      73.0%
```

### Understanding Stats Output

The stats command shows:
- **Size comparison** - Bytes for each format (minified and pretty versions)
- **Absolute savings** - How many bytes HEDL saves
- **Percentage differences** - How much larger other formats are

To include token estimates (for LLM context optimization), add the `--tokens` flag:

```bash
hedl stats employees.hedl --tokens
```

## Step 7: Working with Stdin and Stdout

HEDL commands are designed for Unix-style pipelines.

### Reading from Stdin

Use `-` as the filename to read from stdin:

```bash
cat employees.hedl | hedl validate -
```

### Writing to Stdout

By default, most commands write to stdout:

```bash
hedl format employees.hedl
```

Save the output with shell redirection:

```bash
hedl format employees.hedl > formatted.hedl
```

Or use the `-o` option:

```bash
hedl format employees.hedl -o formatted.hedl
```

### Chaining Commands

Build powerful pipelines:

```bash
# Format, validate, and convert in one pipeline
cat messy.hedl | hedl format - | hedl validate - | hedl to-json - --pretty
```

```bash
# Process multiple files
for file in *.hedl; do
  hedl format "$file" | hedl lint - || echo "Failed: $file"
done
```

```bash
# Convert and compress
hedl to-json data.hedl | gzip > data.json.gz
```

## Step 8: Common Workflow Patterns

### Pattern 1: Validate Before Processing

Always validate before conversion:

```bash
if hedl validate input.hedl; then
  hedl to-json input.hedl -o output.json
  echo "Conversion successful"
else
  echo "Validation failed. Fix errors first."
  exit 1
fi
```

### Pattern 2: Format + Lint + Validate

Create a quality check script:

```bash
#!/bin/bash
# check_quality.sh

hedl format "$1" -o temp.hedl
hedl lint temp.hedl
hedl validate temp.hedl
mv temp.hedl "$1"
```

Usage:

```bash
./check_quality.sh employees.hedl
```

### Pattern 3: Bulk Validation

Check all HEDL files in a directory:

```bash
#!/bin/bash
# validate_all.sh

for file in *.hedl; do
  if hedl validate "$file"; then
    echo "✓ $file"
  else
    echo "✗ $file"
  fi
done
```

### Pattern 4: Pre-Commit Hook

Use HEDL in a git pre-commit hook:

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Get all staged HEDL files
hedl_files=$(git diff --cached --name-only --diff-filter=ACM | grep '\.hedl$')

for file in $hedl_files; do
  if ! hedl validate "$file"; then
    echo "Error: $file is invalid. Commit aborted."
    exit 1
  fi

  # Auto-format staged files
  hedl format "$file" -o "$file"
  git add "$file"
done
```

## Step 9: Getting Help

### Command Help

Every command has built-in help:

```bash
# General help
hedl --help

# Command-specific help
hedl validate --help
hedl format --help
hedl lint --help
```

### Version Information

Check your HEDL version:

```bash
hedl --version
```

## Common Options Across Commands

Most HEDL commands support these common options:

| Option | Short | Description |
|--------|-------|-------------|
| `--output` | `-o` | Specify output file |
| `--help` | `-h` | Show command help |
| `--version` | `-V` | Show version |

## Exit Codes

HEDL commands use standard exit codes:

- `0` - Success
- `1` - Validation error, command failure, or invalid arguments

Use exit codes in scripts:

```bash
if hedl validate data.hedl; then
  echo "Valid!"
else
  echo "Invalid! Exit code: $?"
fi
```

## Best Practices

### 1. Always Validate First

Before any operation, validate your input:

```bash
hedl validate input.hedl && hedl to-json input.hedl -o output.json
```

### 2. Use Pipelines for Transformations

Chain commands for complex operations:

```bash
cat data.hedl | hedl format - | hedl lint - | hedl to-json - --pretty
```

### 3. Save Formatted Versions

Keep a canonically formatted version:

```bash
hedl format data.hedl -o data_canonical.hedl
```

### 4. Check Stats Before Conversion

Understand the efficiency trade-offs:

```bash
hedl stats data.hedl
```

### 5. Use Descriptive Output Names

Make output filenames clear:

```bash
hedl to-json employees.hedl -o employees.json
hedl format employees.hedl -o employees.formatted.hedl
```

## Practice Exercises

### Exercise 1: Quality Pipeline

Create a script that:
1. Formats a HEDL file
2. Lints it
3. Validates it
4. Shows stats
5. Converts to JSON if all checks pass

<details>
<summary>Solution</summary>

```bash
#!/bin/bash
# quality_pipeline.sh

FILE="$1"

echo "Formatting..."
hedl format "$FILE" -o temp.hedl

echo "Linting..."
hedl lint temp.hedl

echo "Validating..."
if hedl validate temp.hedl; then
  echo "Stats:"
  hedl stats temp.hedl

  echo "Converting to JSON..."
  hedl to-json temp.hedl --pretty -o "${FILE%.hedl}.json"

  mv temp.hedl "$FILE"
  echo "✓ Pipeline complete!"
else
  echo "✗ Validation failed"
  rm temp.hedl
  exit 1
fi
```
</details>

### Exercise 2: Batch Formatter

Write a script to format all `.hedl` files in a directory recursively.

### Exercise 3: Validation Report

Create a script that validates multiple HEDL files and generates a summary report showing which files passed/failed.

## Troubleshooting

### Command Not Found

```bash
hedl: command not found
```

**Solution:** Ensure HEDL is installed and in your PATH:

```bash
cargo install hedl-cli
```

### Permission Denied

```bash
Permission denied: employees.hedl
```

**Solution:** Check file permissions:

```bash
chmod 644 employees.hedl
```

### Invalid UTF-8

```bash
Error: Invalid UTF-8 in file
```

**Solution:** Ensure your file is UTF-8 encoded:

```bash
file employees.hedl
iconv -f ISO-8859-1 -t UTF-8 employees.hedl > employees_utf8.hedl
```

## Quick Reference

```bash
# Validation
hedl validate file.hedl
cat file.hedl | hedl validate -

# Formatting
hedl format file.hedl -o formatted.hedl
hedl format file.hedl | less

# Linting
hedl lint file.hedl

# Inspection
hedl inspect file.hedl

# Statistics
hedl stats file.hedl

# Pipeline example
hedl format data.hedl | hedl validate - && echo "OK"
```

## Next Steps

You've mastered the core CLI commands! Continue your learning:

- [Tutorial 3: Batch Processing](03-batch-processing.md) - Process multiple files efficiently
- [How-To: Handle Errors](../how-to/handle-errors.md) - Deal with validation errors
- [Reference: CLI Commands](../reference/cli-commands.md) - Complete command reference

---

**Questions?** Check the [FAQ](../faq.md) or [Troubleshooting](../troubleshooting.md) guides!
