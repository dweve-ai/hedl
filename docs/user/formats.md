# Format Conversion Guide

HEDL supports bidirectional conversion with multiple data formats. This guide covers all supported formats, conversion options, and best practices.

## Table of Contents

1. [Overview](#overview)
2. [JSON](#json)
3. [YAML](#yaml)
4. [XML](#xml)
5. [CSV](#csv)
6. [Parquet](#parquet)
7. [TOON](#toon)
8. [Format Comparison](#format-comparison)
9. [Best Practices](#best-practices)
10. [Advanced: Neo4j Integration (Library)](#advanced-neo4j-integration-library)

## Overview

HEDL acts as a universal interchange format, allowing you to:

- Convert between any supported formats
- Preserve data structure and types
- Optimize for token efficiency
- Validate data during conversion

### Supported Formats (CLI)

| Format | Read | Write | Use Case |
|--------|------|-------|----------|
| JSON | ✓ | ✓ | APIs, web apps, general purpose |
| YAML | ✓ | ✓ | Configuration files, human-readable |
| XML | ✓ | ✓ | Legacy systems, SOAP APIs |
| CSV | ✓ | ✓ | Spreadsheets, tabular data |
| Parquet | ✓ | ✓ | Analytics, big data, columnar storage |
| TOON | ✓ | ✓ | Optimized for LLMs |

**Note**: Neo4j integration is available as a library (`hedl-neo4j` crate) for programmatic use, not through the CLI. See [Advanced: Neo4j Integration (Library)](#advanced-neo4j-integration-library) for details.

## JSON

JSON (JavaScript Object Notation) is the most common data interchange format.

### HEDL to JSON

Convert HEDL to JSON format:

```bash
# Compact JSON (one line)
hedl to-json data.hedl

# Pretty-printed JSON (indented)
hedl to-json data.hedl --pretty

# Include HEDL metadata
hedl to-json data.hedl --metadata --pretty

# Save to file
hedl to-json data.hedl -o output.json
```

#### Example Conversion

**Input (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User
  | u1, Alice, 30
  | u2, Bob, 25
```

**Output (JSON, --pretty):**
```json
{
  "users": [
    {
      "id": "u1",
      "name": "Alice",
      "age": 30
    },
    {
      "id": "u2",
      "name": "Bob",
      "age": 25
    }
  ]
}
```

**Output (JSON, --metadata):**
```json
{
  "users": {
    "__type__": "User",
    "__schema__": ["id", "name", "age"],
    "__count_hint__": 2,
    "items": [
      {
        "id": "u1",
        "name": "Alice",
        "age": 30
      },
      {
        "id": "u2",
        "name": "Bob",
        "age": 25
      }
    ]
  }
}
```

### JSON to HEDL

Convert JSON to HEDL format:

```bash
# Basic conversion
hedl from-json data.json

# Save to file
hedl from-json data.json -o data.hedl
```

#### Example Conversion

**Input (JSON):**
```json
{
  "users": [
    {"id": "u1", "name": "Alice", "age": 30},
    {"id": "u2", "name": "Bob", "age": 25}
  ]
}
```

**Output (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User
  | u1, Alice, 30
  | u2, Bob, 25
```

### Token Efficiency: JSON vs HEDL

For the example above:
- **JSON**: 156 tokens
- **HEDL**: 62 tokens
- **Savings**: 60% fewer tokens

## YAML

YAML (YAML Ain't Markup Language) is popular for configuration files.

### HEDL to YAML

```bash
# Convert to YAML
hedl to-yaml data.hedl

# Save to file
hedl to-yaml data.hedl -o config.yaml
```

#### Example Conversion

**Input (HEDL):**
```hedl
%VERSION: 1.0
---
database:
  host: localhost
  port: 5432
  credentials:
    username: admin
    password: secret
```

**Output (YAML):**
```yaml
database:
  host: localhost
  port: 5432
  credentials:
    username: admin
    password: secret
```

### YAML to HEDL

```bash
# Convert from YAML
hedl from-yaml config.yaml

# Save to file
hedl from-yaml config.yaml -o config.hedl
```

### Use Cases

- Configuration files (app configs, CI/CD)
- Docker Compose files
- Kubernetes manifests
- Ansible playbooks

## XML

XML (eXtensible Markup Language) is used in many enterprise systems.

### HEDL to XML

```bash
# Compact XML
hedl to-xml data.hedl

# Pretty-printed XML
hedl to-xml data.hedl --pretty

# Save to file
hedl to-xml data.hedl -o output.xml
```

#### Example Conversion

**Input (HEDL):**
```hedl
%VERSION: 1.0
---
book:
  title: The Rust Book
  author: Steve Klabnik
  year: 2018
  chapters: 3
    Getting Started
    Common Concepts
    Ownership
```

**Output (XML, --pretty):**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<book>
  <title>The Rust Book</title>
  <author>Steve Klabnik</author>
  <year>2018</year>
  <chapters>
    <item>Getting Started</item>
    <item>Common Concepts</item>
    <item>Ownership</item>
  </chapters>
</book>
```

### XML to HEDL

```bash
# Convert from XML
hedl from-xml data.xml

# Save to file
hedl from-xml data.xml -o data.hedl
```

### XML Attributes

XML attributes are converted to regular HEDL fields (no prefix):

**Input (XML):**
```xml
<book id="b1" format="hardcover">
  <title>Example</title>
</book>
```

**Output (HEDL):**
```hedl
%VERSION: 1.0
---
book:
  id: b1
  format: hardcover
  title: Example
```

### Use Cases

- SOAP APIs
- Legacy enterprise systems
- RSS/Atom feeds
- SVG graphics
- Office document formats

## CSV

CSV (Comma-Separated Values) is the standard for tabular data.

### HEDL to CSV

```bash
# Convert to CSV
hedl to-csv data.hedl

# Save to file
hedl to-csv data.hedl -o output.csv
```

#### Example Conversion

**Input (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, price, quantity]
---
products: @Product
  | p1, Widget, 19.99, 100
  | p2, Gadget, 29.99, 50
  | p3, Doohickey, 9.99, 200
```

**Output (CSV):**
```csv
id,name,price,quantity
p1,Widget,19.99,100
p2,Gadget,29.99,50
p3,Doohickey,9.99,200
```

### CSV to HEDL

```bash
# Convert from CSV (first row is header, first column is ID by default)
hedl from-csv data.csv -t Product

# Save to file
hedl from-csv data.csv -t Product -o data.hedl

# With custom type name
hedl from-csv users.csv -t User -o users.hedl
```

**Important**: The CSV file must have an ID column as the first column. The ID is extracted automatically and used as the node identifier. Only the remaining columns are included in the HEDL struct.

#### Example Conversion

**Input (CSV):**
```csv
id,name,email
1,Alice,alice@example.com
2,Bob,bob@example.com
```

**Output (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: User: [name, email]
---
users: @User
  | 1, Alice, alice@example.com
  | 2, Bob, bob@example.com
```

Note: The `id` column becomes the node ID (first field in each row after `@User`), while `name` and `email` are the struct fields.

### Schema Inference

HEDL automatically infers data types from CSV columns (excluding the ID column):

**Input (CSV):**
```csv
id,name,age,active,score
1,Alice,30,true,95.5
2,Bob,25,false,87.3
```

**Output (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: Record: [name, age, active, score]
---
records: @Record
  | 1, Alice, 30, true, 95.5
  | 2, Bob, 25, false, 87.3
```

Types detected:
- `name`: String
- `age`: Integer
- `active`: Boolean
- `score`: Float

### Use Cases

- Excel/Google Sheets data
- Database exports
- Scientific datasets
- Financial data

## Parquet

Apache Parquet is a columnar storage format optimized for analytics.

### HEDL to Parquet

```bash
# Convert to Parquet
hedl to-parquet data.hedl -o output.parquet

# Note: Parquet output must be written to a file
```

#### Example Conversion

**Input (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: Sale: [id, product, amount, timestamp]
---
sales: @Sale
  | s1, Widget, 99.99, 2024-01-15T10:30:00Z
  | s2, Gadget, 149.99, 2024-01-15T11:45:00Z
  # ... 998 more rows
```

**Output:** Binary Parquet file optimized for analytics

### Parquet to HEDL

```bash
# Convert from Parquet
hedl from-parquet data.parquet -o data.hedl
```

### Schema Mapping

HEDL preserves Parquet schema information:

| Parquet Type | HEDL Type |
|--------------|-----------|
| INT32 | Integer |
| INT64 | Integer |
| FLOAT | Float |
| DOUBLE | Float |
| BOOLEAN | Boolean |
| BYTE_ARRAY | String |
| TIMESTAMP | String (ISO 8601) |

### Use Cases

- Data analytics pipelines
- Big data processing (Spark, Hadoop)
- Data warehousing
- Long-term data archival
- Columnar query optimization

### Performance Characteristics

- **Compression**: Parquet files are typically 70-90% smaller than CSV
- **Query Speed**: 10-100x faster for analytical queries
- **Write Speed**: Slower than CSV but optimized for read-heavy workloads


## TOON

TOON (Token-Oriented Object Notation) is a compact format optimized for LLM consumption.

### HEDL to TOON

```bash
# Generate TOON format
hedl to-toon data.hedl

# Save to file
hedl to-toon data.hedl -o output.toon
```

#### Example Conversion

**Input (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
  | u2, Bob
```

**Output (TOON):**
```
users[2]{id,name}:
  u1,Alice
  u2,Bob
```

### TOON to HEDL

```bash
# Convert TOON to HEDL
hedl from-toon data.toon

# Save to file
hedl from-toon data.toon -o data.hedl
```

### Characteristics

- **Compact**: Optimized for token efficiency
- **No metadata**: Minimal overhead
- **Schema-first**: Structure defined once

### Token Efficiency

For typical datasets:
- **JSON**: 100% (baseline)
- **HEDL**: 40% (60% reduction)
- **TOON**: 25% (75% reduction)

## Format Comparison

### Size and Token Comparison

For a typical dataset of 1000 user records:

| Format | File Size | Tokens | Compression |
|--------|-----------|--------|-------------|
| JSON | 125 KB | 38,500 | - |
| YAML | 98 KB | 29,200 | 24% smaller |
| XML | 187 KB | 52,300 | 50% larger |
| CSV | 45 KB | 12,800 | 64% smaller |
| HEDL | 52 KB | 15,400 | 58% smaller |
| Parquet | 18 KB | N/A | 86% smaller |
| TOON | 38 KB | 10,500 | 70% smaller |

### Feature Comparison

| Feature | JSON | YAML | XML | CSV | Parquet | HEDL |
|---------|------|------|-----|-----|---------|------|
| Human-readable | ✓ | ✓ | ✓ | ✓ | - | ✓ |
| Hierarchical | ✓ | ✓ | ✓ | - | ✓ | ✓ |
| Type-safe | - | - | - | - | ✓ | ✓ |
| Comments | - | ✓ | ✓ | - | - | ✓ |
| References | - | ✓ | - | - | - | ✓ |
| Streaming | ✓ | - | ✓ | ✓ | ✓ | ✓ |
| Token-efficient | - | - | - | ✓ | N/A | ✓ |

### When to Use Each Format

**JSON**
- APIs and web services
- JavaScript applications
- General-purpose data exchange
- Maximum compatibility

**YAML**
- Configuration files
- Human-editable data
- CI/CD pipelines
- Kubernetes/Docker configs

**XML**
- Legacy system integration
- SOAP web services
- Document markup
- Enterprise applications

**CSV**
- Spreadsheet data
- Simple tabular data
- Database exports
- Maximum simplicity

**Parquet**
- Data analytics
- Big data processing
- Long-term archival
- Columnar queries

**TOON**
- LLM context windows
- Token efficiency (75% smaller than JSON)
- Compact serialization

**HEDL**
- AI/ML workflows
- Type-safe data
- Multi-format conversion hub
- Local data transformation (use TOON for LLM context)

**Neo4j (Library)**
- Graph databases and knowledge graphs
- Relationship queries and pattern matching
- Recommend using the `hedl-neo4j` crate for Rust projects

## Best Practices

### Choosing the Right Format

1. **For APIs**: Use JSON (standard, widely supported)
2. **For Configuration**: Use YAML (human-readable, comments)
3. **For Analytics**: Use Parquet (fast queries, compression)
4. **For Spreadsheets**: Use CSV (Excel compatible)
5. **For LLM Context**: Use HEDL (token-efficient)
6. **For Graph Data**: Use Neo4j Cypher (relationships)

### Conversion Strategies

#### Lossless Conversion

Most conversions are lossless. However, be aware of:

**Type Precision**:
- JSON numbers → HEDL preserves precision
- CSV strings → HEDL infers types (may need validation)

**Metadata Loss**:
- HEDL types → JSON (use `--metadata` to preserve)
- XML attributes → HEDL (become regular fields)

**Structure Changes**:
- Flat CSV → Nested HEDL (manual restructuring needed)
- Nested HEDL → Flat CSV (flattens hierarchy)

#### Optimization Tips

1. **Automatic Type Inference**: HEDL automatically infers types from JSON/CSV
   ```bash
   hedl from-json data.json -o data.hedl
   ```

2. **Validate After Conversion**: Always validate converted data
   ```bash
   hedl from-csv data.csv -t Record -o temp.hedl
   hedl validate temp.hedl
   ```

3. **Format for Readability**: Format HEDL for human review
   ```bash
   hedl from-json data.json -o temp.hedl
   hedl format temp.hedl -o clean.hedl
   ```

4. **Batch Convert**: Use parallel processing for multiple files
   ```bash
   hedl batch-format "*.hedl" --parallel
   ```

### Pipeline Processing

Chain conversions using intermediate files:

```bash
# CSV → HEDL → Parquet
hedl from-csv data.csv -t Record -o temp.hedl && hedl to-parquet temp.hedl -o data.parquet

# JSON → HEDL → YAML
hedl from-json api.json -o temp.hedl && hedl to-yaml temp.hedl -o config.yaml

# Multiple format exports from one HEDL
hedl from-csv data.csv -t Data -o data.hedl
hedl to-json data.hedl -o data.json
hedl to-yaml data.hedl -o data.yaml
hedl to-parquet data.hedl -o data.parquet
```

### Error Handling

Always check for conversion errors:

```bash
#!/bin/bash
if hedl from-json data.json -o data.hedl; then
  echo "Conversion successful"
  hedl validate data.hedl
else
  echo "Conversion failed" >&2
  exit 1
fi
```

### Performance Considerations

**Large Files**:
- Use streaming for files > 100MB
- Increase `HEDL_MAX_FILE_SIZE` if needed
- Consider splitting into smaller chunks

**Parallel Processing**:
- Use `--parallel` for batch operations
- Process independent files concurrently
- Monitor memory usage

**Memory Usage**:
- Parquet: Most memory-efficient
- Streaming: Use for large files
- CSV: Low memory overhead

---

## Advanced: Neo4j Integration (Library)

**Neo4j support is available only as a library crate (`hedl-neo4j`), not through the CLI.**

The `hedl-neo4j` crate provides bidirectional conversion between HEDL documents and Neo4j:

- **HEDL to Cypher**: Export HEDL documents as Neo4j CREATE/MERGE statements with automatic relationship detection
- **Neo4j to HEDL**: Import Neo4j query results back to HEDL with schema preservation
- **Batch Processing**: UNWIND-based bulk operations for high-throughput imports
- **Streaming API**: Process large documents without full memory buffering
- **Security**: Unicode normalization, zero-width filtering, string length limits

### Usage (Rust/Cargo)

Add to `Cargo.toml`:
```toml
[dependencies]
hedl-neo4j = "1.2"
```

### Basic Example

```rust
use hedl_core::parse;
use hedl_neo4j::{to_cypher, ToCypherConfig};

let doc = parse(br#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
  | u2, Bob
"#)?;

let config = ToCypherConfig::default();
let cypher = to_cypher(&doc, &config)?;
println!("{}", cypher);
```

### Use Cases

- Knowledge graphs and graph database migrations
- Social network data import
- Relationship mapping and analysis
- Integration with Neo4j-based ML workflows

For complete documentation on Neo4j integration, see the [`hedl-neo4j` README](https://github.com/dweve-ai/hedl/tree/master/crates/hedl-neo4j).

---

**Need help?** Check the [CLI Guide](cli-guide.md) for detailed command options or [Troubleshooting](troubleshooting.md) for common issues.
