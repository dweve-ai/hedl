# Parsing Pipeline Architecture

## Overview

The HEDL parsing pipeline transforms raw text into a validated AST through five distinct phases: Preprocessing → Lexical Analysis → Parsing → Reference Resolution → Validation.

## Pipeline Stages

```mermaid
graph TB
    subgraph "Stage 1: Preprocessing"
        S1A[UTF-8 Validation]
        S1B[Line Iteration]
        S1C[Comment Stripping]
        S1D[Directive Detection]
    end

    subgraph "Stage 2: Lexical Analysis"
        S2A[Tokenization]
        S2B[Indent Validation]
        S2C[Region Scanning]
        S2D[Reference Parsing]
    end

    subgraph "Stage 3: Parsing"
        S3A[Header Parsing]
        S3B[Body Parsing]
        S3C[Matrix List Parsing]
        S3D[Value Inference]
    end

    subgraph "Stage 4: Reference Resolution"
        S4A[Build Type Registry]
        S4B[Resolve References]
        S4C[Build NEST Hierarchy]
    end

    subgraph "Stage 5: Validation"
        S5A[Schema Validation]
        S5B[Resource Limits]
        S5C[Final AST]
    end

    S1A --> S1B --> S1C --> S1D
    S1D --> S2A --> S2B --> S2C --> S2D
    S2D --> S3A --> S3B --> S3C --> S3D
    S3D --> S4A --> S4B --> S4C
    S4C --> S5A --> S5B --> S5C

    style S1A fill:#e3f2fd
    style S5C fill:#fff3e0
```

## Stage 1: Preprocessing

### Responsibilities

Transform raw byte input into structured lines ready for parsing.

### Operations

#### 1. UTF-8 Validation
```rust
fn validate_utf8(input: &[u8]) -> Result<&str> {
    std::str::from_utf8(input)
        .map_err(|e| HedlError::syntax("Invalid UTF-8", e.valid_up_to()))
}
```

**Validates**:
- Input is valid UTF-8
- No byte-order marks (BOM)
- No invalid sequences

#### 2. Line Iteration
```rust
fn preprocess(input: &[u8]) -> impl Iterator<Item = (usize, &str)> {
    std::str::from_utf8(input)
        .unwrap()
        .lines()
        .enumerate()
}
```

**Provides**:
- Line-by-line access
- Line numbers for error reporting
- Lazy evaluation

#### 3. Comment Stripping
```rust
fn strip_comment(line: &str) -> &str {
    if let Some(pos) = line.find('#') {
        &line[..pos]
    } else {
        line
    }
}
```

**Handles**:
- `#` line comments
- Inline comments (after values)
- Preserves `#` inside quoted strings

#### 4. Directive Detection
```rust
fn is_directive(line: &str) -> bool {
    line.trim_start().starts_with('%')
}

fn is_separator(line: &str) -> bool {
    line.trim() == "---"
}
```

**Detects**:
- `%VERSION`, `%STRUCT`, `%ALIAS`, `%NEST` directives
- `---` header/body separator
- Invalid directive names (error)

### Output

**Type**: `PreprocessedInput`
```rust
pub struct PreprocessedInput {
    text: String,
    line_offsets: Vec<(usize, usize, usize)>,
}

impl PreprocessedInput {
    pub fn lines(&self) -> impl Iterator<Item = (usize, &str)> {
        // Returns (line_number, line_content)
    }
}
```

### Performance

- **Complexity**: O(n) where n = file size
- **Memory**: O(1) - streaming iterator, no buffering
- **Optimization**: Uses `memchr` for SIMD comment detection

## Stage 2: Lexical Analysis

### Responsibilities

Identify and validate individual tokens and structures (rows, tensors, expressions) within text lines.

### Operations

#### 1. Token Validation
HEDL uses validation utilities instead of producing a separate token stream:

```rust
use hedl_core::lex::{is_valid_key_token, is_valid_type_name, is_valid_id_token};

assert!(is_valid_key_token("user_name"));
assert!(is_valid_type_name("User"));
```

**Token Types**:
- **Key**: `key:` with colon
- **Value**: Unquoted or quoted string
- **Row Operator**: `|`
- **Ditto Operator**: `^`
- **Expression**: `$(...)`
- **Tensor**: `[...]`
- **Reference**: `@Type:id` or `@id`
- **Alias**: `%alias`

#### 2. Indent Validation
```rust
use hedl_core::lex::{calculate_indent, validate_indent, IndentInfo};

// Calculate indentation from a line
pub fn calculate_indent(line: &str, line_num: u32) -> Result<Option<IndentInfo>, LexError> {
    let bytes = line.as_bytes();
    let mut spaces = 0;

    for &b in bytes {
        match b {
            b' ' => spaces += 1,
            b'\t' => {
                if bytes[spaces..].iter().all(|&b| b.is_ascii_whitespace()) {
                    return Ok(None);
                }
                return Err(LexError::TabInIndentation { 
                    pos: SourcePos::new(line_num as usize, spaces + 1) 
                });
            }
            _ => break,
        }
    }

    if spaces == bytes.len() || bytes[spaces..].iter().all(|&b| b.is_ascii_whitespace()) {
        return Ok(None);
    }

    if spaces % 2 != 0 {
        return Err(LexError::InvalidIndentation { 
            spaces, 
            pos: SourcePos::new(line_num as usize, 1) 
        });
    }

    Ok(Some(IndentInfo { spaces, level: spaces / 2 }))
}
```

#### 4. Reference Parsing
```rust
pub fn parse_reference(token: &str) -> Result<Reference, LexError> {
    if !token.starts_with('@') {
        return Err(LexError::InvalidToken { ... });
    }

    let token = &token[1..];

    if let Some(colon_pos) = token.find(':') {
        let type_name = &token[..colon_pos];
        let id = &token[colon_pos + 1..];
        // ... validation ...
        Ok(Reference {
            type_name: Some(type_name.to_string()),
            id: id.to_string(),
        })
    } else {
        // ... validation ...
        Ok(Reference {
            type_name: None,
            id: token.to_string(),
        })
    }
}
```

**Validates**:
- `@` prefix
- Type name format (PascalCase)
- ID token format (lowercase, alphanumeric)

### Output

Lexical utilities are called by the parser to validate and extract data from preprocessed lines.

### Performance

- **Complexity**: O(n) where n = line length
- **Memory**: O(1) for most validations, O(f) for row parsing (where f = field count)
- **Optimization**: SIMD for comment detection, zero-copy string slices for bare tokens

## Stage 3: Parsing

### Responsibilities

Build AST from token stream using header metadata.

### Operations

#### 1. Header Parsing
```rust
pub fn parse_header(lines: &[(usize, &str)], limits: &Limits) -> HedlResult<(Header, usize)> {
    let mut version = None;
    let mut structs = BTreeMap::new();
    let mut aliases = BTreeMap::new();
    let mut nests = BTreeMap::new();

    for (idx, &(line_num, line)) in lines.iter().enumerate() {
        if line.trim() == "---" {
            return Ok((Header { version, structs, aliases, nests }, idx + 1));
        }
        // ... parse %VERSION, %STRUCT, %ALIAS, %NEST ...
    }
}
```

**Parses**:
- `%VERSION: 1.0` → `(1, 0)`
- `%STRUCT: User: [id, name, email]` → Schema
- `%ALIAS: %pi: "3.14159"` → Constant
- `%NEST: User > Post` → Relationship

#### 2. Body Parsing

The body is parsed using a stack-based recursive descent algorithm that processes lines based on their indentation:

```rust
// Internal parsing state tracking
enum Frame {
    Root { object: BTreeMap<String, Item> },
    Object { key: String, indent: usize, object: BTreeMap<String, Item> },
    List { ... },
}

// The parser maintains a stack of frames to handle nesting
let mut stack = vec![Frame::Root { object: BTreeMap::new() }];
```

**Key Operations**:
- Track current indentation level
- Manage stack frames for nested objects and lists
- Track total key count across all objects for security
- Handle simple key-value pairs, nested objects, and matrix lists

#### 3. Matrix List Parsing

Matrix lists are detected via the `@TypeName` syntax and parsed by processing subsequent pipe-prefixed (`|`) lines:

**Key Features**:
- CSV field parsing with quoting and escaping
- Ditto operator (`^`) for efficient value repetition
- Automatic registration of node IDs for reference resolution
- Validation against the struct schema defined in the header
- Security enforcement of `max_nodes` limit

#### 4. Value Inference
```rust
pub fn infer_value(s: &str, ctx: &InferenceContext, line_num: usize) -> HedlResult<Value> {
    let s = s.trim();

    // Fast path for common values (true, false, ~)
    if let Some(value) = try_lookup_common(s) {
        return Ok(value);
    }

    // First-byte dispatch
    match s.as_bytes().first() {
        Some(b'^') => infer_ditto(ctx, line_num),
        Some(b'[') => parse_tensor(s),
        Some(b'@') => parse_reference(s),
        Some(b'$') => parse_expression(s),
        Some(b'%') => infer_alias(s, ctx),
        Some(b'-') | Some(b'0'..=b'9') => try_parse_number(s),
        _ => Ok(Value::String(s.to_string())),
    }
}
```

**Inference Rules**:
1. `~` → `Value::Null`
2. `true`/`false` → `Value::Bool`
3. `^` → Ditto (copies from previous row)
4. `@...` → `Value::Reference`
5. `$(...)` → `Value::Expression`
6. `[...]` → `Value::Tensor`
7. Numeric → `Value::Int` or `Value::Float`
8. Otherwise → `Value::String`

### Output

**Type**: Parsed `Document`
```rust
pub struct Document {
    pub version: (u32, u32),
    pub aliases: BTreeMap<String, String>,
    pub structs: BTreeMap<String, Vec<String>>,
    pub nests: BTreeMap<String, String>,
    pub root: BTreeMap<String, Item>,
}
```

### Performance

- **Complexity**: O(n) where n = token count
- **Memory**: O(n) for AST (heap allocated)
- **Optimization**: P2 lookup table for common values, P1 first-byte dispatch, zero-copy for strings during lexing (where applicable)

## Stage 4: Reference Resolution

### Responsibilities

Resolve all `@Type:id` and `@id` references to actual nodes.

### Operations

#### 1. ID Collection

The parser collects IDs into the type registry as it encounters them:

```rust
use hedl_core::reference::{TypeRegistry, register_node};

// During body parsing
register_node(
    &mut registries,
    &list.type_name,
    &node.id,
    line_num,
)?;
```

**Registry Structure**:
```rust
pub struct TypeRegistry {
    /// Forward index: type_name -> (id -> line_number)
    by_type: BTreeMap<String, BTreeMap<String, usize>>,
    /// Inverted index: id -> list of type names containing that ID
    by_id: HashMap<String, Vec<String>>,
}
```

#### 2. Resolve References

Reference resolution is handled during parsing with the `strict_refs` option:

```rust
use hedl::{parse_with_limits, ParseOptions};

// Strict mode: fail on unresolved references
let options = ParseOptions {
    limits: Limits::default(),
    strict_refs: true,
};
let doc = parse_with_limits(input.as_bytes(), options)?;

// Lenient mode: unresolved references remain as-is
let options = ParseOptions {
    limits: Limits::default(),
    strict_refs: false,
};
let doc = parse_with_limits(input.as_bytes(), options)?;
```

**Resolution Rules**:
- `@Type:id` → Look up in `registry[Type][id]`
- `@id` → Search all types for matching ID
- Strict mode: Error on unresolved
- Lenient mode: Replace with `null`

#### 3. Build NEST Hierarchy

NEST hierarchies are built based on the `%NEST` directives during reference resolution:

```rust
// NEST relationships are defined in the header
// %NEST: User > Post
// This means Post nodes nest under User nodes

// The nests map stores: BTreeMap<parent_type, child_type>
pub fn get_child_type(doc: &Document, parent_type: &str) -> Option<&String> {
    doc.nests.get(parent_type)
}
```

**NEST Rules**:
- `%NEST: User > Post` means Post nodes nest under User
- Child nodes have reference to parent
- Built during reference resolution

### Output

**Type**: Partially validated `Document` (references resolved)

### Performance

- **Complexity**: O(n + r) where n = nodes, r = references (amortized O(1) lookups via inverted index)
- **Memory**: O(n) for type registry
- **Optimization**: Inverted index for O(1) unqualified reference resolution

## Stage 5: Validation

### Responsibilities

Final validation and security limit enforcement.

### Operations

#### 1. Schema Validation
```rust
pub fn validate_schema(doc: &Document) -> Result<()> {
    for (key, item) in &doc.root {
        if let Item::List(ml) = item {
            // Validate schema exists
            let schema = doc.structs.get(&ml.type_name)
                .ok_or_else(|| HedlError::schema(
                    &format!("Unknown type: {}", ml.type_name),
                    0
                ))?;

            // Validate all rows match schema
            for (i, row) in ml.rows.iter().enumerate() {
                if row.fields.len() != schema.len() {
                    return Err(HedlError::schema(
                        &format!("Row {}: expected {} fields, got {}",
                            i, schema.len(), row.fields.len()),
                        0
                    ));
                }
            }
        }
    }

    Ok(())
}
```

**Validates**:
- All matrix lists have defined schemas
- All rows match schema field count
- Field types match (if type hints provided)

#### 2. Resource Limits

Resource limits are primarily enforced **during the parsing phase** to allow for "fail-fast" behavior and prevent resource exhaustion attacks:

**Limits Enforced**:
- `max_file_size`: Checked before parsing starts
- `max_total_keys`: Tracked and checked as each key is parsed
- `max_indent_depth`: Checked during indentation analysis
- `max_nodes`: Checked as rows are added to matrix lists
- `max_aliases`: Checked during header parsing
- `max_columns`: Checked when parsing struct definitions

### Output

**Type**: Fully validated `Document` (ready for use)

### Performance

- **Complexity**: O(n) where n = document size
- **Memory**: O(1) - in-place validation
- **Optimization**: Single-pass traversal

## Security Considerations

### DoS Protection

The parser enforces multiple defense-in-depth limits:

1. **Input Size Limit** (`max_file_size`): Prevent large input DoS
2. **Total Keys Limit** (`max_total_keys`): Prevent cumulative allocation DoS
3. **Depth Limit** (`max_indent_depth`): Prevent stack overflow
4. **Node Count Limit** (`max_nodes`): Prevent excessive node allocation

### Example Attack Mitigation

**Attack**: Many small objects with cumulative large key count
```hedl
%VERSION: 1.0
---
# 100,000 objects with 10 keys each = 1,000,000 total keys
obj0:
  k0: v0
  k1: v1
obj1:
  k0: v0
```

**Defense**: `max_total_keys` limit (default 10M) rejects document

## Parser Configuration

```rust
// From hedl-core/src/parser.rs
pub struct ParseOptions {
    pub limits: Limits,
    pub strict_refs: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            strict_refs: true,
        }
    }
}
```

## Error Handling

All parsing stages use `Result<T, HedlError>`:

```rust
pub fn parse_with_limits(input: &[u8], options: ParseOptions) -> HedlResult<Document> {
    // 1. Preprocess (handles file size, UTF-8, line splitting)
    let preprocessed = preprocess(input, &options.limits)?;

    // 2. Parse header directives
    let lines: Vec<(usize, &str)> = preprocessed.lines().collect();
    let (header, body_start_idx) = parse_header(&lines, &options.limits)?;

    // 3. Parse body
    let mut registries = TypeRegistry::new();
    let root = parse_body(&lines[body_start_idx..], &header, &options.limits, &mut registries)?;

    // 4. Resolve references
    let doc = Document { ... };
    resolve_references(&doc, options.strict_refs)?;

    Ok(doc)
}
```

## See Also

- [Data Flow](data-flow.md) - Overall data flow architecture
- [Module Dependencies](module-dependencies.md) - Crate dependencies
- [Performance Architecture](performance.md) - Performance optimizations

---

*Last updated: 2026-01-06*
