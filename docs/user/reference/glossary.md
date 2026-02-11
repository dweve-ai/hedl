# HEDL Glossary

Comprehensive glossary of HEDL terminology and concepts.

## Core Concepts

### Alias
A global string constant defined by `%ALIAS` directive and referenced using `%key` syntax. Aliases provide token savings by defining reusable values once.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%ALIAS:yes:true
%ALIAS:no:false
---
config:
 enabled:%yes
 debug:%no
```

### Canonicalization
The process of converting a HEDL document to a standard, deterministic format. The same data always produces the same canonical form, byte-for-byte. Includes normalized directive ordering, consistent quoting, and consistent spacing.

**See:** [Canonicalization Concept](../concepts/canonicalization.md)

### Complex Mode
Using HEDL with full schema definitions (`%STRUCT` directives) and matrix lists for structured, typed data. Enables references and strong validation. Recommended for AI/ML datasets and relational data.

**See:** [Simple Mode](#simple-mode)

### Column
A named field in a matrix list schema, defined in the `%STRUCT` directive. All rows must provide values for each column in order.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |u1,Alice,alice@example.com
```

The columns are: `id`, `name`, `email`.

### Entity
A named collection of data in HEDL, similar to a JSON object key or database table. An entity can be a matrix list, an object, or a scalar value.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
 |u1,Alice
 |u2,Bob
```

Here, `users` is an entity of type `User`.

### Expression
An opaque computation token using `$(...)` syntax. The parser does not evaluate expressions; they are preserved as-is for downstream processing.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
config:
 formula: $(x + y)
 function: $(concat("hello", "world"))
```

### ID Token
A unique identifier for a node in a matrix list, typically the first column value. IDs are ASCII-only and must match the pattern `[a-z_][a-z0-9_\-]*`.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
 |user_1,Alice
 |user-2,Bob
```

Valid IDs: `user_1`, `user-2`, `_system`. Invalid IDs: `User1` (uppercase), `123user` (starts with digit).

### Key Token
A field name in objects or column names in matrix lists. Keys must be lowercase ASCII and match the pattern `[a-z_][a-z0-9_]*`.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
user_data:
 first_name:Alice
 email_address:alice@example.com
```

Valid keys: `user_data`, `first_name`, `_private`. Invalid keys: `firstName` (camelCase), `first-name` (hyphen).

### List Frame
An internal stack frame during parsing representing an active matrix list, tracking its schema, rows, and current position.

**See:** [Parser Architecture](../../developer/concepts/parser-architecture.md)

### List Literal
A parenthesized sequence of scalar values `(elem1, elem2, ...)` for representing heterogeneous lists, distinct from tensor literals.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
config:
 roles:(admin,editor,viewer)
 mixed:(1,"two",true,~)
```

### Local Reference
A reference to a node within the same type namespace using `@id` syntax (without explicit type name).

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,manager]
---
users:@User
 |alice,Alice,@bob
 |bob,Bob,~
```

Here, `@bob` is a local reference (searches within the `User` namespace).

### Matrix List
A table-like structure in HEDL with a schema (column definitions) and rows of data, optimized for token efficiency. Each row is prefixed with `|` and contains comma-separated values matching the schema columns.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Product:[id,name,price]
---
products:@Product
 |p1,Laptop,999.99
 |p2,Mouse,29.99
```

### Matrix Row
A single line of data in a matrix list, starting with `|` and containing comma-separated values corresponding to the schema columns.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,age]
---
users:@User
 |u1,Alice,30
 |u2,Bob,25
```

Each of the last two lines is a matrix row.

### Node
A data structure representing a typed entity with an ID, properties, and optional child entities. Nodes are created implicitly when matrix rows are parsed with a type annotation.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
 |u1,Alice
```

The row `|u1,Alice` creates a Node with ID `u1`, type `User`, and property `name: "Alice"`.

### Node Identity
A stable unique identifier for a node, typically the first column value in a matrix row. Enables references between nodes.

**See:** [Identity and Graph Semantics](../../developer/concepts/parser-architecture.md)

### Null Value
The absence of a value, represented by the null symbol (default `~`). The null symbol is configurable via `%NULL` directive.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |u1,Alice,alice@example.com
 |u2,Bob,~
```

The second user has no email value (null).

### Qualified Reference
A reference to a node in a specific type namespace using `@TypeName:id` syntax. Required when referencing nodes of different types.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Post:[id,author]
---
users:@User
 |u1,Alice

posts:@Post
 |p1,@u1
```

`@User:u1` is a qualified reference to the user `u1`.

### Reference
A pointer from one entity to another using `@id` or `@TypeName:id` syntax. References are type-scoped, allowing unambiguous linking between nodes.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Post:[id,author]
---
posts:@Post
 |p1,@u1
```

`@User:u1` references a user node with ID `u1`.

### Root Object
The implicit top-level object containing all body data in a HEDL document.

**See:** [Data Model Concept](../concepts/data-model.md)

### Row Scope
The most recently parsed matrix row in a list frame, serving as the attachment point for child lists nested under that row.

**See:** [Parser Architecture](../../developer/concepts/parser-architecture.md)

### Type Annotation
The `@TypeName` prefix that specifies the type of a matrix list. Used in entity declarations like `users:@User` to bind a list to a schema.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
 |u1,Alice
```

`@User` is the type annotation.

### Type Inference
HEDL's automatic determination of value types based on syntax: quoted strings are strings, bare digits are numbers, `true`/`false` are booleans, `~` is null, `@` starts a reference, `[` starts a tensor, `$(` starts an expression.

**See:** [Type System Concept](../concepts/type-system.md)

### TypeName Token
A struct name used in schema declarations and type references. TypeNames must use PascalCase (start with uppercase) and match the pattern `[A-Z][A-Za-z0-9]*`.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:BlogPost:[id,title,author]
---
users:@User
 |u1,Alice
```

Valid TypeNames: `User`, `BlogPost`, `Item123`. Invalid TypeNames: `user` (lowercase), `123Item` (starts with digit).

---

## Data Types

### Boolean
`true` or `false` keyword values representing logical truth values.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
active:true
verified:false
```

### Null
The absence of a value, represented by the null symbol (configurable, default `~`). Set via the `%NULL` directive.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
email:~
phone:~
```

### Number
Integer or floating-point numeric values. Integers range from -2^63 to 2^63-1. Floats are IEEE 754 double precision, supporting scientific notation.

**Examples:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
age:30
price:99.99
temperature:-5.2
scientific:1.23e-4
```

### String
Text values, optionally enclosed in quotes (configurable quote character, default `"`). Bare strings are supported for simple text without special characters. Quote character is set via `%QUOTE` directive.

**Examples:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
name:"Alice"            # Quoted string
role:admin              # Bare string
phrase:"hello world"    # Quoted (contains space)
```

### Tensor
A multi-dimensional numerical array literal using square brackets `[1, 2, 3]` or `[[1, 2], [3, 4]]`. Tensors contain only numbers and whitespace; mixed types are invalid.

**Examples:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
vector:[1,2,3]
matrix:[[1.0,2.0],[3.0,4.0]]
nested:[[[1,2],[3,4]],[[5,6],[7,8]]]
```

---

## Structure

### Alias Registry
An internal mapping of alias names (e.g., `%key`) to their string values, populated during header parsing from `%ALIAS` directives.

### Body
The data section of a HEDL document, containing all entities, objects, and values. Follows the `---` separator and the header section.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
--- ← Separator between header and body
users:@User    ← Body starts here
 |u1,Alice
```

### Column Definition
The column specification in a `%STRUCT:` declaration that defines the fields. Each column is a Key Token and defines the order and names for values in matrix rows.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |u1,Alice,alice@example.com
```

The columns are: `[id, name, email]`.

### Context Stack
An internal stack during parsing that tracks active scopes and controls what node types are allowed at each nesting level.

**See:** [Parser Architecture](../../developer/concepts/parser-architecture.md)

### Directive
A configuration line in the header starting with `%`, such as `%V:2.0` (version), `%S:User:[...]` (schema), or `%NULL:~` (null symbol). Directives must appear in dependency order.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
```

### Header
The configuration section at the top of a HEDL document, containing directives like `%V:` (version), `%S:` (schema), `%NULL:` (null symbol), `%QUOTE:` (quote character), and others. The header ends with the `---` separator.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
--- ← Separator (end of header)
```

### Indentation
Significant whitespace using exactly 1 space per nesting level (v2.0) to indicate hierarchical structure. Tabs are not allowed. Indentation level determines scope containment.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
company:         # Indent level 0
 name:Acme       # Indent level 1
 employees:      # Indent level 1
  |e1,Alice      # Indent level 2
```

### Nesting
Hierarchical organization of entities within other entities. Child entities are indented one space further than their parent.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Employee:[id,name]
---
company:
 name:TechCorp
 employees:@Employee
  |e1,Alice
```

The `employees` list is nested under `company`.

### Node Registry
An internal global mapping of `ID → Node` populated during parsing, enabling reference resolution.

**See:** [Parser Architecture](../../developer/concepts/parser-architecture.md)

### Object
A key-value collection at a specific indentation level, containing scalar values or nested collections.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
config:
 host:localhost
 port:8080
 ssl:true
```

`config` is an object with three key-value pairs.

### Object Frame
An internal stack frame during parsing representing an object mapping scope.

**See:** [Parser Architecture](../../developer/concepts/parser-architecture.md)

### Row
A line of data in a matrix list, consisting of an ID and values matching the column definition. Each row starts with `|` and contains comma-separated values.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,age]
---
users:@User
 |u1,Alice,30
 |u2,Bob,25
```

The last two lines are matrix rows.

### Separator
The `---` line that separates the header section from the body section in a HEDL document.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
--- ← Separator
users:@User
```

### Schema
A type definition declaring the structure of a matrix list. Defined using `%S:TypeName:[col1,col2,...]` in the header. Each schema maps a TypeName to ordered columns.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
%S:Post:[id,title,author]
---
users:@User
 |u1,Alice,alice@example.com
```

### Schema Registry
An internal mapping of `TypeName → Columns[]` populated during header parsing from `%S` directives.

### Scoped ID
A node ID that is unique within its type namespace, enabling unambiguous references. For example, `User:admin` and `Role:admin` can coexist because their IDs are scoped to their types.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Role:[id,description]
---
users:@User
 |admin,Alice

roles:@Role
 |admin,Administrator
```

Here, both `User` and `Role` have an entity with ID `admin`, distinguished by their type scope.

### Struct
Synonym for [Schema](#schema). Used interchangeably; `%S:` and `%STRUCT:` are equivalent directives.

### Simple Mode
Using HEDL without schemas (`%STRUCT` directives), similar to JSON or YAML. Data is represented as objects and key-value pairs without type annotations or matrix lists. Good for configuration files and unstructured data.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
config:
 host:localhost
 port:8080
 ssl:true
```

**See:** [Complex Mode](#complex-mode)

### Indent Level
The nesting depth indicated by leading spaces in a line. In HEDL v2.0, exactly 1 space = 1 level. The indent level determines scope and containment.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
company:               # Indent level 0
 name:TechCorp         # Indent level 1 (1 space)
 location:NYC          # Indent level 1
 departments:          # Indent level 1
  |d1,Engineering      # Indent level 2 (2 spaces)
```

### Truncation Detection
A validation mechanism to detect incomplete HEDL documents that end in the middle of a structure (incomplete matrix row, unclosed object, etc.).

---

## Directives and Configuration

### Alias Directive
The `%ALIAS:key:value` directive defines a global string constant that can be reused throughout the document using `%key` syntax.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%ALIAS:yes:true
%ALIAS:no:false
---
config:
 enabled:%yes
 debug:%no
```

### Nest Directive
The `%N:ParentType>ChildType` directive declares which types can be nested under parent types. Useful for documentation and enabling nested matrix lists.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Department:[id,name]
%S:Employee:[id,name]
%N:Department>Employee
---
departments:@Department
 |d1,Engineering
  employees:@Employee
   |e1,Alice
   |e2,Bob
```

### Null Directive
The `%NULL:symbol` directive (REQUIRED in v2.0) defines the character representing null values. Default: `~`. Example: `%NULL:~`, `%NULL:.`, `%NULL:_`.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
user:
 email:~
```

### Quote Directive
The `%QUOTE:symbol` directive (REQUIRED in v2.0) defines the character for quoting strings. Default: `"`. Example: `%QUOTE:"`, `%QUOTE:'`.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
greeting:"hello world"
```

### Schema Directive
The `%S:TypeName:[col1,col2,...]` directive (also `%STRUCT:`) defines a typed matrix list schema. The TypeName must be PascalCase and columns must be Key Tokens.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |u1,Alice,alice@example.com
```

### Version Directive
The `%V:2.0` directive (REQUIRED, must be first line) specifies the HEDL version.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
```

---

## Operations

### Batch Processing
Processing multiple files in a single operation, optionally in parallel across CPU cores.

**Commands:**
- `batch-validate` - Validate multiple files
- `batch-format` - Format multiple files
- `batch-lint` - Lint multiple files
- `batch-convert` - Convert multiple files

### Canonical Form
The standard deterministic representation of a HEDL document. Same input always produces identical byte-for-byte output. Used for round-trip verification and Git-friendly diffs.

**Command:** `format`

### Conversion
Transforming data between HEDL and other formats (JSON, YAML, XML, CSV, Parquet).

**Commands:**
- `to-json` / `from-json` - JSON conversion
- `to-yaml` / `from-yaml` - YAML conversion
- `to-csv` / `from-csv` - CSV conversion
- `to-xml` / `from-xml` - XML conversion
- `to-parquet` / `from-parquet` - Parquet conversion
- `to-jsonl` - Newline-delimited JSON

### Formatting
Converting a HEDL document to canonical form with normalized indentation, directive ordering, and quoting.

**Command:** `format`

**Options:**
- `--check` - Verify if document is already canonical
- `--diff` - Show formatting changes

### Linting
Checking a HEDL document for best practices, style violations, and potential issues.

**Command:** `lint`

**Checks:**
- Unused schemas
- Inefficient encodings
- Style violations
- Potential data errors

### Reference Validation
Checking that all `@TypeName:id` references point to existing nodes of the correct type. Part of the validation process.

**See:** [Reference Integrity](../concepts/references.md)

### Round-trip Stability
The property that parsing and regenerating a HEDL document produces minimal diff noise. Ensures canonical output is stable across runs.

### Schema Validation
Checking that matrix rows match their declared schema (column count, value types, required fields).

### Streaming
Processing large files incrementally without loading them entirely into memory. Enabled via `--streaming` flag.

**Status:** Streaming support is in development for future releases.

### Validation
Checking a HEDL document for syntax errors, type mismatches, reference integrity, and schema compliance.

**Command:** `validate`

**Checks:**
- Syntax correctness
- Header validity
- Schema conformance
- Reference integrity
- Truncation detection

---

## Performance Concepts

### Zero-Copy Optimization
A design pattern where data is not duplicated during parsing. References are resolved via indices rather than copying node data.

**See:** [Zero-Copy Design](../../developer/concepts/zero-copy-optimizations.md)

---

## File Formats

### CSV (Comma-Separated Values)
A flat tabular format consisting of rows of comma-separated values. HEDL can convert matrix lists to/from CSV.

**HEDL to CSV:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |u1,Alice,alice@example.com
```

**CSV output:**
```
id,name,email
u1,Alice,alice@example.com
```

### JSON (JavaScript Object Notation)
A hierarchical key-value format. HEDL documents map naturally to JSON objects and arrays.

**HEDL:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
 |u1,Alice
```

**JSON equivalent:**
```json
{"users": [{"id": "u1", "name": "Alice"}]}
```

### JSONL (Newline-Delimited JSON)
Variant of JSON where each line is a complete JSON object, useful for streaming large datasets.

### Parquet
Apache Parquet, a columnar storage format for analytics with compression and schema inference.

### YAML (YAML Ain't Markup Language)
A human-readable configuration format with indentation-based structure similar to HEDL but with different syntax.

### XML (eXtensible Markup Language)
A hierarchical markup format using tags. HEDL entities map to XML elements.

---

## Configuration

### Chunk Size
Number of rows processed at a time in streaming mode.

**Status:** Streaming support is fully implemented in the `hedl-stream` crate.

**Environment Variable:** `HEDL_CHUNK_SIZE`

**Default:** 10000 rows

### HEDL Max Depth
Maximum nesting depth allowed in a document.

**Environment Variable:** `HEDL_MAX_DEPTH`

**Default:** 50 levels

### HEDL Max File Size
Maximum size of a file that can be processed.

**Environment Variable:** `HEDL_MAX_FILE_SIZE`

**Default:** 1GB (1073741824 bytes)

### HEDL Max Line Length
Maximum length of a single line in characters.

**Environment Variable:** `HEDL_MAX_LINE_LENGTH`

**Default:** 1MB (1048576 characters)

### Parallel Processing
Enable concurrent processing across multiple CPU cores for batch operations.

**Option:** `--parallel` or `--jobs N`

**Default:** Auto-detect CPU count

---

## CLI Terms

### Exit Code
Numeric value returned by a command indicating success (0) or failure (non-zero).

**Standard codes:**
- `0` - Success
- `1` - General error
- `2` - Parse error
- `3` - Validation error
- `4` - I/O error

### Metadata Flag
Option to include type and schema information in output formats.

**Option:** `--metadata` (JSON, YAML)

**Example:**
```bash
hedl to-json data.hedl --metadata
```

### Output Format
Target format for conversion commands.

**Option:** `-o format` or `--output-format format`

**Available:** `json`, `yaml`, `xml`, `csv`, `jsonl`, `parquet`, `hedl`

### Pretty Printing
Formatting output with indentation and line breaks for readability (human-friendly).

**Option:** `--pretty` (JSON, XML, YAML)

**Example:**
```bash
hedl to-json data.hedl --pretty
```

### Standard Input (stdin)
Input from a pipe or redirection, indicated by `-` in commands.

**Example:**
```bash
cat data.hedl | hedl validate -
```

### Standard Output (stdout)
Default destination for command output, can be piped or redirected.

**Example:**
```bash
hedl format data.hedl > formatted.hedl
```

### Strict Mode
Parsing mode where all syntax and semantic rules are enforced strictly. Default mode.

**See:** [Mode Directive](#mode-directive)

### Lenient Mode
Parsing mode where some errors are warnings rather than failures. Configured via `%MODE:lenient`.

---

## Advanced Concepts

### Cell Value Inference
The process of determining a value's type during matrix row parsing, applying the scalar inference ladder (quotes = string, digits = number, etc.).

**See:** [Type Inference](#type-inference)

### Document Normalization
The process of converting a HEDL document to its canonical form, ensuring consistent byte-for-byte output across different generators.

**See:** [Canonicalization](#canonicalization)

### Graph Structure
The representation of data as nodes (entities) connected by references (edges). HEDL supports arbitrary directed graphs via `@TypeName:id` references.

**See:** [References Concept](../concepts/references.md)

### Implicit Child Lists
Automatic parent-child attachment via nesting rules without explicit container declaration. A nested matrix list under a row becomes a child of that row.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Company:[id,name]
%S:Employee:[id,name]
%N:Company>Employee
---
companies:@Company
 |c1,Acme
  employees:@Employee
   |e1,Alice
```

The `employees` list is nested under the `c1` company row.

### Indentation-Driven Structure
HEDL's use of significant whitespace (1 space per level) to represent hierarchical structure, eliminating the need for brackets or explicit delimiters.

**See:** [Indentation](#indentation)

### Inference Ladder
The priority-ordered sequence of type checks applied to unquoted values: null (`~`), boolean (`true`/`false`), number (digits), reference (`@id`), expression (`$()`), string (default).

**See:** [Type Inference](#type-inference)

### Positional Encoding
The use of column position in a matrix list to implicitly map values to field names, reducing token overhead compared to explicit field names in each row.

**Example:**
```hedl
%S:User:[id,name,email]
---
users:@User
 |u1,Alice,alice@example.com  # Position 1 = id, 2 = name, 3 = email
```

### Reference Graph
The directed graph formed by reference relationships between nodes. Enables modeling of complex relational data structures.

**See:** [References Concept](../concepts/references.md)

### Reference Integrity
Verification that all `@TypeName:id` references point to existing nodes of the correct type, and that reference graphs are acyclic where required.

**See:** [References Concept](../concepts/references.md)

### Row Shape Validation
Verification that each matrix row has the correct number of columns and that values match expected type patterns from the schema.

### Schema Inference
Automatically determining the structure and types of data by analyzing a sample of rows. Implemented in the CSV and JSON converters (`from_csv`, `from_json`).

### Token Efficiency
The amount of text/tokens required to represent data, crucial for LLM applications. HEDL achieves ≤50% token count compared to JSON through schema-defined matrices, positional encoding, and optional compression techniques.

**See:** [Token Efficiency](#token-efficiency) under Performance Concepts.

### Type-Scoped IDs
Node IDs that are unique within their type namespace, enabling unambiguous references. For example, a `User` with ID `admin` and a `Role` with ID `admin` are distinct because their IDs are scoped to their types.

**Example:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Role:[id,description]
---
users:@User
 |admin,Alice

roles:@Role
 |admin,Administrator

config:
 admin_user:@admin    # Qualified reference to User
 admin_role:@admin    # Qualified reference to Role
```

### Unique Constraints
Rules ensuring data integrity in matrix lists: IDs must be unique within their type, all rows must have the same column count, ID column cannot be null.

### Value Coercion
The ability to represent any value as a string by quoting it, allowing flexible type handling in loosely-typed contexts.

---

## Acronyms

- **ASCII** - American Standard Code for Information Interchange
- **BOM** - Byte Order Mark
- **CLI** - Command-Line Interface
- **CRLF** - Carriage Return + Line Feed (Windows line ending)
- **CSV** - Comma-Separated Values
- **DOS** - Denial of Service
- **FFI** - Foreign Function Interface
- **HEDL** - Hierarchical Entity Data Language
- **ID** - Identifier
- **IEEE** - Institute of Electrical and Electronics Engineers
- **JSON** - JavaScript Object Notation
- **JSONL** - Newline-Delimited JSON
- **LF** - Line Feed (Unix line ending)
- **LLM** - Large Language Model
- **LSP** - Language Server Protocol
- **MCP** - Model Context Protocol
- **MIME** - Multipurpose Internet Mail Extensions
- **NFC** - Unicode Normalization Form C
- **PascalCase** - Capitalized compound word style (e.g., `UserProfile`)
- **REPL** - Read-Eval-Print Loop
- **RFC** - Request For Comments (IETF standard)
- **TOML** - Tom's Obvious Minimal Language
- **TTY** - Teletypewriter (terminal)
- **URI** - Uniform Resource Identifier
- **UTF-8** - 8-bit Unicode Transformation Format
- **WASM** - WebAssembly
- **XML** - eXtensible Markup Language
- **YAML** - YAML Ain't Markup Language

---

## Token Categories

HEDL recognizes several token types with specific syntax rules:

- **Key Token** - Object and column names: `[a-z_][a-z0-9_]*`
- **TypeName Token** - Schema names: `[A-Z][A-Za-z0-9]*`
- **ID Token** - Entity identifiers: `[a-z_][a-z0-9_\-]*`
- **Reference Token** - Node pointers: `@([TypeName]:)?id`
- **Alias Token** - Alias references: `%[a-z_][a-z0-9_]*`
- **Expression Token** - Opaque computations: `$(...)` with balanced parentheses
- **Directive Token** - Configuration: `%Name` or `%VERBOSE_NAME`

---

## Related Documentation

- [Concepts](../concepts/) - Deep-dive explanations
  - [Data Model](../concepts/data-model.md) - Structure and organization
  - [Type System](../concepts/type-system.md) - Type inference and validation
  - [References](../concepts/references.md) - Node relationships
  - [Canonicalization](../concepts/canonicalization.md) - Standard formatting
- [CLI Guide](../cli-guide.md) - Command reference and examples
- [Configuration](configuration.md) - Settings and environment variables
- [Examples](../examples.md) - Real-world HEDL examples
- [FAQ](../faq.md) - Frequently asked questions
- [Specification](../../SPEC.md) - Complete language specification
