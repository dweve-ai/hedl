# hedl-bench

Benchmarks and performance testing for HEDL.

## Running Benchmarks

```bash
cargo bench -p hedl-bench
```

## Benchmark Categories

### Core Parsing
- Lexer performance
- Parser throughput
- Validation speed

### Format Conversion
- JSON roundtrip
- YAML roundtrip
- XML roundtrip
- Parquet I/O

### Features
- Streaming parser
- Reference resolution
- Canonicalization

### Tools
- LSP operations
- MCP tool calls
- Linting performance

## Generating Reports

```bash
cargo bench -p hedl-bench -- --save-baseline main
cargo bench -p hedl-bench -- --baseline main
```

## Custom Benchmarks

```rust
use hedl_bench::{generate_users, generate_blog};

let users = generate_users(1000);
let blog = generate_blog(10, 100);
```

## License

Apache-2.0
