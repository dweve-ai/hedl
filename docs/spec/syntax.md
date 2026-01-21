# HEDL Syntax Specification

This document provides a formal specification of the HEDL (Hierarchical Entity Data Language) syntax based on the reference parser implementation in `crates/hedl-core`.

## Table of Contents

1. [Document Structure](#document-structure)
2. [Lexical Elements](#lexical-elements)
3. [Header Section](#header-section)
4. [Body Section](#body-section)
5. [Values](#values)
6. [Matrix Lists](#matrix-lists)
7. [Comments](#comments)
8. [Indentation](#indentation)
9. [Security Limits](#security-limits)
10. [Grammar](#grammar)

## Document Structure

A HEDL document consists of three parts:

```
HEADER
---
BODY
```

1. **Header**: Contains directives (version, struct definitions, aliases, nest relationships)
2. **Separator**: Exactly three hyphens `---` on a line by itself
3. **Body**: Contains the actual data in key-value and matrix list format

### Example

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
%ALIAS: %active: "true"
---
users: @User
  | user_1, Alice, alice@example.com
  | user_2, Bob, bob@example.com
```

## Lexical Elements

### Token Types

#### Key Token

**Pattern**: `[a-z_][a-z0-9_]*`

Valid key tokens must:
- Start with a lowercase letter or underscore
- Contain only lowercase letters, digits, and underscores
- Be used for field names and column names

**Valid examples**:
```hedl
name
user_id
_private
item123
```

**Invalid examples**:
```hedl
Name        # No uppercase
123item     # No leading digit
my-key      # No hyphens
```

#### Type Name

**Pattern**: `[A-Z][A-Za-z0-9]*`

Valid type names must:
- Start with an uppercase letter
- Contain only alphanumeric characters (no underscores or hyphens)
- Be used for entity type names

**Valid examples**:
```hedl
User
Post123
MyType
```

**Invalid examples**:
```hedl
user         # Must start uppercase
User_Type    # No underscores
123User      # No leading digit
```

#### ID Token

**Pattern**: `[a-zA-Z_][a-zA-Z0-9_\-]*`

Valid ID tokens must:
- Start with any letter (upper or lower) or underscore
- Contain letters, digits, underscores, or hyphens
- Be used for entity identifiers

**Valid examples**:
```hedl
user_1
item-two
SKU-4020
ABC-DEF-001
```

**Invalid examples**:
```hedl
123item     # No leading digit
-item       # No leading hyphen
id.name     # No dots
```

## Header Section

The header contains directives that define the document structure.

### %VERSION Directive

**Syntax**: `%VERSION: <version>`

Specifies the HEDL format version. Currently only version `1.0` is supported.

**Example**:
```hedl
%VERSION: 1.0
```

### %STRUCT Directive

**Syntax**: `%STRUCT: <TypeName>: [<col1>, <col2>, ...]`

Defines the schema for a matrix list type.

**Rules**:
- TypeName must be a valid Type Name token
- Column names must be valid Key tokens
- At least one column is required
- No duplicate column names
- Maximum columns limited by `max_columns` (default: 100)

**Example**:
```hedl
%STRUCT: User: [id, name, email]
%STRUCT: Post: [id, title, author_id, content]
```

### %ALIAS Directive

**Syntax**: `%ALIAS: %<key>: "<value>"`

Defines a constant that can be referenced later using `%key`.

**Rules**:
- Key must be a valid Key token (after the % prefix)
- Value must be a quoted string
- Escape sequences supported in value (see [Escape Sequences](#escape-sequences))
- Maximum aliases limited by `max_aliases` (default: 10,000)

**Example**:
```hedl
%ALIAS: %active: "true"
%ALIAS: %default_email: "user@example.com"
%ALIAS: %greeting: "Hello \"World\""
```

### %NEST Directive

**Syntax**: `%NEST: <ParentType> > <ChildType>`

Defines a parent-child relationship between two entity types.

**Rules**:
- Both ParentType and ChildType must be valid Type Names
- Both types must be defined in %STRUCT directives
- Defines that ChildType rows can be nested under ParentType rows
- Maximum nest depth limited by `max_nest_depth` (default: 100)

**Example**:
```hedl
%STRUCT: Company: [id, name]
%STRUCT: Division: [id, name]
%NEST: Company > Division
```

## Body Section

The body contains the actual data in hierarchical format.

### Objects

**Syntax**:
```hedl
<key>:
  <nested content>
```

Objects are defined by a key followed by a colon with no value, and contain nested key-value pairs or other structures indented by 2 spaces.

**Example**:
```hedl
user:
  name: Alice
  email: alice@example.com
  settings:
    theme: dark
    notifications: true
```

### Scalar Key-Value Pairs

**Syntax**: `<key>: <value>`

**Rules**:
- Must have exactly one space after the colon
- Key must be a valid Key token
- Value can be any valid value type (see [Values](#values))
- No duplicate keys at the same nesting level
- Maximum keys per object limited by `max_object_keys` (default: 10,000)
- Maximum total keys across all objects limited by `max_total_keys` (default: 10,000,000)

**Example**:
```hedl
name: Alice
age: 30
active: true
score: 95.5
```

### Block Strings

**Syntax**:
```hedl
<key>: |
  line 1
  line 2
  line 3
<key>: |>
```

Block strings are multi-line string values. Two syntaxes are supported:

1. **Multi-line block**: `key: |` followed by indented content
2. **Single-line terminator**: `key: |>` for empty strings

**Rules**:
- Lines are collected until indentation decreases
- Leading/trailing blank lines are removed
- Common indentation is stripped
- Maximum size limited by `max_block_string_size` (default: 10MB)

**Example**:
```hedl
description: |
  This is a multi-line
  description that spans
  several lines.
empty: |>
```

## Values

### Null

**Syntax**: `null`

Represents the absence of a value.

**Example**:
```hedl
optional_field: null
```

### Boolean

**Syntax**: `true` or `false`

Boolean values are case-sensitive.

**Example**:
```hedl
active: true
deleted: false
```

### Integer

**Syntax**: `-?[0-9]+`

Integers are decimal numbers without fractional parts.

**Range**: -2^63 to 2^63-1 (i64)

**Example**:
```hedl
count: 42
offset: -10
zero: 0
```

### Float

**Syntax**: `-?[0-9]+\.[0-9]+`

Floating-point numbers must include a decimal point.

**Example**:
```hedl
temperature: 98.6
latitude: -122.4194
pi: 3.14159
```

**Note**: NaN and Infinity are not allowed.

### String

Strings can be unquoted or quoted.

#### Unquoted String

Simple strings without special characters can be unquoted.

**Example**:
```hedl
name: Alice
city: NewYork
```

#### Quoted String

**Syntax**: `"<content>"`

Quoted strings support escape sequences and can contain any characters.

**Example**:
```hedl
message: "Hello, World!"
path: "C:\\Users\\Alice"
quote: "She said \"hello\""
```

### Escape Sequences

Escape sequences are supported in quoted strings (directives and values):

| Escape | Meaning |
|--------|---------|
| `""` | Literal quote (CSV-style) |
| `\"` | Literal quote (backslash-style) |
| `\\` | Literal backslash |
| `\n` | Newline |
| `\t` | Tab |
| `\r` | Carriage return |

**Example**:
```hedl
csv_style: "say ""hello"""
backslash_style: "say \"hello\""
multiline: "line1\nline2"
tab_separated: "col1\tcol2"
windows_path: "C:\\Users\\test"
```

Unknown escape sequences preserve the backslash:
```hedl
unknown: "test\x"  # Results in: test\x
```

### Tensor Literals

**Syntax**: `[<values>]` or `[[<values>], ...]`

Tensors are multi-dimensional numerical arrays.

**Rules**:
- Must start with `[` and end with `]`
- Elements separated by commas
- Can be nested for multi-dimensional arrays
- All dimensions must be consistent
- Cannot be empty
- Maximum recursion depth: 100
- Maximum total elements: 10,000,000
- NaN and Infinity not allowed

**Examples**:
```hedl
# 1D tensor
vector: [1, 2, 3]

# 2D tensor (matrix)
matrix: [[1, 2], [3, 4]]

# With floats
floats: [1.5, 2.5, 3.5]

# With negatives
negatives: [-1, -2, -3]

# Trailing comma allowed
trailing: [1, 2, 3,]
```

**Invalid examples**:
```hedl
empty: []                    # Empty not allowed
inconsistent: [[1, 2], [3]]  # Inconsistent dimensions
```

### References

**Syntax**: `@<id>` or `@<TypeName>:<id>`

References point to entities defined elsewhere in the document.

**Formats**:
- Local reference: `@<id>` (references any entity with that ID)
- Qualified reference: `@<TypeName>:<id>` (references specific type)

**Rules**:
- ID must be a valid ID token
- TypeName (if present) must be a valid Type Name
- Ambiguous references (multiple entities with same ID) always error
- Unresolved references error in strict mode, ignored in lenient mode

**Example**:
```hedl
# Local reference
author: @user_1

# Qualified reference
author: @User:user_1
```

### Expressions

**Syntax**: `$(<expression>)`

Expressions are evaluated at parse time (or later, depending on implementation).

**Grammar**:
```
expr     = call | access | atom
call     = identifier "(" args ")"
access   = expr "." identifier
atom     = identifier | literal
args     = (expr ("," expr)*)?
literal  = number | string | bool
```

**Supported operations**:
- Function calls: `$(now())`
- Field access: `$(user.name)`
- Nested calls: `$(outer(inner(x)))`
- String literals: `$(concat("a", "b"))`
- Numeric literals: `$(add(1, 2.5))`
- Boolean literals: `$(and(true, false))`

**Example**:
```hedl
timestamp: $(now())
full_name: $(concat(first_name, " ", last_name))
user_name: $(user.profile.name)
result: $(calculate(x, y, 42))
```

**Note**: The actual available functions depend on the evaluation context provided by the implementation.

### Alias References

**Syntax**: `%<key>`

References an alias defined in the header.

**Example**:
```hedl
# In header:
%ALIAS: %active: "true"

# In body:
status: %active
```

## Matrix Lists

Matrix lists are tables of structured entities defined by a schema.

### List Declaration

**Syntax**: `<key>: @<TypeName>` or `<key>: @<TypeName>[<schema>]`

**Formats**:
1. Reference to declared schema: `users: @User`
2. Inline schema: `users: @User[id, name, email]`

**Rules**:
- TypeName must be defined in a %STRUCT directive (format 1)
- Inline schema must match declared schema if both exist
- Inline schema follows same rules as %STRUCT columns

**Optional Count Hint (DEPRECATED)**:
```hedl
users(3): @User
```
The `name(N): @Type` syntax for count hints is deprecated. Use the row-level `|[N]|` syntax instead.

**Example**:
```hedl
# Reference to schema
users: @User
  | user_1, Alice, alice@example.com

# Inline schema
users: @User[id, name, email]
  | user_1, Alice, alice@example.com
```

### Matrix Rows

**Syntax**: `| <csv-values>` or `|[<N>] <csv-values>`

Matrix rows are indented 2 spaces under the list declaration.

**Formats**:
1. Leaf row: `| value1, value2, value3`
2. Parent row with child count: `|[N] value1, value2, value3`

**Rules**:
- Must start with `|` (pipe character)
- Values are CSV-formatted
- Number of values must match schema length
- First column is the ID (must be a string)
- Values can be quoted or unquoted
- Ditto marks (`"`) repeat the value from the previous row in that column
- Child count `[N]` indicates this row has N child rows
- Child count `[0]` indicates a parent row with no children
- Children are indented +2 spaces and must have a NEST relationship defined

**Example**:
```hedl
users: @User[id, name, active]
  | user_1, Alice, true
  | user_2, Bob, "
  | user_3, Charlie, false
```

### Ditto Marks

**Syntax**: `"`

The ditto mark (`"`) in a matrix row cell repeats the value from the same column in the previous row.

**Rules**:
- Only valid in matrix rows, not in the first row
- Cannot be used in the ID column (first column)
- Copies the exact value from the previous row

**Example**:
```hedl
products: @Product[id, category, price]
  | prod_1, Electronics, 99.99
  | prod_2, ", 149.99      # category = Electronics
  | prod_3, ", "           # category = Electronics, price = 149.99
```

### Nested Lists (NEST)

Child entities can be nested under parent rows when a NEST relationship is defined.

**Syntax**:
```hedl
%NEST: <ParentType> > <ChildType>

<parent_list>: @ParentType
  |[<child_count>] <parent_values>
    |<child_values>
    |<child_values>
```

**Rules**:
- NEST relationship must be defined in header
- Child rows are indented +2 spaces relative to parent row
- Child count `[N]` in parent row indicates number of children
- Child count is optional (can be inferred from actual children)
- Maximum nest depth limited by `max_nest_depth` (default: 100)

**Example**:
```hedl
%STRUCT: Company: [id, name]
%STRUCT: Division: [id, name]
%NEST: Company > Division
---
companies: @Company
  |[2] comp_1, Acme Corp
    | div_1, Engineering
    | div_2, Sales
  |[0] comp_2, Beta Inc
  |[1] comp_3, Gamma Ltd
    | div_3, Marketing
```

### Nested List Declarations

Lists can be declared as children of specific rows (alternative to NEST).

**Syntax**:
```hedl
<parent_list>: @ParentType
  | <parent_values>
    <child_list_key>: @ChildType
      | <child_values>
```

**Rules**:
- Child list declaration is indented +2 spaces under parent row
- Child list key must be unique per parent row
- Optional count hint: `divisions(3): @Division`

**Example**:
```hedl
companies: @Company[id, name]
  | comp_1, Acme Corp
    divisions: @Division[id, name]
      | div_1, Engineering
      | div_2, Sales
  | comp_2, Beta Inc
    divisions(1): @Division
      | div_3, Marketing
```

## Comments

**Syntax**: `# <comment text>`

Comments begin with `#` and continue to the end of the line.

**Rules**:
- `#` characters inside quoted strings and expressions are not comments
- Comments are stripped before parsing
- Blank lines and comment-only lines are ignored

**Example**:
```hedl
# This is a comment
name: Alice  # Inline comment
message: "# This is not a comment"
expr: $(x # y)  # Only the part after ) is a comment
```

## Indentation

HEDL uses significant indentation to denote hierarchy.

### Indentation Rules

1. **Increment**: 2 spaces per level
2. **No tabs**: Only spaces allowed in indentation
3. **Even spaces**: Indentation must be an even number of spaces
4. **Consistency**: All content at the same level must have the same indentation
5. **Maximum depth**: Limited by `max_indent_depth` (default: 50)

**Valid indentation**:
```hedl
root:          # 0 spaces
  level1:      # 2 spaces
    level2:    # 4 spaces
      level3:  # 6 spaces
```

**Invalid indentation**:
```hedl
root:
 level1:       # Error: 1 space (odd)
	level2:      # Error: tab character
   level3:     # Error: 3 spaces (odd)
```

### Context-Specific Indentation

- **Object children**: Parent indent + 2
- **Matrix rows**: List declaration indent + 2
- **Nested matrix rows**: Parent row indent + 2
- **Block string lines**: Any indentation (common indent stripped)

## Security Limits

The parser enforces security limits to prevent denial-of-service attacks:

| Limit | Default | Purpose |
|-------|---------|---------|
| `max_file_size` | 1 GB | Maximum input file size |
| `max_line_length` | 1 MB | Maximum line length |
| `max_indent_depth` | 50 | Maximum nesting depth for objects |
| `max_nodes` | 10M | Maximum matrix list nodes |
| `max_aliases` | 10K | Maximum number of aliases |
| `max_columns` | 100 | Maximum columns per schema |
| `max_nest_depth` | 100 | Maximum NEST hierarchy depth |
| `max_block_string_size` | 10 MB | Maximum block string size |
| `max_object_keys` | 10K | Maximum keys per object |
| `max_total_keys` | 10M | Maximum total keys across all objects |
| `timeout` | 30 sec | Maximum parsing time |

All limits are configurable via `ParseOptions`.

## Grammar

Informal BNF-style grammar for HEDL:

```ebnf
document          = header separator body

header            = directive*
directive         = version_directive | struct_directive | alias_directive | nest_directive
version_directive = "%VERSION:" version
struct_directive  = "%STRUCT:" type_name ":" column_list
alias_directive   = "%ALIAS:" "%" key ":" quoted_string
nest_directive    = "%NEST:" type_name ">" type_name

separator         = "---"

body              = (blank_line | comment_line | content_line)*
content_line      = indent (object_start | key_value | list_start | matrix_row | block_string_start)

object_start      = key ":"
key_value         = key ":" " " value
list_start        = key count_hint? ":" " " "@" type_name schema?
matrix_row        = "|" child_count? csv_row
block_string_start = key ":" " |" (">" | newline)

value             = null | bool | integer | float | string | tensor | reference | expression | alias_ref
null              = "null"
bool              = "true" | "false"
integer           = "-"? digit+
float             = "-"? digit+ "." digit+
string            = unquoted_string | quoted_string
tensor            = "[" (value ("," value)* ","?)? "]"
reference         = "@" (type_name ":")? id
expression        = "$(" expr ")"
alias_ref         = "%" key

expr              = call | access | atom
call              = identifier "(" (expr ("," expr)*)? ")"
access            = expr "." identifier
atom              = identifier | literal
literal           = integer | float | quoted_string | bool

column_list       = "[" key ("," key)* "]"
csv_row           = field ("," field)*
field             = quoted_field | unquoted_field | ditto
ditto             = "\""

count_hint        = "(" integer ")"
child_count       = "[" integer "]"
schema            = "[" key ("," key)* "]"

key               = [a-z_][a-z0-9_]*
type_name         = [A-Z][A-Za-z0-9]*
id                = [a-zA-Z_][a-zA-Z0-9_\-]*
identifier        = [a-zA-Z_][a-zA-Z0-9_]*

quoted_string     = "\"" (escape_seq | [^"])* "\""
escape_seq        = "\"\"" | "\\\"" | "\\\\" | "\\n" | "\\t" | "\\r"

comment_line      = "#" [^\n]*
blank_line        = [ \t]*
indent            = ("  ")*
```

## Character Encoding

HEDL documents must be UTF-8 encoded. All text content, including keys, values, and comments, can contain Unicode characters.

## Whitespace

- **Line ending**: LF (`\n`) or CRLF (`\r\n`)
- **Indentation**: Spaces only (no tabs)
- **After colon**: Exactly one space required in key-value pairs
- **In CSV rows**: Optional spaces around commas
- **Trailing whitespace**: Allowed and ignored

## Case Sensitivity

- **Key tokens**: Case-sensitive (`name` ≠ `Name`)
- **Type names**: Case-sensitive (`User` ≠ `user`)
- **Keywords**: Case-sensitive (`true` ≠ `True`, `null` ≠ `NULL`)
- **Directives**: Case-sensitive (`%VERSION` ≠ `%version`)

## Error Handling

The parser provides detailed error messages with line numbers for:
- Syntax errors (invalid tokens, missing colons, etc.)
- Schema errors (undefined types, mismatched schemas)
- Semantic errors (duplicate keys, unresolved references)
- Security errors (limit violations)
- Shape errors (wrong number of columns)

## Implementation Notes

This specification is based on the reference implementation in `crates/hedl-core/src/parser.rs` and related modules. Implementations should strive for compatibility with the reference parser behavior.

Key implementation modules:
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/tokens.rs` - Token validation
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/directives.rs` - Directive parsing
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/regions.rs` - Comment and escape handling
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/indent.rs` - Indentation rules
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/tensor.rs` - Tensor literals
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/expression.rs` - Expression syntax
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/parser.rs` - Overall structure

## Version History

- **1.0** (2025-01): Initial HEDL specification
