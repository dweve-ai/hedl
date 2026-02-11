# The HEDL Command Line: Your Swiss Army Knife

You've just finished writing a HEDL document. Now what?

Maybe you want to check if it's valid. Maybe you want to convert it to JSON for an API. Maybe you want to see how much smaller it is than the equivalent YAML. Maybe you want to format it so your team's diffs are clean.

The `hedl` command-line tool does all of this and more. It's the bridge between your HEDL files and everything else: other formats, other systems, other workflows.

This guide will take you through every command, every option, every pattern. By the end, you'll be able to wield the CLI like a pro, building pipelines that validate, transform, and export your data exactly where it needs to go.

---

## Getting Oriented

The CLI follows a simple pattern:

```bash
hedl <command> [options] <arguments>
```

Every command has `--help`:

```bash
hedl --help              # All commands
hedl validate --help     # Specific command help
hedl to-json --help      # Another command
```

And there's always `--version` if you need to check what you're running:

```bash
hedl --version
# hedl-cli 2.0.0
```

At its core, the CLI organizes around three kinds of operations:

**Core operations** work with HEDL files directly: validation, formatting, linting, inspection, statistics.

**Conversion operations** transform HEDL to and from other formats: JSON, YAML, XML, CSV, Parquet, TOON.

**Batch operations** process multiple files at once: batch-validate, batch-format, batch-lint.

Let's explore each category deeply, starting with the operations you'll use most often.

---

## Core Operations: Working with HEDL Files

These commands operate on HEDL files without converting them to other formats.

### Validation: Is My Document Correct?

The `validate` command checks that your HEDL file is syntactically correct and internally consistent.

```bash
hedl validate document.hedl
```

If the file is valid, you see:

```
✓ document.hedl is valid
```

If there's a problem, you get a detailed error:

```
Error: Parse error at line 15, column 8: expected ',' or end of row
  |
15|  |alice,Alice Chen,alice@example.com
  |                     ^
  = note: Did you mean to add a comma after "Chen"?
```

The error tells you exactly where the problem is and often suggests a fix.

**When to validate:**

Run validation often. After writing. Before committing. In your CI pipeline. Catching errors early saves debugging time later.

```bash
# Quick check while editing
hedl validate data.hedl

# Fail fast in scripts
hedl validate data.hedl || exit 1

# In a pre-commit hook
if ! hedl validate "$file"; then
  echo "Invalid HEDL: $file" >&2
  exit 1
fi
```

**Strict mode** makes validation even more rigorous:

```bash
hedl validate document.hedl --strict
```

In strict mode, any ambiguity or edge case that might be accepted normally becomes an error. Use this when you want to be extra sure your document is clean.

**What validation checks:**

Validation catches more than syntax errors. It verifies:

1. **Syntax correctness.** The document follows HEDL grammar.
2. **Schema compliance.** Every row in a matrix list has the right number of columns.
3. **Reference integrity.** Every `@id` reference points to an entity that exists.
4. **ID uniqueness.** No duplicate IDs within the same entity collection.
5. **Structure validity.** Nesting levels, indentation, required headers.

A valid document is one you can trust. All the pieces fit together. No broken links. No missing data.

---

### Formatting: One True Style

The `format` command transforms your HEDL into canonical form. Every developer's file looks the same after formatting.

```bash
# Format to stdout
hedl format document.hedl

# Format to a new file
hedl format document.hedl -o formatted.hedl

# Format in place (careful: overwrites the original)
hedl format document.hedl -o document.hedl
```

**What formatting does:**

Formatting applies HEDL's canonicalization rules:

- One-space indentation per level
- No spaces after commas
- Unix line endings (LF)
- No trailing whitespace
- Single trailing newline

**Before formatting:**

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
   |u1,Alice,alice@example.com
  |u2,Bob,bob@example.com
```

**After formatting:**

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |u1,Alice,alice@example.com
 |u2,Bob,bob@example.com
```

Clean, consistent, diffable.

**Checking without modifying:**

The `--check` flag verifies whether a file is already canonical without changing it:

```bash
hedl format document.hedl --check
```

If canonical: exit code 0, no output.
If not canonical: exit code 1, shows what would change.

This is perfect for CI:

```yaml
# In your GitHub Actions workflow
- name: Check formatting
  run: hedl format --check *.hedl
```

**Adding count hints:**

The `--with-counts` flag automatically adds count hints to matrix lists:

```bash
hedl format document.hedl --with-counts -o optimized.hedl
```

Count hints tell LLMs how many rows to expect, improving comprehension without requiring them to count:

```hedl
# Before
users:@User
 |u1,Alice
 |u2,Bob
 |u3,Carol

# After --with-counts
users:@User#3
 |u1,Alice
 |u2,Bob
 |u3,Carol
```

---

### Linting: Beyond Correctness

A document can be valid yet still have issues. Unused schemas. Inconsistent naming. Empty lists that might indicate a mistake.

The `lint` command catches these:

```bash
hedl lint document.hedl
```

Output:

```
Linting document.hedl...

Warning [unused-schema] (line 4): Schema 'Product' is defined but never used
  %S:Product:[sku,name,price]
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^
  Consider: Remove this schema or add a matrix list that uses it

Warning [short-id] (line 12): ID 'a' is very short
  |a,Alice Chen,alice@example.com
   ^
  Consider: Use a more descriptive ID like 'alice' or 'user_alice'

Info [empty-list] (line 20): Matrix list 'pending_orders' is empty
  This might be intentional, but verify this is expected.

3 issues found (0 errors, 2 warnings, 1 info)
```

**What the linter checks:**

The linter looks for patterns that suggest problems:

- **unused-schema**: A schema is declared but no entity uses it. Often a leftover from editing.
- **short-id**: IDs like "a" or "1" are valid but often indicate you meant something more descriptive.
- **empty-list**: An empty matrix list might be intentional (no orders yet) or might indicate missing data.
- **inconsistent-naming**: Mixing camelCase and snake_case in the same document.
- **unqualified-kv-ref**: A reference that could be ambiguous in certain contexts.

**Output formats:**

For CI integration, you might want JSON output:

```bash
hedl lint document.hedl --format json
```

Returns structured JSON you can parse programmatically:

```json
{
  "file": "document.hedl",
  "issues": [
    {
      "severity": "warning",
      "rule": "unused-schema",
      "line": 4,
      "message": "Schema 'Product' is defined but never used"
    }
  ]
}
```

**Failing on warnings:**

By default, lint exits 0 even with warnings. To be strict:

```bash
hedl lint document.hedl --warn-error
```

Now any warning causes exit code 1. Useful for enforcing clean documents in CI.

---

### Inspection: Looking Inside

The `inspect` command shows you the parsed structure of a HEDL document:

```bash
hedl inspect document.hedl
```

Output:

```
Document {
  version: (1, 3),
  null_symbol: "~",
  quote_char: '"',
  schemas: [
    Schema { name: "User", columns: ["id", "name", "email"] }
  ],
  root: Object {
    fields: [
      Field {
        key: "users",
        type_annotation: Some("User"),
        value: MatrixList [
          Row { values: ["u1", "Alice", "alice@example.com"] },
          Row { values: ["u2", "Bob", "bob@example.com"] }
        ]
      }
    ]
  }
}
```

**When to use inspection:**

Inspection is a debugging tool. Use it when:

- You want to see how the parser interprets your document
- You're debugging a complex structure
- You need to understand type inference decisions
- You're troubleshooting a conversion that produces unexpected results

For even more detail:

```bash
hedl inspect document.hedl --verbose
```

This shows internal span information, memory layout, and other implementation details.

---

### Statistics: Measuring Efficiency

The `stats` command shows you how HEDL compares to other formats:

```bash
hedl stats document.hedl --tokens
```

Output:

```
Format Comparison for 'document.hedl':

Sizes:
  HEDL:         2,458 bytes   (baseline)
  JSON:         3,841 bytes   (+56%)
  JSON pretty:  5,124 bytes   (+108%)
  YAML:         4,105 bytes   (+67%)
  XML:          6,287 bytes   (+156%)
  CSV:          1,987 bytes   (-19%)
  Parquet:        892 bytes   (-64%)

Token Estimates (for LLM context):
  HEDL:         615 tokens    (baseline)
  JSON:         960 tokens    (+56%, +345 tokens)
  JSON pretty:  1,281 tokens  (+108%, +666 tokens)
  YAML:         1,026 tokens  (+67%, +411 tokens)
  XML:          1,572 tokens  (+156%, +957 tokens)

Token Savings:
  vs JSON:      345 tokens saved per request
  vs YAML:      411 tokens saved per request
  vs XML:       957 tokens saved per request

Cost Impact (at $3/million tokens):
  Single request savings: ~$0.001
  1,000 requests/day:     ~$1.04/day
  Monthly (30K requests): ~$31.05
```

This command answers the question: "Is HEDL actually smaller?" Yes. Usually significantly.

**When to use stats:**

Use stats when:

- Justifying HEDL adoption to your team
- Estimating cost savings for LLM workloads
- Choosing between formats for a specific use case
- Understanding the token economics of your data

---

## Conversion Commands: Bridging Formats

HEDL is a hub. Data flows in from various sources, gets validated and structured in HEDL, then flows out to wherever it needs to go.

### To JSON and From JSON

JSON is everywhere. APIs, databases, configuration files. Converting between HEDL and JSON is probably your most common operation.

**HEDL to JSON:**

```bash
# Compact JSON to stdout
hedl to-json document.hedl

# Pretty-printed JSON
hedl to-json document.hedl --pretty

# Save to file
hedl to-json document.hedl --pretty -o output.json

# Include HEDL metadata (schemas, types)
hedl to-json document.hedl --metadata --pretty -o output.json
```

The `--pretty` flag adds indentation for readability. The `--metadata` flag preserves HEDL information so you can convert back without losing structure.

**JSON to HEDL:**

```bash
# Basic conversion
hedl from-json data.json

# Save to file
hedl from-json data.json -o data.hedl
```

When converting from JSON, HEDL analyzes the structure and infers schemas where possible. Arrays of objects with consistent fields become matrix lists. Nested structures become nested entities.

**Real-world example: API response compression**

```bash
# Fetch API data, convert to HEDL for LLM context
curl -s https://api.example.com/users | \
  hedl from-json - -o users.hedl

# Check the savings
hedl stats users.hedl --tokens
```

If that API returns 10,000 users, you just cut your token usage by 50%+.

---

### To YAML and From YAML

YAML is popular for configuration files. HEDL converts cleanly to and from YAML.

```bash
# HEDL to YAML
hedl to-yaml config.hedl -o config.yaml

# YAML to HEDL
hedl from-yaml config.yaml -o config.hedl
```

**Why convert YAML to HEDL?**

If your YAML configuration has repetitive structures (lists of servers, lists of rules), HEDL will be more compact. Plus, HEDL's validation catches errors that YAML's loose typing misses.

```bash
# Validate a YAML config by converting to HEDL and back
hedl from-yaml config.yaml | hedl validate -
# If validation passes, the config structure is sound
```

---

### To XML and From XML

XML is verbose, but some systems require it. HEDL can bridge the gap.

```bash
# HEDL to compact XML
hedl to-xml document.hedl

# HEDL to pretty XML
hedl to-xml document.hedl --pretty -o output.xml

# XML to HEDL
hedl from-xml data.xml -o data.hedl
```

**The XML size contrast is dramatic:**

```bash
hedl stats document.hedl --tokens
# XML is typically 150-200% larger than HEDL
```

If you're sending context to an LLM and the source is XML, converting to HEDL first is almost always worthwhile.

---

### To CSV and From CSV

CSV is the universal tabular format. Spreadsheets, databases, analytics tools all speak CSV.

**HEDL to CSV:**

```bash
# With headers (default)
hedl to-csv document.hedl -o output.csv

# Without headers
hedl to-csv document.hedl --no-headers -o output.csv
```

HEDL exports the first (or main) matrix list as CSV. The schema columns become CSV headers.

**CSV to HEDL:**

```bash
# Basic conversion (uses 'Row' as schema name)
hedl from-csv data.csv -o data.hedl

# Custom schema name
hedl from-csv users.csv -t User -o users.hedl
```

The `-t` (or `--type-name`) flag specifies what to call the schema. This matters for readability and for references if you later add relationships.

**CSV workflow example:**

```bash
# Get data from a spreadsheet, validate it, make it LLM-ready
hedl from-csv exported_data.csv -t DataRow -o data.hedl
hedl validate data.hedl
hedl stats data.hedl --tokens
```

---

### To Parquet and From Parquet

Apache Parquet is the columnar format for big data. Analytics platforms, data warehouses, Spark, DuckDB all use Parquet.

**HEDL to Parquet:**

```bash
# Parquet output requires a file path
hedl to-parquet document.hedl -o output.parquet
```

Parquet is binary, so you can't pipe it to stdout. Always specify an output file.

**Parquet to HEDL:**

```bash
# Convert back
hedl from-parquet data.parquet -o data.hedl

# Or to stdout for piping
hedl from-parquet data.parquet
```

**Why Parquet?**

Parquet is extremely efficient for columnar data. It's often 60-80% smaller than HEDL and has built-in compression. Use it for:

- Long-term storage
- Analytics workloads
- Data warehouse imports
- Archival

Use HEDL for:

- Human readability
- LLM context
- Validation and editing
- Version control

The typical flow is: source data → HEDL (validate, edit, review) → Parquet (store, analyze).

---

### To TOON and From TOON

TOON (Token-Oriented Object Notation) is another token-efficient format. HEDL can convert to and from it.

```bash
# HEDL to TOON
hedl to-toon document.hedl -o output.toon

# TOON to HEDL
hedl from-toon data.toon -o data.hedl
```

TOON has its own optimizations. Depending on your data shape, one might be more efficient than the other. Use `stats` to compare.

---

## Batch Operations: Processing at Scale

When you have more than a handful of files, batch commands become essential.

### Batch Validate

Validate every HEDL file in your project:

```bash
# All files in current directory
hedl batch-validate *.hedl

# Multiple directories
hedl batch-validate data/*.hedl config/*.hedl

# With verbose progress
hedl batch-validate data/*.hedl --verbose
```

Output:

```
Validating 47 files...

✓ data/users.hedl
✓ data/products.hedl
✗ data/orders.hedl: Parse error at line 23: unexpected token
✓ data/inventory.hedl
✓ config/settings.hedl
... (42 more)

Results: 46 valid, 1 invalid
Total time: 0.3s (156 files/second)
```

Batch validation runs files in parallel. On a multi-core machine, you'll process hundreds of files per second.

**Options:**

```bash
# Strict mode
hedl batch-validate *.hedl --strict

# Limit number of files (useful for testing)
hedl batch-validate *.hedl --max-files 10

# Explicit parallelism
hedl batch-validate *.hedl --parallel
```

**In CI pipelines:**

```yaml
- name: Validate all HEDL files
  run: hedl batch-validate **/*.hedl
```

Exit code is 0 only if all files pass. One failure fails the build.

---

### Batch Format

Format all files to canonical form:

```bash
# Check if files need formatting (dry run)
hedl batch-format *.hedl --check

# Format to an output directory
hedl batch-format src/*.hedl --output-dir formatted/

# Format with count hints
hedl batch-format *.hedl --output-dir formatted/ --with-counts
```

The `--check` flag is crucial for CI. It verifies formatting without modifying files:

```yaml
- name: Check formatting
  run: hedl batch-format **/*.hedl --check
```

If any file isn't canonical, the check fails, and developers know to run the formatter locally.

---

### Batch Lint

Run the linter across your entire codebase:

```bash
# Lint everything
hedl batch-lint *.hedl

# Verbose output
hedl batch-lint *.hedl --verbose

# Fail on warnings
hedl batch-lint *.hedl --warn-error
```

Output:

```
Linting 47 files...

data/users.hedl: ✓ No issues
data/products.hedl: 2 warnings
data/orders.hedl: ✓ No issues
config/settings.hedl: 1 info

Summary: 3 issues (0 errors, 2 warnings, 1 info) across 47 files
```

---

## Utility Commands

### Shell Completion

Type `hedl to-` and press Tab. The shell shows you `to-json`, `to-yaml`, `to-xml`, and more. This is shell completion, and it makes the CLI much faster to use.

**Generate completions for your shell:**

```bash
# See installation instructions
hedl completion bash --install

# Generate the script
hedl completion bash > ~/.local/share/bash-completion/completions/hedl

# For Zsh
hedl completion zsh > ~/.zfunc/_hedl

# For Fish
hedl completion fish > ~/.config/fish/completions/hedl.fish

# For PowerShell
hedl completion powershell >> $PROFILE

# For Elvish
hedl completion elvish > ~/.elvish/lib/hedl.elv
```

After installing, restart your shell or source the config file. Now Tab-completion works everywhere.

---

## Exit Codes and Scripting

HEDL commands use standard exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (parse error, invalid input, I/O error, check failed) |

This makes scripting straightforward:

```bash
#!/bin/bash
set -e  # Exit on any error

# Validate first
hedl validate data.hedl

# Only convert if valid
hedl to-json data.hedl --pretty -o output.json

echo "Done!"
```

**Conditional execution:**

```bash
# Run only if validation passes
hedl validate data.hedl && hedl to-json data.hedl -o output.json

# Handle failure explicitly
if ! hedl validate data.hedl; then
  echo "Validation failed!" >&2
  exit 1
fi
```

---

## Environment Variables

### HEDL_MAX_FILE_SIZE

Controls the maximum file size the CLI will process. Default is 1 GB.

```bash
# Allow 5 GB files
export HEDL_MAX_FILE_SIZE=5368709120

# Process a huge file
hedl validate huge_data.hedl

# One-time override
HEDL_MAX_FILE_SIZE=10737418240 hedl validate even_bigger.hedl
```

This limit is a safety measure. Processing a 100 GB file would exhaust memory. Set the limit based on your available RAM and expected file sizes.

### RAYON_NUM_THREADS

Controls parallelism for batch operations:

```bash
# Use 4 threads
export RAYON_NUM_THREADS=4
hedl batch-validate *.hedl

# Or inline
RAYON_NUM_THREADS=8 hedl batch-format *.hedl --output-dir formatted/
```

By default, batch operations use all available CPU cores.

---

## Real-World Workflows

Let's put it all together with complete workflow examples.

### Workflow 1: Daily Data Import

You receive CSV exports daily. Convert them to HEDL for validation, then to Parquet for the data warehouse.

```bash
#!/bin/bash
# daily_import.sh

set -euo pipefail

DATE=$(date +%Y-%m-%d)
INPUT_DIR="incoming"
HEDL_DIR="processed/hedl"
PARQUET_DIR="warehouse"

echo "[$DATE] Starting daily import..."

# Convert CSVs to HEDL
for csv in "$INPUT_DIR"/*.csv; do
  base=$(basename "$csv" .csv)
  echo "Converting $csv..."
  hedl from-csv "$csv" -t "${base^}" -o "$HEDL_DIR/${base}.hedl"
done

# Validate all HEDL files
echo "Validating..."
hedl batch-validate "$HEDL_DIR"/*.hedl --strict || {
  echo "Validation failed!" >&2
  exit 1
}

# Format for cleanliness
hedl batch-format "$HEDL_DIR"/*.hedl --output-dir "$HEDL_DIR/"

# Export to Parquet for warehouse
echo "Exporting to Parquet..."
for hedl in "$HEDL_DIR"/*.hedl; do
  base=$(basename "$hedl" .hedl)
  hedl to-parquet "$hedl" -o "$PARQUET_DIR/${base}_${DATE}.parquet"
done

echo "[$DATE] Import complete!"
```

### Workflow 2: Pre-Commit Quality Gate

Ensure all HEDL files are valid and formatted before committing.

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Find staged HEDL files
staged=$(git diff --cached --name-only --diff-filter=ACM | grep '\.hedl$' || true)

if [ -z "$staged" ]; then
  exit 0  # No HEDL files staged
fi

echo "Checking HEDL files..."

# Validate
if ! echo "$staged" | xargs hedl batch-validate; then
  echo "❌ Validation failed. Fix errors before committing." >&2
  exit 1
fi

# Check formatting
if ! echo "$staged" | xargs hedl batch-format --check; then
  echo "❌ Files not in canonical format." >&2
  echo "Run: hedl format <file> for each file and re-stage." >&2
  exit 1
fi

# Lint (warnings allowed, just report)
echo "$staged" | xargs hedl batch-lint

echo "✅ All HEDL files look good!"
```

### Workflow 3: Multi-Format Export

Export a single HEDL file to every format for different consumers.

```bash
#!/bin/bash
# export_all.sh <input.hedl>

INPUT="$1"
BASE="${INPUT%.hedl}"

hedl validate "$INPUT" || exit 1

echo "Exporting $INPUT to multiple formats..."

# JSON for APIs
hedl to-json "$INPUT" --pretty -o "${BASE}.json"

# YAML for configs
hedl to-yaml "$INPUT" -o "${BASE}.yaml"

# XML for legacy systems
hedl to-xml "$INPUT" --pretty -o "${BASE}.xml"

# CSV for spreadsheets
hedl to-csv "$INPUT" -o "${BASE}.csv"

# Parquet for analytics
hedl to-parquet "$INPUT" -o "${BASE}.parquet"

# TOON for comparison
hedl to-toon "$INPUT" -o "${BASE}.toon"

echo "Created 6 formats from $INPUT"
ls -la "${BASE}".*
```

### Workflow 4: LLM Context Optimization

Prepare data for LLM consumption with maximum efficiency.

```bash
#!/bin/bash
# optimize_for_llm.sh <input>

INPUT="$1"
OUTPUT="${INPUT%.json}.hedl"

echo "Optimizing $INPUT for LLM context..."

# Convert from JSON (or detect format)
if [[ "$INPUT" == *.json ]]; then
  hedl from-json "$INPUT" -o "$OUTPUT"
elif [[ "$INPUT" == *.yaml ]]; then
  hedl from-yaml "$INPUT" -o "$OUTPUT"
elif [[ "$INPUT" == *.csv ]]; then
  hedl from-csv "$INPUT" -t Data -o "$OUTPUT"
else
  echo "Unknown format: $INPUT" >&2
  exit 1
fi

# Format with count hints
hedl format "$OUTPUT" --with-counts -o "$OUTPUT"

# Show the savings
echo ""
echo "Optimization results:"
hedl stats "$OUTPUT" --tokens

echo ""
echo "Optimized file: $OUTPUT"
```

---

## Troubleshooting Common Issues

**"File not found" errors:**

The CLI requires file paths, not stdin. Use:

```bash
# Correct
hedl validate data.hedl

# Not supported currently
cat data.hedl | hedl validate -
```

**"Parse error: unexpected token":**

Run `hedl inspect` to see how the parser interpreted your file. Often the issue is a missing comma, unquoted string with special characters, or indentation problem.

**"Reference not found":**

The referenced entity doesn't exist. Check:
1. Is the ID spelled correctly?
2. Is the entity defined before or after? (Both should work.)
3. Does the entity with that ID actually exist?

**Large file processing:**

If you hit memory limits:

```bash
export HEDL_MAX_FILE_SIZE=5368709120  # 5 GB
hedl validate huge_file.hedl
```

---

## Quick Reference

**Core commands:**

| Command | Purpose |
|---------|---------|
| `validate` | Check syntax and structure |
| `format` | Canonicalize formatting |
| `lint` | Check best practices |
| `inspect` | Show parsed structure |
| `stats` | Compare format sizes |

**Conversion commands:**

| Command | Direction |
|---------|-----------|
| `to-json` / `from-json` | HEDL ↔ JSON |
| `to-yaml` / `from-yaml` | HEDL ↔ YAML |
| `to-xml` / `from-xml` | HEDL ↔ XML |
| `to-csv` / `from-csv` | HEDL ↔ CSV |
| `to-parquet` / `from-parquet` | HEDL ↔ Parquet |
| `to-toon` / `from-toon` | HEDL ↔ TOON |

**Batch commands:**

| Command | Purpose |
|---------|---------|
| `batch-validate` | Validate multiple files |
| `batch-format` | Format multiple files |
| `batch-lint` | Lint multiple files |

**Common options:**

| Option | Meaning |
|--------|---------|
| `-o, --output` | Output file path |
| `--check` | Verify without modifying |
| `--pretty` | Human-readable output |
| `--verbose` | Detailed progress |
| `--strict` | Fail on any issue |

---

## What's Next?

You now know the CLI inside and out. Here's where to go deeper:

**[Formats Guide](formats.md)** covers each format conversion in detail: what converts, what doesn't, edge cases, and best practices.

**[Examples](examples.md)** shows real-world patterns you can adapt: data pipelines, API compression, configuration management.

**[Troubleshooting](troubleshooting.md)** helps when things go wrong.

Or just start using the CLI. The best way to learn is by doing. Pick a data file you work with regularly, convert it to HEDL, and see what happens.

```bash
hedl from-json your_data.json | hedl stats - --tokens
```

The numbers will speak for themselves.
