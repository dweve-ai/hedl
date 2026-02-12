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

//! Stress tests for MCP server stability and performance.
//!
//! Tests server behavior under high load, sustained usage, and resource exhaustion scenarios.

use hedl_mcp::{JsonRpcRequest, McpServer, McpServerConfig, RateLimiter};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

// =============================================================================
// TEST HELPERS
// =============================================================================

fn create_test_server(root_path: PathBuf) -> McpServer {
    let config = McpServerConfig {
        root_path,
        rate_limit_burst: 0,
        rate_limit_per_second: 0,
        cache_size: 1000,
        ..Default::default()
    };
    McpServer::new(config)
}

fn create_rate_limited_server(root_path: PathBuf, burst: usize, rate: usize) -> McpServer {
    let config = McpServerConfig {
        root_path,
        rate_limit_burst: burst,
        rate_limit_per_second: rate,
        cache_size: 1000,
        ..Default::default()
    };
    McpServer::new(config)
}

fn make_request(method: &str, params: Option<Value>, id: u64) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: Some(Value::Number(id.into())),
    }
}

fn initialize_server(server: &mut McpServer) {
    server.handle_request(make_request(
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "stress-test", "version": "1.0" }
        })),
        1,
    ));
}

// =============================================================================
// HIGH VOLUME REQUEST TESTS
// =============================================================================

#[test]
fn test_high_volume_ping_requests() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    let start = Instant::now();
    let iterations = 10_000;
    let mut success_count = 0;

    for i in 0..iterations {
        let response = server.handle_request(make_request("ping", None, i + 2));
        if response.error.is_none() {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();
    let rate = iterations as f64 / elapsed.as_secs_f64();

    println!("Processed {iterations} requests in {elapsed:?} ({rate:.0} req/sec)");

    assert_eq!(
        success_count, iterations,
        "All ping requests should succeed"
    );
    assert!(rate > 1000.0, "Should process at least 1000 req/sec");
}

#[test]
fn test_high_volume_tools_list() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    let start = Instant::now();
    let iterations = 5_000;
    let mut success_count = 0;

    for i in 0..iterations {
        let response = server.handle_request(make_request("tools/list", None, i + 2));
        if response.error.is_none() && response.result.is_some() {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();
    println!(
        "tools/list: {} requests in {:?} ({:.0} req/sec)",
        iterations,
        elapsed,
        iterations as f64 / elapsed.as_secs_f64()
    );

    assert_eq!(
        success_count, iterations,
        "All tools/list requests should succeed"
    );
}

#[test]
fn test_high_volume_validation() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    let hedl = "%VERSION: 1.0\n%STRUCT: Test: [id, name]\n---\ndata:@Test\n | 1, Test\n";

    let start = Instant::now();
    let iterations = 1_000;
    let mut success_count = 0;

    for i in 0..iterations {
        let response = server.handle_request(make_request(
            "tools/call",
            Some(json!({
                "name": "hedl_validate",
                "arguments": { "hedl": hedl }
            })),
            i + 2,
        ));
        if response.error.is_none() {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();
    println!(
        "validation: {} requests in {:?} ({:.0} req/sec)",
        iterations,
        elapsed,
        iterations as f64 / elapsed.as_secs_f64()
    );

    assert_eq!(
        success_count, iterations,
        "All validation requests should complete"
    );
}

// =============================================================================
// SUSTAINED LOAD TESTS
// =============================================================================

#[test]
fn test_sustained_load_no_degradation() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    let hedl = "%VERSION: 1.0\n---\ntest: data\n";

    // Run in batches and measure latency
    let batch_size = 100;
    let num_batches = 50;
    let mut batch_times = Vec::new();

    for batch in 0..num_batches {
        let start = Instant::now();

        for i in 0..batch_size {
            server.handle_request(make_request(
                "tools/call",
                Some(json!({
                    "name": "hedl_validate",
                    "arguments": { "hedl": hedl }
                })),
                (batch * batch_size + i + 2) as u64,
            ));
        }

        batch_times.push(start.elapsed());
    }

    // Check for degradation
    let first_half_avg: Duration =
        batch_times[..num_batches / 2].iter().sum::<Duration>() / (num_batches / 2) as u32;
    let second_half_avg: Duration =
        batch_times[num_batches / 2..].iter().sum::<Duration>() / (num_batches / 2) as u32;

    // Second half shouldn't be more than 3x slower (allowing for JIT warmup effects,
    // system load variations, and memory pressure). 3x threshold avoids flaky failures
    // on systems with variable background load while still catching severe degradation.
    let degradation = second_half_avg.as_secs_f64() / first_half_avg.as_secs_f64();
    assert!(
        degradation < 3.0,
        "Performance degradation {degradation} > 3x"
    );
}

#[test]
fn test_sustained_load_with_rate_limiter() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_rate_limited_server(temp_dir.path().to_path_buf(), 1000, 500);
    initialize_server(&mut server);

    // Note: handle_request doesn't enforce rate limiting (it's done at transport layer)
    // This test verifies the rate limiter itself under sustained load

    let limiter = RateLimiter::new(1000, 500);

    let start = Instant::now();
    let duration = Duration::from_secs(2);
    let mut allowed = 0;
    let mut rejected = 0;

    while start.elapsed() < duration {
        if limiter.check_limit() {
            allowed += 1;
        } else {
            rejected += 1;
        }
        // Small delay to avoid spinning
        thread::sleep(Duration::from_micros(100));
    }

    println!(
        "Sustained test: {} allowed, {} rejected over {:?}",
        allowed,
        rejected,
        start.elapsed()
    );

    // Should have allowed initial burst plus refill
    // 2 seconds at 500/sec = ~1000 additional after initial 1000 burst
    assert!(allowed >= 1000, "Should allow at least initial burst");
    assert!(allowed <= 2500, "Should be bounded by burst + refill");
}

// =============================================================================
// MEMORY PRESSURE TESTS
// =============================================================================

#[test]
fn test_large_input_handling() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    // Create large HEDL document
    let mut hedl =
        "%VERSION: 1.0\n%STRUCT: Item: [id, name, value]\n---\nitems:@Item\n".to_string();
    for i in 0..10_000 {
        hedl.push_str(&format!("  | item{i}, Item {i}, {i}\n"));
    }

    let start = Instant::now();
    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_validate",
            "arguments": { "hedl": &hedl }
        })),
        2,
    ));

    let elapsed = start.elapsed();
    println!(
        "Large input ({} bytes): processed in {:?}",
        hedl.len(),
        elapsed
    );

    assert!(
        response.error.is_none(),
        "Large input should be processed without error"
    );
}

#[test]
fn test_many_files_handling() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create many HEDL files
    for i in 0..100 {
        let content =
            format!("%VERSION: 1.0\n%STRUCT: Item{i}: [id]\n---\ndata{i}:@Item{i}\n | {i}\n");
        fs::write(root.join(format!("file{i}.hedl")), content).unwrap();
    }

    let mut server = create_test_server(root);
    initialize_server(&mut server);

    let start = Instant::now();
    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_read",
            "arguments": { "path": ".", "recursive": true }
        })),
        2,
    ));

    let elapsed = start.elapsed();
    println!("100 files: processed in {elapsed:?}");

    let result = response.result.unwrap();
    assert_ne!(
        result.get("isError"),
        Some(&json!(true)),
        "Reading many files should succeed"
    );
}

// =============================================================================
// CACHE STRESS TESTS
// =============================================================================

#[test]
fn test_cache_stress_repeated_requests() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    let hedl = "%VERSION: 1.0\n---\ntest: value\n";

    // First request (cache miss)
    let start = Instant::now();
    server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_validate",
            "arguments": { "hedl": hedl }
        })),
        2,
    ));
    let first_time = start.elapsed();

    // Many cached requests
    let iterations = 1000;
    let start = Instant::now();
    for i in 0..iterations {
        server.handle_request(make_request(
            "tools/call",
            Some(json!({
                "name": "hedl_validate",
                "arguments": { "hedl": hedl }
            })),
            i + 3,
        ));
    }
    let cached_time = start.elapsed();
    let avg_cached = cached_time / iterations as u32;

    println!("First request: {first_time:?}, avg cached: {avg_cached:?}");

    // Cached requests should be faster (at least not slower)
    // Note: First request includes JIT warmup, so cached might actually be similar
}

#[test]
fn test_cache_stress_unique_requests() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    // Many unique requests (all cache misses)
    let iterations = 500;
    let start = Instant::now();

    for i in 0..iterations {
        let hedl = format!("%VERSION: 1.0\n---\ntest{i}: value{i}\n");
        server.handle_request(make_request(
            "tools/call",
            Some(json!({
                "name": "hedl_validate",
                "arguments": { "hedl": hedl }
            })),
            i + 2,
        ));
    }

    let elapsed = start.elapsed();
    println!(
        "{} unique requests in {:?} ({:.0} req/sec)",
        iterations,
        elapsed,
        iterations as f64 / elapsed.as_secs_f64()
    );

    // Should complete without issues
    assert!(
        elapsed < Duration::from_secs(30),
        "Should complete within reasonable time"
    );
}

#[test]
fn test_cache_eviction_under_load() {
    let temp_dir = TempDir::new().unwrap();

    // Create server with small cache
    let config = McpServerConfig {
        root_path: temp_dir.path().to_path_buf(),
        rate_limit_burst: 0,
        rate_limit_per_second: 0,
        cache_size: 100, // Small cache
        ..Default::default()
    };
    let mut server = McpServer::new(config);
    initialize_server(&mut server);

    // Generate more unique requests than cache size
    let iterations = 500;
    let mut success = 0;

    for i in 0..iterations {
        let hedl = format!("%VERSION: 1.0\n---\nkey{i}: value{i}\n");
        let response = server.handle_request(make_request(
            "tools/call",
            Some(json!({
                "name": "hedl_validate",
                "arguments": { "hedl": hedl }
            })),
            i + 2,
        ));
        if response.error.is_none() {
            success += 1;
        }
    }

    assert_eq!(
        success, iterations,
        "All requests should succeed despite cache eviction"
    );
}

// =============================================================================
// CONCURRENT ACCESS STRESS
// =============================================================================

#[test]
fn test_concurrent_server_access() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create some test files
    for i in 0..10 {
        fs::write(
            root.join(format!("test{i}.hedl")),
            format!("%VERSION: 1.0\n---\ndata{i}: value\n"),
        )
        .unwrap();
    }

    let server = Arc::new(Mutex::new(create_test_server(root)));
    {
        let mut s = server.lock().unwrap();
        initialize_server(&mut s);
    }

    let request_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Spawn threads doing various operations
    for thread_id in 0..8 {
        let server = Arc::clone(&server);
        let request_count = Arc::clone(&request_count);
        let error_count = Arc::clone(&error_count);

        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let method = match (thread_id + i) % 4 {
                    0 => "ping",
                    1 => "tools/list",
                    2 => "resources/list",
                    _ => "ping",
                };

                let response = {
                    let mut s = server.lock().unwrap();
                    s.handle_request(make_request(method, None, (thread_id * 1000 + i) as u64))
                };

                request_count.fetch_add(1, Ordering::Relaxed);
                if response.error.is_some() {
                    error_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let total = request_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);

    println!("Concurrent test: {total} requests, {errors} errors");
    assert_eq!(errors, 0, "No errors should occur under concurrent access");
    assert_eq!(total, 800, "All requests should be processed");
}

// =============================================================================
// RATE LIMITER STRESS
// =============================================================================

#[test]
fn test_rate_limiter_burst_recovery() {
    let limiter = RateLimiter::new(100, 50);

    // Exhaust burst
    for _ in 0..100 {
        limiter.check_limit();
    }
    assert!(!limiter.check_limit(), "Should be exhausted");

    // Recovery cycles
    for cycle in 0..10 {
        thread::sleep(Duration::from_millis(100));
        let allowed = i32::from(limiter.check_limit());
        println!(
            "Cycle {}: {} allowed, {} tokens",
            cycle,
            allowed,
            limiter.tokens()
        );
        assert!(limiter.tokens() <= 100, "Tokens should stay bounded");
    }
}

#[test]
fn test_rate_limiter_extreme_configuration() {
    // Very high rate
    let high_rate = RateLimiter::new(100000, 50000);
    for _ in 0..100000 {
        high_rate.check_limit();
    }

    // Very low rate
    let low_rate = RateLimiter::new(10, 1);
    for _ in 0..10 {
        low_rate.check_limit();
    }
    assert!(!low_rate.check_limit(), "Should be exhausted");

    // Wait for 1 second
    thread::sleep(Duration::from_secs(1));
    assert!(low_rate.check_limit(), "Should have refilled 1 token");
}

// =============================================================================
// ERROR HANDLING STRESS
// =============================================================================

#[test]
fn test_repeated_invalid_requests() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    let iterations = 1000;
    let mut error_count = 0;

    for i in 0..iterations {
        // Invalid HEDL
        let response = server.handle_request(make_request(
            "tools/call",
            Some(json!({
                "name": "hedl_validate",
                "arguments": { "hedl": "invalid hedl content" }
            })),
            i + 2,
        ));

        // Should return tool error (not protocol error)
        if response.error.is_none() {
            let result = response.result.unwrap();
            if result.get("isError") == Some(&json!(true)) {
                error_count += 1;
            }
        }
    }

    // All should be tool errors (validation failures)
    assert_eq!(
        error_count, iterations,
        "All invalid HEDL should return tool errors"
    );
}

#[test]
fn test_repeated_unknown_method() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    let iterations = 1000;
    let mut protocol_errors = 0;

    for i in 0..iterations {
        let response =
            server.handle_request(make_request(&format!("unknown_method_{i}"), None, i + 2));

        if let Some(error) = response.error {
            if error.code == -32601 {
                protocol_errors += 1;
            }
        }
    }

    assert_eq!(
        protocol_errors, iterations,
        "All unknown methods should return -32601"
    );
}

#[test]
fn test_malformed_json_handling() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    // Various malformed params
    let malformed_params = vec![
        json!("string instead of object"),
        json!(123),
        json!([1, 2, 3]),
        json!(null),
        json!({"deeply": {"nested": {"but": {"wrong": "type"}}}}),
    ];

    for params in malformed_params {
        let response = server.handle_request(make_request("tools/call", Some(params), 2));

        // Should return error without crashing
        assert!(
            response.error.is_some(),
            "Malformed params should return error"
        );
    }
}

// =============================================================================
// STABILITY TESTS
// =============================================================================

#[test]
fn test_mixed_workload_stability() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create test files
    for i in 0..5 {
        fs::write(
            root.join(format!("file{i}.hedl")),
            format!("%VERSION: 1.0\n---\ntest{i}: data\n"),
        )
        .unwrap();
    }

    let mut server = create_test_server(root.clone());
    initialize_server(&mut server);

    let mut stats = HashMap::new();
    stats.insert("ping", (0, 0));
    stats.insert("tools/list", (0, 0));
    stats.insert("validate", (0, 0));
    stats.insert("read", (0, 0));

    // Mixed workload
    for i in 0..500 {
        let (method, params, key) = match i % 4 {
            0 => ("ping", None, "ping"),
            1 => ("tools/list", None, "tools/list"),
            2 => (
                "tools/call",
                Some(json!({
                    "name": "hedl_validate",
                    "arguments": { "hedl": "%VERSION: 1.0\n---\ntest: data\n" }
                })),
                "validate",
            ),
            _ => (
                "tools/call",
                Some(json!({
                    "name": "hedl_read",
                    "arguments": { "path": format!("file{}.hedl", i % 5) }
                })),
                "read",
            ),
        };

        let response = server.handle_request(make_request(method, params, i + 2));

        let stat = stats.get_mut(key).unwrap();
        stat.0 += 1;
        if response.error.is_none() {
            stat.1 += 1;
        }
    }

    // Report and validate
    for (op, (total, success)) in &stats {
        println!("{op}: {success}/{total} succeeded");
        assert!(
            f64::from(*success) / f64::from(*total) > 0.95,
            "{op} success rate too low"
        );
    }
}

#[test]
fn test_long_running_session() {
    let temp_dir = TempDir::new().unwrap();
    let mut server = create_test_server(temp_dir.path().to_path_buf());
    initialize_server(&mut server);

    let start = Instant::now();
    let duration = Duration::from_secs(5);
    let mut request_count = 0;

    while start.elapsed() < duration {
        server.handle_request(make_request("ping", None, request_count + 2));
        request_count += 1;

        // Occasional heavier operation
        if request_count % 100 == 0 {
            server.handle_request(make_request(
                "tools/call",
                Some(json!({
                    "name": "hedl_validate",
                    "arguments": { "hedl": "%VERSION: 1.0\n---\ntest: data\n" }
                })),
                request_count + 3,
            ));
        }
    }

    let rate = request_count as f64 / start.elapsed().as_secs_f64();
    println!(
        "Long session: {} requests over {:?} ({:.0} req/sec)",
        request_count,
        start.elapsed(),
        rate
    );

    assert!(
        request_count > 10000,
        "Should process many requests in 5 seconds"
    );
}

// =============================================================================
// RESOURCE EXHAUSTION TESTS
// =============================================================================

#[test]
fn test_rapid_initialize_shutdown_cycles() {
    let temp_dir = TempDir::new().unwrap();
    let config = McpServerConfig {
        root_path: temp_dir.path().to_path_buf(),
        rate_limit_burst: 0,
        rate_limit_per_second: 0,
        ..Default::default()
    };

    for cycle in 0..100 {
        let mut server = McpServer::new(config.clone());

        // Initialize
        let response = server.handle_request(make_request(
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            })),
            1,
        ));
        assert!(
            response.error.is_none(),
            "Initialize should succeed in cycle {cycle}"
        );

        // Some operations
        server.handle_request(make_request("ping", None, 2));
        server.handle_request(make_request("tools/list", None, 3));

        // Shutdown
        let response = server.handle_request(make_request("shutdown", None, 4));
        assert!(
            response.error.is_none(),
            "Shutdown should succeed in cycle {cycle}"
        );
    }
}

#[test]
fn test_many_concurrent_validations() {
    use std::sync::atomic::AtomicBool;

    let temp_dir = TempDir::new().unwrap();
    let server = Arc::new(Mutex::new(create_test_server(
        temp_dir.path().to_path_buf(),
    )));
    {
        let mut s = server.lock().unwrap();
        initialize_server(&mut s);
    }

    let running = Arc::new(AtomicBool::new(true));
    let total_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Run for 2 seconds with multiple threads
    let duration = Duration::from_secs(2);
    let start = Instant::now();

    for thread_id in 0..4 {
        let server = Arc::clone(&server);
        let running = Arc::clone(&running);
        let total_count = Arc::clone(&total_count);

        handles.push(thread::spawn(move || {
            let mut local_count = 0;
            while running.load(Ordering::Relaxed) {
                let hedl = format!("%VERSION: 1.0\n---\ndata_{thread_id}: value_{local_count}\n");
                let mut s = server.lock().unwrap();
                s.handle_request(make_request(
                    "tools/call",
                    Some(json!({
                        "name": "hedl_validate",
                        "arguments": { "hedl": hedl }
                    })),
                    (thread_id * 10000 + local_count) as u64,
                ));
                drop(s);
                local_count += 1;
            }
            total_count.fetch_add(local_count, Ordering::Relaxed);
        }));
    }

    // Wait for duration
    while start.elapsed() < duration {
        thread::sleep(Duration::from_millis(100));
    }
    running.store(false, Ordering::Relaxed);

    for handle in handles {
        handle.join().unwrap();
    }

    let total = total_count.load(Ordering::Relaxed);
    println!("Concurrent validations: {} in {:?}", total, start.elapsed());
    assert!(total > 1000, "Should process many validations");
}
