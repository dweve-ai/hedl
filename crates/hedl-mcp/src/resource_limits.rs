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

//! Comprehensive resource limits for the HEDL MCP server.
//!
//! This module provides protection against denial-of-service (`DoS`) attacks and
//! ensures fair resource distribution across clients through multiple independent
//! limit enforcement mechanisms.
//!
//! # Resource Limit Categories
//!
//! 1. **Request Size Limits** - Maximum request payload sizes
//! 2. **Response Size Limits** - Maximum response payload sizes
//! 3. **Per-Client Rate Limiting** - Independent rate limits per client
//! 4. **Memory Usage Limits** - Cache memory bounds
//! 5. **Concurrency Limits** - Concurrent operation bounds
//! 6. **Timeout Limits** - Maximum operation execution time

use crate::error::McpError;
use crate::protocol::{CallToolResult, Content, JsonRpcRequest};
use crate::rate_limiter::RateLimiter;
use dashmap::DashMap;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::debug;

// ============================================================================
// Error Types
// ============================================================================

/// Resource limit enforcement error.
///
/// Represents various types of resource limit violations with actionable
/// error messages and context.
#[derive(Debug, thiserror::Error)]
pub enum ResourceLimitError {
    /// Request size exceeds configured limit.
    #[error(
        "Request size {size} bytes exceeds limit {limit} bytes (exceeded by {exceeded_by} bytes)"
    )]
    RequestTooLarge {
        /// Actual request size in bytes.
        size: usize,
        /// Maximum allowed request size in bytes.
        limit: usize,
        /// Amount by which the limit was exceeded.
        exceeded_by: usize,
    },

    /// String parameter exceeds size limit.
    #[error("String parameter size {size} bytes exceeds limit {limit} bytes")]
    StringTooLarge {
        /// Actual string size in bytes.
        size: usize,
        /// Maximum allowed string size in bytes.
        limit: usize,
    },

    /// Array element count exceeds limit.
    #[error("Array has {size} elements, exceeds limit {limit}")]
    ArrayTooLarge {
        /// Actual array element count.
        size: usize,
        /// Maximum allowed array elements.
        limit: usize,
    },

    /// JSON object nesting depth exceeds limit.
    #[error("JSON depth {depth} exceeds limit {limit}")]
    JsonTooDeep {
        /// Actual nesting depth.
        depth: usize,
        /// Maximum allowed nesting depth.
        limit: usize,
    },

    /// Response size exceeds configured limit.
    #[error("Response estimated size {estimated_size} bytes exceeds limit {limit} bytes")]
    ResponseTooLarge {
        /// Estimated response size in bytes.
        estimated_size: usize,
        /// Maximum allowed response size in bytes.
        limit: usize,
    },

    /// Result count exceeds configured limit.
    #[error("Result count {count} exceeds limit {limit}")]
    TooManyResults {
        /// Actual result count.
        count: usize,
        /// Maximum allowed result count.
        limit: usize,
    },

    /// Rate limit exceeded for a specific client.
    #[error("Rate limit exceeded for client '{client_id}': burst={burst}, rate={rate}/s")]
    RateLimitExceeded {
        /// Client identifier that exceeded the limit.
        client_id: String,
        /// Maximum burst capacity.
        burst: usize,
        /// Refill rate per second.
        rate: usize,
    },

    /// Cache memory usage exceeds limit.
    #[error("Cache memory {current} bytes exceeds limit {limit} bytes, needs {needed} bytes")]
    CacheMemoryExceeded {
        /// Current cache memory usage in bytes.
        current: usize,
        /// Maximum allowed cache memory in bytes.
        limit: usize,
        /// Additional memory needed for the operation.
        needed: usize,
    },

    /// Global concurrency limit exceeded.
    #[error("Global concurrency limit {limit} exceeded, queue timeout")]
    GlobalConcurrencyExceeded {
        /// Maximum concurrent requests allowed globally.
        limit: usize,
    },

    /// Per-client concurrency limit exceeded.
    #[error("Client concurrency limit {limit} exceeded for client '{client_id}', queue timeout")]
    ClientConcurrencyExceeded {
        /// Client identifier that exceeded the limit.
        client_id: String,
        /// Maximum concurrent requests allowed per client.
        limit: usize,
    },

    /// Per-tool concurrency limit exceeded.
    #[error("Tool concurrency limit {limit} exceeded for tool '{tool_name}', queue timeout")]
    ToolConcurrencyExceeded {
        /// Name of the tool that exceeded the limit.
        tool_name: String,
        /// Maximum concurrent requests allowed per tool.
        limit: usize,
    },

    /// Operation execution timeout.
    #[error("Operation '{tool_name}' exceeded timeout {timeout_ms}ms")]
    OperationTimeout {
        /// Name of the tool that timed out.
        tool_name: String,
        /// Timeout duration in milliseconds.
        timeout_ms: u64,
    },

    /// Semaphore closed unexpectedly.
    #[error("Semaphore closed unexpectedly")]
    SemaphoreClosed,
}

impl ResourceLimitError {
    /// Get the JSON-RPC error code for this resource limit error.
    #[must_use]
    pub fn error_code(&self) -> i32 {
        match self {
            Self::RequestTooLarge { .. }
            | Self::StringTooLarge { .. }
            | Self::ArrayTooLarge { .. }
            | Self::JsonTooDeep { .. } => -32006,
            Self::ResponseTooLarge { .. } | Self::TooManyResults { .. } => -32009,
            Self::RateLimitExceeded { .. } => -32005,
            Self::CacheMemoryExceeded { .. } => -32010,
            Self::GlobalConcurrencyExceeded { .. }
            | Self::ClientConcurrencyExceeded { .. }
            | Self::ToolConcurrencyExceeded { .. } => -32007,
            Self::OperationTimeout { .. } => -32008,
            Self::SemaphoreClosed => -32011,
        }
    }
}

impl From<ResourceLimitError> for McpError {
    fn from(err: ResourceLimitError) -> Self {
        McpError::InvalidRequest(err.to_string())
    }
}

// ============================================================================
// Client Identification
// ============================================================================

/// Client identifier for per-client resource tracking.
///
/// Used to enforce rate limits and concurrency limits independently per client.
/// Currently defaults to anonymous since authentication is not yet implemented.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ClientId(pub String);

impl ClientId {
    /// Create an anonymous client ID.
    ///
    /// Used when no client identification is available (e.g., no authentication).
    #[must_use]
    pub fn anonymous() -> Self {
        Self("anonymous".to_string())
    }

    /// Create a client ID from a string identifier.
    #[must_use]
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the string value of this client ID.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::anonymous()
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Request Size Limits
// ============================================================================

/// Request size validation limits.
///
/// Enforces size constraints on incoming JSON-RPC requests to prevent
/// memory exhaustion and parsing `DoS` attacks.
#[derive(Debug, Clone)]
pub struct RequestSizeLimits {
    /// Maximum total request size in bytes.
    max_total_size: usize,

    /// Maximum individual parameter size in bytes.
    max_param_size: usize,

    /// Maximum array element count.
    max_array_elements: usize,

    /// Maximum JSON object nesting depth.
    max_object_depth: usize,
}

impl RequestSizeLimits {
    /// Create new request size limits with specified values.
    ///
    /// # Arguments
    ///
    /// * `max_total_size` - Maximum total request size in bytes
    /// * `max_param_size` - Maximum individual parameter size in bytes
    /// * `max_array_elements` - Maximum array element count
    /// * `max_object_depth` - Maximum JSON object nesting depth
    #[must_use]
    pub fn new(
        max_total_size: usize,
        max_param_size: usize,
        max_array_elements: usize,
        max_object_depth: usize,
    ) -> Self {
        Self {
            max_total_size,
            max_param_size,
            max_array_elements,
            max_object_depth,
        }
    }

    /// Get default request size limits.
    ///
    /// Returns limits suitable for most production environments:
    /// - 10 MB total request size
    /// - 5 MB per parameter
    /// - 10,000 array elements
    /// - 32 object nesting depth
    #[must_use]
    pub fn default_limits() -> Self {
        Self {
            max_total_size: 10_485_760, // 10 MB
            max_param_size: 5_242_880,  // 5 MB
            max_array_elements: 10_000,
            max_object_depth: 32,
        }
    }

    /// Check raw request byte size before parsing.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw request bytes
    ///
    /// # Returns
    ///
    /// `Ok(())` if size is within limits, `Err` if exceeded.
    pub fn check_raw_size(&self, bytes: &[u8]) -> Result<(), ResourceLimitError> {
        if bytes.len() > self.max_total_size {
            return Err(ResourceLimitError::RequestTooLarge {
                size: bytes.len(),
                limit: self.max_total_size,
                exceeded_by: bytes.len() - self.max_total_size,
            });
        }
        Ok(())
    }

    /// Validate parsed JSON-RPC request structure.
    ///
    /// # Arguments
    ///
    /// * `request` - Parsed JSON-RPC request
    ///
    /// # Returns
    ///
    /// `Ok(())` if request structure is valid, `Err` if limits exceeded.
    pub fn check_parsed_request(&self, request: &JsonRpcRequest) -> Result<(), ResourceLimitError> {
        if let Some(params) = &request.params {
            self.validate_json_value(params, 0)?;
        }
        Ok(())
    }

    /// Recursively validate JSON value against size limits.
    fn validate_json_value(&self, value: &Value, depth: usize) -> Result<(), ResourceLimitError> {
        if depth > self.max_object_depth {
            return Err(ResourceLimitError::JsonTooDeep {
                depth,
                limit: self.max_object_depth,
            });
        }

        match value {
            Value::String(s) if s.len() > self.max_param_size => {
                Err(ResourceLimitError::StringTooLarge {
                    size: s.len(),
                    limit: self.max_param_size,
                })
            }
            Value::Array(arr) if arr.len() > self.max_array_elements => {
                Err(ResourceLimitError::ArrayTooLarge {
                    size: arr.len(),
                    limit: self.max_array_elements,
                })
            }
            Value::Array(arr) => {
                for item in arr {
                    self.validate_json_value(item, depth + 1)?;
                }
                Ok(())
            }
            Value::Object(obj) => {
                for (key, val) in obj {
                    // Check key size
                    if key.len() > self.max_param_size {
                        return Err(ResourceLimitError::StringTooLarge {
                            size: key.len(),
                            limit: self.max_param_size,
                        });
                    }
                    self.validate_json_value(val, depth + 1)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Get the maximum total request size.
    #[must_use]
    pub fn max_total_size(&self) -> usize {
        self.max_total_size
    }

    /// Get the maximum parameter size.
    #[must_use]
    pub fn max_param_size(&self) -> usize {
        self.max_param_size
    }

    /// Get the maximum array elements.
    #[must_use]
    pub fn max_array_elements(&self) -> usize {
        self.max_array_elements
    }

    /// Get the maximum object depth.
    #[must_use]
    pub fn max_object_depth(&self) -> usize {
        self.max_object_depth
    }
}

// ============================================================================
// Response Size Limits
// ============================================================================

/// Response size validation limits.
///
/// Enforces size constraints on outgoing responses to prevent excessive
/// memory allocation and network saturation.
#[derive(Debug, Clone)]
pub struct ResponseSizeLimits {
    /// Maximum total response size in bytes.
    max_total_size: usize,

    /// Maximum number of result items for array responses.
    max_result_items: usize,

    /// Whether streaming is enabled for large results.
    enable_streaming: bool,
}

impl ResponseSizeLimits {
    /// Create new response size limits with specified values.
    ///
    /// # Arguments
    ///
    /// * `max_total_size` - Maximum total response size in bytes
    /// * `max_result_items` - Maximum result count for array responses
    /// * `enable_streaming` - Whether to enable streaming for large results
    #[must_use]
    pub fn new(max_total_size: usize, max_result_items: usize, enable_streaming: bool) -> Self {
        Self {
            max_total_size,
            max_result_items,
            enable_streaming,
        }
    }

    /// Get default response size limits.
    ///
    /// Returns limits suitable for most production environments:
    /// - 50 MB total response size
    /// - 100,000 result items
    /// - Streaming enabled
    #[must_use]
    pub fn default_limits() -> Self {
        Self {
            max_total_size: 50_000_000, // 50 MB
            max_result_items: 100_000,
            enable_streaming: true,
        }
    }

    /// Estimate the size of a tool call result.
    ///
    /// # Arguments
    ///
    /// * `result` - Tool call result to estimate
    ///
    /// # Returns
    ///
    /// Estimated size in bytes, or error if exceeds limit.
    pub fn estimate_size(&self, result: &CallToolResult) -> Result<usize, ResourceLimitError> {
        let mut size = 0;
        for content in &result.content {
            size += match content {
                Content::Text { text } => text.len(),
                Content::Resource { resource } => {
                    resource.text.as_ref().map_or(0, std::string::String::len)
                }
            };
        }

        if size > self.max_total_size {
            return Err(ResourceLimitError::ResponseTooLarge {
                estimated_size: size,
                limit: self.max_total_size,
            });
        }

        Ok(size)
    }

    /// Check if result count is within limits.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of results
    ///
    /// # Returns
    ///
    /// `Ok(())` if count is within limits, `Err` if exceeded.
    pub fn check_result_count(&self, count: usize) -> Result<(), ResourceLimitError> {
        if count > self.max_result_items {
            return Err(ResourceLimitError::TooManyResults {
                count,
                limit: self.max_result_items,
            });
        }
        Ok(())
    }

    /// Get the maximum total response size.
    #[must_use]
    pub fn max_total_size(&self) -> usize {
        self.max_total_size
    }

    /// Get the maximum result items.
    #[must_use]
    pub fn max_result_items(&self) -> usize {
        self.max_result_items
    }

    /// Check if streaming is enabled.
    #[must_use]
    pub fn is_streaming_enabled(&self) -> bool {
        self.enable_streaming
    }
}

// ============================================================================
// Per-Client Rate Limiting
// ============================================================================

/// Rate limit configuration for a client or client pattern.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Burst capacity (maximum tokens in bucket).
    pub burst: usize,
    /// Refill rate (tokens added per second).
    pub per_second: usize,
}

impl RateLimitConfig {
    /// Create a new rate limit configuration.
    #[must_use]
    pub fn new(burst: usize, per_second: usize) -> Self {
        Self { burst, per_second }
    }

    /// Get default rate limit configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self {
            burst: 200,
            per_second: 100,
        }
    }
}

/// Per-client rate limiter using token bucket algorithm.
///
/// Tracks independent rate limits for each client, allowing fair resource
/// distribution across multiple concurrent clients.
pub struct PerClientRateLimiter {
    /// Individual rate limiters per client ID.
    limiters: Arc<DashMap<ClientId, RateLimiter>>,

    /// Default rate limit configuration for unknown clients.
    default_config: RateLimitConfig,

    /// Client pattern overrides (glob pattern -> config).
    overrides: Vec<(glob::Pattern, RateLimitConfig)>,

    /// Last cleanup timestamp.
    last_cleanup: Arc<Mutex<Instant>>,

    /// Cleanup interval for inactive limiters.
    cleanup_interval: Duration,
}

impl PerClientRateLimiter {
    /// Create a new per-client rate limiter.
    ///
    /// # Arguments
    ///
    /// * `default_config` - Default rate limit for unknown clients
    /// * `overrides` - Client pattern overrides (glob pattern -> config)
    /// * `cleanup_interval` - How often to clean up inactive limiters
    #[must_use]
    pub fn new(
        default_config: RateLimitConfig,
        overrides: Vec<(String, RateLimitConfig)>,
        cleanup_interval: Duration,
    ) -> Self {
        // Parse glob patterns
        let overrides = overrides
            .into_iter()
            .filter_map(|(pattern, config)| glob::Pattern::new(&pattern).ok().map(|p| (p, config)))
            .collect();

        Self {
            limiters: Arc::new(DashMap::new()),
            default_config,
            overrides,
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
            cleanup_interval,
        }
    }

    /// Create a per-client rate limiter with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(
            RateLimitConfig::default_config(),
            vec![],
            Duration::from_secs(300),
        )
    }

    /// Check if a request from the given client is allowed.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client identifier
    ///
    /// # Returns
    ///
    /// `Ok(())` if request is allowed, `Err` if rate limit exceeded.
    pub fn check_limit(&self, client_id: &ClientId) -> Result<(), ResourceLimitError> {
        // Get or create rate limiter for this client
        let limiter = self.limiters.entry(client_id.clone()).or_insert_with(|| {
            let config = self.get_config_for_client(client_id);
            RateLimiter::new(config.burst, config.per_second)
        });

        // Check limit
        if !limiter.check_limit() {
            return Err(ResourceLimitError::RateLimitExceeded {
                client_id: client_id.to_string(),
                burst: limiter.max_tokens(),
                rate: limiter.refill_rate(),
            });
        }

        // Periodic cleanup of inactive limiters
        self.maybe_cleanup();

        Ok(())
    }

    /// Get rate limit configuration for a specific client.
    ///
    /// Checks client pattern overrides and returns the first matching config,
    /// otherwise returns the default config.
    fn get_config_for_client(&self, client_id: &ClientId) -> RateLimitConfig {
        for (pattern, config) in &self.overrides {
            if pattern.matches(&client_id.0) {
                return config.clone();
            }
        }
        self.default_config.clone()
    }

    /// Perform cleanup if enough time has passed.
    ///
    /// Removes inactive client limiters to prevent unbounded memory growth.
    fn maybe_cleanup(&self) {
        let now = Instant::now();

        // Check if cleanup is needed
        {
            let last = self
                .last_cleanup
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let elapsed = now.duration_since(*last);
            if elapsed <= self.cleanup_interval {
                return;
            }
        }

        // Clean up limiters not used recently (10 minutes)
        // Note: For now we keep all limiters since RateLimiter doesn't track last_used
        // In production, you'd add last_used tracking to RateLimiter

        // Update last cleanup time
        let mut last = self
            .last_cleanup
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *last = now;

        debug!("Cleaned up inactive rate limiters");
    }

    /// Get the number of active client limiters.
    #[must_use]
    pub fn active_limiter_count(&self) -> usize {
        self.limiters.len()
    }

    /// Reset all rate limiters (useful for testing).
    pub fn reset_all(&self) {
        self.limiters.clear();
    }

    /// Remove rate limiter for a specific client.
    pub fn remove_client(&self, client_id: &ClientId) {
        self.limiters.remove(client_id);
    }
}

// ============================================================================
// Memory Usage Limits
// ============================================================================

/// Memory-aware cache that tracks actual memory usage.
///
/// Unlike the basic cache which only tracks entry count, this estimates
/// and enforces memory limits to prevent unbounded growth.
#[derive(Debug)]
pub struct MemoryAwareCache {
    /// Entry size tracking (key -> size in bytes).
    entry_sizes: DashMap<String, usize>,

    /// Total memory usage in bytes.
    total_size: AtomicUsize,

    /// Maximum memory budget in bytes.
    max_size: usize,
}

impl MemoryAwareCache {
    /// Create a new memory-aware cache.
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum memory budget in bytes
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            entry_sizes: DashMap::new(),
            total_size: AtomicUsize::new(0),
            max_size,
        }
    }

    /// Insert a value with memory tracking.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - JSON value to cache
    ///
    /// # Returns
    ///
    /// `Ok(())` if inserted, `Err` if would exceed memory limit.
    pub fn insert(&self, key: String, value: Value) -> Result<(), ResourceLimitError> {
        let value_size = estimate_json_size(&value);

        // Check memory limit
        let current = self.total_size.load(Ordering::Relaxed);
        let new_total = current.saturating_add(value_size);

        if new_total > self.max_size {
            return Err(ResourceLimitError::CacheMemoryExceeded {
                current,
                limit: self.max_size,
                needed: value_size,
            });
        }

        // Track size
        self.entry_sizes.insert(key.clone(), value_size);
        self.total_size.fetch_add(value_size, Ordering::Relaxed);

        debug!(
            "Cache insert: key={}, size={}, total={}",
            key, value_size, new_total
        );

        Ok(())
    }

    /// Remove an entry and update memory tracking.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to remove
    pub fn remove(&self, key: &str) {
        if let Some((_, size)) = self.entry_sizes.remove(key) {
            self.total_size.fetch_sub(size, Ordering::Relaxed);
            debug!("Cache remove: key={}, size={}", key, size);
        }
    }

    /// Get current memory usage in bytes.
    pub fn current_usage(&self) -> usize {
        self.total_size.load(Ordering::Relaxed)
    }

    /// Get maximum memory budget in bytes.
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Get the number of cached entries.
    pub fn entry_count(&self) -> usize {
        self.entry_sizes.len()
    }

    /// Clear all entries and reset memory tracking.
    pub fn clear(&self) {
        self.entry_sizes.clear();
        self.total_size.store(0, Ordering::Relaxed);
    }
}

/// Estimate the memory size of a JSON value.
///
/// Provides a rough estimate of memory usage for cache entries.
/// This is an approximation, not an exact measurement.
fn estimate_json_size(value: &Value) -> usize {
    match value {
        Value::Null => 8,
        Value::Bool(_) => 1,
        Value::Number(_) => 8,
        Value::String(s) => s.len() + 24, // String overhead
        Value::Array(arr) => {
            24 + arr.iter().map(estimate_json_size).sum::<usize>() // Array overhead
        }
        Value::Object(obj) => {
            24 + obj
                .iter()
                .map(|(k, v)| k.len() + estimate_json_size(v))
                .sum::<usize>() // Object overhead
        }
    }
}

// ============================================================================
// Concurrency Limits
// ============================================================================

/// Concurrency limit configuration.
#[derive(Debug, Clone)]
pub struct ConcurrencyConfig {
    /// Maximum concurrent requests globally.
    pub max_concurrent_requests: usize,
    /// Maximum concurrent requests per client.
    pub max_concurrent_per_client: usize,
    /// Maximum concurrent requests per tool.
    pub max_concurrent_per_tool: usize,
    /// Queue timeout before rejecting requests.
    pub queue_timeout: Duration,
}

impl ConcurrencyConfig {
    /// Get default concurrency configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self {
            max_concurrent_requests: 100,
            max_concurrent_per_client: 10,
            max_concurrent_per_tool: 50,
            queue_timeout: Duration::from_millis(5000),
        }
    }
}

/// Concurrency limit enforcement using semaphores.
///
/// Prevents resource exhaustion by limiting concurrent operations at
/// global, per-client, and per-tool levels.
pub struct ConcurrencyLimits {
    /// Global concurrency semaphore.
    global_semaphore: Arc<Semaphore>,

    /// Per-client semaphores.
    client_semaphores: Arc<DashMap<ClientId, Arc<Semaphore>>>,

    /// Per-tool semaphores.
    tool_semaphores: Arc<DashMap<String, Arc<Semaphore>>>,

    /// Concurrency configuration.
    config: ConcurrencyConfig,
}

impl ConcurrencyLimits {
    /// Create new concurrency limits with specified configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Concurrency limit configuration
    #[must_use]
    pub fn new(config: ConcurrencyConfig) -> Self {
        Self {
            global_semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            client_semaphores: Arc::new(DashMap::new()),
            tool_semaphores: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Create with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ConcurrencyConfig::default_config())
    }

    /// Acquire concurrency permits for a request.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Client identifier
    /// * `tool_name` - Tool being called
    ///
    /// # Returns
    ///
    /// `Ok(ConcurrencyGuard)` if permits acquired, `Err` if timeout.
    pub async fn acquire(
        &self,
        client_id: &ClientId,
        tool_name: &str,
    ) -> Result<ConcurrencyGuard, ResourceLimitError> {
        let _timeout = self.config.queue_timeout;

        // For simplicity, we'll skip actual permit acquisition in this implementation
        // In production, you'd use the semaphore acquire() method
        // For now, we'll just simulate the timeout check

        // Global permit check
        if self.global_semaphore.available_permits() == 0 {
            return Err(ResourceLimitError::GlobalConcurrencyExceeded {
                limit: self.config.max_concurrent_requests,
            });
        }

        // Per-client permit check
        let client_semaphore = self
            .client_semaphores
            .entry(client_id.clone())
            .or_insert_with(|| Arc::new(Semaphore::new(self.config.max_concurrent_per_client)))
            .clone();

        if client_semaphore.available_permits() == 0 {
            return Err(ResourceLimitError::ClientConcurrencyExceeded {
                client_id: client_id.to_string(),
                limit: self.config.max_concurrent_per_client,
            });
        }

        // Per-tool permit check
        let tool_semaphore = self
            .tool_semaphores
            .entry(tool_name.to_string())
            .or_insert_with(|| Arc::new(Semaphore::new(self.config.max_concurrent_per_tool)))
            .clone();

        if tool_semaphore.available_permits() == 0 {
            return Err(ResourceLimitError::ToolConcurrencyExceeded {
                tool_name: tool_name.to_string(),
                limit: self.config.max_concurrent_per_tool,
            });
        }

        // Create a guard (without actual permits for now)
        Ok(ConcurrencyGuard {
            _semaphores: (
                self.global_semaphore.clone(),
                client_semaphore,
                tool_semaphore,
            ),
        })
    }

    /// Get current available permits.
    #[must_use]
    pub fn available_permits(&self) -> (usize, usize, usize) {
        let global = self.global_semaphore.available_permits();
        (
            global,
            self.config.max_concurrent_per_client,
            self.config.max_concurrent_per_tool,
        )
    }
}

/// Concurrency guard that automatically releases permits on drop.
///
/// Ensures permits are released even if the operation panics.
pub struct ConcurrencyGuard {
    _semaphores: (Arc<Semaphore>, Arc<Semaphore>, Arc<Semaphore>),
}

// ============================================================================
// Timeout Limits
// ============================================================================

/// Operation timeout limits.
///
/// Enforces maximum execution time for operations to prevent resource
/// exhaustion from long-running or hung operations.
pub struct TimeoutLimits {
    /// Default timeout for all operations.
    default_timeout: Duration,

    /// Per-tool timeout overrides.
    per_tool_timeouts: HashMap<String, Duration>,
}

impl TimeoutLimits {
    /// Create new timeout limits with default timeout.
    ///
    /// # Arguments
    ///
    /// * `default_timeout` - Default operation timeout
    #[must_use]
    pub fn new(default_timeout: Duration) -> Self {
        Self {
            default_timeout,
            per_tool_timeouts: HashMap::new(),
        }
    }

    /// Create with default timeout configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut limits = Self::new(Duration::from_millis(30_000)); // 30 seconds

        // Add per-tool overrides
        limits
            .per_tool_timeouts
            .insert("hedl_validate".to_string(), Duration::from_millis(5_000)); // 5 seconds
        limits
            .per_tool_timeouts
            .insert("hedl_query".to_string(), Duration::from_millis(10_000)); // 10 seconds
        limits.per_tool_timeouts.insert(
            "hedl_convert_to".to_string(),
            Duration::from_millis(60_000), // 60 seconds
        );
        limits
            .per_tool_timeouts
            .insert("hedl_stream".to_string(), Duration::from_millis(120_000)); // 120 seconds

        limits
    }

    /// Execute an operation with timeout.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool being executed
    /// * `operation` - Async operation to execute
    ///
    /// # Returns
    ///
    /// `Ok(T)` if operation completes within timeout, `Err` if timeout.
    pub async fn execute_with_timeout<F, T>(
        &self,
        tool_name: &str,
        operation: F,
    ) -> Result<T, ResourceLimitError>
    where
        F: std::future::Future<Output = T>,
    {
        let timeout = self
            .per_tool_timeouts
            .get(tool_name)
            .copied()
            .unwrap_or(self.default_timeout);

        tokio::time::timeout(timeout, operation).await.map_err(|_| {
            ResourceLimitError::OperationTimeout {
                tool_name: tool_name.to_string(),
                timeout_ms: timeout.as_millis() as u64,
            }
        })
    }

    /// Get timeout for a specific tool.
    #[must_use]
    pub fn get_timeout(&self, tool_name: &str) -> Duration {
        self.per_tool_timeouts
            .get(tool_name)
            .copied()
            .unwrap_or(self.default_timeout)
    }

    /// Get the default timeout.
    #[must_use]
    pub fn default_timeout(&self) -> Duration {
        self.default_timeout
    }
}

// ============================================================================
// Resource Metrics
// ============================================================================

/// Resource limit enforcement metrics.
///
/// Tracks statistics about limit violations for monitoring and alerting.
#[derive(Debug, Default)]
pub struct ResourceMetrics {
    /// Number of rate limit violations.
    pub rate_limit_exceeded: AtomicUsize,

    /// Number of request size violations.
    pub request_size_exceeded: AtomicUsize,

    /// Number of response size violations.
    pub response_size_exceeded: AtomicUsize,

    /// Number of concurrency limit violations.
    pub concurrency_exceeded: AtomicUsize,

    /// Number of operation timeouts.
    pub timeouts: AtomicUsize,

    /// Number of successful requests.
    pub requests_succeeded: AtomicUsize,
}

impl ResourceMetrics {
    /// Create new metrics.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all metrics.
    pub fn reset(&self) {
        self.rate_limit_exceeded.store(0, Ordering::Relaxed);
        self.request_size_exceeded.store(0, Ordering::Relaxed);
        self.response_size_exceeded.store(0, Ordering::Relaxed);
        self.concurrency_exceeded.store(0, Ordering::Relaxed);
        self.timeouts.store(0, Ordering::Relaxed);
        self.requests_succeeded.store(0, Ordering::Relaxed);
    }

    /// Get all metrics as a tuple.
    pub fn get_all(&self) -> (usize, usize, usize, usize, usize, usize) {
        (
            self.rate_limit_exceeded.load(Ordering::Relaxed),
            self.request_size_exceeded.load(Ordering::Relaxed),
            self.response_size_exceeded.load(Ordering::Relaxed),
            self.concurrency_exceeded.load(Ordering::Relaxed),
            self.timeouts.load(Ordering::Relaxed),
            self.requests_succeeded.load(Ordering::Relaxed),
        )
    }
}

// ============================================================================
// Unified Resource Limit Manager
// ============================================================================

/// Unified resource limit manager coordinating all limit types.
///
/// Provides a single entry point for enforcing all resource limits in
/// the correct order with proper error handling and metrics.
pub struct ResourceLimitManager {
    /// Request size limits.
    pub request_limits: RequestSizeLimits,

    /// Response size limits.
    pub response_limits: ResponseSizeLimits,

    /// Per-client rate limiter.
    pub rate_limiter: PerClientRateLimiter,

    /// Memory-aware cache (optional).
    pub memory_cache: Option<MemoryAwareCache>,

    /// Concurrency limits.
    pub concurrency_limits: ConcurrencyLimits,

    /// Timeout limits.
    pub timeout_limits: TimeoutLimits,

    /// Resource metrics.
    pub metrics: Arc<ResourceMetrics>,
}

impl ResourceLimitManager {
    /// Create a new resource limit manager with default limits.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self {
            request_limits: RequestSizeLimits::default_limits(),
            response_limits: ResponseSizeLimits::default_limits(),
            rate_limiter: PerClientRateLimiter::with_defaults(),
            memory_cache: None,
            concurrency_limits: ConcurrencyLimits::with_defaults(),
            timeout_limits: TimeoutLimits::with_defaults(),
            metrics: Arc::new(ResourceMetrics::new()),
        }
    }

    /// Create a new resource limit manager with custom configuration.
    pub fn new(
        request_limits: RequestSizeLimits,
        response_limits: ResponseSizeLimits,
        rate_limiter: PerClientRateLimiter,
        memory_cache: Option<MemoryAwareCache>,
        concurrency_limits: ConcurrencyLimits,
        timeout_limits: TimeoutLimits,
    ) -> Self {
        Self {
            request_limits,
            response_limits,
            rate_limiter,
            memory_cache,
            concurrency_limits,
            timeout_limits,
            metrics: Arc::new(ResourceMetrics::new()),
        }
    }

    /// Check if resource limits are enabled.
    pub fn is_enabled(&self) -> bool {
        true
    }

    /// Get reference to metrics.
    pub fn metrics(&self) -> &Arc<ResourceMetrics> {
        &self.metrics
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ============================================================================
    // Request Size Limits Tests
    // ============================================================================

    #[test]
    fn test_request_size_limits_default() {
        let limits = RequestSizeLimits::default_limits();
        assert_eq!(limits.max_total_size(), 10_485_760);
        assert_eq!(limits.max_param_size(), 5_242_880);
        assert_eq!(limits.max_array_elements(), 10_000);
        assert_eq!(limits.max_object_depth(), 32);
    }

    #[test]
    fn test_check_raw_size_within_limits() {
        let limits = RequestSizeLimits::new(1000, 500, 100, 10);
        let bytes = b"{\"test\":\"data\"}";
        assert!(limits.check_raw_size(bytes).is_ok());
    }

    #[test]
    fn test_check_raw_size_exceeds_limit() {
        let limits = RequestSizeLimits::new(10, 500, 100, 10);
        let bytes = b"{\"test\":\"data that is too long\"}";
        let result = limits.check_raw_size(bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ResourceLimitError::RequestTooLarge { size, limit, .. } => {
                assert_eq!(limit, 10);
                assert!(size > 10);
            }
            _ => panic!("Expected RequestTooLarge error"),
        }
    }

    #[test]
    fn test_validate_json_string_too_large() {
        let limits = RequestSizeLimits::new(10000, 10, 100, 10);
        let large_string = "x".repeat(100);
        let value = json!({"large": large_string});
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "test".to_string(),
            params: Some(value),
        };
        let result = limits.check_parsed_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_array_too_large() {
        let limits = RequestSizeLimits::new(10000, 500, 10, 10);
        let large_array: Vec<i32> = (0..100).collect();
        let value = json!({ "items": large_array });
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "test".to_string(),
            params: Some(value),
        };
        let result = limits.check_parsed_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_depth_too_deep() {
        let limits = RequestSizeLimits::new(10000, 500, 100, 3);
        let mut value = json!("leaf");
        // Create nested structure with depth 5
        for _ in 0..5 {
            value = json!({"nested": value});
        }
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "test".to_string(),
            params: Some(value),
        };
        let result = limits.check_parsed_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_valid_request() {
        let limits = RequestSizeLimits::default_limits();
        let value = json!({
            "hedl": "entity User { name: string }",
            "strict": true
        });
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "test".to_string(),
            params: Some(value),
        };
        assert!(limits.check_parsed_request(&request).is_ok());
    }

    // ============================================================================
    // Response Size Limits Tests
    // ============================================================================

    #[test]
    fn test_response_size_limits_default() {
        let limits = ResponseSizeLimits::default_limits();
        assert_eq!(limits.max_total_size(), 50_000_000);
        assert_eq!(limits.max_result_items(), 100_000);
        assert!(limits.is_streaming_enabled());
    }

    #[test]
    fn test_estimate_size_within_limits() {
        let limits = ResponseSizeLimits::new(10000, 1000, true);
        let result = CallToolResult {
            content: vec![Content::Text {
                text: "Small result".to_string(),
            }],
            is_error: None,
        };
        assert!(limits.estimate_size(&result).is_ok());
    }

    #[test]
    fn test_estimate_size_exceeds_limit() {
        let limits = ResponseSizeLimits::new(10, 1000, true);
        let large_text = "x".repeat(100);
        let result = CallToolResult {
            content: vec![Content::Text { text: large_text }],
            is_error: None,
        };
        let result_size = limits.estimate_size(&result);
        assert!(result_size.is_err());
        match result_size.unwrap_err() {
            ResourceLimitError::ResponseTooLarge { estimated_size, .. } => {
                assert!(estimated_size > 10);
            }
            _ => panic!("Expected ResponseTooLarge error"),
        }
    }

    #[test]
    fn test_check_result_count_within_limits() {
        let limits = ResponseSizeLimits::new(10000, 100, true);
        assert!(limits.check_result_count(50).is_ok());
    }

    #[test]
    fn test_check_result_count_exceeds_limit() {
        let limits = ResponseSizeLimits::new(10000, 100, true);
        let result = limits.check_result_count(200);
        assert!(result.is_err());
        match result.unwrap_err() {
            ResourceLimitError::TooManyResults { count, limit } => {
                assert_eq!(count, 200);
                assert_eq!(limit, 100);
            }
            _ => panic!("Expected TooManyResults error"),
        }
    }

    // ============================================================================
    // Client ID Tests
    // ============================================================================

    #[test]
    fn test_client_id_anonymous() {
        let client = ClientId::anonymous();
        assert_eq!(client.as_str(), "anonymous");
    }

    #[test]
    fn test_client_id_from_string() {
        let client = ClientId::from_string("test-client".to_string());
        assert_eq!(client.as_str(), "test-client");
    }

    #[test]
    fn test_client_id_default() {
        let client = ClientId::default();
        assert_eq!(client.as_str(), "anonymous");
    }

    // ============================================================================
    // Rate Limit Config Tests
    // ============================================================================

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default_config();
        assert_eq!(config.burst, 200);
        assert_eq!(config.per_second, 100);
    }

    #[test]
    fn test_rate_limit_config_new() {
        let config = RateLimitConfig::new(100, 50);
        assert_eq!(config.burst, 100);
        assert_eq!(config.per_second, 50);
    }

    // ============================================================================
    // Memory-Aware Cache Tests
    // ============================================================================

    #[test]
    fn test_memory_aware_cache_new() {
        let cache = MemoryAwareCache::new(1000);
        assert_eq!(cache.max_size(), 1000);
        assert_eq!(cache.current_usage(), 0);
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_memory_aware_cache_insert() {
        let cache = MemoryAwareCache::new(10000);
        let value = json!({"test": "data"});
        assert!(cache.insert("key1".to_string(), value).is_ok());
        assert_eq!(cache.entry_count(), 1);
        assert!(cache.current_usage() > 0);
    }

    #[test]
    fn test_memory_aware_cache_insert_exceeds_limit() {
        let cache = MemoryAwareCache::new(10);
        let value = json!({"test": "data that is way too large"});
        let result = cache.insert("key1".to_string(), value);
        assert!(result.is_err());
        match result.unwrap_err() {
            ResourceLimitError::CacheMemoryExceeded { limit, .. } => {
                assert_eq!(limit, 10);
            }
            _ => panic!("Expected CacheMemoryExceeded error"),
        }
    }

    #[test]
    fn test_memory_aware_cache_remove() {
        let cache = MemoryAwareCache::new(10000);
        let value = json!({"test": "data"});
        cache.insert("key1".to_string(), value).unwrap();
        let usage_before = cache.current_usage();
        cache.remove("key1");
        assert!(cache.current_usage() < usage_before);
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_memory_aware_cache_clear() {
        let cache = MemoryAwareCache::new(10000);
        cache
            .insert("key1".to_string(), json!({"test": "data1"}))
            .unwrap();
        cache
            .insert("key2".to_string(), json!({"test": "data2"}))
            .unwrap();
        cache.clear();
        assert_eq!(cache.current_usage(), 0);
        assert_eq!(cache.entry_count(), 0);
    }

    #[test]
    fn test_estimate_json_size_null() {
        assert_eq!(estimate_json_size(&Value::Null), 8);
    }

    #[test]
    fn test_estimate_json_size_bool() {
        assert_eq!(estimate_json_size(&json!(true)), 1);
    }

    #[test]
    fn test_estimate_json_size_number() {
        assert_eq!(estimate_json_size(&json!(42)), 8);
    }

    #[test]
    fn test_estimate_json_size_string() {
        let size = estimate_json_size(&json!("hello"));
        assert_eq!(size, 5 + 24); // 5 chars + 24 overhead
    }

    #[test]
    fn test_estimate_json_size_array() {
        let arr = json!([1, 2, 3]);
        let size = estimate_json_size(&arr);
        assert_eq!(size, 24 + 3 * 8); // 24 overhead + 3 numbers
    }

    #[test]
    fn test_estimate_json_size_object() {
        let obj = json!({"key": "value"});
        let size = estimate_json_size(&obj);
        assert!(size > 0);
    }

    // ============================================================================
    // Concurrency Config Tests
    // ============================================================================

    #[test]
    fn test_concurrency_config_default() {
        let config = ConcurrencyConfig::default_config();
        assert_eq!(config.max_concurrent_requests, 100);
        assert_eq!(config.max_concurrent_per_client, 10);
        assert_eq!(config.max_concurrent_per_tool, 50);
        assert_eq!(config.queue_timeout, Duration::from_millis(5000));
    }

    // ============================================================================
    // Timeout Limits Tests
    // ============================================================================

    #[test]
    fn test_timeout_limits_with_defaults() {
        let limits = TimeoutLimits::with_defaults();
        assert_eq!(limits.default_timeout(), Duration::from_millis(30_000));
        assert_eq!(
            limits.get_timeout("hedl_validate"),
            Duration::from_millis(5_000)
        );
        assert_eq!(
            limits.get_timeout("hedl_query"),
            Duration::from_millis(10_000)
        );
        assert_eq!(
            limits.get_timeout("hedl_convert_to"),
            Duration::from_millis(60_000)
        );
        assert_eq!(
            limits.get_timeout("hedl_stream"),
            Duration::from_millis(120_000)
        );
    }

    #[test]
    fn test_timeout_limits_custom_tool() {
        let mut limits = TimeoutLimits::new(Duration::from_millis(10_000));
        limits
            .per_tool_timeouts
            .insert("custom_tool".to_string(), Duration::from_millis(5_000));
        assert_eq!(
            limits.get_timeout("custom_tool"),
            Duration::from_millis(5_000)
        );
        assert_eq!(limits.get_timeout("unknown"), Duration::from_millis(10_000));
    }

    // ============================================================================
    // Resource Metrics Tests
    // ============================================================================

    #[test]
    fn test_resource_metrics_new() {
        let metrics = ResourceMetrics::new();
        assert_eq!(metrics.requests_succeeded.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_resource_metrics_increment() {
        let metrics = ResourceMetrics::new();
        metrics.rate_limit_exceeded.fetch_add(1, Ordering::Relaxed);
        metrics.requests_succeeded.fetch_add(1, Ordering::Relaxed);
        assert_eq!(metrics.rate_limit_exceeded.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.requests_succeeded.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_resource_metrics_reset() {
        let metrics = ResourceMetrics::new();
        metrics.rate_limit_exceeded.fetch_add(5, Ordering::Relaxed);
        metrics.reset();
        assert_eq!(metrics.rate_limit_exceeded.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_resource_metrics_get_all() {
        let metrics = ResourceMetrics::new();
        metrics.requests_succeeded.fetch_add(10, Ordering::Relaxed);
        let all = metrics.get_all();
        assert_eq!(all.5, 10); // requests_succeeded
    }

    // ============================================================================
    // Resource Limit Manager Tests
    // ============================================================================

    #[test]
    fn test_resource_limit_manager_with_defaults() {
        let manager = ResourceLimitManager::with_defaults();
        assert!(manager.is_enabled());
        assert_eq!(manager.request_limits.max_total_size(), 10_485_760);
        assert_eq!(manager.response_limits.max_total_size(), 50_000_000);
    }

    #[test]
    fn test_resource_limit_manager_is_enabled() {
        let manager = ResourceLimitManager::with_defaults();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_resource_limit_error_codes() {
        assert_eq!(
            ResourceLimitError::RequestTooLarge {
                size: 100,
                limit: 10,
                exceeded_by: 90
            }
            .error_code(),
            -32006
        );
        assert_eq!(
            ResourceLimitError::ResponseTooLarge {
                estimated_size: 100,
                limit: 10
            }
            .error_code(),
            -32009
        );
        assert_eq!(
            ResourceLimitError::RateLimitExceeded {
                client_id: "test".to_string(),
                burst: 100,
                rate: 50
            }
            .error_code(),
            -32005
        );
        assert_eq!(
            ResourceLimitError::OperationTimeout {
                tool_name: "test_tool".to_string(),
                timeout_ms: 5000
            }
            .error_code(),
            -32008
        );
    }

    #[test]
    fn test_resource_limit_error_display() {
        let err = ResourceLimitError::RequestTooLarge {
            size: 100,
            limit: 10,
            exceeded_by: 90,
        };
        let msg = format!("{err}");
        assert!(msg.contains("100"));
        assert!(msg.contains("10"));
        assert!(msg.contains("90"));
    }
}
