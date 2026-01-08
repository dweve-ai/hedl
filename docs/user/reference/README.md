# HEDL Reference Documentation

Complete technical reference for HEDL syntax, commands, formats, and configuration.

## Reference Guides

### 1. [CLI Commands](cli-commands.md)
**Complete command-line reference**

Comprehensive documentation for all `hedl` CLI commands:
- Command syntax and options
- Input/output formats
- Exit codes
- Examples for each command

### 2. [File Formats](file-formats.md)
**Supported format specifications**

Detailed specifications for all supported formats:
- JSON, YAML, XML, CSV, Parquet
- Format-specific options
- Conversion mappings
- Limitations and edge cases

### 3. [Configuration](configuration.md)
**Configuration options and environment variables**

Complete configuration reference:
- Environment variables
- Resource limits
- Performance tuning
- Security settings

### 4. [Glossary](glossary.md)
**Terminology and definitions**

Comprehensive glossary of HEDL terms:
- Core concepts
- Technical terminology
- Command options
- Format-specific terms

---

## Quick Reference

### Common Commands

```bash
# Validation
hedl validate file.hedl

# Formatting
hedl format file.hedl -o formatted.hedl

# Conversion
hedl to-json file.hedl --pretty -o output.json
hedl from-csv data.csv -o output.hedl

# Batch operations
hedl batch-validate *.hedl --parallel
```

### Common Options

| Option | Description |
|--------|-------------|
| `-o, --output` | Output file path |
| `--pretty` | Pretty-print output |
| `--parallel` | Enable parallel processing |
| `--help` | Show help |

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Validation error, command failure, or invalid arguments |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HEDL_MAX_FILE_SIZE` | 1GB | Maximum file size |

### Configuration Files

## Using This Reference

### Finding Information

**By task:**
1. Look up the command in [CLI Commands](cli-commands.md)
2. Check format details in [File Formats](file-formats.md)
3. Configure via [Configuration](configuration.md)

**By term:**
1. Check the [Glossary](glossary.md)
2. Follow cross-references to detailed docs

### Reference Structure

Each reference page follows this structure:

1. **Overview** - High-level description
2. **Syntax** - Exact command/option syntax
3. **Parameters** - Detailed parameter descriptions
4. **Examples** - Working examples
5. **Notes** - Edge cases and limitations

### Conventions

**Command syntax notation:**
```
<required>  - Required parameter
[optional]  - Optional parameter
...         - Can be repeated
|           - OR (alternatives)
```

**Example:**
```bash
hedl validate <FILE>...
```
Means:
- `validate` is the command
- `FILE` is required
- Multiple files can be specified

---

## Complementary Documentation

- **Learning HEDL?** Start with [Tutorials](../tutorials/)
- **Solving a problem?** Check [How-To Guides](../how-to/)
- **Understanding concepts?** Read [Concepts](../concepts/)
- **Need help?** See [FAQ](../faq.md) or [Troubleshooting](../troubleshooting.md)

---

**Choose a reference guide:**

- [CLI Commands](cli-commands.md) - Complete command reference
- [File Formats](file-formats.md) - Format specifications
- [Configuration](configuration.md) - Settings and tuning
- [Glossary](glossary.md) - Terminology
