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

//! Resource limit configuration loading from TOML.
//!
//! Provides configuration structures and loading functions for resource limits
//! from TOML configuration files.

use crate::resource_limits::{
    ConcurrencyConfig, ConcurrencyLimits, MemoryAwareCache, PerClientRateLimiter, RateLimitConfig,
    RequestSizeLimits, ResourceLimitManager, ResponseSizeLimits, TimeoutLimits,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tracing::info;

/// Top-level configuration for resource limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitConfig {
    /// Whether resource limits are enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Request size limits.
    #[serde(default)]
    pub request: RequestSizeConfig,

    /// Response size limits.
    #[serde(default)]
    pub response: ResponseSizeConfig,

    /// Rate limiting configuration.
    #[serde(default)]
    pub rate_limiting: RateLimitingConfig,

    /// Memory limits.
    #[serde(default)]
    pub memory: MemoryConfig,

    /// Concurrency limits.
    #[serde(default)]
    pub concurrency: ConcurrencyConfigTOML,

    /// Timeout limits.
    #[serde(default)]
    pub timeouts: TimeoutConfig,
}

impl Default for ResourceLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            request: RequestSizeConfig::default(),
            response: ResponseSizeConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            memory: MemoryConfig::default(),
            concurrency: ConcurrencyConfigTOML::default(),
            timeouts: TimeoutConfig::default(),
        }
    }
}

fn default_enabled() -> bool {
    true
}

/// Request size limits configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSizeConfig {
    /// Maximum total request size in bytes.
    #[serde(default = "default_max_request_size")]
    pub max_total_size_bytes: usize,

    /// Maximum individual parameter size in bytes.
    #[serde(default = "default_max_param_size")]
    pub max_param_size_bytes: usize,

    /// Maximum array element count.
    #[serde(default = "default_max_array_elements")]
    pub max_array_elements: usize,

    /// Maximum JSON object nesting depth.
    #[serde(default = "default_max_object_depth")]
    pub max_object_depth: usize,
}

impl Default for RequestSizeConfig {
    fn default() -> Self {
        Self {
            max_total_size_bytes: default_max_request_size(),
            max_param_size_bytes: default_max_param_size(),
            max_array_elements: default_max_array_elements(),
            max_object_depth: default_max_object_depth(),
        }
    }
}

fn default_max_request_size() -> usize {
    10_485_760 // 10 MB
}

fn default_max_param_size() -> usize {
    5_242_880 // 5 MB
}

fn default_max_array_elements() -> usize {
    10_000
}

fn default_max_object_depth() -> usize {
    32
}

/// Response size limits configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSizeConfig {
    /// Maximum total response size in bytes.
    #[serde(default = "default_max_response_size")]
    pub max_total_size_bytes: usize,

    /// Maximum number of result items for array responses.
    #[serde(default = "default_max_result_items")]
    pub max_result_items: usize,

    /// Whether streaming is enabled for large results.
    #[serde(default = "default_enable_streaming")]
    pub enable_streaming: bool,
}

impl Default for ResponseSizeConfig {
    fn default() -> Self {
        Self {
            max_total_size_bytes: default_max_response_size(),
            max_result_items: default_max_result_items(),
            enable_streaming: default_enable_streaming(),
        }
    }
}

fn default_max_response_size() -> usize {
    50_000_000 // 50 MB
}

fn default_max_result_items() -> usize {
    100_000
}

fn default_enable_streaming() -> bool {
    true
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Rate limiting mode: "`per_client`" or "global".
    #[serde(default = "default_rate_limit_mode")]
    pub mode: String,

    /// Default burst capacity.
    #[serde(default = "default_burst")]
    pub default_burst: usize,

    /// Default refill rate (requests per second).
    #[serde(default = "default_per_second")]
    pub default_per_second: usize,

    /// Cleanup interval for inactive client limiters (seconds).
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval_seconds: usize,

    /// Client-specific rate limit overrides.
    #[serde(default)]
    pub overrides: Vec<RateLimitOverride>,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            mode: default_rate_limit_mode(),
            default_burst: default_burst(),
            default_per_second: default_per_second(),
            cleanup_interval_seconds: default_cleanup_interval(),
            overrides: Vec::new(),
        }
    }
}

fn default_rate_limit_mode() -> String {
    "per_client".to_string()
}

fn default_burst() -> usize {
    200
}

fn default_per_second() -> usize {
    100
}

fn default_cleanup_interval() -> usize {
    300 // 5 minutes
}

/// Rate limit override for a client pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitOverride {
    /// Glob pattern matching client IDs.
    pub client_pattern: String,

    /// Burst capacity for matching clients.
    pub burst: usize,

    /// Refill rate for matching clients.
    pub per_second: usize,
}

/// Memory limits configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum cache memory in bytes.
    #[serde(default = "default_max_cache_memory")]
    pub max_cache_memory_bytes: usize,

    /// Maximum operation memory in bytes.
    #[serde(default = "default_max_operation_memory")]
    pub max_operation_memory_bytes: usize,

    /// Whether memory tracking is enabled.
    #[serde(default = "default_enable_memory_tracking")]
    pub enable_memory_tracking: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_cache_memory_bytes: default_max_cache_memory(),
            max_operation_memory_bytes: default_max_operation_memory(),
            enable_memory_tracking: default_enable_memory_tracking(),
        }
    }
}

fn default_max_cache_memory() -> usize {
    104_857_600 // 100 MB
}

fn default_max_operation_memory() -> usize {
    52_428_800 // 50 MB
}

fn default_enable_memory_tracking() -> bool {
    true
}

/// Concurrency limits configuration (TOML format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfigTOML {
    /// Maximum concurrent requests globally.
    #[serde(default = "default_max_concurrent_requests")]
    pub max_concurrent_requests: usize,

    /// Maximum concurrent requests per client.
    #[serde(default = "default_max_concurrent_per_client")]
    pub max_concurrent_per_client: usize,

    /// Maximum concurrent requests per tool.
    #[serde(default = "default_max_concurrent_per_tool")]
    pub max_concurrent_per_tool: usize,

    /// Queue timeout before rejecting requests (milliseconds).
    #[serde(default = "default_queue_timeout")]
    pub queue_timeout_ms: usize,
}

impl Default for ConcurrencyConfigTOML {
    fn default() -> Self {
        Self {
            max_concurrent_requests: default_max_concurrent_requests(),
            max_concurrent_per_client: default_max_concurrent_per_client(),
            max_concurrent_per_tool: default_max_concurrent_per_tool(),
            queue_timeout_ms: default_queue_timeout(),
        }
    }
}

fn default_max_concurrent_requests() -> usize {
    100
}

fn default_max_concurrent_per_client() -> usize {
    10
}

fn default_max_concurrent_per_tool() -> usize {
    50
}

fn default_queue_timeout() -> usize {
    5000 // 5 seconds
}

/// Timeout limits configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Default operation timeout (milliseconds).
    #[serde(default = "default_timeout")]
    pub default_timeout_ms: usize,

    /// Per-tool timeout overrides.
    #[serde(default)]
    pub per_tool: std::collections::HashMap<String, usize>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        let mut per_tool = std::collections::HashMap::new();
        per_tool.insert("hedl_validate".to_string(), 5_000);
        per_tool.insert("hedl_query".to_string(), 10_000);
        per_tool.insert("hedl_convert_to".to_string(), 60_000);
        per_tool.insert("hedl_stream".to_string(), 120_000);

        Self {
            default_timeout_ms: default_timeout(),
            per_tool,
        }
    }
}

fn default_timeout() -> usize {
    30_000 // 30 seconds
}

impl ResourceLimitConfig {
    /// Load configuration from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    ///
    /// `Ok(ResourceLimitConfig)` if loading succeeds, `Err` otherwise.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| ConfigError::IoError {
            path: path.as_ref().display().to_string(),
            source: e,
        })?;

        let config: ResourceLimitConfig =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError {
                path: path.as_ref().display().to_string(),
                source: e,
            })?;

        info!(
            "Loaded resource limit config from {}",
            path.as_ref().display()
        );

        Ok(config)
    }

    /// Load configuration from a TOML string.
    ///
    /// # Arguments
    ///
    /// * `content` - TOML configuration content
    ///
    /// # Returns
    ///
    /// `Ok(ResourceLimitConfig)` if parsing succeeds, `Err` otherwise.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        let config: ResourceLimitConfig =
            toml::from_str(content).map_err(|e| ConfigError::ParseError {
                path: "<string>".to_string(),
                source: e,
            })?;

        info!("Loaded resource limit config from string");

        Ok(config)
    }

    /// Convert this configuration into a `ResourceLimitManager`.
    ///
    /// # Returns
    ///
    /// A configured `ResourceLimitManager` instance.
    #[must_use]
    pub fn to_manager(&self) -> ResourceLimitManager {
        // Request size limits
        let request_limits = RequestSizeLimits::new(
            self.request.max_total_size_bytes,
            self.request.max_param_size_bytes,
            self.request.max_array_elements,
            self.request.max_object_depth,
        );

        // Response size limits
        let response_limits = ResponseSizeLimits::new(
            self.response.max_total_size_bytes,
            self.response.max_result_items,
            self.response.enable_streaming,
        );

        // Rate limiting
        let default_config = RateLimitConfig::new(
            self.rate_limiting.default_burst,
            self.rate_limiting.default_per_second,
        );

        let overrides = self
            .rate_limiting
            .overrides
            .iter()
            .map(|o| {
                (
                    o.client_pattern.clone(),
                    RateLimitConfig::new(o.burst, o.per_second),
                )
            })
            .collect();

        let cleanup_interval =
            Duration::from_secs(self.rate_limiting.cleanup_interval_seconds as u64);
        let rate_limiter = PerClientRateLimiter::new(default_config, overrides, cleanup_interval);

        // Memory limits
        let memory_cache = if self.memory.enable_memory_tracking {
            Some(MemoryAwareCache::new(self.memory.max_cache_memory_bytes))
        } else {
            None
        };

        // Concurrency limits
        let concurrency_config = ConcurrencyConfig {
            max_concurrent_requests: self.concurrency.max_concurrent_requests,
            max_concurrent_per_client: self.concurrency.max_concurrent_per_client,
            max_concurrent_per_tool: self.concurrency.max_concurrent_per_tool,
            queue_timeout: Duration::from_millis(self.concurrency.queue_timeout_ms as u64),
        };
        let concurrency_limits = ConcurrencyLimits::new(concurrency_config);

        // Timeout limits
        let timeout_limits = TimeoutLimits::new(Duration::from_millis(
            self.timeouts.default_timeout_ms as u64,
        ));
        // Note: Per-tool timeouts would need to be set here
        // For simplicity, we'll use the defaults

        ResourceLimitManager::new(
            request_limits,
            response_limits,
            rate_limiter,
            memory_cache,
            concurrency_limits,
            timeout_limits,
        )
    }

    /// Validate the configuration.
    ///
    /// # Returns
    ///
    /// `Ok(())` if configuration is valid, `Err` with details if invalid.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate request size limits
        if self.request.max_total_size_bytes == 0 {
            return Err(ConfigError::ValidationError(
                "max_total_size_bytes must be greater than 0".to_string(),
            ));
        }

        if self.request.max_param_size_bytes > self.request.max_total_size_bytes {
            return Err(ConfigError::ValidationError(
                "max_param_size_bytes cannot exceed max_total_size_bytes".to_string(),
            ));
        }

        // Validate response size limits
        if self.response.max_total_size_bytes == 0 {
            return Err(ConfigError::ValidationError(
                "max_total_size_bytes (response) must be greater than 0".to_string(),
            ));
        }

        // Validate rate limiting
        if self.rate_limiting.default_burst == 0 {
            return Err(ConfigError::ValidationError(
                "default_burst must be greater than 0".to_string(),
            ));
        }

        if self.rate_limiting.default_per_second == 0 {
            return Err(ConfigError::ValidationError(
                "default_per_second must be greater than 0".to_string(),
            ));
        }

        // Validate concurrency limits
        if self.concurrency.max_concurrent_requests == 0 {
            return Err(ConfigError::ValidationError(
                "max_concurrent_requests must be greater than 0".to_string(),
            ));
        }

        if self.concurrency.max_concurrent_per_client == 0 {
            return Err(ConfigError::ValidationError(
                "max_concurrent_per_client must be greater than 0".to_string(),
            ));
        }

        if self.concurrency.max_concurrent_per_tool == 0 {
            return Err(ConfigError::ValidationError(
                "max_concurrent_per_tool must be greater than 0".to_string(),
            ));
        }

        // Validate timeout
        if self.timeouts.default_timeout_ms == 0 {
            return Err(ConfigError::ValidationError(
                "default_timeout_ms must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// Configuration error types.
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum ConfigError {
    /// IO error reading configuration file.
    #[error("IO error reading config from '{path}': {source}")]
    IoError {
        /// Path to the configuration file.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// TOML parsing error.
    #[error("Failed to parse TOML from '{path}': {source}")]
    ParseError {
        /// Path to the configuration file.
        path: String,
        /// Underlying TOML parsing error.
        #[source]
        source: toml::de::Error,
    },

    /// Configuration validation error.
    #[error("Configuration validation failed: {0}")]
    ValidationError(
        /// Description of the validation error.
        String,
    ),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Default Config Tests
    // ============================================================================

    #[test]
    fn test_default_config() {
        let config = ResourceLimitConfig::default();
        assert!(config.enabled);
        assert_eq!(config.request.max_total_size_bytes, 10_485_760);
        assert_eq!(config.response.max_total_size_bytes, 50_000_000);
        assert_eq!(config.rate_limiting.default_burst, 200);
        assert_eq!(config.rate_limiting.default_per_second, 100);
        assert_eq!(config.memory.max_cache_memory_bytes, 104_857_600);
        assert_eq!(config.concurrency.max_concurrent_requests, 100);
        assert_eq!(config.timeouts.default_timeout_ms, 30_000);
    }

    #[test]
    fn test_request_size_config_default() {
        let config = RequestSizeConfig::default();
        assert_eq!(config.max_total_size_bytes, 10_485_760);
        assert_eq!(config.max_param_size_bytes, 5_242_880);
        assert_eq!(config.max_array_elements, 10_000);
        assert_eq!(config.max_object_depth, 32);
    }

    #[test]
    fn test_response_size_config_default() {
        let config = ResponseSizeConfig::default();
        assert_eq!(config.max_total_size_bytes, 50_000_000);
        assert_eq!(config.max_result_items, 100_000);
        assert!(config.enable_streaming);
    }

    #[test]
    fn test_rate_limiting_config_default() {
        let config = RateLimitingConfig::default();
        assert_eq!(config.mode, "per_client");
        assert_eq!(config.default_burst, 200);
        assert_eq!(config.default_per_second, 100);
        assert_eq!(config.cleanup_interval_seconds, 300);
        assert!(config.overrides.is_empty());
    }

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert_eq!(config.max_cache_memory_bytes, 104_857_600);
        assert_eq!(config.max_operation_memory_bytes, 52_428_800);
        assert!(config.enable_memory_tracking);
    }

    #[test]
    fn test_concurrency_config_default() {
        let config = ConcurrencyConfigTOML::default();
        assert_eq!(config.max_concurrent_requests, 100);
        assert_eq!(config.max_concurrent_per_client, 10);
        assert_eq!(config.max_concurrent_per_tool, 50);
        assert_eq!(config.queue_timeout_ms, 5000);
    }

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.default_timeout_ms, 30_000);
        assert_eq!(config.per_tool.get("hedl_validate"), Some(&5_000));
        assert_eq!(config.per_tool.get("hedl_query"), Some(&10_000));
        assert_eq!(config.per_tool.get("hedl_convert_to"), Some(&60_000));
        assert_eq!(config.per_tool.get("hedl_stream"), Some(&120_000));
    }

    // ============================================================================
    // Config Parsing Tests
    // ============================================================================

    #[test]
    fn test_parse_config_from_str() {
        let toml_str = r#"
enabled = true

[request]
max_total_size_bytes = 2048
max_param_size_bytes = 1024
max_array_elements = 100
max_object_depth = 10

[response]
max_total_size_bytes = 4096
max_result_items = 500
enable_streaming = false

[rate_limiting]
mode = "per_client"
default_burst = 50
default_per_second = 25
cleanup_interval_seconds = 60
"#;

        let result = ResourceLimitConfig::from_str(toml_str);
        assert!(result.is_ok(), "Failed to parse: {result:?}");
        let config = result.unwrap();
        assert!(config.enabled);
        assert_eq!(config.request.max_total_size_bytes, 2048);
        assert_eq!(config.response.max_total_size_bytes, 4096);
        assert_eq!(config.rate_limiting.default_burst, 50);
    }

    #[test]
    fn test_parse_config_with_overrides() {
        let toml_str = r#"
[rate_limiting]
default_burst = 200
default_per_second = 100

[[rate_limiting.overrides]]
client_pattern = "premium-*"
burst = 1000
per_second = 500

[[rate_limiting.overrides]]
client_pattern = "free-*"
burst = 50
per_second = 10
"#;

        let result = ResourceLimitConfig::from_str(toml_str);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.rate_limiting.overrides.len(), 2);
        assert_eq!(
            config.rate_limiting.overrides[0].client_pattern,
            "premium-*"
        );
        assert_eq!(config.rate_limiting.overrides[0].burst, 1000);
        assert_eq!(config.rate_limiting.overrides[1].client_pattern, "free-*");
        assert_eq!(config.rate_limiting.overrides[1].burst, 50);
    }

    #[test]
    fn test_parse_invalid_toml() {
        let invalid_toml = r"
[resource_limits
enabled = true
"; // Missing closing bracket

        let result = ResourceLimitConfig::from_str(invalid_toml);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ParseError { .. } => {}
            _ => panic!("Expected ParseError"),
        }
    }

    // ============================================================================
    // Config Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_valid_config() {
        let config = ResourceLimitConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_total_size() {
        let mut config = ResourceLimitConfig::default();
        config.request.max_total_size_bytes = 0;
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationError(msg) => {
                assert!(msg.contains("max_total_size_bytes"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validate_param_exceeds_total() {
        let mut config = ResourceLimitConfig::default();
        config.request.max_param_size_bytes = 20_000_000; // Exceeds total
        config.request.max_total_size_bytes = 10_000_000;
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationError(msg) => {
                assert!(msg.contains("max_param_size_bytes"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validate_zero_burst() {
        let mut config = ResourceLimitConfig::default();
        config.rate_limiting.default_burst = 0;
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationError(msg) => {
                assert!(msg.contains("default_burst"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validate_zero_per_second() {
        let mut config = ResourceLimitConfig::default();
        config.rate_limiting.default_per_second = 0;
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationError(msg) => {
                assert!(msg.contains("default_per_second"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validate_zero_concurrent_requests() {
        let mut config = ResourceLimitConfig::default();
        config.concurrency.max_concurrent_requests = 0;
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationError(msg) => {
                assert!(msg.contains("max_concurrent_requests"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validate_zero_timeout() {
        let mut config = ResourceLimitConfig::default();
        config.timeouts.default_timeout_ms = 0;
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationError(msg) => {
                assert!(msg.contains("default_timeout_ms"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    // ============================================================================
    // To Manager Tests
    // ============================================================================

    #[test]
    fn test_to_manager() {
        let config = ResourceLimitConfig::default();
        let manager = config.to_manager();
        assert!(manager.is_enabled());
        assert_eq!(manager.request_limits.max_total_size(), 10_485_760);
        assert_eq!(manager.response_limits.max_total_size(), 50_000_000);
    }

    #[test]
    fn test_to_manager_with_memory_tracking() {
        let mut config = ResourceLimitConfig::default();
        config.memory.enable_memory_tracking = true;
        let manager = config.to_manager();
        assert!(manager.memory_cache.is_some());
        assert_eq!(
            manager.memory_cache.as_ref().unwrap().max_size(),
            104_857_600
        );
    }

    #[test]
    fn test_to_manager_without_memory_tracking() {
        let mut config = ResourceLimitConfig::default();
        config.memory.enable_memory_tracking = false;
        let manager = config.to_manager();
        assert!(manager.memory_cache.is_none());
    }

    // ============================================================================
    // Config Error Tests
    // ============================================================================

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::ValidationError("test error".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("test error"));
    }

    #[test]
    fn test_rate_limit_override() {
        let override_config = RateLimitOverride {
            client_pattern: "test-*".to_string(),
            burst: 500,
            per_second: 250,
        };
        assert_eq!(override_config.client_pattern, "test-*");
        assert_eq!(override_config.burst, 500);
        assert_eq!(override_config.per_second, 250);
    }
}
