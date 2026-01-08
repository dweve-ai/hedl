# MCP Server API Reference

**Model Context Protocol server for AI/LLM integration**

---

## Overview

The HEDL MCP server provides a standardized interface for AI/LLM systems to interact with HEDL files and data. It implements the Model Context Protocol specification, allowing seamless integration with Claude, GPT-4, and other LLM tools.

---

## Quick Start

### Installation

```bash
# Install from source
cargo install hedl-mcp

# Or build locally
cargo build --release --bin hedl-mcp
```

### Running the Server

```bash
# Serve HEDL files from a directory
hedl-mcp --root /path/to/hedl/files

# Use synchronous mode (not recommended)
hedl-mcp --async false
```

### Configuration

Configuration is primarily handled via CLI arguments. Security limits and other settings use defaults optimized for production AI workloads.

---

## Available Tools

The MCP server provides 10 tools for HEDL operations:

### 1. `hedl_read`

Read and parse HEDL files from a directory.

**Input Schema**:
```json
{
    "path": "path/to/file.hedl",     // Required: File or directory path
    "recursive": true,                // Optional: Recursive directory search (default: true)
    "include_json": false             // Optional: Include JSON in response (default: false)
}
```

**Returns**:
```json
{
    "files": [
        {
            "path": "users.hedl",
            "size": 1024,
            "version": "1.0",
            "schemas": 3,
            "entities": 150,
            "json": "{...}"           // If include_json=true
        }
    ]
}
```

**Example**:
```json
{
    "name": "hedl_read",
    "arguments": {
        "path": "data/users.hedl",
        "include_json": true
    }
}
```

---

### 2. `hedl_query`

Query entities by type and/or ID with graph-aware lookups.

**Input Schema**:
```json
{
    "hedl": "...",                    // Required: HEDL content to query
    "type_name": "User",              // Optional: Filter by type
    "id": "alice",                    // Optional: Filter by ID
    "include_children": true          // Optional: Include nested children (default: true)
}
```

**Returns**:
```json
{
    "matches": [
        {
            "type": "User",
            "id": "alice",
            "fields": {
                "name": "Alice Smith",
                "email": "alice@example.com"
            },
            "children": {
                "Post": [...]
            }
        }
    ]
}
```

**Example**:
```json
{
    "name": "hedl_query",
    "arguments": {
        "hedl": "%VERSION: 1.0\n...",
        "type_name": "User",
        "id": "alice"
    }
}
```

---

### 3. `hedl_validate`

Validate HEDL input and return detailed diagnostics.

**Input Schema**:
```json
{
    "hedl": "...",                    // Required: HEDL content
    "strict": true,                   // Optional: Strict mode (default: true)
    "lint": true                      // Optional: Run linter (default: true)
}
```

**Returns**:
```json
{
    "valid": true,
    "errors": [],
    "warnings": [
        {
            "line": 10,
            "message": "Unused alias",
            "severity": "warning",
            "rule": "unused-alias"
        }
    ]
}
```

---

### 4. `hedl_optimize`

Convert JSON to optimized HEDL format (40-60% token savings).

**Input Schema**:
```json
{
    "json": "{...}",                  // Required: JSON content
    "ditto": true,                    // Optional: Use ditto operator (default: true)
    "compact": false                  // Optional: Minimize whitespace (default: false)
}
```

**Returns**:
```json
{
    "hedl": "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n",
    "stats": {
        "original_tokens": 1000,
        "hedl_tokens": 450,
        "savings_percent": 55
    }
}
```

**Example**:
```json
{
    "name": "hedl_optimize",
    "arguments": {
        "json": "{\"users\": [{\"id\": \"alice\", \"name\": \"Alice\"}]}",
        "ditto": true
    }
}
```

---

### 5. `hedl_stats`

Get token usage statistics comparing HEDL vs JSON.

**Input Schema**:
```json
{
    "hedl": "...",                    // Required: HEDL content
    "tokenizer": "simple"             // Optional: "cl100k" or "simple" (default: "simple")
}
```

**Returns**:
```json
{
    "hedl": {
        "bytes": 500,
        "tokens": 125,
        "lines": 20
    },
    "json": {
        "bytes": 1200,
        "tokens": 300
    },
    "savings": {
        "percent": 58,
        "tokens": 175
    }
}
```

---

### 6. `hedl_format`

Format HEDL to canonical form.

**Input Schema**:
```json
{
    "hedl": "...",                    // Required: HEDL content
    "ditto": true                     // Optional: Apply ditto optimization (default: true)
}
```

**Returns**:
```json
{
    "formatted": "%VERSION: 1.0\n..."
}
```

---

### 7. `hedl_write`

Write HEDL content to a file.

**Input Schema**:
```json
{
    "path": "output.hedl",            // Required: Output file path
    "content": "...",                 // Required: HEDL content
    "validate": true,                 // Optional: Validate before write (default: true)
    "format": true,                   // Optional: Format before write (default: false)
    "backup": true                    // Optional: Create backup (default: false)
}
```

**Returns**:
```json
{
    "success": true,
    "path": "/full/path/to/output.hedl",
    "bytes_written": 1024
}
```

---

### 8. `hedl_convert_to`

Convert HEDL to other formats.

**Input Schema**:
```json
{
    "hedl": "...",                    // Required: HEDL content
    "format": "json",                 // Required: "json", "yaml", "xml", "csv", "parquet", "cypher"
    "options": {                      // Optional: Format-specific options
        "pretty": true
    }
}
```

**Returns**:
```json
{
    "output": "{...}",
    "format": "json"
}
```

**Supported Formats**:
- `json`: JavaScript Object Notation
- `yaml`: YAML Ain't Markup Language
- `xml`: Extensible Markup Language
- `csv`: Comma-Separated Values
- `parquet`: Apache Parquet (binary)
- `cypher`: Neo4j Cypher statements

---

### 9. `hedl_convert_from`

Convert other formats to HEDL.

**Input Schema**:
```json
{
    "content": "...",                 // Required: Source content
    "format": "json"                  // Required: Source format
}
```

**Returns**:
```json
{
    "hedl": "%VERSION: 1.0\n..."
}
```

---

### 10. `hedl_stream`

Stream parse a large HEDL document with pagination.

**Input Schema**:
```json
{
    "hedl": "...",                    // Required: HEDL content
    "limit": 100,                     // Optional: Max entities to return (default: 100)
    "offset": 0,                      // Optional: Number of entities to skip (default: 0)
    "type_filter": "User"             // Optional: Only return entities of this type
}
```

**Returns**:
```json
{
    "entities": [
        {
            "type": "User",
            "id": "alice",
            "fields": [...]
        }
    ],
    "count": 1,
    "offset": 0,
    "limit": 100
}
```

---

## MCP Protocol

### Server Info

```json
{
    "name": "hedl-mcp",
    "version": "0.1.0",
    "protocol_version": "1.0"
}
```

### Capabilities

```json
{
    "tools": true,
    "resources": false,
    "prompts": false
}
```

---

## Performance Features

### Caching

The MCP server implements intelligent caching:

- **File Content Cache**: Parsed documents cached by file path
- **Query Result Cache**: Entity query results cached by parameters
- **LRU Eviction**: Least-recently-used items evicted when cache is full
- **Configurable Size**: Default 1000 entries, configurable via `--cache-size`

**Cache Statistics**:
```rust
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub current_size: usize,
}
```

---

### Rate Limiting

Protect against excessive requests:

- **Token Bucket Algorithm**: Smooth rate limiting
- **Per-Tool Limits**: Different limits for different operations
- **Configurable**: Default 50 requests/second, configurable via `--rate-limit`

```bash
# Allow 100 requests per second
hedl-mcp --rate-limit 100
```

---

## Error Handling

All tools return errors in a consistent format:

```json
{
    "error": {
        "code": "parse_error",
        "message": "Syntax error at line 10: unexpected token",
        "details": {
            "line": 10,
            "column": 5
        }
    }
}
```

**Common Error Codes**:
- `parse_error`: HEDL parsing failed
- `validation_error`: Validation failed
- `io_error`: File I/O error
- `conversion_error`: Format conversion failed
- `rate_limit_exceeded`: Too many requests
- `cache_error`: Cache operation failed

---

## Programmatic Usage

### Rust

```rust
use hedl_mcp::{McpServer, McpServerConfig};

#[tokio::main]
async fn main() {
    let config = McpServerConfig {
        root_dir: "/path/to/data".into(),
        cache_size: 1000,
        rate_limit: 50,
    };

    let server = McpServer::new(config);
    server.run().await.unwrap();
}
```

---

### Integration with Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
    "mcpServers": {
        "hedl": {
            "command": "hedl-mcp",
            "args": [
                "--root",
                "/path/to/hedl/files"
            ]
        }
    }
}
```

---

### Integration with Custom MCP Client

```typescript
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const transport = new StdioClientTransport({
    command: 'hedl-mcp',
    args: ['--root', '/path/to/data']
});

const client = new Client({
    name: 'hedl-client',
    version: '1.0.0'
}, {
    capabilities: {}
});

await client.connect(transport);

// Call tool
const result = await client.request({
    method: 'tools/call',
    params: {
        name: 'hedl_read',
        arguments: {
            path: 'users.hedl'
        }
    }
});

console.log(result);
```

---

## Use Cases

### 1. AI-Assisted Data Analysis

```json
{
    "name": "hedl_read",
    "arguments": {
        "path": "analytics/user_behavior.hedl",
        "include_json": true
    }
}
```

LLM can analyze the data structure and provide insights.

---

### 2. Context Optimization for LLMs

```json
{
    "name": "hedl_optimize",
    "arguments": {
        "json": "{large_json_context}"
    }
}
```

Reduce token usage by 40-60% before injecting into LLM context.

---

### 3. Data Validation in Pipelines

```json
{
    "name": "hedl_validate",
    "arguments": {
        "hedl": "...",
        "strict": true,
        "lint": true
    }
}
```

Automated validation in data processing pipelines.

---

### 4. Graph Query for Knowledge Bases

```json
{
    "name": "hedl_query",
    "arguments": {
        "hedl": "...",
        "type_name": "Concept",
        "include_children": true
    }
}
```

Navigate knowledge graphs with entity references.

---

## Security Considerations

### File System Access

The server restricts access to:
- Only files within the configured `root` directory
- Files with allowed extensions (`.hedl`, `.json`, etc.)
- Files below the maximum size limit

**Path Traversal Protection**:
```rust
// Automatically prevents "../../../etc/passwd"
let safe_path = server.resolve_path(user_provided_path)?;
```

---

### Resource Limits

- **Maximum file size**: 500 MB (configurable)
- **Maximum cache size**: 1000 entries (configurable)
- **Rate limiting**: 50 req/sec (configurable)
- **Parsing limits**: Configurable depth, key count, etc.

---

### Input Validation

All inputs are validated before processing:
- UTF-8 encoding verification
- Size limit checks
- Path sanitization
- Schema validation

---

## Monitoring

### Metrics

The server exposes metrics for monitoring:

```json
{
    "requests_total": 1000,
    "requests_success": 950,
    "requests_error": 50,
    "cache_hit_rate": 0.75,
    "avg_response_time_ms": 45
}
```

---

### Logging

Structured JSON logging for easy parsing:

```json
{
    "timestamp": "2025-01-06T10:30:00Z",
    "level": "info",
    "tool": "hedl_read",
    "duration_ms": 23,
    "status": "success"
}
```

---

## Best Practices

### 1. Use Caching Effectively

```json
// First call: cache miss
{"name": "hedl_read", "arguments": {"path": "data.hedl"}}

// Second call: cache hit (fast)
{"name": "hedl_read", "arguments": {"path": "data.hedl"}}
```

---

### 2. Batch Related Operations

```json
// Good: Single read with all data
{"name": "hedl_read", "arguments": {"path": "users.hedl", "include_json": true}}

// Less efficient: Multiple queries
{"name": "hedl_query", "arguments": {"type": "User", "id": "alice"}}
{"name": "hedl_query", "arguments": {"type": "User", "id": "bob"}}
```

---

### 3. Validate Before Processing

```json
// Step 1: Validate
{"name": "hedl_validate", "arguments": {"hedl": "..."}}

// Step 2: If valid, process
{"name": "hedl_convert_to", "arguments": {"hedl": "...", "format": "json"}}
```

---

**Next**: [LSP API Reference](lsp-api.md)
