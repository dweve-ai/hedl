# HEDL Syntax Specification

This document provides a formal specification of the HEDL (Hierarchical Entity Data Language) syntax based on the reference parser implementation in `crates/hedl-core`.

## Contents

This document covers document structure, lexical elements, header directives, body syntax, value types, matrix lists, comments, indentation rules, security limits, and formal grammar.

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
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
%A:%active:"true"
---
users:@User
 |user_1,Alice,alice@example.com
 |user_2,Bob,bob@example.com
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

### %V Directive (Version)

**Syntax**: `%V:<version>`

Specifies the HEDL format version. Current version is `2.0`.

**Example**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
```

All HEDL documents should use the `%V:2.0` directive.

### %S Directive (Struct)

**Syntax**:
- `%S:<TypeName>:[<col1>,<col2>,...]`
- With count hint: `%S:<TypeName>(<count>):[<col1>,<col2>,...]`

Defines the schema for a matrix list type.

**Rules**:
- TypeName must be a valid Type Name token
- Column names must be valid Key tokens
- At least one column is required
- No duplicate column names
- Maximum columns limited by `max_columns` (default: 100)
- Optional count hint `(N)` indicates expected number of rows (informational only)

**Examples**:
```hedl
%S:User:[id,name,email]
%S:Post:[id,title,author_id,content]
%S:Product(100):[id,sku,name,price]
```

### %A Directive (Alias)

**Syntax**: `%A:%<key>:"<value>"`

Defines a constant that can be referenced later using `%key`.

**Rules**:
- Key must be a valid Key token (after the % prefix)
- Value must be a quoted string
- Escape sequences supported in value (see [Escape Sequences](#escape-sequences))
- Maximum aliases limited by `max_aliases` (default: 10,000)

**Examples**:
```hedl
%A:%active:"true"
%A:%admin:"Administrator"
%A:%default_email:"user@example.com"
```

### %N Directive (Nest)

**Syntax**: `%N:<ParentType>><ChildType>`

Defines a parent-child relationship between two entity types.

**Rules**:
- Both ParentType and ChildType must be valid Type Names
- Both types must be defined in %S directives
- Defines that ChildType rows can be nested under ParentType rows
- Maximum nest depth limited by `max_nest_depth` (default: 100)

**Examples**:
```hedl
%S:Company:[id,name]
%S:Division:[id,name]
%N:Company>Division
```

### %NULL Directive

**Syntax**: `%NULL:<char>`

Defines the character used to represent null values in the document.

**Rules**:
- Must be a single character
- Required in all documents
- Standard value is `~`

**Examples**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
value: ~     # null value

%V:2.0
%NULL:-
%QUOTE:"
---
value: -     # null value (using custom character)
```

### %QUOTE Directive

**Syntax**: `%QUOTE:<char>`

Defines the character used for quoting strings.

**Rules**:
- Must be a single character
- Required in all documents
- Can only be specified once per document
- Standard value is `"`

**Examples**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
name: "Alice"

%V:2.0
%NULL:~
%QUOTE:'
---
name: 'Alice'    # using single quotes as the quote character
```

### %COUNT Directive

**Syntax**:
- Total count: `%C:<Type>.total=<N>`
- Distribution: `%C:<Type>.<field>:<val1>=<N1>,<val2>=<N2>,...`

Provides statistical metadata about the data. This is informational for LLM comprehension and tooling; it does not affect parsing.

**Examples**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Product:[id,sku,name,category,price]
%C:Product.total=15
%C:Product.category:electronics=9,clothing=3,sports=3
---
products:@Product
 |p1,SKU-001,Laptop,electronics,999.99
  # ... 14 more products
```

**Use Cases**:
- LLM comprehension: Helps models understand data distribution without scanning all rows
- Validation: Tools can verify actual counts match declared counts
- Preview: Quick summary of large datasets

## Body Section

The body contains the actual data in hierarchical format.

### Objects

**Syntax**:
```hedl
<key>:
  <nested content>
```

Objects are defined by a key followed by a colon with no value, and contain nested key-value pairs or other structures indented by 1 space.

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

 |Escape |Meaning |
|--------|---------|
 |`""` |Literal quote (CSV-style) |
 |`\"` |Literal quote (backslash-style) |
 |`\\` |Literal backslash |
 |`\n` |Newline |
 |`\t` |Tab |

**Example**:
```hedl
csv_style: "say ""hello"""
backslash_style: "say \"hello\""
multiline: "line1\nline2"
tab_separated: "col1\tcol2"
windows_path: "C:\\Users\\test"
```

Unknown escape sequences are invalid and result in a parse error.

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
author:@user_1

# Qualified reference
author:@User:user_1
```

### Expressions

**Syntax**: `$(<expression>)`

Expressions are evaluated at parse time (or later, depending on implementation).

**Grammar**:
```
expr     = call |access |atom
call     = identifier "(" args ")"
access   = expr "." identifier
atom     = identifier |literal
args     = (expr ("," expr)*)?
literal  = number |string |bool
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
%A:%active: "true"

# In body:
status: %active
```

### List Literals

**Syntax**: `(<elem1>, <elem2>, ...)`

List literals are ordered sequences of scalar values enclosed in parentheses.

**Rules**:
- Elements are separated by commas
- Elements can be any scalar value (strings, references, booleans, numbers)
- Empty list is allowed: `()`
- Distinct from tensors `[...]` which are numeric-only arrays

**Examples**:
```hedl
# List of strings
tags: (rust, performance, data)

# List of references
team: (@User:alice, @User:bob)

# Empty list
categories: ()

# In matrix rows
%S:Article:[id,title,tags]
---
articles:@Article
 |art-1,Intro,(tutorial,beginner)
 |art-2,Advanced,(expert)
```

**Note**: Use `(...)` for lists of any scalar values. Use `[...]` for numeric tensors only.

## Matrix Lists

Matrix lists are tables of structured entities defined by a schema.

### List Declaration

**Syntax**: `<key>:@<TypeName>` or `<key>:@<TypeName>[<schema>]`

**Formats**:
1. Reference to declared schema: `users:@User`
2. Inline schema: `users:@User[id, name, email]`

**Rules**:
- TypeName must be defined in a %STRUCT directive (format 1)
- Inline schema must match declared schema if both exist
- Inline schema follows same rules as %STRUCT columns

**Example**:
```hedl
# Reference to schema
users:@User
 |user_1,Alice,alice@example.com

# Inline schema
users:@User[id,name,email]
 |user_1,Alice,alice@example.com
```

### Matrix Rows

**Syntax**: `| <csv-values>` or `|[<N>] <csv-values>`

Matrix rows are indented 1 space under the list declaration.

**Formats**:
1. Leaf row: `| value1, value2, value3`
2. Parent row with child count: `|[N] value1, value2, value3`

**Rules**:
- Must start with `|` (pipe character)
- Values are CSV-formatted
- Number of values must match schema length
- First column is the ID (must be a string)
- Values can be quoted or unquoted
- Child count `[N]` indicates this row has N child rows
- Child count `[0]` indicates a parent row with no children
- Children are indented +1 space and must have a NEST relationship defined

### Nested Lists (NEST)

Child entities can be nested under parent rows when a NEST relationship is defined.

**v2.0 Syntax** (recommended):
```hedl
%N:<ParentType>><ChildType>
%C:<ParentType>.total=N
%C:<ChildType>.total=M

<parent_list>:@<ParentType>
 |<parent_values>
  @<ChildType>#<count>:
  |<child_values>
  |<child_values>
```

**Rules**:
- NEST relationship must be defined in header with `%N:`
- Counts declared in header with `%C:<Type>.total=N`
- Child rows use `@Type#N:` block syntax (multi-line) or `@Type#N:|row` (inline)
- Child blocks are indented +1 space relative to parent row
- Maximum nest depth limited by `max_nest_depth` (default: 100)

**Example**:
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Company:[id,name]
%S:Division:[id,name]
%N:Company>Division
%C:Company.total=3
%C:Division.total=3
---
companies:@Company
 |comp_1,Acme Corp
  @Division#2:
  |div_1,Engineering
  |div_2,Sales
 |comp_2,Beta Inc
 |comp_3,Gamma Ltd
  @Division#1:|div_3,Marketing
```

### Nested List Declarations

Lists can be declared as children of specific rows (alternative to NEST).

**Syntax**:
```hedl
<parent_list>:@ParentType
 | <parent_values>
    <child_list_key>:@ChildType
   | <child_values>
```

**Rules**:
- Child list declaration is indented +1 space under parent row
- Child list key must be unique per parent row
- Optional count hint: `divisions(3):@Division`

**Example**:
```hedl
companies:@Company[id,name]
 |comp_1,Acme Corp
    divisions:@Division[id,name]
   |div_1,Engineering
   |div_2,Sales
 |comp_2,Beta Inc
    divisions(1):@Division
   |div_3,Marketing
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

1. **Increment**: 1 space per level2. **No tabs**: Only spaces allowed in indentation
3. **Even spaces**: Indentation must be an even number of spaces
4. **Consistency**: All content at the same level must have the same indentation
5. **Maximum depth**: Limited by `max_indent_depth` (default: 50)

**Valid indentation**:
```hedl
root:          # 0 spaces
  level1:      # 1 space    level2:    # 4 spaces
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

- **Object children**: Parent indent + 1- **Matrix rows**: List declaration indent + 1- **Nested matrix rows**: Parent row indent + 1- **Block string lines**: Any indentation (common indent stripped)

## Security Limits

The parser enforces security limits to prevent denial-of-service attacks:

 |Limit |Default |Purpose |
|-------|---------|---------|
 |`max_file_size` |1 GB |Maximum input file size |
 |`max_line_length` |1 MB |Maximum line length |
 |`max_indent_depth` |50 |Maximum nesting depth for objects |
 |`max_nodes` |10M |Maximum matrix list nodes |
 |`max_aliases` |10K |Maximum number of aliases |
 |`max_columns` |100 |Maximum columns per schema |
 |`max_nest_depth` |100 |Maximum NEST hierarchy depth |
 |`max_block_string_size` |10 MB |Maximum block string size |
 |`max_object_keys` |10K |Maximum keys per object |
 |`max_total_keys` |10M |Maximum total keys across all objects |
 |`timeout` |30 sec |Maximum parsing time |

All limits are configurable via `ParseOptions`.

## Grammar

Informal BNF-style grammar for HEDL:

```ebnf
document          = header separator body

header            = directive*
directive         = version_directive |struct_directive |alias_directive |nest_directive
version_directive = "%VERSION:" version
struct_directive  = "%STRUCT:" type_name ":" column_list
alias_directive   = "%A:" "%" key ":" quoted_string
nest_directive    = "%NEST:" type_name ">" type_name

separator         = "---"

body              = (blank_line |comment_line |content_line)*
content_line      = indent (object_start |key_value |list_start |matrix_row |block_string_start)

object_start      = key ":"
key_value         = key ":" " " value
list_start        = key count_hint? ":" " " "@" type_name schema?
matrix_row        = "|" child_count? csv_row
block_string_start = key ":" " |" (">" |newline)

value             = null |bool |integer |float |string |tensor |list |reference |expression |alias_ref
null              = "null"
bool              = "true" |"false"
integer           = "-"? digit+
float             = "-"? digit+ "." digit+
string            = unquoted_string |quoted_string
tensor            = "[" (value ("," value)* ","?)? "]"
list              = "(" (scalar ("," scalar)*)? ")"
scalar            = null |bool |integer |float |string |reference |alias_ref
reference         = "@" (type_name ":")? id
expression        = "$(" expr ")"
alias_ref         = "%" key

expr              = call |access |atom
call              = identifier "(" (expr ("," expr)*)? ")"
access            = expr "." identifier
atom              = identifier |literal
literal           = integer |float |quoted_string |bool

column_list       = "[" key ("," key)* "]"
csv_row           = field ("," field)*
field             = quoted_field |unquoted_field

count_hint        = "(" integer ")"
child_count       = "[" integer "]"
schema            = "[" key ("," key)* "]"

key               = [a-z_][a-z0-9_]*
type_name         = [A-Z][A-Za-z0-9]*
id                = [a-zA-Z_][a-zA-Z0-9_\-]*
identifier        = [a-zA-Z_][a-zA-Z0-9_]*

quoted_string     = "\"" (escape_seq | [^"])* "\""
escape_seq        = "\"\"" |"\\\"" |"\\\\" |"\\n" |"\\t" |"\\r"

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

This specification is based on the reference implementation in `crates/hedl-core/src/parser/` and related modules. Implementations should strive for compatibility with the reference parser behavior.

Key implementation modules:
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/tokens.rs` - Token validation
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/directives.rs` - Directive parsing
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/regions.rs` - Comment and escape handling
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/indent.rs` - Indentation rules
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/tensor.rs` - Tensor literals
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/lex/expression.rs` - Expression syntax
- `/home/marc/dev/projects/hedl/crates/hedl-core/src/parser/` - Overall structure

## Complete Example

This example demonstrates all major HEDL features working together: schemas, aliases, nesting, references, and count directives.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%A:%pg:"Point Guard"
%A:%sg:"Shooting Guard"
%S:Team:[id,name,city]
%S:Player:[id,name,position,number]
%N:Team>Player
%C:Team.total=2
%C:Player.total=4
---
teams:@Team
 |t1,Lakers,Los Angeles
  @Player#2:
  |p1,LeBron,%sg,23
  |p2,Davis,Center,3
 |t2,Celtics,Boston
  @Player#2:
  |p3,Tatum,%sg,0
  |p4,Brown,%pg,7
```

**What this document defines:**

1. **Aliases** (`%A`): Shorthand `%pg` and `%sg` expand to full position names
2. **Schemas** (`%S`): Team and Player structures with their fields
3. **Nesting** (`%N`): Players nest under Teams
4. **Counts** (`%C`): 2 teams total, 4 players total
5. **Data**: Two NBA teams with two players each, using references and aliases
