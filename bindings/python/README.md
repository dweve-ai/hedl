# HEDL Python Bindings

Python bindings for HEDL (Hierarchical Entity Data Language) - a token-efficient data format optimized for LLM context windows.

## Installation

```bash
pip install hedl
```

**Note:** The shared library (`libhedl_ffi.so`/`.dylib`/`.dll`) must be available in your library path or specified via `HEDL_LIB_PATH`.

### Building the shared library

```bash
cd /path/to/hedl
cargo build --release -p hedl-ffi
```

## Quick Start

```python
import hedl

# Parse HEDL content
doc = hedl.parse('''
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
''')

# Get document info
print(f"Version: {doc.version}")  # (1, 0)
print(f"Schemas: {doc.schema_count}")  # 1

# Convert to JSON
json_str = doc.to_json(pretty=True)
print(json_str)

# Convert to other formats
yaml_str = doc.to_yaml()
xml_str = doc.to_xml()
csv_str = doc.to_csv()
cypher_str = doc.to_cypher()

# Clean up (or use context manager)
doc.close()
```

## Context Manager

```python
import hedl

with hedl.parse(content) as doc:
    json_str = doc.to_json()
    # Document automatically closed at end of block
```

## Format Conversion

### JSON to HEDL

```python
import hedl

doc = hedl.from_json('{"users": [{"id": "alice", "name": "Alice"}]}')
hedl_str = doc.canonicalize()
print(hedl_str)
```

### YAML to HEDL

```python
doc = hedl.from_yaml('''
users:
  - id: alice
    name: Alice
''')
```

### Parquet (Binary)

```python
# To Parquet
parquet_bytes = doc.to_parquet()
with open('data.parquet', 'wb') as f:
    f.write(parquet_bytes)

# From Parquet
with open('data.parquet', 'rb') as f:
    doc = hedl.from_parquet(f.read())
```

## Validation

```python
import hedl

# Quick validation (no document created)
is_valid = hedl.validate(content)

# Validation with diagnostics
doc = hedl.parse(content, strict=False)
with doc.lint() as diagnostics:
    for message, severity in diagnostics:
        level = ['HINT', 'WARNING', 'ERROR'][severity]
        print(f"[{level}] {message}")
```

## Neo4j Integration

```python
# Generate Cypher queries
cypher = doc.to_cypher(use_merge=True)
print(cypher)
# MERGE (alice:User {id: 'alice'}) SET alice.name = 'Alice Smith' ...
```

## Error Handling

```python
import hedl
from hedl import HedlError

try:
    doc = hedl.parse("invalid content")
except HedlError as e:
    print(f"Error: {e.message}")
    print(f"Code: {e.code}")
```

## API Reference

### Module Functions

| Function | Description |
|----------|-------------|
| `parse(content, strict=True)` | Parse HEDL string/bytes |
| `validate(content, strict=True)` | Validate without creating document |
| `from_json(content)` | Parse JSON to HEDL document |
| `from_yaml(content)` | Parse YAML to HEDL document |
| `from_xml(content)` | Parse XML to HEDL document |
| `from_parquet(bytes)` | Parse Parquet to HEDL document |

### Document Methods

| Method | Description |
|--------|-------------|
| `version` | Get (major, minor) version tuple |
| `schema_count` | Number of schema definitions |
| `canonicalize()` | Convert to canonical HEDL |
| `to_json(include_metadata=False)` | Convert to JSON |
| `to_yaml(include_metadata=False)` | Convert to YAML |
| `to_xml()` | Convert to XML |
| `to_csv()` | Convert to CSV |
| `to_parquet()` | Convert to Parquet bytes |
| `to_cypher(use_merge=True)` | Convert to Neo4j Cypher |
| `lint()` | Run linting, return Diagnostics |
| `close()` | Free document resources |

### Diagnostics

```python
with doc.lint() as diag:
    print(f"Issues: {len(diag)}")
    print(f"Errors: {diag.errors}")
    print(f"Warnings: {diag.warnings}")
    print(f"Hints: {diag.hints}")
```

## Environment Variables

| Variable | Description | Default | Recommended |
|----------|-------------|---------|-------------|
| `HEDL_LIB_PATH` | Path to the HEDL shared library | Auto-detected | - |
| `HEDL_MAX_OUTPUT_SIZE` | Maximum output size in bytes for conversions | 100 MB | 500 MB - 1 GB |

### Resource Limits

The `HEDL_MAX_OUTPUT_SIZE` environment variable controls the maximum size of output from conversion operations (`to_json()`, `to_yaml()`, `to_xml()`, etc.). The default of 100 MB is conservative and may be too restrictive for many real-world data processing scenarios.

**Setting the limit:**

```bash
# In your shell (before running Python)
export HEDL_MAX_OUTPUT_SIZE=1073741824  # 1 GB

# Or in Python (must be set BEFORE importing hedl)
import os
os.environ['HEDL_MAX_OUTPUT_SIZE'] = '1073741824'  # 1 GB
import hedl
```

**Recommended values:**

- **Small configs (10-50 MB)**: Default 100 MB is usually sufficient
- **Medium datasets (100-500 MB)**: Set to `524288000` (500 MB)
- **Large datasets (500 MB - 5 GB)**: Set to `1073741824` or higher (1 GB+)
- **Very large datasets**: Set to `5368709120` (5 GB) or `10737418240` (10 GB)
- **No practical limit**: Set to a very high value appropriate for your system

**Error handling:**

When the output size exceeds the limit, a `HedlError` will be raised:

```python
try:
    large_output = doc.to_json()
except hedl.HedlError as e:
    if e.code == hedl.HEDL_ERR_ALLOC:
        print(f"Output too large: {e.message}")
        print("Increase HEDL_MAX_OUTPUT_SIZE environment variable")
```

## Type Safety

The bindings include type annotations for IDE autocompletion and static type checking with mypy/pyright.

## Testing

```bash
pytest tests/ -v
```

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
