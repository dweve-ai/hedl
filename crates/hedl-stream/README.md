# hedl-stream

Streaming parser for HEDL documents. When you have a multi-gigabyte file and cannot load it into memory, this crate processes it incrementally with constant memory overhead.

`hedl-core` parses entire documents into memory before giving you access. That works for configuration files and small datasets, but fails for database exports, log archives, and data pipelines where files reach tens of gigabytes. This crate solves that by emitting events as it parses: you receive each node immediately after it's read from the input stream, and memory usage stays proportional to nesting depth rather than file size.

## Getting Started

```toml
[dependencies]
hedl-stream = "1"
```

For async support with Tokio:

```toml
[dependencies]
hedl-stream = { version = "1", features = ["async"] }
tokio = { version = "1", features = ["io-util"] }
```

## Basic Usage

Open a file, iterate through events, do something with each node:

```rust
use hedl_stream::{StreamingParser, NodeEvent};
use std::fs::File;

let file = File::open("massive_data.hedl")?;
let parser = StreamingParser::new(file)?;

let mut node_count = 0;
for event in parser {
    match event? {
        NodeEvent::Header(header) => {
            println!("Found schemas: {:?}", header.structs.keys());
        }
        NodeEvent::ListStart { key, type_name, .. } => {
            println!("Starting list '{}' of type {}", key, type_name);
        }
        NodeEvent::Node(node) => {
            node_count += 1;
            // Do something with each node
            // Memory stays constant no matter how many you process
        }
        NodeEvent::ListEnd { key, count, .. } => {
            println!("Finished '{}': {} nodes", key, count);
        }
        NodeEvent::EndOfDocument => break,
        _ => {}
    }
}
println!("Processed {} nodes", node_count);
```

## Working with Untrusted Input

When parsing files you don't control, timeouts prevent someone from feeding you a file designed to hang your server:

```rust
use hedl_stream::{StreamingParser, StreamingParserConfig, MemoryLimits};
use std::time::Duration;
use std::fs::File;

let config = StreamingParserConfig {
    max_line_length: 500_000,                      // 500 KB lines max
    max_indent_depth: 50,                          // 50 levels of nesting
    buffer_size: 128 * 1024,                       // 128 KB I/O buffer
    timeout: Some(Duration::from_secs(30)),        // Give up after 30 seconds
    memory_limits: MemoryLimits::untrusted(),      // Stricter limits
    enable_pooling: false,
};

let file = File::open("user_upload.hedl")?;
let parser = StreamingParser::with_config(file, config)?;
```

## Async Parsing

Same API, but for when you're juggling thousands of concurrent streams:

```rust
use hedl_stream::AsyncStreamingParser;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("large_data.hedl").await?;
    let mut parser = AsyncStreamingParser::new(file).await?;

    loop {
        match parser.next_event().await? {
            Some(NodeEvent::Node(node)) => {
                // process node
            }
            Some(NodeEvent::EndOfDocument) => break,
            Some(_) => {}
            None => break,
        }
    }

    Ok(())
}
```

## Events You'll See

The parser emits events as it encounters them:

**Header** comes first, containing the document's version, schemas, aliases, and nesting rules.

**ListStart** marks the beginning of an entity list (like `users: @User[id, name, age]`), telling you the field name, type, and column schema.

**Node** is an individual row from a list. Each node carries its type, ID, field values, nesting depth, and parent information if nested.

**ListEnd** fires when a list finishes, including a count of how many nodes it contained.

**Scalar** appears for simple key-value pairs outside of lists.

**ObjectStart** and **ObjectEnd** bracket nested objects that aren't entity lists.

**EndOfDocument** means you've reached the end successfully.

## HEDL Features the Parser Handles

Matrix rows with CSV-like syntax:

```hedl
users: @User[id, name, age]
 | alice, Alice Smith, 30
 | bob, "Jones, Bob", 25
 | carol, "Said \"hello\"", 35
```

Entity references are detected automatically:

```hedl
customer: @User:alice     # Qualified reference to User alice
parent: @previous_item    # Local reference
```

Alias substitution:

```hedl
%A:api_url: https://api.example.com
---
config:
  endpoint: $api_url      # Becomes https://api.example.com
```

Comments anywhere:

```hedl
# Full line comment
users: @User[id, name]
 | alice, Alice    # Inline comment
 | bob, "Bob # This is not a comment (inside quotes)"
```

## Error Handling

Errors tell you what went wrong and where:

```rust
for event in parser {
    match event {
        Ok(event) => { /* process */ }
        Err(StreamError::Syntax { line, message }) => {
            eprintln!("Line {}: {}", line, message);
        }
        Err(StreamError::ShapeMismatch { line, expected, got }) => {
            eprintln!("Line {}: expected {} columns, got {}", line, expected, got);
        }
        Err(StreamError::Timeout { elapsed, limit }) => {
            eprintln!("Took too long: {:?}", elapsed);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Performance Notes

Memory usage depends on nesting depth, not file size. A 50 GB file with 10 levels of nesting uses the same memory as a 50 KB file with 10 levels of nesting.

The default 64 KB I/O buffer works well for most cases. Bump it to 128-256 KB for maximum throughput on large files.

On x86_64 with AVX2, comment detection uses SIMD for faster scanning of comment-heavy files.

Timeout checks happen every 100 operations, adding about 0.1% overhead.

## Dependencies

- `hedl-core` for shared types and lexer utilities
- `thiserror` for error definitions
- `tokio` (optional, with the "async" feature) for async I/O

## License

Apache-2.0
