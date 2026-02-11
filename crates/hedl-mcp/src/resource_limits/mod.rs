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

mod client;
mod concurrency;
mod error;
mod memory_cache;
mod metrics;
mod rate_limiter;
mod request_limits;
mod response_limits;
mod timeout;

// Re-export all public types
pub use client::ClientId;
pub use concurrency::{ConcurrencyConfig, ConcurrencyGuard, ConcurrencyLimits};
pub use error::ResourceLimitError;
pub use memory_cache::MemoryAwareCache;
pub use metrics::ResourceMetrics;
pub use rate_limiter::{PerClientRateLimiter, RateLimitConfig};
pub use request_limits::RequestSizeLimits;
pub use response_limits::ResponseSizeLimits;
pub use timeout::TimeoutLimits;

use std::sync::Arc;

/// Unified resource limit manager coordinating all limit types.
///
/// Provides a single entry point for enforcing all resource limits in
/// the correct order with proper error handling and metrics.
pub struct LimitEnforcer {
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

impl LimitEnforcer {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{CallToolResult, Content, JsonRpcRequest};
    use serde_json::json;
    use std::sync::atomic::Ordering;

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
        use memory_cache::estimate_json_size;
        use serde_json::Value;
        assert_eq!(estimate_json_size(&Value::Null), 8);
    }

    #[test]
    fn test_estimate_json_size_bool() {
        use memory_cache::estimate_json_size;
        assert_eq!(estimate_json_size(&json!(true)), 1);
    }

    #[test]
    fn test_estimate_json_size_number() {
        use memory_cache::estimate_json_size;
        assert_eq!(estimate_json_size(&json!(42)), 8);
    }

    #[test]
    fn test_estimate_json_size_string() {
        use memory_cache::estimate_json_size;
        let size = estimate_json_size(&json!("hello"));
        assert_eq!(size, 5 + 24); // 5 chars + 24 overhead
    }

    #[test]
    fn test_estimate_json_size_array() {
        use memory_cache::estimate_json_size;
        let arr = json!([1, 2, 3]);
        let size = estimate_json_size(&arr);
        assert_eq!(size, 24 + 3 * 8); // 24 overhead + 3 numbers
    }

    #[test]
    fn test_estimate_json_size_object() {
        use memory_cache::estimate_json_size;
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
        assert_eq!(config.queue_timeout, std::time::Duration::from_millis(5000));
    }

    // ============================================================================
    // Timeout Limits Tests
    // ============================================================================

    #[test]
    fn test_timeout_limits_with_defaults() {
        let limits = TimeoutLimits::with_defaults();
        assert_eq!(
            limits.default_timeout(),
            std::time::Duration::from_millis(30_000)
        );
        assert_eq!(
            limits.get_timeout("hedl_validate"),
            std::time::Duration::from_millis(5_000)
        );
        assert_eq!(
            limits.get_timeout("hedl_query"),
            std::time::Duration::from_millis(10_000)
        );
        assert_eq!(
            limits.get_timeout("hedl_convert_to"),
            std::time::Duration::from_millis(60_000)
        );
        assert_eq!(
            limits.get_timeout("hedl_stream"),
            std::time::Duration::from_millis(120_000)
        );
    }

    #[test]
    fn test_timeout_limits_custom_tool() {
        let mut limits = TimeoutLimits::new(std::time::Duration::from_millis(10_000));
        limits.set_tool_timeout(
            "custom_tool".to_string(),
            std::time::Duration::from_millis(5_000),
        );
        assert_eq!(
            limits.get_timeout("custom_tool"),
            std::time::Duration::from_millis(5_000)
        );
        assert_eq!(
            limits.get_timeout("unknown"),
            std::time::Duration::from_millis(10_000)
        );
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
        let manager = LimitEnforcer::with_defaults();
        assert!(manager.is_enabled());
        assert_eq!(manager.request_limits.max_total_size(), 10_485_760);
        assert_eq!(manager.response_limits.max_total_size(), 50_000_000);
    }

    #[test]
    fn test_resource_limit_manager_is_enabled() {
        let manager = LimitEnforcer::with_defaults();
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
