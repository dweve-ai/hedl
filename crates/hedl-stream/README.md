# hedl-stream

Streaming parser for processing large HEDL files.

## Installation

```toml
[dependencies]
hedl-stream = "1.0"
```

## Usage

```rust
use hedl_stream::{StreamingParser, Event};
use std::fs::File;

let file = File::open("large.hedl")?;
let parser = StreamingParser::new(file);

for event in parser {
    match event? {
        Event::StartDocument(header) => println!("Version: {:?}", header.version),
        Event::StartNode(key) => println!("Node: {}", key),
        Event::Value(val) => println!("Value: {:?}", val),
        Event::EndNode => println!("End node"),
        Event::EndDocument => break,
    }
}
```

## Features

- **Memory efficient** - Process files larger than RAM
- **Event-based** - SAX-style streaming API
- **Async support** - Async/await compatible
- **Backpressure** - Control parsing rate

## Async Usage

```rust
use hedl_stream::AsyncStreamingParser;
use tokio::fs::File;

let file = File::open("large.hedl").await?;
let mut parser = AsyncStreamingParser::new(file);

while let Some(event) = parser.next().await {
    // Process event
}
```

## License

Apache-2.0
