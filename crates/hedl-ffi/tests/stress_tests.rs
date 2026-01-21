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

//! Stress tests for extreme concurrency scenarios.
//!
//! These tests push the thread safety mechanisms to their limits:
//! - Very high thread counts (100-1000 threads)
//! - Long-running tests (minutes of execution)
//! - Memory pressure scenarios
//! - Sustained high contention

use hedl_ffi::*;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

// Test data
const VALID_HEDL: &[u8] = b"%VERSION: 1.0\n---\nkey: value\0";
const INVALID_HEDL: &[u8] = b"not valid hedl\0";

// =============================================================================
// Extreme Thread Count Tests
// =============================================================================

#[test]
#[ignore] // Long-running test - run with: cargo test --test stress_tests -- --ignored
fn test_extreme_thread_count_1000() {
    const NUM_THREADS: usize = 1000;
    const ITERATIONS: usize = 10;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let start = Instant::now();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for i in 0..ITERATIONS {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    let result = hedl_parse(valid_clone.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

                    assert_eq!(result, HEDL_OK, "Thread {thread_id} iteration {i} failed");

                    // Verify error isolation
                    let err = hedl_get_last_error();
                    assert!(
                        err.is_null(),
                        "Thread {thread_id} iteration {i} has unexpected error"
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

    let elapsed = start.elapsed();
    println!("Completed {NUM_THREADS} threads x {ITERATIONS} iterations in {elapsed:?}");

    // Should complete in reasonable time (< 30 seconds)
    assert!(elapsed < Duration::from_secs(30));
}

#[test]
#[ignore] // Long-running test
fn test_extreme_thread_count_100() {
    const NUM_THREADS: usize = 100;
    const ITERATIONS: usize = 100;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let start = Instant::now();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                for _ in 0..ITERATIONS {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
                    hedl_free_document(doc);
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    println!("Completed {NUM_THREADS} threads x {ITERATIONS} iterations in {elapsed:?}");

    assert!(elapsed < Duration::from_secs(60));
}

// =============================================================================
// Sustained High Contention Tests
// =============================================================================

#[test]
#[ignore] // Long-running test
fn test_sustained_high_contention() {
    const NUM_THREADS: usize = 64;
    const DURATION_SECONDS: u64 = 5;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();
            let running_clone = running.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                let mut iterations = 0;
                while running_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

                    let mut out_str: *mut c_char = ptr::null_mut();
                    hedl_canonicalize(doc, &mut out_str);
                    hedl_free_string(out_str);

                    hedl_free_document(doc);

                    iterations += 1;
                }

                (thread_id, iterations)
            })
        })
        .collect();

    // Let threads run for specified duration
    thread::sleep(Duration::from_secs(DURATION_SECONDS));
    running.store(false, std::sync::atomic::Ordering::Relaxed);

    let mut total_iterations = 0;
    for handle in handles {
        let (thread_id, iterations) = handle.join().unwrap();
        total_iterations += iterations;
        println!("Thread {thread_id} completed {iterations} iterations");
    }

    println!("Completed {total_iterations} total iterations in {DURATION_SECONDS} seconds");

    // Should complete significant work
    assert!(total_iterations > 1000);
}

// =============================================================================
// Memory Pressure Under Concurrency
// =============================================================================

#[test]
#[ignore] // Long-running test
fn test_memory_pressure_concurrent_allocations() {
    const NUM_THREADS: usize = 16;
    const DOCS_PER_THREAD: usize = 1000;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                let mut docs = Vec::new();

                // Allocate many documents
                for _ in 0..DOCS_PER_THREAD {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    let result = hedl_parse(valid_clone.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

                    if result == HEDL_OK {
                        docs.push(doc);
                    }
                }

                // Verify all documents are valid
                for (i, doc) in docs.iter().enumerate() {
                    let mut major: i32 = 0;
                    let mut minor: i32 = 0;
                    let result = hedl_get_version(*doc, &mut major, &mut minor);
                    assert_eq!(
                        result, HEDL_OK,
                        "Thread {thread_id} document {i} is invalid"
                    );
                }

                // Free all documents
                for doc in docs {
                    hedl_free_document(doc);
                }

                (thread_id, DOCS_PER_THREAD)
            })
        })
        .collect();

    let mut total_docs = 0;
    for handle in handles {
        let (thread_id, count) = handle.join().unwrap();
        total_docs += count;
        println!("Thread {thread_id} allocated and freed {count} documents");
    }

    println!("Total documents allocated and freed: {total_docs}");

    assert_eq!(total_docs, NUM_THREADS * DOCS_PER_THREAD);
}

// =============================================================================
// Error Handling Stress Tests
// =============================================================================

#[test]
#[ignore] // Long-running test
fn test_error_handling_stress() {
    const NUM_THREADS: usize = 32;
    const ITERATIONS: usize = 1000;

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
                    // Alternate between valid and invalid
                    let input = if i % 2 == 0 {
                        &valid_clone
                    } else {
                        &invalid_clone
                    };

                    let result = hedl_validate(input.as_ptr().cast::<c_char>(), -1, 0);

                    if result == HEDL_OK {
                        success_count += 1;
                        assert!(hedl_get_last_error().is_null());
                    } else {
                        error_count += 1;
                        let err = hedl_get_last_error();
                        assert!(!err.is_null());

                        // Verify error message is valid
                        let msg = CStr::from_ptr(err).to_str();
                        assert!(msg.is_ok());
                    }
                }

                (thread_id, success_count, error_count)
            })
        })
        .collect();

    let mut total_success = 0;
    let mut total_errors = 0;

    for handle in handles {
        let (thread_id, success_count, error_count) = handle.join().unwrap();
        total_success += success_count;
        total_errors += error_count;
        println!("Thread {thread_id} - Success: {success_count}, Errors: {error_count}");
    }

    println!("Total - Success: {total_success}, Errors: {total_errors}");

    assert_eq!(total_success + total_errors, NUM_THREADS * ITERATIONS);
    assert!(total_success > 0);
    assert!(total_errors > 0);
}

// =============================================================================
// Rapid Thread Creation and Destruction
// =============================================================================

#[test]
#[ignore] // Long-running test
fn test_rapid_thread_creation_destruction() {
    const WAVE_COUNT: usize = 50;
    const THREADS_PER_WAVE: usize = 10;

    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    for wave in 0..WAVE_COUNT {
        let handles: Vec<_> = (0..THREADS_PER_WAVE)
            .map(|thread_id| {
                let valid_clone = valid_hedl.clone();

                thread::spawn(move || unsafe {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
                    hedl_free_document(doc);
                    thread_id
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        if wave % 10 == 0 {
            println!("Completed wave {wave} of {WAVE_COUNT}");
        }
    }

    println!("Completed {WAVE_COUNT} waves of {THREADS_PER_WAVE} threads");
}

// =============================================================================
// Callback Stress Tests
// =============================================================================

#[cfg(feature = "json")]
#[test]
#[ignore] // Long-running test
fn test_callback_stress() {
    use std::os::raw::c_void;
    use std::slice;

    const NUM_THREADS: usize = 32;
    const CALLBACKS_PER_THREAD: usize = 100;

    unsafe extern "C" fn counting_callback(
        data: *const c_char,
        len: usize,
        user_data: *mut c_void,
    ) {
        let count = &mut *user_data.cast::<usize>();
        *count += len;

        // Verify data is valid
        if !data.is_null() && len > 0 {
            let slice = slice::from_raw_parts(data.cast::<u8>(), len);
            assert!(!slice.is_empty());
        }
    }

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                let mut total_bytes = 0;

                for _ in 0..CALLBACKS_PER_THREAD {
                    let mut doc: *mut HedlDocument = ptr::null_mut();
                    hedl_parse(valid_clone.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

                    let result = hedl_to_json_callback(
                        doc,
                        0,
                        counting_callback,
                        std::ptr::addr_of_mut!(total_bytes).cast::<c_void>(),
                    );

                    assert_eq!(result, HEDL_OK);

                    hedl_free_document(doc);
                }

                (thread_id, total_bytes)
            })
        })
        .collect();

    let mut total_bytes = 0;
    for handle in handles {
        let (thread_id, bytes) = handle.join().unwrap();
        total_bytes += bytes;
        println!("Thread {thread_id} processed {bytes} bytes");
    }

    println!("Total bytes processed by callbacks: {total_bytes}");
    assert!(total_bytes > 0);
}

// =============================================================================
// Long-Running Stability Test
// =============================================================================

#[test]
#[ignore] // Very long-running test - run manually
fn test_long_running_stability() {
    const NUM_THREADS: usize = 8;
    const DURATION_SECONDS: u64 = 60;

    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let valid_hedl = Arc::new(VALID_HEDL.to_vec());
    let invalid_hedl = Arc::new(INVALID_HEDL.to_vec());
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));

    let start = Instant::now();

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let barrier_clone = barrier.clone();
            let valid_clone = valid_hedl.clone();
            let invalid_clone = invalid_hedl.clone();
            let running_clone = running.clone();

            thread::spawn(move || unsafe {
                barrier_clone.wait();

                let mut operations = 0;
                let mut errors = 0;

                while running_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    // Mix of different operations
                    match operations % 5 {
                        0 => {
                            // Parse valid document
                            let mut doc: *mut HedlDocument = ptr::null_mut();
                            if hedl_parse(valid_clone.as_ptr().cast::<c_char>(), -1, 0, &mut doc)
                                == HEDL_OK
                            {
                                hedl_free_document(doc);
                            }
                        }
                        1 => {
                            // Validate invalid document
                            if hedl_validate(invalid_clone.as_ptr().cast::<c_char>(), -1, 0)
                                != HEDL_OK
                            {
                                errors += 1;
                            }
                        }
                        2 => {
                            // Parse and convert
                            let mut doc: *mut HedlDocument = ptr::null_mut();
                            if hedl_parse(valid_clone.as_ptr().cast::<c_char>(), -1, 0, &mut doc)
                                == HEDL_OK
                            {
                                let mut out_str: *mut c_char = ptr::null_mut();
                                hedl_canonicalize(doc, &mut out_str);
                                hedl_free_string(out_str);
                                hedl_free_document(doc);
                            }
                        }
                        3 => {
                            // Error handling
                            hedl_clear_error_threadsafe();
                        }
                        4 => {
                            // Get error (should be null)
                            if hedl_get_last_error().is_null() {
                                // Good - no error
                            } else {
                                errors += 1;
                            }
                        }
                        _ => unreachable!(),
                    }

                    operations += 1;

                    // Periodic check-in
                    if operations % 1000 == 0 {
                        println!("Thread {thread_id} - Operations: {operations}, Errors: {errors}");
                    }
                }

                (thread_id, operations, errors)
            })
        })
        .collect();

    // Run for specified duration
    while start.elapsed() < Duration::from_secs(DURATION_SECONDS) {
        thread::sleep(Duration::from_secs(5));
        println!("Running... {:?}", start.elapsed());
    }

    running.store(false, std::sync::atomic::Ordering::Relaxed);

    let mut total_operations = 0;
    let mut total_errors = 0;

    for handle in handles {
        let (thread_id, operations, errors) = handle.join().unwrap();
        total_operations += operations;
        total_errors += errors;
        println!("Thread {thread_id} final - Operations: {operations}, Errors: {errors}");
    }

    let elapsed = start.elapsed();
    println!("\n=== Stability Test Results ===");
    println!("Duration: {elapsed:?}");
    println!("Total operations: {total_operations}");
    println!("Total errors: {total_errors}");
    println!(
        "Operations per second: {:.2}",
        f64::from(total_operations) / elapsed.as_secs_f64()
    );

    // Verify stability
    assert!(total_operations > 1000, "Should complete many operations");
    let error_rate = f64::from(total_errors) / f64::from(total_operations);
    assert!(
        error_rate < 0.5,
        "Error rate should be low (got {:.2}%)",
        error_rate * 100.0
    );
}

// =============================================================================
// Burst Load Test
// =============================================================================

#[test]
#[ignore] // Long-running test
fn test_burst_load() {
    const NUM_BURSTS: usize = 20;
    const THREADS_PER_BURST: usize = 50;
    const ITERATIONS_PER_BURST: usize = 10;

    let valid_hedl = Arc::new(VALID_HEDL.to_vec());

    for burst in 0..NUM_BURSTS {
        let start = Instant::now();

        let handles: Vec<_> = (0..THREADS_PER_BURST)
            .map(|_| {
                let valid_clone = valid_hedl.clone();

                thread::spawn(move || unsafe {
                    for _ in 0..ITERATIONS_PER_BURST {
                        let mut doc: *mut HedlDocument = ptr::null_mut();
                        hedl_parse(valid_clone.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
                        hedl_free_document(doc);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        println!("Burst {burst} completed {THREADS_PER_BURST} threads in {elapsed:?}");
    }
}
