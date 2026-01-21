// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive resource limit tests for hedl-mcp.
//!
//! Tests all resource limit categories: request size, response size,
//! rate limiting, memory limits, concurrency, and timeouts.

use hedl_mcp::*;
use serde_json::json;
use std::time::Duration;

// ============================================================================
// PerClientRateLimiter Tests
// ============================================================================

#[test]
fn test_per_client_rate_limiter_basic() {
    let limiter = PerClientRateLimiter::with_defaults();
    let client = ClientId::from_string("test-client".to_string());

    // Should allow initial requests
    for _ in 0..100 {
        assert!(limiter.check_limit(&client).is_ok());
    }
}

#[test]
fn test_per_client_rate_limiter_different_clients() {
    let limiter = PerClientRateLimiter::with_defaults();
    let client1 = ClientId::from_string("client-1".to_string());
    let client2 = ClientId::from_string("client-2".to_string());

    // Each client has independent limits
    for _ in 0..100 {
        assert!(limiter.check_limit(&client1).is_ok());
        assert!(limiter.check_limit(&client2).is_ok());
    }

    assert_eq!(limiter.active_limiter_count(), 2);
}

#[test]
fn test_per_client_rate_limiter_with_overrides() {
    let overrides = vec![
        ("premium-*".to_string(), RateLimitConfig::new(1000, 500)),
        ("free-*".to_string(), RateLimitConfig::new(10, 5)),
    ];

    let limiter = PerClientRateLimiter::new(
        RateLimitConfig::new(200, 100),
        overrides,
        Duration::from_secs(300),
    );

    let premium_client = ClientId::from_string("premium-user-1".to_string());
    let free_client = ClientId::from_string("free-user-1".to_string());

    // Premium client should have higher limits
    for _ in 0..200 {
        assert!(limiter.check_limit(&premium_client).is_ok());
    }

    // Free client should hit limits faster
    for _ in 0..10 {
        limiter.check_limit(&free_client).ok();
    }
}

#[test]
fn test_per_client_rate_limiter_reset() {
    let limiter = PerClientRateLimiter::with_defaults();
    let client = ClientId::from_string("test-client".to_string());

    limiter.check_limit(&client).ok();
    assert_eq!(limiter.active_limiter_count(), 1);

    limiter.reset_all();
    assert_eq!(limiter.active_limiter_count(), 0);
}

#[test]
fn test_per_client_rate_limiter_remove_client() {
    let limiter = PerClientRateLimiter::with_defaults();
    let client = ClientId::from_string("test-client".to_string());

    limiter.check_limit(&client).ok();
    assert_eq!(limiter.active_limiter_count(), 1);

    limiter.remove_client(&client);
    assert_eq!(limiter.active_limiter_count(), 0);
}

// ============================================================================
// MemoryAwareCache Advanced Tests
// ============================================================================

#[test]
fn test_memory_cache_size_tracking() {
    let cache = MemoryAwareCache::new(10000);

    cache
        .insert("key1".to_string(), json!({"data": "value1"}))
        .unwrap();
    let initial_usage = cache.current_usage();
    assert!(initial_usage > 0);

    cache
        .insert("key2".to_string(), json!({"data": "value2"}))
        .unwrap();
    let after_second = cache.current_usage();
    assert!(after_second > initial_usage);
}

#[test]
fn test_memory_cache_exceeds_limit() {
    let cache = MemoryAwareCache::new(100); // Very small limit

    let large_value = json!({"data": "x".repeat(1000)});
    let result = cache.insert("key1".to_string(), large_value);

    assert!(result.is_err());
    match result.unwrap_err() {
        ResourceLimitError::CacheMemoryExceeded { .. } => {}
        _ => panic!("Expected CacheMemoryExceeded error"),
    }
}

#[test]
fn test_memory_cache_remove_updates_size() {
    let cache = MemoryAwareCache::new(10000);

    cache
        .insert("key1".to_string(), json!({"data": "value"}))
        .unwrap();
    let with_entry = cache.current_usage();

    cache.remove("key1");
    let after_remove = cache.current_usage();

    assert!(after_remove < with_entry);
}

#[test]
fn test_memory_cache_clear_resets_size() {
    let cache = MemoryAwareCache::new(10000);

    cache.insert("key1".to_string(), json!("value1")).unwrap();
    cache.insert("key2".to_string(), json!("value2")).unwrap();

    assert!(cache.current_usage() > 0);

    cache.clear();

    assert_eq!(cache.current_usage(), 0);
    assert_eq!(cache.entry_count(), 0);
}

#[test]
fn test_memory_cache_entry_count() {
    let cache = MemoryAwareCache::new(10000);

    assert_eq!(cache.entry_count(), 0);

    cache.insert("key1".to_string(), json!("value")).unwrap();
    assert_eq!(cache.entry_count(), 1);

    cache.insert("key2".to_string(), json!("value")).unwrap();
    assert_eq!(cache.entry_count(), 2);

    cache.remove("key1");
    assert_eq!(cache.entry_count(), 1);
}

// ============================================================================
// ConcurrencyLimits Tests
// ============================================================================

#[tokio::test]
async fn test_concurrency_limits_available_permits() {
    let config = ConcurrencyConfig {
        max_concurrent_requests: 100,
        max_concurrent_per_client: 10,
        max_concurrent_per_tool: 50,
        queue_timeout: Duration::from_millis(1000),
    };

    let limits = ConcurrencyLimits::new(config);
    let (global, _, _) = limits.available_permits();
    assert_eq!(global, 100);
}

#[tokio::test]
async fn test_concurrency_limits_acquire() {
    let limits = ConcurrencyLimits::with_defaults();
    let client = ClientId::from_string("test-client".to_string());

    let result: Result<ConcurrencyGuard, ResourceLimitError> =
        limits.acquire(&client, "test_tool").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrency_limits_different_tools() {
    let limits = ConcurrencyLimits::with_defaults();
    let client = ClientId::from_string("test-client".to_string());

    let _guard1: ConcurrencyGuard = limits.acquire(&client, "tool1").await.unwrap();
    let _guard2: ConcurrencyGuard = limits.acquire(&client, "tool2").await.unwrap();

    // Should allow different tools
}

// ============================================================================
// TimeoutLimits Tests
// ============================================================================

#[test]
fn test_timeout_limits_defaults() {
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
fn test_timeout_limits_unknown_tool() {
    let limits = TimeoutLimits::with_defaults();

    // Unknown tools use default timeout
    assert_eq!(
        limits.get_timeout("unknown_tool"),
        Duration::from_millis(30_000)
    );
}

#[tokio::test]
async fn test_timeout_limits_execute_success() {
    let limits = TimeoutLimits::with_defaults();

    let result: Result<i32, ResourceLimitError> = limits
        .execute_with_timeout("test_tool", async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        })
        .await;

    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_timeout_limits_execute_timeout() {
    let limits = TimeoutLimits::new(Duration::from_millis(10));

    let result: Result<i32, ResourceLimitError> = limits
        .execute_with_timeout("test_tool", async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ResourceLimitError::OperationTimeout { .. } => {}
        _ => panic!("Expected OperationTimeout error"),
    }
}

// ============================================================================
// ResourceMetrics Tests
// ============================================================================

#[test]
fn test_resource_metrics_initial() {
    let metrics = ResourceMetrics::new();
    let (rate_limit, request_size, response_size, concurrency, timeouts, succeeded) =
        metrics.get_all();

    assert_eq!(rate_limit, 0);
    assert_eq!(request_size, 0);
    assert_eq!(response_size, 0);
    assert_eq!(concurrency, 0);
    assert_eq!(timeouts, 0);
    assert_eq!(succeeded, 0);
}

#[test]
fn test_resource_metrics_increment() {
    let metrics = ResourceMetrics::new();

    metrics
        .rate_limit_exceeded
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    metrics
        .request_size_exceeded
        .fetch_add(2, std::sync::atomic::Ordering::Relaxed);
    metrics
        .requests_succeeded
        .fetch_add(10, std::sync::atomic::Ordering::Relaxed);

    let (rate_limit, request_size, _, _, _, succeeded) = metrics.get_all();
    assert_eq!(rate_limit, 1);
    assert_eq!(request_size, 2);
    assert_eq!(succeeded, 10);
}

#[test]
fn test_resource_metrics_reset() {
    let metrics = ResourceMetrics::new();

    metrics
        .rate_limit_exceeded
        .fetch_add(5, std::sync::atomic::Ordering::Relaxed);
    metrics
        .requests_succeeded
        .fetch_add(10, std::sync::atomic::Ordering::Relaxed);

    metrics.reset();

    let (rate_limit, _, _, _, _, succeeded) = metrics.get_all();
    assert_eq!(rate_limit, 0);
    assert_eq!(succeeded, 0);
}

// ============================================================================
// ResourceLimitManager Tests
// ============================================================================

#[test]
fn test_resource_limit_manager_creation() {
    let manager = ResourceLimitManager::with_defaults();

    assert!(manager.is_enabled());
    assert_eq!(manager.request_limits.max_total_size(), 10_485_760);
    assert_eq!(manager.response_limits.max_total_size(), 50_000_000);
}

#[test]
fn test_resource_limit_manager_metrics() {
    let manager = ResourceLimitManager::with_defaults();
    let metrics = manager.metrics();

    let (rate_limit, _, _, _, _, succeeded) = metrics.get_all();
    assert_eq!(rate_limit, 0);
    assert_eq!(succeeded, 0);
}

#[test]
fn test_resource_limit_manager_custom() {
    let manager = ResourceLimitManager::new(
        RequestSizeLimits::new(1000, 500, 100, 10),
        ResponseSizeLimits::new(5000, 1000, true),
        PerClientRateLimiter::with_defaults(),
        None,
        ConcurrencyLimits::with_defaults(),
        TimeoutLimits::with_defaults(),
    );

    assert!(manager.is_enabled());
    assert_eq!(manager.request_limits.max_total_size(), 1000);
    assert_eq!(manager.response_limits.max_total_size(), 5000);
}

// ============================================================================
// Advanced Request Size Validation Tests
// ============================================================================

#[test]
fn test_validate_nested_objects() {
    let limits = RequestSizeLimits::new(10000, 1000, 100, 5);

    let mut nested = json!("leaf");
    for _ in 0..10 {
        nested = json!({"nested": nested});
    }

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(nested),
    };

    let result = limits.check_parsed_request(&request);
    assert!(result.is_err());
    match result.unwrap_err() {
        ResourceLimitError::JsonTooDeep { depth, limit } => {
            assert!(depth > limit);
        }
        _ => panic!("Expected JsonTooDeep error"),
    }
}

#[test]
fn test_validate_large_object_keys() {
    let limits = RequestSizeLimits::new(10000, 10, 100, 10);

    let mut obj = serde_json::Map::new();
    obj.insert(
        "very_long_key_that_exceeds_limit".to_string(),
        json!("value"),
    );

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!(obj)),
    };

    let result = limits.check_parsed_request(&request);
    assert!(result.is_err());
}

#[test]
fn test_validate_deeply_nested_arrays() {
    let limits = RequestSizeLimits::new(10000, 1000, 100, 5);

    let mut nested = json!([1]);
    for _ in 0..10 {
        nested = json!([nested]);
    }

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(nested),
    };

    let result = limits.check_parsed_request(&request);
    assert!(result.is_err());
}

// ============================================================================
// Response Size Estimation Tests
// ============================================================================

#[test]
fn test_estimate_response_size_text_content() {
    let limits = ResponseSizeLimits::new(10000, 1000, true);

    let result = CallToolResult {
        content: vec![
            Content::Text {
                text: "short".to_string(),
            },
            Content::Text {
                text: "another short".to_string(),
            },
        ],
        is_error: None,
    };

    let size = limits.estimate_size(&result).unwrap();
    assert!(size > 0);
}

#[test]
fn test_estimate_response_size_resource_content() {
    let limits = ResponseSizeLimits::new(10000, 1000, true);

    let result = CallToolResult {
        content: vec![Content::Resource {
            resource: ResourceContent {
                uri: "file:///test.hedl".to_string(),
                mime_type: Some("text/hedl".to_string()),
                text: Some("content here".to_string()),
            },
        }],
        is_error: None,
    };

    let size = limits.estimate_size(&result).unwrap();
    assert!(size > 0);
}

#[test]
fn test_estimate_response_size_exceeds_limit() {
    let limits = ResponseSizeLimits::new(10, 1000, true);

    let large_text = "x".repeat(1000);
    let result = CallToolResult {
        content: vec![Content::Text { text: large_text }],
        is_error: None,
    };

    let result_size = limits.estimate_size(&result);
    assert!(result_size.is_err());
}

// ============================================================================
// Error Code Tests
// ============================================================================

#[test]
fn test_resource_limit_error_codes_unique() {
    let errors = [
        ResourceLimitError::RequestTooLarge {
            size: 100,
            limit: 10,
            exceeded_by: 90,
        },
        ResourceLimitError::ResponseTooLarge {
            estimated_size: 100,
            limit: 10,
        },
        ResourceLimitError::RateLimitExceeded {
            client_id: "test".to_string(),
            burst: 100,
            rate: 50,
        },
        ResourceLimitError::CacheMemoryExceeded {
            current: 100,
            limit: 50,
            needed: 60,
        },
        ResourceLimitError::GlobalConcurrencyExceeded { limit: 100 },
        ResourceLimitError::OperationTimeout {
            tool_name: "test".to_string(),
            timeout_ms: 5000,
        },
    ];

    let mut codes: Vec<i32> = errors
        .iter()
        .map(|e: &ResourceLimitError| e.error_code())
        .collect();
    codes.sort_unstable();
    codes.dedup();

    // All error codes should be unique
    assert_eq!(codes.len(), errors.len());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_request_and_response_limits_together() {
    let request_limits = RequestSizeLimits::new(1000, 500, 100, 10);
    let response_limits = ResponseSizeLimits::new(5000, 1000, true);

    // Valid request
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "test".to_string(),
        params: Some(json!({"key": "value"})),
    };

    assert!(request_limits.check_parsed_request(&request).is_ok());

    // Valid response
    let result = CallToolResult {
        content: vec![Content::Text {
            text: "result".to_string(),
        }],
        is_error: None,
    };

    assert!(response_limits.estimate_size(&result).is_ok());
}
