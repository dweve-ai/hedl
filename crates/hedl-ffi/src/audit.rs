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

//! Audit logging for FFI function calls.
//!
//! This module provides comprehensive audit logging for all FFI operations,
//! capturing function calls, parameters, outcomes, and performance metrics.
//!
//! # Log Levels
//!
//! - **DEBUG**: Detailed parameter information (sanitized for security)
//! - **INFO**: Function call entry/exit with basic context
//! - **WARN**: Recoverable errors or unusual conditions
//! - **ERROR**: Function failures with error details
//!
//! # Performance Metrics
//!
//! Each function call is instrumented with timing information to enable
//! performance monitoring and anomaly detection.
//!
//! # Security
//!
//! Sensitive information (raw pointer addresses, full input data) is sanitized
//! or redacted from logs to prevent information leakage.
//!
//! # Examples
//!
//! ```rust,no_run
//! use hedl_ffi::audit::{audit_call_start, audit_call_success, audit_call_failure};
//! use std::time::Instant;
//!
//! unsafe fn my_ffi_function(input: *const i8) -> i32 {
//!     let start = Instant::now();
//!     audit_call_start("my_ffi_function", &[("input_ptr", "sanitized")]);
//!
//!     // ... perform operation ...
//!     let result = 0;
//!
//!     if result == 0 {
//!         audit_call_success("my_ffi_function", start.elapsed());
//!     } else {
//!         audit_call_failure("my_ffi_function", result, "Operation failed", start.elapsed());
//!     }
//!
//!     result
//! }
//! ```

use std::os::raw::{c_char, c_int};
use std::time::Duration;
use tracing::{debug, error, info, warn};

// =============================================================================
// Audit Context
// =============================================================================

/// Thread-local storage for audit context.
///
/// This allows tracking the current FFI call context, including caller
/// information and nested call depth.
#[derive(Debug, Clone)]
pub struct AuditContext {
    /// The current function being called
    pub function: &'static str,
    /// Nested call depth (for tracking recursive FFI calls)
    pub depth: usize,
    /// Thread ID for correlation
    pub thread_id: std::thread::ThreadId,
}

thread_local! {
    static AUDIT_CONTEXT: std::cell::RefCell<Option<AuditContext>> = const { std::cell::RefCell::new(None) };
}

/// Get the current audit context.
///
/// Returns `None` if no FFI call is currently in progress on this thread.
pub fn get_audit_context() -> Option<AuditContext> {
    AUDIT_CONTEXT.with(|ctx| ctx.borrow().clone())
}

/// Set the current audit context.
///
/// This should be called at the start of each FFI function.
fn set_audit_context(context: AuditContext) {
    AUDIT_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(context);
    });
}

/// Clear the current audit context.
///
/// This should be called at the end of each FFI function.
fn clear_audit_context() {
    AUDIT_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    });
}

// =============================================================================
// Parameter Sanitization
// =============================================================================

/// Sanitize a pointer for logging.
///
/// Instead of logging the actual pointer address (which could leak ASLR
/// information), we log a hash or sanitized representation.
pub fn sanitize_pointer<T>(ptr: *const T) -> String {
    if ptr.is_null() {
        "NULL".to_string()
    } else {
        // Don't log actual addresses for security
        format!("PTR@{:016x}", ptr as usize & 0xFFFF)
    }
}

/// Sanitize an input string for logging.
///
/// Limits the length of logged strings to prevent log flooding and
/// potential information leakage.
pub fn sanitize_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:?}", s)
    } else {
        format!("{:?}... ({} bytes total)", &s[..max_len], s.len())
    }
}

/// Sanitize a C string pointer for logging.
///
/// Safely converts a C string to a Rust string and sanitizes it.
pub unsafe fn sanitize_c_string(ptr: *const c_char, max_len: usize) -> String {
    if ptr.is_null() {
        "NULL".to_string()
    } else {
        match std::ffi::CStr::from_ptr(ptr).to_str() {
            Ok(s) => sanitize_string(s, max_len),
            Err(_) => "<invalid UTF-8>".to_string(),
        }
    }
}

/// Sanitize byte data for logging.
///
/// Shows a hex preview of the first few bytes.
pub fn sanitize_bytes(data: &[u8], preview_len: usize) -> String {
    if data.is_empty() {
        "[]".to_string()
    } else if data.len() <= preview_len {
        format!("{:02x?}", data)
    } else {
        format!(
            "{:02x?}... ({} bytes total)",
            &data[..preview_len],
            data.len()
        )
    }
}

// =============================================================================
// Audit Logging Functions
// =============================================================================

/// Log the start of an FFI function call.
///
/// # Arguments
///
/// * `function` - Name of the FFI function being called
/// * `params` - Sanitized parameter key-value pairs for logging
///
/// # Examples
///
/// ```rust,no_run
/// use hedl_ffi::audit::audit_call_start;
///
/// unsafe fn hedl_parse(input: *const i8, len: i32) -> i32 {
///     audit_call_start("hedl_parse", &[
///         ("input_len", &len.to_string()),
///         ("strict", "true"),
///     ]);
///     // ... function implementation ...
///     0
/// }
/// ```
pub fn audit_call_start(function: &'static str, params: &[(&str, &str)]) {
    let thread_id = std::thread::current().id();
    let depth = get_audit_context().map(|ctx| ctx.depth + 1).unwrap_or(0);

    let context = AuditContext {
        function,
        depth,
        thread_id,
    };
    set_audit_context(context.clone());

    info!(
        target: "hedl_ffi::audit",
        function = function,
        thread_id = ?thread_id,
        depth = depth,
        "FFI call started"
    );

    if !params.is_empty() {
        debug!(
            target: "hedl_ffi::audit",
            function = function,
            ?params,
            "FFI call parameters"
        );
    }
}

/// Log successful completion of an FFI function call.
///
/// # Arguments
///
/// * `function` - Name of the FFI function
/// * `duration` - Time taken to execute the function
///
/// # Examples
///
/// ```rust,no_run
/// use hedl_ffi::audit::audit_call_success;
/// use std::time::Instant;
///
/// unsafe fn hedl_parse(input: *const i8, len: i32) -> i32 {
///     let start = Instant::now();
///     // ... function implementation ...
///     audit_call_success("hedl_parse", start.elapsed());
///     0
/// }
/// ```
pub fn audit_call_success(function: &'static str, duration: Duration) {
    let duration_ms = duration.as_secs_f64() * 1000.0;

    info!(
        target: "hedl_ffi::audit",
        function = function,
        duration_ms = duration_ms,
        status = "success",
        "FFI call completed"
    );

    clear_audit_context();
}

/// Log failure of an FFI function call.
///
/// # Arguments
///
/// * `function` - Name of the FFI function
/// * `error_code` - Error code returned by the function
/// * `error_message` - Human-readable error description
/// * `duration` - Time taken before the error occurred
///
/// # Examples
///
/// ```rust,no_run
/// use hedl_ffi::audit::audit_call_failure;
/// use std::time::Instant;
///
/// unsafe fn hedl_parse(input: *const i8, len: i32) -> i32 {
///     let start = Instant::now();
///     // ... function implementation ...
///     let error_code = -1;
///     audit_call_failure("hedl_parse", error_code, "Parse error", start.elapsed());
///     error_code
/// }
/// ```
pub fn audit_call_failure(
    function: &'static str,
    error_code: c_int,
    error_message: &str,
    duration: Duration,
) {
    let duration_ms = duration.as_secs_f64() * 1000.0;

    error!(
        target: "hedl_ffi::audit",
        function = function,
        error_code = error_code,
        error_message = error_message,
        duration_ms = duration_ms,
        status = "failure",
        "FFI call failed"
    );

    clear_audit_context();
}

/// Log a warning during an FFI function call.
///
/// Use this for recoverable errors or unusual conditions that don't
/// cause the function to fail.
///
/// # Arguments
///
/// * `function` - Name of the FFI function
/// * `message` - Warning message
///
/// # Examples
///
/// ```rust,no_run
/// use hedl_ffi::audit::audit_warning;
///
/// unsafe fn hedl_parse(input: *const i8, len: i32) -> i32 {
///     if len > 1_000_000 {
///         audit_warning("hedl_parse", "Input size exceeds recommended limit");
///     }
///     // ... function implementation ...
///     0
/// }
/// ```
pub fn audit_warning(function: &'static str, message: &str) {
    warn!(
        target: "hedl_ffi::audit",
        function = function,
        message = message,
        "FFI call warning"
    );
}

// =============================================================================
// Audit Macros
// =============================================================================

/// Helper macro to wrap an FFI function with audit logging.
///
/// This macro automatically handles timing, success/failure logging,
/// and context management.
///
/// # Examples
///
/// ```text
/// use hedl_ffi::audit_ffi_call;
///
/// #[no_mangle]
/// pub unsafe extern "C" fn hedl_parse(
///     input: *const c_char,
///     len: c_int,
///     out: *mut *mut HedlDocument,
/// ) -> c_int {
///     audit_ffi_call!("hedl_parse", {
///         // Function implementation
///         HEDL_OK
///     })
/// }
/// ```
#[macro_export]
macro_rules! audit_ffi_call {
    ($function:expr, $body:expr) => {{
        use std::time::Instant;
        use $crate::audit::{audit_call_failure, audit_call_start, audit_call_success};

        audit_call_start($function, &[]);
        let start = Instant::now();

        let result = $body;
        let duration = start.elapsed();

        if result == $crate::types::HEDL_OK {
            audit_call_success($function, duration);
        } else {
            let error_msg = $crate::error::get_thread_local_error();
            audit_call_failure($function, result, &error_msg, duration);
        }

        result
    }};
}

// =============================================================================
// Performance Metrics
// =============================================================================

/// Performance metrics collector for FFI calls.
///
/// Tracks call counts, total duration, min/max/avg latencies per function.
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    /// Total number of calls
    pub call_count: u64,
    /// Total duration across all calls
    pub total_duration: Duration,
    /// Minimum call duration
    pub min_duration: Option<Duration>,
    /// Maximum call duration
    pub max_duration: Option<Duration>,
    /// Number of successful calls
    pub success_count: u64,
    /// Number of failed calls
    pub failure_count: u64,
}

impl PerformanceMetrics {
    /// Create a new performance metrics collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful call.
    pub fn record_success(&mut self, duration: Duration) {
        self.call_count += 1;
        self.success_count += 1;
        self.total_duration += duration;
        self.update_duration_bounds(duration);
    }

    /// Record a failed call.
    pub fn record_failure(&mut self, duration: Duration) {
        self.call_count += 1;
        self.failure_count += 1;
        self.total_duration += duration;
        self.update_duration_bounds(duration);
    }

    /// Update min/max duration bounds.
    fn update_duration_bounds(&mut self, duration: Duration) {
        self.min_duration = Some(
            self.min_duration
                .map(|min| min.min(duration))
                .unwrap_or(duration),
        );
        self.max_duration = Some(
            self.max_duration
                .map(|max| max.max(duration))
                .unwrap_or(duration),
        );
    }

    /// Get the average call duration.
    pub fn avg_duration(&self) -> Option<Duration> {
        if self.call_count > 0 {
            Some(self.total_duration / self.call_count as u32)
        } else {
            None
        }
    }

    /// Get the success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        if self.call_count > 0 {
            (self.success_count as f64 / self.call_count as f64) * 100.0
        } else {
            0.0
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_pointer() {
        assert_eq!(sanitize_pointer(std::ptr::null::<u8>()), "NULL");

        let value = 42;
        let ptr = &value as *const i32;
        let sanitized = sanitize_pointer(ptr);
        assert!(sanitized.starts_with("PTR@"));
        assert_ne!(sanitized, format!("PTR@{:016x}", ptr as usize));
    }

    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("hello", 10), "\"hello\"");
        assert_eq!(
            sanitize_string("hello world", 5),
            "\"hello\"... (11 bytes total)"
        );
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
    fn test_audit_context() {
        assert!(get_audit_context().is_none());

        let ctx = AuditContext {
            function: "test_function",
            depth: 0,
            thread_id: std::thread::current().id(),
        };
        set_audit_context(ctx.clone());

        let retrieved = get_audit_context().unwrap();
        assert_eq!(retrieved.function, "test_function");
        assert_eq!(retrieved.depth, 0);

        clear_audit_context();
        assert!(get_audit_context().is_none());
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();
        assert_eq!(metrics.call_count, 0);

        metrics.record_success(Duration::from_millis(100));
        assert_eq!(metrics.call_count, 1);
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.failure_count, 0);

        metrics.record_failure(Duration::from_millis(200));
        assert_eq!(metrics.call_count, 2);
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.failure_count, 1);

        assert_eq!(metrics.min_duration, Some(Duration::from_millis(100)));
        assert_eq!(metrics.max_duration, Some(Duration::from_millis(200)));
        assert_eq!(metrics.avg_duration(), Some(Duration::from_millis(150)));
        assert_eq!(metrics.success_rate(), 50.0);
    }

    #[test]
    fn test_audit_call_lifecycle() {
        // Test that audit functions don't panic
        audit_call_start("test_function", &[("param1", "value1")]);

        let duration = Duration::from_millis(10);
        audit_call_success("test_function", duration);

        audit_call_start("test_function_2", &[]);
        audit_call_failure("test_function_2", -1, "Test error", duration);
    }

    #[test]
    fn test_audit_warning() {
        // Test that warning logging doesn't panic
        audit_warning("test_function", "Test warning message");
    }
}
