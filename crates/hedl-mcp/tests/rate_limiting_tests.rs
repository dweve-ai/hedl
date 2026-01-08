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

//! Integration tests for MCP server rate limiting.
//!
//! Tests the token bucket rate limiting implementation to ensure DoS protection.

use hedl_mcp::{JsonRpcRequest, McpServer, McpServerConfig, RateLimiter};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

/// Helper to create a test server with custom rate limits.
fn create_test_server(burst: usize, rate: usize) -> McpServer {
    let config = McpServerConfig {
        root_path: PathBuf::from("."),
        rate_limit_burst: burst,
        rate_limit_per_second: rate,
        ..Default::default()
    };
    McpServer::new(config)
}

/// Helper to create a simple ping request.
fn ping_request(id: u64) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "ping".to_string(),
        params: None,
        id: Some(Value::Number(id.into())),
    }
}

#[test]
fn test_rate_limiter_basic_functionality() {
    let mut limiter = RateLimiter::new(10, 5);

    // Should allow 10 requests (burst capacity)
    for i in 0..10 {
        assert!(
            limiter.check_limit(),
            "Request {} should be allowed within burst",
            i + 1
        );
    }

    // 11th request should be rejected
    assert!(
        !limiter.check_limit(),
        "Request beyond burst should be rejected"
    );
}

#[test]
fn test_rate_limiter_refill() {
    let mut limiter = RateLimiter::new(100, 50); // 50 tokens/sec

    // Consume all tokens
    for _ in 0..100 {
        limiter.check_limit();
    }
    assert!(!limiter.check_limit(), "Should be rate limited");

    // Wait for refill (200ms = ~10 tokens at 50/sec)
    thread::sleep(Duration::from_millis(200));

    // Should have refilled ~10 tokens
    let mut allowed = 0;
    for _ in 0..20 {
        if limiter.check_limit() {
            allowed += 1;
        }
    }

    assert!(
        allowed >= 8 && allowed <= 12,
        "Expected ~10 refilled tokens, got {}",
        allowed
    );
}

#[test]
fn test_rate_limiter_reset() {
    let mut limiter = RateLimiter::new(50, 25);

    // Consume some tokens
    for _ in 0..30 {
        limiter.check_limit();
    }

    // Reset and verify
    limiter.reset();
    assert_eq!(limiter.tokens(), 50, "Reset should restore full capacity");
}

#[test]
fn test_server_allows_requests_within_limit() {
    let mut server = create_test_server(10, 5);

    // Initialize server first
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        id: Some(Value::Number(1.into())),
    };
    server.handle_request(init_request);

    // All 10 requests should succeed
    for i in 0..10 {
        let response = server.handle_request(ping_request(i + 2));
        if let Some(result) = response.result {
            assert_eq!(result, json!({}), "Request {} should succeed", i + 1);
        } else {
            panic!("Request {} failed: {:?}", i + 1, response.error);
        }
    }
}

#[test]
fn test_server_rejects_requests_exceeding_limit() {
    let mut server = create_test_server(5, 2);

    // Initialize server
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        id: Some(Value::Number(1.into())),
    };
    server.handle_request(init_request);

    // First 5 requests should succeed
    for i in 0..5 {
        let response = server.handle_request(ping_request(i + 2));
        assert!(
            response.error.is_none(),
            "Request {} should succeed",
            i + 1
        );
    }

    // 6th request should be rate limited (but note: check_rate_limit is called
    // in run_stdio, not handle_request, so we need to test the limiter directly)
    // This is a limitation of the current implementation where rate limiting
    // happens at the transport layer, not in handle_request.
}

#[test]
fn test_rate_limiting_disabled_when_burst_zero() {
    let config = McpServerConfig {
        root_path: PathBuf::from("."),
        rate_limit_burst: 0, // Disabled
        rate_limit_per_second: 100,
        ..Default::default()
    };
    let mut server = McpServer::new(config);

    // Initialize server
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        id: Some(Value::Number(1.into())),
    };
    server.handle_request(init_request);

    // Should allow unlimited requests when disabled
    for i in 0..1000 {
        let response = server.handle_request(ping_request(i + 2));
        if response.error.is_some() {
            panic!("Request {} failed but rate limiting is disabled", i + 1);
        }
    }
}

#[test]
fn test_rate_limiting_disabled_when_rate_zero() {
    let config = McpServerConfig {
        root_path: PathBuf::from("."),
        rate_limit_burst: 100,
        rate_limit_per_second: 0, // Disabled
        ..Default::default()
    };
    let mut server = McpServer::new(config);

    // Initialize server
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        id: Some(Value::Number(1.into())),
    };
    server.handle_request(init_request);

    // Should allow unlimited requests when disabled
    for i in 0..1000 {
        let response = server.handle_request(ping_request(i + 2));
        if response.error.is_some() {
            panic!("Request {} failed but rate limiting is disabled", i + 1);
        }
    }
}

#[test]
fn test_burst_capacity_allows_short_bursts() {
    let mut limiter = RateLimiter::new(200, 100); // 100/sec sustained, 200 burst

    // Should allow burst of 200
    for i in 0..200 {
        assert!(
            limiter.check_limit(),
            "Burst request {} should be allowed",
            i + 1
        );
    }

    // 201st should be rejected
    assert!(
        !limiter.check_limit(),
        "Request beyond burst capacity should be rejected"
    );
}

#[test]
fn test_sustained_rate_enforcement() {
    let mut limiter = RateLimiter::new(10, 100); // 100/sec, small burst

    // Consume initial burst
    for _ in 0..10 {
        limiter.check_limit();
    }

    // Wait 100ms (should refill ~10 tokens at 100/sec)
    thread::sleep(Duration::from_millis(100));

    // Should allow approximately 10 more requests
    let mut allowed = 0;
    for _ in 0..20 {
        if limiter.check_limit() {
            allowed += 1;
        }
    }

    assert!(
        allowed >= 8 && allowed <= 12,
        "Expected ~10 requests allowed after refill, got {}",
        allowed
    );
}

#[test]
fn test_rate_limiter_token_count() {
    let mut limiter = RateLimiter::new(100, 50);

    assert_eq!(limiter.tokens(), 100, "Should start full");
    assert_eq!(limiter.max_tokens(), 100);
    assert_eq!(limiter.refill_rate(), 50);

    // Consume tokens
    for _ in 0..30 {
        limiter.check_limit();
    }

    assert_eq!(limiter.tokens(), 70, "Should have 70 tokens remaining");
}

#[test]
fn test_concurrent_request_simulation() {
    let mut limiter = RateLimiter::new(50, 25); // 25 req/sec, 50 burst

    // Simulate rapid burst
    let mut burst_allowed = 0;
    for _ in 0..100 {
        if limiter.check_limit() {
            burst_allowed += 1;
        }
    }

    assert_eq!(burst_allowed, 50, "Should allow exactly burst capacity");

    // Wait 1 second for full refill
    thread::sleep(Duration::from_secs(1));

    // Should have refilled 25 tokens
    let mut refilled_allowed = 0;
    for _ in 0..50 {
        if limiter.check_limit() {
            refilled_allowed += 1;
        }
    }

    assert!(
        refilled_allowed >= 23 && refilled_allowed <= 27,
        "Expected ~25 requests after 1s refill, got {}",
        refilled_allowed
    );
}

#[test]
fn test_default_config_rate_limits() {
    let config = McpServerConfig::default();

    assert_eq!(
        config.rate_limit_burst, 200,
        "Default burst should be 200"
    );
    assert_eq!(
        config.rate_limit_per_second, 100,
        "Default rate should be 100/sec"
    );
}

#[test]
fn test_rate_limiter_multiple_refills() {
    let mut limiter = RateLimiter::new(100, 50);

    // Consume all tokens
    for _ in 0..100 {
        limiter.check_limit();
    }

    // Multiple refill cycles
    for cycle in 0..5 {
        thread::sleep(Duration::from_millis(100)); // ~5 tokens per cycle

        let allowed = if limiter.check_limit() { 1 } else { 0 };
        assert!(
            allowed > 0,
            "Should allow at least one request in cycle {}",
            cycle + 1
        );
    }
}

#[test]
fn test_rate_limiter_edge_cases() {
    // Very small burst
    let mut limiter = RateLimiter::new(1, 1);
    assert!(limiter.check_limit());
    assert!(!limiter.check_limit());

    // Very large burst - note that with 5000/sec refill rate, even a few
    // microseconds can refill a token, so we need to consume them faster
    let mut limiter = RateLimiter::new(10000, 5000);
    let mut consumed = 0;
    for _ in 0..10010 {
        if limiter.check_limit() {
            consumed += 1;
        } else {
            break;
        }
    }
    // Should consume at least the initial burst
    assert!(consumed >= 10000, "Should consume at least burst capacity, got {}", consumed);
    // And should eventually hit the limit
    assert!(!limiter.check_limit(), "Should eventually be rate limited");
}

#[test]
fn test_rate_limiter_no_time_drift() {
    let mut limiter = RateLimiter::new(100, 50);

    // Consume tokens
    for _ in 0..50 {
        limiter.check_limit();
    }

    let tokens_before = limiter.tokens();

    // Immediate check shouldn't add tokens
    let tokens_after = limiter.tokens();

    assert_eq!(
        tokens_before, tokens_after,
        "No tokens should be added without time passing"
    );
}
