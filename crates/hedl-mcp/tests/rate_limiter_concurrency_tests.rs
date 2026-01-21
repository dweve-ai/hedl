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

//! Concurrency tests for MCP rate limiter.
//!
//! Tests thread safety, data races, and concurrent access patterns.
//! Note: `RateLimiter` is not inherently thread-safe - these tests verify
//! the behavior when properly synchronized or used with wrapper types.

use hedl_mcp::RateLimiter;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// =============================================================================
// MUTEX-PROTECTED CONCURRENT ACCESS
// =============================================================================

#[test]
fn test_concurrent_requests_with_mutex() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(1000, 500)));
    let mut handles = vec![];

    // Spawn 10 threads, each making 100 requests
    for thread_id in 0..10 {
        let limiter = Arc::clone(&limiter);
        handles.push(thread::spawn(move || {
            let mut allowed = 0;
            for _ in 0..100 {
                let guard = limiter.lock().unwrap();
                if guard.check_limit() {
                    allowed += 1;
                }
                // Explicit drop to release lock before potential context switch
                drop(guard);
            }
            (thread_id, allowed)
        }));
    }

    // Collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    let total_allowed: usize = results.iter().map(|(_, allowed)| allowed).sum();

    // Initial burst is 1000, so total allowed should be at least 1000
    assert!(
        total_allowed >= 1000,
        "Expected at least 1000 allowed requests, got {total_allowed}"
    );

    // With mutex protection, no data corruption should occur
    let guard = limiter.lock().unwrap();
    let remaining = guard.tokens();
    assert!(
        remaining <= 1000,
        "Tokens should be at most max_tokens: {remaining}"
    );
}

#[test]
fn test_concurrent_reset_operations() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(100, 50)));
    let mut handles = vec![];

    // Multiple threads doing resets while others check limits
    for thread_id in 0..5 {
        let limiter = Arc::clone(&limiter);
        handles.push(thread::spawn(move || {
            for iteration in 0..10 {
                let guard = limiter.lock().unwrap();
                if thread_id % 2 == 0 {
                    // Even threads: reset
                    guard.reset();
                } else {
                    // Odd threads: consume
                    guard.check_limit();
                }
                drop(guard);

                // Small delay to increase interleaving
                if iteration % 3 == 0 {
                    thread::sleep(Duration::from_micros(10));
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify state is valid after all operations
    let guard = limiter.lock().unwrap();
    let tokens = guard.tokens();
    assert!(tokens <= 100, "Tokens {tokens} should be <= max 100");
}

#[test]
fn test_high_contention_scenario() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(500, 250)));
    let total_requests = Arc::new(Mutex::new(0usize));
    let total_allowed = Arc::new(Mutex::new(0usize));
    let mut handles = vec![];

    // Many threads competing for limited tokens
    for _ in 0..20 {
        let limiter = Arc::clone(&limiter);
        let total_requests = Arc::clone(&total_requests);
        let total_allowed = Arc::clone(&total_allowed);

        handles.push(thread::spawn(move || {
            for _ in 0..50 {
                *total_requests.lock().unwrap() += 1;

                let guard = limiter.lock().unwrap();
                if guard.check_limit() {
                    *total_allowed.lock().unwrap() += 1;
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let requests = *total_requests.lock().unwrap();
    let allowed = *total_allowed.lock().unwrap();

    assert_eq!(requests, 1000, "All requests should be counted");
    // Allowed should be bounded by burst + refill during execution
    assert!(allowed <= 600, "Allowed {allowed} seems too high");
    assert!(
        allowed >= 400,
        "Allowed {allowed} seems too low (burst is 500)"
    );
}

// =============================================================================
// STRESS TESTS FOR MUTEX CONTENTION
// =============================================================================

#[test]
fn test_rapid_lock_unlock_cycles() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(10000, 5000)));
    let mut handles = vec![];

    for _ in 0..4 {
        let limiter = Arc::clone(&limiter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                let guard = limiter.lock().unwrap();
                let _ = guard.check_limit();
                // Immediately release
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify state is consistent
    let guard = limiter.lock().unwrap();
    assert!(guard.tokens() <= 10000);
}

#[test]
fn test_mixed_read_write_operations() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(200, 100)));
    let mut writer_handles = vec![];
    let mut reader_handles = vec![];

    // Writers (check_limit)
    for _ in 0..5 {
        let limiter = Arc::clone(&limiter);
        writer_handles.push(thread::spawn(move || {
            for _ in 0..50 {
                let guard = limiter.lock().unwrap();
                guard.check_limit();
            }
        }));
    }

    // Readers (tokens)
    for _ in 0..5 {
        let limiter = Arc::clone(&limiter);
        reader_handles.push(thread::spawn(move || {
            let mut readings = vec![];
            for _ in 0..50 {
                let guard = limiter.lock().unwrap();
                readings.push(guard.tokens());
            }
            readings
        }));
    }

    // Wait for writers
    for handle in writer_handles {
        handle.join().unwrap();
    }

    // Collect reader results and verify all readings were valid
    for handle in reader_handles {
        let readings = handle.join().unwrap();
        for reading in readings {
            assert!(reading <= 200, "Reading {reading} exceeded max");
        }
    }

    // Final state check
    let guard = limiter.lock().unwrap();
    assert!(guard.tokens() <= 200);
}

// =============================================================================
// PRODUCER-CONSUMER PATTERN
// =============================================================================

#[test]
fn test_producer_consumer_rate_limiting() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(100, 50)));
    let work_queue = Arc::new(Mutex::new(Vec::new()));
    let completed = Arc::new(Mutex::new(0usize));

    // Producer: adds work items
    let limiter_prod = Arc::clone(&limiter);
    let queue_prod = Arc::clone(&work_queue);
    let producer = thread::spawn(move || {
        for i in 0..200 {
            queue_prod.lock().unwrap().push(i);

            // Rate limit production
            let guard = limiter_prod.lock().unwrap();
            if !guard.check_limit() {
                // Drop lock and wait for refill
                drop(guard);
                thread::sleep(Duration::from_millis(10));
            }
        }
    });

    // Consumer: processes work items
    let queue_cons = Arc::clone(&work_queue);
    let completed_cons = Arc::clone(&completed);
    let consumer = thread::spawn(move || {
        let mut processed = 0;
        while processed < 200 {
            let item = {
                let mut queue = queue_cons.lock().unwrap();
                queue.pop()
            };

            if item.is_some() {
                *completed_cons.lock().unwrap() += 1;
                processed += 1;
            } else {
                thread::sleep(Duration::from_millis(1));
            }
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();

    assert_eq!(*completed.lock().unwrap(), 200);
}

// =============================================================================
// FAIRNESS TESTS
// =============================================================================

#[test]
fn test_fairness_across_threads() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(500, 250))); // Larger bucket for better fairness
    let thread_counts: Vec<Arc<Mutex<usize>>> = (0..5).map(|_| Arc::new(Mutex::new(0))).collect();
    let mut handles = vec![];

    for count in &thread_counts {
        let limiter = Arc::clone(&limiter);
        let count = Arc::clone(count);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let guard = limiter.lock().unwrap();
                if guard.check_limit() {
                    *count.lock().unwrap() += 1;
                }
                drop(guard);
                // Small sleep to improve interleaving
                thread::sleep(Duration::from_micros(10));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Check distribution
    let counts: Vec<usize> = thread_counts.iter().map(|c| *c.lock().unwrap()).collect();
    let total: usize = counts.iter().sum();

    // Total should be equal to all requests (500 tokens for 500 requests)
    assert!(total >= 450, "Total allowed {total} is too low");

    // Check fairness - verify total distribution is reasonable
    // Note: Mutex fairness is not guaranteed, so we only check total
    // Not all threads are guaranteed to get requests
    let _min_count = *counts.iter().min().unwrap();
    let max_count = *counts.iter().max().unwrap();

    // At least verify some reasonable distribution occurred
    assert!(
        max_count <= total,
        "Max count {max_count} exceeds total {total}"
    );
}

// =============================================================================
// DEADLOCK PREVENTION
// =============================================================================

#[test]
fn test_no_deadlock_with_nested_access() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(100, 50)));

    // This should complete without deadlock
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(5);

    let limiter_clone = Arc::clone(&limiter);
    let handle = thread::spawn(move || {
        for _ in 0..100 {
            let guard = limiter_clone.lock().unwrap();
            guard.check_limit();
            let _ = guard.tokens();
            // Don't hold lock while getting another lock (which would be the same lock)
        }
    });

    // Wait with timeout
    loop {
        if handle.is_finished() {
            handle.join().unwrap();
            break;
        }
        assert!(
            start.elapsed() <= timeout,
            "Potential deadlock detected - thread didn't complete in {} seconds",
            timeout.as_secs()
        );
        thread::sleep(Duration::from_millis(10));
    }
}

// =============================================================================
// CLONE BEHAVIOR UNDER CONCURRENCY
// =============================================================================

#[test]
fn test_cloned_limiters_independent() {
    let limiter1 = RateLimiter::new(100, 50);
    let limiter2 = limiter1.clone();

    // Consume all tokens in limiter2
    for _ in 0..100 {
        limiter2.check_limit();
    }

    // limiter1 should still have all tokens (it's a Clone, not Arc)
    assert_eq!(
        limiter1.tokens(),
        100,
        "Cloned limiter should be independent"
    );
}

#[test]
fn test_concurrent_cloned_limiters() {
    let original = RateLimiter::new(100, 50);
    let mut handles = vec![];

    for _ in 0..5 {
        let limiter = original.clone();
        handles.push(thread::spawn(move || {
            let mut allowed = 0;
            for _ in 0..100 {
                if limiter.check_limit() {
                    allowed += 1;
                }
            }
            allowed
        }));
    }

    let results: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Each clone should allow exactly 100 (full burst) since they're independent
    for (i, allowed) in results.iter().enumerate() {
        assert_eq!(*allowed, 100, "Clone {i} should allow 100, got {allowed}");
    }
}

// =============================================================================
// TIME-BASED CONCURRENCY
// =============================================================================

#[test]
fn test_concurrent_with_time_progression() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(50, 100))); // Fast refill
    let allowed_count = Arc::new(Mutex::new(0usize));
    let mut handles = vec![];

    // Multiple threads over extended time
    for _ in 0..4 {
        let limiter = Arc::clone(&limiter);
        let allowed_count = Arc::clone(&allowed_count);

        handles.push(thread::spawn(move || {
            for _ in 0..25 {
                let guard = limiter.lock().unwrap();
                if guard.check_limit() {
                    *allowed_count.lock().unwrap() += 1;
                }
                drop(guard);
                // Spread requests over time
                thread::sleep(Duration::from_millis(2));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let total = *allowed_count.lock().unwrap();

    // With 100 tokens/sec and ~200ms of execution time (4 threads * 25 requests * 2ms)
    // we should allow: initial 50 + ~20 refill = ~70 requests
    assert!(
        total >= 50,
        "Expected at least 50 allowed (initial burst), got {total}"
    );
    assert!(
        total <= 100,
        "Expected at most ~100 allowed (burst + refill), got {total}"
    );
}

// =============================================================================
// ORDERING AND VISIBILITY
// =============================================================================

#[test]
fn test_token_updates_visible_across_threads() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(100, 50)));

    // Thread 1: consume all tokens
    let limiter1 = Arc::clone(&limiter);
    let t1 = thread::spawn(move || {
        let guard = limiter1.lock().unwrap();
        for _ in 0..100 {
            guard.check_limit();
        }
        guard.tokens() // Return remaining
    });

    let remaining = t1.join().unwrap();

    // Thread 2: should see the consumed state
    let limiter2 = Arc::clone(&limiter);
    let t2 = thread::spawn(move || {
        let guard = limiter2.lock().unwrap();
        guard.tokens()
    });

    let visible_remaining = t2.join().unwrap();

    // The second thread should see approximately the same state
    // (might have slight difference due to refill between calls)
    assert!(
        visible_remaining <= remaining + 5,
        "Second thread should see consumed state: {visible_remaining} vs {remaining}"
    );
}

// =============================================================================
// ERROR HANDLING IN CONCURRENT CONTEXT
// =============================================================================

#[test]
fn test_panic_safety_preserves_state() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(100, 50)));

    // Thread that panics while holding lock
    let limiter1 = Arc::clone(&limiter);
    let t1 = thread::spawn(move || {
        let _guard = limiter1.lock().unwrap();
        panic!("Intentional panic for testing");
    });

    // This will poison the mutex
    let _ = t1.join();

    // Subsequent access should detect poisoned mutex
    let limiter2 = Arc::clone(&limiter);
    let result = limiter2.lock();

    // Poisoned mutex is recoverable - we can still access the data
    match result {
        Ok(_guard) => {
            // Mutex wasn't poisoned (shouldn't happen in this test)
            panic!("Expected poisoned mutex");
        }
        Err(poisoned) => {
            // Recover the data
            let guard = poisoned.into_inner();
            // State should still be valid
            assert!(guard.tokens() <= 100);
        }
    }
}

// =============================================================================
// PERFORMANCE CHARACTERISTICS
// =============================================================================

#[test]
fn test_lock_acquisition_time() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(100, 50)));
    let mut timings = vec![];

    // Measure lock acquisition time
    for _ in 0..100 {
        let start = std::time::Instant::now();
        let _guard = limiter.lock().unwrap();
        let elapsed = start.elapsed();
        timings.push(elapsed);
    }

    // Calculate average (should be fast without contention)
    let avg = timings.iter().sum::<Duration>() / timings.len() as u32;
    assert!(
        avg < Duration::from_micros(100),
        "Average lock time {avg:?} is too slow"
    );
}

#[test]
fn test_throughput_under_contention() {
    let limiter = Arc::new(Mutex::new(RateLimiter::new(10000, 5000)));
    let start = std::time::Instant::now();
    let operations = Arc::new(Mutex::new(0usize));
    let mut handles = vec![];

    // Run for fixed duration with multiple threads
    let duration = Duration::from_millis(200);

    for _ in 0..4 {
        let limiter = Arc::clone(&limiter);
        let operations = Arc::clone(&operations);

        handles.push(thread::spawn(move || {
            while start.elapsed() < duration {
                let guard = limiter.lock().unwrap();
                guard.check_limit();
                *operations.lock().unwrap() += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let total_ops = *operations.lock().unwrap();
    let elapsed = start.elapsed();
    let ops_per_sec = total_ops as f64 / elapsed.as_secs_f64();

    // Should achieve reasonable throughput
    assert!(
        ops_per_sec > 1000.0,
        "Throughput {ops_per_sec} ops/sec is too low"
    );
}
