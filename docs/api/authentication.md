# Authentication

## Overview

HEDL API authentication varies by interface:

- **Rust API**: No authentication required (library dependency)
- **FFI API**: No authentication required (linked library)
- **WASM API**: No authentication required (browser/Node.js module)
- **MCP Server**: Full authentication support (API Key, JWT, OAuth2)
- **LSP Server**: No authentication (local editor integration)

## MCP Server Authentication

The HEDL Model Context Protocol (MCP) server provides enterprise-grade authentication with multiple schemes:

- **API Key**: Simple key-based authentication
- **JWT**: JSON Web Token authentication with claims validation
- **OAuth2**: Third-party provider integration
- **Session Management**: Secure session handling
- **Authorization Policies**: Fine-grained access control

Additionally, the server provides:

- **Rate limiting**: Token bucket algorithm for DoS protection (global and per-client)
- **Caching**: LRU cache for immutable operations
- **Root path scoping**: File operations restricted to configured directory
- **Resource limits**: Request/response size limits, concurrency limits, timeouts

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
    version: "1.2.0".to_string(),

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

The MCP server caches results of immutable operations (validate, query, stats):

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

**Cached Operations**:
- `hedl_validate`: Validation results (key: content hash + strict + lint)
- `hedl_query`: Query results (key: content hash + type_name + id + include_children)
- `hedl_stats`: Token statistics (key: content hash + tokenizer)

### API Key Authentication

Simple key-based authentication for straightforward deployments:

```rust
use hedl_mcp::auth::{ApiKeyAuth, ApiKeyInfo, ApiKeyStore};
use std::sync::Arc;

pub struct ApiKeyAuth {
    key_store: Arc<dyn ApiKeyStore>,
    key_prefix: Option<String>,
}

impl ApiKeyAuth {
    pub fn new(key_store: Arc<dyn ApiKeyStore>, key_prefix: Option<String>) -> Self;
    pub async fn authenticate(&self, key: &str) -> Result<ClientMetadata, AuthError>;
}
```

### JWT Authentication

JSON Web Token authentication with claims validation:

```rust
use hedl_mcp::auth::{JwtAuth, JwtAuthConfig, TokenValidationCache};
use jsonwebtoken::{EncodingKey, DecodingKey};

pub struct JwtAuth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    config: JwtAuthConfig,
    cache: Option<TokenValidationCache>,
}

impl JwtAuth {
    pub fn new_with_secret(secret: &str, config: Option<JwtAuthConfig>) -> Self;
    pub fn authenticate(&self, token: &str) -> Result<ClientMetadata, AuthError>;
    pub fn create_token(&self, claims: &JwtClaims) -> Result<String, AuthError>;
}
```

### Session Management

Secure session handling with configurable timeouts:

```rust
use hedl_mcp::auth::{SessionManager, SessionConfig, SessionId, Session};
use dashmap::DashMap;
use std::sync::Arc;

pub struct SessionManager {
    sessions: Arc<DashMap<SessionId, Session>>,
    config: SessionConfig,
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Self;
    pub fn create_session(&self, client_metadata: ClientMetadata) -> Session;
    pub fn validate_session(&self, session_id: &SessionId) -> Result<Session, AuthError>;
    pub fn end_session(&self, session_id: &SessionId) -> Option<Session>;
    pub fn cleanup_expired(&self) -> usize;
}
```

### Authorization Policies

Fine-grained access control with policy rules:

```rust
use hedl_mcp::auth::{AuthorizationPolicy, PolicyRule, DefaultPolicy};

pub struct AuthorizationPolicy {
    pub rules: Vec<PolicyRule>,
    pub default_policy: DefaultPolicy,
}

pub enum DefaultPolicy {
    Deny,   // Deny by default (recommended)
    Allow,  // Allow by default
}

impl AuthorizationPolicy {
    pub fn new() -> Self;
    pub fn add_rule(&mut self, rule: PolicyRule);
    pub fn check(
        &self,
        client: &ClientMetadata,
        resource: &AuthResource,
        action: &Action,
    ) -> AuthResult<()>;
}
```

### Credential Security

The auth module provides secure credential handling:

- `CredentialStore`: Encrypted credential storage
- `ApiKeyHasher`: Secure API key hashing
- `secure_write`: Safe credential file writes

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
4. **Authentication**: API Key, JWT, and OAuth2 support for secure access

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
        version: "1.2.0".to_string(),

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
