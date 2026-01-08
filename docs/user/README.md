# HEDL User Documentation

Welcome to the HEDL (Hierarchical Entity Data Language) user documentation! HEDL is a powerful, token-efficient data format designed for AI/ML workflows, data interchange, and human-readable data representation.

## What is HEDL?

HEDL is a compact, structured data format that combines the best features of JSON, YAML, and CSV while being significantly more token-efficient for LLM applications. It's designed to be:

- **Token-Efficient**: Up to 60% fewer tokens than equivalent JSON
- **Human-Readable**: Clear, intuitive syntax
- **Type-Safe**: Strong typing with validation
- **Interoperable**: Convert to/from JSON, YAML, XML, CSV, and Parquet
- **Fast**: High-performance parsing and conversion

## Quick Example

Here's a simple HEDL document:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, bob@example.com
  | u3, Charlie, charlie@example.com
```

This represents structured data with:
- A header declaring the format version with `%VERSION: 1.0`
- A struct definition with `%STRUCT: User: [id, name, email]`
- A separator line `---` dividing header from body
- A typed entity collection (`users: @User`)
- Three user records with pipe-delimited fields

## Documentation Sections

### Getting Started
- [**Getting Started**](getting-started.md) - Installation, first steps, and basic usage
- [**Examples**](examples.md) - Common use cases and practical examples

### Reference
- [**Supported Formats**](formats.md) - Complete guide to all supported data formats
- [**CLI Guide**](cli-guide.md) - Command-line interface reference
- [**FAQ**](faq.md) - Frequently asked questions
- [**Troubleshooting**](troubleshooting.md) - Common issues and solutions

## Use Cases

### AI/ML Workflows
- Reduce token costs in LLM applications by 40-60%
- Efficient training data representation
- Compact model configuration files

### Data Interchange
- Convert between JSON, YAML, XML, CSV, and Parquet
- Validate and lint data files
- Batch process large datasets

### Configuration Management
- Human-readable config files
- Type-safe configuration validation
- Version-controlled data files

### Data Analysis
- Import CSV data with automatic schema inference
- Export to Parquet for analytics
- Generate Neo4j Cypher statements for graph databases

## Key Features

### Token Efficiency
HEDL uses compact syntax and intelligent defaults to minimize token count:

**JSON (156 tokens):**
```json
{
  "users": [
    {"id": "u1", "name": "Alice", "email": "alice@example.com"},
    {"id": "u2", "name": "Bob", "email": "bob@example.com"},
    {"id": "u3", "name": "Charlie", "email": "charlie@example.com"}
  ]
}
```

**HEDL (significantly fewer tokens):**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, bob@example.com
  | u3, Charlie, charlie@example.com
```

### Format Conversion
Seamlessly convert between formats:

```bash
# JSON to HEDL
hedl from-json data.json -o data.hedl

# HEDL to JSON (with pretty formatting)
hedl to-json data.hedl --pretty -o data.json

# Validate and format
hedl validate data.hedl
hedl format data.hedl -o formatted.hedl
```

### Validation and Linting
Ensure data quality with built-in validation:

```bash
# Validate HEDL syntax
hedl validate data.hedl

# Format to canonical form
hedl format data.hedl -o formatted.hedl

# Get statistics
hedl stats data.hedl --tokens
```

### Batch Processing
Process multiple files efficiently:

```bash
# Convert all JSON files to HEDL
for f in *.json; do hedl from-json "$f" -o "${f%.json}.hedl"; done

# Validate all HEDL files
for f in *.hedl; do hedl validate "$f"; done
```

## Installation

### From Source
```bash
git clone https://github.com/dweve-ai/hedl
cd hedl
cargo install --path crates/hedl-cli
```

### Using Cargo
```bash
cargo install hedl-cli
```

### Verify Installation
```bash
hedl --version
```

## Quick Start

1. **Create a HEDL file** (`example.hedl`):
   ```hedl
   %VERSION: 1.0
   ---
   message: Hello, HEDL!
   count: 42
   ```

2. **Validate it**:
   ```bash
   hedl validate example.hedl
   ```

3. **Convert to JSON**:
   ```bash
   hedl to-json example.hedl --pretty
   ```

4. **Format it**:
   ```bash
   hedl format example.hedl
   ```

## Next Steps

- Follow the [Getting Started Guide](getting-started.md) for a complete tutorial
- Explore [Examples](examples.md) for common use cases
- Check the [CLI Guide](cli-guide.md) for all available commands
- Read the [Formats Guide](formats.md) to understand format conversion

## Support

- **GitHub Issues**: [https://github.com/dweve-ai/hedl/issues](https://github.com/dweve-ai/hedl/issues)
- **Documentation**: This guide and inline `--help` for all commands
- **Examples**: See `examples/` directory in the repository

## License

HEDL is dual-licensed under Apache-2.0 and MIT licenses.

---

**Ready to get started?** Head to the [Getting Started Guide](getting-started.md)!
