# HEDL Glossary

Comprehensive glossary of HEDL terminology and concepts.

## Core Concepts

### Canonicalization
The process of converting a HEDL document to a standard, deterministic format. The same data always produces the same canonical form, byte-for-byte.

**See:** [Canonicalization Concept](../concepts/canonicalization.md)

### Ditto Operator
The `^` symbol used to repeat the previous value in a matrix list row, saving tokens and reducing redundancy.

**Example:**
```hedl
%VERSION: 1.0
%STRUCT: Task: [id, status, assignee]
---
tasks: @Task
  | t1, pending, Alice
  | t2, ^, ^                    # status="pending", assignee="Alice"
```

### Entity
A named collection of data in HEDL, similar to a JSON object key or database table.

**Example:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

### Matrix List
A table-like structure in HEDL with a schema (column definitions) and rows of data, optimized for token efficiency.

**Example:**
```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, price]
---
products: @Product
  | p1, Laptop, 999.99
  | p2, Mouse, 29.99
```

### Reference
A pointer from one entity to another using `@TypeName:id` syntax.

**Example:**
```hedl
%VERSION: 1.0
%STRUCT: Post: [id, author]
---
posts: @Post
  | p1, @User:u1
```

### Type Annotation
The `@TypeName` prefix that specifies the type of an entity.

**Example:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

### Type Inference
HEDL's automatic determination of value types based on syntax (quotes = string, digits = number, etc.).

**See:** [Type System Concept](../concepts/type-system.md)

---

## Data Types

### Boolean
`true` or `false` values.

**Example:**
```hedl
%VERSION: 1.0
---
active: true
verified: false
```

### Null
The absence of a value, represented by `~`.

**Example:**
```hedl
%VERSION: 1.0
---
email: ~
```

### Number
Integer or floating-point numeric values.

**Examples:**
```hedl
%VERSION: 1.0
---
age: 30
price: 99.99
temperature: -5.2
```

### String
Text values, optionally enclosed in double quotes. Bare strings are supported for simple text without special characters.

**Examples:**
```hedl
%VERSION: 1.0
---
name: "Alice"      # Quoted string
role: admin        # Bare string
```

---

## Structure

### Column Definition
The column specification in a `%STRUCT:` declaration that defines the fields.

**Example:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
```

### Header
The first line of a HEDL document: `%VERSION: 1.0` followed by optional `%STRUCT:` declarations and the `---` separator.

### Indentation
HEDL uses exactly 2 spaces per nesting level to indicate structure.

### Nesting
Hierarchical organization of entities within other entities.

**Example:**
```hedl
%VERSION: 1.0
%STRUCT: Employee: [id, name]
---
employees: @Employee
  | e1, Alice
```

### Row
A line of data in a matrix list, consisting of an ID and values matching the column definition.

**Example:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

---

## Operations

### Batch Processing
Processing multiple files in a single operation, optionally in parallel.

**Commands:**
- `batch-validate`
- `batch-format`
- `batch-lint`

### Conversion
Transforming data between HEDL and other formats (JSON, YAML, XML, CSV, Parquet).

**Commands:**
- `to-json`, `from-json`
- `to-yaml`, `from-yaml`
- etc.

### Formatting
Converting a HEDL document to canonical form.

**Command:** `format`

### Linting
Checking a HEDL document for best practices and potential issues.

**Command:** `lint`

### Streaming
Processing large files incrementally without loading them entirely into memory.

**Status:** Streaming support is currently in development for future releases.

### Validation
Checking a HEDL document for syntax errors, type mismatches, and reference integrity.

**Command:** `validate`

---

## File Formats

### CSV
Comma-Separated Values, a flat tabular format.

### JSON
JavaScript Object Notation, a hierarchical key-value format.

### Parquet
Apache Parquet, a columnar storage format for analytics.

### XML
eXtensible Markup Language, a hierarchical markup format.

### YAML
YAML Ain't Markup Language, a human-readable configuration format.

---

## Configuration

### Chunk Size
Number of rows processed at a time in streaming mode (planned feature).

**Status:** Streaming support is currently in development.

### Maximum File Size
Maximum size of a file that can be processed.

**Environment Variable:** `HEDL_MAX_FILE_SIZE`

**Default:** 1GB (1073741824 bytes)

---

## CLI Terms

### Exit Code
Numeric value returned by a command indicating success (0) or failure (non-zero).

### Parallel Processing
Running operations concurrently across multiple CPU cores.

**Option:** `--parallel`

### Pretty Printing
Formatting output with indentation and line breaks for readability.

**Option:** `--pretty` (JSON, XML)

### Standard Input (stdin)
Input from a pipe or redirection, indicated by `-` in commands.

**Example:**
```bash
cat data.hedl | hedl validate -
```

### Standard Output (stdout)
Default destination for command output, can be piped or redirected.

---

## Advanced Concepts

### Reference Integrity
Ensuring that all references point to existing entities.

### Schema Inference
Automatically determining the structure and types of data.

### Token Efficiency
The amount of text/tokens required to represent data, crucial for LLM applications.

### Type-Scoped IDs
IDs that are unique within their entity type, enabling unambiguous references.

**Example:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | u1, Alice

posts: @Post
  | p1, @User:u1    # Scoped to User type
```

---

## Acronyms

- **CLI** - Command-Line Interface
- **CSV** - Comma-Separated Values
- **DOS** - Denial of Service
- **FFI** - Foreign Function Interface
- **HEDL** - Hierarchical Entity Data Language
- **JSON** - JavaScript Object Notation
- **LLM** - Large Language Model
- **LSP** - Language Server Protocol
- **MCP** - Model Context Protocol
- **MIME** - Multipurpose Internet Mail Extensions
- **REPL** - Read-Eval-Print Loop
- **TOML** - Tom's Obvious Minimal Language
- **TTY** - Teletypewriter (terminal)
- **URI** - Uniform Resource Identifier
- **UTF-8** - 8-bit Unicode Transformation Format
- **WASM** - WebAssembly
- **XML** - eXtensible Markup Language
- **YAML** - YAML Ain't Markup Language

---

**Related:**
- [Concepts](../concepts/) - Deep-dive explanations
- [CLI Commands](cli-commands.md) - Command reference
- [Configuration](configuration.md) - Settings reference
