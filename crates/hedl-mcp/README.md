# hedl-mcp

**Model Context Protocol server for HEDL—complete AI/LLM integration with 10 tools, caching, rate limiting, and streaming support.**

AI/LLM systems need seamless access to HEDL files: reading documents, validating schemas, converting formats, querying entities, optimizing token usage. `hedl-mcp` provides a production-grade MCP server that bridges AI systems with the HEDL ecosystem through 10 specialized tools, high-performance caching (2-5x speedup), rate limiting (DoS protection), and streaming support for large files.

This is the official Model Context Protocol server for HEDL. Connect any MCP-compatible AI system (Claude Desktop, custom agents, LLM applications) to HEDL with comprehensive tools for validation, conversion, optimization, and querying.

## What's Implemented

Complete MCP server with production-ready infrastructure:

1. **10 MCP Tools**: Read, query, validate, optimize, stats, format, write, convert (to/from), stream
2. **JSON-RPC 2.0 Protocol**: Full MCP specification compliance (protocol version 2024-11-05)
3. **High-Performance Caching**: LRU cache with 2-5x speedup on repeated operations
4. **Rate Limiting**: Token bucket algorithm with 200 burst capacity, 100 req/s sustained
5. **Streaming Support**: Memory-efficient pagination for large documents
6. **Security Features**: Path traversal protection, input size limits (10 MB), safe file operations
7. **Parallel Processing**: Configurable thread pool for directory scanning
8. **Resource Protocol**: List and read HEDL files as MCP resources
9. **Stdio Transport**: Both sync and async modes for flexible integration
10. **Comprehensive Testing**: All tools with valid/invalid inputs, edge cases, caching, rate limiting

## Installation

```bash
# From source
cargo install hedl-mcp

# Or build locally
cd crates/hedl-mcp
cargo build --release
```

Binary location: `target/release/hedl-mcp`

## Quick Start

### Standalone Server

```bash
# Start server with stdio transport
hedl-mcp --stdio

# Server listens on stdin/stdout for JSON-RPC 2.0 messages
```

### Claude Desktop Integration

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or equivalent:

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

**Environment Variables**:
- `HEDL_ROOT` - Root directory for scoped file operations (default: current directory)

### Custom MCP Client Integration

```rust
use hedl_mcp::{McpServer, McpServerConfig};
use std::path::PathBuf;

let config = McpServerConfig {
    root_path: PathBuf::from("/data/hedl"),
    name: "hedl-server".to_string(),
    version: "1.0.0".to_string(),
    rate_limit_burst: 200,         // Burst capacity
    rate_limit_per_second: 100,    // Sustained rate
    cache_size: 1000,              // LRU cache entries
};

let server = McpServer::new(config);
server.run_stdio().await?;  // Async mode
// or
server.run_stdio_sync()?;   // Sync mode
```

## MCP Tools (10 Total)

### 1. hedl_read - Read and Parse HEDL Files

Read HEDL files from directory or specific file with optional JSON representation:

```json
{
  "tool": "hedl_read",
  "arguments": {
    "path": "data/users.hedl"
  }
}
```

**Arguments**:
- `path` (string, required) - File or directory path (scoped to root_path)
- `recursive` (boolean, optional) - Recursively scan directories (default: false)
- `parallel` (boolean, optional) - Use parallel processing (default: false)
- `threads` (number, optional) - Thread count for parallel mode (default: num_cpus)
- `include_json` (boolean, optional) - Include JSON representation in output (default: false)

**Output**:
```json
{
  "files_read": 1,
  "documents": [
    {
      "path": "data/users.hedl",
      "version": "1.0",
      "schemas": ["User", "Post"],
      "entity_count": 125,
      "json": { ... }  // Optional JSON representation
    }
  ]
}
```

**Parallel Processing**: Processes multiple files concurrently with configurable thread pool for maximum throughput.

### 2. hedl_query - Query Entity Registry

Query parsed entities by type and ID with graph-aware nested children support:

```json
{
  "tool": "hedl_query",
  "arguments": {
    "path": "data/users.hedl",
    "type": "User",
    "id": "alice"
  }
}
```

**Arguments**:
- `path` (string, required) - HEDL file path
- `type` (string, optional) - Filter by entity type
- `id` (string, optional) - Filter by entity ID

**Output**:
```json
{
  "matches": [
    {
      "type": "User",
      "id": "alice",
      "fields": ["alice", "Alice Smith", "alice@example.com", "2024-01-15"],
      "line": 15,
      "children": [
        {
          "type": "Post",
          "id": "post1",
          "fields": ["post1", "My First Post", "..."],
          "line": 23
        }
      ]
    }
  ],
  "count": 1
}
```

**Features**:
- Filter by type only: returns all entities of that type
- Filter by ID only: returns entities with matching ID across all types
- Recursive children traversal via %NEST relationships

### 3. hedl_validate - Validate HEDL Documents

Validate syntax, schema, and references with detailed diagnostics:

```json
{
  "tool": "hedl_validate",
  "arguments": {
    "content": "%VERSION: 1.0\n---\nusers: @User[id, name]\n  | alice, Alice",
    "strict": true,
    "lint": true
  }
}
```

**Arguments**:
- `content` (string, required) - HEDL content to validate
- `strict` (boolean, optional) - Treat lint warnings as errors (default: false)
- `lint` (boolean, optional) - Include linting diagnostics (default: true)

**Output** (valid document):
```json
{
  "valid": true,
  "version": "1.0",
  "schemas": 1,
  "entities": 1,
  "diagnostics": []
}
```

**Output** (invalid document):
```json
{
  "valid": false,
  "errors": [
    {
      "line": 3,
      "severity": "error",
      "message": "Parse error: unexpected end of line, expected ']'",
      "rule": null
    }
  ],
  "warnings": [
    {
      "line": 2,
      "severity": "warning",
      "message": "Matrix list 'users' missing count hint",
      "rule": "add-count-hints"
    }
  ]
}
```

**Validation Levels**:
- **Syntax**: Parse errors (missing brackets, invalid tokens)
- **Schema**: Type mismatches, field count errors
- **References**: Unresolved entity references
- **Lint**: Best practices (unused aliases, missing count hints)

### 4. hedl_optimize - JSON → HEDL Optimization

Convert JSON to optimized HEDL format with token savings statistics:

```json
{
  "tool": "hedl_optimize",
  "arguments": {
    "json": "{\"users\": [{\"id\": \"alice\", \"name\": \"Alice\"}]}",
    "enable_ditto": true
  }
}
```

**Arguments**:
- `json` (string, required) - JSON content to convert
- `enable_ditto` (boolean, optional) - Enable ditto optimization (default: true)

**Output**:
```json
{
  "hedl": "users: @User[id, name]\n  | alice, Alice",
  "json_tokens": 150,
  "hedl_tokens": 90,
  "savings": 60,
  "savings_percent": 40.0,
  "optimization": "40% token reduction"
}
```

**Reported Capability**: 40-60% token savings for typical JSON documents

### 5. hedl_stats - Token Usage Statistics

Compare HEDL vs JSON token counts with detailed breakdown:

```json
{
  "tool": "hedl_stats",
  "arguments": {
    "content": "users: @User[id, name]\n  | alice, Alice\n  | bob, Bob",
    "tokenizer": "simple"
  }
}
```

**Arguments**:
- `content` (string, required) - HEDL content to analyze
- `tokenizer` (string, optional) - Tokenizer to use: "simple" or "cl100k" (default: "simple")

**Output**:
```json
{
  "hedl": {
    "bytes": 58,
    "tokens": 15
  },
  "json_compact": {
    "bytes": 95,
    "tokens": 24,
    "increase_percent": 60.0
  },
  "json_pretty": {
    "bytes": 142,
    "tokens": 36,
    "increase_percent": 140.0
  },
  "savings": {
    "vs_compact": 9,
    "vs_pretty": 21,
    "efficiency": "37.5% fewer tokens vs JSON compact"
  }
}
```

**Tokenizers**:
- **simple**: ~4 chars/token heuristic (fast, approximate)
- **cl100k**: OpenAI tiktoken cl100k_base (accurate for GPT models)

### 6. hedl_format - Format to Canonical Form

Canonicalize HEDL with optional ditto optimization:

```json
{
  "tool": "hedl_format",
  "arguments": {
    "content": "users:@User[id,name]\n|alice,Alice",
    "enable_ditto": true
  }
}
```

**Arguments**:
- `content` (string, required) - HEDL content to format
- `enable_ditto` (boolean, optional) - Enable ditto optimization (default: false)

**Output**:
```json
{
  "formatted": "users: @User[id, name]\n  | alice, Alice",
  "canonical": true
}
```

**Canonicalization**:
- Normalizes whitespace (2-space indentation)
- Adds spaces after colons and commas
- Alphabetically sorts header directives
- Optionally compresses repeated values with ditto (`^`)

### 7. hedl_write - Write HEDL Content to File

Write HEDL content with optional validation and formatting:

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

**Arguments**:
- `path` (string, required) - Output file path (scoped to root_path)
- `content` (string, required) - HEDL content to write
- `validate` (boolean, optional) - Validate before writing (default: false)
- `format` (boolean, optional) - Canonicalize before writing (default: false)
- `backup` (boolean, optional) - Create .hedl.bak backup (default: false)

**Output**:
```json
{
  "written": true,
  "path": "output/data.hedl",
  "bytes": 58,
  "backup_created": "output/data.hedl.bak"
}
```

**Safety Features**:
- Path traversal protection (canonicalize + prefix checking)
- Optional validation prevents writing invalid HEDL
- Backup creation preserves existing files

### 8. hedl_convert_to - Convert HEDL to Other Formats

Export HEDL to JSON, YAML, CSV, Parquet (base64), or Cypher (Neo4j):

```json
{
  "tool": "hedl_convert_to",
  "arguments": {
    "content": "users: @User[id, name]\n  | alice, Alice",
    "format": "json",
    "pretty": true
  }
}
```

**Arguments**:
- `content` (string, required) - HEDL content to convert
- `format` (string, required) - Target format: "json", "yaml", "csv", "parquet", "cypher"
- `pretty` (boolean, optional) - Pretty-print output (JSON, YAML) (default: false)
- `headers` (boolean, optional) - Include CSV headers (default: true)
- `include_constraints` (boolean, optional) - Add Cypher constraints (default: false)

**Output**:
```json
{
  "format": "json",
  "content": "{\n  \"users\": [\n    {\"id\": \"alice\", \"name\": \"Alice\"}\n  ]\n}"
}
```

**Parquet Output**: Base64-encoded bytes (binary format)
**Cypher Output**: Neo4j CREATE/MERGE statements with optional constraints

### 9. hedl_convert_from - Convert Other Formats to HEDL

Import JSON, YAML, CSV, or Parquet into HEDL:

```json
{
  "tool": "hedl_convert_from",
  "arguments": {
    "content": "{\"users\": [{\"id\": \"alice\", \"name\": \"Alice\"}]}",
    "format": "json",
    "canonicalize": true
  }
}
```

**Arguments**:
- `content` (string, required) - Source content to convert
- `format` (string, required) - Source format: "json", "yaml", "csv", "parquet"
- `canonicalize` (boolean, optional) - Canonicalize output (default: false)
- `schema` (array of strings, optional) - CSV column names (required for CSV without headers)

**Output**:
```json
{
  "hedl": "users: @User[id, name]\n  | alice, Alice",
  "canonical": true
}
```

**CSV Conversion**: Requires schema for headerless CSV, auto-detects types for values

### 10. hedl_stream - Stream Parse Large Documents

Memory-efficient parsing with pagination and type filtering:

```json
{
  "tool": "hedl_stream",
  "arguments": {
    "path": "large_export.hedl",
    "type_filter": "User",
    "offset": 0,
    "limit": 100
  }
}
```

**Arguments**:
- `path` (string, required) - HEDL file path
- `type_filter` (string, optional) - Filter entities by type
- `offset` (number, optional) - Skip first N entities (default: 0)
- `limit` (number, optional) - Maximum entities to return (default: 1000)

**Output**:
```json
{
  "entities": [
    {
      "type": "User",
      "id": "alice",
      "fields": ["alice", "Alice Smith", "alice@example.com"],
      "line": 15
    }
  ],
  "count": 1,
  "total": 125,
  "has_more": true,
  "next_offset": 100
}
```

**Performance**: O(1) memory per entity—processes multi-gigabyte files without loading entire document.

## High-Performance Caching

LRU cache for immutable operations provides 2-5x speedup on repeated requests:

### Cached Operations

- `hedl_validate` - Validation results for identical content
- `hedl_query` - Entity lookups for unchanged files
- `hedl_stats` - Token statistics for same content

**Cache Key**: Operation name + SHA256 hash of inputs

**Configuration**:
```rust
McpServerConfig {
    cache_size: 1000,  // Max entries (default)
    ...
}
```

**Cache Statistics**:
```json
{
  "cache": {
    "hits": 1523,
    "misses": 478,
    "evictions": 23,
    "hit_rate": 76.1
  }
}
```

**Impact**: Validation of frequently-accessed documents drops from 50ms to <1ms.

## Rate Limiting: DoS Protection

Token bucket algorithm prevents request flooding:

**Configuration**:
```rust
McpServerConfig {
    rate_limit_burst: 200,         // Burst capacity
    rate_limit_per_second: 100,    // Sustained rate
    ...
}
```

**Behavior**:
- **Burst**: Allow up to 200 requests instantly
- **Sustained**: Refill 100 tokens/second
- **Exceeded**: Returns error with retry-after time

**Error Response**:
```json
{
  "error": {
    "code": -32000,
    "message": "Rate limit exceeded. Try again in 0.5s"
  }
}
```

**Use Case**: Prevents aggressive clients from overwhelming server with thousands of requests.

## Security Features

### Path Traversal Protection

All file operations are scoped to configured `root_path`:

```rust
// Canonicalize path and check prefix
let canonical = std::fs::canonicalize(&path)?;
if !canonical.starts_with(&self.config.root_path) {
    return Err(PathTraversalError);
}
```

**Blocked Attempts**:
```
/etc/passwd              → Error: Path traversal detected
../../../secrets.hedl    → Error: Path traversal detected
/data/hedl/users.hedl    → OK (within root_path)
```

### Input Size Validation

**Maximum Input**: 10 MB per request

```rust
if content.len() > 10_000_000 {
    return Err("Input size exceeds 10 MB limit");
}
```

**Purpose**: Prevents memory exhaustion from malicious large inputs.

### Safe File Operations

- All writes create parent directories if needed
- Backup files preserve original content (.hedl.bak)
- Validation prevents writing malformed HEDL

## Resource Protocol Support

List and read HEDL files as MCP resources:

### resources/list

```json
{
  "method": "resources/list"
}
```

**Response**:
```json
{
  "resources": [
    {
      "uri": "file:///data/hedl/users.hedl",
      "name": "users.hedl",
      "mimeType": "application/x-hedl"
    },
    {
      "uri": "file:///data/hedl/config.hedl",
      "name": "config.hedl",
      "mimeType": "application/x-hedl"
    }
  ]
}
```

### resources/read

```json
{
  "method": "resources/read",
  "params": {
    "uri": "file:///data/hedl/users.hedl"
  }
}
```

**Response**:
```json
{
  "contents": [
    {
      "uri": "file:///data/hedl/users.hedl",
      "mimeType": "application/x-hedl",
      "text": "users: @User[id, name]\n  | alice, Alice"
    }
  ]
}
```

## MCP Protocol Implementation

### Supported Methods

**Lifecycle**:
- `initialize` - Protocol handshake with capability negotiation
- `initialized` - Client confirmation notification
- `shutdown` - Graceful server termination

**Tools**:
- `tools/list` - List all 10 available tools with schemas
- `tools/call` - Execute specific tool with arguments

**Resources**:
- `resources/list` - List HEDL files in root_path
- `resources/read` - Read HEDL file content

**Health**:
- `ping` - Health check endpoint (always returns pong)

### Server Capabilities

```json
{
  "capabilities": {
    "tools": {
      "listChanged": false
    },
    "resources": {
      "subscribe": false,
      "listChanged": false
    }
  },
  "protocolVersion": "2024-11-05",
  "serverInfo": {
    "name": "hedl-mcp",
    "version": "1.0.0"
  }
}
```

## Error Handling

Comprehensive error types with MCP error codes:

**Error Codes**:
- `-32700`: Parse error (invalid JSON-RPC)
- `-32600`: Invalid request (malformed method/params)
- `-32601`: Method not found
- `-32000`: Server error (HEDL-specific errors)

**HEDL-Specific Errors**:
```json
{
  "code": -32000,
  "message": "Parse error at line 5: unexpected token",
  "data": {
    "line": 5,
    "column": 12
  }
}
```

**Tool Errors**: Returned as successful responses with `is_error: true`:
```json
{
  "content": [
    {
      "type": "text",
      "text": "Error: Path traversal detected"
    }
  ],
  "isError": true
}
```

## Use Cases

**AI-Powered HEDL Editing**: LLMs read HEDL files, suggest edits, validate changes, write back canonical HEDL—all through MCP tools.

**Automated Optimization**: AI agents convert JSON to HEDL, analyze token savings, optimize with ditto, validate output, deploy optimized configs.

**Interactive Query Assistant**: Chat with HEDL data—ask "Show me all users created in 2024", agent uses hedl_query to fetch entities, formats response.

**Multi-Format Pipeline**: Agent reads CSV exports, converts to HEDL via hedl_convert_from, validates schemas, exports to Neo4j Cypher, tracks token efficiency.

**Bulk Document Processing**: AI orchestrates parallel HEDL validation across directories, aggregates lint issues, generates summary reports.

**LLM Context Optimization**: Analyze HEDL vs JSON token usage with hedl_stats, optimize prompts with hedl_optimize, validate context window limits.

## What This Crate Doesn't Do

**Direct File Watching**: No file system watching—client must explicitly call tools to check for changes. Use inotify/fsevents in client if needed.

**Multi-Document Transactions**: Each tool call is independent—no transactional updates across multiple files. Implement transactions in client layer if required.

**Schema Evolution**: No automatic schema migration—manual handling of %STRUCT changes required. Use hedl-lint for schema consistency validation.

**Distributed Coordination**: Single-server design—no distributed consensus or multi-server coordination. Deploy multiple instances behind load balancer if needed.

## Performance Characteristics

**Tool Execution**: ~50-200 MB/s parsing throughput depending on complexity

**Caching**: 2-5x speedup for repeated validation/query operations

**Streaming**: O(1) memory per entity regardless of file size

**Rate Limiting**: <0.1ms overhead per request (~1% impact)

**Parallel Processing**: Linear speedup up to CPU core count for directory scanning

Detailed performance benchmarks are available in the HEDL repository benchmark suite.

## Testing Coverage

Comprehensive test suite covering:
- **All 10 Tools**: Valid/invalid inputs, edge cases, error conditions
- **Caching**: Hits, misses, LRU eviction, statistics
- **Rate Limiting**: Burst capacity, sustained rate, token refill, overflow
- **Security**: Path traversal detection, input size validation
- **Parallel Processing**: Custom threads, recursive scanning, error collection
- **Format Conversions**: Round-trip fidelity for all supported formats

## Dependencies

- `hedl-core` 1.0 - HEDL parsing and data model
- `hedl-c14n` 1.0 - Canonicalization
- `hedl-lint` 1.0 - Best practices linting
- `hedl-json`, `hedl-yaml`, `hedl-xml`, `hedl-csv`, `hedl-parquet`, `hedl-neo4j` 1.0 - Format conversion
- `serde` 1.0, `serde_json` 1.0 - JSON serialization
- `tokio` 1.35 - Async runtime
- `dashmap` 5.5 - Concurrent HashMap for caching
- `rayon` 1.8 - Parallel processing
- `thiserror` 1.0 - Error type definitions

## License

Apache-2.0
