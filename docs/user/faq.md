# Frequently Asked Questions (FAQ)

Common questions about HEDL and their answers.

## Table of Contents

1. [General Questions](#general-questions)
2. [Format and Syntax](#format-and-syntax)
3. [Performance](#performance)
4. [Conversion](#conversion)
5. [Use Cases](#use-cases)
6. [Troubleshooting](#troubleshooting)
7. [Development](#development)

## General Questions

### What is HEDL?

HEDL (Hierarchical Entity Data Language) is a data format designed for token efficiency, human readability, and interoperability. It's particularly useful for AI/ML workflows where reducing token count is important for LLM context windows and API costs.

### Why use HEDL instead of JSON?

HEDL offers several advantages:
- **Token Efficiency**: 40-60% fewer tokens than JSON
- **Type Safety**: Strong typing with validation
- **Readability**: More concise and readable syntax
- **Interoperability**: Easily convert to/from JSON, YAML, XML, CSV, and Parquet
- **References**: Built-in support for entity references
- **Validation**: Built-in syntax and structure validation

### Is HEDL production-ready?

Yes! HEDL is at version 1.0 and includes:
- Comprehensive test suite (unit, integration, property-based, fuzz tests)
- Security hardening (DoS protection, resource limits)
- Performance optimization (zero-copy parsing, SIMD)
- Multiple language bindings (Rust, Python, JavaScript/WASM)
- Production-grade CLI tool

### What are the main use cases?

- **AI/ML Workflows**: Reduce LLM token costs by 40-60%
- **Data Interchange**: Convert between multiple formats
- **Configuration Files**: Human-readable, version-controlled configs
- **Data Pipelines**: Validate and transform data
- **Graph Databases**: Export to Neo4j Cypher
- **Analytics**: Convert to Parquet for efficient queries

## Format and Syntax

### What's the basic syntax?

Every HEDL document starts with a version header:

```hedl
%VERSION: 1.0
---
name "value"
number 42
```

Matrix lists (typed entities) use this syntax:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, bob@example.com
```

### What data types are supported?

- **Strings**: `"hello"` (double-quoted)
- **Integers**: `42`, `-10`
- **Floats**: `3.14`, `-0.5`, `1e10`
- **Booleans**: `true`, `false`
- **Arrays**: `[1, 2, 3]`
- **Objects**: Nested structures
- **References**: `@Type:id`
- **Null**: `~`

### What is a matrix list?

A matrix list is HEDL's efficient way of representing structured data with a common schema:

```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, price]
---
products: @Product
  | p1, Widget, 19.99
  | p2, Gadget, 29.99
  | p3, Doohickey, 9.99
```

This is equivalent to:
```json
{
  "products": [
    {"id": "p1", "name": "Widget", "price": 19.99},
    {"id": "p2", "name": "Gadget", "price": 29.99},
    {"id": "p3", "name": "Doohickey", "price": 9.99}
  ]
}
```

**Benefits:** 60% fewer tokens, clearer structure, automatic validation.

### How do entity references work?

Use `@Type:id` syntax to reference other entities:

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

The reference `@User:u1` links to the user with ID `u1`.

### What is the ditto operator?

The `^` operator repeats the value from the previous row in the same column:

```hedl
%VERSION: 1.0
%STRUCT: Task: [id, status, priority]
---
tasks: @Task
  | t1, pending, high
  | t2, ^, ^           # status="pending", priority="high"
  | t3, complete, ^    # status="complete", priority="high"
```

**Benefit:** Reduces redundancy and token count.

### Can I add comments?

Yes! Use `#` for line comments:

```hedl
%VERSION: 1.0
---
# This is a comment
name "Alice"  # inline comment
```

### What are the nesting limits?

For security, HEDL has these default limits:
- **Maximum nesting depth**: 50 levels (configurable via `max_indent_depth`)
- **Maximum string length**: 1MB per line, 10MB per block string
- **Maximum file size**: 1GB (configurable via `HEDL_MAX_FILE_SIZE`)

These protect against DoS attacks and can be customized when using HEDL as a library.

## Performance

### How fast is HEDL parsing?

HEDL is highly optimized:
- **Efficient parsing**: Minimizes unnecessary allocations during the parsing phase.
- **SIMD optimizations**: Vectorized parsing for strings and numbers
- **Parallel processing**: Batch operations use all CPU cores
- **Benchmarks**: Typically 2-5x faster than equivalent JSON parsers

Example throughput: ~500 MB/sec on modern hardware.

### What's the memory usage?

HEDL is memory-efficient:
- **String handling**: Strings are owned in the final AST for simplicity and safety.
- **Streaming support**: Process large files without loading everything into memory
- **Efficient data structures**: Optimized internal representations

Typical memory usage: ~1.2x the file size (vs 2-3x for many JSON parsers).

### How does token efficiency work?

HEDL reduces tokens through:
1. **Schema reuse**: Matrix lists define structure once
2. **Compact syntax**: Minimal punctuation and keywords
3. **Ditto operator**: Avoid repeating values
4. **Type inference**: Implicit types when obvious

Example comparison (same data):
- **JSON**: 156 tokens
- **HEDL**: 62 tokens
- **Savings**: 60%

### Can HEDL handle large files?

Yes! HEDL supports:
- **Streaming parsing**: Process files larger than memory
- **Configurable limits**: Set `HEDL_MAX_FILE_SIZE` as needed
- **Batch processing**: Parallel processing of multiple files
- **Incremental parsing**: Parse documents incrementally

For very large files (>1GB), consider:
- Splitting into smaller chunks
- Using Parquet for analytics (more efficient for columnar queries)
- Streaming APIs (available in library, not CLI yet)

## Conversion

### Can I convert any JSON to HEDL?

Yes! HEDL can represent any valid JSON structure:

```bash
hedl from-json any.json -o output.hedl
```

The conversion automatically handles structure detection:

```bash
hedl from-json data.json -o optimized.hedl
```

### Is conversion lossless?

Generally yes, but with some caveats:

**Lossless:**
- JSON ↔ HEDL (with `--metadata`)
- YAML ↔ HEDL
- Parquet ↔ HEDL (schema preserved)

**May Lose Information:**
- XML attributes (converted to `_attr_` prefix fields)
- CSV → HEDL (type inference may differ)
- HEDL → CSV (nested structures flattened)

Always validate after conversion:
```bash
hedl from-json data.json | hedl validate -
```

### How do I preserve HEDL types when converting?

Use the `--metadata` flag when converting to JSON:

```bash
# Preserve HEDL structure
hedl to-json data.hedl --metadata --pretty -o with_metadata.json

# Convert back (metadata is automatically recognized)
hedl from-json with_metadata.json -o restored.hedl
```

### Can I convert between non-HEDL formats?

Yes! Use HEDL as an intermediary:

```bash
# JSON → HEDL → YAML
hedl from-json input.json | hedl to-yaml - > output.yaml

# JSON → HEDL → CSV
hedl from-json input.json | hedl to-csv - > output.csv
```

### Which format should I use?

| Use Case | Recommended Format |
|----------|-------------------|
| API responses | JSON (standard) |
| LLM context | HEDL (token-efficient) |
| Configuration | YAML or HEDL |
| Spreadsheets | CSV |
| Analytics | Parquet |
| Graph databases | HEDL + Cypher export |
| Long-term storage | Parquet (compressed) |

## Use Cases

### How do I reduce LLM API costs?

Use HEDL to reduce token count in prompts and context:

```bash
# Convert JSON data to HEDL
hedl from-json large_context.json -o context.hedl

# Check token savings
hedl stats context.hedl --tokens
# Output: 60% fewer tokens than JSON

# Use in LLM prompt (convert back as needed)
hedl to-json context.hedl | llm-tool process
```

**Typical savings**: 40-60% reduction in token count = 40-60% reduction in API costs.

### Can I use HEDL for configuration files?

Absolutely! HEDL is great for configs:

```hedl
%VERSION: 1.0
---
app:
  name: MyApp
  server:
    host: localhost
    port: 8080
  database:
    url: postgresql://localhost/mydb
    pool_size: 10
```

**Benefits:**
- Human-readable
- Type-safe validation
- Version control friendly
- Convert to JSON/YAML for deployment

### How do I use HEDL in data pipelines?

Use HEDL as a validation and conversion hub:

```bash
# Pipeline: CSV → HEDL → Validate → Parquet
hedl from-csv raw_data.csv -t DataRow -o data.hedl
hedl validate data.hedl
hedl lint data.hedl
hedl to-parquet data.hedl -o analytics.parquet
```

### Can I use HEDL with databases?

Yes! Several ways:

**1. Export to SQL:**
Convert HEDL to SQL-compatible formats:
```bash
hedl to-csv data.hedl | psql -c "COPY table FROM STDIN CSV HEADER"
```

**2. Neo4j Graph Database:**
HEDL can represent graph data using references. Export capabilities for Neo4j Cypher are available through the `hedl-neo4j` crate (see developer documentation for API details).

**3. Parquet for Analytics:**
```bash
hedl to-parquet data.hedl -o data.parquet
# Use with Spark, Pandas, DuckDB, etc.
```

## Troubleshooting

### Why am I getting "file too large" errors?

HEDL has a default 1GB file size limit for security. Increase it:

```bash
# Allow 5GB files
export HEDL_MAX_FILE_SIZE=5368709120
hedl validate large_file.hedl
```

### Why is my conversion producing unexpected results?

Common issues:

**1. Type Inference:**
Type inference is applied automatically when converting from JSON/CSV:
```bash
hedl from-json data.json -o output.hedl
```

**2. Encoding Issues:**
Ensure UTF-8 encoding:
```bash
file -i myfile.hedl  # Check encoding
iconv -f ISO-8859-1 -t UTF-8 myfile.hedl > utf8.hedl
```

**3. Line Endings:**
Normalize line endings (especially on Windows):
```bash
dos2unix myfile.hedl  # Unix
unix2dos myfile.hedl  # Windows
```

### How do I debug parsing errors?

Use `inspect` to see the parsed structure:

```bash
# See internal representation
hedl inspect data.hedl

# Validate with detailed errors
hedl validate data.hedl
```

### Why are batch commands slow?

Batch commands use parallelism by default. If slow:

**1. Enable parallelism:**
```bash
hedl batch-validate *.hedl --parallel
```

**2. Adjust thread count:**
```bash
# Use 8 threads
export RAYON_NUM_THREADS=8
hedl batch-format *.hedl --output-dir formatted/ --parallel
```

## Development

### Can I use HEDL as a library?

Yes! HEDL provides Rust, Python, and JavaScript (WASM) libraries:

**Rust:**
```toml
[dependencies]
hedl = "1.0"
```

**Python:**
```python
import hedl
doc = hedl.parse("%VERSION: 1.0\n---\nname: Alice")
json_str = hedl.to_json(doc)
```

**JavaScript (WASM):**
```javascript
import init, { parse, toJson } from './hedl_wasm.js';
await init();
const doc = parse("%VERSION: 1.0\n---\nname: Alice");
const json = toJson(doc);
```

### How do I contribute?

Contributions welcome! See the repository:
- **GitHub**: https://github.com/dweve-ai/hedl
- **Issues**: Report bugs or request features
- **Pull Requests**: Submit improvements

### Is there an LSP/IDE support?

Yes! HEDL includes a Language Server Protocol (LSP) implementation:

**Features:**
- Syntax highlighting
- Auto-completion
- Go-to-definition
- Inline diagnostics
- Hover documentation

See `crates/hedl-lsp` for integration with your editor.

### How do I customize parsing limits?

When using HEDL as a library, you can configure parsing limits through the `ParseOptions` struct. Refer to the API documentation for details on available configuration options.

CLI users can configure the maximum file size using the `HEDL_MAX_FILE_SIZE` environment variable.

### Where can I find more examples?

Check these resources:
- **Examples Guide**: [examples.md](examples.md)
- **Repository Examples**: `examples/` directory
- **Crate Examples**: Each crate has `examples/` subdirectory
- **Tests**: Look at test files for usage patterns

---

## Still Have Questions?

- **Documentation**: Check the [User Guide Index](README.md)
- **CLI Reference**: See [CLI Guide](cli-guide.md)
- **Issues**: Search or ask on [GitHub Issues](https://github.com/dweve-ai/hedl/issues)
- **Troubleshooting**: See [Troubleshooting Guide](troubleshooting.md)
