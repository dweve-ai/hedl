# Format Conversion Guide

HEDL supports bidirectional conversion with multiple data formats. This guide covers all supported formats, conversion options, and best practices.

## Table of Contents

1. [Overview](#overview)
2. [JSON](#json)
3. [YAML](#yaml)
4. [XML](#xml)
5. [CSV](#csv)
6. [Parquet](#parquet)
7. [Neo4j Cypher](#neo4j-cypher)
8. [TOON](#toon)
9. [Format Comparison](#format-comparison)
10. [Best Practices](#best-practices)

## Overview

HEDL acts as a universal interchange format, allowing you to:

- Convert between any supported formats
- Preserve data structure and types
- Optimize for token efficiency
- Validate data during conversion

### Supported Formats

| Format | Read | Write | Use Case |
|--------|------|-------|----------|
| JSON | ✓ | ✓ | APIs, web apps, general purpose |
| YAML | ✓ | ✓ | Configuration files, human-readable |
| XML | ✓ | ✓ | Legacy systems, SOAP APIs |
| CSV | ✓ | ✓ | Spreadsheets, tabular data |
| Parquet | ✓ | ✓ | Analytics, big data, columnar storage |
| Neo4j | ✓ | ✓ | Graph databases, relationships |
| TOON | - | ✓ | Optimized for LLMs |

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
  "version": "1.0",
  "users": {
    "_type": "User",
    "_count": 2,
    "data": [
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

XML attributes are converted to HEDL fields with an `_attr_` prefix:

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
  _attr_id: b1
  _attr_format: hardcover
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
# Convert from CSV (first row is treated as header by default)
hedl from-csv data.csv -t Product

# Save to file
hedl from-csv data.csv -t Product -o data.hedl
```

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
%STRUCT: User: [id, name, email]
---
users: @User
  | 1, Alice, alice@example.com
  | 2, Bob, bob@example.com
```

### Schema Inference

HEDL automatically infers data types from CSV:

**Input (CSV):**
```csv
name,age,active,score
Alice,30,true,95.5
Bob,25,false,87.3
```

**Output (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: Record: [name, age, active, score]
---
records: @Record
  | Alice, 30, true, 95.5
  | Bob, 25, false, 87.3
```

Types detected:
- `name`: String (quoted)
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

### Neo4j Cypher

Generate Cypher statements for graph database import.

**Note:** Cypher export is currently available via the `hedl-neo4j` library or MCP server.

```rust
// Using the Rust library
use hedl_neo4j::{to_cypher, ToCypherConfig};

let cypher = to_cypher(&doc, &ToCypherConfig::default())?;
```

#### Example Conversion

**Input (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Friendship: [from, to]
---
users: @User
  | u1, Alice
  | u2, Bob

friendships: @Friendship
  | @User:u1, @User:u2
```

**Output (Cypher):**
```cypher
CREATE (u1:User {id: 'u1', name: 'Alice'})
CREATE (u2:User {id: 'u2', name: 'Bob'})
CREATE (u1)-[:FRIENDSHIP]->(u2)
```

### Relationship Mapping

HEDL references (`@Type:id`) are converted to Neo4j relationships:

**Input (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: Person: [id, name]
%STRUCT: WorksAt: [person, company]
---
people: @Person
  | p1, Alice
  | p2, Bob

works_at: @WorksAt
  | @Person:p1, @Company:c1
```

**Output (Cypher):**
```cypher
CREATE (p1:Person {id: 'p1', name: 'Alice'})
CREATE (p2:Person {id: 'p2', name: 'Bob'})
MATCH (person:Person {id: 'p1'}), (company:Company {id: 'c1'})
CREATE (person)-[:WORKS_AT]->(company)
```

### Use Cases

- Graph database migrations
- Social network data import
- Knowledge graph construction
- Relationship mapping
- Neo4j data pipelines

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

**HEDL**
- AI/ML workflows
- Token-efficient storage
- Type-safe data
- Multi-format conversion hub

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
- XML attributes → HEDL (use `_attr_` prefix)

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
   hedl from-csv data.csv -t Record | hedl validate -
   ```

3. **Format for Readability**: Format HEDL for human review
   ```bash
   hedl from-json data.json | hedl format - -o clean.hedl
   ```

4. **Batch Convert**: Use parallel processing for multiple files
   ```bash
   hedl batch-format "*.hedl" --parallel
   ```

### Pipeline Processing

Chain conversions efficiently:

```bash
# CSV → HEDL → Parquet (using intermediate file as parquet requires file output)
hedl from-csv data.csv -t Record -o temp.hedl && hedl to-parquet temp.hedl -o data.parquet

# JSON → HEDL → YAML
hedl from-json api.json | hedl to-yaml - -o config.yaml

# Multiple JSON → Single HEDL
cat *.json | jq -s '.' | hedl from-json - -o combined.hedl
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

**Need help?** Check the [CLI Guide](cli-guide.md) for detailed command options or [Troubleshooting](troubleshooting.md) for common issues.
