# hedl-mcp

**Model Context Protocol server for HEDL: AI systems deserve structured data that doesn't waste tokens.**

LLMs work with context windows measured in tokens, and every token counts. JSON is verbose, repetitive, and expensive. HEDL delivers the same structured data in 56% fewer tokens on average, but AI systems need a way to work with HEDL files natively.

That's what `hedl-mcp` provides: a complete MCP server that gives any compatible AI system (Claude Desktop, custom agents, LLM applications) the ability to read, validate, convert, query, and optimize HEDL documents. Eleven specialized tools cover the full lifecycle from parsing to persistence, backed by high-performance caching and rate limiting to keep things fast and safe.

## Getting Started

```bash
cargo install hedl-mcp
```

Or build from source:

```bash
cd crates/hedl-mcp
cargo build --release
# Binary at target/release/hedl-mcp
```

### Running the Server

The server uses stdio transport for JSON-RPC 2.0 communication:

```bash
hedl-mcp --stdio
```

### Claude Desktop Integration

Add this to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "hedl": {
      "command": "/path/to/hedl-mcp",
      "args": ["--stdio"],
      "env": {
        "HEDL_ROOT": "/path/to/hedl/documents"
      }
    }
  }
}
```

The `HEDL_ROOT` environment variable scopes all file operations to a specific directory for security.

### Programmatic Integration

```rust
use hedl_mcp::{McpServer, McpServerConfig};
use std::path::PathBuf;

let config = McpServerConfig {
    root_path: PathBuf::from("/data/hedl"),
    name: "hedl-server".to_string(),
    version: "2.0.0".to_string(),
    rate_limit_burst: 200,
    rate_limit_per_second: 100,
    cache_size: 1000,
};

let server = McpServer::new(config);
server.run_stdio().await?;  // Async mode
// or
server.run_stdio_sync()?;   // Sync mode
```

## Tools

### hedl_read

Read and parse HEDL files from the filesystem. Handles both individual files and recursive directory scans with parallel processing.

```json
{
  "tool": "hedl_read",
  "arguments": {
    "path": "data/users.hedl",
    "recursive": true,
    "include_json": false,
    "num_threads": 4
  }
}
```

Returns parsed document metadata including schemas, entity counts, and optionally the JSON representation of the data.

### hedl_query

Query entities by type and ID with graph-aware nested children support. Understands %NEST relationships for hierarchical data traversal.

```json
{
  "tool": "hedl_query",
  "arguments": {
    "hedl": "%V:2.0\n---\nusers: @User[id, name]\n  | alice, Alice Smith",
    "type_name": "User",
    "id": "alice",
    "include_children": true
  }
}
```

### hedl_validate

Full document validation with syntax checking, schema validation, reference resolution, and optional linting. Returns detailed diagnostics with line numbers and severity levels.

```json
{
  "tool": "hedl_validate",
  "arguments": {
    "hedl": "%V:2.0\n---\nusers: @User[id, name]\n  | alice, Alice",
    "strict": true,
    "lint": true
  }
}
```

### hedl_optimize

Convert JSON to HEDL and see exactly how many tokens you're saving. The output includes before/after token counts and percentage savings.

```json
{
  "tool": "hedl_optimize",
  "arguments": {
    "json": "{\"users\": [{\"id\": \"alice\", \"name\": \"Alice\"}]}",
    "compact": false
  }
}
```

### hedl_stats

Compare HEDL vs JSON token usage with your choice of tokenizer: a fast 4-chars-per-token heuristic, or OpenAI's cl100k_base for GPT-accurate counts.

```json
{
  "tool": "hedl_stats",
  "arguments": {
    "hedl": "users: @User[id, name]\n  | alice, Alice\n  | bob, Bob",
    "tokenizer": "cl100k"
  }
}
```

### hedl_format

Canonicalize HEDL documents to consistent formatting: 2-space indentation, normalized whitespace, sorted header directives.

```json
{
  "tool": "hedl_format",
  "arguments": {
    "hedl": "users:@User[id,name]\n|alice,Alice"
  }
}
```

### hedl_write

Write HEDL content to files with optional pre-write validation, canonicalization, and backup creation. Path traversal protection keeps writes scoped to the configured root.

```json
{
  "tool": "hedl_write",
  "arguments": {
    "path": "output/data.hedl",
    "content": "users: @User[id, name]\n  | alice, Alice",
    "validate": true,
    "format": true,
    "backup": true
  }
}
```

### hedl_convert_to

Export HEDL to JSON, YAML, XML, CSV, Parquet (base64), Cypher (Neo4j), or TOON. Each format has specific options for pretty-printing, headers, metadata inclusion, and more.

```json
{
  "tool": "hedl_convert_to",
  "arguments": {
    "hedl": "users: @User[id, name]\n  | alice, Alice",
    "format": "json",
    "options": { "pretty": true }
  }
}
```

### hedl_convert_from

Import from JSON, YAML, XML, CSV, Parquet, or TOON into HEDL. Automatically infers schemas and types from the source data.

```json
{
  "tool": "hedl_convert_from",
  "arguments": {
    "content": "{\"users\": [{\"id\": \"alice\", \"name\": \"Alice\"}]}",
    "format": "json",
    "options": { "canonicalize": true }
  }
}
```

### hedl_stream

Memory-efficient parsing with pagination for large documents. Process millions of entities without loading everything into memory.

```json
{
  "tool": "hedl_stream",
  "arguments": {
    "hedl": "...",
    "type_filter": "User",
    "offset": 0,
    "limit": 100
  }
}
```

### batch

Execute multiple operations in a single request. Supports dependency resolution between operations, parallel execution of independent tasks, transaction semantics, and timeout protection.

```json
{
  "tool": "batch",
  "arguments": {
    "operations": [
      { "id": "v1", "tool": "hedl_validate", "arguments": { "hedl": "..." } },
      { "id": "f1", "tool": "hedl_format", "arguments": { "hedl": "..." }, "depends_on": ["v1"] }
    ],
    "mode": "continue_on_error",
    "parallel": true,
    "transaction": false,
    "timeout": 30
  }
}
```

## Caching and Rate Limiting

Immutable operations like validation, queries, and stats are cached using an LRU cache keyed by SHA256 hashes. Repeated requests for the same content return cached results in under 1ms instead of re-parsing.

Rate limiting uses a token bucket algorithm (200 burst, 100/sec sustained by default) to prevent runaway clients from overwhelming the server. Exceeded limits return a retry-after time.

## Security

All file operations are scoped to the configured `root_path` using canonicalized path checking. Attempts to escape via `../` sequences are blocked. Input size is capped at 10MB to prevent memory exhaustion.

## Resource Protocol

The server also supports the MCP resource protocol, allowing clients to list and read HEDL files directly:

```json
{"method": "resources/list"}
{"method": "resources/read", "params": {"uri": "file:///data/hedl/users.hedl"}}
```

## Real-World Usage

AI agents can use these tools to build sophisticated HEDL workflows: convert JSON configs to HEDL for 40% token savings, validate documents before deployment, query specific entities for targeted responses, stream through large datasets without memory pressure, and batch multiple operations for efficiency.

The server handles the complexity of parsing, validation, and format conversion so AI systems can focus on reasoning about the data.

## License

Apache-2.0
