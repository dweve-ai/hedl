# hedl-stream

**Memory-efficient streaming parser for HEDL documents—process multi-gigabyte files with constant memory usage.**

Large HEDL files don't fit in RAM. Database exports, log archives, data pipelines—gigabytes of structured data that need processing without loading everything into memory. Traditional parsing loads the entire document first, then gives you access. That doesn't scale.

`hedl-stream` provides event-driven streaming parsing with O(1) memory regardless of file size. Process 10 GB files with 100 MB RAM. Iterate through nodes as they're parsed. Build custom processing pipelines with standard Rust iterators. Add timeout protection for untrusted input. Optional async support for high-concurrency scenarios.

## What's Implemented

Production-grade streaming with comprehensive features:

1. **Event-Driven Streaming**: SAX-style iterator API yielding nodes as they're parsed
2. **Constant Memory**: O(nesting_depth) memory usage regardless of file size
3. **Full HEDL Support**: Headers, schemas, matrix rows, nested structures, references, aliases
4. **Timeout Protection**: Configurable timeout for untrusted input (DoS protection)
5. **SIMD Optimization**: AVX2-accelerated comment detection (optional, x86_64 only)
6. **Async API**: Full tokio-based async implementation (optional feature)
7. **Comprehensive Parsing**: CSV-like rows, ditto operator, quoted strings, escape sequences
8. **Security Hardening**: Max line length (1MB), max indent depth (100), timeout enforcement
9. **Type Inference**: Automatic detection of null, bool, int, float, string, reference values
10. **Error Recovery**: Line numbers on all errors, specific error types, clear messages

## Installation

```toml
[dependencies]
hedl-stream = "1.0"

# For async support:
hedl-stream = { version = "1.0", features = ["async"] }
tokio = { version = "1", features = ["io-util"] }
```

## Streaming API

### Basic Usage

Process large HEDL files with constant memory:

```rust
use hedl_stream::{StreamingParser, NodeEvent};
use std::fs::File;

// Open large HEDL file (e.g., 5 GB database export)
let file = File::open("massive_data.hedl")?;
let parser = StreamingParser::new(file)?;

let mut node_count = 0;
for event in parser {
    match event? {
        NodeEvent::Header(header) => {
            println!("Version: {}.{}", header.version.0, header.version.1);
            println!("Schemas: {:?}", header.structs.keys());
        }
        NodeEvent::ListStart { key, type_name, schema, .. } => {
            println!("Processing list '{}' of type {}", key, type_name);
        }
        NodeEvent::Node(node) => {
            node_count += 1;
            // Process individual node
            // Memory usage stays constant regardless of total nodes
        }
        NodeEvent::ListEnd { key, count, .. } => {
            println!("Completed list '{}': {} nodes", key, count);
        }
        NodeEvent::EndOfDocument => break,
        _ => {}
    }
}
println!("Processed {} nodes from multi-GB file", node_count);
```

**Memory Usage**: Only the current line and context stack are in memory. A 5 GB file uses the same memory as a 5 MB file.

### Custom Configuration

Fine-tune parsing with `StreamingParserConfig`:

```rust
use hedl_stream::{StreamingParser, StreamingParserConfig};
use std::time::Duration;
use std::fs::File;

let config = StreamingParserConfig {
    max_line_length: 500_000,        // 500 KB max line (default: 1 MB)
    max_indent_depth: 50,            // 50 levels max nesting (default: 100)
    buffer_size: 128 * 1024,         // 128 KB I/O buffer (default: 64 KB)
    timeout: Some(Duration::from_secs(30)), // 30-second timeout (default: None)
};

let file = File::open("untrusted_input.hedl")?;
let parser = StreamingParser::with_config(file, config)?;

// Parsing will error if it exceeds 30 seconds (DoS protection)
```

### Configuration Options

**max_line_length** (default: 1,000,000 bytes)
- Protection against malformed input with extremely long lines
- Raises `Syntax` error if exceeded
- Recommended: 1 MB for normal data, lower for untrusted input

**max_indent_depth** (default: 100 levels)
- Limits nesting depth to prevent stack overflow
- Raises `Syntax` error if exceeded
- Recommended: 100 for normal data, 20-50 for untrusted input

**buffer_size** (default: 65,536 bytes)
- I/O buffer size for reading input
- Larger buffers improve performance for large files
- Trade-off: memory vs syscall frequency
- Recommended: 64 KB general, 128-256 KB for high-throughput

**timeout** (default: None)
- Optional duration limit for parsing operations
- Checked every 100 operations (low overhead ~0.1%)
- Raises `Timeout` error if exceeded
- Critical for untrusted input: prevents CPU DoS attacks
- Recommended: None for trusted data, 10-60s for untrusted

## Event Types

### NodeEvent Enum

**Header(HeaderInfo)**
- Emitted first with document metadata
- Contains: version, schemas (%STRUCT), aliases (%ALIAS), nesting rules (%NEST)

**ListStart { key, type_name, schema, line }**
- Marks beginning of entity list
- `key`: Field name (e.g., "users")
- `type_name`: Entity type (e.g., "User")
- `schema`: Column names for matrix
- `line`: Source line number

**Node(NodeInfo)**
- Individual entity row from matrix list
- Contains: type_name, id, fields (Vec<Value>), depth, parent info, line
- Emitted immediately after parsing (no buffering)

**ListEnd { key, type_name, count }**
- Marks end of entity list
- `count`: Total nodes in list
- Emitted when indentation decreases or EOF

**Scalar { key, value, line }**
- Key-value pair (not part of matrix list)
- Example: `name: "My App"`

**ObjectStart { key, line }** / **ObjectEnd { key }**
- Nested object boundaries
- Used for non-list hierarchical data

**EndOfDocument**
- Marks successful parse completion
- Always emitted at EOF

### NodeInfo Structure

```rust
pub struct NodeInfo {
    pub type_name: String,            // Entity type (e.g., "User")
    pub id: String,                   // Entity ID (first field)
    pub fields: Vec<Value>,           // All field values including ID
    pub depth: usize,                 // Nesting depth (0 = root)
    pub parent_id: Option<String>,    // Parent entity ID if nested
    pub parent_type: Option<String>,  // Parent entity type if nested
    pub line: usize,                  // Source line number
}
```

**Methods**:
- `get_field(index) -> Option<&Value>` - Get field by column index
- `is_nested() -> bool` - Check if node has parent

### HeaderInfo Structure

```rust
pub struct HeaderInfo {
    pub version: (u32, u32),                        // Major.Minor
    pub structs: BTreeMap<String, Vec<String>>,    // Type schemas
    pub aliases: BTreeMap<String, String>,         // Variable aliases
    pub nests: BTreeMap<String, String>,           // Parent->Child rules
}
```

**Methods**:
- `get_schema(type_name) -> Option<&Vec<String>>` - Lookup type schema
- `get_child_type(parent_type) -> Option<&String>` - Get child type for parent

## Async Support

For high-concurrency scenarios with thousands of concurrent streams:

```rust
use hedl_stream::AsyncStreamingParser;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("large_data.hedl").await?;
    let mut parser = AsyncStreamingParser::new(file).await?;

    let mut count = 0;
    loop {
        match parser.next_event().await? {
            Some(NodeEvent::Node(_)) => count += 1,
            Some(NodeEvent::EndOfDocument) => break,
            Some(_) => {}
            None => break,
        }
    }
    println!("Processed {} nodes", count);

    Ok(())
}
```

**Performance**: Async version has identical memory profile to sync version. Suitable for processing thousands of files concurrently without blocking.

## Parsing Features

### Matrix Row Parsing

CSV-like comma-separated rows prefixed with `|`:

```hedl
users: @User[id, name, age, active]
  | alice, Alice Smith, 30, true
  | bob, Bob Jones, 25, false
  | carol, Carol White, 35, true
```

**Features**:
- Quoted string handling: `| id1, "Smith, John", 30`
- Escape sequences: `| id2, "Quote: \"value\"", 25`
- Type inference: null, bool, int, float, string, reference

### Ditto Operator

Repeat previous value with `^`:

```hedl
orders: @Order[id, customer, status]
  | ord1, @User:alice, pending
  | ord2, ^, shipped          # customer = @User:alice (from previous row)
  | ord3, @User:bob, pending
  | ord4, ^, ^                # customer = @User:bob, status = pending
```

### Reference Parsing

Automatic detection of entity references:

```hedl
# Qualified reference
customer: @User:alice     # Reference(qualified("User", "alice"))

# Local reference
parent: @previous_item    # Reference(local("previous_item"))
```

### Alias Substitution

Variable substitution with `$`:

```hedl
%ALIAS: api_url: https://api.example.com
---
config:
  endpoint: $api_url      # Substituted to "https://api.example.com"
```

### Comment Handling

Full-line and inline comments:

```hedl
# This is a full-line comment
users: @User[id, name]
  | alice, Alice    # This is an inline comment
  | bob, "Bob # Not a comment (inside quotes)"
```

**SIMD Optimization**: When compiled with AVX2 support (x86_64), comment detection uses 32-byte SIMD scanning for 2-3x speedup on comment-heavy files.

## Error Handling

Comprehensive error types with line numbers:

```rust
use hedl_stream::{StreamingParser, StreamError};

for event in parser {
    match event {
        Ok(event) => { /* process */ }
        Err(StreamError::Syntax { line, message }) => {
            eprintln!("Syntax error at line {}: {}", line, message);
        }
        Err(StreamError::ShapeMismatch { line, expected, got }) => {
            eprintln!("Line {}: expected {} columns, got {}", line, expected, got);
        }
        Err(StreamError::OrphanRow { line, message }) => {
            eprintln!("Line {}: orphan row - {}", line, message);
        }
        Err(StreamError::Timeout { elapsed, limit }) => {
            eprintln!("Parsing timeout: {:?} exceeded limit {:?}", elapsed, limit);
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }
}
```

### Error Types

- `Io(std::io::Error)` - I/O read failures
- `Utf8 { line, message }` - Invalid UTF-8 encoding
- `Syntax { line, message }` - Parse syntax error
- `Schema { line, message }` - Schema/type mismatch
- `Header(String)` - Invalid header format
- `MissingVersion` - No %VERSION directive
- `InvalidVersion(String)` - Malformed version string
- `OrphanRow { line, message }` - Child row without parent entity
- `ShapeMismatch { line, expected, got }` - Column count doesn't match schema
- `Timeout { elapsed, limit }` - Parsing exceeded timeout duration

## Use Cases

**Database Export Processing**: Stream multi-GB database exports, transform data row-by-row, write to different format without loading entire export into memory.

**Log File Analysis**: Parse massive HEDL log archives, filter events, aggregate statistics, generate reports—all with constant memory usage.

**Data Pipeline Integration**: Read HEDL from network streams, process incrementally, forward to downstream systems without buffering.

**ETL Workflows**: Extract from large HEDL files, transform with custom logic, load to database with batch inserts—process millions of rows efficiently.

**Real-Time Processing**: Parse HEDL data as it arrives (stdin, network socket), emit events immediately, support backpressure naturally.

**Untrusted Input Validation**: Parse user-uploaded HEDL with timeout protection, validate structure, reject malicious input before full processing.

## What This Crate Doesn't Do

**Full Document Construction**: Doesn't build complete `Document` objects—that's hedl-core's job. For full document parsing, use `hedl_core::parse()`. Use streaming when you need memory efficiency.

**Random Access**: Sequential-only parser. Can't jump to arbitrary positions. For random access, load full document with hedl-core.

**Modification**: Read-only parser. Can't modify nodes during parsing. For transformations, consume events and write new HEDL output.

**Validation**: Parses structure, doesn't validate business rules. For schema validation, use `hedl-lint` on parsed documents.

## Performance Characteristics

**Memory**: O(nesting_depth) regardless of file size. Typically <1 MB for files of any size with reasonable nesting.

**I/O**: Configurable buffer size (default 64 KB) minimizes syscalls. Batched reads for optimal throughput.

**Parsing**: Linear pass through input. SIMD-accelerated comment detection (AVX2, ~2-3x faster for comment-heavy files).

**Timeout Checks**: Every 100 operations (~0.1% overhead). Negligible impact on normal workloads.

**Async**: Same memory profile as sync. Non-blocking I/O yields to runtime during reads. Suitable for thousands of concurrent streams.

## Dependencies

- `hedl-core` (workspace) - Core types (Value, Reference), lexer utilities
- `thiserror` 1.0 - Error type definitions
- `tokio` 1.35 (optional, "async" feature) - Async I/O runtime

## License

Apache-2.0
