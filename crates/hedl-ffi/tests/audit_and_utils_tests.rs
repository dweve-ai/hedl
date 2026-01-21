// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tests for audit logging and utility functions.

use hedl_ffi::audit::*;
use std::time::Duration;

#[test]
fn test_sanitize_pointer_null() {
    let result = sanitize_pointer(std::ptr::null::<u8>());
    assert_eq!(result, "NULL");
}

#[test]
fn test_sanitize_pointer_non_null() {
    let value = 42;
    let ptr = std::ptr::addr_of!(value);
    let result = sanitize_pointer(ptr);

    assert!(result.starts_with("PTR@"));
    assert_ne!(result, "NULL");
}

#[test]
fn test_sanitize_string_short() {
    let result = sanitize_string("hello", 10);
    assert_eq!(result, "\"hello\"");
}

#[test]
fn test_sanitize_string_long() {
    let long_str = "a".repeat(100);
    let result = sanitize_string(&long_str, 10);

    assert!(result.contains("..."));
    assert!(result.contains("100 bytes total"));
}

#[test]
fn test_sanitize_string_exact_max() {
    let result = sanitize_string("hello", 5);
    assert_eq!(result, "\"hello\"");
}

#[test]
fn test_sanitize_string_empty() {
    let result = sanitize_string("", 10);
    assert_eq!(result, "\"\"");
}

#[test]
fn test_sanitize_bytes_empty() {
    let result = sanitize_bytes(&[], 4);
    assert_eq!(result, "[]");
}

#[test]
fn test_sanitize_bytes_short() {
    let result = sanitize_bytes(&[0x01, 0x02], 4);
    assert_eq!(result, "[01, 02]");
}

#[test]
fn test_sanitize_bytes_long() {
    let bytes = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let result = sanitize_bytes(&bytes, 3);

    assert!(result.contains("..."));
    assert!(result.contains("5 bytes total"));
}

#[test]
fn test_performance_metrics_new() {
    let metrics = PerformanceMetrics::new();

    assert_eq!(metrics.call_count, 0);
    assert_eq!(metrics.success_count, 0);
    assert_eq!(metrics.failure_count, 0);
    assert_eq!(metrics.total_duration, Duration::ZERO);
    assert!(metrics.min_duration.is_none());
    assert!(metrics.max_duration.is_none());
}

#[test]
fn test_performance_metrics_record_success() {
    let mut metrics = PerformanceMetrics::new();
    let duration = Duration::from_millis(100);

    metrics.record_success(duration);

    assert_eq!(metrics.call_count, 1);
    assert_eq!(metrics.success_count, 1);
    assert_eq!(metrics.failure_count, 0);
    assert_eq!(metrics.total_duration, duration);
    assert_eq!(metrics.min_duration, Some(duration));
    assert_eq!(metrics.max_duration, Some(duration));
}

#[test]
fn test_performance_metrics_record_failure() {
    let mut metrics = PerformanceMetrics::new();
    let duration = Duration::from_millis(50);

    metrics.record_failure(duration);

    assert_eq!(metrics.call_count, 1);
    assert_eq!(metrics.success_count, 0);
    assert_eq!(metrics.failure_count, 1);
}

#[test]
fn test_performance_metrics_multiple_calls() {
    let mut metrics = PerformanceMetrics::new();

    metrics.record_success(Duration::from_millis(100));
    metrics.record_success(Duration::from_millis(200));
    metrics.record_failure(Duration::from_millis(50));

    assert_eq!(metrics.call_count, 3);
    assert_eq!(metrics.success_count, 2);
    assert_eq!(metrics.failure_count, 1);
    assert_eq!(metrics.total_duration, Duration::from_millis(350));
    assert_eq!(metrics.min_duration, Some(Duration::from_millis(50)));
    assert_eq!(metrics.max_duration, Some(Duration::from_millis(200)));
}

#[test]
fn test_performance_metrics_avg_duration() {
    let mut metrics = PerformanceMetrics::new();

    metrics.record_success(Duration::from_millis(100));
    metrics.record_success(Duration::from_millis(200));

    let avg = metrics.avg_duration().unwrap();
    assert_eq!(avg, Duration::from_millis(150));
}

#[test]
fn test_performance_metrics_avg_duration_empty() {
    let metrics = PerformanceMetrics::new();
    assert!(metrics.avg_duration().is_none());
}

#[test]
fn test_performance_metrics_success_rate() {
    let mut metrics = PerformanceMetrics::new();

    metrics.record_success(Duration::from_millis(1));
    metrics.record_success(Duration::from_millis(1));
    metrics.record_failure(Duration::from_millis(1));
    metrics.record_failure(Duration::from_millis(1));

    assert_eq!(metrics.success_rate(), 50.0);
}

#[test]
fn test_performance_metrics_success_rate_all_success() {
    let mut metrics = PerformanceMetrics::new();

    metrics.record_success(Duration::from_millis(1));
    metrics.record_success(Duration::from_millis(1));

    assert_eq!(metrics.success_rate(), 100.0);
}

#[test]
fn test_performance_metrics_success_rate_all_failure() {
    let mut metrics = PerformanceMetrics::new();

    metrics.record_failure(Duration::from_millis(1));
    metrics.record_failure(Duration::from_millis(1));

    assert_eq!(metrics.success_rate(), 0.0);
}

#[test]
fn test_performance_metrics_success_rate_empty() {
    let metrics = PerformanceMetrics::new();
    assert_eq!(metrics.success_rate(), 0.0);
}

#[test]
fn test_audit_context_isolation() {
    // Context should start as None
    assert!(get_audit_context().is_none());

    // After audit_call_start, context should be set
    audit_call_start("test_function", &[]);
    let ctx = get_audit_context();
    assert!(ctx.is_some());

    let ctx = ctx.unwrap();
    assert_eq!(ctx.function, "test_function");
    assert_eq!(ctx.depth, 0);
}

#[test]
fn test_audit_call_lifecycle() {
    // Start
    audit_call_start("test_fn", &[("param", "value")]);

    // Context should be set
    assert!(get_audit_context().is_some());

    // Success should clear context
    audit_call_success("test_fn", Duration::from_millis(10));

    // Context should be cleared after success
    assert!(get_audit_context().is_none());
}

#[test]
fn test_audit_call_failure_lifecycle() {
    // Start
    audit_call_start("test_fn", &[]);

    // Failure should clear context
    audit_call_failure("test_fn", -1, "error", Duration::from_millis(5));

    // Context should be cleared
    assert!(get_audit_context().is_none());
}

#[test]
fn test_audit_warning() {
    // Should not crash
    audit_warning("test_fn", "This is a warning");
}

#[test]
fn test_multiple_audit_calls() {
    for i in 0..10 {
        audit_call_start("test_fn", &[("iteration", &i.to_string())]);
        audit_call_success("test_fn", Duration::from_micros(100));
    }
}

#[test]
fn test_nested_audit_depth() {
    audit_call_start("outer", &[]);

    let outer_ctx = get_audit_context().unwrap();
    assert_eq!(outer_ctx.depth, 0);

    audit_call_success("outer", Duration::from_millis(1));
}

#[test]
fn test_sanitize_c_string_null() {
    unsafe {
        let result = sanitize_c_string(std::ptr::null(), 10);
        assert_eq!(result, "NULL");
    }
}

#[test]
fn test_sanitize_c_string_valid() {
    unsafe {
        let c_str = std::ffi::CString::new("hello").unwrap();
        let result = sanitize_c_string(c_str.as_ptr(), 10);
        assert_eq!(result, "\"hello\"");
    }
}

#[test]
fn test_sanitize_c_string_long() {
    unsafe {
        let long = "a".repeat(100);
        let c_str = std::ffi::CString::new(long).unwrap();
        let result = sanitize_c_string(c_str.as_ptr(), 10);

        assert!(result.contains("..."));
    }
}

#[test]
fn test_sanitize_c_string_invalid_utf8() {
    unsafe {
        // Create invalid UTF-8
        let bytes = [0xFF, 0xFE, 0x00];
        let result = sanitize_c_string(bytes.as_ptr().cast::<i8>(), 10);
        assert_eq!(result, "<invalid UTF-8>");
    }
}

#[test]
fn test_performance_metrics_min_max_tracking() {
    let mut metrics = PerformanceMetrics::new();

    metrics.record_success(Duration::from_millis(50));
    assert_eq!(metrics.min_duration, Some(Duration::from_millis(50)));
    assert_eq!(metrics.max_duration, Some(Duration::from_millis(50)));

    metrics.record_success(Duration::from_millis(30));
    assert_eq!(metrics.min_duration, Some(Duration::from_millis(30)));
    assert_eq!(metrics.max_duration, Some(Duration::from_millis(50)));

    metrics.record_success(Duration::from_millis(100));
    assert_eq!(metrics.min_duration, Some(Duration::from_millis(30)));
    assert_eq!(metrics.max_duration, Some(Duration::from_millis(100)));
}

#[test]
fn test_performance_metrics_duration_accumulation() {
    let mut metrics = PerformanceMetrics::new();

    metrics.record_success(Duration::from_millis(10));
    assert_eq!(metrics.total_duration, Duration::from_millis(10));

    metrics.record_success(Duration::from_millis(20));
    assert_eq!(metrics.total_duration, Duration::from_millis(30));

    metrics.record_failure(Duration::from_millis(5));
    assert_eq!(metrics.total_duration, Duration::from_millis(35));
}

#[test]
fn test_audit_call_with_empty_params() {
    audit_call_start("test", &[]);
    audit_call_success("test", Duration::from_micros(1));
}

#[test]
fn test_audit_call_with_multiple_params() {
    audit_call_start(
        "test",
        &[
            ("param1", "value1"),
            ("param2", "value2"),
            ("param3", "value3"),
        ],
    );
    audit_call_success("test", Duration::from_micros(1));
}

#[test]
fn test_sanitize_bytes_single_byte() {
    let result = sanitize_bytes(&[0xAB], 4);
    assert_eq!(result, "[ab]");
}

#[test]
fn test_sanitize_string_unicode() {
    let result = sanitize_string("hello 世界", 20);
    assert!(result.contains("hello"));
    assert!(result.contains("世界"));
}

#[test]
fn test_performance_metrics_with_zero_duration() {
    let mut metrics = PerformanceMetrics::new();
    metrics.record_success(Duration::ZERO);

    assert_eq!(metrics.call_count, 1);
    assert_eq!(metrics.min_duration, Some(Duration::ZERO));
    assert_eq!(metrics.max_duration, Some(Duration::ZERO));
}

#[test]
fn test_sanitize_pointer_consistency() {
    let value = 42;
    let ptr = std::ptr::addr_of!(value);

    let result1 = sanitize_pointer(ptr);
    let result2 = sanitize_pointer(ptr);

    // Same pointer should produce same sanitized output
    assert_eq!(result1, result2);
}

#[test]
fn test_sanitize_different_pointers() {
    let value1 = 42;
    let value2 = 43;
    let ptr1 = std::ptr::addr_of!(value1);
    let ptr2 = std::ptr::addr_of!(value2);

    let result1 = sanitize_pointer(ptr1);
    let result2 = sanitize_pointer(ptr2);

    // Different pointers may or may not produce different sanitized output
    // (depends on address masking) but both should be valid
    assert!(result1.starts_with("PTR@"));
    assert!(result2.starts_with("PTR@"));
}
