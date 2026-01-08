# How-To: Convert Between Formats

Practical recipes for converting data between HEDL and other formats (JSON, YAML, XML, CSV, Parquet).

## Table of Contents

1. [JSON Conversions](#json-conversions)
2. [YAML Conversions](#yaml-conversions)
3. [XML Conversions](#xml-conversions)
4. [CSV Conversions](#csv-conversions)
5. [Parquet Conversions](#parquet-conversions)
6. [Multi-Step Conversions](#multi-step-conversions)
7. [Preserving Metadata](#preserving-metadata)
8. [Batch Conversions](#batch-conversions)

---

## JSON Conversions

### JSON to HEDL

**Goal:** Convert JSON files to HEDL format.

**Basic Conversion:**
```bash
hedl from-json input.json -o output.hedl
```

**Example:**

Input (`users.json`):
```json
{
  "users": [
    {"id": "u1", "name": "Alice", "age": 30},
    {"id": "u2", "name": "Bob", "age": 25}
  ]
}
```

Command:
```bash
hedl from-json users.json -o users.hedl
```

Output (`users.hedl`):
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User
  | u1, Alice, 30
  | u2, Bob, 25
```

**Variations:**

```bash
# From stdin
cat data.json | hedl from-json - -o output.hedl

# Multiple files
for f in *.json; do
  hedl from-json "$f" -o "${f%.json}.hedl"
done
```

### HEDL to JSON

**Goal:** Convert HEDL files to JSON format.

**Basic Conversion:**
```bash
hedl to-json input.hedl -o output.json
```

**Pretty-Printed:**
```bash
hedl to-json input.hedl --pretty -o output.json
```

**With Metadata:**
```bash
# Include HEDL type information
hedl to-json input.hedl --metadata --pretty -o output.json
```

**Example:**

Input (`products.hedl`):
```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, price]
---
products: @Product
  | p1, Laptop, 999.99
  | p2, Mouse, 29.99
```

Command:
```bash
hedl to-json products.hedl --pretty -o products.json
```

Output (`products.json`):
```json
{
  "products": [
    {"id": "p1", "name": "Laptop", "price": 999.99},
    {"id": "p2", "name": "Mouse", "price": 29.99}
  ]
}
```

**Variations:**

```bash
# Compact JSON
hedl to-json input.hedl | jq -c

# To stdout for piping
hedl to-json input.hedl | curl -X POST https://api.example.com/data
```

---

## YAML Conversions

### YAML to HEDL

**Goal:** Convert YAML configuration files to HEDL.

**Basic Conversion:**
```bash
hedl from-yaml config.yaml -o config.hedl
```

**Example:**

Input (`config.yaml`):
```yaml
servers:
  - id: srv1
    host: example.com
    port: 8080
  - id: srv2
    host: backup.com
    port: 8080
```

Command:
```bash
hedl from-yaml config.yaml -o config.hedl
```

Output (`config.hedl`):
```hedl
%VERSION: 1.0
%STRUCT: Server: [id, host, port]
---
servers: @Server
  | srv1, example.com, 8080
  | srv2, backup.com, 8080
```

### HEDL to YAML

**Goal:** Generate YAML from HEDL.

**Basic Conversion:**
```bash
hedl to-yaml input.hedl -o output.yaml
```

**Example:**
```bash
hedl to-yaml config.hedl -o config.yaml
```

**Variations:**

```bash
# To stdout
hedl to-yaml config.hedl

# Pipeline to validator
hedl to-yaml config.hedl | yamllint -
```

---

## XML Conversions

### XML to HEDL

**Goal:** Convert XML documents to HEDL.

**Basic Conversion:**
```bash
hedl from-xml data.xml -o data.hedl
```

**Example:**

Input (`books.xml`):
```xml
<?xml version="1.0"?>
<library>
  <book id="b1">
    <title>Rust Programming</title>
    <author>Steve Klabnik</author>
    <year>2018</year>
  </book>
  <book id="b2">
    <title>Clean Code</title>
    <author>Robert Martin</author>
    <year>2008</year>
  </book>
</library>
```

Command:
```bash
hedl from-xml books.xml -o books.hedl
```

Output (`books.hedl`):
```hedl
%VERSION: 1.0
%STRUCT: Book: [id, title, author, year]
---
books: @Book
  | b1, Rust Programming, Steve Klabnik, 2018
  | b2, Clean Code, Robert Martin, 2008
```

### HEDL to XML

**Goal:** Generate XML from HEDL.

**Basic Conversion:**
```bash
hedl to-xml data.hedl -o data.xml
```

**Pretty Print:**
```bash
hedl to-xml data.hedl --pretty -o data.xml
```

**Example:**
```bash
hedl to-xml books.hedl --pretty -o books.xml
```

---

## CSV Conversions

### CSV to HEDL

**Goal:** Import CSV data into HEDL format.

**Basic Conversion:**
```bash
hedl from-csv data.csv -o data.hedl
```

**Note:** The first row of the CSV file is treated as the header row containing column names.

**Example:**

Input (`employees.csv`):
```csv
id,name,department,salary
e1,Alice,Engineering,95000
e2,Bob,Sales,72000
e3,Carol,Engineering,87000
```

Command:
```bash
hedl from-csv employees.csv --type-name Employee -o employees.hedl
```

Output (`employees.hedl`):
```hedl
%VERSION: 1.0
%STRUCT: Employee: [id, name, department, salary]
---
employees: @Employee
  | e1, Alice, Engineering, 95000
  | e2, Bob, Sales, 72000
  | e3, Carol, Engineering, 87000
```

### HEDL to CSV

**Goal:** Export HEDL data to CSV format.

**Basic Conversion:**
```bash
hedl to-csv data.hedl -o data.csv
```

**With Headers:**
```bash
hedl to-csv data.hedl --headers -o data.csv
```

**Example:**
```bash
hedl to-csv employees.hedl -o employees.csv
```

---

## Parquet Conversions

### HEDL to Parquet

**Goal:** Export HEDL to Apache Parquet for analytics.

**Basic Conversion:**
```bash
hedl to-parquet data.hedl -o data.parquet
```

**Example:**
```bash
hedl to-parquet events.hedl -o events.parquet
```

### Parquet to HEDL

**Goal:** Import Parquet files into HEDL.

**Basic Conversion:**
```bash
hedl from-parquet data.parquet -o data.hedl
```

**Example:**
```bash
hedl from-parquet analytics.parquet -o analytics.hedl
```

---

## Multi-Step Conversions

### CSV → HEDL → Parquet

**Goal:** Convert CSV data to Parquet via HEDL.

**Pipeline:**
```bash
hedl from-csv data.csv | \
  hedl to-parquet - -o data.parquet
```

**With Intermediate Validation:**
```bash
hedl from-csv data.csv -o temp.hedl
hedl validate temp.hedl
hedl to-parquet temp.hedl -o data.parquet
rm temp.hedl
```

### JSON → HEDL → XML

**Goal:** Convert JSON API response to XML.

**Pipeline:**
```bash
curl https://api.example.com/data | \
  hedl from-json - | \
  hedl to-xml --pretty -
```

**With Formatting:**
```bash
curl https://api.example.com/data | \
  hedl from-json - | \
  hedl format - | \
  hedl to-xml --pretty - > output.xml
```

### Multiple Formats from Single Source

**Goal:** Convert one HEDL file to multiple formats.

**Script:**
```bash
#!/bin/bash

INPUT="data.hedl"
BASE_NAME=$(basename "$INPUT" .hedl)

# Validate first
hedl validate "$INPUT"

# Convert to all formats
hedl to-json "$INPUT" --pretty -o "${BASE_NAME}.json"
hedl to-yaml "$INPUT" -o "${BASE_NAME}.yaml"
hedl to-xml "$INPUT" --pretty -o "${BASE_NAME}.xml"
hedl to-csv "$INPUT" -o "${BASE_NAME}.csv"
hedl to-parquet "$INPUT" -o "${BASE_NAME}.parquet"

echo "Converted $INPUT to all formats"
```

---

## Preserving Metadata

### Preserve HEDL Type Information

**JSON Output with Metadata:**
```bash
hedl to-json data.hedl --metadata --pretty -o data.json
```

**Output includes types:**
```json
{
  "_hedl_version": "1.0",
  "_hedl_types": {
    "users": "User"
  },
  "users": [...]
}
```

### Roundtrip Preservation

**Ensure lossless roundtrip:**
```bash
# Original → JSON → HEDL → Compare
hedl to-json original.hedl --metadata -o temp.json
hedl from-json temp.json -o roundtrip.hedl
diff <(hedl format original.hedl) <(hedl format roundtrip.hedl)
```



---

## Batch Conversions

### Convert Directory of JSON Files

**Script:**
```bash
#!/bin/bash

INPUT_DIR="json_files"
OUTPUT_DIR="hedl_files"

mkdir -p "$OUTPUT_DIR"

for json in "$INPUT_DIR"/*.json; do
  base=$(basename "$json" .json)
  hedl from-json "$json" -o "$OUTPUT_DIR/${base}.hedl"
  echo "✓ Converted $json"
done
```

### Parallel Batch Conversion

**Using GNU Parallel:**
```bash
find json_files -name "*.json" | \
  parallel "hedl from-json {} -o {.}.hedl"
```

**Using xargs:**
```bash
find json_files -name "*.json" -print0 | \
  xargs -0 -n 1 -P 4 -I {} sh -c 'hedl from-json "{}" -o "${1%.json}.hedl"' _ {}
```

---

## Format-Specific Tips

### JSON Best Practices

```bash
# Validate JSON before converting
jq empty input.json && hedl from-json input.json -o output.hedl

# Pretty-print output
hedl to-json data.hedl --pretty | jq . > output.json

# Minify JSON output
hedl to-json data.hedl | jq -c > output.min.json
```

### CSV Best Practices

```bash
# Verify headers before converting
head -n 1 data.csv

# Handle quotes in data (automatically handled by parser)
hedl from-csv data.csv -o data.hedl
```

### Parquet Best Practices

```bash
# Verify output
parquet-tools schema data.parquet
parquet-tools rowcount data.parquet
```

---

## Troubleshooting

### Common Issues

**Issue: Type inference fails**
Ensure your input data has consistent types. HEDL infers types automatically.

**Issue: Large file conversion runs out of memory**
Split large files into smaller chunks before converting.

**Issue: Character encoding problems**
Convert encoding to UTF-8 before processing:
```bash
# Convert encoding first
iconv -f ISO-8859-1 -t UTF-8 data.csv | hedl from-csv - -o data.hedl
```

---

## Quick Reference

```bash
# JSON conversions
hedl from-json input.json -o output.hedl
hedl to-json input.hedl --pretty -o output.json

# YAML conversions
hedl from-yaml input.yaml -o output.hedl
hedl to-yaml input.hedl -o output.yaml

# XML conversions
hedl from-xml input.xml -o output.hedl
hedl to-xml input.hedl --pretty -o output.xml

# CSV conversions
hedl from-csv input.csv -o output.hedl
hedl to-csv input.hedl -o output.csv

# Parquet conversions
hedl from-parquet input.parquet -o output.hedl
hedl to-parquet input.hedl -o output.parquet

# Pipelines
cat input.json | hedl from-json - | hedl to-parquet - -o output.parquet
```

---

**Related Guides:**
- [Handle Errors](handle-errors.md) - Fix conversion issues
- [Optimize Performance](optimize-performance.md) - Speed up conversions
- [Validate Documents](validate-documents.md) - Ensure conversion quality
