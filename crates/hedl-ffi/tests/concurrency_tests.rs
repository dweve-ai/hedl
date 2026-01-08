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

//! Comprehensive concurrency tests for the HEDL FFI interface.
//!
//! These tests verify:
//! - Thread-local error storage isolation
//! - Concurrent FFI operations without cross-contamination
//! - Error handling in multi-threaded scenarios
//! - Stress testing with many concurrent threads
//! - Race condition detection
//! - Memory safety in concurrent scenarios

use hedl_ffi::*;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Barrier};
use std::thread;

// Test data
const VALID_HEDL: &[u8] = b"%VERSION: 1.0\n---\nkey: value\0";
const INVALID_HEDL: &[u8] = b"not valid hedl\0";

// =============================================================================
// Basic Thread-Local Error Isolation Tests
// =============================================================================

#[test]
fn test_thread_local_error_isolation_basic() {
    unsafe {
        // Create two threads with different error states
        let invalid = Arc::new(INVALID_HEDL.to_vec());
        let valid = Arc::new(VALID_HEDL.to_vec());

        let invalid_clone = invalid.clone();
        let thread1 = thread::spawn(move || {
            // Thread 1: Trigger an error
            let result = hedl_validate(invalid_clone.as_ptr() as *const c_char, -1, 0);
            assert_ne!(result, HEDL_OK);

            let err = hedl_get_last_error();
            assert!(!err.is_null());
            let msg = CStr::from_ptr(err).to_str().unwrap();
            assert!(msg.contains("Parse error"));
            msg.to_string()
        });

        let valid_clone = valid.clone();
        let thread2 = thread::spawn(move || {
            // Thread 2: No error
            let mut doc: *mut HedlDocument = ptr::null_mut();
            hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

            let err = hedl_get_last_error();
            let has_error = !err.is_null();

            hedl_free_document(doc);
            has_error
        });

        let error_msg = thread1.join().unwrap();
        let had_error = thread2.join().unwrap();

        assert!(error_msg.contains("Parse error"));
        assert!(!had_error); // Thread 2 should not see thread 1's error
    }
}

#[test]
fn test_hedl_get_last_error_threadsafe_isolation() {
    unsafe {
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = vec![];

        for i in 0..3 {
            let barrier_clone = barrier.clone();
            let invalid = Arc::new(INVALID_HEDL.to_vec());

            let handle = thread::spawn(move || {
                // All threads wait at the barrier
                barrier_clone.wait();

                // All threads trigger errors simultaneously
                let result = hedl_validate(invalid.as_ptr() as *const c_char, -1, 0);
                assert_ne!(result, HEDL_OK);

                // Each thread should get its own error
                let err = hedl_get_last_error_threadsafe();
                assert!(!err.is_null(), "Thread {} got NULL error", i);

                let msg = CStr::from_ptr(err).to_str().unwrap();
                assert!(msg.contains("Parse error"), "Thread {} got wrong error", i);

                msg.to_string()
            });

            handles.push(handle);
        }

        // All threads should successfully get their errors
        for handle in handles {
            let error_msg = handle.join().unwrap();
            assert!(error_msg.contains("Parse error"));
        }
    }
}

#[test]
fn test_hedl_clear_error_threadsafe() {
    unsafe {
        let handles: Vec<_> = (0..4)
            .map(|_| {
                thread::spawn(|| {
                    // Trigger an error
                    hedl_validate(INVALID_HEDL.as_ptr() as *const c_char, -1, 0);
                    assert!(!hedl_get_last_error_threadsafe().is_null());

                    // Clear the error
                    hedl_clear_error_threadsafe();
                    assert!(hedl_get_last_error_threadsafe().is_null());

                    // Trigger another error
                    hedl_validate(INVALID_HEDL.as_ptr() as *const c_char, -1, 0);
                    assert!(!hedl_get_last_error_threadsafe().is_null());
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

// =============================================================================
// Concurrent Parse Operations
// =============================================================================

#[test]
fn test_concurrent_parse_operations() {
    const NUM_THREADS: usize = 8;
    const ITERATIONS_PER_THREAD: usize = 100;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for i in 0..ITERATIONS_PER_THREAD {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    let result =
                        hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

                    assert_eq!(
                        result, HEDL_OK,
                        "Thread {} iteration {} failed",
                        thread_id, i
                    );
                    assert!(!doc.is_null());

                    // Verify no error
                    let err = hedl_get_last_error_threadsafe();
                    assert!(
                        err.is_null(),
                        "Thread {} iteration {} has unexpected error",
                        thread_id,
                        i
                    );

                    hedl_free_document(doc);
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_parse_with_errors() {
    const NUM_THREADS: usize = 16;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());
    let invalid_hedl = Arc::new(INVALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();
            let invalid_clone = invalid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                // Even-numbered threads parse valid HEDL
                // Odd-numbered threads parse invalid HEDL
                let input = if thread_id % 2 == 0 {
                    &valid_clone
                } else {
                    &invalid_clone
                };

                let mut doc: *mut HedlDocument = ptr::null_mut();
                let result = hedl_parse(input.as_ptr() as *const c_char, -1, 0, &mut doc);

                if thread_id % 2 == 0 {
                    // Even threads should succeed
                    assert_eq!(result, HEDL_OK, "Thread {} should succeed", thread_id);
                    assert!(!doc.is_null());
                    assert!(hedl_get_last_error_threadsafe().is_null());
                    hedl_free_document(doc);
                } else {
                    // Odd threads should fail
                    assert_ne!(result, HEDL_OK, "Thread {} should fail", thread_id);
                    assert!(doc.is_null());

                    let err = hedl_get_last_error_threadsafe();
                    assert!(!err.is_null(), "Thread {} should have error", thread_id);

                    let msg = CStr::from_ptr(err).to_str().unwrap();
                    assert!(msg.contains("Parse error"));
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// Concurrent Conversion Operations
// =============================================================================

#[cfg(feature = "json")]
#[test]
fn test_concurrent_json_conversions() {
    const NUM_THREADS: usize = 8;
    const ITERATIONS: usize = 50;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for i in 0..ITERATIONS {
                    // Parse
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

                    // Convert to JSON
                    let mut json_str: *mut c_char = ptr::null_mut();
                    let result = hedl_to_json(doc, 1, &mut json_str);
                    assert_eq!(
                        result, HEDL_OK,
                        "Thread {} iteration {} JSON conversion failed",
                        thread_id, i
                    );

                    // Verify JSON is valid
                    assert!(!json_str.is_null());
                    let json = CStr::from_ptr(json_str).to_str().unwrap();
                    assert!(json.contains("key"));

                    // Clean up
                    hedl_free_string(json_str);
                    hedl_free_document(doc);
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_concurrent_yaml_conversions() {
    const NUM_THREADS: usize = 8;
    const ITERATIONS: usize = 50;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for i in 0..ITERATIONS {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

                    let mut yaml_str: *mut c_char = ptr::null_mut();
                    let result = hedl_to_yaml(doc, 1, &mut yaml_str);
                    assert_eq!(
                        result, HEDL_OK,
                        "Thread {} iteration {} YAML conversion failed",
                        thread_id, i
                    );

                    assert!(!yaml_str.is_null());
                    hedl_free_string(yaml_str);
                    hedl_free_document(doc);
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// Concurrent Canonicalization Operations
// =============================================================================

#[test]
fn test_concurrent_canonicalize() {
    const NUM_THREADS: usize = 8;
    const ITERATIONS: usize = 50;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for i in 0..ITERATIONS {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

                    let mut canon_str: *mut c_char = ptr::null_mut();
                    let result = hedl_canonicalize(doc, &mut canon_str);
                    assert_eq!(
                        result, HEDL_OK,
                        "Thread {} iteration {} canonicalize failed",
                        thread_id, i
                    );

                    assert!(!canon_str.is_null());
                    hedl_free_string(canon_str);
                    hedl_free_document(doc);
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// Concurrent Lint Operations
// =============================================================================

#[test]
fn test_concurrent_lint() {
    const NUM_THREADS: usize = 8;
    const ITERATIONS: usize = 50;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for i in 0..ITERATIONS {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

                    let mut diag: *mut HedlDiagnostics = ptr::null_mut();
                    let result = hedl_lint(doc, &mut diag);
                    assert_eq!(
                        result, HEDL_OK,
                        "Thread {} iteration {} lint failed",
                        thread_id, i
                    );

                    assert!(!diag.is_null());
                    let count = hedl_diagnostics_count(diag);
                    assert!(count >= 0);

                    hedl_free_diagnostics(diag);
                    hedl_free_document(doc);
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// Mixed Operations Stress Test
// =============================================================================

#[test]
fn test_concurrent_mixed_operations() {
    const NUM_THREADS: usize = 16;
    const ITERATIONS: usize = 25;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for i in 0..ITERATIONS {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

                    // Perform different operations based on thread ID
                    match thread_id % 4 {
                        0 => {
                            // Canonicalize
                            let mut canon_str: *mut c_char = ptr::null_mut();
                            hedl_canonicalize(doc, &mut canon_str);
                            hedl_free_string(canon_str);
                        }
                        1 => {
                            // Lint
                            let mut diag: *mut HedlDiagnostics = ptr::null_mut();
                            hedl_lint(doc, &mut diag);
                            hedl_free_diagnostics(diag);
                        }
                        2 => {
                            // Get version
                            let mut major: i32 = 0;
                            let mut minor: i32 = 0;
                            hedl_get_version(doc, &mut major, &mut minor);
                        }
                        3 => {
                            // Get counts
                            hedl_schema_count(doc);
                            hedl_alias_count(doc);
                            hedl_root_item_count(doc);
                        }
                        _ => unreachable!(),
                    }

                    hedl_free_document(doc);

                    // Verify no error leaked from other threads
                    let err = hedl_get_last_error_threadsafe();
                    assert!(
                        err.is_null(),
                        "Thread {} iteration {} has unexpected error",
                        thread_id,
                        i
                    );
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// Error State Isolation Stress Test
// =============================================================================

#[test]
fn test_error_state_isolation_stress() {
    const NUM_THREADS: usize = 32;
    const ITERATIONS: usize = 100;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());
    let invalid_hedl = Arc::new(INVALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();
            let invalid_clone = invalid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                let mut error_count = 0;
                let mut success_count = 0;

                for i in 0..ITERATIONS {
                    // Alternate between valid and invalid to create error/success pattern
                    let input = if (thread_id + i) % 2 == 0 {
                        &valid_clone
                    } else {
                        &invalid_clone
                    };

                    let result = hedl_validate(input.as_ptr() as *const c_char, -1, 0);

                    if result == HEDL_OK {
                        success_count += 1;
                        // Should have no error
                        assert!(hedl_get_last_error_threadsafe().is_null());
                    } else {
                        error_count += 1;
                        // Should have error
                        let err = hedl_get_last_error_threadsafe();
                        assert!(!err.is_null());
                        let msg = CStr::from_ptr(err).to_str().unwrap();
                        assert!(msg.contains("Parse error"));
                    }
                }

                assert!(
                    error_count > 0 && success_count > 0,
                    "Thread {} should have both errors and successes",
                    thread_id
                );

                (thread_id, error_count, success_count)
            })
        })
        .collect();

    for handle in handles {
        let (thread_id, error_count, success_count) = handle.join().unwrap();
        assert!(
            error_count > 0,
            "Thread {} should have errors",
            thread_id
        );
        assert!(
            success_count > 0,
            "Thread {} should have successes",
            thread_id
        );
    }
}

// =============================================================================
// Callback Function Concurrency (zero-copy operations)
// =============================================================================

#[cfg(feature = "json")]
#[test]
fn test_concurrent_callback_operations() {
    const NUM_THREADS: usize = 8;
    const ITERATIONS: usize = 50;

    unsafe extern "C" fn json_callback(
        _data: *const c_char,
        _len: usize,
        _user_data: *mut std::ffi::c_void,
    ) {
        // Simple callback that just validates we received data
    }

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for i in 0..ITERATIONS {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

                    let result = hedl_to_json_callback(doc, 1, json_callback, ptr::null_mut());
                    assert_eq!(
                        result, HEDL_OK,
                        "Thread {} iteration {} callback failed",
                        thread_id, i
                    );

                    hedl_free_document(doc);
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// Memory Safety in Concurrent Scenarios
// =============================================================================

#[test]
fn test_concurrent_memory_safety() {
    const NUM_THREADS: usize = 8;
    const ITERATIONS: usize = 100;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for i in 0..ITERATIONS {
                    // Allocate multiple resources
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

                    let mut canon_str: *mut c_char = ptr::null_mut();
                    hedl_canonicalize(doc, &mut canon_str);

                    let mut diag: *mut HedlDiagnostics = ptr::null_mut();
                    hedl_lint(doc, &mut diag);

                    // Free all resources
                    hedl_free_string(canon_str);
                    hedl_free_diagnostics(diag);
                    hedl_free_document(doc);

                    // Verify no errors
                    assert!(
                        hedl_get_last_error_threadsafe().is_null(),
                        "Thread {} iteration {} leaked error",
                        thread_id,
                        i
                    );
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// Thread Pool Simulation
// =============================================================================

#[test]
fn test_thread_pool_error_handling() {
    const POOL_SIZE: usize = 4;
    const TASKS_PER_THREAD: usize = 50;

    let barrier = Arc::new(Barrier::new(POOL_SIZE));
    let tasks: Arc<Vec<Vec<u8>>> = Arc::new(
        (0..POOL_SIZE * TASKS_PER_THREAD)
            .map(|i| {
                if i % 3 == 0 {
                    INVALID_HEDL.to_vec()
                } else {
                    VALID_HEDL.to_vec()
                }
            })
            .collect(),
    );

    let handles: Vec<_> = (0..POOL_SIZE)
        .map(|worker_id| {
            let barrier_clone = barrier.clone();
            let tasks_clone = tasks.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                // Clear any residual error state (simulating thread pool reuse)
                hedl_clear_error_threadsafe();

                let mut processed = 0;
                let mut errors = 0;

                for task_id in 0..TASKS_PER_THREAD {
                    let global_task_id = worker_id * TASKS_PER_THREAD + task_id;
                    let task = &tasks_clone[global_task_id];

                    let result = hedl_validate(task.as_ptr() as *const c_char, -1, 0);

                    if result == HEDL_OK {
                        processed += 1;
                    } else {
                        errors += 1;
                        // Verify error message is available
                        let err = hedl_get_last_error_threadsafe();
                        assert!(!err.is_null());
                    }

                    // Clear error for next task (simulating error handling)
                    hedl_clear_error_threadsafe();
                }

                (worker_id, processed, errors)
            })
        })
        .collect();

    let mut total_processed = 0;
    let mut total_errors = 0;

    for handle in handles {
        let (worker_id, processed, errors) = handle.join().unwrap();
        total_processed += processed;
        total_errors += errors;
        assert!(
            processed > 0,
            "Worker {} should process some tasks",
            worker_id
        );
    }

    assert_eq!(
        total_processed + total_errors,
        POOL_SIZE * TASKS_PER_THREAD,
        "All tasks should be accounted for"
    );
}

// =============================================================================
// High-Contention Scenario
// =============================================================================

#[test]
fn test_high_contention_scenario() {
    const NUM_THREADS: usize = 64;
    const ITERATIONS: usize = 50;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for _ in 0..ITERATIONS {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    let result =
                        hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

                    assert_eq!(result, HEDL_OK);
                    assert!(!doc.is_null());
                    assert!(hedl_get_last_error_threadsafe().is_null());

                    hedl_free_document(doc);
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
