# Authentication

## Overview

HEDL API authentication varies by interface:

- **Rust API**: No authentication required (library dependency)
- **FFI API**: No authentication required (linked library)
- **WASM API**: No authentication required (browser/Node.js module)
- **MCP Server**: Rate limiting and caching only (no authentication features)
- **LSP Server**: No authentication (local editor integration)

## MCP Server Configuration

The HEDL Model Context Protocol (MCP) server does NOT support authentication features like API keys or IP filtering. It provides:

- **Rate limiting**: Token bucket algorithm for DoS protection
- **Caching**: LRU cache for immutable operations
- **Root path scoping**: File operations restricted to configured directory

### Configuration

The MCP server is configured via `McpServerConfig`:

```rust
use hedl_mcp::{McpServer, McpServerConfig};
use std::path::PathBuf;

let config = McpServerConfig {
    // Root path for file operations (required)
    root_path: PathBuf::from("/data/hedl"),

    // Server identification
    name: "hedl-mcp".to_string(),
    version: "0.1.0".to_string(),

    // Rate limiting (token bucket)
    rate_limit_burst: 200,         // Maximum burst size
    rate_limit_per_second: 100,    // Sustained rate (requests/sec)

    // Caching for immutable operations
    cache_size: 1000,              // LRU cache entries

    ..Default::default()
};

let server = McpServer::new(config);
```

### Rate Limiting

The MCP server uses a token bucket algorithm to prevent DoS attacks:

```rust
let config = McpServerConfig {
    // Maximum burst size (tokens in bucket)
    rate_limit_burst: 200,

    // Sustained rate (tokens refilled per second)
    rate_limit_per_second: 100,

    // Set both to 0 to disable rate limiting
    ..Default::default()
};
```

**How it works**:
1. Bucket starts with `rate_limit_burst` tokens
2. Each request consumes 1 token
3. Tokens refill at `rate_limit_per_second` rate
4. Requests are rejected when bucket is empty

Example with proper HEDL syntax:
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
```

### Caching

The MCP server caches results of immutable operations (validate, lint, analyze_schema):

```rust
let config = McpServerConfig {
    // Number of LRU cache entries
    cache_size: 1000,

    // Set to 0 to disable caching
    ..Default::default()
};
```

**Benefits**:
- 2-5x speedup on repeated requests
- Thread-safe using DashMap
- Automatic eviction of oldest entries

## LSP Server Authentication

The Language Server Protocol (LSP) server runs locally and does not require authentication. It communicates with the editor over stdio or local sockets.

### Editor Configuration

#### VS Code
```json
{
  "hedl.lsp.enable": true,
  "hedl.lsp.path": "/path/to/hedl-lsp"
}
```

#### Neovim
```lua
require('lspconfig').hedl.setup{
  cmd = { "/path/to/hedl-lsp" },
}
```

## Security Best Practices

### File System Security

The MCP server restricts all file operations to the configured `root_path`:

```rust
use hedl_mcp::{McpServer, McpServerConfig};
use std::path::PathBuf;

let config = McpServerConfig {
    // All file operations scoped to this directory
    root_path: PathBuf::from("/var/hedl/data"),

    // Rate limiting for DoS protection
    rate_limit_burst: 200,
    rate_limit_per_second: 100,

    ..Default::default()
};

let server = McpServer::new(config);
```

**Security features**:
1. **Path traversal protection**: Canonical path validation prevents `../` attacks
2. **Root path scoping**: Cannot access files outside configured directory
3. **Rate limiting**: Token bucket prevents request flooding
4. **No network authentication**: Designed for local stdio communication only

### Deployment Recommendations

1. **Use appropriate file permissions** on HEDL data files
2. **Run MCP server with minimal privileges** (not as root)
3. **Limit root_path** to only necessary directories
4. **Monitor rate limit violations** for potential attacks
5. **Use firewall rules** if exposing over network (not recommended)

## Example: Production MCP Server Setup

```rust
use hedl_mcp::{McpServer, McpServerConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Production configuration
    let config = McpServerConfig {
        // Restrict to data directory
        root_path: PathBuf::from("/var/hedl/data"),

        // Server identification
        name: "hedl-mcp".to_string(),
        version: "0.1.0".to_string(),

        // Conservative rate limiting
        rate_limit_burst: 100,      // Lower burst for production
        rate_limit_per_second: 50,  // 50 req/sec sustained

        // Enable caching for performance
        cache_size: 1000,
    };

    // Create and run server
    let server = McpServer::new(config);
    // Server runs over stdio (no network exposure)

    Ok(())
}
```

## No Authentication Required

For the core HEDL libraries (Rust, FFI, WASM), authentication is not applicable as they are:

- **Local libraries** running in the same process
- **No network communication** involved
- **No external services** accessed

Security considerations for these APIs focus on:
- Input validation (preventing malicious HEDL documents)
- Memory safety (FFI boundary protection)
- Resource limits (preventing DoS via large documents)

See [Error Handling](errors.md) for details on input validation and security.
