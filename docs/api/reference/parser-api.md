# Parser API Reference

Complete reference for HEDL parsing functions and options.

## Main Parsing Functions

### parse

Parse a HEDL string with strict validation.

```rust
pub fn parse(input: &str) -> Result<Document, HedlError>
```

**Parameters:**
- `input`: HEDL document as string slice

**Returns:**
- `Ok(Document)`: Parsed document
- `Err(HedlError)`: Parse error with location and details

**Example:**
```rust
use hedl::parse;

let doc = parse("%VERSION: 1.0\n---\nkey: value")?;
assert_eq!(doc.version, (1, 0));
```

**Errors:**
- `HedlErrorKind::Syntax`: Invalid HEDL syntax
- `HedlErrorKind::Reference`: Unresolved reference in strict mode
- `HedlErrorKind::Security`: Document exceeds limits

### parse_lenient

Parse with lenient reference handling (unresolved refs become null).

```rust
pub fn parse_lenient(input: &str) -> Result<Document, HedlError>
```

**Parameters:**
- `input`: HEDL document as string slice

**Returns:**
- `Ok(Document)`: Parsed document with nullified unresolved references
- `Err(HedlError)`: Parse error (non-reference errors)

**Example:**
```rust
use hedl::parse_lenient;

// This would fail with strict parse()
let doc = parse_lenient("%VERSION: 1.0\n---\nuser: @User:missing")?;
// user field becomes null
```

### parse_with_limits

Parse with custom resource limits and options.

```rust
pub fn parse_with_limits(
    input: &[u8],
    options: ParseOptions
) -> Result<Document, HedlError>
```

**Parameters:**
- `input`: HEDL document as bytes
- `options`: Parser configuration

**Returns:**
- `Ok(Document)`: Parsed document
- `Err(HedlError)`: Parse error

**Example:**
```rust
use hedl::{parse_with_limits, ParseOptions, Limits};

let options = ParseOptions {
    limits: Limits {
        max_indent_depth: 10,
        max_nodes: 1000,
        ..Limits::default()
    },
    strict_refs: false,
};

let doc = parse_with_limits(input.as_bytes(), options)?;
```

## Validation Functions

### lint

Detailed document linting for best practices.

```rust
pub fn lint(doc: &Document) -> Vec<Diagnostic>
```

**Parameters:**
- `doc`: Parsed document to lint

**Returns:**
- Vector of diagnostics (warnings, errors, suggestions)

**Example:**
```rust
use hedl::{parse, lint};

let doc = parse(input)?;
let diagnostics = lint(&doc);

for d in diagnostics {
    println!("[{:?}] Line {:?}: {}", d.severity(), d.line(), d.message());
}
```

### lint_with_config

Lint with custom rule configuration.

```rust
pub fn lint_with_config(
    doc: &Document,
    config: LintConfig
) -> Vec<Diagnostic>
```

**Parameters:**
- `doc`: Document to lint
- `config`: Linting configuration

**Returns:**
- Vector of diagnostics

**Example:**
```rust
use hedl::lint::{lint_with_config, LintConfig};

let config = LintConfig::default();
let diagnostics = lint_with_config(&doc, config);
```

## Configuration Types

### ParseOptions

```rust
pub struct ParseOptions {
    pub limits: Limits,
    pub strict_refs: bool,
}
```

**Fields:**
- **limits**: Resource limits
- **strict_refs**: Require all references to be resolved (default: true)

**Builder:**
```rust
use hedl_core::ParseOptions;

let options = ParseOptions::builder()
    .strict(false)
    .max_depth(20)
    .max_array_length(5000)
    .max_file_size(100 * 1024 * 1024)
    .build();
```

### ParseOptionsBuilder Methods

```rust
impl ParseOptionsBuilder {
    pub fn new() -> Self;
    pub fn max_depth(self, depth: usize) -> Self;
    pub fn max_array_length(self, length: usize) -> Self;
    pub fn strict(self, strict: bool) -> Self;
    pub fn max_file_size(self, size: usize) -> Self;
    pub fn max_line_length(self, length: usize) -> Self;
    pub fn max_aliases(self, count: usize) -> Self;
    pub fn max_columns(self, count: usize) -> Self;
    pub fn max_nest_depth(self, depth: usize) -> Self;
    pub fn max_block_string_size(self, size: usize) -> Self;
    pub fn max_object_keys(self, count: usize) -> Self;
    pub fn max_total_keys(self, count: usize) -> Self;
    pub fn build(self) -> ParseOptions;
}
```

### Limits

```rust
pub struct Limits {
    pub max_file_size: usize,        // Default: 1GB
    pub max_line_length: usize,      // Default: 1MB
    pub max_indent_depth: usize,     // Default: 50
    pub max_nodes: usize,            // Default: 10M
    pub max_aliases: usize,          // Default: 10k
    pub max_columns: usize,          // Default: 100
    pub max_nest_depth: usize,       // Default: 100
    pub max_block_string_size: usize, // Default: 10MB
    pub max_object_keys: usize,      // Default: 10k
    pub max_total_keys: usize,       // Default: 10M
}
```

**Example:**
```rust
use hedl_core::Limits;

let limits = Limits {
    max_indent_depth: 20,
    max_nodes: 100_000,
    ..Limits::default()
};
```

## Diagnostic Types

### Diagnostic

```rust
pub struct Diagnostic {
    severity: Severity,
    kind: DiagnosticKind,
    message: String,
    line: Option<usize>,
    rule_id: String,
    suggestion: Option<String>,
}
```

Access fields via methods: `severity()`, `kind()`, `message()`, `line()`, `rule_id()`, `suggestion()`.

### Severity

```rust
pub enum Severity {
    Hint,
    Warning,
    Error,
}
```

### DiagnosticKind

```rust
pub enum DiagnosticKind {
    IdNaming,              // ID naming convention violation
    TypeNaming,            // Type naming convention violation
    UnusedSchema,          // Unused schema definition
    UnusedAlias,           // Unused alias definition
    AmbiguousReference,    // Potentially ambiguous reference
    EmptyList,             // Empty matrix list
    InconsistentDitto,     // Inconsistent ditto usage
    MissingIdColumn,       // Missing ID column
    DuplicateKey,          // Duplicate keys in object
    UnqualifiedKvReference, // Unqualified reference in Key-Value context
    Custom(String),        // Custom rule violation
}
```

## Streaming Parsing

For large files, use `hedl-stream`:

```rust
use hedl_stream::{StreamingParser, NodeEvent};
use std::io::BufReader;
use std::fs::File;

let file = File::open("large-dataset.hedl")?;
let reader = BufReader::new(file);

let parser = StreamingParser::new(reader)?;

for event in parser {
    match event {
        Ok(NodeEvent::Node(node)) => {
            println!("{}:{}", node.type_name, node.id);
        }
        Ok(NodeEvent::ListStart { type_name, .. }) => {
            println!("List started: {}", type_name);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            break;
        }
        _ => {}
    }
}
```

### Timeout Protection

For untrusted input:

```rust
use hedl_stream::{StreamingParser, StreamingParserConfig};
use std::time::Duration;
use std::io::Cursor;

let config = StreamingParserConfig {
    timeout: Some(Duration::from_secs(10)),
    ..Default::default()
};

let parser = StreamingParser::with_config(
    Cursor::new(untrusted_input),
    config
)?;
```

### Async Streaming (feature = "async")

```rust
use hedl_stream::{AsyncStreamingParser, NodeEvent};
use tokio::fs::File;

let file = File::open("large-dataset.hedl").await?;
let parser = AsyncStreamingParser::new(file).await?;

while let Some(event) = parser.next_event().await? {
    match event {
        NodeEvent::Node(node) => {
            println!("{}:{}", node.type_name, node.id);
        }
        _ => {}
    }
}
```

## Lexer Utilities

### scan_regions

Scan a line for protected regions (quoted strings and expressions) where special characters like `#` and `,` lose their usual meaning.

```rust
use hedl::lex::{scan_regions, Region, RegionType};

let line = "name: \"John Doe\", age: $(calculate_age(birth_date)) # comment";
let regions = scan_regions(line);

for region in regions {
    match region.region_type {
        RegionType::Quote => {
            println!("Quoted string from {} to {}", region.start, region.end);
        }
        RegionType::Expression => {
            println!("Expression from {} to {}", region.start, region.end);
        }
    }
}
```

## Error Handling

See [Error Handling Guide](../guides/error-handling.md) for patterns.

```rust
use hedl::{parse, HedlErrorKind};

match parse(input) {
    Ok(doc) => process(doc),
    Err(e) => match e.kind {
        HedlErrorKind::Syntax => handle_syntax_error(e),
        HedlErrorKind::Reference => try_lenient_parse(input),
        HedlErrorKind::Security => handle_limit_exceeded(e),
        _ => handle_other_error(e),
    }
}
```

## Parallel Parsing

For independent documents:

```rust
use rayon::prelude::*;

let docs: Vec<Result<Document, HedlError>> = inputs
    .par_iter()
    .map(|input| hedl::parse(input))
    .collect();
```

## See Also

- [Core Types](core-types.md) - Type definitions
- [Serializer API](serializer-api.md) - Serialization functions
- [Error Handling Guide](../guides/error-handling.md) - Error patterns
- [Rust API Reference](../rust-api.md) - Complete Rust API
