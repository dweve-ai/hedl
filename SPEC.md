# HEDL Specification v2.0.0

Hierarchical Entity Data Language

**Status:** Released
**Version:** 2.0.0
**MIME Type:** `application/hedl`
**File Extension:** `.hedl`
**Release Date:** 2026-01-25

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
* **Strict indentation as structure**: Eliminates brackets and explicit delimiters through consistent 1-space indentation (v2.0)
* **Document-wide identity system**: Global IDs enable graph relationships without duplication
* **Implicit child lists**: Automatic parent-child attachment via nesting rules without explicit container declaration
* **Scoped ditto operator** (pre-v2.0 only, removed in v2.0): Repeats previous values within bounded contexts, reducing redundancy
* **Alias system**: Global constants for token substitution and schema sharing
* **Simple and complex modes**: Progressive disclosure from basic key-value pairs to full schematized graphs
* **Tensor literals**: Built-in support for numerical arrays in AI/ML workflows
* **Compact directive syntax**: Shortened directive names for minimal token overhead

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

* **Indent Level**: `LeadingSpaces` (1 space = 1 level, Section 4.3)
* **Schema Registry**: Map of `TypeName → ordered Columns[]` defined by `%S` (or `%STRUCT`) directives
* **Matrix List**: A keyed list of typed rows, initiated by `key:@TypeName` or `key:@TypeName[...]`
* **Matrix Row**: A `|`-prefixed CSV record parsed according to its enclosing list's schema
* **Context Stack**: Stack of active scopes controlling what node types are allowed
* **List Frame**: Stack frame representing an active matrix list, tracking schema and row state
* **Object Frame**: Stack frame representing an object mapping scope
* **Row Scope**: The most recently parsed row in a list frame, serving as attachment point for child lists
* **Node Registry**: Global mapping of `ID → Node` populated during parsing
* **Alias Registry**: Global mapping of `%key → string` defined by `%ALIAS` directives
* **Truncation State**: Tracks whether the document ends in the middle of a structure
* **List Literal**: Parenthesized sequence of scalars `(elem, ...)` (v1.1+, Section 4.6.9)
* **Count Registry**: Map of `Type.field → counts` from `%C` directives (v1.2, Section 6.10)
* **Parse Mode**: `strict` (default) or `lenient`, from `%MODE` directive (v1.1+, Section 6.6)
* **Null Symbol**: Character representing null, from `%NULL` directive (default `~`)
* **Quote Symbol**: Character for quoting strings, from `%QUOTE` directive (default `"`)

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
   - Achieved through: implicit structure, positional encoding, optional schemas

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
* **ID Tokens**: ID tokens MUST be ASCII-only for v1.0/v1.1 to ensure consistent reference resolution across platforms. Future versions may support Unicode IDs.
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
3. **Step Size**: Exactly 1 space per indent level (v2.0)
4. **Maximum Depth**: Parsers SHOULD enforce a maximum indent depth (default 50)
5. **Zero Indent**: The first non-header, non-blank line MUST have indent level 0
6. **Whitespace Definition**: Throughout this specification, "whitespace" refers to ASCII space (`U+0020`) only, unless explicitly stated otherwise. Unicode whitespace characters (e.g., NBSP, zero-width spaces) are NOT treated as whitespace for parsing purposes and SHOULD cause warnings or errors if found in structural positions.

**Definition**: For a line with `LeadingSpaces` (count of leading spaces after normalization):
```
IndentLevel = LeadingSpaces  (1 space = 1 level)
```

**Indentation Examples**:
```hedl
level0:      # IndentLevel = 0
 level1:     # IndentLevel = 1 (1 space)
  level2:    # IndentLevel = 2 (2 spaces)
 level1_2:   # IndentLevel = 1 (back to 1 space)
```

**Syntax Error Examples**:
```hedl
level0:
	level1:    # ERROR: tab character not allowed
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

```rust
/// Type of protected region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    Quote,
    Expression,
}

/// A protected region in a line.
#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub start: usize,  // Start byte offset
    pub end: usize,    // End byte offset (exclusive)
    pub region_type: RegionType,
}

/// Strip inline comment from a line, respecting protected regions.
/// Returns the line with comment removed (trimmed).
pub fn strip_comment(line: &str) -> &str {
    let bytes = line.as_bytes();

    // Find first # character
    let hash_pos = match bytes.iter().position(|&b| b == b'#') {
        Some(pos) => pos,
        None => return line.trim_end(),
    };

    // Check if # appears before any protected region starts
    let has_quote_before = bytes.iter().take(hash_pos).any(|&b| b == b'"');
    let has_expr_before = bytes.windows(2).take(hash_pos).any(|w| w == b"$(");

    if !has_quote_before && !has_expr_before {
        // No protected regions before #, safe to strip
        return line[..hash_pos].trim_end();
    }

    // Scan regions to find unprotected #
    let regions = scan_regions(line);

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'#' {
            let in_region = regions.iter().any(|r| r.start <= i && i < r.end);
            if !in_region {
                return line[..i].trim_end();
            }
        }
    }

    line.trim_end()
}
```

**Normative Algorithm: `scan_regions(line)`**

This algorithm scans a line and identifies regions of quoted strings and expressions. It returns a list of `Region` structs indicating where special characters (like `#` or `,`) lose their usual meaning.

```rust
/// Scan a line for protected regions (quoted strings and expressions).
pub fn scan_regions(line: &str) -> Vec<Region> {
    let mut regions = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'"' {
            // Start of quoted string
            let start = i;
            i += 1;

            while i < bytes.len() {
                if bytes[i] == b'"' {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                        // Escaped quote ""
                        i += 2;
                    } else {
                        // End of quoted string
                        regions.push(Region {
                            start,
                            end: i + 1,
                            region_type: RegionType::Quote,
                        });
                        i += 1;
                        break;
                    }
                } else {
                    i += 1;
                }
            }

            // Unclosed quote extends to end of line
            if i >= bytes.len() && (regions.is_empty() || regions.last().unwrap().start != start) {
                regions.push(Region {
                    start,
                    end: bytes.len(),
                    region_type: RegionType::Quote,
                });
            }
        } else if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'(' {
            // Start of expression
            let start = i;
            i += 2;
            let mut depth = 1;
            let mut in_expr_quotes = false;

            while i < bytes.len() && depth > 0 {
                let b = bytes[i];

                if b == b'"' {
                    if in_expr_quotes {
                        if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                            i += 2;
                            continue;
                        } else {
                            in_expr_quotes = false;
                        }
                    } else {
                        in_expr_quotes = true;
                    }
                }

                if !in_expr_quotes {
                    if b == b'(' {
                        depth += 1;
                    } else if b == b')' {
                        depth -= 1;
                    }
                }

                i += 1;
            }

            regions.push(Region {
                start,
                end: i,
                region_type: RegionType::Expression,
            });
        } else {
            i += 1;
        }
    }

    regions
}
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
- **Note**: ASCII-only for v1.0/v1.1 to ensure consistent reference resolution. Future versions may support Unicode.

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

#### 4.6.7 Ditto Token (v1.2 only)

> **Note**: The ditto operator (`^`) is NOT allowed in v2.0. Every cell must have an explicit value. This section documents v1.2 behavior for reference.

**Pattern**: `^` (single caret)
- **Used for**: copying value from same column of previous row (v1.2 only)
- **Context**: Only valid in matrix cells, not in Key-Value pairs
- **Invalid contexts**: ID column, first row of list, Key-Value values, **all v2.0 documents**

#### 4.6.8 Tensor Literal

**Pattern**: Starts with `[`, contains balanced brackets with numeric values
- **Used for**: multi-dimensional numerical arrays
- **Format**: `[1, 2, 3]` or `[[1, 2], [3, 4]]`
- **Rules**: Must contain only numbers, commas, spaces, and balanced brackets
- **Examples**: `[1, 2, 3]`, `[[1.5, 2.0], [3.1, 4.2]]`
- **Invalid**: `[1, "text"]` (mixed types), `[1, 2` (unbalanced)

#### 4.6.9 List Literal (v1.1)

**Pattern**: Starts with `(`, contains balanced parentheses with scalar values
- **Used for**: ordered sequences of scalar values (distinct from numeric tensors)
- **Format**: `(elem1, elem2, ...)` or `()`
- **Delimiters**: `(` and `)` (distinct from tensor brackets `[` `]`)
- **Rules**:
  - Empty list: `()` is always valid
  - Elements are parsed using the existing scalar inference ladder
  - Elements are separated by commas
  - Lists MAY be homogeneous or heterogeneous
  - Lists can contain any scalar type: strings, numbers, booleans, null, references, expressions
- **Distinction from Tensors**:
  - `[...]` = numeric tensor (numbers only)
  - `(...)` = list of scalars (any scalar type)
- **Examples**:
  - `(admin, editor, viewer)` - list of strings
  - `(true, false, true)` - list of booleans
  - `(1, "two", ~, @ref1)` - heterogeneous list
  - `()` - empty list
- **Invalid**: `(1, 2` (unbalanced), `((nested))` (nested lists not supported)

**List Literal Parsing Algorithm**:

```rust
fn parse_list_literal(value_str: &str) -> Vec<ScalarValue> {
    /// Parse a list literal from a string.
    /// Precondition: value_str starts with '(' and ends with ')'.
    /// Returns: list of parsed scalar values.
    if value_str == "()" {
        return Vec::new();
    }

    // Remove outer parentheses
    let inner = value_str[1..value_str.len()-1].trim();
    if inner.is_empty() {
        return Vec::new();
    }

    // Split by comma, respecting quotes and nested expressions
    let mut elements = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_quotes = false;
    let mut in_expr = false;
    let mut expr_depth = 0;

    let chars: Vec<char> = inner.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let char = chars[i];

        if char == '"' && !in_expr {
            in_quotes = !in_quotes;
            current.push(char);
        } else if char == '$' && i + 1 < chars.len() && chars[i + 1] == '(' && !in_quotes {
            in_expr = true;
            expr_depth = 1;
            current.push_str("$(");
            i += 1;
        } else if in_expr {
            current.push(char);
            if char == '(' && !in_quotes {
                expr_depth += 1;
            } else if char == ')' && !in_quotes {
                expr_depth -= 1;
                if expr_depth == 0 {
                    in_expr = false;
                }
            }
        } else if char == ',' && !in_quotes && depth == 0 {
            // End of element
            let elem_str = current.trim();
            elements.push(infer_scalar_value(elem_str));
            current = String::new();
        } else {
            current.push(char);
        }

        i += 1;
    }

    // Don't forget the last element
    if !current.is_empty() {
        let elem_str = current.trim();
        elements.push(infer_scalar_value(elem_str));
    }

    elements
}
```

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
key:@Type    # Matrix List Start

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
ref:@user1                 # Valid - reference (starts with @)
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
- **Optional**: Header may contain only `%V:2.0` and separator for simple documents

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
%V:2.0
---  # Valid with trailing spaces
```

```hedl
%V:2.0
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
* Minimum valid document: `%V:2.0\n---\n`
* A document with only header and separator but no body is valid (empty root object)

**Examples**:
```hedl
# Invalid - no version
---
```

```hedl
# Invalid - no separator
%V:2.0
```

```hedl
# Valid minimal document
%V:2.0
---
```

---

## 6. Header Section

The Header configures parsing state through directives. All directives start with `%` and use `:` to separate name from payload.

### 6.1 Directive Format

HEDL v2.0 supports two directive formats: **compact** (preferred) and **verbose** (legacy):

**Compact Format (v1.2+)**:
```
%D:payload
```

**Verbose Format (v1.0/v1.1)**:
```
%DIRECTIVE: payload
```

* `D` or `DIRECTIVE` is case-sensitive ASCII uppercase
* In compact format, `:` is immediately followed by payload (no space)
* In verbose format, `:` MUST be followed by at least one space
* `payload` format depends on the directive
* **Order**: Directives MUST appear in dependency order:
  - `%V` (or `%VERSION`) MUST be first (REQUIRED)
  - `%S` (or `%STRUCT`) definitions MUST appear before they are referenced by `%N` (or `%NEST`)
* **Comment handling**: Inline comments allowed after payload, stripped before parsing payload
* **Spacing**: In verbose format, implementations MUST accept one or more spaces after `:`

**Directive Name Mapping (Compact ↔ Verbose)**:

 |Compact | Verbose | Description |
|---------|---------|-------------|
 |`%V` | `%VERSION` | Version declaration |
 |`%S` | `%STRUCT` | Schema definition |
 |`%N` | `%NEST` | Nesting relationship |
 |`%C` | `%COUNT` | Count/statistics (v1.2+) |
 |`%NULL` | `%NULL` | Null symbol (REQUIRED in v2.0) |
 |`%QUOTE` | `%QUOTE` | Quote symbol (REQUIRED in v2.0) |

### 6.2 `%V` / `%VERSION` Directive (REQUIRED)

Declares the HEDL specification version.

**Syntax**:
- Compact (v1.2+): `%V:major.minor`
- Verbose (v1.0/v1.1): `%VERSION: major.minor`

**Parameters**:
- `major`: Non-negative integer
- `minor`: Non-negative integer
- Both separated by exactly one `.`
- No leading zeros (except `0` itself)

**Examples**:
```hedl
%V:2.0           # Compact (preferred for v1.2+)
%VERSION: 2.0    # Verbose (v1.0/v1.1 style)
```

**Invalid Examples**:
```hedl
%V:1             # Missing minor
%V:2.0.0  # Too many parts
%V:01.0          # Leading zero
%VERSION: a.b    # Non-numeric
```

**Parser Behavior**:
1. Parse major.minor as integers
2. If parsing fails → `VersionError`
3. If file `major > parser.major`: raise `VersionError` (incompatible)
4. If file `major < parser.major`: MAY accept (backward compatibility)
5. If `major` matches but `minor > parser.minor`: MAY accept if new features can be safely ignored
6. Otherwise: proceed normally

**Note**: This specification is version `2.0`.

### 6.3 `%S` / `%STRUCT` Directive (Optional)

Defines a named schema for typed matrix lists.

**Syntax**:
- Compact (v1.2+): `%S:TypeName:[col1,col2,...]`
- Verbose (v1.0/v1.1): `%S:TypeName:[col1, col2, ...]`

**Requirements**:
- `TypeName` MUST be a TypeName Token
- Column names MUST be Key Tokens and unique within the struct
- At least one column REQUIRED
- Maximum columns: implementation-defined (RECOMMENDED ≥ 100)
- First column is the ID column (Section 10.1)
- Column order defines CSV parsing order

**Examples**:
```hedl
# Compact (v2.0 preferred)
%S:User:[id,name,email]
%S:Post:[id,author_id,content,timestamp]
%S:Item:[id,name,price,quantity,category]

# Verbose (v1.0/v1.1 style)
%S:User:[id,name,email]
%S:Post:[id,author_id,content,timestamp]
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

A column list has the form `[col1,col2,...]` (compact) or `[col1, col2, ...]` (verbose).

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
%S:User:[id,name,email]                   # Valid - compact (preferred)
%S:User:[id, name, email]          # Valid - verbose
%S:User:[ id , name , email ]      # Valid - extra spaces
%S:User:[id,name,email,]                  # SyntaxError - trailing comma
%S:User:[]                                # SyntaxError - empty
%S:User:[id,id]                           # SchemaError - duplicate
```

### 6.4 `%N` / `%NEST` Directive (Optional)

Declares implicit parent-child relationships for automatic list nesting.

**Syntax**:
- Compact (v1.2+): `%N:ParentType>ChildType`
- Verbose (v1.0/v1.1): `%N:ParentType>ChildType`

**Requirements**:
- `ParentType` MUST be defined via `%S` or `%STRUCT`
- `ChildType` MUST be defined via `%S` or `%STRUCT`
- Each `(ParentType, ChildType)` pair MUST be unique (no duplicate rules)
- A `ParentType` MAY have multiple `%N`/`%NEST` rules for different `ChildType`s
- No circular nesting chains (not validated but must be acyclic)

**Semantics**:
When parsing a list of `ParentType`, rows indented one level deeper are interpreted as belonging to a child list of the appropriate `ChildType` (determined by the `@Type` marker), attached to the most recent parent row.

**Error Conditions**:
- Duplicate `%N`/`%NEST` directive for same `(ParentType, ChildType)` pair: `SchemaError`
- `ParentType` not in Schema Registry: `SchemaError`
- `ChildType` not in Schema Registry: `SchemaError`

**Example**:
```hedl
# Compact (v2.0 preferred)
%S:User:[id,name]
%S:Post:[id,content]
%N:User>Post

# Verbose (v1.0/v1.1 style)
%S:User:[id,name]
%S:Post:[id,content]
%N:User>Post
```

**Nesting Chains**: Multiple levels allowed:
```hedl
%S:Project:[id,name]
%S:Task:[id,description]
%S:SubTask:[id,details]
%N:Project>Task
%N:Task>SubTask
```

**Multiple Children**: Not supported in v1.0/v1.1 (one parent type, one child type). For complex hierarchies with multiple child types, use flattened lists with explicit parent references (foreign keys).

### 6.5 `%ALIAS` Directive (Optional)

Defines global constants for token substitution.

**Syntax**: `%A:%key: "expansion value"`

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
   - Example: `%A:%true: "true"`. Usage `| %true`. Expands to `true`. Inferred as **Boolean**.
   - Example: `%A:%val: "123"`. Usage `| %val`. Expands to `123`. Inferred as **Integer**.
   - It is NOT possible to alias a Quoted String structure. Usage `| "%val"` treats `%val` as a literal string.
4. NO recursive expansion (aliases cannot reference other aliases)
5. Aliases are only expanded for unquoted tokens
6. Quoted alias references (e.g., `"%active"`) are not expanded

**Examples**:
```hedl
%A:%active: "true"          # Expands to "true", then inferred as boolean true
%A:%inactive: "false"       # Expands to "false", then boolean false
%A:%empty: ""               # Expands to empty string
%A:%pi: "3.14159"           # Expands to "3.14159", then inferred as float
%A:%name: "John ""Doc"" Doe"  # Expands to John "Doc" Doe
```

**Invalid Examples**:
```hedl
%A:active: "true"           # Missing % on key
%A:%active: true            # Value not quoted
%A:%active: "true"          # OK
%A:%active: "false"         # AliasError - duplicate key
```

### 6.6 `%MODE` Directive (Optional, v1.1)

Controls parsing strictness for constraint violations.

**Syntax**: `%MODE: strict` or `%MODE: lenient`

**Values**:
- `strict` (default): First constraint violation is a hard error; parsing stops immediately
- `lenient`: Constraint violations become `~` (null); diagnostics are emitted out-of-band (implementation-defined)

**Requirements**:
- If present, MUST appear after `%VERSION` but before `%STRUCT` directives
- Only one `%MODE` directive is allowed per document
- If omitted, defaults to `strict`

**Examples**:
```hedl
%V:2.0
%MODE: strict
---
```

```hedl
%V:2.0
%MODE: lenient
---
```

**Lenient Mode Semantics**:
1. When a value violates a validation rule:
   - The value is replaced with `~` (null)
   - A diagnostic message is emitted (implementation-defined mechanism)
   - Parsing continues
2. Lenient mode does NOT affect syntax errors (which always halt parsing)
3. Lenient mode does NOT affect schema shape errors (which always halt parsing)
4. Diagnostics SHOULD include: line number, column, constraint violated, actual value

### 6.7-6.9 Removed Directives

The `%ENUM`, `%DICT`, and `%CONSTRAINT` directives were proposed in v1.1 but never shipped.
They are removed in v2.0. Parsers MUST reject these directives with a clear error message
indicating that they were removed and suggesting explicit values or external validation instead.

**Error Conditions**:
- Invalid predicate syntax: `SyntaxError`
- Unknown enum reference: `SchemaError`
- Constraint violation in strict mode: `ConstraintError`

### 6.10 `%PROMPT` Directive (Optional, v1.1)

Provides metadata hints for LLM/tooling consumption.

**Syntax**: `%PROMPT: "instruction text"`

**Requirements**:
- Value MUST be a quoted string
- Content is non-semantic (does not affect parsing or validation)
- Multiple `%PROMPT` directives are allowed; they are concatenated

**Semantics**:
1. Parsers MUST store prompt content as document metadata
2. Parsers MUST NOT use prompt content for parsing decisions
3. Tools MAY surface prompt content to LLMs or other consumers
4. Prompt content is preserved during round-trip canonicalization

**Examples**:
```hedl
%V:2.0
%PROMPT: "Answer questions by referencing entity IDs. Do not invent data."
%PROMPT: "When listing employees, include their department."
%S:Employee:[id, name, department]
---
employees:@Employee
 |e1, Alice, Engineering
 |e2, Bob, Sales
```

**Use Cases**:
- LLM instruction injection for RAG systems
- Documentation hints for tooling
- Processing directives for downstream consumers

### 6.11 `%NULL` Directive (REQUIRED for v2.0)

Declares the character used to represent null values.

**Syntax**: `%NULL:char`

**Requirements**:
- `char` MUST be a single ASCII character
- In v2.0+, this directive is REQUIRED and MUST be `%NULL:~`
- In v1.2 and earlier, default is `~` if not specified
- MUST appear after `%V` and before `%QUOTE`

**Examples**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
```

**Semantics**:
1. The declared character represents null in both key-value pairs and matrix cells
2. In matrix cells, an unquoted null character becomes the null value
3. To represent the null character literally, quote it: `"~"`

### 6.12 `%QUOTE` Directive (REQUIRED for v2.0)

Declares the character used for quoting strings.

**Syntax**: `%QUOTE:char`

**Requirements**:
- `char` MUST be a single ASCII character (typically `"`)
- In v2.0+, this directive is REQUIRED and MUST be `%QUOTE:"`
- In v1.2 and earlier, default is `"` if not specified
- MUST appear after `%NULL` and before the separator `---`

**Examples**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
```

**Semantics**:
1. The declared character delimits quoted strings
2. To include the quote character within a quoted string, escape it with backslash: `\"`
3. Standard escape sequences are supported within quoted strings: `\n`, `\t`, `\\`, `\"`

### 6.13 `%C` / `%COUNT` Directive (Optional, v1.2)

Declares count statistics and categorical distributions for validation and documentation.

**Syntax**:
- Total count: `%C:TypeName.total=N`
- Distribution: `%C:TypeName.field:value1=N1,value2=N2,...`

**Requirements**:
- `TypeName` SHOULD be defined via `%S` or `%STRUCT`
- Count values MUST be non-negative integers
- Multiple `%C` directives are allowed

**Examples**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,status,role]
%C:User.total=100
%C:User.status:active=85,inactive=10,suspended=5
%C:User.role:admin=5,editor=20,viewer=75
---
```

**Semantics**:
1. `total` specifies the expected total count of entities of that type
2. Field distributions specify the expected count for each categorical value
3. Parsers MAY validate that actual counts match declared counts
4. In `strict` mode, count mismatches MAY cause warnings (implementation-defined)
5. Count directives are primarily for documentation and tooling hints

**Use Cases**:
- Data quality validation
- LLM context hints about data distribution
- Documentation of dataset statistics

### 6.14 Minimal Header

For v2.0 documents, three header directives are REQUIRED:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
# Simple key-value pairs follow
```

For v1.2 and earlier (legacy):
```hedl
%V:1.2
---
```

---

## 7. Body Section

### 7.1 Body Line Classification

Each non-blank, non-comment line in the Body MUST be classified as exactly one of:

 |Type | Pattern | Description | Valid Context |
|------|---------|-------------|---------------|
 |Object Start | `key:` | Begins nested object mapping | Root, Object |
 |Key-Value | `key: value` | Assigns scalar to current object | Root, Object |
 |Matrix List Start | `key:@TypeName[...]` | Begins typed list with schema | Root, Object |
 |Matrix Row | `\| cell1, cell2, ...` | Data row in active matrix list | List |

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
   - Example: `users:@User` or `items:@Item[id, name]`
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

 |Current Context | Allowed Line Types |
|----------------|-------------------|
 |Root | Object Start, Key-Value, Matrix List Start |
 |Object | Object Start, Key-Value, Matrix List Start |
 |List | Matrix Row only (peer or child rows) |

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
ref:@user1        # Reference
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

**Format**: `key:@TypeName` or `key:@TypeName[col1, col2, ...]`

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
%S:User:[id,name]
---
users:@User
 |u1,Alice
 |u2,Bob

# Using inline schema (no %STRUCT needed)
items:@Item[id,name,price]
 |i1,Apple,1.99
 |i2,Banana,0.99

# Error - schema mismatch
%S:User:[id,name,email]
---
users:@User[id,name]  # SchemaError - column mismatch
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
```rust
fn parse_block_string(lines: &[String], base_indent: usize) -> Result<String, SyntaxError> {
    /// Parse a block string starting after the opening '"""'.
    /// base_indent is the indentation of the key line.
    let mut content_lines = Vec::new();
    for line in lines {
        let stripped = line.trim_start();
        if stripped == "\"\"\"" {
            // Closing found - join all content lines with newlines
            return Ok(content_lines.join("\n"));
        }
        // Strip base indentation from content line
        let indent_str = " ".repeat(base_indent);
        if line.starts_with(&indent_str) {
            content_lines.push(line[base_indent..].to_string());
        } else {
            content_lines.push(line.trim_start().to_string());
        }
    }
    Err(SyntaxError::new("Unclosed block string"))
}
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

3. **List Literal** (v1.1): Starts with `(` → attempt to parse as list
   - Must validate parenthesis balance
   - May contain any scalar values (null, references, expressions, booleans, numbers, strings)
   - Empty list: `()` is valid
   - Elements are parsed recursively using this inference ladder
   - **Failure Mode**: If a value starts with `(` but fails to parse (unbalanced parens, etc.), it is a `SyntaxError`
   - See Section 8.3 for detailed rules

4. **Reference**: Token starting with `@` → validate as Reference token:
   - MUST match pattern `@([A-Z][A-Za-z0-9]*:)?[a-z_][a-z0-9_\-]*`
   - If pattern not matched: `SyntaxError`
   - Otherwise: `Reference(ID)` where ID is the full reference (e.g., `user_1` or `User:user_1`)
   - Resolution happens later (Section 10.3)

5. **Expression**: Starts with `$(` and forms a valid Expression Token per Section 4.6.6 → `Expression(text)` (opaque)
   - `text` is everything between `$(` and the closing `)` (excluding delimiters)
   - No validation of expression content

6. **Alias**: Exact match of alias key → expand to defined string value
   - Apply inference to the expanded string:
     - If matches **Boolean** (true/false) → Boolean
     - If matches **Number** → Integer or Float
     - Otherwise → String (Note: The SyntaxError rule regarding quotes in unquoted strings does NOT apply here; the expanded value is accepted as-is)

7. **Boolean**: `true` or `false` (case-sensitive) → boolean
   - Exact match, lowercase
   - No type coercion (e.g., `"true"` → string, not boolean)

8. **Number**: Matches `^-?[0-9]+(\.[0-9]+)?$` → integer or float
   - **Integer**: No decimal point: `42`, `-1` → integer
   - **Float**: Contains decimal point: `42.0`, `3.14` → float
   - No scientific notation (`1e10` is string)
   - Leading zeros allowed (`001` → integer 1)
   - No underscores in numbers (`1_000` is string)

9. **String**: Anything else → string
   - Unquoted strings are trimmed
   - May contain any characters except those prohibited in Section 4.8
   - Empty unquoted string not possible (would be Object Start)

### 8.3 List Literals (v1.1)

List literals provide ordered sequences of scalar values, distinct from numeric tensors.

**Syntax**: `(elem1, elem2, ...)` or `()`

**Key Differences from Tensors**:
- **Tensors** (`[...]`): Numeric arrays only; used for AI/ML data
- **List Literals** (`(...)`): Any scalar types; used for general sequences

**Rules**:
1. Empty list `()` is always valid
2. Elements are separated by commas
3. Each element is parsed using the existing scalar inference ladder
4. Lists MAY contain heterogeneous types
5. Nested lists are NOT supported (use multiple columns or references)
6. Quoted elements preserve their string type

**Examples**:
```hedl
# Homogeneous lists
roles: (admin, editor, viewer)        # list of strings
flags: (true, false, true)            # list of booleans
counts: (1, 2, 3, 4, 5)               # list of integers

# Heterogeneous lists
mixed: (1, "two", ~, @ref1)           # int, string, null, reference

# Empty list
empty: ()                              # empty list

# In matrix cells
|e1, Alice, (admin, editor)           # roles column as list
```

**Inference in List Elements**:
- `admin` → string "admin"
- `true` → boolean true
- `42` → integer 42
- `3.14` → float 3.14
- `~` → null
- `@ref` → reference
- `"quoted"` → string "quoted" (no inference)

**Use Cases**:
- Multi-value fields (roles, tags, categories)
- Boolean flag arrays
- Reference collections
- Any ordered collection of scalars

### 8.4 Special Cases

* **Ditto**: `^` in Key-Value context is literal string `"^"` (doesn't trigger ditto behavior)
* **Empty Value**: `key:` (no value) is Object Start, NOT Key-Value
* **Whitespace Preservation**: Only in quoted strings; unquoted values are trimmed
* **Quoted Strings**: Always parsed as strings, no inference
* **Mixed Quoting**: Not allowed; a value like `"hello` without closing quote is Syntax Error

### 8.5 Examples

```hedl
# Key-Value examples
null_val: ~                     # null
tensor_val: [[1, 2], [3, 4]]   # tensor/array
ref_val:@node1                 # Reference("node1")
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

```rust
fn extract_csv_content(line: &str) -> Result<String, SyntaxError> {
    /// Extracts the CSV content from a matrix row line.
    /// Precondition: line is known to contain '|'.

    // 1. Find delimiter
    let pipe_idx = match line.find('|') {
        Some(idx) => idx,
        None => return Err(SyntaxError::new("Matrix row missing '|'")),
    };

    // 2. Extract raw content after the pipe
    let raw_content = &line[pipe_idx+1..];

    // 3. Strip comments using the standard strip_comment function (Section 4.5)
    // This handles comments respecting quotes and expressions
    let comment_stripped = strip_comment(raw_content);

    // 4. Trim leading/trailing whitespace
    let csv_content = comment_stripped.trim().to_string();

    Ok(csv_content)
}
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

```rust
fn parse_csv_row(csv_string: &str) -> Result<Vec<(String, bool)>, SyntaxError> {
    /// Parse a CSV string into a list of (value, is_quoted) tuples.
    /// Uses a state machine that respects quoted strings and expressions.
    if csv_string.is_empty() {
        return Ok(Vec::new());
    }

    // Check for trailing comma
    if csv_string.trim_end().ends_with(',') {
        return Err(SyntaxError::new("Trailing comma not allowed in matrix row"));
    }

    let mut fields = Vec::new();  // List of (value, is_quoted) tuples
    let mut current_field = String::new();
    let mut current_is_quoted = false;
    let mut state = "START_FIELD";
    let mut i = 0;
    let mut expression_depth = 0;
    let mut in_expr_quotes = false;  // Track quotes inside expressions (must match scan_regions)

    let chars: Vec<char> = csv_string.chars().collect();

    while i < chars.len() {
        let char = chars[i];

        if state == "START_FIELD" {
            current_is_quoted = false;
            if char.is_whitespace() {
                i += 1;
                continue;
            } else if char == '"' {
                current_is_quoted = true;
                state = "IN_QUOTED_FIELD";
                i += 1;
            } else if char == '$' && i + 1 < chars.len() && chars[i + 1] == '(' {
                // Start of expression
                current_field.push_str("$(");
                state = "IN_EXPRESSION";
                expression_depth = 1;
                i += 2;
            } else {
                state = "IN_UNQUOTED_FIELD";
                current_field.push(char);
                i += 1;
            }
        } else if state == "IN_UNQUOTED_FIELD" {
            if char == ',' {
                // End of field
                let field = current_field.trim().to_string();
                if field.contains('"') {
                    return Err(SyntaxError::new(&format!("Quote character '\"' found in unquoted CSV field: '{}'", field)));
                }
                fields.push((field, false));
                current_field = String::new();
                state = "START_FIELD";
                i += 1;
            } else {
                current_field.push(char);
                i += 1;
            }
        } else if state == "IN_QUOTED_FIELD" {
            if char == '"' {
                if i + 1 < chars.len() && chars[i + 1] == '"' {
                    // Escaped quote via "" - add single quote to field
                    current_field.push('"');
                    i += 2;
                } else {
                    // End of quoted field
                    state = "AFTER_QUOTE";
                    i += 1;
                }
            } else if char == '\\' && i + 1 < chars.len() {
                // Escape sequence handling
                let next_char = chars[i + 1];
                if next_char == 'n' {
                    current_field.push('\n');
                    i += 2;
                } else if next_char == 't' {
                    current_field.push('\t');
                    i += 2;
                } else if next_char == 'r' {
                    current_field.push('\r');
                    i += 2;
                } else if next_char == '\\' {
                    current_field.push('\\');
                    i += 2;
                } else if next_char == '"' {
                    current_field.push('"');
                    i += 2;
                } else {
                    // Unknown escape - treat backslash literally
                    current_field.push(char);
                    i += 1;
                }
            } else {
                current_field.push(char);
                i += 1;
            }
        } else if state == "AFTER_QUOTE" {
            if char.is_whitespace() {
                i += 1;
                continue;
            } else if char == ',' {
                fields.push((current_field.clone(), true));
                current_field = String::new();
                state = "START_FIELD";
                i += 1;
            } else {
                return Err(SyntaxError::new(&format!("Expected comma after closing quote, got '{}'", char)));
            }
        } else if state == "IN_EXPRESSION" {
            current_field.push(char);
            // Handle quotes inside expressions (must match scan_regions behavior)
            if char == '"' {
                if in_expr_quotes {
                    if i + 1 < chars.len() && chars[i + 1] == '"' {
                        // Escaped quote inside expression
                        current_field.push(chars[i + 1]);
                        i += 2;
                        continue;
                    } else {
                        in_expr_quotes = false;
                    }
                } else {
                    in_expr_quotes = true;
                }
            } else if !in_expr_quotes {
                if char == '(' {
                    expression_depth += 1;
                } else if char == ')' {
                    expression_depth -= 1;
                    if expression_depth == 0 {
                        // End of expression
                        state = "IN_UNQUOTED_FIELD";
                        in_expr_quotes = false;  // Reset for safety
                    }
                }
            }
            i += 1;
        }
    }

    // Handle end of string
    if state == "IN_QUOTED_FIELD" {
        return Err(SyntaxError::new("Unclosed quoted string in CSV field"));
    } else if state == "IN_EXPRESSION" {
        return Err(SyntaxError::new("Unclosed expression in CSV field"));
    } else if state == "AFTER_QUOTE" {
        fields.push((current_field, true));
    } else if !current_field.is_empty() {
        let field = current_field.trim().to_string();
        if field.contains('"') {
            return Err(SyntaxError::new(&format!("Quote character '\"' found in unquoted CSV field: '{}'", field)));
        }
        fields.push((field, false));
    }

    Ok(fields)
}
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

### 9.4 Ditto Scoping Rules (v1.2 only)

> **v2.0 Breaking Change**: The ditto operator (`^`) is NOT allowed in v2.0 documents. Every cell must have an explicit value. Parsers MUST reject `^` in v2.0 documents with a `SemanticError`. This section documents v1.2 behavior for backward compatibility.

The `^` operator copies from the **same column** of the **previous row** in the **same list frame** (v1.2 only):

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
data:@Item[id,name,count,price]
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
%S:User:[id,name,email]
---
users:@User
 |u1,Alice               # ShapeError: Expected 3 columns, got 2
 |u2,Bob,bob@ex.com,extra  # ShapeError: Expected 3 columns, got 4
 |u3,Carol,carol@ex.com # OK
```

### 9.6 Count Hints (v1.2 Only - DEPRECATED in v2.0)

> **DEPRECATED**: Inline count hints (`|[N]`) are removed in v2.0. Use `%C:` header directives for counts and `@Type#N:` child blocks for structure. See Section 6.10 for v2.0 count directives.

Count hints provide optional metadata about the number of direct children for parent rows in nested hierarchies. They are particularly useful for LLM consumption, as they help models understand data structure boundaries.

**Syntax (v1.2 only)**:
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

```rust
fn extract_count_hint(csv_content: &str) -> Result<(Option<usize>, String), SyntaxError> {
    /// Extract count hint from CSV content.
    /// Returns (count_hint, remaining_csv) where count_hint is int or None.
    let trimmed = csv_content.trim_start();
    if !trimmed.starts_with('[') {
        return Ok((None, csv_content.to_string()));
    }

    // Find closing bracket
    let close_idx = match trimmed.find(']') {
        Some(idx) => idx,
        None => return Err(SyntaxError::new("Unclosed count hint bracket")),
    };

    // Extract count value
    let count_str = trimmed[1..close_idx].trim();
    if !count_str.chars().all(|c| c.is_ascii_digit()) {
        return Err(SyntaxError::new(&format!("Invalid count hint: [{}] must be non-negative integer", count_str)));
    }

    let count = count_str.parse::<usize>()
        .map_err(|_| SyntaxError::new("Invalid count hint value"))?;
    let remaining = trimmed[close_idx+1..].trim_start().to_string();

    Ok((Some(count), remaining))
}
```

**Examples (v1.2 syntax)**:
```hedl
%V:1.2
%S:Organization:[id,name]
%S:Department:[id,name]
%S:Employee:[id,name]
%N:Organization>Department
%N:Department>Employee
---
organizations:@Organization
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

**v2.0 equivalent** (using child blocks and header counts):
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Organization:[id,name]
%S:Department:[id,name]
%S:Employee:[id,name]
%N:Organization>Department
%N:Department>Employee
%C:Organization.total=2
%C:Department.total=3
%C:Employee.total=4
---
organizations:@Organization
 |org1,TechCorp
  @Department#2:
  |dept1,Engineering
   @Employee#3:
   |emp1,Alice
   |emp2,Bob
   |emp3,Carol
  |dept2,Sales
   @Employee#1:|emp4,David
 |org2,DataCo
  @Department#1:
  |dept3,Research
```

**Validation** (Optional):

Implementations MAY optionally validate that count hints match actual child counts:

```rust
fn validate_count_hint(node: &Node, count_hint: Option<usize>) {
    /// Optional validation that count hint matches actual children.
    if let Some(hint) = count_hint {
        let actual_count = node.children.len();
        if hint != actual_count {
            // Warning or error at implementation discretion
            warn(&format!("Count hint mismatch: expected {}, got {}", hint, actual_count));
        }
    }
}
```

**Use Cases**:
- Helping LLMs understand hierarchical structure boundaries
- Providing metadata for streaming parsers about upcoming data
- Enabling integrity checks for data transmission
- Documenting expected data structure for human readers

**Canonical Form**:

Canonical formatters SHOULD include accurate count hints for parent rows in nested hierarchies. Count hints SHOULD be omitted for leaf rows (rows with no children).

### 9.7 Inline Child Lists (v1.2)

HEDL v1.2 introduces compact inline child list syntax for attaching child rows to parent rows without additional line breaks. This significantly reduces token count while maintaining structural clarity.

**Syntax**:
```
|parent_row_data
 @ChildType#N:|child1|child2|...|childN
```

Where:
- Space indent (one level deeper than parent)
- `@ChildType` references the child type schema
- `#N` declares the count of inline children (REQUIRED for inline format)
- `:` separates the declaration from the data
- Children are separated by `|` on the same line
- NO space after `|` in child data (use `|data` NOT `| data`)

**Expanded Form (v1.1)**:
```
|parent_row_data
 |child1
 |child2
  ...
 |childN
```

**Inline Form (v1.2)**:
```
|parent_row_data
 @ChildType#N:|child1|child2|...|childN
```

**Requirements**:
1. Maximum 5 inline children allowed (MUST enforce). For more than 5 children, use expanded form.
2. The `#N` count hint is REQUIRED for inline format (unlike optional for parent rows)
3. The `#N` count MUST match the actual number of pipe-separated children
4. `ChildType` MUST be defined via `%S` or `%STRUCT`
5. `ChildType` MUST be declared as child of the parent type via `%N` or `%NEST`
6. The inline children follow the same CSV parsing rules as regular matrix rows
7. NO space after `|` delimiter in inline child data (correct: `|data`, incorrect: `| data`)
8. Ditto (`^`) works within inline children (references previous child on same line)
9. Multiple child types can each have their own inline declaration

**Example**:
```hedl
%V:1.2
%S:Product:[id,name,price]
%S:Review:[id,rating,text]
%N:Product>Review
---
products:@Product
 |prod-001,Laptop,999.99
    @Review#3:|rev-001,5,Great product|rev-002,4,Good value|rev-003,^,Recommended
 |prod-002,Phone,599.99
    @Review#2:|rev-004,5,Excellent|rev-005,3,Average
```

**Mixed Inline and Expanded**:

You may combine inline and expanded forms for the same child type. When a parent has more than 5 children, you MUST use the expanded form:

```hedl
products:@Product
 |prod-001,Laptop,999.99
    @Review#2:|rev-001,5,Great|rev-002,4,Good
 |prod-002,Phone,599.99
    @Review#7:
  |rev-003,5,Excellent
  |rev-004,4,Very good
  |rev-005,3,Okay
  |rev-006,2,Needs improvement
  |rev-007,5,Amazing
  |rev-008,4,Nice
  |rev-009,3,Decent
```

**Note**: When using expanded form after `@Type#N:`, the count hint `#N` is still REQUIRED, and child rows are indented one level deeper than the `@Type#N:` line.

**Multiple Child Types**:

```hedl
products:@Product
 |prod-001,Laptop,999.99
    @Review#2:|rev-001,5,Great|rev-002,4,Good
    @Inventory#2:|inv-001,warehouse-east,50|inv-002,warehouse-west,30
```

**Parsing Algorithm**:

1. Detect inline child declaration: line starts with whitespace + `@TypeName#N:`
2. Parse count N from `#N` (REQUIRED, not optional)
3. Validate N <= 5 (if N > 5, raise SyntaxError: "Inline child lists limited to 5 entries")
4. Split remaining content after `:` by `|` to get child rows
5. Validate actual child count matches N (if mismatch, raise SyntaxError)
6. Parse each child as a standard CSV row
7. Attach children to most recent parent row

**Indentation Rules**:

Inline child lists follow strict indentation rules to maintain structural clarity:

1. **List declaration** (e.g., `products:@Product`): Indent level 0 (top-level)
2. **Parent rows** (e.g., `|prod-001,...`): Indent level 1 (1 space)
3. **Inline child declarations** (e.g., `@Review#2:|...`): Indent level 2 (2 spaces, one level deeper than parent)
4. **Expanded child rows** (when using `@Type#N:` without inline data): Indent level 2 (2 spaces, same as declaration)

**Complete Example**:
```hedl
products:@Product                         # Level 0: List declaration
 |prod-001,Laptop,999.99                   # Level 1: Parent row (1 space)
  @Review#3:|r1,5,Great|r2,4,Good|r3,3,OK  # Level 2: Inline children (2 spaces)
 |prod-002,Tablet,299.99                   # Level 1: Parent row (1 space)
  @Review#7:                               # Level 2: Expanded declaration (2 spaces)
  |r4,5,Amazing                            # Level 2: Child row (2 spaces)
  |r5,4,Good                               # Level 2: Child row (2 spaces)
  |r6,3,OK                                 # Level 2: Child row (2 spaces)
  |r7,5,Love                               # Level 2: Child row (2 spaces)
  |r8,4,Nice                               # Level 2: Child row (2 spaces)
  |r9,3,Decent                             # Level 2: Child row (2 spaces)
  |r10,2,Meh                               # Level 2: Child row (2 spaces)
```

**Use Cases**:
- Reducing token count for LLM context windows
- Compact serialization of sparse hierarchies
- Streaming scenarios where row-by-row output is preferred

**Design Guidelines**:

When to use inline format (`@Type#N:|data|data|...`):
- Small number of children (1-5 entries)
- Simple, short field values
- Prioritizing token efficiency
- Data that benefits from visual compactness

When to use expanded format (`@Type#N:` followed by `|` rows):
- More than 5 children (REQUIRED)
- Complex field values with many columns
- Data that benefits from readability
- When vertical alignment aids comprehension

**Rationale for 5-Entry Limit**:
- Maintains readability of inline format
- Prevents excessively long lines (recommended max line length: 1MB, see Section 4.2)
- Balances token efficiency with human comprehension
- Simplifies parser implementation and error reporting

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
%S:User:[id,name]
%S:Product:[id,name]
---
users:@User
 |admin,Alice
products:@Product
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
%S:Task:[id,name,depends_on]
---
tasks:@Task
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
%V:2.0
%S:User:[id,name]
%S:Post:[id,content]
---
users:@User
 |alice,Alice
posts:@Post
 |p1,Hello
config:
  admin_ref:@User:alice    # Qualified - recommended
  post_ref:@Post:p1        # Qualified - recommended
  ambiguous:@alice         # Unqualified - searches all types, finds User:alice
```

### 10.4 Child List Attachment

When `%N:Parent>Child` is active:

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
%S:User:[id,name]
%S:Post:[id,content]
%N:User>Post
---
users:@User
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
%S:User:[id,name,age]
%S:Post:[id,title]
%N:User>Post
---
users:@User
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
```rust
fn preprocess(input_data: &[u8], max_size: usize) -> Result<Vec<String>, SyntaxError> {
    // Default max_size: 1GB
    let max_size = if max_size == 0 { 1024 * 1024 * 1024 } else { max_size };

    // Check size
    if input_data.len() > max_size {
        return Err(SecurityError::new(&format!("File too large: {} > {}", input_data.len(), max_size)));
    }

    // Validate and Decode UTF-8
    let mut text = std::str::from_utf8(input_data)
        .map_err(|_| SyntaxError::new("Invalid UTF-8 encoding"))?
        .to_string();

    // Skip BOM if present
    if text.starts_with('\u{FEFF}') {
        text = text[3..].to_string();
    }

    // Check for control characters (allow LF, CR, TAB)
    for (i, ch) in text.chars().enumerate() {
        let code = ch as u32;
        // Allow: LF (0x0A), CR (0x0D), TAB (0x09)
        if code < 0x20 && code != 0x0A && code != 0x0D && code != 0x09 {
            return Err(SyntaxError::new(&format!("Control character U+{:04X} at position {}", code, i)));
        }
    }

    // Note: Tab usage is restricted by specific parsers:
    // - Indentation: Tabs PROHIBITED (Section 4.3)
    // - Unquoted Strings: Tabs PROHIBITED (Section 4.8)
    // - Quoted Strings: Tabs ALLOWED (Section 8.1.1)

    // Normalize line endings: CRLF -> LF, reject bare CR
    if text.contains('\r') {
        // Replace CRLF first
        text = text.replace("\r\n", "\n");
        // Now check for any remaining bare CR (not part of CRLF)
        if text.contains('\r') {
            // Find the line number where bare CR occurs
            let line_num = text[..text.find('\r').unwrap()].matches('\n').count() + 1;
            return Err(SyntaxError::new(&format!("Bare CR (U+000D) found at line {}", line_num)));
        }
    }

    // Split lines
    let lines = text.split('\n').map(|s| s.to_string()).collect();
    Ok(lines)
}
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
```rust
fn parse_header(lines: &[String]) -> Result<(HashMap<String, Vec<String>>, HashMap<String, String>, HashMap<String, String>, usize), SyntaxError> {
    let mut schemas = HashMap::new();
    let mut aliases = HashMap::new();
    let mut nests = HashMap::new();
    let mut version_seen = false;
    let mut first_directive = true;

    for (line_num, line) in lines.iter().enumerate() {
        let line_num = line_num + 1; // 1-based line numbers

        // Strip leading/trailing whitespace for separator check
        let stripped_line = line.trim_end_matches('\n');

        // Check for separator with strict leading space rule
        if stripped_line == "---" || stripped_line.starts_with("--- ") {
            if !version_seen {
                return Err(SyntaxError::new(&format!("Missing %VERSION directive in header before separator at line {}", line_num)));
            }
            // Valid separator found
            return Ok((schemas, aliases, nests, line_num + 1));
        }

        // Check for malformed separator (leading spaces or extra hyphens)
        if stripped_line.trim_start().starts_with("---") {
            if stripped_line.starts_with(' ') {
                return Err(SyntaxError::new(&format!("Separator '---' must not have leading spaces at line {}", line_num)));
            } else {
                // Something like '----'
                let preview = if stripped_line.len() > 10 { &stripped_line[..10] } else { stripped_line };
                return Err(SyntaxError::new(&format!("Separator must be exactly '---', found '{}...' at line {}", preview, line_num)));
            }
        }

        if line.trim().is_empty() || line.trim().starts_with('#') {
            continue;
        }

        // Parse directive
        if !line.starts_with('%') {
            return Err(SyntaxError::new(&format!("Expected directive at line {}", line_num)));
        }

        // Split directive name and payload with flexible spacing
        if !line.contains(':') {
            return Err(SyntaxError::new(&format!("Invalid directive format at line {}", line_num)));
        }

        let parts: Vec<&str> = line.splitn(2, ':').collect();
        let name = parts[0];
        let payload = parts[1];

        if !payload.starts_with(' ') {
            return Err(SyntaxError::new(&format!("Directive ':' must be followed by at least one space at line {}", line_num)));
        }

        // Enforce %VERSION as first directive
        if first_directive {
            if name != "%VERSION" {
                return Err(SyntaxError::new(&format!("%VERSION must be the first directive, found {} at line {}", name, line_num)));
            }
            first_directive = false;
        }

        let payload = payload.trim_start_matches(' ');

        // Remove inline comment from payload
        let payload = strip_comment(payload);

        // Dispatch based on directive name
        if name == "%VERSION" {
            parse_version(&payload, line_num)?;
            version_seen = true;
        } else if name == "%STRUCT" {
            parse_struct(&payload, &mut schemas, line_num)?;
        } else if name == "%ALIAS" {
            parse_alias(&payload, &mut aliases, line_num)?;
        } else if name == "%NEST" {
            parse_nest(&payload, &mut nests, &schemas, line_num)?;
        // v1.1 directives
        } else if name == "%MODE" {
            parse_mode(&payload, line_num)?;  // Sets strict or lenient mode
        } else if name == "%ENUM" || name == "%DICT" || name == "%CONSTRAINT" {
            return Err(SyntaxError("removed directive: %ENUM, %DICT, and %CONSTRAINT are not supported in v2.0", line_num));
        } else if name == "%PROMPT" {
            parse_prompt(&payload, &mut prompts, line_num)?;
        } else if name.starts_with("%X-") {
            // Experimental directive - accept with warning (v1.1)
            warn(&format!("Unknown experimental directive '{}' at line {}", name, line_num));
            store_experimental(name, &payload, line_num)?;
        } else {
            return Err(SyntaxError::new(&format!("Unknown directive {} at line {}", name, line_num)));
        }
    }

    Err(SyntaxError::new("Missing separator '---'"))
}
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
- For explicit lists (from `key:@TypeName`): `listStartIndent` is the indent level of the list start line
- For implicit lists (via `%NEST`): `listStartIndent` is synthetic = current indent level - 1
- `rowIndent` is always `listStartIndent + 1`

### 11.4 Scope Closing (Pop Rules)

Before processing each Body line at indent level `I`:

```rust
fn pop_frames(stack: &mut Vec<Frame>, current_indent: usize) {
    /// Pop frames that are no longer relevant.
    loop {
        if stack.is_empty() {
            break;
        }

        let top = &stack[stack.len() - 1];

        if top.kind == FrameKind::List {
            // Pop list if we're leaving its row scope
            if current_indent < top.row_indent {
                stack.pop();
                continue;
            }
        } else if top.kind == FrameKind::Object {
            // Pop object if we're returning to its level or shallower
            if current_indent <= top.indent {
                stack.pop();
                continue;
            }
        }

        // No more frames to pop
        break;
    }
}
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

#### Case 3: Matrix List Start (`key:@TypeName`)
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
   ```rust
   fn resolve_references(node: &mut Node, type_registries: &HashMap<String, HashMap<String, Node>>, current_type: Option<&str>, strict: bool) -> Result<(), ReferenceError> {
       match node {
           Node::Reference(ref_node) => {
               // Determine target type and ID
               let (target_type, target_id) = if ref_node.id.contains(':') {
                   let parts: Vec<&str> = ref_node.id.splitn(2, ':').collect();
                   let mut target_type = parts[0].to_string();
                   // Strip optional @ if present in split (grammar handles this, but for safety)
                   if target_type.starts_with('@') {
                       target_type = target_type[1..].to_string();
                   }
                   (Some(target_type), parts[1].to_string())
               } else {
                   (current_type.map(|s| s.to_string()), ref_node.id.clone())
               };

               // Resolve
               if let Some(ref target_type_str) = target_type {
                   if let Some(registry) = type_registries.get(target_type_str) {
                       if registry.contains_key(&target_id) {
                           return Ok(()); // Resolved
                       } else if strict {
                           return Err(ReferenceError::new(&format!("Unresolved reference @{} in type {}", target_id, target_type_str)));
                       } else {
                           *node = Node::Null;
                           return Ok(());
                       }
                   } else if strict {
                       return Err(ReferenceError::new(&format!("Cannot resolve reference to unknown type {}", target_type_str)));
                   }
               }
               Ok(())
           }
           Node::Object(obj) => {
               // If this dict represents a typed Node, update current_type
               // (Assuming Node structure from Section 10.5 where properties are merged)
               let new_type = obj.get("type")
                   .and_then(|t| if let Node::String(s) = t { Some(s.as_str()) } else { None })
                   .or(current_type);

               for (_, v) in obj.iter_mut() {
                   resolve_references(v, type_registries, new_type, strict)?;
               }
               Ok(())
           }
           Node::List(list) => {
               for item in list.iter_mut() {
                   resolve_references(item, type_registries, current_type, strict)?;
               }
               Ok(())
           }
           _ => Ok(()),
       }
   }
   ```

4. **Validation**: Ensure no dangling references in strict mode
5. **Return Root**: Return the root object

### 11.7 Complete Pseudo-Code with Truncation Detection

```rust
struct HEDLParser {
    strict: bool,
    max_indent: usize,
    max_nodes: usize,
    schemas: HashMap<String, Vec<String>>,
    aliases: HashMap<String, String>,
    nests: HashMap<String, String>,
    type_registries: HashMap<String, HashMap<String, Node>>,  // type -> {id -> node}
    references: Vec<(String, String, String)>,  // List of (type, id, path) tuples
    node_count: usize,
    current_type: Option<String>,  // Track current type for reference resolution
}

impl HEDLParser {
    fn new(strict: bool, max_indent: usize, max_nodes: usize) -> Self {
        HEDLParser {
            strict,
            max_indent,
            max_nodes,
            schemas: HashMap::new(),
            aliases: HashMap::new(),
            nests: HashMap::new(),
            type_registries: HashMap::new(),
            references: Vec::new(),
            node_count: 0,
            current_type: None,
        }
    }

    fn parse(&mut self, text: &str) -> Result<Node, HEDLError> {
        // Phase 1: Preprocessing
        let lines = self.preprocess(text)?;

        // Phase 2: Header parsing
        let body_start = match self.parse_header(&lines) {
            Err(e) => {
                // Check if error is due to EOF before separator
                if e.message.contains("Missing separator") {
                    return Err(SyntaxError::new("Truncated file: missing separator '---'"));
                }
                return Err(e);
            }
            Ok(start) => start,
        };

        // Phase 3: Body parsing
        let root = self.parse_body(&lines[body_start..])?;

        // Phase 4: Post-processing
        // Truncation validation is handled inside parse_body
        self.resolve_references(&root, &self.type_registries)?;

        Ok(root)
    }

    fn parse_body(&mut self, lines: &[String]) -> Result<Node, HEDLError> {
        let mut stack = vec![Frame {
            kind: FrameKind::Root,
            indent: -1,
            object: HashMap::new(),
        }];

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;

            if self.is_blank(line) || self.is_comment(line) {
                continue;
            }

            let stripped_line_content = line.trim();
            if stripped_line_content == "---" {
                return Err(SyntaxError::new(&format!("Multiple separators '---' are not allowed. Found at line {}.", line_num)));
            }

            let indent = self.calculate_indent(line)?;
            self.validate_indent(indent)?;

            // Scope closing
            self.pop_frames(&mut stack, indent);

            // Classify and parse line
            let line_content = &line[indent * 2..];  // Remove indentation

            if line_content.starts_with('|') {
                self.parse_matrix_row(&mut stack, line_content, indent, line_num)?;
            } else {
                self.parse_non_matrix_line(&mut stack, line_content, indent, line_num)?;
            }
        }

        // Pop remaining frames except root
        while stack.len() > 1 {
            let frame = stack.pop().unwrap();
            if frame.kind == FrameKind::Object {
                return Err(SyntaxError::new(&format!("Unclosed object '{}' at end of file", frame.parent_key.unwrap_or("?".to_string()))));
            } else if frame.kind == FrameKind::List {
                return Err(SyntaxError::new(&format!("Unclosed list '{}' at end of file", frame.type_name.unwrap_or("?".to_string()))));
            }
        }

        // Check for unterminated tokens in last line
        if !lines.is_empty() && !self.is_blank(&lines[lines.len() - 1]) {
            let last_line = lines[lines.len() - 1].trim_end_matches('\n');
            if self.is_unterminated_token(last_line) {
                return Err(SyntaxError::new("Truncated token at end of file"));
            }
        }

        Ok(stack[0].object.clone())
    }

    fn is_unterminated_token(&self, line: &str) -> bool {
        /// Check if line ends with unterminated token using scan_regions.
        let regions = scan_regions(line);

        // Check for unclosed quoted string or expression that extends to the end of the line
        for (start, end, _type) in regions {
            if end == line.len() { // Region extends to end of line
                let chars: Vec<char> = line.chars().collect();
                if _type == "quote" && chars[end - 1] != '"' {
                    return true; // Unclosed quote
                }
                if _type == "expression" && chars[end - 1] != ')' {
                    return true; // Unclosed expression
                }
            }
        }

        false
    }
}
```

---

## 12. Error Hierarchy

### 12.1 Error Categories

 |Error | When Raised | Recoverable? | Example |
|-------|-------------|--------------|---------|
 |`SyntaxError` | Lexical or structural violation | No | Odd indentation, tab character, unclosed structure |
 |`VersionError` | Unsupported version | No | `%V:2.0` with 1.0 parser |
 |`SchemaError` | Schema violation or mismatch | No | Duplicate struct, nest to undefined type |
 |`AliasError` | Duplicate or invalid alias | No | `%A:%key: "val"` (duplicate) |
 |`ShapeError` | Wrong number of cells in row | No | Expected 3 columns, got 2 |
 |`SemanticError` | Logical error | No | Ditto in ID column, null in ID column |
 |`OrphanRowError` | Child row without %NEST | No | Indented row with no nest rule |
 |`CollisionError` | Duplicate ID within type | No | Same ID in same type |
 |`ReferenceError` | Unresolved reference (strict mode) | No | `@missing` with no definition |
 |`SecurityError` | Security limit exceeded | No | File too large, nesting too deep |

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
```rust
match parser.parse(&text) {
    Ok(result) => result,
    Err(e) => {
        if let Some(line) = e.line {
            eprintln!("Error at line {}: {}", line, e.message);
        } else {
            eprintln!("Error: {}", e.message);
        }
        if let Some(context) = e.context {
            eprintln!("Context: {}", context);
        }
        return Err(e);
    }
}
```

### 12.4 Error Class Definition

```rust
struct HEDLError {
    /// Base class for all HEDL errors.
    message: String,
    line: Option<usize>,
    column: Option<usize>,
    context: Option<String>,
}

impl HEDLError {
    fn new(message: &str, line: Option<usize>, column: Option<usize>, context: Option<String>) -> Self {
        HEDLError {
            message: message.to_string(),
            line,
            column,
            context,
        }
    }

    fn format_message(&self, error_type: &str) -> String {
        let mut parts = Vec::new();
        if let Some(line) = self.line {
            parts.push(format!("line {}", line));
        }
        if let Some(column) = self.column {
            parts.push(format!("column {}", column));
        }
        let location = if !parts.is_empty() {
            format!(" at {}", parts.join(":"))
        } else {
            String::new()
        };
        format!("{}{}: {}", error_type, location, self.message)
    }
}

struct SyntaxError(HEDLError);

struct VersionError(HEDLError);

struct SchemaError(HEDLError);

struct AliasError(HEDLError);

struct ShapeError(HEDLError);

struct SemanticError(HEDLError);

struct OrphanRowError(HEDLError);

struct CollisionError(HEDLError);

struct ReferenceError(HEDLError);

struct SecurityError(HEDLError);

```

---

## 13. Canonicalization (Generators)

To ensure stable hashing, diffing, and deterministic output:

### 13.1 Required Practices

1. **Line Endings**: `\n` only
2. **No Trailing Whitespace**: Trim end of every line
3. **Separator**: Exactly `---\n`
4. **Indentation**: Exactly 1 space per level (v2.0), no tabs
5. **No BOM**: Do not include UTF-8 BOM

### 13.2 Header Directive Order

Generate directives in this order:

1. `%V:2.0`
2. `%ALIAS`: Sorted by key (ASCII ascending)
3. `%STRUCT`: Sorted by TypeName (ASCII)
4. `%NEST`: Sorted by ParentType then ChildType (ASCII)

**Example**:
```hedl
%V:2.0
%A:%active: "true"
%A:%inactive: "false"
%S:Post:[id,content]
%S:User:[id,name]
%N:User>Post
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

### 13.4 Ditto Optimization (v1.2 only)

> **v2.0 Breaking Change**: Ditto optimization is NOT available in v2.0. All values must be written explicitly. This section applies only to v1.2 documents.

Use `^` when value equals previous row same column in same list (v1.2 only).

**Rules**:
1. Only in matrix cells, not Key-Value
2. Not in ID column
3. Not in first row
4. Compare values deeply (including type)
5. **Not allowed in v2.0 documents**

**Example (v1.2 only)**:
```hedl
%V:1.2
---
data:@Item
|a,Apple,1.99
|b,^,0.99    # Apple copied (v1.2 only)
|c,Orange,^  # 0.99 copied (v1.2 only)
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
```rust
struct MatrixList {
    type_name: String,
    schema: Vec<String>,
    rows: Vec<HashMap<String, Value>>,  // List of dicts mapping column->value
}

impl MatrixList {
    fn new(type_name: String, schema: Vec<String>) -> Self {
        MatrixList {
            type_name,
            schema,
            rows: Vec::new(),
        }
    }

    // Allow list-like access
    fn len(&self) -> usize {
        self.rows.len()
    }

    fn get(&self, idx: usize) -> Option<&HashMap<String, Value>> {
        self.rows.get(idx)
    }

    fn append(&mut self, row: HashMap<String, Value>) {
        self.rows.push(row);
    }
}
```

### 13.8 Canonicalization of v1.1 Features

**List Literals**:
- Canonical form: `(elem1, elem2, ...)` with single space after commas
- Empty list: `()`
- Elements follow standard scalar canonicalization


**%MODE Directive**:
- If mode is `strict` (default), MAY be omitted in canonical output
- If mode is `lenient`, MUST be included

**%PROMPT Directives**:
- Multiple prompts MUST be preserved in original order
- Content is preserved exactly (no normalization)

**Header Directive Order**:
1. `%V:2.0`
2. `%NULL`: Null symbol declaration
3. `%QUOTE`: Quote character declaration
4. `%MODE`: if lenient (omit if strict)
5. `%ALIAS`: Sorted by key (ASCII ascending)
6. `%STRUCT`: Sorted by TypeName (ASCII)
7. `%NEST`: Sorted by ParentType then ChildType (ASCII)
8. `%COUNT`: Sorted by TypeName (ASCII)
9. `%PROMPT`: In original order
10. `%X-*`: Experimental directives preserved in original order

### 13.9 Complete Canonicalization Algorithm (Informative)

**Note**: This section is non-normative and provided for illustration only. Implementations MUST adhere to the normative requirements in Sections 13.1-13.7 but are NOT required to follow this specific algorithm.

```rust
fn canonicalize(data: &Value, indent: usize) -> Vec<String> {
    /// Convert data structure to canonical HEDL. (Informative example)
    if let Value::Object(map) = data {
        // Sort keys
        let mut items: Vec<_> = map.iter().collect();
        items.sort_by_key(|(k, _)| *k);

        let mut lines = Vec::new();
        for (key, value) in items {
            if let Value::MatrixList(matrix_list) = value {
                // Matrix list with metadata
                lines.push(canonicalize_matrix_list(key, matrix_list, indent));
            } else if let Value::Object(_) = value {
                // Object
                lines.push(format!("{}{}: ", " ".repeat(indent), key));
                lines.extend(canonicalize(value, indent + 1));
            } else {
                // Scalar
                lines.push(format!("{}{}: {}", " ".repeat(indent), key, canonicalize_value(value)));
            }
        }
        lines
    } else {
        Vec::new()
    }
}

fn canonicalize_matrix_list(key: &str, matrix_list: &MatrixList, indent: usize) -> String {
    /// Canonicalize a matrix list.
    let type_name = &matrix_list.type_name;
    let schema = &matrix_list.schema;

    let mut lines = Vec::new();
    // Use inline schema for canonical form
    lines.push(format!("{}{}:@{}[{}]", " ".repeat(indent), key, type_name, schema.join(", ")));

    // Output rows with ditto optimization
    let mut last_values: Option<Vec<Value>> = None;
    for row in &matrix_list.rows {
        // row is dict mapping column name to value
        let row_values: Vec<Value> = schema.iter().map(|col| row[col].clone()).collect();
        let canonical_row = canonicalize_matrix_row(&row_values, last_values.as_ref());
        lines.push(format!("{}  {}", " ".repeat(indent), canonical_row));
        last_values = Some(row_values);
    }

    lines.join("\n")
}

fn canonicalize_matrix_row(values: &[Value], last_values: Option<&Vec<Value>>) -> String {
    /// Canonicalize a single matrix row with ditto optimization.
    let mut cells = Vec::new();
    for (i, value) in values.iter().enumerate() {
        if let Some(last) = last_values {
            if value == &last[i] {
                cells.push("^".to_string());
            } else {
                cells.push(canonicalize_value(value));
            }
        } else {
            cells.push(canonicalize_value(value));
        }
    }
    format!("| {}", cells.join(", "))
}
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
```rust
struct SafeHEDLParser {
    parser: HEDLParser,
    max_indent: usize,
    max_line_length: usize,
    max_nodes: usize,
    max_file_size: usize,
    node_count: usize,
}

impl SafeHEDLParser {
    fn new(max_indent: usize, max_line_length: usize, max_nodes: usize, max_file_size: usize) -> Self {
        SafeHEDLParser {
            parser: HEDLParser::new(true, max_indent, max_nodes),
            max_indent,
            max_line_length,
            max_nodes,
            max_file_size,
            node_count: 0,
        }
    }

    fn validate_line(&self, line: &str) -> Result<(), SecurityError> {
        if line.len() > self.max_line_length {
            return Err(SecurityError::new(&format!("Line too long: {} > {}", line.len(), self.max_line_length)));
        }
        Ok(())
    }

    fn validate_indent(&self, indent: usize) -> Result<(), SecurityError> {
        if indent > self.max_indent {
            return Err(SecurityError::new(&format!("Indent too deep: {} > {}", indent, self.max_indent)));
        }
        Ok(())
    }

    fn register_node(&mut self, node_id: &str, type_name: &str) -> Result<(), SecurityError> {
        self.node_count += 1;
        if self.node_count > self.max_nodes {
            return Err(SecurityError::new(&format!("Too many nodes: {} > {}", self.node_count, self.max_nodes)));
        }
        Ok(())
    }
}
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
```rust
fn detect_truncation(lines: &[String]) -> bool {
    /// Check for truncation indicators using scan_regions.
    if lines.is_empty() {
        return false;
    }

    let last_line = lines.last().unwrap().trim_end_matches('\n');

    // Check for partial separator
    if last_line.starts_with('-') && last_line != "---" {
        return true;
    }

    // Check for unterminated tokens using normative scan_regions
    let regions = scan_regions(last_line);
    let bytes = last_line.as_bytes();
    for region in &regions {
        if region.end == bytes.len() {
            match region.region_type {
                RegionType::Quote if bytes[region.end - 1] != b'"' => return true,
                RegionType::Expression if bytes[region.end - 1] != b')' => return true,
                _ => {}
            }
        }
    }

    false
}
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
%V:2.0
---
config:
  database:
    host: localhost
    port: 5432
  logging:
    level: info
    file: "/var/log/app.log"
# Lists require inline schema in Simple Mode (first column is always ID)
users:@User[id,name,email,active]
 |alice,Alice,alice@example.com,true
 |bob,Bob,bob@example.com,false
```

### 16.2 Basic Typed List

```hedl
%V:2.0
%S:User:[id,name,email,active]
---
users:@User
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
%V:2.0
%S:Project:[id,name]
%S:Task:[id,description,status]
%N:Project>Task
---
projects:@Project
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
%V:2.0
%A:%pending: "pending"
%A:%done: "done"
%S:Task:[id,description,status,depends_on]
---
tasks:@Task
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
%V:2.0
%S:Measurement:[id,timestamp,values]
---
experiment:
  name: "Temperature Test"
  metadata:
    sensor_count: 3
    duration: 3600
  measurements:@Measurement
  |m1,1625097600,[23.5,24.1,22.9]
  |m2,1625097660,[23.7,24.0,23.1]
  |m3,^,[23.6,24.0,23.0]
```

**Note**: Ditto (`^`) works at the cell level, not inside tensor literals. In row m3, `^` copies the entire timestamp value `1625097660` from row m2. Tensor literals must contain only numbers.

### 16.6 Type-Scoped IDs Example

```hedl
%V:2.0
%S:User:[id,name]
%S:Role:[id,name]
---
users:@User
 |admin,Alice
 |user1,Bob
roles:@Role
 |admin,Administrator  # OK - different type namespace
 |user,Regular User
```

### 16.7 Comprehensive Features Example

This example demonstrates v2.0 features working together.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%MODE: strict
%A: dept: "Engineering"
%S:Employee:[id, name, department, salary, active]
%C:Employee:100
%PROMPT: "Reference employees by ID. Do not invent data."
---
employees:@Employee
 |e1, Alice, Engineering, 95000, true
 |e2, Bob, Engineering, 65000, true
 |e3, Carol, Marketing, 72000, false
```

**Parsed Structure**:
```json
{
  "employees": [
    {"id": "e1", "name": "Alice", "department": "Engineering", "salary": 95000, "active": true},
    {"id": "e2", "name": "Bob", "department": "Engineering", "salary": 65000, "active": true},
    {"id": "e3", "name": "Carol", "department": "Marketing", "salary": 72000, "active": false}
  ],
  "_metadata": {
    "prompts": ["Reference employees by ID. Do not invent data."]
  }
}
```

**Features Demonstrated**:
1. **%V:2.0**: Required version directive in compact form
2. **%NULL:~**: Required null symbol declaration
3. **%QUOTE:"**: Required quote character declaration
4. **%MODE: strict**: Explicit mode setting
5. **%A (alias)**: Global constant for reuse
6. **%S (struct)**: Compact schema definition
7. **%C (count)**: Capacity hint for implementations
8. **%PROMPT**: Metadata stored for LLM consumption

### 16.8 Lenient Mode Example

This example shows how lenient mode handles unknown values gracefully.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%MODE: lenient
%S:Employee:[id, name, status, salary]
---
employees:@Employee
 |e1, Alice, active, 50000
 |e2, Bob, active, 75000
 |e3, Carol, inactive, 0
```

### 16.9 Compact v1.2 Example

This example demonstrates v1.2 compact syntax with inline child lists.

```hedl
%V:1.2
%NULL:~
%QUOTE:"
%S:Product:[id,sku,name,category,price]
%S:Review:[id,rating,text,verified]
%S:Inventory:[id,warehouse,qty]
%C:Product.total=3
%C:Product.category:electronics=2,home=1
%C:Review.total=6
%C:Inventory.total=6
%N:Product>Review
%N:Product>Inventory
---
products:@Product
 |prod-001,SKU-001,Laptop,electronics,999.99
    @Review#2:|rev-001,5,Great product,true|rev-002,4,Good value,^
    @Inventory#2:|inv-001,east,50|inv-002,west,30
 |prod-002,SKU-002,Monitor,^,299.99
    @Review#3:|rev-003,5,Crystal clear,true|rev-004,^,Excellent,^|rev-005,3,Average,false
    @Inventory#2:|inv-003,east,100|inv-004,west,^
 |prod-003,SKU-003,Smart Speaker,home,79.99
    @Review#1:|rev-006,4,Good sound,true
    @Inventory#2:|inv-005,east,200|inv-006,west,150
```

**Demonstrates**:
1. **%V**: Compact version directive
2. **%NULL**: Explicit null symbol declaration
3. **%QUOTE**: Explicit quote character declaration
4. **%S**: Compact schema definitions
5. **%C**: Count statistics for validation hints
6. **%N**: Compact nesting relationships
7. **Inline children**: `@Type#N:|row1|row2|...` syntax (max 5 entries per inline list)
8. **Ditto in inline**: `^` works within inline children
9. **No spaces after pipes**: Inline format uses `|data` not `| data`
10. **Proper indentation**: Child declarations indented one level deeper than parent rows

**Token Savings vs v1.1**:
- ~15% reduction in header tokens
- ~30% reduction in body tokens with inline children
- Overall ~25% token savings for hierarchical data

---

## 17. Extensions and Versioning

### 17.1 Versioning Scheme

**Format**: `major.minor`

- **Major version**: Breaking changes
- **Minor version**: Backward-compatible additions

**Current Version**: 2.0

**Version History**:
- **v1.0**: Initial release with core features
- **v1.1**: Added %MODE, %PROMPT, list literals (note: %ENUM, %DICT, %CONSTRAINT were proposed but removed in v2.0)
- **v1.2**: Compact directive syntax (%V, %S, %N, %C), inline child lists (max 5 entries), %NULL, %QUOTE
- **v2.0**: Required headers (%V:2.0, %NULL:~, %QUOTE:"), 1-space indentation, no ditto operator

**Backward Compatibility**:
- v1.2 parser MUST parse v1.0 and v1.1 files (verbose syntax supported)
- v1.1 parser MUST parse v1.0 files
- v1.1 parser CANNOT parse v1.2 compact syntax (will reject)
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

### 17.4 Forward Compatibility: Experimental Directives (v1.1)

Directives starting with `%X-` are designated as **EXPERIMENTAL** and enable forward compatibility.

**Parser Behavior for `%X-*` Directives**:
1. Parsers MUST accept any directive starting with `%X-` without raising a hard error
2. Parsers MUST emit a WARNING when encountering an unknown `%X-*` directive
3. Parsers SHOULD store the directive's content as opaque metadata
4. The stored metadata SHOULD include:
   - The full directive name (e.g., `X-MAXROWS`)
   - The payload string (unparsed)
   - The line number
5. Unknown `%X-*` directives MUST NOT affect parsing semantics

**Standard (Non-Experimental) Directives**:
- Unknown directives NOT starting with `%X-` remain hard errors
- This preserves strict validation while enabling experimentation

**Examples**:
```hedl
%V:2.0
%X-MAXROWS: User: 1000    # Experimental - accepted with warning
%X-CUSTOM: some payload   # Experimental - accepted with warning
%UNKNOWN: foo             # ERROR: Unknown directive (not experimental)
---
```

**Rationale**: This mechanism allows:
- Implementers to experiment with new features
- Documents to include hints for future parser versions
- Graceful degradation when older parsers encounter newer features

**Warning Message Format** (RECOMMENDED):
```
Warning at line N: Unknown experimental directive '%X-NAME' ignored
```

### 17.5 Version Negotiation

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
5. Validate indentation (1 space per level in v2.0, no tabs for indentation)
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

1. **Minimal Document**: `%V:2.0\n---\n`
2. **Simple Object**: `%V:2.0\n---\na: 1\nb: 2`
3. **Nested Object**: `%V:2.0\n---\na:\n  b: 1`
4. **Matrix List**: `%V:2.0\n%S:T:[id,v]\n---\nd:@T\n  |x,1`
5. **References**: `%V:2.0\n%S:T:[id,ref]\n---\nd:@T\n  |a,~\n  |b,@a`
6. **Type-Scoped IDs**: `%V:2.0\n%S:A:[id,v]\n%S:B:[id,v]\n---\na:@A\n  |x,1\nb:@B\n  |x,2` (should NOT error)

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
%V:2.0
---
```

For forward compatibility:
1. Use the earliest version that supports needed features
2. Avoid experimental extensions unless agreed
3. Document any non-standard usage

---

## Appendix A: Implementation Guidelines

### A.1 Recommended Pseudo-Code Structure

```rust
struct HEDLParser {
    strict: bool,
    max_indent: usize,
    registries: HashMap<String, Value>,
    type_registries: HashMap<String, HashMap<String, Node>>,
    references: Vec<Reference>,
}

impl HEDLParser {
    fn new(strict: bool, max_indent: usize) -> Self {
        HEDLParser {
            strict,
            max_indent,
            registries: HashMap::new(),
            type_registries: HashMap::new(),
            references: Vec::new(),
        }
    }

    fn parse(&mut self, input_text: &str) -> Result<Node, HEDLError> {
        // Phase 1: Preprocessing
        let lines = self.normalize_lines(input_text)?;

        // Phase 2: Header parsing
        let (header_lines, body_lines) = self.split_sections(&lines)?;
        self.parse_header(&header_lines)?;

        // Phase 3: Body parsing
        let root = self.parse_body(&body_lines)?;

        // Phase 4: Reference resolution
        self.resolve_references()?;

        Ok(root)
    }

    fn parse_body(&mut self, lines: &[String]) -> Result<Node, HEDLError> {
        let mut stack = vec![RootFrame::new()];
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            if self.is_blank(line) || self.is_comment(line) {
                continue;
            }

            let indent = self.calculate_indent(line)?;
            self.validate_indent(indent)?;

            // Scope closing
            self.pop_frames(&mut stack, indent);

            // Parse based on current top frame
            self.parse_line(&mut stack, line, indent)?;
        }
        Ok(stack[0].object.clone())
    }
}
```

### A.2 Handling ID Column Validation

```rust
fn parse_matrix_cell(&self, cell_data: (&str, bool), column_index: usize, schema: &[String], last_row_values: Option<&[Value]>) -> Result<Value, SemanticError> {
    /// Parse a single matrix cell with special handling for ID column.
    let (value_str, is_quoted) = cell_data;

    // Special handling for ID column (first column)
    if column_index == 0 {
        if !is_quoted {
            if value_str == "^" {
                return Err(SemanticError::new("Ditto not permitted in ID column"));
            }
            if value_str == "~" {
                return Err(SemanticError::new("Null not permitted in ID column"));
            }
        }
    }

    // Apply normal inference ladder
    let value = if is_quoted {
        Value::String(value_str.to_string())
    } else {
        self.infer_value(value_str, last_row_values)?
    };

    // Validate ID column
    if column_index == 0 {
        if let Value::String(id) = &value {
            // Validate ID token pattern (lowercase or underscore start)
            let id_pattern = Regex::new(r"^[a-z_][a-z0-9_\-]*$").unwrap();
            if !id_pattern.is_match(id) {
                return Err(SemanticError::new(&format!("Invalid ID format: {}", id)));
            }
        } else {
            return Err(SemanticError::new(&format!("ID must be string, got {}", value.type_name())));
        }
    }

    Ok(value)
}
```

### A.3 Matrix Row Comment Stripping

```rust
fn strip_matrix_row_comment(&self, line: &str) -> String {
    /// Strip comment from matrix row line, preserving CSV-style quoted fields.
    let mut in_quotes = false;
    let mut i = 0;
    let mut result = String::new();
    let chars: Vec<char> = line.chars().collect();

    while i < chars.len() {
        let ch = chars[i];

        if !in_quotes && ch == '#' {
            // Found comment start outside quotes
            break;
        }

        if ch == '"' {
            // Check for escaped quote ""
            if i + 1 < chars.len() && chars[i + 1] == '"' {
                result.push_str("\"\"");
                i += 2;
                continue;
            } else {
                in_quotes = !in_quotes;
                result.push('"');
            }
        } else {
            result.push(ch);
        }

        i += 1;
    }

    result.trim_end().to_string()
}
```

### A.4 Streaming Parser Architecture

For large files, implement streaming:

```rust
struct StreamingHEDLParser {
    state: String,
    stack: Vec<Frame>,
    registries: HashMap<String, Value>,
}

impl StreamingHEDLParser {
    fn new() -> Self {
        StreamingHEDLParser {
            state: "HEADER".to_string(),
            stack: Vec::new(),
            registries: HashMap::new(),
        }
    }

    fn feed(&mut self, line: &str) -> Result<(), HEDLError> {
        /// Process a single line.
        if self.state == "HEADER" {
            if line.trim() == "---" {
                self.state = "BODY".to_string();
            } else {
                self.parse_header_line(line)?;
            }
        } else {
            self.parse_body_line(line)?;
        }
        Ok(())
    }

    fn parse_body_line(&mut self, line: &str) -> Result<(), HEDLError> {
        // Similar to parse_body but stateful
        let indent = self.calculate_indent(line)?;
        self.pop_frames(indent);

        if line.trim_start().starts_with('|') {
            self.parse_matrix_row(line, indent)?;
        } else {
            self.parse_object_line(line, indent)?;
        }
        Ok(())
    }

    fn get_result(&self) -> Node {
        /// Get parsed result after all lines processed.
        self.stack[0].object.clone()
    }
}
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
11. **Nest to undefined**: `%N:A>B` where B undefined → Schema Error

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
35. **Only Header + Separator**: `%V:2.0\n---\n` → Success (empty root object)
36. **Maximum Nesting**: 50 levels deep → Success (or configured limit)
37. **Empty Matrix**: List with no rows → Success (empty list)
38. **Object Start with Comment**: `key: # comment` → Object Start (comment stripped)
39. **Empty Alias**: `%A:%empty: ""` → Success, expands to empty string
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
%V:2.0
%A:%true: "true"
%S:Test:[id,value,ref]
%S:Child:[id,data]
%N:Test>Child
---
tests:@Test
 |t1,"simple",~
  |c1,child
 |t2,42,@t1
  |c2,child
 |t3,%true,@t2
 |t4,^,^
tensor_test:@TensorTest[id,data]
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
%V:2.0
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
%V:2.0
%S:User:[id,name]
%S:Post:[id,text]
%N:User>Post
---
users:@User
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
%V:2.0
---
config:
  database:
    host: localhost
    port: 5432
  servers:@Server[name,ip]
  |web1,192.168.1.1
  |web2,192.168.1.2
```

**HEDL Equivalent (Complex Mode)**:
```hedl
%V:2.0
%S:Server:[name,ip]
---
config:
  database:
    host: localhost
    port: 5432
  servers:@Server
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
%V:2.0
%S:User:[id,name]
%S:Post:[id,text,author_id]
---
users:@User
 |u1,Alice
 |bob,Bob  # Note: ID must start with lowercase
posts:@Post
 |p1,Hello,@u1
 |p2,World,@u1
 |p3,Hi,@bob
```

**Conversion Rules**:
1. CSV header row → %STRUCT definition
2. CSV data rows → matrix rows
3. Repeated data → separate tables with references
4. Hierarchical data → use %NEST

### C.4 From v1.1 to v1.2

HEDL v1.2 introduces compact directive syntax and inline child lists. Migration is straightforward:

**Directive Name Changes**:

 |v1.1 | v1.2 | Notes |
|------|------|-------|
 |`%V:2.0` | `%V:1.2` | No space after colon |
 |`%S:Type:[cols]` | `%S:Type:[cols]` | No spaces |
 |`%N:Parent>Child` | `%N:Parent>Child` | No spaces around `>` |

**New v1.2 Directives**:
- `%NULL:~` declares the null symbol (default `~`)
- `%QUOTE:"` declares the quote character (default `"`)
- `%C:Type.field=value` declares count statistics

**Child Row Syntax**:

v1.1 expanded form:
```hedl
%V:2.0
%S:Product:[id, name]
%S:Review:[id, rating]
%N:Product>Review
---
products:@Product
 |prod-001, Laptop
  |rev-001, 5
  |rev-002, 4
```

v1.2 inline form (max 5 children per inline list):
```hedl
%V:1.2
%S:Product:[id,name]
%S:Review:[id,rating]
%N:Product>Review
---
products:@Product
 |prod-001,Laptop
    @Review#2:|rev-001,5|rev-002,4
```

Note: Inline format is limited to 5 children. For more than 5, use expanded form with `@Type#N:` followed by `|` rows on separate lines.

**Migration Script Example**:
```bash
# Convert v1.1 directives to v1.2
sed -i 's/%VERSION: /%V:/g' file.hedl
sed -i 's/%STRUCT: /%S:/g' file.hedl
sed -i 's/%NEST: /%N:/g' file.hedl
sed -i 's/: \[/:[/g' file.hedl
sed -i 's/, /,/g' file.hedl
```

**Backward Compatibility**:
- v1.2 parsers MUST accept v1.1 verbose syntax
- v1.1 parsers will reject v1.2 compact syntax

### C.5 Migration Tools

Implementations SHOULD provide:
1. JSON → HEDL converter
2. HEDL → JSON converter (for compatibility)
3. Schema inference from JSON/CSV
4. ID generation for data without IDs
5. v1.1 → v1.2 format converter

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
```rust
struct OptimizedHEDLParser;

impl OptimizedHEDLParser {
    fn parse_line_fast(&self, line: &str) -> Result<ParsedLine, SyntaxError> {
        // Fast path for common cases
        if line.starts_with("  ") {  // Common indent
            let indent = line.len() - line.trim_start_matches(' ').len();
            if indent & 1 != 0 {  // Check odd (bitwise AND)
                return Err(SyntaxError::new("Odd indentation"));
            }
            let indent_level = indent >> 1;  // Divide by 2

            let content = &line[indent..];
            if content.starts_with('|') {
                return self.parse_matrix_row_fast(content, indent_level);
            }
            // ... other cases
        }
        // ... fallback to slow path
        Ok(ParsedLine::default())
    }
}
```

### D.2 Generator Optimization

**Canonicalization**:
1. **Ditto detection**: Compare with previous row
2. **Quoting decision**: Fast check for special characters
3. **Sorting**: Use stable sort for object keys
4. **Buffer reuse**: For string building

**Memory Efficient Generation**:
```rust
fn generate_canonical(data: &Value, output: &mut impl Write) -> Result<(), std::io::Error> {
    /// Stream canonical HEDL to output.
    if let Value::Object(map) = data {
        let mut keys: Vec<_> = map.keys().collect();
        keys.sort();
        for key in keys {
            let value = &map[key];
            if let Value::MatrixList(matrix_list) = value {
                generate_matrix_list(key, matrix_list, output)?;
            } else {
                generate_scalar(key, value, output)?;
            }
        }
    }
    Ok(())
}
```

### D.3 Large File Handling

**Streaming Parser**:
- Process line by line
- Yield nodes as parsed
- Don't build full tree in memory

**Example**:
```rust
fn parse_stream<R: BufRead>(fileobj: R) -> impl Iterator<Item = Result<Event, HEDLError>> {
    /// Parse HEDL file as stream of events.
    let mut parser = StreamingHEDLParser::new();
    fileobj.lines().flat_map(move |line_result| {
        match line_result {
            Ok(line) => {
                parser.feed(&line).ok();
                parser.get_events()
            }
            Err(e) => vec![Err(HEDLError::from(e))],
        }
    })
}
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
```rust
fn benchmark_parser() {
    // Parse 1000-node document
    let start = Instant::now();
    let result = parser.parse(&large_document).unwrap();
    let elapsed = start.elapsed();

    println!("Parsed {} nodes in {:.3}s", result.len(), elapsed.as_secs_f64());
    println!("Rate: {:.0} nodes/sec", result.len() as f64 / elapsed.as_secs_f64());
}
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
                  | ModeDirective | PromptDirective | ExperimentalDirective
                  | NullDirective | QuoteDirective | CountDirective
Separator       ::= '---' Newline

# Core directives (v1.0/v1.1 verbose form)
VersionDirective ::= '%VERSION:' WS+ Version Newline
                   | '%V:' Version Newline  # Compact form (v1.2)
Version         ::= Digit+ '.' Digit+
StructDirective ::= '%STRUCT:' WS+ TypeName ':' WS+ ColumnList Newline
                  | '%S:' TypeName ':' ColumnListCompact Newline  # Compact form (v1.2)
NestDirective   ::= '%NEST:' WS+ TypeName WS+ '>' WS+ TypeName Newline
                  | '%N:' TypeName '>' TypeName Newline  # Compact form (v1.2)
AliasDirective  ::= '%A:' WS+ AliasKey ':' WS+ QuotedString Newline

# New directives (v1.1)
ModeDirective   ::= '%MODE:' WS+ ('strict' | 'lenient') Newline
PromptDirective ::= '%PROMPT:' WS+ QuotedString Newline
ExperimentalDirective ::= '%X-' [A-Z][A-Z0-9_]* ':' WS+ .* Newline  # Any payload

# New directives (v1.2)
NullDirective   ::= '%NULL:' Char Newline  # Null symbol declaration
QuoteDirective  ::= '%QUOTE:' Char Newline  # Quote character declaration
CountDirective  ::= '%C:' TypeName '.' KeyToken '=' CountValue Newline
                  | '%C:' TypeName '.' KeyToken ':' CountDistribution Newline
CountValue      ::= Digit+
CountDistribution ::= KeyToken '=' Digit+ (',' KeyToken '=' Digit+)*

# Support productions for v1.1 directives
EnumScope       ::= TypeName '.' KeyToken | KeyToken  # Scoped or global
CodeMap         ::= '{' CodeMapping (',' CodeMapping)* '}'
CodeMapping     ::= KeyToken ':' QuotedString
Predicate       ::= RangePred | RegexPred | RefPred | EnumPred
                  | ListPred | LenPred | 'bool' | 'number'
RangePred       ::= 'range(' Number ',' (Number | 'inf') ')'
RegexPred       ::= 'regex(' QuotedString ')'
RefPred         ::= 'ref(' TypeName ')'
EnumPred        ::= 'enum(' EnumScope ')'
ListPred        ::= 'list(' Predicate ')'
LenPred         ::= 'len(' Digit+ ',' (Digit+ | 'inf') ')'

ColumnList      ::= '[' Column (',' WS* Column)* ']'  # Verbose with optional spaces
ColumnListCompact ::= '[' Column (',' Column)* ']'  # Compact without spaces
Column          ::= KeyToken

Body            ::= (Object | KeyValue | MatrixList)*
Object          ::= Indent KeyToken ':' Newline (Object | KeyValue | MatrixList)*
KeyValue        ::= Indent KeyToken ':' WS+ Value Newline
              | Indent KeyToken ':' WS+ BlockString
MatrixList      ::= Indent KeyToken ':' WS* '@' TypeName ColumnList? Newline MatrixRow*
                  | Indent KeyToken ':' '@' TypeName Newline MatrixRow*  # Compact (v1.2)
MatrixRow       ::= Indent '|' CountHint? CSVRow Newline InlineChildList*
CountHint       ::= '[' Digit+ ']' WS*  # Optional count of direct children

# Inline child list syntax (v1.2)
InlineChildList ::= Indent '@' TypeName '#' Digit+ ':' InlineChildRows Newline  # Inline form (max 5 children)
                  | Indent '@' TypeName '#' Digit+ ':' Newline MatrixRow+       # Expanded form (>5 children)
InlineChildRows ::= '|' CSVRow ('|' CSVRow)*                                   # Pipe-separated on same line

Value           ::= Null | Tensor | ListLiteral | Reference | Expression
                  | AliasRef | Boolean | Number | String | QuotedString
Null            ::= '~'
Tensor          ::= '[' (Number | Tensor) (',' (Number | Tensor))* ']'
ListLiteral     ::= '(' ')' | '(' ScalarValue (',' ScalarValue)* ')'  # v1.1
ScalarValue     ::= Null | Reference | Expression | AliasRef | Boolean | Number | String | QuotedString
Reference       ::= '@' (TypeName ':')? IDToken
Expression      ::= '$(' BalancedText ')'
AliasRef        ::= '%' KeyToken
Boolean         ::= 'true' | 'false'
Number          ::= '-'? Digit+ ('.' Digit+)?
String          ::= [^:#@$%~[()\s][^:#@$()\s]*  # Simplified - not starting with special chars
QuotedString    ::= '"' (Char | '""')* '"'   # No escape sequences in key-value context
BlockString     ::= '"""' Newline BlockContent '"""'
BlockContent    ::= (Char | Newline)*        # Raw content, no escape processing

CSVRow          ::= CSVField (',' CSVField)*
CSVField        ::= QuotedCSVField | UnquotedCSVField
QuotedCSVField  ::= '"' (Char | '""' | EscapeSeq)* '"'
UnquotedCSVField ::= [^,#\n\r]*  # Cannot contain comma, hash, newline, CR

# Escape sequences (only valid in quoted CSV fields)
EscapeSeq       ::= '\n' | '\t' | '\r' | '\\' | '\"'

Indent          ::= Space*  # 1 space per indent level (v2.0)

# Tokens
TypeName        ::= [A-Z][A-Za-z0-9]*
KeyToken        ::= [a-z_][a-z0-9_]*
IDToken         ::= [a-z_][a-z0-9_\-]*  # ASCII-only in v1.0/v1.1
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
3. **Line continuations**: Not supported in v1.0/v1.1
4. **Unicode**: Structural tokens are strictly ASCII-only. Data values (strings, comments, tensor numbers) allow Unicode.
5. **BalancedText**: Defined by balanced-parentheses algorithm in Section 4.6.6
6. **Block Strings**: Triple-quoted strings (`"""`) for multiline key-value content. No escape processing; all content is literal. See Section 8.1.2.
7. **Escape Sequences**: Only processed in quoted CSV fields (matrix cells). Key-value strings treat backslash literally. See Section 9.2.
8. **List Literals** (v1.1): Parenthesized sequences `(elem, ...)` distinct from tensor brackets `[...]`. See Section 4.6.9.
9. **Experimental Directives** (v1.1): `%X-*` directives are accepted with warning; payload is opaque. See Section 17.4.
8. **Count Hints**: Optional metadata in matrix rows indicating number of direct children. Format: `[N]` where N is a non-negative integer. See Section 9.6.

### E.3 Grammar Validation

This grammar is:
- **LL(1)**: Can be parsed with one-token lookahead
- **Deterministic**: No ambiguous constructs
- **Complete**: Covers all valid HEDL 1.0 documents
- **Unambiguous**: Each valid document has one parse tree

### E.4 Grammar Implementation

Example recursive descent parser skeleton:

```rust
struct GrammarParser {
    tokens: Vec<Token>,
    position: usize,
}

impl GrammarParser {
    fn parse_document(&mut self) -> Result<Node, HEDLError> {
        self.parse_header()?;
        self.expect("---")?;
        self.parse_body()
    }

    fn parse_header(&mut self) -> Result<(), HEDLError> {
        while self.peek() != Some("---") {
            if let Some(token) = self.peek() {
                if token.starts_with("%VERSION") {
                    self.parse_version()?;
                } else if token.starts_with("%STRUCT") {
                    self.parse_struct()?;
                }
                // ... other directives
            }
        }
        Ok(())
    }

    fn parse_body(&mut self) -> Result<Node, HEDLError> {
        while !self.eof() {
            let indent = self.parse_indent()?;
            if let Some(token) = self.peek() {
                if token.ends_with(':') {
                    let key = self.parse_key()?;
                    self.expect(":")?;
                    if let Some(next) = self.peek() {
                        if next.starts_with('@') {
                            self.parse_matrix_list(&key, indent)?;
                        } else if next.trim().is_empty() {  // Next line has content
                            self.parse_key_value(&key, indent)?;
                        } else {  // Object start
                            self.parse_object(&key, indent)?;
                        }
                    }
                } else if token.starts_with('|') {
                    self.parse_matrix_row(indent)?;
                }
            }
        }
        Ok(Node::default())
    }

    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.position).map(|t| t.as_str())
    }

    fn eof(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn expect(&mut self, expected: &str) -> Result<(), HEDLError> {
        if self.peek() == Some(expected) {
            self.position += 1;
            Ok(())
        } else {
            Err(SyntaxError::new(&format!("Expected '{}', got '{:?}'", expected, self.peek())))
        }
    }
}
```

---

## Appendix F: Frequently Asked Questions

### F.1 Why type-scoped IDs instead of global?

**Answer**: Type-scoped IDs enable:
- Modular data composition (safe file concatenation)
- Natural naming (`user:admin`, `role:admin`)
- Simpler reference resolution within same type
- **Cross-type references** via qualified syntax (`@Type:id`)

### F.2 Why ASCII-only IDs in v1.0/v1.1?

**Answer**: ASCII-only ensures:
- Consistent reference resolution across platforms
- Simpler implementation for v1.0
- Clear migration path to Unicode in future versions
- Interoperability with existing systems

### F.3 Why no optional columns?

**Answer**: Simplicity and performance. Fixed schemas enable:
- Faster parsing (no per-row column count checks)
- Clearer data shape
- Simpler tooling and validation
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
%V:2.0
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
%V:2.0
---
users:@User[id,name,email]
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

 |Feature | HEDL | TOON | JSON | YAML | CSV | Protobuf |
|---------|------|------|------|------|-----|----------|
 |Human Readable | ✓ Yes | ✓ Yes | ✓ Yes | ✓ Yes | ○ Limited | ✗ No |
 |Token Efficient | ★ Excellent | ★ Excellent | ✗ Poor | ○ Fair | ○ Good | N/A (binary) |
 |LLM Accuracy | ★ 63-71% (near-JSON) | 61-71% | 68-73% (baseline) | 68-73% | 27-31% | N/A |
 |Graph Support | ★ Native | ✗ No | ○ Manual | ○ Manual | ✗ No | ✗ No |
 |Schema Support | ★ Built-in | ○ Inline | ○ External | ○ External | ○ Header | ★ Required |
 |Streaming | ✓ Yes | ✓ Yes | ✓ Yes | ○ Limited | ✓ Yes | ✓ Yes |
 |Ditto Markers | ★ Native (`^`) | ✗ No | ✗ No | ✗ No | ✗ No | ✗ No |
 |References | ★ `@id` syntax | ✗ No | ✗ Manual | ✗ Manual | ✗ No | ✗ No |

### H.2 HEDL's Unique Features

HEDL provides features that other token-efficient formats (like TOON) lack. These features cover the majority of real-world use cases.

**HEDL Advantages** (benchmarked with cl100k_base tokenizer):

 |Comparison | HEDL |
|------------|------|
 |vs JSON | **56% token savings** |
 |Graph references | `@id` syntax saves **51.5%** vs duplicating entities |
 |Schema reuse | `%STRUCT` definitions shared across files |

Some tabular-only formats (like TOON) are marginally more efficient on pure flat data without relationships, but lack the features above.

**LLM Accuracy**: HEDL achieves 80.4% accuracy across providers (+10.3pp vs JSON, +12.2pp vs TOON). Tested on 571 questions across 7 datasets with DeepSeek, Mistral, and NVIDIA GLM-4.7.

**Feature Comparison**:

 |Aspect | HEDL | Tabular-Only Formats |
|--------|------|----------------------|
 |Data Model | Extended (graph semantics) | Tree-based |
 |References | `@id` native syntax (**51% savings**) | Not supported |
 |Global Aliases | `%ALIAS` directive | Not supported |
 |Schema Declaration | `@Type[...]` inline or `%STRUCT` | Inline only |
 |Array Length | Implicit (auto-counted) | Often required |

**HEDL is the right choice for**:
- Graph relationships (`@author` references save 51% tokens)
- Reusable schemas across files (`%STRUCT` definitions)
- No manual array length counting needed
- Most real-world datasets with references and repetition

### H.3 HEDL vs JSON

**Token Savings**: HEDL typically achieves ~56% token reduction compared to JSON.

 |Dataset Type | HEDL vs JSON Savings |
|--------------|---------------------|
 |Average across datasets | 56% |
 |Flat lists (users, events) | 50-55% |
 |Nested hierarchies (org charts) | 60-70% |
 |Cross-references (knowledge graphs) | 55-60% |
 |Time-series (metrics) | 53-60% |

**LLM Accuracy**: 80.4% average across providers (+10.3pp vs JSON, +12.2pp vs TOON). Tested on 571 questions across 7 datasets with DeepSeek, Mistral, and NVIDIA GLM-4.7.

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

**End of HEDL Specification v2.0**