// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! MCP Server implementation.

use crate::cache::OperationCache;
use crate::error::McpResult;
use crate::protocol::{
    CallToolParams, CallToolResult, Content, InitializeParams, InitializeResult, JsonRpcRequest,
    JsonRpcResponse, ListResourcesResult, ListToolsResult, ReadResourceParams, ReadResourceResult,
    Resource, ResourceContent, ResourcesCapability, ServerCapabilities, ServerInfo,
    ToolsCapability,
};
use crate::rate_limiter::RateLimiter;
use crate::resource_limits::ResourceLimitManager;
use crate::tools::helpers::resolve_safe_path;
use crate::tools::{execute_tool, get_tools};
use crate::{SERVER_NAME, VERSION};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{debug, error, info, warn};

/// MCP Server configuration.
///
/// Configuration for the HEDL Model Context Protocol server, including
/// the root directory for file operations and server metadata.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Root path for file operations.
    ///
    /// All file-based tool operations (read, write, list) are scoped to this
    /// directory for security. Path traversal attempts are rejected.
    pub root_path: PathBuf,

    /// Server name reported in protocol handshake.
    pub name: String,

    /// Server version reported in protocol handshake.
    pub version: String,

    /// Maximum token bucket capacity (burst size).
    ///
    /// Allows short bursts of requests up to this limit.
    /// Default: 200 requests.
    /// Set to 0 to disable rate limiting.
    pub rate_limit_burst: usize,

    /// Token refill rate (requests per second).
    ///
    /// Determines sustained request rate limit.
    /// Default: 100 requests/second.
    pub rate_limit_per_second: usize,

    /// Cache size for immutable operations (validate, lint, `analyze_schema`).
    ///
    /// Caches results of expensive operations to improve performance on
    /// repeated requests. Set to 0 to disable caching.
    /// Default: 1000 entries.
    pub cache_size: usize,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            root_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            name: SERVER_NAME.to_string(),
            version: VERSION.to_string(),
            rate_limit_burst: 200,      // Allow bursts up to 200 requests
            rate_limit_per_second: 100, // 100 requests/sec sustained
            cache_size: 1000,           // 1000 cached entries
        }
    }
}

/// HEDL MCP Server.
///
/// Implements the Model Context Protocol (MCP) for AI/LLM integration with HEDL.
/// Provides JSON-RPC 2.0 communication over stdio transport with support for:
///
/// - Tool execution (11 HEDL manipulation tools)
/// - Resource management (HEDL file discovery and reading)
/// - Protocol lifecycle (initialize, shutdown)
/// - Rate limiting (token bucket algorithm)
///
/// # Thread Safety
///
/// The cache uses `DashMap` for lock-free concurrent access. Rate limiter uses
/// atomic operations. Request handling remains sequential via stdio transport.
///
/// # Security
///
/// All file operations are scoped to `config.root_path` with path traversal
/// protection via canonical path validation. Rate limiting protects against
/// `DoS` attacks via request flooding.
pub struct McpServer {
    /// Server configuration including root path and metadata.
    config: McpServerConfig,

    /// Initialization state tracking.
    ///
    /// Set to `true` after successful `initialize` handshake, reset to `false`
    /// on `shutdown`. Used to enforce proper protocol lifecycle.
    initialized: bool,

    /// Rate limiter for `DoS` protection.
    ///
    /// Uses token bucket algorithm to limit request rate. None if rate limiting
    /// is disabled (burst = 0).
    rate_limiter: Option<RateLimiter>,

    /// Operation cache for immutable operations (validate, lint, `analyze_schema`).
    ///
    /// Thread-safe LRU cache using `DashMap`. Provides 2-5x speedup on repeated
    /// requests with the same content. None if caching is disabled (size = 0).
    cache: Option<Arc<OperationCache>>,

    /// Comprehensive resource limit manager.
    ///
    /// Manages all resource limits including request/response size, per-client
    /// rate limiting, memory usage, concurrency, and timeouts. None if resource
    /// limits are disabled.
    resource_limits: Option<Arc<ResourceLimitManager>>,
}

impl McpServer {
    /// Create a new MCP server with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration including root path and rate limits
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_mcp::{McpServer, McpServerConfig};
    /// use std::path::PathBuf;
    ///
    /// let config = McpServerConfig {
    ///     root_path: PathBuf::from("/data/hedl"),
    ///     rate_limit_burst: 200,
    ///     rate_limit_per_second: 100,
    ///     cache_size: 1000,
    ///     ..Default::default()
    /// };
    /// let server = McpServer::new(config);
    /// ```
    #[must_use]
    pub fn new(config: McpServerConfig) -> Self {
        // Create rate limiter if enabled (burst > 0)
        let rate_limiter = if config.rate_limit_burst > 0 && config.rate_limit_per_second > 0 {
            Some(RateLimiter::new(
                config.rate_limit_burst,
                config.rate_limit_per_second,
            ))
        } else {
            None
        };

        // Create cache if enabled (size > 0)
        let cache = if config.cache_size > 0 {
            Some(Arc::new(OperationCache::new(config.cache_size)))
        } else {
            None
        };

        Self {
            config,
            initialized: false,
            rate_limiter,
            cache,
            resource_limits: None,
        }
    }

    /// Set the resource limit manager for this server.
    ///
    /// # Arguments
    ///
    /// * `resource_limits` - Resource limit manager to use
    pub fn with_resource_limits(mut self, resource_limits: ResourceLimitManager) -> Self {
        self.resource_limits = Some(Arc::new(resource_limits));
        self
    }

    /// Get a reference to the resource limit manager (if enabled).
    ///
    /// # Returns
    ///
    /// Reference to the resource limit manager if enabled, `None` otherwise.
    pub fn resource_limits(&self) -> Option<&Arc<ResourceLimitManager>> {
        self.resource_limits.as_ref()
    }

    /// Check if rate limit is exceeded and consume a token.
    ///
    /// Returns `Ok(())` if the request is allowed, `Err(())` if rate limit exceeded.
    ///
    /// # Implementation
    ///
    /// Uses token bucket algorithm:
    /// 1. Refill tokens based on elapsed time
    /// 2. Check if tokens are available
    /// 3. Consume one token if available
    ///
    /// # Security
    ///
    /// Protects against `DoS` attacks via request flooding.
    ///
    /// # Thread Safety
    ///
    /// The rate limiter uses interior mutability, so this method can take
    /// `&self` instead of `&mut self`.
    fn check_rate_limit(&self) -> Result<(), ()> {
        match &self.rate_limiter {
            Some(limiter) => {
                if limiter.check_limit() {
                    Ok(())
                } else {
                    Err(())
                }
            }
            None => Ok(()), // Rate limiting disabled
        }
    }

    /// Create a new MCP server with default config and specified root path.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root directory for file operations
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_mcp::McpServer;
    /// use std::path::PathBuf;
    ///
    /// let server = McpServer::with_root(PathBuf::from("/data"));
    /// ```
    #[must_use]
    pub fn with_root(root_path: PathBuf) -> Self {
        Self::new(McpServerConfig {
            root_path,
            ..Default::default()
        })
    }

    /// Get a reference to the operation cache (if enabled).
    ///
    /// # Returns
    ///
    /// Reference to the cache if enabled, `None` otherwise.
    pub fn cache(&self) -> Option<&Arc<OperationCache>> {
        self.cache.as_ref()
    }

    /// Get cache statistics.
    ///
    /// # Returns
    ///
    /// Cache statistics if caching is enabled, `None` otherwise.
    pub fn cache_stats(&self) -> Option<crate::cache::CacheStats> {
        self.cache.as_ref().map(|c| c.stats())
    }

    /// Run the server using stdio transport (synchronous).
    ///
    /// Reads JSON-RPC 2.0 requests from stdin line-by-line and writes responses
    /// to stdout. This is a blocking operation that runs until stdin is closed.
    ///
    /// # Protocol
    ///
    /// - Input: One JSON-RPC request per line on stdin
    /// - Output: One JSON-RPC response per line on stdout
    /// - Transport: Synchronous, line-buffered I/O
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - I/O operations fail (reading stdin or writing stdout)
    /// - Response serialization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hedl_mcp::{McpServer, McpServerConfig};
    ///
    /// let mut server = McpServer::new(McpServerConfig::default());
    /// server.run_stdio().expect("Server failed");
    /// ```
    pub fn run_stdio(&mut self) -> McpResult<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = BufReader::new(stdin.lock());

        info!("HEDL MCP Server starting on stdio");

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            debug!("Received: {}", line);

            // Check rate limit before processing request
            if let Err(()) = self.check_rate_limit() {
                let error_response = JsonRpcResponse::error(
                    None,
                    -32005, // Custom error code for rate limiting
                    "Rate limit exceeded. Too many requests.".to_string(),
                    Some(json!({
                        "burst_capacity": self.config.rate_limit_burst,
                        "rate_per_second": self.config.rate_limit_per_second
                    })),
                );
                let response_str = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{response_str}")?;
                stdout.flush()?;
                continue;
            }

            match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => {
                    // Validate JSON-RPC protocol compliance
                    if let Err(validation_error) = request.validate() {
                        let response = JsonRpcResponse::error(
                            request.id.clone(),
                            -32600, // Invalid Request error code
                            format!("Invalid request: {validation_error}"),
                            None,
                        );
                        let response_str = serde_json::to_string(&response)?;
                        debug!("Sending validation error: {}", response_str);
                        writeln!(stdout, "{response_str}")?;
                        stdout.flush()?;
                        continue;
                    }

                    let response = self.handle_request(request);
                    let response_str = serde_json::to_string(&response)?;
                    debug!("Sending: {}", response_str);
                    writeln!(stdout, "{response_str}")?;
                    stdout.flush()?;
                }
                Err(e) => {
                    let response =
                        JsonRpcResponse::error(None, -32700, format!("Parse error: {e}"), None);
                    let response_str = serde_json::to_string(&response)?;
                    writeln!(stdout, "{response_str}")?;
                    stdout.flush()?;
                }
            }
        }

        Ok(())
    }

    /// Run the server using stdio transport (asynchronous).
    ///
    /// Asynchronous version of `run_stdio()` using Tokio runtime. Reads JSON-RPC
    /// requests from stdin and writes responses to stdout using non-blocking I/O.
    ///
    /// # Protocol
    ///
    /// - Input: One JSON-RPC request per line on stdin
    /// - Output: One JSON-RPC response per line on stdout
    /// - Transport: Asynchronous, line-buffered I/O
    ///
    /// # Performance
    ///
    /// Recommended for production use. Non-blocking I/O provides better CPU
    /// utilization and allows for potential future concurrent request handling.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - I/O operations fail (reading stdin or writing stdout)
    /// - Response serialization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hedl_mcp::{McpServer, McpServerConfig};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut server = McpServer::new(McpServerConfig::default());
    ///     server.run_stdio_async().await.expect("Server failed");
    /// }
    /// ```
    pub async fn run_stdio_async(&mut self) -> McpResult<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = AsyncBufReader::new(stdin);

        info!("HEDL MCP Server starting on stdio (async)");

        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    debug!("Received: {}", line);

                    // Check request size limits before parsing
                    if let Some(ref limits) = self.resource_limits {
                        if let Err(e) = limits.request_limits.check_raw_size(line.as_bytes()) {
                            let error_response = JsonRpcResponse::error(
                                None,
                                e.error_code(),
                                format!("Request size limit exceeded: {e}"),
                                Some(json!({
                                    "error": e.to_string(),
                                    "limit": limits.request_limits.max_total_size()
                                })),
                            );
                            let response_str = serde_json::to_string(&error_response)?;
                            stdout.write_all(response_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                            continue;
                        }
                    }

                    // Check rate limit before processing request
                    if let Err(()) = self.check_rate_limit() {
                        let error_response = JsonRpcResponse::error(
                            None,
                            -32005, // Custom error code for rate limiting
                            "Rate limit exceeded. Too many requests.".to_string(),
                            Some(json!({
                                "burst_capacity": self.config.rate_limit_burst,
                                "rate_per_second": self.config.rate_limit_per_second
                            })),
                        );
                        let response_str = serde_json::to_string(&error_response)?;
                        stdout.write_all(response_str.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                        continue;
                    }

                    match serde_json::from_str::<JsonRpcRequest>(line) {
                        Ok(request) => {
                            // Validate JSON-RPC protocol compliance
                            if let Err(validation_error) = request.validate() {
                                let response = JsonRpcResponse::error(
                                    request.id.clone(),
                                    -32600, // Invalid Request error code
                                    format!("Invalid request: {validation_error}"),
                                    None,
                                );
                                let response_str = serde_json::to_string(&response)?;
                                debug!("Sending validation error: {}", response_str);
                                stdout.write_all(response_str.as_bytes()).await?;
                                stdout.write_all(b"\n").await?;
                                stdout.flush().await?;
                                continue;
                            }

                            let response = self.handle_request(request);
                            let response_str = serde_json::to_string(&response)?;
                            debug!("Sending: {}", response_str);
                            stdout.write_all(response_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                        Err(e) => {
                            let response = JsonRpcResponse::error(
                                None,
                                -32700,
                                format!("Parse error: {e}"),
                                None,
                            );
                            let response_str = serde_json::to_string(&response)?;
                            stdout.write_all(response_str.as_bytes()).await?;
                            stdout.write_all(b"\n").await?;
                            stdout.flush().await?;
                        }
                    }
                }
                Err(e) => {
                    error!("Read error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a single JSON-RPC request.
    ///
    /// Routes the request to the appropriate handler based on the method name.
    /// Implements the MCP protocol methods and standard JSON-RPC patterns.
    ///
    /// # Supported Methods
    ///
    /// - `initialize` - Protocol handshake with capability negotiation
    /// - `initialized` - Notification after handshake completion
    /// - `shutdown` - Graceful server shutdown
    /// - `tools/list` - List available HEDL tools
    /// - `tools/call` - Execute a specific tool
    /// - `resources/list` - List available HEDL files
    /// - `resources/read` - Read HEDL file content
    /// - `ping` - Health check endpoint
    ///
    /// # Arguments
    ///
    /// * `request` - JSON-RPC 2.0 request with method and parameters
    ///
    /// # Returns
    ///
    /// A JSON-RPC response with either a result or error. Unknown methods
    /// return a "Method not found" error (-32601).
    pub fn handle_request_string(&mut self, request_str: &str) -> String {
        if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(request_str) {
            let response = self.handle_request(request);
            serde_json::to_string(&response).unwrap_or_else(|_| {
                serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32603, "message": "Internal error"}, "id": null}).to_string()
            })
        } else {
            // Parse error
            let error_response =
                JsonRpcResponse::error(None, -32700, "Parse error".to_string(), None);
            serde_json::to_string(&error_response).unwrap_or_else(|_| {
                r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null}"#
                    .to_string()
            })
        }
    }

    /// Handles an incoming JSON-RPC request and returns a response.
    pub fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();

        // Check parsed request size limits
        if let Some(ref limits) = self.resource_limits {
            if let Err(e) = limits.request_limits.check_parsed_request(&request) {
                return JsonRpcResponse::error(
                    id,
                    e.error_code(),
                    format!("Request validation failed: {e}"),
                    Some(json!({
                        "error": e.to_string()
                    })),
                );
            }
        }

        match request.method.as_str() {
            "initialize" => self.handle_initialize(id, request.params),
            "initialized" => self.handle_initialized(id),
            "shutdown" => self.handle_shutdown(id),
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(id, request.params),
            "resources/list" => self.handle_resources_list(id),
            "resources/read" => self.handle_resources_read(id, request.params),
            "ping" => JsonRpcResponse::success(id, json!({})),
            method => {
                warn!("Unknown method: {}", method);
                JsonRpcResponse::error(id, -32601, format!("Method not found: {method}"), None)
            }
        }
    }

    /// Handle the `initialize` method for MCP handshake.
    ///
    /// Performs protocol version negotiation and advertises server capabilities.
    /// This must be the first method called in the MCP protocol lifecycle.
    ///
    /// # Arguments
    ///
    /// * `id` - JSON-RPC request ID
    /// * `params` - Initialize parameters including protocol version and client info
    ///
    /// # Returns
    ///
    /// Success response with server capabilities, protocol version, and server info.
    /// Error response if parameters are invalid or missing.
    fn handle_initialize(&mut self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let _params: InitializeParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        format!("Invalid params: {e}"),
                        None,
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(id, -32602, "Missing params".to_string(), None);
            }
        };

        self.initialized = true;
        info!("Server initialized");

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                prompts: None,
            },
            server_info: ServerInfo {
                name: self.config.name.clone(),
                version: self.config.version.clone(),
            },
        };

        JsonRpcResponse::success(
            id,
            serde_json::to_value(result).expect("InitializeResult serialization cannot fail"),
        )
    }

    /// Handle the `initialized` notification.
    ///
    /// Sent by the client after receiving the `initialize` response to confirm
    /// the handshake is complete. This is a notification with no response expected.
    ///
    /// # Arguments
    ///
    /// * `id` - JSON-RPC request ID (may be None for notifications)
    ///
    /// # Returns
    ///
    /// Empty success response acknowledging the notification.
    fn handle_initialized(&self, id: Option<Value>) -> JsonRpcResponse {
        info!("Client sent initialized notification");
        JsonRpcResponse::success(id, json!({}))
    }

    /// Handle the `shutdown` method for graceful termination.
    ///
    /// Resets the server state and prepares for shutdown. After this method,
    /// the client should close the connection.
    ///
    /// # Arguments
    ///
    /// * `id` - JSON-RPC request ID
    ///
    /// # Returns
    ///
    /// Empty success response confirming shutdown readiness.
    fn handle_shutdown(&mut self, id: Option<Value>) -> JsonRpcResponse {
        info!("Server shutting down");
        self.initialized = false;
        JsonRpcResponse::success(id, json!({}))
    }

    /// Handle the `tools/list` method.
    ///
    /// Returns the catalog of all available HEDL tools with their schemas.
    /// Currently provides 10 tools for HEDL manipulation and conversion.
    ///
    /// # Arguments
    ///
    /// * `id` - JSON-RPC request ID
    ///
    /// # Returns
    ///
    /// List of tools with names, descriptions, and JSON Schema input definitions.
    fn handle_tools_list(&self, id: Option<Value>) -> JsonRpcResponse {
        let tools = get_tools();
        let result = ListToolsResult { tools };
        JsonRpcResponse::success(
            id,
            serde_json::to_value(result).expect("ListToolsResult serialization cannot fail"),
        )
    }

    /// Handle the `tools/call` method for tool execution with caching.
    ///
    /// Executes a specific tool by name with provided arguments. For immutable
    /// operations (validate, lint, query, stats), checks the cache first and
    /// stores results after execution. Tool errors are returned as successful
    /// responses with `is_error: true` to distinguish from protocol-level errors.
    ///
    /// # Arguments
    ///
    /// * `id` - JSON-RPC request ID
    /// * `params` - Tool call parameters including tool name and arguments
    ///
    /// # Returns
    ///
    /// Success response containing tool result (text or resource content).
    /// Tool execution errors are returned as success with `is_error: true`.
    /// Protocol errors (invalid params) return JSON-RPC error responses.
    ///
    /// # Caching
    ///
    /// The following operations are cached based on input content:
    /// - `hedl_validate` - Validation results (including lint diagnostics)
    /// - `hedl_query` - Entity query results
    /// - `hedl_stats` - Token statistics
    ///
    /// Cache keys combine operation name + input hash for deterministic results.
    fn handle_tools_call(&self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let params: CallToolParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        format!("Invalid params: {e}"),
                        None,
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(id, -32602, "Missing params".to_string(), None);
            }
        };

        // Check cache for immutable operations
        if self.cache.is_some() {
            if let Some(cached_result) = self.try_get_cached(&params.name, &params.arguments) {
                debug!("Cache hit for tool: {}", params.name);
                return JsonRpcResponse::success(id, cached_result);
            }
        }

        // Execute tool (cache miss or non-cacheable operation)
        // Special handling for batch tool which needs cache access
        let result = if params.name == "batch" {
            crate::tools::execute_batch(
                params.arguments.clone(),
                &self.config.root_path,
                self.cache.clone(),
            )
        } else {
            execute_tool(
                &params.name,
                params.arguments.clone(),
                &self.config.root_path,
            )
        };

        match result {
            Ok(result) => {
                let result_value = serde_json::to_value(&result)
                    .expect("CallToolResult serialization cannot fail");

                // Cache result for immutable operations (not batch)
                if self.cache.is_some() && params.name != "batch" {
                    self.try_cache_result(&params.name, &params.arguments, &result_value);
                }

                JsonRpcResponse::success(id, result_value)
            }
            Err(e) => {
                let result = CallToolResult {
                    content: vec![Content::Text {
                        text: format!("Error: {e}"),
                    }],
                    is_error: Some(true),
                };
                JsonRpcResponse::success(
                    id,
                    serde_json::to_value(result).expect("CallToolResult serialization cannot fail"),
                )
            }
        }
    }

    /// Try to get a cached result for an immutable operation.
    ///
    /// Returns `Some(cached_result)` if the operation is cacheable and a cached
    /// result exists, `None` otherwise.
    fn try_get_cached(&self, tool_name: &str, arguments: &Option<Value>) -> Option<Value> {
        let cache = self.cache.as_ref()?;
        let args = arguments.as_ref()?;
        let cache_key = Self::build_cache_key(tool_name, args)?;
        cache.get(tool_name, &cache_key)
    }

    /// Build a cache key for cacheable operations.
    /// Uses content hash instead of full content to avoid memory bloat.
    fn build_cache_key(tool_name: &str, args: &Value) -> Option<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Hash content to avoid storing full HEDL in cache key
        let hash_content = |s: &str| -> u64 {
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            hasher.finish()
        };

        match tool_name {
            "hedl_validate" => {
                let hedl = args.get("hedl")?.as_str()?;
                let hedl_hash = hash_content(hedl);
                let strict = args
                    .get("strict")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                let lint = args
                    .get("lint")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                Some(format!("{hedl_hash:016x}:{strict}:{lint}"))
            }
            "hedl_query" => {
                let hedl = args.get("hedl")?.as_str()?;
                let hedl_hash = hash_content(hedl);
                let type_name = args.get("type_name").and_then(|v| v.as_str()).unwrap_or("");
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let include_children = args
                    .get("include_children")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                Some(format!(
                    "{hedl_hash:016x}:{type_name}:{id}:{include_children}"
                ))
            }
            "hedl_stats" => {
                let hedl = args.get("hedl")?.as_str()?;
                let hedl_hash = hash_content(hedl);
                let tokenizer = args
                    .get("tokenizer")
                    .and_then(|v| v.as_str())
                    .unwrap_or("simple");
                Some(format!("{hedl_hash:016x}:{tokenizer}"))
            }
            _ => None, // Non-cacheable operation
        }
    }

    /// Try to cache the result of an immutable operation.
    fn try_cache_result(&self, tool_name: &str, arguments: &Option<Value>, result: &Value) {
        let cache = match &self.cache {
            Some(c) => c,
            None => return,
        };

        let args = match arguments {
            Some(a) => a,
            None => return,
        };

        // Use shared cache key builder (DRY)
        let cache_key = match Self::build_cache_key(tool_name, args) {
            Some(k) => k,
            None => return,
        };

        cache.insert(tool_name, &cache_key, result.clone());
    }

    /// Handle the `resources/list` method.
    ///
    /// Lists all HEDL files in the configured root directory as resources.
    /// Resources are exposed with `file://` URIs and MIME type `text/hedl`.
    ///
    /// # Security
    ///
    /// All files listed are guaranteed to be within the `root_path` directory.
    /// The listing only scans the configured root directory (non-recursive).
    ///
    /// # Arguments
    ///
    /// * `id` - JSON-RPC request ID
    ///
    /// # Returns
    ///
    /// List of resources with URIs, names, descriptions, and MIME types.
    /// Returns empty list if directory cannot be read or contains no HEDL files.
    fn handle_resources_list(&self, id: Option<Value>) -> JsonRpcResponse {
        // List .hedl files in root as resources
        let mut resources = Vec::new();

        // Canonicalize root to ensure we're working with the real path
        let canonical_root = match self.config.root_path.canonicalize() {
            Ok(path) => path,
            Err(e) => {
                error!("Failed to canonicalize root_path: {}", e);
                return JsonRpcResponse::error(
                    id,
                    -32002,
                    format!("Failed to access root directory: {e}"),
                    None,
                );
            }
        };

        if let Ok(entries) = std::fs::read_dir(&canonical_root) {
            for entry in entries.filter_map(std::result::Result::ok) {
                let path = entry.path();

                // Security: Double-check the path is still within root
                // This handles any race conditions with symlink changes
                if let Ok(canonical_path) = path.canonicalize() {
                    if !canonical_path.starts_with(&canonical_root) {
                        debug!(
                            "Skipping file outside root path: {}",
                            canonical_path.display()
                        );
                        continue;
                    }
                }

                if path.is_file() && path.extension().is_some_and(|ext| ext == "hedl") {
                    if let Some(name) = path.file_name() {
                        resources.push(Resource {
                            uri: format!("file://{}", path.display()),
                            name: name.to_string_lossy().to_string(),
                            description: Some("HEDL document".to_string()),
                            mime_type: Some("text/hedl".to_string()),
                        });
                    }
                }
            }
        }

        let result = ListResourcesResult { resources };
        JsonRpcResponse::success(
            id,
            serde_json::to_value(result).expect("ListResourcesResult serialization cannot fail"),
        )
    }

    /// Handle the `resources/read` method.
    ///
    /// Reads the content of a HEDL resource identified by URI. Supports `file://`
    /// URIs and plain file paths. All file access is sandboxed to `root_path`
    /// with path traversal protection via canonical path validation.
    ///
    /// # Security
    ///
    /// - Path traversal attempts are blocked and logged
    /// - Symbolic links pointing outside root are rejected
    /// - Only files within the configured `root_path` can be read
    ///
    /// # Arguments
    ///
    /// * `id` - JSON-RPC request ID
    /// * `params` - Resource read parameters including URI
    ///
    /// # Returns
    ///
    /// Success response with resource content (text).
    /// Error response if resource cannot be read or parameters are invalid.
    fn handle_resources_read(&self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let params: ReadResourceParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return JsonRpcResponse::error(
                        id,
                        -32602,
                        format!("Invalid params: {e}"),
                        None,
                    );
                }
            },
            None => {
                return JsonRpcResponse::error(id, -32602, "Missing params".to_string(), None);
            }
        };

        // Parse file:// URI to extract the path
        let path_str = if params.uri.starts_with("file://") {
            &params.uri[7..]
        } else {
            &params.uri
        };

        // SECURITY: Validate path is within root directory
        // This prevents path traversal attacks like ../../etc/passwd
        let safe_path = match resolve_safe_path(&self.config.root_path, path_str) {
            Ok(path) => path,
            Err(e) => {
                // Log security event for path traversal attempts
                warn!(
                    "Path traversal attempt blocked in resources/read: uri={}, error={}",
                    params.uri, e
                );
                return JsonRpcResponse::error(
                    id,
                    -32003, // PathTraversal error code
                    format!("Path traversal not allowed: {e}"),
                    None,
                );
            }
        };

        match std::fs::read_to_string(&safe_path) {
            Ok(content) => {
                let result = ReadResourceResult {
                    contents: vec![ResourceContent {
                        uri: params.uri,
                        mime_type: Some("text/hedl".to_string()),
                        text: Some(content),
                    }],
                };
                JsonRpcResponse::success(
                    id,
                    serde_json::to_value(result)
                        .expect("ReadResourceResult serialization cannot fail"),
                )
            }
            Err(e) => {
                JsonRpcResponse::error(id, -32002, format!("Failed to read resource: {e}"), None)
            }
        }
    }
}
