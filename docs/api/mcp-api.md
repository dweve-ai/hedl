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

## Authentication

The MCP server supports two authentication methods for production deployments.

### OAuth2 Authentication

Full OAuth2 flow for external integrations:

```bash
# Start with OAuth2 enabled
hedl-mcp --auth oauth2 \
    --oauth2-client-id "your-client-id" \
    --oauth2-client-secret "your-client-secret" \
    --oauth2-auth-url "https://auth.example.com/authorize" \
    --oauth2-token-url "https://auth.example.com/token"
```

**Provider Configuration**:
```rust
pub struct OAuth2Provider {
    pub issuer: String,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub introspection_endpoint: String,
}
```

**Supported Flows**:
- Token introspection (validates bearer tokens against provider)

### API Key Authentication

Simple API key authentication for internal services:

```bash
# Start with API key auth
hedl-mcp --auth api-key --api-keys "key1,key2,key3"

# Or load from file
hedl-mcp --auth api-key --api-keys-file /path/to/keys.txt
```

**Request Header**:
```
Authorization: Bearer <api-key>
```

**Key Management**:

The MCP server uses a trait-based API key storage system:

```rust
// API Key authentication handler
pub struct ApiKeyAuth {
    key_store: Arc<dyn ApiKeyStore>,
    key_prefix: Option<String>,  // e.g., "hedl_"
}

// Storage backends: InMemoryApiKeyStore, FileApiKeyStore
pub trait ApiKeyStore: Send + Sync {
    async fn validate(&self, key: &str) -> Result<ClientMetadata, AuthError>;
    async fn create(&self, client_id: &str, scopes: Vec<String>) -> Result<String, AuthError>;
    async fn revoke(&self, key: &str) -> Result<(), AuthError>;
    async fn list_for_client(&self, client_id: &str) -> Result<Vec<ApiKeyInfo>, AuthError>;
}
```

### JWT Authentication

JSON Web Token authentication for stateless verification:

```bash
hedl-mcp --auth jwt \
    --jwt-secret "your-secret-key" \
    --jwt-issuer "https://auth.example.com"
```

### mTLS Authentication

Mutual TLS for certificate-based authentication:

```bash
hedl-mcp --auth mtls \
    --tls-cert /path/to/server.crt \
    --tls-key /path/to/server.key \
    --tls-ca /path/to/ca.crt
```

### No Authentication (Development)

For local development only:

```bash
hedl-mcp --auth none  # Default for localhost
```

---

## Resource Limits

Configurable limits prevent resource exhaustion:

```bash
hedl-mcp \
    --max-file-size 500mb \
    --max-request-size 10mb \
    --max-concurrent 100 \
    --max-memory 2gb
```

**Configuration**:
```rust
pub struct ResourceLimitConfig {
    pub enabled: bool,
    pub request: RequestSizeConfig,      // max_total_size: 10 MB, max_param_size: 5 MB
    pub response: ResponseSizeConfig,    // max_total_size: 50 MB, max_result_items: 100K
    pub rate_limiting: RateLimitingConfig, // 200 burst, 100 req/sec
    pub memory: MemoryConfig,            // 100 MB cache, 50 MB per operation
    pub concurrency: ConcurrencyConfig,  // 100 global, 10 per client, 50 per tool
    pub timeouts: TimeoutConfig,         // 30s default, per-tool overrides
}
```

**Enforcement**:
- Requests exceeding limits return `429 Too Many Requests` or `413 Payload Too Large`
- Memory usage monitored per-request
- Concurrent request queuing when limit reached

---

## Available Tools

The MCP server provides 11 tools for HEDL operations:

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
    "files_read": 1,
    "results": [
        {
            "file": "users.hedl",
            "version": "1.0",
            "schemas": ["User", "Product"],
            "aliases": 2,
            "nests": 3,
            "entities": 150,
            "data": "{...}"           // If include_json=true
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
        "json_tokens": 1000,
        "hedl_tokens": 450,
        "savings_percent": 55,
        "tokens_saved": 550
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
```
%VERSION: 1.0
...
(Formatted HEDL content directly as text)
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
    "format": "json",                 // Required: "json", "yaml", "xml", "csv", "parquet", "cypher", "toon"
    "options": {                      // Optional: Format-specific options
        "pretty": true
    }
}
```

**Returns**:
- For most formats: Converted content directly as text
- For `parquet` (binary): Base64-encoded JSON response:
```json
{
    "parquet_base64": "...",
    "bytes": 2048
}
```

**Supported Formats**:
- `json`: JavaScript Object Notation
- `yaml`: YAML Ain't Markup Language
- `xml`: Extensible Markup Language
- `csv`: Comma-Separated Values
- `parquet`: Apache Parquet (binary, returns base64)
- `cypher`: Neo4j Cypher statements
- `toon`: TOON (Tiny Object Oriented Notation)

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
    "hedl": "%VERSION: 1.0\n...",
    "entities": 150
}
```

**Supported Formats**:
- `json`: JavaScript Object Notation
- `yaml`: YAML Ain't Markup Language
- `xml`: Extensible Markup Language
- `csv`: Comma-Separated Values (with type_name and optional schema in options)
- `parquet`: Apache Parquet (binary, base64-encoded input)
- `toon`: TOON (Tiny Object Oriented Notation)

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

### 11. `batch`

Execute multiple operations in a single request for better throughput.

**Input Schema**:
```json
{
    "operations": [                    // Required: Array of operations
        {
            "id": "op1",               // Required: Unique operation identifier
            "tool": "hedl_read",       // Required: Tool name (e.g., "hedl_validate", "hedl_format")
            "arguments": {"path": "users.hedl"},  // Optional: Tool arguments
            "depends_on": []           // Optional: Array of operation IDs to wait for
        },
        {
            "id": "op2",               // Required: Unique operation identifier
            "tool": "hedl_validate",   // Required: Tool name
            "arguments": {"hedl": "..."},  // Optional: Tool arguments
            "depends_on": ["op1"]      // Optional: This operation depends on op1 completing
        }
    ],
    "mode": "continue_on_error",       // Optional: "continue_on_error" (default) or "stop_on_error"
    "parallel": true,                  // Optional: Execute in parallel for independent operations (default: true)
    "transaction": false,              // Optional: All-or-nothing transaction semantics (default: false)
    "timeout": 300                     // Optional: Maximum execution time in seconds (1-3600)
}
```

**Returns**:
```json
{
    "success": true,
    "results": [
        {
            "id": "op1",
            "tool": "hedl_read",
            "success": true,
            "result": {...}
        },
        {
            "id": "op2",
            "tool": "hedl_validate",
            "success": false,
            "error": {...}
        }
    ],
    "summary": {
        "total": 2,
        "succeeded": 1,
        "failed": 1,
        "duration_ms": 45
    }
}
```

**Operation Fields**:
- `id` (required): Unique identifier for result correlation and dependency resolution
- `tool` (required): Name of the tool to execute (e.g., "hedl_validate", "hedl_format")
- `arguments` (optional): Tool arguments as JSON object matching the tool's input schema
- `depends_on` (optional): Array of operation IDs that must complete successfully before this operation executes. Circular dependencies are detected and rejected.

**Execution Modes**:
- `continue_on_error`: Continue executing remaining operations even if some fail (default)
- `stop_on_error`: Stop batch execution on the first error

**Transaction Mode**:
- `transaction: false` (default): Failures don't affect other operations
- `transaction: true`: All-or-nothing semantics; any failure rolls back the entire batch

**Parallel Execution**:
- When `parallel: true`, independent operations (without dependencies) execute concurrently
- Operations with dependencies execute in topologically sorted order
- Circular dependencies are detected and rejected with an error

**Use Cases**:
- Bulk file processing with dependency chains
- Multi-format conversion pipelines
- Validation of multiple documents
- Complex workflows with sequential and parallel steps

---

## MCP Protocol

### Server Info

```json
{
    "name": "hedl-mcp",
    "version": "1.2.0",
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
    pub size: usize,
    pub max_size: usize,
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

**Error Codes** (JSON-RPC style):

| Code | Name | Description |
|------|------|-------------|
| -32001 | Parse | HEDL parsing failed |
| -32002 | Io | File I/O error |
| -32003 | PathTraversal | Path traversal attempt blocked |
| -32004 | FileNotFound | File not found |
| -32005 | ResourceLimit | Resource limit exceeded |
| -32600 | InvalidRequest | Invalid request structure |
| -32601 | ToolNotFound | Unknown tool name |
| -32602 | InvalidArguments | Invalid tool arguments |
| -32603 | ResourceNotFound | Resource not found |
| -32700 | Json | JSON serialization error |

---

## Programmatic Usage

### Rust

```rust
use hedl_mcp::{McpServer, McpServerConfig};

#[tokio::main]
async fn main() {
    let config = McpServerConfig {
        root_path: "/path/to/data".into(),
        name: "hedl-mcp".to_string(),
        version: "1.2.0".to_string(),
        rate_limit_burst: 200,        // Burst capacity
        rate_limit_per_second: 100,   // Sustained rate
        cache_size: 1000,
    };

    let mut server = McpServer::new(config);
    server.run_stdio_async().await.unwrap();
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
