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

//! Tests for audit logging functionality in FFI calls.

use hedl_ffi::audit::{
    audit_call_failure, audit_call_start, audit_call_success, get_audit_context, sanitize_bytes,
    sanitize_c_string, sanitize_pointer, sanitize_string, PerformanceMetrics,
};
use hedl_ffi::{hedl_free_document, hedl_parse, HedlDocument, HEDL_OK};
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize tracing for tests.
fn init_tracing() {
    let _ = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_test_writer()
        .try_init();
}

#[test]
fn test_audit_call_lifecycle() {
    init_tracing();

    audit_call_start("test_function", &[("param1", "value1"), ("param2", "42")]);

    // Simulate some work
    std::thread::sleep(Duration::from_millis(1));

    audit_call_success("test_function", Duration::from_millis(1));
}

#[test]
fn test_audit_call_failure_logging() {
    init_tracing();

    audit_call_start("test_function_fail", &[]);

    audit_call_failure(
        "test_function_fail",
        -1,
        "Test error message",
        Duration::from_millis(2),
    );
}

#[test]
fn test_audit_context() {
    init_tracing();

    // Initially no context
    assert!(get_audit_context().is_none());

    audit_call_start("test_context", &[]);

    // Context should be set
    let ctx = get_audit_context();
    assert!(ctx.is_some());
    let ctx = ctx.unwrap();
    assert_eq!(ctx.function, "test_context");
    assert_eq!(ctx.depth, 0);

    audit_call_success("test_context", Duration::from_millis(1));

    // Context should be cleared
    assert!(get_audit_context().is_none());
}

#[test]
fn test_nested_audit_calls() {
    init_tracing();

    audit_call_start("outer_function", &[]);

    // Simulate nested call
    audit_call_start("inner_function", &[]);

    let inner_ctx = get_audit_context().unwrap();
    assert_eq!(inner_ctx.function, "inner_function");
    assert_eq!(inner_ctx.depth, 1);

    audit_call_success("inner_function", Duration::from_millis(1));

    // After inner completes, context is cleared
    assert!(get_audit_context().is_none());
}

#[test]
fn test_sanitize_pointer() {
    let value = 42;
    let ptr = &value as *const i32;

    let sanitized = sanitize_pointer(ptr);
    assert!(sanitized.starts_with("PTR@"));
    assert_eq!(sanitize_pointer(std::ptr::null::<u8>()), "NULL");
}

#[test]
fn test_sanitize_string() {
    assert_eq!(sanitize_string("hello", 10), "\"hello\"");
    assert_eq!(
        sanitize_string("this is a long string", 10),
        "\"this is a \"... (21 bytes total)"
    );
}

#[test]
fn test_sanitize_c_string() {
    unsafe {
        let c_str = CString::new("test string").unwrap();
        let sanitized = sanitize_c_string(c_str.as_ptr(), 20);
        assert_eq!(sanitized, "\"test string\"");

        let long_str = CString::new("this is a very long test string").unwrap();
        let sanitized = sanitize_c_string(long_str.as_ptr(), 10);
        assert!(sanitized.contains("bytes total"));

        // Test null pointer
        let sanitized = sanitize_c_string(ptr::null(), 10);
        assert_eq!(sanitized, "NULL");
    }
}

#[test]
fn test_sanitize_bytes() {
    assert_eq!(sanitize_bytes(&[], 4), "[]");
    assert_eq!(sanitize_bytes(&[0x01, 0x02], 4), "[01, 02]");
    assert_eq!(
        sanitize_bytes(&[0x01, 0x02, 0x03, 0x04, 0x05], 3),
        "[01, 02, 03]... (5 bytes total)"
    );
}

#[test]
fn test_performance_metrics() {
    let mut metrics = PerformanceMetrics::new();

    assert_eq!(metrics.call_count, 0);
    assert_eq!(metrics.success_count, 0);
    assert_eq!(metrics.failure_count, 0);

    metrics.record_success(Duration::from_millis(100));
    assert_eq!(metrics.call_count, 1);
    assert_eq!(metrics.success_count, 1);
    assert_eq!(metrics.min_duration, Some(Duration::from_millis(100)));

    metrics.record_success(Duration::from_millis(50));
    assert_eq!(metrics.call_count, 2);
    assert_eq!(metrics.min_duration, Some(Duration::from_millis(50)));
    assert_eq!(metrics.max_duration, Some(Duration::from_millis(100)));

    metrics.record_failure(Duration::from_millis(200));
    assert_eq!(metrics.call_count, 3);
    assert_eq!(metrics.failure_count, 1);
    assert_eq!(metrics.max_duration, Some(Duration::from_millis(200)));

    assert_eq!(metrics.success_rate(), 66.66666666666666);

    let avg = metrics.avg_duration().unwrap();
    assert!(
        avg >= Duration::from_millis(115) && avg <= Duration::from_millis(120),
        "Average duration should be around 116-117ms, got {:?}",
        avg
    );
}

#[test]
fn test_ffi_function_with_audit_logging() {
    init_tracing();

    unsafe {
        let input = CString::new("%VERSION: 1.0\n---\ntest: value\n").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        hedl_free_document(doc);
    }
}

#[test]
fn test_ffi_error_with_audit_logging() {
    init_tracing();

    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();

        // Null pointer should be logged as error
        let result = hedl_parse(ptr::null(), -1, 0, &mut doc);

        assert_ne!(result, HEDL_OK);
    }
}

#[test]
fn test_multithreaded_audit_logging() {
    init_tracing();

    let threads: Vec<_> = (0..4)
        .map(|i| {
            std::thread::spawn(move || {
                audit_call_start(
                    "thread_function",
                    &[("thread_id", &format!("{}", i))],
                );

                std::thread::sleep(Duration::from_millis(10));

                let ctx = get_audit_context().unwrap();
                assert_eq!(ctx.function, "thread_function");

                audit_call_success("thread_function", Duration::from_millis(10));
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
fn test_performance_metrics_thread_safety() {
    // PerformanceMetrics is not thread-safe by design (must be used per-thread)
    // This test verifies that each thread can have its own metrics

    let threads: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(|| {
                let mut metrics = PerformanceMetrics::new();

                for _ in 0..10 {
                    metrics.record_success(Duration::from_millis(10));
                }

                assert_eq!(metrics.call_count, 10);
                assert_eq!(metrics.success_count, 10);
                assert_eq!(metrics.success_rate(), 100.0);
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
fn test_audit_logging_with_shared_state() {
    init_tracing();

    // Test that audit logging works correctly with Arc-wrapped state
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let threads: Vec<_> = (0..4)
        .map(|_| {
            let counter = Arc::clone(&counter);
            std::thread::spawn(move || {
                audit_call_start("increment", &[]);

                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                audit_call_success("increment", Duration::from_micros(1));
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }

    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 4);
}
