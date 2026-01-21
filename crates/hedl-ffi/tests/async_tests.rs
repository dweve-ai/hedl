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

#![cfg(feature = "async-ffi")]

//! Async operation tests.
//!
//! Tests the async FFI API for parsing, conversion, and other operations.

// Allow Arc with non-Send/Sync for FFI test callbacks
#![allow(clippy::arc_with_non_send_sync)]

use hedl_ffi::*;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const VALID_HEDL: &[u8] = b"%VERSION: 1.0\n---\nkey: value\0";
const INVALID_HEDL: &[u8] = b"invalid hedl syntax\0";

// =============================================================================
// Helper Structures
// =============================================================================

/// Test context for callbacks.
struct TestContext {
    completed: Arc<AtomicBool>,
    status: Arc<AtomicI32>,
    result: Arc<Mutex<*mut c_void>>,
    error_msg: Arc<Mutex<String>>,
}

impl TestContext {
    fn new() -> Self {
        TestContext {
            completed: Arc::new(AtomicBool::new(false)),
            status: Arc::new(AtomicI32::new(0)),
            result: Arc::new(Mutex::new(std::ptr::null_mut())),
            error_msg: Arc::new(Mutex::new(String::new())),
        }
    }

    fn wait_for_completion(&self, timeout_ms: u64) -> bool {
        let start = std::time::Instant::now();
        while !self.completed.load(Ordering::Acquire) {
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                return false;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        true
    }
}

// =============================================================================
// Async Parse Tests
// =============================================================================

#[test]
fn test_async_parse_success() {
    unsafe {
        let ctx = TestContext::new();
        let ctx_ptr = std::ptr::addr_of!(ctx) as *mut c_void;

        extern "C" fn callback(
            status: c_int,
            result: *mut c_void,
            error: *const c_char,
            user_data: *mut c_void,
        ) {
            let ctx = unsafe { &*(user_data as *const TestContext) };
            ctx.status.store(status, Ordering::Release);

            if !result.is_null() {
                *ctx.result.lock().unwrap() = result;
            }

            if !error.is_null() {
                let error_str = unsafe { CStr::from_ptr(error).to_string_lossy().into_owned() };
                *ctx.error_msg.lock().unwrap() = error_str;
            }

            ctx.completed.store(true, Ordering::Release);
        }

        let op = hedl_parse_async(
            VALID_HEDL.as_ptr().cast::<c_char>(),
            -1,
            0,
            Some(callback),
            ctx_ptr,
        );

        assert!(!op.is_null(), "Operation submission should succeed");

        // Wait for completion
        assert!(
            ctx.wait_for_completion(5000),
            "Operation should complete within timeout"
        );

        // Verify success
        assert_eq!(ctx.status.load(Ordering::Acquire), HEDL_OK);

        let doc_ptr = (*ctx.result.lock().unwrap()).cast::<HedlDocument>();
        assert!(!doc_ptr.is_null(), "Document should not be null");

        // Clean up
        hedl_free_document(doc_ptr);
        hedl_async_free(op);
    }
}

#[test]
fn test_async_parse_failure() {
    unsafe {
        let ctx = TestContext::new();
        let ctx_ptr = std::ptr::addr_of!(ctx) as *mut c_void;

        extern "C" fn callback(
            status: c_int,
            result: *mut c_void,
            error: *const c_char,
            user_data: *mut c_void,
        ) {
            let ctx = unsafe { &*(user_data as *const TestContext) };
            ctx.status.store(status, Ordering::Release);

            if !result.is_null() {
                *ctx.result.lock().unwrap() = result;
            }

            if !error.is_null() {
                let error_str = unsafe { CStr::from_ptr(error).to_string_lossy().into_owned() };
                *ctx.error_msg.lock().unwrap() = error_str;
            }

            ctx.completed.store(true, Ordering::Release);
        }

        let op = hedl_parse_async(
            INVALID_HEDL.as_ptr().cast::<c_char>(),
            -1,
            0,
            Some(callback),
            ctx_ptr,
        );

        assert!(!op.is_null(), "Operation submission should succeed");

        // Wait for completion
        assert!(
            ctx.wait_for_completion(5000),
            "Operation should complete within timeout"
        );

        // Verify failure
        assert_eq!(ctx.status.load(Ordering::Acquire), HEDL_ERR_PARSE);

        let doc_ptr = *ctx.result.lock().unwrap();
        assert!(doc_ptr.is_null(), "Document should be null on error");

        let error_msg = ctx.error_msg.lock().unwrap();
        assert!(!error_msg.is_empty(), "Error message should not be empty");
        assert!(
            error_msg.contains("Parse error"),
            "Error message should indicate parse error"
        );

        // Clean up
        hedl_async_free(op);
    }
}

#[test]
fn test_async_parse_null_input() {
    unsafe {
        let ctx = TestContext::new();
        let ctx_ptr = std::ptr::addr_of!(ctx) as *mut c_void;

        extern "C" fn callback(
            _status: c_int,
            _result: *mut c_void,
            _error: *const c_char,
            _user_data: *mut c_void,
        ) {
            panic!("Callback should not be invoked for null input");
        }

        let op = hedl_parse_async(std::ptr::null(), -1, 0, Some(callback), ctx_ptr);

        // Should fail immediately
        assert!(op.is_null(), "Operation should fail with null input");

        let error = hedl_get_last_error();
        assert!(!error.is_null());
        let error_str = CStr::from_ptr(error).to_str().unwrap();
        assert!(
            error_str.contains("Null"),
            "Error should mention null input"
        );
    }
}

#[test]
fn test_async_parse_cancellation() {
    unsafe {
        let ctx = TestContext::new();
        let ctx_ptr = std::ptr::addr_of!(ctx) as *mut c_void;

        extern "C" fn callback(
            status: c_int,
            result: *mut c_void,
            error: *const c_char,
            user_data: *mut c_void,
        ) {
            let ctx = unsafe { &*(user_data as *const TestContext) };
            ctx.status.store(status, Ordering::Release);

            if !result.is_null() {
                *ctx.result.lock().unwrap() = result;
            }

            if !error.is_null() {
                let error_str = unsafe { CStr::from_ptr(error).to_string_lossy().into_owned() };
                *ctx.error_msg.lock().unwrap() = error_str;
            }

            ctx.completed.store(true, Ordering::Release);
        }

        let op = hedl_parse_async(
            VALID_HEDL.as_ptr().cast::<c_char>(),
            -1,
            0,
            Some(callback),
            ctx_ptr,
        );

        // Cancel immediately
        hedl_async_cancel(op);

        // Wait for callback
        assert!(
            ctx.wait_for_completion(5000),
            "Cancelled operation should still invoke callback"
        );

        // Verify cancellation status or successful completion
        // Note: For very fast operations, the parse may complete before cancellation takes effect.
        // This is expected behavior - cancellation is best-effort.
        let status = ctx.status.load(Ordering::Acquire);
        assert!(
            status == HEDL_ERR_CANCELLED || status == HEDL_OK,
            "Status should be either HEDL_ERR_CANCELLED or HEDL_OK, got {status}"
        );

        // If cancelled, document should be null. If completed, document may be present.
        if status == HEDL_ERR_CANCELLED {
            let doc_ptr = *ctx.result.lock().unwrap();
            assert!(
                doc_ptr.is_null(),
                "Document should be null for cancelled operation"
            );
        } else {
            // Clean up document if parse succeeded
            let doc_ptr = (*ctx.result.lock().unwrap()).cast::<HedlDocument>();
            if !doc_ptr.is_null() {
                hedl_free_document(doc_ptr);
            }
        }

        // Clean up
        hedl_async_free(op);
    }
}

// =============================================================================
// Async Canonicalize Tests
// =============================================================================

#[test]
fn test_async_canonicalize_success() {
    unsafe {
        // First parse a document
        let mut doc: *mut HedlDocument = std::ptr::null_mut();
        let result = hedl_parse(VALID_HEDL.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);

        let ctx = TestContext::new();
        let ctx_ptr = std::ptr::addr_of!(ctx) as *mut c_void;

        extern "C" fn callback(
            status: c_int,
            result: *mut c_void,
            error: *const c_char,
            user_data: *mut c_void,
        ) {
            let ctx = unsafe { &*(user_data as *const TestContext) };
            ctx.status.store(status, Ordering::Release);

            if !result.is_null() {
                *ctx.result.lock().unwrap() = result;
            }

            if !error.is_null() {
                let error_str = unsafe { CStr::from_ptr(error).to_string_lossy().into_owned() };
                *ctx.error_msg.lock().unwrap() = error_str;
            }

            ctx.completed.store(true, Ordering::Release);
        }

        let op = hedl_canonicalize_async(doc, Some(callback), ctx_ptr);
        assert!(!op.is_null(), "Canonicalize operation should submit");

        // Wait for completion
        assert!(
            ctx.wait_for_completion(5000),
            "Operation should complete within timeout"
        );

        // Verify success
        assert_eq!(ctx.status.load(Ordering::Acquire), HEDL_OK);

        let canonical_ptr = (*ctx.result.lock().unwrap()).cast::<c_char>();
        assert!(
            !canonical_ptr.is_null(),
            "Canonical string should not be null"
        );

        let canonical_str = CStr::from_ptr(canonical_ptr).to_str().unwrap();
        assert!(
            !canonical_str.is_empty(),
            "Canonical output should not be empty"
        );

        // Clean up
        hedl_free_string(canonical_ptr);
        hedl_async_free(op);
        hedl_free_document(doc);
    }
}

#[test]
fn test_async_canonicalize_null_document() {
    unsafe {
        let ctx = TestContext::new();
        let ctx_ptr = std::ptr::addr_of!(ctx) as *mut c_void;

        extern "C" fn callback(
            _status: c_int,
            _result: *mut c_void,
            _error: *const c_char,
            _user_data: *mut c_void,
        ) {
            panic!("Callback should not be invoked for null document");
        }

        let op = hedl_canonicalize_async(std::ptr::null(), Some(callback), ctx_ptr);

        // Should fail immediately
        assert!(op.is_null(), "Operation should fail with null document");

        let error = hedl_get_last_error();
        assert!(!error.is_null());
        let error_str = CStr::from_ptr(error).to_str().unwrap();
        assert!(
            error_str.contains("Invalid"),
            "Error should mention invalid document"
        );
    }
}

// =============================================================================
// Async Lint Tests
// =============================================================================

#[test]
fn test_async_lint_success() {
    unsafe {
        // First parse a document
        let mut doc: *mut HedlDocument = std::ptr::null_mut();
        let result = hedl_parse(VALID_HEDL.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);

        let ctx = TestContext::new();
        let ctx_ptr = std::ptr::addr_of!(ctx) as *mut c_void;

        extern "C" fn callback(
            status: c_int,
            result: *mut c_void,
            error: *const c_char,
            user_data: *mut c_void,
        ) {
            let ctx = unsafe { &*(user_data as *const TestContext) };
            ctx.status.store(status, Ordering::Release);

            if !result.is_null() {
                *ctx.result.lock().unwrap() = result;
            }

            if !error.is_null() {
                let error_str = unsafe { CStr::from_ptr(error).to_string_lossy().into_owned() };
                *ctx.error_msg.lock().unwrap() = error_str;
            }

            ctx.completed.store(true, Ordering::Release);
        }

        let op = hedl_lint_async(doc, Some(callback), ctx_ptr);
        assert!(!op.is_null(), "Lint operation should submit");

        // Wait for completion
        assert!(
            ctx.wait_for_completion(5000),
            "Operation should complete within timeout"
        );

        // Verify success
        assert_eq!(ctx.status.load(Ordering::Acquire), HEDL_OK);

        let diag_ptr = (*ctx.result.lock().unwrap()).cast::<HedlDiagnostics>();
        assert!(!diag_ptr.is_null(), "Diagnostics should not be null");

        let count = hedl_diagnostics_count(diag_ptr);
        assert!(count >= 0, "Diagnostics count should be non-negative");

        // Clean up
        hedl_free_diagnostics(diag_ptr);
        hedl_async_free(op);
        hedl_free_document(doc);
    }
}

// =============================================================================
// Async Conversion Tests
// =============================================================================

#[cfg(feature = "json")]
#[test]
fn test_async_to_json_success() {
    unsafe {
        // First parse a document
        let mut doc: *mut HedlDocument = std::ptr::null_mut();
        let result = hedl_parse(VALID_HEDL.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);

        let ctx = TestContext::new();
        let ctx_ptr = std::ptr::addr_of!(ctx) as *mut c_void;

        extern "C" fn callback(
            status: c_int,
            result: *mut c_void,
            error: *const c_char,
            user_data: *mut c_void,
        ) {
            let ctx = unsafe { &*(user_data as *const TestContext) };
            ctx.status.store(status, Ordering::Release);

            if !result.is_null() {
                *ctx.result.lock().unwrap() = result;
            }

            if !error.is_null() {
                let error_str = unsafe { CStr::from_ptr(error).to_string_lossy().into_owned() };
                *ctx.error_msg.lock().unwrap() = error_str;
            }

            ctx.completed.store(true, Ordering::Release);
        }

        let op = hedl_to_json_async(doc, 0, Some(callback), ctx_ptr);
        assert!(!op.is_null(), "JSON conversion operation should submit");

        // Wait for completion
        assert!(
            ctx.wait_for_completion(5000),
            "Operation should complete within timeout"
        );

        // Verify success
        assert_eq!(ctx.status.load(Ordering::Acquire), HEDL_OK);

        let json_ptr = (*ctx.result.lock().unwrap()).cast::<c_char>();
        assert!(!json_ptr.is_null(), "JSON string should not be null");

        let json_str = CStr::from_ptr(json_ptr).to_str().unwrap();
        assert!(!json_str.is_empty(), "JSON output should not be empty");
        assert!(
            json_str.contains("key"),
            "JSON should contain the key field"
        );

        // Clean up
        hedl_free_string(json_ptr);
        hedl_async_free(op);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_async_to_yaml_success() {
    unsafe {
        // First parse a document
        let mut doc: *mut HedlDocument = std::ptr::null_mut();
        let result = hedl_parse(VALID_HEDL.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);

        let ctx = TestContext::new();
        let ctx_ptr = std::ptr::addr_of!(ctx) as *mut c_void;

        extern "C" fn callback(
            status: c_int,
            result: *mut c_void,
            error: *const c_char,
            user_data: *mut c_void,
        ) {
            let ctx = unsafe { &*(user_data as *const TestContext) };
            ctx.status.store(status, Ordering::Release);

            if !result.is_null() {
                *ctx.result.lock().unwrap() = result;
            }

            if !error.is_null() {
                let error_str = unsafe { CStr::from_ptr(error).to_string_lossy().into_owned() };
                *ctx.error_msg.lock().unwrap() = error_str;
            }

            ctx.completed.store(true, Ordering::Release);
        }

        let op = hedl_to_yaml_async(doc, 0, Some(callback), ctx_ptr);
        assert!(!op.is_null(), "YAML conversion operation should submit");

        // Wait for completion
        assert!(
            ctx.wait_for_completion(5000),
            "Operation should complete within timeout"
        );

        // Verify success
        assert_eq!(ctx.status.load(Ordering::Acquire), HEDL_OK);

        let yaml_ptr = (*ctx.result.lock().unwrap()).cast::<c_char>();
        assert!(!yaml_ptr.is_null(), "YAML string should not be null");

        let yaml_str = CStr::from_ptr(yaml_ptr).to_str().unwrap();
        assert!(!yaml_str.is_empty(), "YAML output should not be empty");

        // Clean up
        hedl_free_string(yaml_ptr);
        hedl_async_free(op);
        hedl_free_document(doc);
    }
}

// =============================================================================
// Concurrent Operations Test
// =============================================================================

#[test]
fn test_concurrent_async_operations() {
    use std::thread;

    let num_threads = 10;
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let handle = thread::spawn(move || unsafe {
            let ctx = TestContext::new();
            let ctx_ptr = std::ptr::addr_of!(ctx) as *mut c_void;

            extern "C" fn callback(
                status: c_int,
                result: *mut c_void,
                _error: *const c_char,
                user_data: *mut c_void,
            ) {
                let ctx = unsafe { &*(user_data as *const TestContext) };
                ctx.status.store(status, Ordering::Release);

                if !result.is_null() {
                    *ctx.result.lock().unwrap() = result;
                }

                ctx.completed.store(true, Ordering::Release);
            }

            let op = hedl_parse_async(
                VALID_HEDL.as_ptr().cast::<c_char>(),
                -1,
                0,
                Some(callback),
                ctx_ptr,
            );

            assert!(
                !op.is_null(),
                "Thread {thread_id} should submit operation successfully"
            );

            // Wait for completion
            assert!(
                ctx.wait_for_completion(10000),
                "Thread {thread_id} operation should complete"
            );

            // Verify success
            assert_eq!(
                ctx.status.load(Ordering::Acquire),
                HEDL_OK,
                "Thread {thread_id} should succeed"
            );

            let doc_ptr = (*ctx.result.lock().unwrap()).cast::<HedlDocument>();
            assert!(
                !doc_ptr.is_null(),
                "Thread {thread_id} document should not be null"
            );

            // Clean up
            hedl_free_document(doc_ptr);
            hedl_async_free(op);
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// Null Pointer Tests
// =============================================================================

#[test]
fn test_async_cancel_null_pointer() {
    unsafe {
        // Should not crash
        hedl_async_cancel(std::ptr::null_mut());
    }
}

#[test]
fn test_async_free_null_pointer() {
    unsafe {
        // Should not crash
        hedl_async_free(std::ptr::null_mut());
    }
}

// =============================================================================
// Multiple Operations on Same Document
// =============================================================================

#[test]
fn test_multiple_async_operations_same_document() {
    unsafe {
        // Parse a document
        let mut doc: *mut HedlDocument = std::ptr::null_mut();
        let result = hedl_parse(VALID_HEDL.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);

        // Submit multiple operations concurrently
        let ctx1 = TestContext::new();
        let ctx2 = TestContext::new();
        let ctx3 = TestContext::new();

        let ctx1_ptr = std::ptr::addr_of!(ctx1) as *mut c_void;
        let ctx2_ptr = std::ptr::addr_of!(ctx2) as *mut c_void;
        let ctx3_ptr = std::ptr::addr_of!(ctx3) as *mut c_void;

        extern "C" fn callback(
            status: c_int,
            result: *mut c_void,
            _error: *const c_char,
            user_data: *mut c_void,
        ) {
            let ctx = unsafe { &*(user_data as *const TestContext) };
            ctx.status.store(status, Ordering::Release);

            if !result.is_null() {
                *ctx.result.lock().unwrap() = result;
            }

            ctx.completed.store(true, Ordering::Release);
        }

        let op1 = hedl_canonicalize_async(doc, Some(callback), ctx1_ptr);
        let op2 = hedl_lint_async(doc, Some(callback), ctx2_ptr);

        #[cfg(feature = "json")]
        let op3 = hedl_to_json_async(doc, 0, Some(callback), ctx3_ptr);

        assert!(!op1.is_null());
        assert!(!op2.is_null());
        #[cfg(feature = "json")]
        assert!(!op3.is_null());

        // Wait for all operations
        assert!(ctx1.wait_for_completion(5000));
        assert!(ctx2.wait_for_completion(5000));
        #[cfg(feature = "json")]
        assert!(ctx3.wait_for_completion(5000));

        // Verify all succeeded
        assert_eq!(ctx1.status.load(Ordering::Acquire), HEDL_OK);
        assert_eq!(ctx2.status.load(Ordering::Acquire), HEDL_OK);
        #[cfg(feature = "json")]
        assert_eq!(ctx3.status.load(Ordering::Acquire), HEDL_OK);

        // Clean up results
        let canonical_ptr = (*ctx1.result.lock().unwrap()).cast::<c_char>();
        let diag_ptr = (*ctx2.result.lock().unwrap()).cast::<HedlDiagnostics>();
        #[cfg(feature = "json")]
        let json_ptr = (*ctx3.result.lock().unwrap()).cast::<c_char>();

        hedl_free_string(canonical_ptr);
        hedl_free_diagnostics(diag_ptr);
        #[cfg(feature = "json")]
        hedl_free_string(json_ptr);

        hedl_async_free(op1);
        hedl_async_free(op2);
        #[cfg(feature = "json")]
        hedl_async_free(op3);

        hedl_free_document(doc);
    }
}
