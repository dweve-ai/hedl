# HEDL Specification v1.0.0

Hierarchical Entity Data Language

**Status:** Standards Track  
**Version:** 1.0.0  
**MIME Type:** `application/hedl`  
**File Extension:** `.hedl`  
**Release Date:** 2025-12-25  

**Notice:** This is the original release of HEDL v1.0.0, incorporating all fixes from review feedback before initial publication.

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Conformance and Terminology](#2-conformance-and-terminology)
3. [Design Goals](#3-design-goals)
4. [Lexical Structure](#4-lexical-structure)
5. [Document Structure](#5-document-structure)
6. [Header Section](#6-header-section)
7. [Body Section](#7-body-section)
8. [Scalars in Key-Value Pairs](#8-scalars-in-key-value-pairs)
9. [Matrix Row and Cell Parsing](#9-matrix-row-and-cell-parsing)
10. [Identity and Graph Semantics](#10-identity-and-graph-semantics)
11. [Parsing Algorithm (Normative)](#11-parsing-algorithm-normative)
12. [Error Hierarchy](#12-error-hierarchy)
13. [Canonicalization (Generators)](#13-canonicalization-generators)
14. [Security Considerations](#14-security-considerations)
15. [IANA Considerations](#15-iana-considerations)
16. [Normative Examples](#16-normative-examples)
17. [Extensions and Versioning](#17-extensions-and-versioning)
18. [Implementation Requirements](#18-implementation-requirements)
19. [Compliance and Interoperability](#19-compliance-and-interoperability)
20. [Appendix A: Implementation Guidelines](#appendix-a-implementation-guidelines)
21. [Appendix B: Conformance Test Suite](#appendix-b-conformance-test-suite)
22. [Appendix C: Migration Guide](#appendix-c-migration-guide)
23. [Appendix D: Performance Guidelines](#appendix-d-performance-guidelines)
24. [Appendix E: Formal Grammar](#appendix-e-formal-grammar)
25. [Appendix F: Frequently Asked Questions](#appendix-f-frequently-asked-questions)
26. [Appendix G: Complete Implementation Reference](#appendix-g-complete-implementation-reference)
27. [Appendix H: Format Comparisons](#appendix-h-format-comparisons-informational)

---

## 1. Introduction

HEDL (Hierarchical Entity Data Language) is a text-based data serialization format optimized for AI/ML workflows, combining the minimal token overhead of CSV with the structural expressiveness of JSON/YAML and the relational semantics of graph databases. Designed specifically for efficient representation in large language model (LLM) context windows, HEDL achieves significant structural token reduction while maintaining deterministic parsing and human readability.

### 1.1 Core Innovations

* **Schema-defined positional matrices**: Typed lists encoded as CSV-like rows with implicit column mapping
* **Strict indentation as structure**: Eliminates brackets and explicit delimiters through consistent 2-space indentation
* **Document-wide identity system**: Global IDs enable graph relationships without duplication
* **Implicit child lists**: Automatic parent-child attachment via nesting rules without explicit container declaration
* **Scoped ditto operator**: Repeats previous values within bounded contexts, reducing redundancy
* **Alias system**: Global constants for token substitution and schema sharing
* **Simple and complex modes**: Progressive disclosure from basic key-value pairs to full schematized graphs
* **Tensor literals**: Built-in support for numerical arrays in AI/ML workflows

### 1.2 Data Model

HEDL represents data as a typed graph where:
- Each node has a stable string identity (ID)
- Nodes are typed via schemas defining ordered columns
- Relationships are established through references (`@id`)
- Hierarchical structure is represented via nesting
- Scalar values follow a deterministic inference ladder
- Both schematized and schema-less data are supported

### 1.3 Design Philosophy

1. **Progressive disclosure**: Simple use cases require minimal syntax; advanced features are optional
2. **Fail fast**: Syntax and semantic errors caught early during parsing
3. **Token minimalism**: Structural characters minimized for LLM efficiency
4. **Round-trip stability**: Parsed data can be regenerated in canonical form
5. **Extensible core**: Versioning and expression system allow future extension
6. **Deterministic parsing**: No ambiguous constructs; same input always yields same output
7. **Truncation detection**: Partial files can be detected and rejected
8. **Internationalization support**: Full Unicode in data values with clear ASCII/Unicode boundaries

---

## 2. Conformance and Terminology

### 2.1 Key Words

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as described in RFC 2119.

### 2.2 Document Terms

* **Header**: Directive section that configures parsing registries (Section 6)
* **Body**: Data section containing objects, lists, and matrix rows (Section 7)
* **Separator**: The `---` line dividing Header and Body (Section 5.1)
* **Root Object**: The implicit top-level object containing all Body data
* **Simple Mode**: Using HEDL without schemas, similar to JSON/YAML
* **Complex Mode**: Using HEDL with full schema definitions and matrix lists

### 2.3 Parsing Terms

* **Indent Level**: `LeadingSpaces / 2` (integer division, Section 4.3)
* **Schema Registry**: Map of `TypeName → ordered Columns[]` defined by `%STRUCT` directives
* **Matrix List**: A keyed list of typed rows, initiated by `key: @TypeName` or `key: @TypeName[...]`
* **Matrix Row**: A `|`-prefixed CSV record parsed according to its enclosing list's schema
* **Context Stack**: Stack of active scopes controlling what node types are allowed
* **List Frame**: Stack frame representing an active matrix list, tracking schema and row state
* **Object Frame**: Stack frame representing an object mapping scope
* **Row Scope**: The most recently parsed row in a list frame, serving as attachment point for child lists
* **Node Registry**: Global mapping of `ID → Node` populated during parsing
* **Alias Registry**: Global mapping of `%key → string` defined by `%ALIAS` directives
* **Truncation State**: Tracks whether the document ends in the middle of a structure

### 2.4 Data Terms

* **Reference**: Token starting with `@` referencing a node by its ID
* **Expression**: Opaque token `$(...)` for embedding computation expressions (not evaluated by parser)
* **Ditto**: The `^` token copying the previous row's value in the same column
* **Alias**: Global string constant defined by `%ALIAS` and referenced by `%key`
* **Node**: Data structure representing a typed entity with properties and optional children
* **Tensor**: Multi-dimensional array literal for numerical data: `[[1, 2], [3, 4]]`

### 2.5 Data Model Principles

* **Object Keys**: Object key ordering has no semantic significance; parsers MAY preserve order for round-trip but MUST NOT depend on order
* **Node Identity**: Each node is identified by a string ID stored in the first column of its schema
* **Graph Structure**: Relationships are established via references (`@id`) and nested lists
* **Deterministic Parsing**: Identical input yields identical structure without heuristics or configuration
* **Scoped IDs**: Node IDs are unique within their type namespace (e.g., `User:admin` and `Role:admin` can coexist)

---

## 3. Design Goals

### 3.1 Primary Objectives

1. **Token Efficiency**: Minimize structural overhead for LLM context windows
   - Target: ≤50% token count compared to JSON for typical AI datasets
   - Achieved through: implicit structure, ditto operator, positional encoding, optional schemas

2. **Deterministic Parsing**: Identical input yields identical structure without heuristics
   - No ambiguous grammar
   - Strict validation catches errors early
   - No configuration-dependent parsing

3. **Progressive Disclosure**: Simple use cases require minimal learning
   - Start with key-value pairs like JSON/YAML
   - Add schemas and matrix lists only when needed
   - Clear migration path from simple to complex

4. **Graph-native**: Support references and relationships as first-class citizens
   - Scoped ID namespace
   - Directed relationship support via references
   - Efficient relationship encoding

5. **Strict Validation**: Catch errors early with schema validation and structural rules
   - Pre-parse schema validation
   - Real-time shape checking
   - Comprehensive error messages

6. **Truncation Detection**: Detect and reject incomplete files
   - Validate all structures are properly closed
   - Detect incomplete matrix rows
   - Reject files ending mid-token

### 3.2 Secondary Objectives

7. **Round-trip Stability**: Parsed data can be regenerated with minimal diff noise
8. **Streaming Support**: Ability to parse large files incrementally
9. **Schema Evolution**: Forward/backward compatibility considerations
10. **Tooling Ecosystem**: Support for validation, transformation, visualization tools

### 3.3 Target Use Cases

* **AI/ML dataset serialization**: Training examples, embeddings, annotations with relationships
* **Knowledge graph representation**: Typed nodes and relationships with properties
* **Configuration files**: Complex AI pipelines with hierarchical settings
* **Intermediate representation**: Data exchange between AI system components
* **Version-controlled datasets**: Minimal diff noise for Git-friendly serialization
* **API payloads**: Efficient transport of structured data with references

### 3.4 Non-Goals

1. **Human editing as primary interface**: While readable, HEDL is optimized for machine generation/consumption
2. **Arbitrary graph query language**: Focus is on serialization, not querying
3. **Binary efficiency**: Text-based format prioritizes token efficiency over byte efficiency
4. **Runtime computation**: Expressions are opaque; no built-in evaluation engine
5. **General-purpose programming language**: HEDL is a data format, not a programming language

---

## 4. Lexical Structure

### 4.1 Character Encoding

* Files MUST be UTF-8 encoded without null bytes
* **Structural Tokens**: All structural tokens (Keys, TypeNames, Directives) MUST be ASCII-only. This ensures maximum interoperability and simplicity for tooling.
* **ID Tokens**: ID tokens MUST be ASCII-only for v1.0.0 to ensure consistent reference resolution across platforms. Future versions may support Unicode IDs.
* **Data Values**: String values, comments, and tensor literals MAY contain any valid UTF-8 sequence.
* A UTF-8 BOM (Byte Order Mark) SHOULD NOT be present
* If a BOM is present, parsers MUST:
  1. Recognize it as `EF BB BF`
  2. Skip it during parsing
  3. NOT include it in line/column counting
  4. Report a warning (optional but RECOMMENDED)

* **Normalization Form**: Unicode Normalization Form C (NFC) is RECOMMENDED for data values but not required
* **Invalid Sequences**: Any invalid UTF-8 byte sequence MUST cause a Syntax Error
* **Control Characters**: Any ASCII control character (0x00-0x1F, except 0x0A, 0x0D, 0x09) is a Syntax Error. Tab (0x09) is allowed only in quoted strings and expressions.
* **Maximum File Size**: Parsers SHOULD enforce a maximum file size (RECOMMENDED 1GB)

### 4.2 Line Endings

* Lines MUST be terminated by either:
  * LF (`\n`, U+000A) - Unix style
  * CRLF (`\r\n`, U+000D U+000A) - Windows style
* CR-only (`\r`, U+000D) line endings are NOT permitted and MUST cause a Syntax Error
* Parsers MUST normalize all line endings to LF (`\n`) before processing
* Line terminators inside quoted CSV fields are NOT permitted (Section 9.1)
* **Empty Files**: A zero-byte file is a Syntax Error
* **Trailing Newline**: A trailing newline at end of file is OPTIONAL but RECOMMENDED
* **Maximum Line Length**: Parsers SHOULD enforce a maximum line length (RECOMMENDED 1MB)

### 4.3 Indentation Rules

HEDL uses significant whitespace for structure:

1. **Indentation Characters**: Only ASCII space (`U+0020`) is allowed for indentation
2. **Tab Prohibition**: Tab characters (`U+0009`) are NOT allowed for indentation but MAY appear inside quoted strings and expressions
3. **Step Size**: Exactly 2 spaces per indent level
4. **Validation**: If `LeadingSpaces mod 2 ≠ 0`, it's a Syntax Error (unless line is blank)
5. **Maximum Depth**: Parsers SHOULD enforce a maximum indent depth (default 50)
6. **Zero Indent**: The first non-header, non-blank line MUST have indent level 0
7. **Whitespace Definition**: Throughout this specification, "whitespace" refers to ASCII space (`U+0020`) only, unless explicitly stated otherwise. Unicode whitespace characters (e.g., NBSP, zero-width spaces) are NOT treated as whitespace for parsing purposes and SHOULD cause warnings or errors if found in structural positions.

**Definition**: For a line with `LeadingSpaces` (count of leading spaces after normalization):
```
IndentLevel = LeadingSpaces // 2  (integer division)
```

**Indentation Examples**:
```hedl
level0:       # IndentLevel = 0
  level1:     # IndentLevel = 1 (2 spaces)
    level2:   # IndentLevel = 2 (4 spaces)
  level1_2:   # IndentLevel = 1 (back to 2 spaces)
```

**Syntax Error Examples**:
```hedl
level0:
   level1:    # ERROR: 3 spaces (odd number)
	level1:    # ERROR: tab character for indentation
```

### 4.4 Blank Lines

* Blank lines (containing only whitespace) are allowed anywhere
* Blank lines MUST be ignored during parsing
* Blank lines do NOT affect the context stack or scope
* Blank lines in matrix lists do NOT reset ditto state
* **Header Blank Lines**: Allowed between directives
* **Body Blank Lines**: Allowed between any elements at same indent level

### 4.5 Comments

Comments provide documentation without affecting parsed data:

1. **Comment Character**: `#` (U+0023)
2. **Full-line Comments**: Line where first non-space character is `#`
3. **Inline Comments**: May appear after any meaningful content
4. **Matrix Row Comments**: Allowed but MUST be handled specially (Section 9.1)
5. **Header Comments**: Allowed between and after directives

**Comment Stripping Rule**:

For all non-matrix-row line types, inline comments MUST be stripped by scanning the line left-to-right. The first `#` that occurs **outside any quoted string or expression region** (as identified by `scan_regions`) begins the comment, and the remainder of the line MUST be ignored.

**Normative Algorithm: `strip_comment(line)`**

```python
def strip_comment(line):
    """
    Strips inline comments from a line, respecting quoted strings and expressions.
    Returns the line with comment removed.
    """
    regions = scan_regions(line)
    region_idx = 0
    
    for i, char in enumerate(line):
        # Advance region pointer if we passed the current region
        while region_idx < len(regions) and regions[region_idx][1] <= i:
            region_idx += 1
            
        # Check if current char is inside the active region
        is_in_protected_region = False
        if region_idx < len(regions):
            start, end, _ = regions[region_idx]
            if start <= i < end:
                is_in_protected_region = True
        
        if char == '#' and not is_in_protected_region:
            return line[:i].rstrip() # Comment found outside protected region
            
    return line.rstrip() # No comment found
```

**Normative Algorithm: `scan_regions(line)`**

This algorithm scans a line and identifies regions of quoted strings and expressions. It returns a list of tuples `(start_index, end_index, type)` where `type` is either `"quote"` or `"expression"`. These regions indicate where special characters (like `#` or `,`) might lose their usual meaning.

```python
def scan_regions(line):
    """
    Scans a line for quoted string and expression regions.
    Returns a list of (start_index, end_index, type) tuples.
    """
    regions = []
    i = 0
    while i < len(line):
        if line[i] == '"':
            start_quote = i
            i += 1
            while i < len(line):
                if line[i] == '"':
                    if i + 1 < len(line) and line[i + 1] == '"': # Escaped quote
                        i += 2
                    else: # Closing quote
                        regions.append((start_quote, i + 1, "quote"))
                        i += 1
                        break
                else:
                    i += 1
            if i == len(line) and line[i-1] != '"': # Unclosed quote at end of line
                 regions.append((start_quote, len(line), "quote")) # Mark as region until end
        elif line[i:i+2] == '$(':
            start_expr = i
            i += 2
            depth = 1
            in_expr_quotes = False
            
            while i < len(line):
                char = line[i]
                
                if char == '"':
                    if in_expr_quotes:
                        if i + 1 < len(line) and line[i + 1] == '"': # Escaped quote
                            i += 2
                            continue
                        else:
                            in_expr_quotes = False
                    else:
                        in_expr_quotes = True
                
                if not in_expr_quotes:
                    if char == '(':
                        depth += 1
                    elif char == ')':
                        depth -= 1
                
                if depth == 0:
                    regions.append((start_expr, i + 1, "expression"))
                    i += 1
                    break
                elif char == '\n': # Expression cannot span multiple lines
                    # This should ideally be caught by lexical analysis before scan_regions
                    # For safety, if encountered, consider it ends here but is malformed
                    regions.append((start_expr, i, "expression"))
                    break
                i += 1
            if depth != 0: # Unclosed expression at end of line
                 regions.append((start_expr, len(line), "expression")) # Mark as region until end
        else:
            i += 1
    return regions
```

**Examples**:
```hedl
# This is a full-line comment
key: value  # This is an inline comment
  child: value  # Indented inline comment
|value1,value2  # Comment on matrix row (stripped before CSV parse)
|"value # with hash",other  # Hash inside quotes is data
```

### 4.6 Token Definitions

#### 4.6.1 Key Token

**Pattern**: `[a-z_][a-z0-9_]*`
- **Used for**: object keys, column names, alias names (without `%` prefix)
- **Case**: ASCII lowercase only (case-sensitive)
- **First character**: ASCII lowercase letter (`a-z`) or underscore (`_`)
- **Examples**: `name`, `user_id`, `_private`
- **Invalid**: `myKey` (uppercase), `my-key` (hyphen), `1_item` (starts with digit), `über` (non-ASCII)

#### 4.6.2 TypeName Token

**Pattern**: `[A-Z][A-Za-z0-9]*`
- **Used for**: struct names in `%STRUCT`, type references in `@TypeName`
- **Case**: PascalCase (must start with ASCII uppercase)
- **Examples**: `User`, `Post`, `Item123`
- **Invalid**: `user` (lowercase), `123Item` (starts with digit), `my_type` (underscore)

#### 4.6.3 ID Token

**Pattern**: `[a-z_][a-z0-9_\-]*`
- **Used for**: node IDs in first column of matrix rows
- **First character**: ASCII lowercase letter or underscore
- **Allowed**: ASCII lowercase letters, digits, underscore, hyphen
- **Examples**: `user_1`, `item-two`, `_system`
- **Invalid**: `User1` (starts with uppercase), `123` (starts with digit), `two words` (space), `ITEM` (uppercase letters)
- **Note**: ASCII-only for v1.0.0 to ensure consistent reference resolution. Future versions may support Unicode.

#### 4.6.4 Reference Token

**Pattern**: `@([A-Z][A-Za-z0-9]*:)?[a-z_][a-z0-9_\-]*`
- **Used for**: referencing existing nodes by ID
- **Format**:
  - **Local Reference**: `@id` (searches current type namespace)
  - **Qualified Reference**: `@Type:id` (searches specified type namespace)
- **Examples**: `@user_1`, `@User:user_1`, `@Post:p-123`
- **Invalid**: `@User1` (uppercase ID), `@123` (starts with digit), `User:id` (missing @)
- **Note**: Qualified references are REQUIRED when referencing a node of a different type.

#### 4.6.5 Alias Key Token

**Pattern**: `%[a-z_][a-z0-9_]*`
- **Used for**: referencing aliases defined by `%ALIAS`
- **Format**: `%` followed by Key Token
- **Examples**: `%active`, `%default_value`, `%pi`
- **Invalid**: `%Active` (uppercase), `%my-alias` (hyphen), `%` (empty), `%123` (starts with digit)

#### 4.6.6 Expression Token (Normative)

* **Starts with**: `$(`
* **Ends with**: the `)` that closes the initial `$(`, using balanced-parentheses scanning
* **Content**: any characters except physical newlines
* **Algorithm**: After reading `$(`, set `depth = 1`. For each subsequent character:
  - If `"`: toggle quoted state (handle `""` escape). Parentheses inside quotes are ignored.
  - If `(` and not quoted: `depth += 1`
  - If `)` and not quoted: `depth -= 1`; if `depth == 0`, the expression ends here
  - newline before `depth == 0` → **SyntaxError** (no multi-line expressions)
* If EOF is reached with `depth != 0` → **SyntaxError** (unclosed expression)
* **Backslash**: remains literal; no escaping rules
* **Examples**: `$(x + 1)`, `$((a + b))` → `Expression("(a + b)")`, `$(concat("hello", "world"))`

#### 4.6.7 Ditto Token

**Pattern**: `^` (single caret)
- **Used for**: copying value from same column of previous row
- **Context**: Only valid in matrix cells, not in Key-Value pairs
- **Invalid contexts**: ID column, first row of list, Key-Value values

#### 4.6.8 Tensor Literal

**Pattern**: Starts with `[`, contains balanced brackets with numeric values
- **Used for**: multi-dimensional numerical arrays
- **Format**: `[1, 2, 3]` or `[[1, 2], [3, 4]]`
- **Rules**: Must contain only numbers, commas, spaces, and balanced brackets
- **Examples**: `[1, 2, 3]`, `[[1.5, 2.0], [3.1, 4.2]]`
- **Invalid**: `[1, "text"]` (mixed types), `[1, 2` (unbalanced)

### 4.7 Colon Spacing (Body Section Only)

In the Body Section, a colon `:` in a statement line MUST be followed by exactly one of:

1. **End-of-line / whitespace / comment only** (Object Start), or
2. **At least one space** (Key-Value or Matrix List Start).

Any non-whitespace character immediately following `:` in the Body (e.g., `key:value`) is a **SyntaxError**.

**Examples**:
```hedl
# Valid
key:          # Object Start
key: value    # Key-Value
key: @Type    # Matrix List Start

# Invalid
key:value     # SyntaxError - no space after colon
key:  value   # Valid - multiple spaces allowed
```

### 4.8 Character Restrictions in Unquoted Strings

Unquoted strings have context-dependent restrictions based on whether they appear in Key-Value pairs or matrix cells.

#### 4.8.1 Common Restrictions (Both Contexts)

1. **May contain anywhere**: Any UTF-8 character except:
   - `"` (quotes must be quoted)
   - `#` (comment character)
   - Control characters (0x00-0x1F, 0x7F)

2. **Special note**: The characters `@` and `$` are allowed **inside** unquoted strings (e.g., `alice@example.com`, `total$`). They only trigger special parsing when they appear at the **beginning** of a token.

#### 4.8.2 Key-Value Context

In Key-Value pairs, unquoted values:
1. **Must not start with**: `~`, `@`, `$`, `%`, `[` (would trigger special parsing)
2. **Ditto (`^`) is NOT special**: In Key-Value context, `^` is parsed as the literal string `"^"` (not ditto operator)
3. **Colon (`:`)**: Allowed in values but not in keys

#### 4.8.3 Matrix Cell Context

In matrix cells (CSV fields), unquoted values:
1. **Must not start with**: `~`, `@`, `$`, `%`, `^`, `[` (would trigger special parsing)
2. **Ditto (`^`) IS special**: In matrix cells, `^` triggers ditto behavior (copy from previous row)
3. **Additional forbidden characters**: `,` and `|` (must be quoted if needed in data)

**Examples**:
```hedl
# Key-Value context
email: alice@example.com    # Valid - @ inside string
cost: 100$                  # Valid - $ inside string
ref: @user1                 # Valid - reference (starts with @)
expr: $(x + 1)              # Valid - expression (starts with $()
alias: %active              # Valid - alias (starts with %)
ditto: ^                    # Valid - string "^" (ditto NOT special in Key-Value)

# Matrix cell context
|alice@example.com,value  # Valid - @ inside string
|@user1,value             # Reference (starts with @)
|^,value                  # DITTO operator (copies from previous row)
|"^",value                # String "^" (quoted to prevent ditto)
```

---

## 5. Document Structure

### 5.1 Three-Part Organization

Every HEDL document consists of three mandatory parts in order:

```
[Header Section]
[Separator Line]
[Body Section]
```

### 5.2 Header Section

- **Purpose**: Configure parsing state through directives
- **Location**: From start of file to separator line
- **Directives**: Start with `%`, case-sensitive uppercase
- **Order**: Directives MUST appear in dependency order (Section 6.1)
- **Optional**: Header may contain only `%VERSION: 1.0` and separator for simple documents

### 5.3 Separator Line

* **Format**: The line MUST start with exactly three hyphens: `---`
* **Delimiter**: The three hyphens MUST be followed immediately by a newline, space, or `#`. Any other character (e.g., a fourth hyphen) is a Syntax Error.
* **Trailing Content**: Any characters after the delimiter are ignored (treated as whitespace or comment)
* **Leading Spaces**: MUST NOT have leading spaces (Syntax Error)
* **Canonical Form**: `---\n` (no trailing spaces or comments)
* **Requirement**: MUST appear exactly once, after all header directives
* **Error**: Missing separator is a Syntax Error
* **Truncation Detection**: File ending with `--` or `-` is a Syntax Error (truncated separator)

**Examples**:
```hedl
%VERSION: 1.0
---  # Valid with trailing spaces
```

```hedl
%VERSION: 1.0
  ---  # SyntaxError - leading spaces
```

### 5.4 Body Section

- **Purpose**: Contains the actual data
- **Location**: From after separator to end of file
- **Structure**: Hierarchical objects and typed lists
- **Root**: Implicit root object at indent level 0
- **Simple Mode**: Can contain only key-value pairs and nested objects (no schemas required)
- **Complex Mode**: Can include matrix lists with schemas

### 5.5 Empty Documents

* An empty document (zero bytes) is invalid
* A document containing only whitespace and/or comments is invalid
* Minimum valid document: `%VERSION: 1.0\n---\n`
* A document with only header and separator but no body is valid (empty root object)

**Examples**:
```hedl
# Invalid - no version
---
```

```hedl
# Invalid - no separator
%VERSION: 1.0
```

```hedl
# Valid minimal document
%VERSION: 1.0
---
```

---

## 6. Header Section

The Header configures parsing state through directives. All directives start with `%` and use `:` to separate name from payload.

### 6.1 Directive Format

```
%DIRECTIVE: payload
```

* `DIRECTIVE` is case-sensitive ASCII uppercase
* `:` MUST be followed by at least one space
* `payload` format depends on the directive
* **Order**: Directives MUST appear in dependency order:
  - `%VERSION` MUST be first (REQUIRED)
  - `%STRUCT` definitions MUST appear before they are referenced by `%NEST`
* **Comment handling**: Inline comments allowed after payload, stripped before parsing payload
* **Spacing**: Implementations MUST accept one or more spaces after `:`

### 6.2 `%VERSION` Directive (REQUIRED)

Declares the HEDL specification version.

**Syntax**: `%VERSION: major.minor`

**Parameters**:
- `major`: Non-negative integer
- `minor`: Non-negative integer
- Both separated by exactly one `.`
- No leading zeros (except `0` itself)

**Examples**:
```hedl
%VERSION: 1.0
%VERSION: 2.5
```

**Invalid Examples**:
```hedl
%VERSION: 1      # Missing minor
%VERSION: 1.0.0  # Too many parts
%VERSION: 01.0   # Leading zero
%VERSION: a.b    # Non-numeric
```

**Parser Behavior**:
1. Parse major.minor as integers
2. If parsing fails → `VersionError`
3. If file `major > parser.major`: raise `VersionError` (incompatible)
4. If file `major < parser.major`: MAY accept (backward compatibility)
5. If `major` matches but `minor > parser.minor`: MAY accept if new features can be safely ignored
6. Otherwise: proceed normally

**Note**: This specification is version `1.0`.

### 6.3 `%STRUCT` Directive (Optional)

Defines a named schema for typed matrix lists.

**Syntax**: `%STRUCT: TypeName: [col1, col2, ...]`

**Requirements**:
- `TypeName` MUST be a TypeName Token
- Column names MUST be Key Tokens and unique within the struct
- At least one column REQUIRED
- Maximum columns: implementation-defined (RECOMMENDED ≥ 100)
- First column is the ID column (Section 10.1)
- Column order defines CSV parsing order

**Examples**:
```hedl
%STRUCT: User: [id,name,email]
%STRUCT: Post: [id,author_id,content,timestamp]
%STRUCT: Item: [id,name,price,quantity,category]
```

**Redefinition Rules**:
1. Same `TypeName` with identical columns: allowed (idempotent)
2. Same `TypeName` with different columns: `SchemaError`
3. Column order is significant for matrix row parsing

**Semantic Constraints**:
- Column names should be descriptive but concise
- Avoid reserved words (not enforced but recommended)
- ID column should be named `id` (convention, not requirement)

#### 6.3.1 Column List Parsing (Normative)

A column list has the form `[col1, col2, ...]`.

Parsing algorithm:
1. Strip inline comment if present
2. Trim whitespace
3. MUST start with `[` and end with `]`
4. Remove `[` and `]` delimiters
5. Split remaining string by comma `,`
6. For each part:
   - Trim whitespace
   - Validate as Key Token
   - Check for duplicates
7. Validate at least one column

**Examples**:
```hedl
%STRUCT: User: [id, name, email]          # Valid
%STRUCT: User: [ id , name , email ]      # Valid - spaces allowed
%STRUCT: User: [id,name,email]            # Valid - no spaces
%STRUCT: User: [id, name, email,]         # SyntaxError - trailing comma
%STRUCT: User: []                         # SyntaxError - empty
%STRUCT: User: [id, id]                   # SchemaError - duplicate
```

### 6.4 `%NEST` Directive (Optional)

Declares implicit parent-child relationships for automatic list nesting.

**Syntax**: `%NEST: ParentType > ChildType`

**Requirements**:
- `ParentType` MUST be defined via `%STRUCT`
- `ChildType` MUST be defined via `%STRUCT` (for v1.0.0)
- Each `ParentType` can have AT MOST one `%NEST` rule
- No circular nesting chains (not validated but must be acyclic)

**Semantics**:
When parsing a list of `ParentType`, rows indented one level deeper are interpreted as belonging to a child list of `ChildType`, attached to the most recent parent row.

**Error Conditions**:
- Multiple `%NEST` directives with same `ParentType`: `SchemaError`
- `ParentType` not in Schema Registry: `SchemaError`
- `ChildType` not in Schema Registry: `SchemaError`

**Example**:
```hedl
%STRUCT: User: [id,name]
%STRUCT: Post: [id,content]
%NEST: User > Post
```

**Nesting Chains**: Multiple levels allowed:
```hedl
%STRUCT: Project: [id,name]
%STRUCT: Task: [id,description]
%STRUCT: SubTask: [id,details]
%NEST: Project > Task
%NEST: Task > SubTask
```

**Multiple Children**: Not supported in v1.0.0 (one parent type, one child type). For complex hierarchies with multiple child types, use flattened lists with explicit parent references (foreign keys).

### 6.5 `%ALIAS` Directive (Optional)

Defines global constants for token substitution.

**Syntax**: `%ALIAS: %key: "expansion value"`

**Requirements**:
- Key MUST be an Alias Key Token (`%` + Key Token)
- Value MUST be a quoted string (double quotes)
- Keys MUST be unique (`AliasError` if duplicate)
- Value may be empty string
- No recursive expansion (aliases cannot reference other aliases)

**Quoted String Rules**:
- Standard HEDL quoted string parsing (Section 8.1.1)
- Escaped quotes: `""` → `"`
- Backslash literal: `\` → `\`
- No multi-line strings

**Expansion Semantics**:
1. Alias values are string literals from the quoted payload
2. During parsing, alias references (`%key`) are replaced by the literal string value
3. The replaced value then enters the normal inference ladder (Sections 8.2, 9.3)
   - **Important**: The expansion replaces the *unquoted* alias token. The result is treated as raw text for inference.
   - Example: `%ALIAS: %true: "true"`. Usage `| %true`. Expands to `true`. Inferred as **Boolean**.
   - Example: `%ALIAS: %val: "123"`. Usage `| %val`. Expands to `123`. Inferred as **Integer**.
   - It is NOT possible to alias a Quoted String structure. Usage `| "%val"` treats `%val` as a literal string.
4. NO recursive expansion (aliases cannot reference other aliases)
5. Aliases are only expanded for unquoted tokens
6. Quoted alias references (e.g., `"%active"`) are not expanded

**Examples**:
```hedl
%ALIAS: %active: "true"          # Expands to "true", then inferred as boolean true
%ALIAS: %inactive: "false"       # Expands to "false", then boolean false
%ALIAS: %empty: ""               # Expands to empty string
%ALIAS: %pi: "3.14159"           # Expands to "3.14159", then inferred as float
%ALIAS: %name: "John ""Doc"" Doe"  # Expands to John "Doc" Doe
```

**Invalid Examples**:
```hedl
%ALIAS: active: "true"           # Missing % on key
%ALIAS: %active: true            # Value not quoted
%ALIAS: %active: "true"          # OK
%ALIAS: %active: "false"         # AliasError - duplicate key
```

### 6.6 Minimal Header

For simple documents without schemas, only the version directive is required:

```hedl
%VERSION: 1.0
---
# Simple key-value pairs follow
```

---

## 7. Body Section

### 7.1 Body Line Classification

Each non-blank, non-comment line in the Body MUST be classified as exactly one of:

| Type | Pattern | Description | Valid Context |
|------|---------|-------------|---------------|
| Object Start | `key:` | Begins nested object mapping | Root, Object |
| Key-Value | `key: value` | Assigns scalar to current object | Root, Object |
| Matrix List Start | `key: @TypeName[...]` | Begins typed list with schema | Root, Object |
| Matrix Row | `\| cell1, cell2, ...` | Data row in active matrix list | List |

### 7.2 Classification Algorithm

After stripping indentation, let `line` be the string with leading/trailing whitespace removed.

**For Matrix Row lines** (starting with `|`):
- Handle comments specially (Section 9.1), then parse as Matrix Row

**For all other line types**:
1. **Strip inline comment**: Remove the first `#` and everything after it (unless inside quotes)
2. **Trim**: Remove any remaining leading/trailing whitespace
3. Now classify the stripped line using these rules (applied in order):

**Classification Rules (after comment stripping and trim)**:

1. If line starts with `|`: Matrix Row (already handled)
2. Else if line matches `^[a-z_][a-z0-9_]*:\s+@[A-Z][A-Za-z0-9]*(\[[^\]]*\])?\s*$`: **Matrix List Start**
   - Example: `users: @User` or `items: @Item[id, name]`
   - Pattern breakdown:
     - `^[a-z_][a-z0-9_]*:`: key with colon
     - `\s+`: at least one space (required by Section 4.7)
     - `@[A-Z][A-Za-z0-9]*`: TypeName with @ prefix
     - `(\[[^\]]*\])?`: optional inline schema
     - `\s*$`: optional trailing whitespace
3. Else if line matches `^[a-z_][a-z0-9_]*:\s*$`: **Object Start**
   - Example: `config:` or `database:` (after comment stripping)
   - Note: No non-whitespace after colon
4. Else if line matches `^[a-z_][a-z0-9_]*:\s+\S.*$`: **Key-Value Pair**
   - Example: `host: localhost` or `port: 8080`
   - Requires: at least one space after colon, then non-whitespace
5. Else if line contains `:`: **Syntax Error** (violates Section 4.7)
   - Could be `key:value` (no space) or other malformed line
6. Otherwise: **Syntax Error** (no colon in non-matrix line)

**Note**: The Matrix List Start pattern ensures references (`@lowercase`) are NOT matched as lists.

### 7.3 Contextual Validity

Different line types are valid in different contexts:

| Current Context | Allowed Line Types |
|----------------|-------------------|
| Root | Object Start, Key-Value, Matrix List Start |
| Object | Object Start, Key-Value, Matrix List Start |
| List | Matrix Row only (peer or child rows) |

### 7.4 Object Start

**Format**: `key:` (colon required, no non-whitespace value before comment)

**Behavior**:
1. Creates new empty object mapping
2. Assigns it to `key` in current object
3. **Duplicate Key Check**: If `key` already exists in current object → `SemanticError`
4. Pushes Object frame onto context stack
5. New object becomes current scope
**Indentation**: MUST be exactly one level deeper than parent (Indent 0 for top-level objects)

**Examples**:
```hedl
config:  # Object Start
  database:  # Nested Object Start
    host: localhost  # Key-Value
  logging:  # Back to config level
    level: info
```

**Error Cases**:
```hedl
parent:
  child:        # OK (IndentLevel = 1)
    grand: 1    # OK (IndentLevel = 2)
 wrong: 1       # SyntaxError - 1 leading space (odd indentation)
```

### 7.5 Key-Value Pair

**Format**: `key: value` (non-whitespace after colon, with at least one space after `:`)

**Value Parsing** (detailed in Section 8):
1. Strip inline comment if present
2. Apply inference ladder to remaining value
3. The ditto operator (`^`) has no special meaning in Key-Value context; an unquoted `^` token is parsed as the string `"^"`

**Indentation**: MUST be exactly one level deeper than parent (Indent 0 for top-level pairs)

**Examples**:
```hedl
name: "Alice"      # String
age: 30            # Integer
active: true       # Boolean
ref: @user1        # Reference
expr: $(x + 1)     # Expression
alias: %active     # Alias expansion
ditto: ^           # String "^" (not ditto operator)
```

**Error Cases**:
```hedl
parent:
  key:value  # SyntaxError - no space after colon
  key:       # Object Start, not Key-Value
```

### 7.6 Matrix List Start

**Format**: `key: @TypeName` or `key: @TypeName[col1, col2, ...]`

**Schema Resolution**:
1. **Declared Schema**: `@TypeName` alone → MUST exist in Schema Registry
2. **Inline Schema**: `@TypeName[...]` → defines schema for this instance
   - If TypeName in Registry: columns MUST match exactly
   - If not in Registry: defines it locally (only for this list)

**Inline schema column lists MUST be parsed and validated per Section 6.3.1.**

**Behavior**:
1. Check if `key` exists in `currentObject`; if so, `SemanticError`. Else create empty list at `currentObject[key]`.
2. Pushes List frame with schema, tracking state:
   - `typeName`: TypeName
   - `schema`: column array
   - `lastRowValues`: null initially
   - `lastItem`: null initially
   - `rowIndent`: current indent + 1
3. Subsequent matrix rows fill this list
4. A matrix list MAY contain zero rows; an empty list is produced if the list frame is closed before any `|` rows appear

**Examples**:
```hedl
# Using declared schema
%STRUCT: User: [id,name]
---
users: @User
  |u1,Alice
  |u2,Bob

# Using inline schema (no %STRUCT needed)
items: @Item[id,name,price]
  |i1,Apple,1.99
  |i2,Banana,0.99

# Error - schema mismatch
%STRUCT: User: [id,name,email]
---
users: @User[id,name]  # SchemaError - column mismatch
```

---

## 8. Scalars in Key-Value Pairs

### 8.1 Parsing Algorithm

For a Key-Value line `key: value`:

1. **Strip Comment**: Remove inline comment if present (first `#` outside quotes)
2. **Extract Value**: Take substring from after `:` to end (or comment start)
3. **Trim**: Remove leading/trailing whitespace
4. **Inference**: Apply ladder below (Section 8.2)

#### 8.1.1 Quoted String Parsing in Key-Value

For quoted strings in Key-Value values:

1. **Quoting Character**: Double quote `"`
2. **Escaping**: Inside quoted strings, `""` represents a literal `"`
3. **No Other Escapes**: Backslash (`\`) has no special meaning and is treated literally
4. **Newlines**: A quoted string MUST NOT span multiple physical lines. For example, a value that starts with `msg: "Line1` on one line and continues with `Line2"` on the next is a **Syntax Error**.
5. **Comment Stripping**: When stripping comments from Key-Value lines, `#` characters inside quoted strings are NOT comment delimiters.
6. **Empty Strings**: `""` → empty string
7. **Whitespace Preservation**: Leading/trailing whitespace inside quotes preserved
8. **Tab Characters**: Tab characters ARE allowed inside quoted strings (unlike in indentation)

**Quoted String State Machine**:
```
State OUTSIDE:
  On ": enter INSIDE, start collecting
  On #: comment start (if not inside quotes)

State INSIDE:
  On ": 
    If next char is ": treat as escaped quote, add ", skip next
    Else: end string
  On other: add to string

State UNQUOTED (implicitly applies to Key-Value scalar parsing without explicit quotes):
  On '"': SyntaxError (quote in unquoted field)
  On end: emit field (trimmed), done
  On other: add to field
```

**Examples**:
- `msg: "Hello ""world""!"` → String: `Hello "world"!`
- `msg: "Hello \"world\"!"` → String: `Hello \"world\"!` (backslash literal)
- `msg: "Text # not comment"` → String: `Text # not comment`
- `msg: "  spaces  "` → String: `  spaces  ` (preserved)
- `msg: ""` → String: `` (empty)

#### 8.1.2 Block Strings

Block strings use triple double-quotes (`"""`) to represent multiline string content in key-value pairs. They provide a clean way to include literal newlines without escape sequences.

**Syntax**:
```hedl
key: """
Line 1
Line 2
Line 3
"""
```

**Rules**:
1. **Opening**: `"""` MUST be followed immediately by a newline (no content on the opening line)
2. **Closing**: `"""` MUST appear on its own line with only indentation before it
3. **Content**: All lines between opening and closing are included literally
4. **No Escape Processing**: Backslash has no special meaning; `\n` is literal backslash-n
5. **Quote Escaping**: Not needed; `"` and `""` are literal within block strings
6. **Indentation**: The closing `"""` indentation level is stripped from all content lines
7. **Leading/Trailing Newlines**: The newline after `"""` and before `"""` are NOT included in the value

**State Machine**:
```
State BLOCK_STRING_START:
  On '"""' followed by newline: enter BLOCK_STRING_CONTENT, record indent level
  Otherwise: not a block string, parse as normal value

State BLOCK_STRING_CONTENT:
  On line starting with indent + '"""': end block string, emit collected content
  Otherwise: add line content (minus base indent) plus newline to content
```

**Indentation Stripping Algorithm**:
```python
def parse_block_string(lines, base_indent):
    """
    Parse a block string starting after the opening '"""'.
    base_indent is the indentation of the key line.
    """
    content_lines = []
    for line in lines:
        stripped = line.lstrip()
        if stripped == '"""':
            # Closing found - join all content lines with newlines
            return '\n'.join(content_lines)
        # Strip base indentation from content line
        if line.startswith(' ' * base_indent):
            content_lines.append(line[base_indent:])
        else:
            content_lines.append(line.lstrip())
    raise SyntaxError("Unclosed block string")
```

**Examples**:
```hedl
# Simple multiline
description: """
This is line 1.
This is line 2.
"""
# Value: "This is line 1.\nThis is line 2."

# With quotes (no escaping needed)
code: """
print("Hello")
"""
# Value: "print(\"Hello\")"

# With indentation preserved
script: """
  if condition:
    do_something()
"""
# Value: "  if condition:\n    do_something()"
```

**Canonicalization**: Canonical output SHOULD use block strings for string values containing newlines. See Section 13.3.

### 8.2 Inference Ladder

Apply in strict order:

1. **Null**: `~` → `null`
   - Exact match, case-sensitive
   - No whitespace allowed around `~`

2. **Tensor Literal**: Starts with `[` → attempt to parse as tensor
   - Must validate bracket balance
   - Must contain only numbers (integers or floats), commas, spaces, and balanced brackets
   - May be multi-dimensional: `[[1, 2], [3, 4]]`
   - Implementation representation: array or nested arrays
   - **Failure Mode**: If a value starts with `[` but fails to parse as a valid tensor (unbalanced brackets, non-numeric content, etc.), it is a `SyntaxError` - NOT a fallthrough to String

3. **Reference**: Token starting with `@` → validate as Reference token:
   - MUST match pattern `@([A-Z][A-Za-z0-9]*:)?[a-z_][a-z0-9_\-]*`
   - If pattern not matched: `SyntaxError`
   - Otherwise: `Reference(ID)` where ID is the full reference (e.g., `user_1` or `User:user_1`)
   - Resolution happens later (Section 10.3)

4. **Expression**: Starts with `$(` and forms a valid Expression Token per Section 4.6.6 → `Expression(text)` (opaque)
   - `text` is everything between `$(` and the closing `)` (excluding delimiters)
   - No validation of expression content

5. **Alias**: Exact match of alias key → expand to defined string value
   - Apply inference to the expanded string:
     - If matches **Boolean** (true/false) → Boolean
     - If matches **Number** → Integer or Float
     - Otherwise → String (Note: The SyntaxError rule regarding quotes in unquoted strings does NOT apply here; the expanded value is accepted as-is)

6. **Boolean**: `true` or `false` (case-sensitive) → boolean
   - Exact match, lowercase
   - No type coercion (e.g., `"true"` → string, not boolean)

7. **Number**: Matches `^-?[0-9]+(\.[0-9]+)?$` → integer or float
   - **Integer**: No decimal point: `42`, `-1` → integer
   - **Float**: Contains decimal point: `42.0`, `3.14` → float
   - No scientific notation (`1e10` is string)
   - Leading zeros allowed (`001` → integer 1)
   - No underscores in numbers (`1_000` is string)

8. **String**: Anything else → string
   - Unquoted strings are trimmed
   - May contain any characters except those prohibited in Section 4.8
   - Empty unquoted string not possible (would be Object Start)

### 8.3 Special Cases

* **Ditto**: `^` in Key-Value context is literal string `"^"` (doesn't trigger ditto behavior)
* **Empty Value**: `key:` (no value) is Object Start, NOT Key-Value
* **Whitespace Preservation**: Only in quoted strings; unquoted values are trimmed
* **Quoted Strings**: Always parsed as strings, no inference
* **Mixed Quoting**: Not allowed; a value like `"hello` without closing quote is Syntax Error

### 8.4 Examples

```hedl
# Key-Value examples
null_val: ~                     # null
tensor_val: [[1, 2], [3, 4]]   # tensor/array
ref_val: @node1                 # Reference("node1")
expr_val: $(x + 1)              # Expression("x + 1")
alias_val: %active              # expands to "true", then becomes boolean true
bool_true: true                 # boolean true
bool_false: false               # boolean false
int_val: 42                     # integer 42
int_zero: 0                     # integer 0
float_val: 3.14                 # float 3.14
float_explicit: 42.0            # float 42.0
string_val: hello               # string "hello"
string_num: "42"                # string "42", not integer
string_bool: "true"             # string "true", not boolean
ditto_val: ^                    # string "^" (not ditto operator)
quoted_val: "  spaces  "        # string "  spaces  "
empty_quoted: ""                # string ""
mixed_quotes: "he said ""hi"""  # string 'he said "hi"'
```

---

## 9. Matrix Row and Cell Parsing

### 9.1 Matrix Row Preprocessing

A matrix row line begins with `|`. The parser MUST extract the CSV content using the following algorithm:

**Normative Algorithm: `extract_csv_content(line)`**

```python
def extract_csv_content(line):
    """
    Extracts the CSV content from a matrix row line.
    Precondition: line is known to contain '|'.
    """
    # 1. Find delimiter
    pipe_idx = line.find('|')
    if pipe_idx == -1:
         raise SyntaxError("Matrix row missing '|'")
         
    # 2. Extract raw content after the pipe
    raw_content = line[pipe_idx+1:]
    
    # 3. Strip comments using the standard strip_comment function (Section 4.5)
    # This handles comments respecting quotes and expressions
    comment_stripped = strip_comment(raw_content)
    
    # 4. Trim leading/trailing whitespace
    csv_content = comment_stripped.strip()
    
    return csv_content
```

**Important**: This order implies:
- A `#` inside a quoted CSV field is NOT a comment (handled by `strip_comment`)
- A `#` outside quotes ends the string
- Whitespace around the CSV content is ignored

### 9.2 CSV Record Parsing (Normative State Machine)

Parse the CSV substring using a state machine with these rules:

1. **Delimiter**: Comma `,`
2. **Quoting**: Double quotes `"` only
3. **Escaping**: Inside quoted fields:
   - `""` → literal `"` (double-quote escape)
   - `\n` → newline (U+000A)
   - `\t` → tab (U+0009)
   - `\r` → carriage return (U+000D)
   - `\\` → literal backslash
   - `\"` → literal `"` (alternative to `""`)
4. **Whitespace**:
   - Unquoted fields trimmed of leading/trailing whitespace
   - Quoted fields preserve internal whitespace
5. **Newlines**: NOT allowed as literal characters inside fields (use `\n` escape in quoted fields)
6. **Empty Fields**: `,,` → empty string between commas
7. **Trailing Comma**: `| a, b,` is a **SyntaxError** (trailing comma not allowed; use `""` or `~` explicitly)
8. **Leading/Trailing Spaces**: Around commas ignored in unquoted fields

**Note**: Escape sequences are ONLY processed in quoted matrix cell fields. In key-value pairs, backslash is always literal. Use block strings (Section 8.1.2) for multiline key-value content.

**Normative Algorithm: `parse_csv_row(csv_string)`**

```python
def parse_csv_row(csv_string):
    """
    Parse a CSV string into a list of (value, is_quoted) tuples.
    Uses a state machine that respects quoted strings and expressions.
    """
    if not csv_string:
        return []

    # Check for trailing comma
    if csv_string.rstrip().endswith(','):
        raise SyntaxError("Trailing comma not allowed in matrix row")

    fields = []  # List of (value, is_quoted) tuples
    current_field = []
    current_is_quoted = False
    state = "START_FIELD"
    i = 0
    expression_depth = 0
    in_expr_quotes = False  # Track quotes inside expressions (must match scan_regions)

    while i < len(csv_string):
        char = csv_string[i]

        if state == "START_FIELD":
            current_is_quoted = False
            if char.isspace():
                i += 1
                continue
            elif char == '"':
                current_is_quoted = True
                state = "IN_QUOTED_FIELD"
                i += 1
            elif char == '$' and i + 1 < len(csv_string) and csv_string[i + 1] == '(':
                # Start of expression
                current_field.append('$(')
                state = "IN_EXPRESSION"
                expression_depth = 1
                i += 2
            else:
                state = "IN_UNQUOTED_FIELD"
                current_field.append(char)
                i += 1

        elif state == "IN_UNQUOTED_FIELD":
            if char == ',':
                # End of field
                field = ''.join(current_field).strip()
                if '"' in field:
                    raise SyntaxError(f"Quote character '\"' found in unquoted CSV field: '{field}'")
                fields.append((field, False))
                current_field = []
                state = "START_FIELD"
                i += 1
            else:
                current_field.append(char)
                i += 1

        elif state == "IN_QUOTED_FIELD":
            if char == '"':
                if i + 1 < len(csv_string) and csv_string[i + 1] == '"':
                    # Escaped quote via "" - add single quote to field
                    current_field.append('"')
                    i += 2
                else:
                    # End of quoted field
                    state = "AFTER_QUOTE"
                    i += 1
            elif char == '\\' and i + 1 < len(csv_string):
                # Escape sequence handling
                next_char = csv_string[i + 1]
                if next_char == 'n':
                    current_field.append('\n')
                    i += 2
                elif next_char == 't':
                    current_field.append('\t')
                    i += 2
                elif next_char == 'r':
                    current_field.append('\r')
                    i += 2
                elif next_char == '\\':
                    current_field.append('\\')
                    i += 2
                elif next_char == '"':
                    current_field.append('"')
                    i += 2
                else:
                    # Unknown escape - treat backslash literally
                    current_field.append(char)
                    i += 1
            else:
                current_field.append(char)
                i += 1

        elif state == "AFTER_QUOTE":
            if char.isspace():
                i += 1
                continue
            elif char == ',':
                fields.append((''.join(current_field), True))
                current_field = []
                state = "START_FIELD"
                i += 1
            else:
                raise SyntaxError(f"Expected comma after closing quote, got '{char}'")

        elif state == "IN_EXPRESSION":
            current_field.append(char)
            # Handle quotes inside expressions (must match scan_regions behavior)
            if char == '"':
                if in_expr_quotes:
                    if i + 1 < len(csv_string) and csv_string[i + 1] == '"':
                        # Escaped quote inside expression
                        current_field.append(csv_string[i + 1])
                        i += 2
                        continue
                    else:
                        in_expr_quotes = False
                else:
                    in_expr_quotes = True
            elif not in_expr_quotes:
                if char == '(':
                    expression_depth += 1
                elif char == ')':
                    expression_depth -= 1
                    if expression_depth == 0:
                        # End of expression
                        state = "IN_UNQUOTED_FIELD"
                        in_expr_quotes = False  # Reset for safety
            i += 1

    # Handle end of string
    if state == "IN_QUOTED_FIELD":
        raise SyntaxError("Unclosed quoted string in CSV field")
    elif state == "IN_EXPRESSION":
        raise SyntaxError("Unclosed expression in CSV field")
    elif state == "AFTER_QUOTE":
        fields.append((''.join(current_field), True))
    elif current_field:
        field = ''.join(current_field).strip()
        if '"' in field:
            raise SyntaxError(f"Quote character '\"' found in unquoted CSV field: '{field}'")
        fields.append((field, False))

    return fields
```

**Returns**: A list of `(value, is_quoted)` tuples, where `value` is the string content and `is_quoted` is a boolean indicating if the field was enclosed in quotes.

**Examples**:
```hedl
|simple,values           # ["simple", "values"]
|"quoted, field",other   # ["quoted, field", "other"]
|empty,,fields            # ["empty", "", "fields"]
|"escaped ""quote"""      # ['escaped "quote"']
|spaced,values            # ["spaced", "values"] (trimmed)
|"  spaced  ",values      # ["  spaced  ", "values"] (preserved)
```

### 9.3 Cell Value Inference

For each CSV field (after unquoting if quoted):

**If field is quoted**: Always string, no inference

**If field is unquoted**: Apply ladder:

1. **Null**: `~` → `null`
   - Exception: Not allowed in ID column (first column)

2. **Ditto**: `^` → copy from same column, previous row (scoped, Section 9.4)
   - Exception: Not allowed in ID column
   - Exception: Not allowed in first row of list

3. **Tensor Literal**: Starts with `[` → attempt to parse as tensor
   - Must validate bracket balance
   - Must contain only numbers, commas, spaces, and balanced brackets
   - May be multi-dimensional
   - **Failure Mode**: If starts with `[` but fails to parse as valid tensor → `SyntaxError`

4. **Reference**: Token starting with `@` → validate as Reference token:
   - MUST match pattern `@([A-Z][A-Za-z0-9]*:)?[a-z_][a-z0-9_\-]*`
   - If pattern not matched: `SyntaxError`
   - Otherwise: `Reference(ID)` where ID is the full reference (e.g., `user_1` or `User:user_1`)

5. **Expression**: Starts with `$(` and forms a valid Expression Token per Section 4.6.6 → `Expression(text)`
   - `text` is content between parentheses
   - No validation of expression syntax

6. **Alias**: Exact match of alias key → expand to defined string value
   - Apply inference to the expanded string:
     - If matches **Boolean** → Boolean
     - If matches **Number** → Integer or Float
     - Otherwise → String

7. **Boolean**: `true` or `false` → boolean
   - Case-sensitive, lowercase

8. **Number**: Matches `^-?[0-9]+(\.[0-9]+)?$` → integer or float
   - Same rules as Key-Value numbers (Section 8.2)
   - `42` → integer, `42.0` → float

9. **String**: Default
   - Unquoted strings are trimmed
   - Empty unquoted string → empty string

**ID Column Special Handling**: For the first column (ID column):
- If raw unquoted token is `^`: raise `SemanticError` with message "Ditto not permitted in ID column"
- If raw unquoted token is `~`: raise `SemanticError` with message "Null not permitted in ID column"
- After inference, value MUST be string and MUST match ID token pattern (Section 4.6)
- If inference produces non-string: `SemanticError`

### 9.4 Ditto Scoping Rules

The `^` operator copies from the **same column** of the **previous row** in the **same list frame**:

1. **Scope**: Current List Frame only
   - Doesn't copy from parent or child lists
   - Each list maintains its own `lastRowValues`

2. **History**: `LastRowValues` tracked per frame
   - Reset when list frame is popped
   - Updated after each successful row parse

3. **First Row**: `^` on first row → `SemanticError`
   - No previous row to copy from

4. **Type Preservation**: Copies value AS IS (including type)
   - If previous value was `Reference("id")`, ditto copies the reference
   - If previous value was `Expression("x+1")`, ditto copies the expression
   - If previous value was `null`, ditto copies `null`

5. **Expression Ditto**: If copying an expression, copies the expression object, not its evaluation

6. **Quoted Ditto**: `"^"` (quoted) is string `"^"`, not ditto operator

**Example**:
```hedl
data: @Item[id,name,count,price]
  |i1,Apple,5,1.99
  |i2,^,3,^      # name copies "Apple", price copies 1.99
  |i3,Orange,^,2.49  # count copies 3 (integer)
```

**Ditto Chain Example**:
```hedl
|a,1,true
|b,^,^     # copies 1, true
|c,^,false # copies 1 (from previous row), false (new value)
|d,2,^     # 2 (new), false (copied)
```

### 9.5 Shape Validation

After parsing CSV cells:
1. Count cells in row
2. Compare with schema column count
3. If mismatch → `ShapeError`
4. Cell count MUST match exactly (no optional columns)

**ShapeError Messages**:
- Too few cells: `Expected X columns, got Y`
- Too many cells: `Expected X columns, got Y`

**Examples**:
```hedl
%STRUCT: User: [id,name,email]
---
users: @User
  |u1,Alice               # ShapeError: Expected 3 columns, got 2
  |u2,Bob,bob@ex.com,extra  # ShapeError: Expected 3 columns, got 4
  |u3,Carol,carol@ex.com # OK
```

### 9.6 Count Hints (Optional)

Count hints provide optional metadata about the number of direct children for parent rows in nested hierarchies. They are particularly useful for LLM consumption, as they help models understand data structure boundaries.

**Syntax**:
- Parent rows with N children: `|[N] data` where N is a non-negative integer
- Leaf rows (no children): `|data` (no count prefix)
- The brackets `[N]` clearly separate the count from the data

**Rules**:
1. Count hints are OPTIONAL; parsers MUST accept rows with or without count hints
2. When present, the count hint appears immediately after the `|` delimiter and before the first data field
3. The count N MUST be a non-negative integer (0 or positive)
4. The count indicates the number of DIRECT children only (not all descendants)
5. Count hints are informational; parsers MAY validate accuracy but are NOT REQUIRED to
6. Whitespace between `|`, `[N]`, and the first data field follows standard CSV trimming rules

**Preprocessing Algorithm**:

When processing a matrix row line, parsers MUST extract count hints before CSV parsing:

```python
def extract_count_hint(csv_content):
    """
    Extract count hint from CSV content.
    Returns (count_hint, remaining_csv) where count_hint is int or None.
    """
    trimmed = csv_content.lstrip()
    if not trimmed.startswith('['):
        return (None, csv_content)

    # Find closing bracket
    close_idx = trimmed.find(']')
    if close_idx == -1:
        raise SyntaxError("Unclosed count hint bracket")

    # Extract count value
    count_str = trimmed[1:close_idx].strip()
    if not count_str.isdigit():
        raise SyntaxError(f"Invalid count hint: [{count_str}] must be non-negative integer")

    count = int(count_str)
    remaining = trimmed[close_idx+1:].lstrip()

    return (count, remaining)
```

**Examples**:
```hedl
%STRUCT: Organization: [id,name]
%STRUCT: Department: [id,name]
%STRUCT: Employee: [id,name]
%NEST: Organization > Department
%NEST: Department > Employee
---
organizations: @Organization
  |[2] org1,TechCorp          # This org has 2 direct children (departments)
    |[3] dept1,Engineering    # This dept has 3 direct children (employees)
      |emp1,Alice             # Leaf node - no count hint
      |emp2,Bob               # Leaf node
      |emp3,Carol             # Leaf node
    |[1] dept2,Sales          # This dept has 1 direct child
      |emp4,David             # Leaf node
  |[1] org2,DataCo            # This org has 1 direct child
    |[0] dept3,Research       # This dept has 0 children (empty department)
```

**Validation** (Optional):

Implementations MAY optionally validate that count hints match actual child counts:

```python
def validate_count_hint(node, count_hint):
    """
    Optional validation that count hint matches actual children.
    """
    if count_hint is not None:
        actual_count = len(node.children)
        if count_hint != actual_count:
            # Warning or error at implementation discretion
            warn(f"Count hint mismatch: expected {count_hint}, got {actual_count}")
```

**Use Cases**:
- Helping LLMs understand hierarchical structure boundaries
- Providing metadata for streaming parsers about upcoming data
- Enabling integrity checks for data transmission
- Documenting expected data structure for human readers

**Canonical Form**:

Canonical formatters SHOULD include accurate count hints for parent rows in nested hierarchies. Count hints SHOULD be omitted for leaf rows (rows with no children).

---

## 10. Identity and Graph Semantics

### 10.1 Implicit Node IDs

**Rule**: First column of every schema is the ID column

**ID Column Requirements**:
1. ID value MUST be string after parsing
2. If inference produces non-string (number, boolean, null, reference, expression): `SemanticError`
3. IDs MUST match the ID token pattern (Section 4.6):
   - Start with lowercase letter or underscore
   - Contain only letters, digits, underscore, hyphen
   - Case-sensitive (`user1` ≠ `User1`)
4. IDs are scoped within their type (Section 10.2)

**ID Column Examples**:
```hedl
# Valid IDs
|user_1,...      # OK
|item-two,...    # OK
|_system,...     # OK
|config_file,... # OK

# Invalid IDs
|User1,...       # SemanticError - starts uppercase
|123item,...     # SemanticError - starts with digit
|user id,...     # SemanticError - contains space
|"",...          # SemanticError - empty string
|~,...           # SemanticError - null not allowed
|^,...           # SemanticError - ditto not allowed
|42,...          # SemanticError - number, not string
|true,...        # SemanticError - boolean, not string
```

### 10.2 Scoped Namespace

IDs are scoped within their type to prevent naming conflicts:

**Scoping Rules**:
1. IDs MUST be unique within their type (`User`, `Post`, etc.)
2. Same ID MAY appear in different types (`user:admin` and `role:admin` can coexist)
3. **Local References** (`@id`): Search ONLY in the current list's type namespace
4. **Qualified References** (`@Type:id`): Search in the specified `Type` namespace
5. **Cross-Type References**: MUST use qualified syntax (e.g., `@Role:admin` from inside a `User` list)

**Rationale**: Enables modular data and safe file concatenation while allowing rich graph relationships.

**Collision Examples**:
```hedl
%STRUCT: User: [id,name]
%STRUCT: Product: [id,name]
---
users: @User
  |admin,Alice
products: @Product
  |admin,Laptop  # OK - different type namespace
```

### 10.3 Reference Resolution

References (`@id`) create placeholders resolved in second pass:

**Two-Pass Algorithm**:
1. **Pass 1**: Parse structure, populate Type-scoped Node Registries, record references
2. **Pass 2**: Resolve each reference against appropriate Type Registry

**Resolution Rules**:
1. **Strict Mode** (REQUIRED): Unresolved reference → `ReferenceError`
2. References are resolved within the current type's namespace
3. **Forward References**: Allowed within same type (reference before definition)

**Reference Examples**:
```hedl
%STRUCT: Task: [id,name,depends_on]
---
tasks: @Task
  |t1,Design,~
  |t2,Implement,@t1    # Forward reference - OK (same type)
  |t3,Test,@t2
  |t4,Deploy,@t99      # ReferenceError - t99 doesn't exist in Task namespace
```

**Reference Cycles**: Allowed (no validation)
```hedl
|a,Task A,@b
|b,Task B,@a  # Circular reference - allowed but may cause issues in applications
```

#### 10.3.1 Key-Value Reference Resolution (Normative)

References in Key-Value context (outside matrix lists) follow these rules:

1. **Qualified References Required**: In Key-Value context, references SHOULD be qualified (`@Type:id`) to ensure unambiguous resolution
2. **Unqualified References in Key-Value**: If an unqualified reference (`@id`) appears in a Key-Value context:
   - The parser MUST search ALL type registries for a matching ID
   - If exactly one match is found: resolve to that node
   - If multiple matches are found (same ID in different types): `ReferenceError` with message "Ambiguous unqualified reference '@id' matches multiple types: [Type1, Type2, ...]"
   - If no match is found: `ReferenceError` (strict mode) or `null` (lenient mode)
3. **Recommendation**: Tool authors SHOULD emit warnings for unqualified references in Key-Value context

**Examples**:
```hedl
%VERSION: 1.0
%STRUCT: User: [id,name]
%STRUCT: Post: [id,content]
---
users: @User
  |alice,Alice
posts: @Post
  |p1,Hello
config:
  admin_ref: @User:alice    # Qualified - recommended
  post_ref: @Post:p1        # Qualified - recommended
  ambiguous: @alice         # Unqualified - searches all types, finds User:alice
```

### 10.4 Child List Attachment

When `%NEST: Parent > Child` is active:

**Semantics**:
- Child rows attach to most recent parent row
- Attachment is implicit via indentation
- Creates parent-child relationship in graph
- Child list is accessible via `parent.children["ChildType"]`

**Attachment Rules**:
1. Child row MUST be indented exactly one level deeper than parent row
2. Parent must have been parsed (child cannot come before parent in same list)
3. If no parent row parsed yet → `OrphanRowError`

**In-Memory Representation**:
```javascript
// Node structure
{
  id: "parent_id",
  type: "ParentType",
  properties: { /* schema columns */ },
  children: {
    "ChildType": [childNode1, childNode2, ...]
  }
}
```

**Serialization Requirement**:
Generators MUST indent child rows one level deeper than parent rows.

**Example**:
```hedl
%STRUCT: User: [id,name]
%STRUCT: Post: [id,content]
%NEST: User > Post
---
users: @User
  |u1,Alice
    |p1,"Hello world"    # Attaches to u1
    |p2,"Second post"    # Attaches to u1
  |u2,Bob
    |p3,"Hi there"       # Attaches to u2
```

### 10.5 Node Structure and Representation

A matrix row produces a **Node** with the following structure:

1. **Type**: The TypeName from the schema
2. **Properties**: A mapping from column name (per schema) to parsed value
   - For schema `[id, name, age]` and row `| u1, Alice, 30`:
     - Node properties: `{"id": "u1", "name": "Alice", "age": 30}`

3. **Children** (optional): A mapping from child TypeName to list of child Nodes
   - Only present if node has child lists via `%NEST`
   - Format: `children: Map<TypeName, List<Node>>`
   - Example: `{"Post": [childNode1, childNode2]}`

4. **Type Registry**: Per-type mapping `ID → Node`
   - Each node MUST have unique ID within its type
   - Registry used for reference resolution

**Reference Resolution (normative observable model)**:

* Parsed references MUST be represented as `Reference(id)` values in the resulting tree (the observable value is the referenced ID).
* During Pass 2, the parser MUST validate that each `Reference(id)` resolves to an existing Node ID in strict mode (or convert to `null` in lenient mode).
* Implementations MAY internally attach a direct pointer to the resolved Node (e.g., `ref.target`), but they MUST preserve the logical scalar value as `Reference(id)` for API and test equivalence.

**Complete Node Example**:
```hedl
%STRUCT: User: [id,name,age]
%STRUCT: Post: [id,title]
%NEST: User > Post
---
users: @User
  |u1,Alice,30
    |p1,"First post"
    |p2,"Second post"
  |u2,Bob,25
```

Resulting structure:
```javascript
{
  "users": [
    {
      "id": "u1",
      "type": "User",
      "name": "Alice", 
      "age": 30,
      "children": {
        "Post": [
          {"id": "p1", "type": "Post", "title": "First post"},
          {"id": "p2", "type": "Post", "title": "Second post"}
        ]
      }
    },
    {
      "id": "u2",
      "type": "User",
      "name": "Bob",
      "age": 25,
      "children": {}  // No posts
    }
  ]
}
```

---

## 11. Parsing Algorithm (Normative)

### 11.1 Preprocessing Phase

Parser MUST perform in order:

1. **Read Input**: As bytes or UTF-8 stream
2. **BOM Handling**: Skip UTF-8 BOM if present (optional warning)
3. **UTF-8 Validation**: Validate complete UTF-8 sequences, reject invalid bytes
4. **Control Character Check**: Reject ASCII control characters (0x00-0x1F, except 0x0A, 0x0D, 0x09). Tab (0x09) allowed only in quoted strings and expressions.
5. **Line Ending Normalization**: Convert `\r\n` to `\n`. Reject bare `\r` (CR-only).
6. **Split Lines**: On `\n`, preserving empty lines
7. **Line Number Tracking**: Maintain 1-based line numbers for error reporting

**Pseudocode**:
```python
def preprocess(input_data, max_size=1024*1024*1024):  # 1GB default
    # Check size
    if len(input_data) > max_size:
        raise SecurityError(f"File too large: {len(input_data)} > {max_size}")
    
    # Validate and Decode UTF-8
    try:
        text = input_data.decode('utf-8')
    except UnicodeDecodeError:
        raise SyntaxError("Invalid UTF-8 encoding")
    
    # Skip BOM if present
    if text.startswith('\ufeff'):
        text = text[1:]
    
    # Check for control characters (allow LF, CR, TAB)
    for i, ch in enumerate(text):
        code = ord(ch)
        # Allow: LF (0x0A), CR (0x0D), TAB (0x09)
        if code < 0x20 and code not in (0x0A, 0x0D, 0x09):
            raise SyntaxError(f"Control character U+{code:04X} at position {i}")
    
    # Note: Tab usage is restricted by specific parsers:
    # - Indentation: Tabs PROHIBITED (Section 4.3)
    # - Unquoted Strings: Tabs PROHIBITED (Section 4.8)
    # - Quoted Strings: Tabs ALLOWED (Section 8.1.1)
    
    # Normalize line endings: CRLF -> LF, reject bare CR
    if '\r' in text:
        # Replace CRLF first
        text = text.replace('\r\n', '\n')
        # Now check for any remaining bare CR (not part of CRLF)
        if '\r' in text:
            # Find the line number where bare CR occurs
            line_num = text[:text.index('\r')].count('\n') + 1
            raise SyntaxError(f"Bare CR (U+000D) found at line {line_num}")
    
    # Split lines
    lines = text.split('\n')
    return lines
```

### 11.2 Header Parsing

1. **Initialize Registries**:
   - SchemaRegistry: dict TypeName → [columns]
   - AliasRegistry: dict key → string
   - NestRegistry: dict ParentType → ChildType
   - TypeNodeRegistries: dict TypeName → dict ID → Node

2. **Read Lines** until separator (`---`)
   - Track line numbers
   - Skip blank lines and comments

3. **Parse Directives**:
   - Before validating and parsing a directive line, strip inline comments using the non-matrix rule in Section 4.5.
   - Validate format `%NAME: payload`
   - Update appropriate registry
   - Validate constraints (unique names, etc.)

4. **Separator Detection**: Strict validation per Section 5.3
5. **Missing Separator**: → `SyntaxError`

**Header Parsing Algorithm**:
```python
def parse_header(lines):
    schemas = {}
    aliases = {}
    nests = {}
    version_seen = False
    first_directive = True
    
    for line_num, line in enumerate(lines, 1):
        # Strip leading/trailing whitespace for separator check
        stripped_line = line.rstrip('\n')
        
        # Check for separator with strict leading space rule
        if stripped_line == '---' or stripped_line.startswith('--- '):
            if not version_seen:
                raise SyntaxError(f"Missing %VERSION directive in header before separator at line {line_num}")
            # Valid separator found
            return schemas, aliases, nests, line_num + 1
            
        # Check for malformed separator (leading spaces or extra hyphens)
        if stripped_line.lstrip().startswith('---'):
            if stripped_line.startswith(' '):
                raise SyntaxError(f"Separator '---' must not have leading spaces at line {line_num}")
            else:
                # Something like '----' 
                raise SyntaxError(f"Separator must be exactly '---', found '{stripped_line[:10]}...' at line {line_num}")
        
        if not line.strip() or line.strip().startswith('#'):
            continue
            
        # Parse directive
        if not line.startswith('%'):
            raise SyntaxError(f"Expected directive at line {line_num}")
            
        # Split directive name and payload with flexible spacing
        if ':' not in line:
            raise SyntaxError(f"Invalid directive format at line {line_num}")
            
        name, payload = line.split(':', 1)
        if not payload.startswith(' '):
            raise SyntaxError(f"Directive ':' must be followed by at least one space at line {line_num}")
        
        # Enforce %VERSION as first directive
        if first_directive:
            if name != '%VERSION':
                raise SyntaxError(f"%VERSION must be the first directive, found {name} at line {line_num}")
            first_directive = False
        
        payload = payload.lstrip(' ')
        
        # Remove inline comment from payload
        payload = strip_comment(payload)
        
        # Dispatch based on directive name
        if name == '%VERSION':
            parse_version(payload, line_num)
            version_seen = True
        elif name == '%STRUCT':
            parse_struct(payload, schemas, line_num)
        elif name == '%ALIAS':
            parse_alias(payload, aliases, line_num)
        elif name == '%NEST':
            parse_nest(payload, nests, schemas, line_num)
        else:
            raise SyntaxError(f"Unknown directive {name} at line {line_num}")
    
    raise SyntaxError("Missing separator '---'")
```

### 11.3 Context Stack Frames

Each frame contains:

**Root Frame** (initial):
```typescript
{
  kind: "Root",
  indent: -1,           // Virtual indent level
  object: {}            // Root object being built
}
```

**Object Frame**:
```typescript
{
  kind: "Object",
  indent: number,       // indent level of object start line
  object: {},           // the object being built
  parent: Frame,        // parent frame
  parentKey: string     // key in parent object
}
```

**List Frame**:
```typescript
{
  kind: "List",
  typeName: string,     // e.g., "User"
  schema: string[],     // column names
  listStartIndent: number,  // indent of list start line
  rowIndent: number,        // indent level where rows appear
  lastRowValues: any[] | null,  // values from previous row
  lastItem: any | null,     // last node created
  parentObject: object,     // object containing the list
  parentKey: string,        // key in parent object
  list: any[]               // the list being built
}
```

**Note on `listStartIndent`**:
- For explicit lists (from `key: @TypeName`): `listStartIndent` is the indent level of the list start line
- For implicit lists (via `%NEST`): `listStartIndent` is synthetic = current indent level - 1
- `rowIndent` is always `listStartIndent + 1`

### 11.4 Scope Closing (Pop Rules)

Before processing each Body line at indent level `I`:

```python
def pop_frames(stack, current_indent):
    """Pop frames that are no longer relevant."""
    while True:
        top = stack[-1]
        
        if top.kind == "List":
            # Pop list if we're leaving its row scope
            if current_indent < top.rowIndent:
                stack.pop()
                continue
                
        elif top.kind == "Object":
            # Pop object if we're returning to its level or shallower
            if current_indent <= top.indent:
                stack.pop()
                continue
                
        # No more frames to pop
        break
```

**Rationale**: Objects close when we return to their level or shallower; lists close when we leave their row scope.

### 11.5 Line Dispatch

After popping, dispatch based on line content:

#### Case 1: Object Start (`key:`)
- **Requirement**: Top is Root or Object, `I == top.indent + 1`
- **Action**: 
  1. Create empty object
  2. Assign to `parent[key]`
  3. Push Object frame with `indent = I`
- **Error**: Wrong indent → `SyntaxError`

#### Case 2: Key-Value (`key: value`)
- **Requirement**: Top is Root or Object, `I == top.indent + 1`
- **Action**: 
  1. Parse value (Section 8)
  2. Assign to `parent[key]`
- **Error**: Wrong indent → `SyntaxError`

#### Case 3: Matrix List Start (`key: @TypeName`)
- **Requirement**: Top is Root or Object, `I == top.indent + 1`
- **Action**:
  1. Resolve schema (declared or inline)
  2. Create empty list at `parent[key]`
  3. Push List frame with:
     - `listStartIndent = I`
     - `rowIndent = I + 1`
     - `lastRowValues = null`
     - `lastItem = null`

#### Case 4: Matrix Row (`| ...`)
- **Requirement**: Top is List frame
- **Subcases**:

**Peer Row** (`I == L.rowIndent`):
1. Verify `I == L.rowIndent` else `SyntaxError`
2. Strip comment from line
3. Parse CSV, validate shape
4. Apply cell inference (with ID column special handling)
5. Create node, register ID (check for type-scoped collisions)
6. Update `L.lastRowValues`, `L.lastItem`
7. Append to `L.list`

**Child Row** (`I == L.rowIndent + 1`):
1. Verify `I == L.rowIndent + 1` else `SyntaxError`
2. Verify `L.lastItem` exists → else `SemanticError` ("Orphan child row")
3. Look up `%NEST` for `L.typeName` → else `OrphanRowError`
4. Get or create child list in `L.lastItem.children[ChildType]`
5. Push new List frame for child list with:
   - `listStartIndent = I - 1` (synthetic)
   - `rowIndent = I`
   - `parentObject = L.lastItem`
   - `parentKey = ChildType`
   - `list = child list from step 4`
6. Re-parse current line as peer row in new frame

**Invalid**: Any other indentation → `SyntaxError`

### 11.6 Post-Processing and Truncation Detection

After Body parsing:

1. **Validate Stack**: Stack MUST contain only the Root frame
   - If other frames remain → `SyntaxError` ("Unclosed structure at end of file")
   - This detects truncation in the middle of objects/lists

2. **Validate Incomplete Tokens**: 
   - Check for unterminated quoted strings in last line
   - Check for unterminated expressions in last line
   - If found → `SyntaxError` ("Truncated token at end of file")

3. **Reference Resolution**: Resolve all recorded `@id` references within type namespaces
   ```python
   def resolve_references(node, type_registries, current_type=None, strict=True):
       if isinstance(node, Reference):
           # Determine target type and ID
           if ':' in node.id:
               target_type, target_id = node.id.split(':', 1)
               # Strip optional @ if present in split (grammar handles this, but for safety)
               if target_type.startswith('@'): target_type = target_type[1:]
           else:
               target_type = current_type
               target_id = node.id

           # Resolve
           if target_type and target_type in type_registries:
               registry = type_registries[target_type]
               if target_id in registry:
                   return node  # Resolved
               elif strict:
                   raise ReferenceError(f"Unresolved reference @{target_id} in type {target_type}")
               else:
                   return None
           elif strict:
                raise ReferenceError(f"Cannot resolve reference to unknown type {target_type}")
           return node
           
       elif isinstance(node, dict):
           # If this dict represents a typed Node, update current_type
           # (Assuming Node structure from Section 10.5 where properties are merged)
           new_type = node.get("type", current_type)
           return {k: resolve_references(v, type_registries, new_type, strict) for k, v in node.items()}
           
       elif isinstance(node, list):
           return [resolve_references(item, type_registries, current_type, strict) for item in node]
       else:
           return node
   ```

4. **Validation**: Ensure no dangling references in strict mode
5. **Return Root**: Return the root object

### 11.7 Complete Pseudo-Code with Truncation Detection

```python
class HEDLParser:
    def __init__(self, strict=True, max_indent=50, max_nodes=10_000_000):
        self.strict = strict
        self.max_indent = max_indent
        self.max_nodes = max_nodes
        self.schemas = {}
        self.aliases = {}
        self.nests = {}
        self.type_registries = {}  # type -> {id -> node}
        self.references = []  # List of (type, id, path) tuples
        self.node_count = 0
        self.current_type = None  # Track current type for reference resolution
        
    def parse(self, text):
        # Phase 1: Preprocessing
        lines = self.preprocess(text)
        
        # Phase 2: Header parsing
        try:
            body_start = self.parse_header(lines)
        except SyntaxError as e:
            # Check if error is due to EOF before separator
            if "Missing separator" in str(e):
                raise SyntaxError("Truncated file: missing separator '---'")
            raise
        
        # Phase 3: Body parsing
        root = self.parse_body(lines[body_start:])
        
        # Phase 4: Post-processing
        # Truncation validation is handled inside parse_body
        self.resolve_references(root, self.type_registries)
        
        return root
    
    def parse_body(self, lines):
        stack = [{"kind": "Root", "indent": -1, "object": {}}]
        
        for line_num, line in enumerate(lines, 1):
            if self.is_blank(line) or self.is_comment(line):
                continue
            
            stripped_line_content = line.strip()
            if stripped_line_content == '---':
                raise SyntaxError(f"Multiple separators '---' are not allowed. Found at line {line_num}.")
                
            indent = self.calculate_indent(line)
            self.validate_indent(indent)
            
            # Scope closing
            self.pop_frames(stack, indent)
            
            # Classify and parse line
            line_content = line[indent*2:]  # Remove indentation
            
            if line_content.startswith('|'):
                self.parse_matrix_row(stack, line_content, indent, line_num)
            else:
                self.parse_non_matrix_line(stack, line_content, indent, line_num)
        
        # Pop remaining frames except root
        while len(stack) > 1:
            frame = stack.pop()
            if frame["kind"] == "Object":
                raise SyntaxError(f"Unclosed object '{frame.get('parentKey', '?')}' at end of file")
            elif frame["kind"] == "List":
                raise SyntaxError(f"Unclosed list '{frame.get('typeName', '?')}' at end of file")
        
        # Check for unterminated tokens in last line
        if lines and not self.is_blank(lines[-1]):
            last_line = lines[-1].rstrip('\n')
            if self.is_unterminated_token(last_line):
                raise SyntaxError("Truncated token at end of file")
            
        return stack[0]["object"]
    
    def is_unterminated_token(self, line):
        """Check if line ends with unterminated token using scan_regions."""
        regions = scan_regions(line)
        
        # Check for unclosed quoted string or expression that extends to the end of the line
        for start, end, _type in regions:
            if end == len(line): # Region extends to end of line
                if _type == "quote" and line[end-1] != '"':
                    return True # Unclosed quote
                if _type == "expression" and line[end-1] != ')':
                    return True # Unclosed expression
        
        return False
```

---

## 12. Error Hierarchy

### 12.1 Error Categories

| Error | When Raised | Recoverable? | Example |
|-------|-------------|--------------|---------|
| `SyntaxError` | Lexical or structural violation | No | Odd indentation, tab character, unclosed structure |
| `VersionError` | Unsupported version | No | `%VERSION: 2.0` with 1.0 parser |
| `SchemaError` | Schema violation or mismatch | No | Duplicate struct, nest to undefined type |
| `AliasError` | Duplicate or invalid alias | No | `%ALIAS: %key: "val"` (duplicate) |
| `ShapeError` | Wrong number of cells in row | No | Expected 3 columns, got 2 |
| `SemanticError` | Logical error | No | Ditto in ID column, null in ID column |
| `OrphanRowError` | Child row without %NEST | No | Indented row with no nest rule |
| `CollisionError` | Duplicate ID within type | No | Same ID in same type |
| `ReferenceError` | Unresolved reference (strict mode) | No | `@missing` with no definition |
| `SecurityError` | Security limit exceeded | No | File too large, nesting too deep |

### 12.2 Error Details and Messages

**SyntaxError Examples**:
- `Line X: Invalid indentation - tabs are not allowed for indentation`
- `Line X: Expected 2-space indentation, got 3 spaces`
- `Line X: Missing space after colon in key:value`
- `Line X: Unclosed quoted string`
- `Line X: Unclosed structure at end of file (truncated)`
- `Line X: Unterminated expression`
- `Line X: Bare CR (U+000D) found`

**SchemaError Examples**:
- `Struct 'User' already defined with different columns`
- `Nest parent type 'User' not defined`
- `Inline schema for 'User' doesn't match declared schema`

**SemanticError Examples**:
- `Line X: Ditto (^) not permitted in ID column`
- `Line X: Null (~) not permitted in ID column`
- `Line X: ID must be string, got number`
- `Line X: Invalid ID format 'User1' - must start with lowercase or underscore`

**CollisionError Example**:
- `Duplicate ID 'user1' in type 'User' at line Y, previously defined at line X`

### 12.3 Recovery Guidelines

Parsers SHOULD:
1. Report first error encountered with line number and column
2. Provide clear error message explaining violation
3. Include context (e.g., "in User list started at line 5")
4. MAY continue parsing for additional errors (best effort)

Parsers MUST NOT:
1. Guess or auto-correct errors
2. Ignore errors (except BOM warning)
3. Provide different output for erroneous input
4. Implement "lenient" mode for syntax errors (only for reference resolution)

**Error Recovery Example**:
```python
try:
    result = parser.parse(text)
except HEDLError as e:
    print(f"Error at line {e.line}: {e.message}")
    if e.context:
        print(f"Context: {e.context}")
    raise
```

### 12.4 Error Class Definition

```python
class HEDLError(Exception):
    """Base class for all HEDL errors."""
    def __init__(self, message, line=None, column=None, context=None):
        self.message = message
        self.line = line
        self.column = column
        self.context = context
        super().__init__(self.format_message())
        
    def format_message(self):
        parts = []
        if self.line is not None:
            parts.append(f"line {self.line}")
        if self.column is not None:
            parts.append(f"column {self.column}")
        if parts:
            location = " at " + ":".join(parts)
        else:
            location = ""
        return f"{self.__class__.__name__}{location}: {self.message}"

class SyntaxError(HEDLError):
    pass

class VersionError(HEDLError):
    pass

class SchemaError(HEDLError):
    pass

class AliasError(HEDLError):
    pass

class ShapeError(HEDLError):
    pass

class SemanticError(HEDLError):
    pass

class OrphanRowError(HEDLError):
    pass

class CollisionError(HEDLError):
    pass

class ReferenceError(HEDLError):
    pass

class SecurityError(HEDLError):
    pass
```

---

## 13. Canonicalization (Generators)

To ensure stable hashing, diffing, and deterministic output:

### 13.1 Required Practices

1. **Line Endings**: `\n` only
2. **No Trailing Whitespace**: Trim end of every line
3. **Separator**: Exactly `---\n`
4. **Indentation**: Exactly 2 spaces per level, no tabs
5. **No BOM**: Do not include UTF-8 BOM

### 13.2 Header Directive Order

Generate directives in this order:

1. `%VERSION: 1.0`
2. `%ALIAS`: Sorted by key (ASCII ascending)
3. `%STRUCT`: Sorted by TypeName (ASCII)
4. `%NEST`: Sorted by ParentType then ChildType (ASCII)

**Example**:
```hedl
%VERSION: 1.0
%ALIAS: %active: "true"
%ALIAS: %inactive: "false"
%STRUCT: Post: [id,content]
%STRUCT: User: [id,name]
%NEST: User > Post
---
```

### 13.3 Quoting Strategy

**Matrix Cells**: Quote if field contains:
- Comma `,`
- Quote `"` (then escape as `""` or `\"`)
- Pipe `|`
- Hash `#`
- Leading or trailing whitespace
- Control characters (newline, tab, carriage return) - use escape sequences `\n`, `\t`, `\r`
- Backslash `\` - escape as `\\`
- Would trigger unwanted inference (e.g., `true` as string, not boolean)

**Escape Sequences in Matrix Cells**: When canonical output contains control characters:
- Newline → `\n`
- Tab → `\t`
- Carriage return → `\r`
- Backslash → `\\`
- Quote → `\"` or `""`

Example: A cell with value `Hello` followed by newline and `World` becomes `"Hello\nWorld"`.

**Key-Value Values**: Quote to preserve:
- Leading/trailing whitespace
- Hash `#` (to avoid comment interpretation)
- When inference should be prevented
- When value equals alias name (e.g., `"%active"` to prevent expansion)

**Block Strings for Key-Value**: When a key-value string contains newlines:
- MUST use block strings (`"""`) for canonical output
- Do NOT use escape sequences in key-value context (backslash is literal)

Example:
```hedl
description: """
Line 1
Line 2
"""
```

**Empty Strings**: In matrix cells, represent as empty field (no quotes) `, ,`. EXCEPTION: If the last column is empty, it MUST be represented as `""` to avoid a trailing comma (which is a SyntaxError).

**Boolean and Null**: Use unquoted `true`, `false`, `~`.

**Numbers**: Integers represented without decimals (`42`). Floats represented with decimal point (`42.0`) to preserve type.

**Tensor Literals**: Always unquoted, with consistent spacing: `[1, 2, 3]` not `[1,2,3]`.

**Matrix Row Comments**: Canonical output MUST omit all comments. Pretty-printers MAY preserve comments in human-oriented output.

### 13.4 Ditto Optimization

Use `^` when value equals previous row same column in same list.

**Rules**:
1. Only in matrix cells, not Key-Value
2. Not in ID column
3. Not in first row
4. Compare values deeply (including type)

**Example**:
```hedl
|a,Apple,1.99
|b,^,0.99    # Apple copied
|c,Orange,^  # 0.99 copied
```

### 13.5 Object Key Sorting

Object keys sorted ASCII ascending (order not significant semantically).

**Example**:
```hedl
# Instead of:
zebra: 1
apple: 2

# Canonical:
apple: 2
zebra: 1
```

### 13.6 ID Format

Always valid ID token (no quoting needed).
- Start with lowercase or underscore
- Use hyphens for word separation (convention)
- Be descriptive but concise

### 13.7 Matrix List Metadata

Matrix lists in the parsed output MUST include metadata for canonicalization:

**Normative Requirement**: 
- The parsed representation of a matrix list MUST include `type_name` and `schema` properties
- These properties MAY be implemented as metadata attributes, wrapper objects, or separate data structures
- Generators MUST have access to this metadata for round-trip canonicalization

**Example Implementation**:
```python
class MatrixList:
    def __init__(self, type_name, schema):
        self.type_name = type_name
        self.schema = schema
        self.rows = []  # List of dicts mapping column->value
        
    # Allow list-like access
    def __len__(self):
        return len(self.rows)
    
    def __getitem__(self, idx):
        return self.rows[idx]
    
    def append(self, row):
        self.rows.append(row)
```

### 13.8 Complete Canonicalization Algorithm (Informative)

**Note**: This section is non-normative and provided for illustration only. Implementations MUST adhere to the normative requirements in Sections 13.1-13.7 but are NOT required to follow this specific algorithm.

```python
def canonicalize(data, indent=0):
    """Convert Python data structure to canonical HEDL. (Informative example)"""
    if isinstance(data, dict):
        # Sort keys
        items = sorted(data.items(), key=lambda x: x[0])
        
        lines = []
        for key, value in items:
            if isinstance(value, MatrixList):
                # Matrix list with metadata
                lines.append(canonicalize_matrix_list(key, value, indent))
            elif isinstance(value, dict):
                # Object
                lines.append(' ' * indent + f"{key}:")
                lines.extend(canonicalize(value, indent + 1))
            else:
                # Scalar
                lines.append(' ' * indent + f"{key}: {canonicalize_value(value)}")
        return lines
    else:
        return []

def canonicalize_matrix_list(key, matrix_list, indent):
    """Canonicalize a matrix list."""
    type_name = matrix_list.type_name
    schema = matrix_list.schema
    
    lines = []
    # Use inline schema for canonical form
    lines.append(' ' * indent + f"{key}: @{type_name}[{', '.join(schema)}]")
    
    # Output rows with ditto optimization
    last_values = None
    for row in matrix_list.rows:
        # row is dict mapping column name to value
        row_values = [row[col] for col in schema]
        canonical_row = canonicalize_matrix_row(row_values, last_values)
        lines.append(' ' * (indent + 1) + canonical_row)
        last_values = row_values
    
    return '\n'.join(lines)

def canonicalize_matrix_row(values, last_values):
    """Canonicalize a single matrix row with ditto optimization."""
    cells = []
    for i, value in enumerate(values):
        if last_values is not None and value == last_values[i]:
            cells.append('^')
        else:
            cells.append(canonicalize_value(value, in_matrix=True))
    return '| ' + ', '.join(cells)
```

---

## 14. Security Considerations

### 14.1 Denial of Service

Parsers SHOULD enforce:

1. **Maximum Indent Depth**: Default 50 levels (100 spaces)
2. **Maximum Line Length**: Default 1MB per line
3. **Maximum Nodes**: Default 10 million nodes
4. **Maximum Aliases**: Default 10,000 aliases
5. **Maximum Columns**: Default 100 columns per schema
6. **Maximum File Size**: Default 1GB total
7. **Recursion Limits**: For nested structures
8. **Memory Limits**: Based on system capabilities
9. **Time Limits**: Maximum parsing time

**Implementation Guidance**:
```python
class SafeHEDLParser(HEDLParser):
    def __init__(self, max_indent=50, max_line_length=1024*1024, max_nodes=10_000_000, max_file_size=1024*1024*1024):
        super().__init__()
        self.max_indent = max_indent
        self.max_line_length = max_line_length
        self.max_nodes = max_nodes
        self.max_file_size = max_file_size
        self.node_count = 0
        
    def validate_line(self, line):
        if len(line) > self.max_line_length:
            raise SecurityError(f"Line too long: {len(line)} > {self.max_line_length}")
    
    def validate_indent(self, indent):
        if indent > self.max_indent:
            raise SecurityError(f"Indent too deep: {indent} > {self.max_indent}")
        
    def register_node(self, node_id, type_name):
        self.node_count += 1
        if self.node_count > self.max_nodes:
            raise SecurityError(f"Too many nodes: {self.node_count} > {self.max_nodes}")
```

### 14.2 Injection Prevention

1. **Alias Expansion**: Values are strings, expanded before inference
   - No recursion (aliases can't reference other aliases)
   - No code execution

2. **Expression Opaque**: `$(...)` never evaluated by parser
   - Treat as black box
   - Pass through unchanged

3. **Reference Resolution**: Only to existing nodes in same document
   - No external references (URLs, file paths)
   - No resolution beyond document boundaries

4. **No Code Execution**: Parser MUST NOT eval any content
   - Expressions remain strings
   - No JavaScript, no shell commands

### 14.3 Memory Safety

1. **Bounded Allocation**: Pre-allocate based on size hints if possible
2. **Streaming Parsers**: Recommended for large files
   - Process line by line
   - Don't keep entire document in memory
3. **Integer Overflow**: Validate numeric ranges for target language
   - 32-bit vs 64-bit considerations
   - Reject numbers outside safe range
4. **UTF-8 Validation**: Reject invalid byte sequences
   - Use safe UTF-8 decoder
   - Replace or reject invalid sequences

### 14.4 Confidentiality

1. **No Implicit Fetching**: References are internal only
2. **No Network Access**: Parser shouldn't resolve external URIs
3. **Information Leakage**: Errors shouldn't reveal sensitive data
   - Don't include full paths
   - Don't include sensitive values in error messages
4. **Logging**: Be careful what gets logged

### 14.5 Truncation Detection

Parsers MUST detect and reject truncated files:

1. **Unclosed Structures**: If file ends while inside object/list → SyntaxError
2. **Unterminated Tokens**: If file ends mid-quote or mid-expression → SyntaxError
3. **Partial Separator**: If file ends with `--` or `-` → SyntaxError
4. **Incomplete Directive**: If header ends mid-directive → SyntaxError
5. **Bare CR**: If file contains CR without LF → SyntaxError

**Truncation Detection Algorithm**:
```python
def detect_truncation(lines):
    """Check for truncation indicators using scan_regions."""
    if not lines:
        return False
    
    last_line = lines[-1].rstrip('\n')
    
    # Check for partial separator
    if last_line.startswith('-') and last_line != '---':
        return True
        
    # Check for unterminated tokens using normative scan_regions
    regions = scan_regions(last_line)
    for start, end, _type in regions:
        if end == len(last_line):
            if _type == "quote" and last_line[end-1] != '"':
                return True
            if _type == "expression" and last_line[end-1] != ')':
                return True
    
    return False
```

### 14.6 Implementation Security Checklist

- [ ] Validate UTF-8 encoding
- [ ] Reject control characters (except LF, CR, TAB in quoted strings)
- [ ] Limit recursion depth
- [ ] Limit memory allocation
- [ ] No eval() of expressions
- [ ] No external reference resolution
- [ ] Safe integer parsing
- [ ] Timeout for malicious inputs
- [ ] Fuzz testing recommended
- [ ] Detect truncated files
- [ ] Validate complete tokenization

---

## 15. IANA Considerations

### 15.1 Media Type Registration (Provisional)

- **Type name**: `application`
- **Subtype name**: `hedl`
- **Required parameters**: none
- **Optional parameters**:
  - `version`: HEDL version (e.g., `version=1.0`)
  - `charset`: Character encoding (default `utf-8`)
- **Encoding considerations**: binary, UTF-8 encoded
- **Security considerations**: See Section 14
- **Interoperability considerations**: Deterministic parsing, versioned format
- **Published specification**: This document
- **Applications**: AI/ML data serialization, configuration files, knowledge graphs
- **File extensions**: `.hedl`
- **Mac OS Type Code**: `TEXT`
- **Uniform Type Identifier**: `public.hedl-text`
- **Fragment identifiers**: none
- **Additional information**:
  - Magic numbers: none
  - Deprecated: false
  - Restrictions: none

### 15.2 Internet Media Type Example

```
Content-Type: application/hedl; version=1.0; charset=utf-8
```

### 15.3 File Extension Registration (Provisional)

- **Extension**: `.hedl`
- **MIME Type**: `application/hedl`
- **Description**: HEDL (Hierarchical Entity Data Language) file
- **Mac OS Type**: `TEXT`
- **UTI**: `public.hedl-text`
- **Recommended**: Use UTF-8 encoding, LF line endings

---

## 16. Normative Examples

### 16.1 Simple Mode (No Schemas)

```hedl
%VERSION: 1.0
---
config:
  database:
    host: localhost
    port: 5432
  logging:
    level: info
    file: "/var/log/app.log"
# Lists require inline schema in Simple Mode (first column is always ID)
users: @User[id,name,email,active]
  |alice,Alice,alice@example.com,true
  |bob,Bob,bob@example.com,false
```

### 16.2 Basic Typed List

```hedl
%VERSION: 1.0
%STRUCT: User: [id,name,email,active]
---
users: @User
  |u1,"Alice, Admin",alice@example.com,true
  |u2,bob,bob@example.com,false
  |u3,carol,carol@example.com,^
```

**Parsed as**:
```json
{
  "users": [
    {"id": "u1", "name": "Alice, Admin", "email": "alice@example.com", "active": true},
    {"id": "u2", "name": "bob", "email": "bob@example.com", "active": false},
    {"id": "u3", "name": "carol", "email": "carol@example.com", "active": false}
  ]
}
```

### 16.3 Nested Hierarchy

```hedl
%VERSION: 1.0
%STRUCT: Project: [id,name]
%STRUCT: Task: [id,description,status]
%NEST: Project > Task
---
projects: @Project
  |p1,Website Redesign
    |t1,Design mockups,pending
    |t2,Implement frontend,in_progress
  |p2,API Migration
    |t3,Update endpoints,done
```

**Parsed as**:
```json
{
  "projects": [
    {
      "id": "p1",
      "name": "Website Redesign",
      "children": {
        "Task": [
          {"id": "t1", "description": "Design mockups", "status": "pending"},
          {"id": "t2", "description": "Implement frontend", "status": "in_progress"}
        ]
      }
    },
    {
      "id": "p2",
      "name": "API Migration",
      "children": {
        "Task": [
          {"id": "t3", "description": "Update endpoints", "status": "done"}
        ]
      }
    }
  ]
}
```

### 16.4 References and Aliases

```hedl
%VERSION: 1.0
%ALIAS: %pending: "pending"
%ALIAS: %done: "done"
%STRUCT: Task: [id,description,status,depends_on]
---
tasks: @Task
  |t1,Design,%pending,~
  |t2,Implement,%pending,@t1
  |t3,Test,%done,@t2
```

**Parsed as**:
```json
{
  "tasks": [
    {"id": "t1", "description": "Design", "status": "pending", "depends_on": null},
    {"id": "t2", "description": "Implement", "status": "pending", "depends_on": "@t1"},
    {"id": "t3", "description": "Test", "status": "done", "depends_on": "@t2"}
  ]
}
```

### 16.5 Tensor Literals and Mixed Structure

```hedl
%VERSION: 1.0
%STRUCT: Measurement: [id,timestamp,values]
---
experiment:
  name: "Temperature Test"
  metadata:
    sensor_count: 3
    duration: 3600
  measurements: @Measurement
    |m1,1625097600,[23.5,24.1,22.9]
    |m2,1625097660,[23.7,24.0,23.1]
    |m3,^,[23.6,24.0,23.0]
```

**Note**: Ditto (`^`) works at the cell level, not inside tensor literals. In row m3, `^` copies the entire timestamp value `1625097660` from row m2. Tensor literals must contain only numbers.

### 16.6 Type-Scoped IDs Example

```hedl
%VERSION: 1.0
%STRUCT: User: [id,name]
%STRUCT: Role: [id,name]
---
users: @User
  |admin,Alice
  |user1,Bob
roles: @Role
  |admin,Administrator  # OK - different type namespace
  |user,Regular User
```

---

## 17. Extensions and Versioning

### 17.1 Versioning Scheme

**Format**: `major.minor`

- **Major version**: Breaking changes
- **Minor version**: Backward-compatible additions

**Backward Compatibility**:
- v1.1 parser MUST parse v1.0 files
- v1.0 parser MAY parse v1.1 files (if new features ignored)
- v2.0 parser MAY parse v1.x files (with compatibility mode)

### 17.2 Extension Points

1. **New Directives**: Future versions may add new `%DIRECTIVE` types
2. **New Token Types**: New special tokens
3. **Schema Extensions**: Additional column constraints
4. **New Data Types**: Beyond current inference ladder
5. **Nesting Extensions**: Multiple children per parent
6. **Qualified References**: `@TypeName:id` syntax for cross-type references
7. **Unicode IDs**: Support for Unicode in ID tokens

### 17.3 Extension Guidelines

When designing extensions:
1. Use new directive names starting with `X-` for experimental
2. Don't break existing valid documents
3. Provide migration path
4. Update reference implementation

**Example Experimental Directive**:
```hedl
%X-MAXROWS: User: 1000  # Experimental limit
```

### 17.4 Version Negotiation

Parsers SHOULD:
1. Read `%VERSION` directive first
2. Decide if they can parse
3. Warn about unknown minor versions
4. Error on unsupported major versions

---

## 18. Implementation Requirements

### 18.1 Mandatory Features

All implementations MUST:
1. Parse valid HEDL 1.0 documents correctly
2. Reject invalid documents with appropriate errors
3. Support UTF-8 encoding
4. Handle LF and CRLF line endings, reject bare CR
5. Validate indentation (2 spaces, no tabs for indentation)
6. Enforce ID column constraints
7. Detect duplicate IDs within types
8. Resolve references within type namespaces (or error in strict mode)
9. Support all inference ladder types including tensors
10. Handle ditto operator correctly
11. Detect truncated files
12. Validate complete tokenization
13. Support both simple and complex modes

### 18.2 Recommended Features

Implementations SHOULD:
1. Provide streaming API for large files
2. Support lenient reference mode (convert unresolved to null)
3. Include pretty-printing/canonicalization
4. Provide conversion to/from JSON
5. Include comprehensive test suite
6. Support schema validation
7. Provide error recovery for multiple errors
8. Include performance optimizations
9. Include security limits
10. Provide truncation detection
11. Preserve matrix list metadata (type_name, schema)

### 18.3 Optional Features

Implementations MAY:
1. Preserve comments during round-trip
2. Support schema inference from data
3. Provide graphical visualization
4. Include query capabilities
5. Support binary encoding for storage
6. Include compression integration
7. Provide IDE/editor integrations
8. Support tensor operations

### 18.4 Compliance Testing

To claim HEDL 1.0 compliance, implementations MUST:
1. Pass all tests in Appendix B
2. Handle edge cases in Section 12
3. Produce identical output for canonical examples
4. Reject all invalid examples with correct errors
5. Detect and reject truncated files
6. Support type-scoped IDs correctly

---

## 19. Compliance and Interoperability

### 19.1 Test Vectors

Implementations MUST pass these test vectors:

1. **Minimal Document**: `%VERSION: 1.0\n---\n`
2. **Simple Object**: `%VERSION: 1.0\n---\na: 1\nb: 2`
3. **Nested Object**: `%VERSION: 1.0\n---\na:\n  b: 1`
4. **Matrix List**: `%VERSION: 1.0\n%STRUCT: T: [id,v]\n---\nd: @T\n  |x,1`
5. **References**: `%VERSION: 1.0\n%STRUCT: T: [id,ref]\n---\nd: @T\n  |a,~\n  |b,@a`
6. **Type-Scoped IDs**: `%VERSION: 1.0\n%STRUCT: A: [id,v]\n%STRUCT: B: [id,v]\n---\na: @A\n  |x,1\nb: @B\n  |x,2` (should NOT error)

### 19.2 Round-trip Requirements

After parsing and re-serializing:
1. Data MUST be semantically equivalent
2. IDs MUST be preserved
3. References MUST resolve to same targets
4. Type information MUST be preserved
5. Tensor literals MUST preserve structure and values
6. Order of object keys MAY change (no semantic significance)

### 19.3 Interoperability Guidelines

For systems exchanging HEDL:
1. Agree on HEDL version
2. Share schemas if needed
3. Document alias conventions
4. Agree on ID naming conventions
5. Test with sample documents

### 19.4 Version Header Best Practices

Always include version header:
```hedl
%VERSION: 1.0
---
```

For forward compatibility:
1. Use the earliest version that supports needed features
2. Avoid experimental extensions unless agreed
3. Document any non-standard usage

---

## Appendix A: Implementation Guidelines

### A.1 Recommended Pseudo-Code Structure

```python
class HEDLParser:
    def __init__(self, strict=True, max_indent=50):
        self.strict = strict
        self.max_indent = max_indent
        self.registries = {}
        self.type_registries = {}
        self.references = []
        
    def parse(self, input_text):
        # Phase 1: Preprocessing
        lines = self.normalize_lines(input_text)
        
        # Phase 2: Header parsing
        header_lines, body_lines = self.split_sections(lines)
        self.parse_header(header_lines)
        
        # Phase 3: Body parsing
        root = self.parse_body(body_lines)
        
        # Phase 4: Reference resolution
        self.resolve_references()
        
        return root
    
    def parse_body(self, lines):
        stack = [RootFrame()]
        for line_num, line in enumerate(lines, 1):
            if self.is_blank(line) or self.is_comment(line):
                continue
                
            indent = self.calculate_indent(line)
            self.validate_indent(indent)
            
            # Scope closing
            self.pop_frames(stack, indent)
            
            # Parse based on current top frame
            self.parse_line(stack, line, indent)
```

### A.2 Handling ID Column Validation

```python
def parse_matrix_cell(self, cell_data, column_index, schema, last_row_values):
    """Parse a single matrix cell with special handling for ID column."""
    value_str, is_quoted = cell_data
    
    # Special handling for ID column (first column)
    if column_index == 0:
        if not is_quoted:
            if value_str == '^':
                raise SemanticError("Ditto not permitted in ID column")
            if value_str == '~':
                raise SemanticError("Null not permitted in ID column")
    
    # Apply normal inference ladder
    if is_quoted:
        value = value_str
    else:
        value = self.infer_value(value_str, last_row_values)
    
    # Validate ID column
    if column_index == 0:
        if not isinstance(value, str):
            raise SemanticError(f"ID must be string, got {type(value).__name__}")
        
        # Validate ID token pattern (lowercase or underscore start)
        import re
        id_pattern = re.compile(r'^[a-z_][a-z0-9_\-]*$')
        if not id_pattern.match(value):
            raise SemanticError(f"Invalid ID format: {value}")
    
    return value
```

### A.3 Matrix Row Comment Stripping

```python
def strip_matrix_row_comment(self, line):
    """Strip comment from matrix row line, preserving CSV-style quoted fields."""
    in_quotes = False
    i = 0
    result = []
    
    while i < len(line):
        ch = line[i]
        
        if not in_quotes and ch == '#':
            # Found comment start outside quotes
            break
            
        if ch == '"':
            # Check for escaped quote ""
            if i + 1 < len(line) and line[i + 1] == '"':
                result.append('""')
                i += 2
                continue
            else:
                in_quotes = not in_quotes
                result.append('"')
        else:
            result.append(ch)
            
        i += 1
    
    return ''.join(result).rstrip()
```

### A.4 Streaming Parser Architecture

For large files, implement streaming:

```python
class StreamingHEDLParser:
    def __init__(self):
        self.state = 'HEADER'
        self.stack = []
        self.registries = {}
        
    def feed(self, line):
        """Process a single line."""
        if self.state == 'HEADER':
            if line.strip() == '---':
                self.state = 'BODY'
            else:
                self.parse_header_line(line)
        else:
            self.parse_body_line(line)
    
    def parse_body_line(self, line):
        # Similar to parse_body but stateful
        indent = self.calculate_indent(line)
        self.pop_frames(indent)
        
        if line.lstrip().startswith('|'):
            self.parse_matrix_row(line, indent)
        else:
            self.parse_object_line(line, indent)
    
    def get_result(self):
        """Get parsed result after all lines processed."""
        return self.stack[0].object
```

---

## Appendix B: Conformance Test Suite

Implementations MUST pass these minimal tests:

### B.1 Syntax Validation
1. **Odd Indentation**: Line with 3 leading spaces → Syntax Error
2. **Tab Character**: Tab character for indentation → Syntax Error
3. **Missing Separator**: No `---` line → Syntax Error
4. **Multiple Separators**: More than one `---` → Syntax Error
5. **Body missing space after colon**: `a:1` → Syntax Error (Section 4.7)
6. **Invalid Reference uppercase**: `@User1` → Syntax Error (Reference Token requires lowercase/underscore ID start)
7. **Control Characters**: ASCII control char (except LF, CR, TAB in quotes) → Syntax Error
8. **Bare CR**: `\r` without `\n` → Syntax Error

### B.2 Schema Validation
8. **Unknown Type**: `@UnknownType` without inline schema → Schema Error
9. **Schema Mismatch**: Inline schema differs from %STRUCT → Schema Error
10. **Duplicate Struct**: Same TypeName with different columns → Schema Error
11. **Nest to undefined**: `%NEST: A > B` where B undefined → Schema Error

### B.3 Data Validation
12. **Shape Mismatch**: Wrong cell count in matrix row → Shape Error
13. **First Row Ditto**: `^` in first row of list → Semantic Error
14. **Orphan Child Row**: Child row without %NEST → Orphan Row Error
15. **Duplicate ID within type**: Same ID in same type → Collision Error
16. **Different ID across types**: Same ID in different types → Success
17. **Invalid ID Type**: Number as ID value → Semantic Error
18. **Invalid ID format uppercase**: In ID column `User1` → SemanticError (fails ID Token)
19. **Valid ID with dash**: `config-file` as ID → Success
20. **Ditto in ID column**: `^` in first column → Semantic Error with clear message
21. **Null in ID column**: `~` in first column → Semantic Error with clear message

### B.4 Reference Validation
22. **Forward Reference**: Reference to later-defined node in same type → Success (in strict mode)
23. **Missing Reference**: Unresolved `@missing` → Reference Error (strict) or null (lenient)
24. **Self Reference**: `@self` where self exists → Success
25. **Circular Reference**: A references B, B references A → Success (allowed)

### B.5 Parsing Correctness
26. **Ditto Scoping**: `^` doesn't copy from different list
27. **Child Attachment**: Child rows attach to correct parent
28. **Alias Expansion**: `%key` expands and infers correctly
29. **Comment Preservation**: `#` in quoted CSV field is data
30. **Matrix Row Comment**: Comment stripped before CSV parse
31. **Quoted String Escaping**: `""` in quoted field → single `"`
32. **Number Inference**: `42` → integer, `42.0` → float, `42.5` → float
33. **Tensor Literal**: `[1, 2, 3]` → array, `[[1,2],[3,4]]` → nested array
34. **@ and $ in strings**: `alice@example.com` → string, not reference

### B.6 Edge Cases and Truncation Detection
35. **Only Header + Separator**: `%VERSION: 1.0\n---\n` → Success (empty root object)
36. **Maximum Nesting**: 50 levels deep → Success (or configured limit)
37. **Empty Matrix**: List with no rows → Success (empty list)
38. **Object Start with Comment**: `key: # comment` → Object Start (comment stripped)
39. **Empty Alias**: `%ALIAS: %empty: ""` → Success, expands to empty string
40. **Whitespace Preservation**: `key: "  spaces  "` → preserves spaces
41. **Boolean Case**: `True` → string "True", not boolean
42. **Expression with parens**: `$((a + b))` → Expression("(a + b)")
43. **Unclosed Quote**: `key: "unclosed` → Syntax Error
44. **Truncated Separator**: File ends with `--` → Syntax Error
45. **Unclosed Structure**: File ends inside object → Syntax Error
46. **UTF-8 Invalid**: Invalid UTF-8 byte sequence → Syntax Error
47. **Tab in quoted string**: `key: "a\tb"` → Success (tab allowed in quotes)

### B.7 Test Document

```hedl
# conformance.hedl
%VERSION: 1.0
%ALIAS: %true: "true"
%STRUCT: Test: [id,value,ref]
%STRUCT: Child: [id,data]
%NEST: Test > Child
---
tests: @Test
  |t1,"simple",~
    |c1,child
  |t2,42,@t1
    |c2,child
  |t3,%true,@t2
  |t4,^,^
tensor_test: @TensorTest[id,data]
  |t5,[1,2,3]
  |t6,[[1,2],[3,4]]
```

Expected result includes:
- t1 with child c1
- t2 with child c2 (data = "child", explicit value since ditto not allowed in first row of child list)
- t3 value = true (via alias expansion)
- t4 value = true (ditto), ref = @t2 (ditto)
- t5 data = array [1, 2, 3]
- t6 data = nested array [[1, 2], [3, 4]]
- All references resolved within type namespaces

---

## Appendix C: Migration Guide

### C.1 From JSON

**Pattern**:
```json
{
  "users": [
    {"id": "u1", "name": "Alice", "posts": [
      {"id": "p1", "text": "Hello"}
    ]}
  ]
}
```

**HEDL Equivalent (Simple Mode)**:
```hedl
%VERSION: 1.0
---
# In simple mode without schemas, use Maps/Objects for hierarchy
users:
  u1:
    name: Alice
    posts:
      p1:
        text: Hello
```

**HEDL Equivalent (Complex Mode)**:
```hedl
%VERSION: 1.0
%STRUCT: User: [id,name]
%STRUCT: Post: [id,text]
%NEST: User > Post
---
users: @User
  |u1,Alice
    |p1,Hello
```

**Conversion Rules**:
1. Objects with arrays of similar objects → Matrix lists
2. Nested arrays → %NEST directives
3. String IDs → ensure lowercase/underscore start
4. Mixed types in arrays → separate by type or use most general schema

### C.2 From YAML

**Pattern**:
```yaml
config:
  database:
    host: localhost
    port: 5432
  servers:
    - name: web1
      ip: 192.168.1.1
    - name: web2
      ip: 192.168.1.2
```

**HEDL Equivalent (Simple Mode)**:
```hedl
%VERSION: 1.0
---
config:
  database:
    host: localhost
    port: 5432
  servers: @Server[name,ip]
    |web1,192.168.1.1
    |web2,192.168.1.2
```

**HEDL Equivalent (Complex Mode)**:
```hedl
%VERSION: 1.0
%STRUCT: Server: [name,ip]
---
config:
  database:
    host: localhost
    port: 5432
  servers: @Server
    |web1,192.168.1.1
    |web2,192.168.1.2
```

**Conversion Rules**:
1. YAML objects → HEDL objects
2. YAML lists of objects → HEDL matrix lists
3. YAML anchors (&, *) → HEDL references (@id)
4. YAML multi-line strings → HEDL quoted strings (no multi-line in v1.0)

### C.3 From CSV

**Pattern** (CSV + relationships):
```csv
user_id,user_name,post_id,post_text
u1,Alice,p1,Hello
u1,Alice,p2,World
u2,Bob,p3,Hi
```

**HEDL Equivalent**:
```hedl
%VERSION: 1.0
%STRUCT: User: [id,name]
%STRUCT: Post: [id,text,author_id]
---
users: @User
  |u1,Alice
  |bob,Bob  # Note: ID must start with lowercase
posts: @Post
  |p1,Hello,@u1
  |p2,World,@u1
  |p3,Hi,@bob
```

**Conversion Rules**:
1. CSV header row → %STRUCT definition
2. CSV data rows → matrix rows
3. Repeated data → separate tables with references
4. Hierarchical data → use %NEST

### C.4 Migration Tools

Implementations SHOULD provide:
1. JSON → HEDL converter
2. HEDL → JSON converter (for compatibility)
3. Schema inference from JSON/CSV
4. ID generation for data without IDs

---

## Appendix D: Performance Guidelines

### D.1 Parser Optimization

**Fast Paths**:
1. **Indent calculation**: Use bit shift for `/ 2`
2. **Line classification**: Early checks for `|` and `:`
3. **CSV parsing**: Optimize for common case (no quotes, no escapes)
4. **Number parsing**: Use native parser with validation
5. **Tensor parsing**: Validate bracket balance without full parsing if possible

**Memory Optimization**:
1. **String interning**: For common values (true, false, null)
2. **Schema sharing**: Single schema instance per TypeName
3. **Reference resolution**: Lazy resolution if possible
4. **Streaming**: Don't keep entire document in memory

**Example Optimized Parser**:
```python
class OptimizedHEDLParser:
    def parse_line_fast(self, line):
        # Fast path for common cases
        if line.startswith('  '):  # Common indent
            indent = len(line) - len(line.lstrip(' '))
            if indent & 1:  # Check odd (bitwise AND)
                raise SyntaxError("Odd indentation")
            indent_level = indent >> 1  # Divide by 2
            
            content = line[indent:]
            if content.startswith('|'):
                return self.parse_matrix_row_fast(content, indent_level)
            # ... other cases
```

### D.2 Generator Optimization

**Canonicalization**:
1. **Ditto detection**: Compare with previous row
2. **Quoting decision**: Fast check for special characters
3. **Sorting**: Use stable sort for object keys
4. **Buffer reuse**: For string building

**Memory Efficient Generation**:
```python
def generate_canonical(data, output):
    """Stream canonical HEDL to output."""
    if isinstance(data, dict):
        for key in sorted(data.keys()):
            value = data[key]
            if isinstance(value, MatrixList):
                generate_matrix_list(key, value, output)
            else:
                generate_scalar(key, value, output)
```

### D.3 Large File Handling

**Streaming Parser**:
- Process line by line
- Yield nodes as parsed
- Don't build full tree in memory

**Example**:
```python
def parse_stream(fileobj):
    """Parse HEDL file as stream of events."""
    parser = StreamingHEDLParser()
    for line in fileobj:
        parser.feed(line)
        for event in parser.get_events():
            yield event
```

**Memory Mapped Files**:
- Use mmap for large files
- Avoid copying data
- Parse in chunks

### D.4 Benchmark Suite

Implementations SHOULD include benchmarks for:
1. **Parsing speed**: Documents/sec
2. **Memory usage**: Peak memory
3. **Canonicalization**: Round-trip time
4. **Large files**: Streaming performance

**Example Benchmark**:
```python
def benchmark_parser():
    # Parse 1000-node document
    start = time.time()
    result = parser.parse(large_document)
    elapsed = time.time() - start
    
    print(f"Parsed {len(result)} nodes in {elapsed:.3f}s")
    print(f"Rate: {len(result)/elapsed:.0f} nodes/sec")
```

### D.5 Performance Targets

For typical implementations:
- **Parsing**: ≥ 10,000 nodes/second
- **Memory**: ≤ 2x document size
- **Canonicalization**: ≤ 1.5x parse time
- **Startup**: < 10ms for empty document

---

## Appendix E: Formal Grammar

### E.1 Context-Free Grammar

```
Document        ::= Header Separator Body
Header          ::= Directive*
Directive       ::= VersionDirective | StructDirective | NestDirective | AliasDirective
Separator       ::= '---' Newline

VersionDirective ::= '%VERSION:' WS+ Version Newline
Version         ::= Digit+ '.' Digit+
StructDirective ::= '%STRUCT:' WS+ TypeName ':' WS+ ColumnList Newline
NestDirective   ::= '%NEST:' WS+ TypeName WS+ '>' WS+ TypeName Newline
AliasDirective  ::= '%ALIAS:' WS+ AliasKey ':' WS+ QuotedString Newline

ColumnList      ::= '[' Column (',' Column)* ']'
Column          ::= KeyToken

Body            ::= (Object | KeyValue | MatrixList)*
Object          ::= Indent KeyToken ':' Newline (Object | KeyValue | MatrixList)*
KeyValue        ::= Indent KeyToken ':' WS+ Value Newline
              | Indent KeyToken ':' WS+ BlockString
MatrixList      ::= Indent KeyToken ':' WS+ '@' TypeName ColumnList? Newline MatrixRow*
MatrixRow       ::= Indent '|' CountHint? CSVRow Newline
CountHint       ::= '[' Digit+ ']' WS*  # Optional count of direct children

Value           ::= Null | Tensor | Reference | Expression | AliasRef | Boolean | Number | String | QuotedString
Null            ::= '~'
Tensor          ::= '[' (Number | Tensor) (',' (Number | Tensor))* ']'
Reference       ::= '@' (TypeName ':')? IDToken
Expression      ::= '$(' BalancedText ')'
AliasRef        ::= '%' KeyToken
Boolean         ::= 'true' | 'false'
Number          ::= '-'? Digit+ ('.' Digit+)?
String          ::= [^:#@$%~[\s][^:#@$\s]*  # Simplified - not starting with special chars
QuotedString    ::= '"' (Char | '""')* '"'   # No escape sequences in key-value context
BlockString     ::= '"""' Newline BlockContent '"""'
BlockContent    ::= (Char | Newline)*        # Raw content, no escape processing

CSVRow          ::= CSVField (',' CSVField)*
CSVField        ::= QuotedCSVField | UnquotedCSVField
QuotedCSVField  ::= '"' (Char | '""' | EscapeSeq)* '"'
UnquotedCSVField ::= [^,#\n\r]*  # Cannot contain comma, hash, newline, CR

# Escape sequences (only valid in quoted CSV fields)
EscapeSeq       ::= '\n' | '\t' | '\r' | '\\' | '\"'

Indent          ::= Space Space*  # Multiple of 2 spaces

# Tokens
TypeName        ::= [A-Z][A-Za-z0-9]*
KeyToken        ::= [a-z_][a-z0-9_]*
IDToken         ::= [a-z_][a-z0-9_\-]*  # ASCII-only in v1.0
AliasKey        ::= '%' KeyToken

# Character classes
Digit           ::= [0-9]
Space           ::= ' '
Newline         ::= '\n'
Char            ::= Any Unicode character except control characters (0x00-0x1F, 0x7F)
WS              ::= Space
```

### E.2 Lexical Notes

1. **Comments**: Not part of grammar; stripped before parsing
2. **Whitespace**: Significant only as indentation; otherwise ignored
3. **Line continuations**: Not supported in v1.0
4. **Unicode**: Structural tokens are strictly ASCII-only. Data values (strings, comments, tensor numbers) allow Unicode.
5. **BalancedText**: Defined by balanced-parentheses algorithm in Section 4.6.6
6. **Block Strings**: Triple-quoted strings (`"""`) for multiline key-value content. No escape processing; all content is literal. See Section 8.1.2.
7. **Escape Sequences**: Only processed in quoted CSV fields (matrix cells). Key-value strings treat backslash literally. See Section 9.2.
8. **Count Hints**: Optional metadata in matrix rows indicating number of direct children. Format: `[N]` where N is a non-negative integer. See Section 9.6.

### E.3 Grammar Validation

This grammar is:
- **LL(1)**: Can be parsed with one-token lookahead
- **Deterministic**: No ambiguous constructs
- **Complete**: Covers all valid HEDL 1.0 documents
- **Unambiguous**: Each valid document has one parse tree

### E.4 Grammar Implementation

Example recursive descent parser skeleton:

```python
class GrammarParser:
    def parse_document(self):
        self.parse_header()
        self.expect('---')
        self.parse_body()
    
    def parse_header(self):
        while not self.peek() == '---':
            if self.peek().startswith('%VERSION'):
                self.parse_version()
            elif self.peek().startswith('%STRUCT'):
                self.parse_struct()
            # ... other directives
    
    def parse_body(self):
        while not self.eof():
            indent = self.parse_indent()
            if self.peek().endswith(':'):
                key = self.parse_key()
                self.expect(':')
                if self.peek().startswith('@'):
                    self.parse_matrix_list(key, indent)
                elif self.peek().isspace():  # Next line has content
                    self.parse_key_value(key, indent)
                else:  # Object start
                    self.parse_object(key, indent)
            elif self.peek().startswith('|'):
                self.parse_matrix_row(indent)
```

---

## Appendix F: Frequently Asked Questions

### F.1 Why type-scoped IDs instead of global?

**Answer**: Type-scoped IDs enable:
- Modular data composition (safe file concatenation)
- Natural naming (`user:admin`, `role:admin`)
- Simpler reference resolution within same type
- **Cross-type references** via qualified syntax (`@Type:id`)

### F.2 Why ASCII-only IDs in v1.0.0?

**Answer**: ASCII-only ensures:
- Consistent reference resolution across platforms
- Simpler implementation for v1.0
- Clear migration path to Unicode in future versions
- Interoperability with existing systems

### F.3 Why no optional columns?

**Answer**: Simplicity and performance. Fixed schemas enable:
- Faster parsing (no per-row column count checks)
- Clearer data shape
- Better compression via ditto
- Simpler tooling

Workaround: Use `~` (null) for optional values.

### F.4 Why case-sensitive IDs?

**Answer**: Predictability and simplicity. Case-insensitive matching causes:
- Ambiguity (`User` vs `user`)
- Locale issues (Turkish dotted i)
- Implementation complexity
- Surprising behavior

### F.5 Why no multi-line strings?

**Answer**: Token efficiency and parsing simplicity. Multi-line strings:
- Increase token count significantly
- Complicate line-based parsing
- Rarely needed in target use cases (AI/ML data)

Workaround: Use `\n` in string and parse post-hoc.

### F.6 Why strict 2-space indentation?

**Answer**: Consistency and error detection. Allowing mixed indentation:
- Causes subtle bugs
- Makes parsing ambiguous
- Reduces scanability
- Complicates tooling

### F.7 Can I use HEDL without schemas?

**Answer**: Yes! HEDL supports **simple mode** for key-value and nested object data:
```hedl
%VERSION: 1.0
---
config:
  host: localhost
  port: 8080
admin:
  name: Alice
  email: alice@example.com
```

For lists, you can use inline schemas without `%STRUCT` directives:
```hedl
%VERSION: 1.0
---
users: @User[id,name,email]
  |alice,Alice,alice@example.com
  |bob,Bob,bob@example.com
```

Add `%STRUCT` directives when you need to reference schemas multiple times or use `%NEST`.

### F.8 How to handle large binary data?

**Answer**: HEDL is for structured data, not binary blobs. Options:
1. Store paths/URLs in HEDL, data externally
2. Encode as base64 in strings (not efficient in v1.0)
3. Use companion binary format with HEDL metadata

### F.9 Is there a binary version?

**Answer**: Not in v1.0. HEDL prioritizes:
- Token efficiency for LLMs
- Human scanability
- Diff-friendliness

Binary encoding may be considered in future versions.

### F.10 How does truncation detection work?

**Answer**: Parsers check for:
1. Unclosed structures (objects, lists) at EOF
2. Unterminated tokens (quotes, expressions)
3. Partial separator (`--` instead of `---`)
4. Incomplete directives in header
5. Bare CR line endings

This ensures truncated files are rejected rather than partially parsed.

---

## Appendix G: Complete Implementation Reference (Informational)

**Note**: This appendix is non-normative and provided for informational purposes only.

### G.1 Reference Implementation

A complete reference implementation is available at:
- **GitHub**: `https://github.com/dweve-ai/hedl-format`
- **Language**: Python 3.9+
- **License**: MIT

### G.2 Test Suite

Comprehensive test suite includes:
- 500+ unit tests
- Fuzz testing corpus
- Performance benchmarks
- Compliance verification

### G.3 Language Bindings

Official language bindings (planned):
- **Python**: `pip install hedl`
- **JavaScript/TypeScript**: `npm install hedl`
- **Rust**: `cargo add hedl`
- **Go**: `go get github.com/dweve-ai/go-hedl`

### G.4 Tooling Ecosystem

Recommended tools:
- **HEDL Linter**: Static analysis and validation
- **HEDL Formatter**: Canonical formatting
- **HEDL Visualizer**: Graph visualization
- **HEDL Converter**: JSON/YAML/CSV conversion
- **HEDL IDE Plugin**: Syntax highlighting, validation

### G.5 Specification Compliance

To verify compliance:
1. Run the official test suite
2. Validate against reference implementation
3. Check error messages match specification
4. Verify truncation detection works
5. Test type-scoped ID handling

### G.6 Contributing

Contributions welcome:
- Report issues on GitHub
- Submit pull requests
- Join specification discussions
- Create language bindings

---

## Appendix H: Format Comparisons (Informational)

**Note**: This appendix is non-normative and provided to help users choose the right format for their use case.

### H.1 HEDL vs Other Formats

| Feature | HEDL | TOON | JSON | YAML | CSV | Protobuf |
|---------|------|------|------|------|-----|----------|
| Human Readable | ✓ Yes | ✓ Yes | ✓ Yes | ✓ Yes | ○ Limited | ✗ No |
| Token Efficient | ★ Excellent | ★ Excellent | ✗ Poor | ○ Fair | ○ Good | N/A (binary) |
| LLM Accuracy | ★ 63-71% (near-JSON) | 61-71% | 68-73% (baseline) | 68-73% | 27-31% | N/A |
| Graph Support | ★ Native | ✗ No | ○ Manual | ○ Manual | ✗ No | ✗ No |
| Schema Support | ★ Built-in | ○ Inline | ○ External | ○ External | ○ Header | ★ Required |
| Streaming | ✓ Yes | ✓ Yes | ✓ Yes | ○ Limited | ✓ Yes | ✓ Yes |
| Ditto Markers | ★ Native (`^`) | ✗ No | ✗ No | ✗ No | ✗ No | ✗ No |
| References | ★ `@id` syntax | ✗ No | ✗ Manual | ✗ Manual | ✗ No | ✗ No |

### H.2 HEDL's Unique Features

HEDL provides features that other token-efficient formats (like TOON) lack. These features cover the majority of real-world use cases.

**HEDL Advantages** (benchmarked with GPT-4 tokenizer):

| Comparison | HEDL |
|------------|------|
| vs JSON | **47-74% token savings** |
| Graph references | `@id` syntax saves **51.5%** vs duplicating entities |
| Repeated values | Ditto markers (`^`) eliminate redundancy |
| Schema reuse | `%STRUCT` definitions shared across files |

Some tabular-only formats (like TOON) are ~8% more efficient on pure flat data without relationships, but lack the features above.

**LLM Accuracy**: HEDL achieves 62.7-71.2% accuracy across providers (within 1.7-5.1pp of JSON, matching or beating TOON). Tested on 59 questions across 13 datasets with DeepSeek, OpenAI, and Mistral. Accuracy is determined by data structure, not syntax.

**Feature Comparison**:

| Aspect | HEDL | Tabular-Only Formats |
|--------|------|----------------------|
| Data Model | Extended (graph semantics) | Tree-based |
| References | `@id` native syntax (**51% savings**) | Not supported |
| Ditto Markers | `^` for repetition | Not supported |
| Global Aliases | `%ALIAS` directive | Not supported |
| Schema Declaration | `@Type[...]` inline or `%STRUCT` | Inline only |
| Array Length | Implicit (auto-counted) | Often required |

**HEDL is the right choice for**:
- Graph relationships (`@author` references save 51% tokens)
- Repeated adjacent values (ditto markers `^`)
- Reusable schemas across files (`%STRUCT` definitions)
- No manual array length counting needed
- Most real-world datasets with references and repetition

### H.3 HEDL vs JSON

**Token Savings**: HEDL typically achieves ~50% token reduction compared to JSON.

| Dataset Type | HEDL vs JSON Savings |
|--------------|---------------------|
| Flat lists (users, events) | 47-52% |
| Nested hierarchies (org charts) | 66-74% |
| Cross-references (knowledge graphs) | 53% |
| Time-series (metrics) | 53-60% |

**LLM Accuracy**: 62.7-71.2% across providers (within 1.7-5.1pp of JSON) when using proper schema field ordering (positions 1-4 for frequently queried fields). Tested on 59 questions across 13 datasets with DeepSeek, OpenAI, and Mistral.

### H.4 Choosing the Right Format

**Use HEDL when**:
- Maximum token efficiency for LLM context windows
- Graph semantics with references between entities
- Schema-defined structured data with repetition
- Bidirectional conversion with JSON/YAML needed
- Most real-world datasets with relationships and repetition

**Use JSON when**:
- Maximum compatibility required
- Ad-hoc unstructured data
- Human editing is primary use case

**Use CSV when**:
- Simple flat tables only
- Spreadsheet compatibility needed
- No nested structures

---

**End of HEDL Specification v1.0.0**