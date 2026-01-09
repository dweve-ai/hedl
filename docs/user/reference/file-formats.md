# File Formats Reference

Complete specifications for all supported data formats.

## Supported Formats

| Format | Extension | Read | Write | Streaming | Description |
|--------|-----------|------|-------|-----------|-------------|
| **HEDL** | `.hedl` | ✓ | ✓ | ✓ | Native format |
| **JSON** | `.json` | ✓ | ✓ | ✓ | JavaScript Object Notation |
| **YAML** | `.yaml`, `.yml` | ✓ | ✓ | ✗ | YAML Ain't Markup Language |
| **XML** | `.xml` | ✓ | ✓ | ✓ | eXtensible Markup Language |
| **CSV** | `.csv` | ✓ | ✓ | ✓ | Comma-Separated Values |
| **Parquet** | `.parquet` | ✓ | ✓ | ✓ | Apache Parquet columnar format |
| **Neo4j Cypher** | `.cypher` | ✓ | ✓ | ✓ | Neo4j graph database import |
| **TOON** | `.toon` | ✓ | ✓ | ✗ | Token-Oriented Object Notation for LLMs |

---

## HEDL Format

### Specification

**File Extension:** `.hedl`

**MIME Type:** `application/x-hedl` (unofficial)

**Character Encoding:** UTF-8

### Structure

```hedl
%VERSION: 1.0
%STRUCT: TypeName: [col1, col2]
---
entity_name: @TypeName
  | id1, value1, value2
  | id2, value1, value2
```

### Features

- Matrix lists for token efficiency
- Type annotations (`@TypeName`)
- References (`@Type:id`)
- Ditto operator (`^`)
- Null values (`~`)
- Comments (`#`)

### Limitations

- Maximum nesting depth: 50 (configurable)
- Maximum string length: 1MB (lines), 10MB (blocks)
- Maximum file size: 1GB (configurable)

---

## JSON Format

### Specification

**Standard:** [RFC 8259](https://tools.ietf.org/html/rfc8259)

**File Extension:** `.json`

**MIME Type:** `application/json`

### Conversion Mapping

**HEDL → JSON:**

| HEDL | JSON |
|------|------|
| Matrix list | Array of objects |
| Entity | Object key |
| String | String |
| Number | Number |
| Boolean | Boolean |
| Null (`~`) | `null` |
| Reference | `{"@ref": "Type:id"}` |

**Example:**

HEDL:
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

JSON:
```json
{
  "users": [
    {"id": "u1", "name": "Alice"}
  ]
}
```

### Options

**Output:**
- `--pretty` - Pretty-print with indentation
- `--metadata` - Include HEDL type metadata

**Input:**
- No specific options

---

## YAML Format

### Specification

**Standard:** [YAML 1.2](https://yaml.org/spec/1.2/spec.html)

**File Extension:** `.yaml`, `.yml`

**MIME Type:** `application/x-yaml`

### Conversion Mapping

Similar to JSON mapping, but with YAML-specific features:

- Flow style for simple lists
- Block style for nested structures
- Anchors/aliases supported on import (resolved automatically)

**Example:**


HEDL:
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

YAML:
```yaml
users:
  - id: u1
    name: Alice
```

### Options

**Output:**
- Standard YAML formatting

**Input:**
- No specific options

### Known Issues

- YAML's "Norway problem" avoided in HEDL
- Implicit type coercion differences

---

## XML Format

### Specification

**Standard:** [XML 1.0](https://www.w3.org/TR/xml/)

**File Extension:** `.xml`

**MIME Type:** `application/xml`

### Conversion Mapping

**HEDL → XML:**

| HEDL | XML |
|------|-----|
| Entity | Element |
| Matrix list | Multiple child elements |
| String value | Text content |
| Attributes | Mapped to fields |

**Example:**

HEDL:
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

XML:
```xml
<?xml version="1.0"?>
<root>
  <users>
    <User id="u1">
      <name>Alice</name>
    </User>
  </users>
</root>
```

### Options

**Output:**
- `--pretty` - Pretty-print with indentation

**Input:**
- No specific options

---

## CSV Format

### Specification

**Standard:** [RFC 4180](https://tools.ietf.org/html/rfc4180)

**File Extension:** `.csv`

**MIME Type:** `text/csv`

### Conversion Mapping

**HEDL matrix list → CSV table:**

HEDL:
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, bob@example.com
```

CSV:
```csv
id,name,email
u1,Alice,alice@example.com
u2,Bob,bob@example.com
```

### Options

**Output:**
- `--headers` - Include header row (default: true)

**Input:**
- `--type-name <NAME>` - Type name for the matrix list (default: "Row")

**Note:** First row is always treated as headers containing column names.

### Limitations

- Flat structure only (no nesting)
- Single entity per file
- Type inference required on import

---

## Parquet Format

### Specification

**Standard:** [Apache Parquet](https://parquet.apache.org/)

**File Extension:** `.parquet`

**MIME Type:** `application/octet-stream`

### Features

- Columnar storage format
- Efficient compression
- Schema embedded
- Fast analytics queries

### Compression Types

| Type | Speed | Ratio | Use Case |
|------|-------|-------|----------|
| **none** | Fastest | 1.0x | Already compressed data |
| **snappy** | Fast | 2-3x | Balanced (default) |
| **gzip** | Medium | 3-5x | Better compression |
| **zstd** | Medium | 4-6x | Best compression |

### Options

**Output:**
- No specific options

**Input:**
- No specific options

---

## Neo4j Cypher

### Specification

**Format:** [Cypher Query Language](https://neo4j.com/docs/cypher-manual/)

**File Extension:** `.cypher`

### Generated Cypher

**Nodes from entities:**
```cypher
CREATE (:User {id: 'u1', name: 'Alice'})
```

**Relationships from references:**
```cypher
MATCH (u:User {id: 'u1'}), (p:Post {id: 'p1'})
CREATE (u)-[:AUTHORED]->(p)
```

### Options

**Output:**
- No specific options

### Use Cases

- Import HEDL data into Neo4j graph database
- Create knowledge graphs from structured data

---

## Format Comparison

### Token Efficiency

For a sample dataset with 100 user records:

| Format | Bytes | Tokens | vs HEDL |
|--------|-------|--------|---------|
| **HEDL** | 2,450 | 612 | Baseline |
| **JSON** | 6,120 | 1,532 | +150% |
| **YAML** | 4,890 | 1,223 | +100% |
| **XML** | 8,740 | 2,186 | +257% |
| **CSV** | 1,830 | 458 | -25% |

**Note:** CSV is more compact but lacks nesting and typing.

### Feature Support

| Feature | HEDL | JSON | YAML | XML | CSV | Parquet |
|---------|------|------|------|-----|-----|---------|
| **Nesting** | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ |
| **Types** | ✓ | ✗ | ~ | ✗ | ✗ | ✓ |
| **References** | ✓ | ✗ | ✗ | ~ | ✗ | ✗ |
| **Comments** | ✓ | ✗ | ✓ | ✓ | ✗ | ✗ |
| **Schema** | ✓ | ✗ | ✗ | ~ | ✗ | ✓ |

---

## Best Practices

### Choosing a Format

**Use JSON when:**
- Interacting with web APIs
- JavaScript compatibility needed
- Deeply nested structures

**Use YAML when:**
- Human-readable config files
- Comments are important

**Use XML when:**
- Legacy system integration
- SOAP APIs
- Complex namespace requirements

**Use CSV when:**
- Spreadsheet compatibility
- Flat tabular data
- No nesting needed

**Use Parquet when:**
- Analytics and big data
- Columnar access patterns
- Storage efficiency critical

**Use HEDL when:**
- LLM applications (token efficiency)
- Tabular data with nesting
- Strong typing needed
- Reference relationships important

---

**Related:**
- [CLI Commands](cli-commands.md) - Conversion commands
- [How-To: Convert Formats](../how-to/convert-formats.md) - Conversion recipes
