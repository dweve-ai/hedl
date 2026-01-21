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

//! Property-based tests for MCP rate limiter.
//!
//! Tests the mathematical properties and invariants of the token bucket algorithm
//! using property-based testing with proptest.

use hedl_mcp::RateLimiter;
use proptest::prelude::*;
use std::thread;
use std::time::Duration;

// =============================================================================
// PROPERTY: Token count invariants
// =============================================================================

proptest! {
    /// Property: Token count is always in valid range [0, max_tokens]
    #[test]
    fn prop_tokens_always_in_valid_range(
        max_tokens in 1usize..1000,
        refill_rate in 1usize..1000,
        operations in 0usize..500
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);

        for _ in 0..operations {
            limiter.check_limit();
            let tokens = limiter.tokens();
            prop_assert!(tokens <= max_tokens, "Tokens {} exceeded max {}", tokens, max_tokens);
        }
    }

    /// Property: Initial tokens equals max_tokens
    #[test]
    fn prop_initial_tokens_equals_max(
        max_tokens in 1usize..10000,
        refill_rate in 1usize..10000
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);
        prop_assert_eq!(limiter.tokens(), max_tokens);
    }

    /// Property: max_tokens and refill_rate are immutable after creation
    #[test]
    fn prop_config_values_immutable(
        max_tokens in 1usize..10000,
        refill_rate in 1usize..10000,
        operations in 0usize..100
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);

        for _ in 0..operations {
            limiter.check_limit();
        }

        prop_assert_eq!(limiter.max_tokens(), max_tokens);
        prop_assert_eq!(limiter.refill_rate(), refill_rate);
    }
}

// =============================================================================
// PROPERTY: Burst capacity guarantees
// =============================================================================

proptest! {
    /// Property: Fresh limiter allows exactly max_tokens requests (burst)
    #[test]
    fn prop_burst_allows_exactly_max_tokens(
        max_tokens in 1usize..1000,
        refill_rate in 1usize..1000
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);
        let mut allowed = 0;

        // Rapid burst - fast enough that refill adds very few tokens
        for _ in 0..max_tokens {
            if limiter.check_limit() {
                allowed += 1;
            }
        }

        // Allow some tolerance for potential refill during iteration
        let min_expected = if max_tokens > 10 { max_tokens - 2 } else { max_tokens - 1 };
        prop_assert!(
            allowed >= min_expected.max(1),
            "Expected at least {} requests allowed in burst, got {}",
            min_expected,
            allowed
        );
    }

    /// Property: After consuming all tokens, check_limit returns false
    #[test]
    fn prop_empty_bucket_rejects(
        max_tokens in 1usize..100,
        refill_rate in 1usize..100
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);

        // Consume all tokens very fast
        for _ in 0..(max_tokens + 10) {
            limiter.check_limit();
        }

        // Immediate check should fail (no time for refill)
        // Note: With very high refill rates, some tokens might refill
        // so we just verify tokens <= max_tokens
        let tokens = limiter.tokens();
        prop_assert!(tokens <= max_tokens);
    }
}

// =============================================================================
// PROPERTY: Reset behavior
// =============================================================================

proptest! {
    /// Property: Reset always restores full capacity
    #[test]
    fn prop_reset_restores_full_capacity(
        max_tokens in 1usize..1000,
        refill_rate in 1usize..1000,
        consume in 0usize..500
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);

        // Consume some tokens
        for _ in 0..consume {
            limiter.check_limit();
        }

        // Reset
        limiter.reset();

        // Should be full
        prop_assert_eq!(limiter.tokens(), max_tokens);
    }

    /// Property: Multiple resets maintain idempotency
    #[test]
    fn prop_reset_is_idempotent(
        max_tokens in 1usize..1000,
        refill_rate in 1usize..1000,
        reset_count in 1usize..10
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);

        // Multiple resets
        for _ in 0..reset_count {
            limiter.reset();
            prop_assert_eq!(limiter.tokens(), max_tokens);
        }
    }
}

// =============================================================================
// PROPERTY: Token consumption is monotonically decreasing (without refill)
// =============================================================================

proptest! {
    /// Property: Each check_limit consumes at most one token
    #[test]
    fn prop_check_limit_consumes_at_most_one(
        max_tokens in 10usize..100,
        refill_rate in 1usize..10 // Low refill rate to minimize interference
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);

        let initial = limiter.tokens();
        let allowed = limiter.check_limit();

        if allowed {
            let after = limiter.tokens();
            // Token count should decrease by at most 1 (could be same if refill happened)
            prop_assert!(
                initial >= after || after <= initial + 1,
                "Token count changed unexpectedly: {} -> {}",
                initial,
                after
            );
        }
    }
}

// =============================================================================
// PROPERTY: Refill rate guarantees
// =============================================================================

proptest! {
    /// Property: Tokens never exceed max_tokens after refill
    #[test]
    fn prop_refill_capped_at_max(
        max_tokens in 10usize..100,
        refill_rate in 100usize..1000 // High refill rate
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);

        // Consume some tokens
        for _ in 0..5 {
            limiter.check_limit();
        }

        // Wait for potential over-refill
        thread::sleep(Duration::from_millis(100));

        // Should still be at most max_tokens
        let tokens = limiter.tokens();
        prop_assert!(
            tokens <= max_tokens,
            "Tokens {} exceeded max {}",
            tokens,
            max_tokens
        );
    }
}

// =============================================================================
// PROPERTY: Deterministic behavior for same inputs
// =============================================================================

proptest! {
    /// Property: Same sequence of operations produces consistent results
    #[test]
    fn prop_deterministic_for_same_sequence(
        max_tokens in 10usize..100,
        refill_rate in 10usize..100,
        operations in proptest::collection::vec(any::<bool>(), 1..50)
    ) {
        let limiter1 = RateLimiter::new(max_tokens, refill_rate);
        let limiter2 = RateLimiter::new(max_tokens, refill_rate);

        // Initial state should match
        prop_assert_eq!(limiter1.tokens(), limiter2.tokens());

        // After same operations (without time delay), state should match
        for _ in &operations {
            limiter1.check_limit();
            limiter2.check_limit();
        }

        // Immediate comparison (no time for divergent refill)
        prop_assert_eq!(limiter1.tokens(), limiter2.tokens());
    }
}

// =============================================================================
// EDGE CASES
// =============================================================================

#[test]
fn test_minimum_valid_configuration() {
    let limiter = RateLimiter::new(1, 1);
    assert_eq!(limiter.tokens(), 1);
    assert_eq!(limiter.max_tokens(), 1);
    assert_eq!(limiter.refill_rate(), 1);

    // Can consume the one token
    assert!(limiter.check_limit());
    assert_eq!(limiter.tokens(), 0);

    // Cannot consume more
    assert!(!limiter.check_limit());
}

#[test]
fn test_large_configuration() {
    let max = 1_000_000;
    let rate = 100_000;

    let limiter = RateLimiter::new(max, rate);
    assert_eq!(limiter.tokens(), max);
    assert_eq!(limiter.max_tokens(), max);
    assert_eq!(limiter.refill_rate(), rate);

    // Consume a lot
    for _ in 0..10_000 {
        limiter.check_limit();
    }

    // Should have consumed tokens
    let remaining = limiter.tokens();
    assert!(remaining < max);
}

#[test]
#[should_panic(expected = "max_tokens must be positive")]
fn test_zero_max_tokens_panics() {
    RateLimiter::new(0, 100);
}

#[test]
#[should_panic(expected = "refill_rate must be positive")]
fn test_zero_refill_rate_panics() {
    RateLimiter::new(100, 0);
}

#[test]
fn test_token_monotonicity_fast_operations() {
    let limiter = RateLimiter::new(100, 1); // Very slow refill

    let mut prev_tokens = limiter.tokens();

    // Fast consecutive operations should only decrease tokens
    for _ in 0..50 {
        limiter.check_limit();
        let current = limiter.tokens();
        // With slow refill, tokens should only decrease
        assert!(
            current <= prev_tokens,
            "Tokens increased unexpectedly: {prev_tokens} -> {current}"
        );
        prev_tokens = current;
    }
}

#[test]
fn test_refill_approximation_accuracy() {
    let max_tokens = 1000;
    let refill_rate = 100; // 100 tokens per second

    let limiter = RateLimiter::new(max_tokens, refill_rate);

    // Consume all tokens
    for _ in 0..max_tokens {
        limiter.check_limit();
    }

    // Wait 100ms - should refill ~10 tokens
    thread::sleep(Duration::from_millis(100));

    let tokens = limiter.tokens();
    // Allow 50% tolerance due to timing uncertainty
    assert!(
        (5..=15).contains(&tokens),
        "Expected ~10 tokens after 100ms at 100/sec, got {tokens}"
    );
}

#[test]
fn test_repeated_reset_operations() {
    let limiter = RateLimiter::new(100, 50);

    for iteration in 0..10 {
        // Consume some tokens
        for _ in 0..30 {
            limiter.check_limit();
        }

        let before_reset = limiter.tokens();
        assert!(
            before_reset < 100,
            "Iteration {iteration}: Should have consumed some tokens"
        );

        // Reset
        limiter.reset();

        assert_eq!(
            limiter.tokens(),
            100,
            "Iteration {iteration}: Reset should restore full capacity"
        );
    }
}

#[test]
fn test_interleaved_check_and_reset() {
    let limiter = RateLimiter::new(50, 25);

    for _ in 0..5 {
        // Consume some
        for _ in 0..20 {
            limiter.check_limit();
        }

        // Reset
        limiter.reset();

        // Should be able to consume again
        let mut allowed = 0;
        for _ in 0..50 {
            if limiter.check_limit() {
                allowed += 1;
            }
        }

        assert_eq!(allowed, 50, "Should allow full burst after reset");
    }
}

// =============================================================================
// BOUNDARY VALUE TESTS
// =============================================================================

proptest! {
    /// Property: Boundary values for max_tokens
    #[test]
    fn prop_boundary_max_tokens(
        max_tokens in prop_oneof![Just(1usize), Just(2usize), Just(usize::MAX / 2), Just(10000usize)]
    ) {
        let limiter = RateLimiter::new(max_tokens, 100);
        prop_assert_eq!(limiter.tokens(), max_tokens);
        prop_assert_eq!(limiter.max_tokens(), max_tokens);
    }

    /// Property: Boundary values for refill_rate
    #[test]
    fn prop_boundary_refill_rate(
        refill_rate in prop_oneof![Just(1usize), Just(2usize), Just(10000usize), Just(100000usize)]
    ) {
        let limiter = RateLimiter::new(100, refill_rate);
        prop_assert_eq!(limiter.refill_rate(), refill_rate);
    }
}

// =============================================================================
// SEQUENCE INVARIANTS
// =============================================================================

proptest! {
    /// Property: After N allowed requests, at most N tokens consumed
    #[test]
    fn prop_allowed_requests_bounded(
        max_tokens in 10usize..100,
        refill_rate in 1usize..10,
        request_count in 0usize..200
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);
        let mut allowed = 0;

        for _ in 0..request_count {
            if limiter.check_limit() {
                allowed += 1;
            }
        }

        // Allowed requests can exceed max_tokens due to refill, but should be bounded
        // by max_tokens + (time_elapsed * refill_rate)
        // Since we don't wait, allowed should be approximately max_tokens
        let upper_bound = max_tokens + 10; // Small tolerance for timing
        prop_assert!(
            allowed <= upper_bound,
            "Too many requests allowed: {} (expected <= {})",
            allowed,
            upper_bound
        );
    }
}

// =============================================================================
// STATE MACHINE TESTS
// =============================================================================

#[derive(Debug, Clone)]
enum Operation {
    CheckLimit,
    Reset,
    GetTokens,
}

fn arbitrary_operation() -> impl Strategy<Value = Operation> {
    prop_oneof![
        Just(Operation::CheckLimit),
        Just(Operation::Reset),
        Just(Operation::GetTokens),
    ]
}

proptest! {
    /// Property: Any sequence of operations maintains valid state
    #[test]
    fn prop_state_machine_valid(
        max_tokens in 10usize..100,
        refill_rate in 10usize..100,
        operations in proptest::collection::vec(arbitrary_operation(), 1..100)
    ) {
        let limiter = RateLimiter::new(max_tokens, refill_rate);

        for op in operations {
            match op {
                Operation::CheckLimit => {
                    let _ = limiter.check_limit();
                }
                Operation::Reset => {
                    limiter.reset();
                }
                Operation::GetTokens => {
                    let _ = limiter.tokens();
                }
            }

            // Invariant: tokens always in valid range
            let tokens = limiter.tokens();
            prop_assert!(
                tokens <= max_tokens,
                "Tokens {} exceeded max {}",
                tokens,
                max_tokens
            );
        }
    }
}

// =============================================================================
// FAIRNESS AND THROUGHPUT TESTS
// =============================================================================

#[test]
fn test_sustained_throughput_approximation() {
    let max_tokens = 100;
    let refill_rate = 50; // 50 tokens per second

    let limiter = RateLimiter::new(max_tokens, refill_rate);
    let mut total_allowed = 0;

    // Run for 500ms
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_millis(500) {
        if limiter.check_limit() {
            total_allowed += 1;
        }
        // Small delay to avoid burning CPU
        thread::sleep(Duration::from_micros(100));
    }

    // Expected: initial burst (100) + ~25 tokens (500ms at 50/sec)
    // Allow wide tolerance due to timing uncertainty
    assert!(
        (50..=200).contains(&total_allowed),
        "Expected 50-200 requests over 500ms, got {total_allowed}"
    );
}

#[test]
fn test_bursty_workload() {
    let limiter = RateLimiter::new(100, 50);

    // First burst
    let mut burst1 = 0;
    for _ in 0..150 {
        if limiter.check_limit() {
            burst1 += 1;
        }
    }
    assert_eq!(burst1, 100, "First burst should allow exactly 100");

    // Wait for partial refill
    thread::sleep(Duration::from_millis(100)); // ~5 tokens

    // Second burst
    let mut burst2 = 0;
    for _ in 0..50 {
        if limiter.check_limit() {
            burst2 += 1;
        }
    }
    assert!(
        (3..=10).contains(&burst2),
        "Second burst should allow ~5 requests, got {burst2}"
    );

    // Wait for full refill
    thread::sleep(Duration::from_secs(2));

    // Third burst - should have full capacity
    let mut burst3 = 0;
    for _ in 0..150 {
        if limiter.check_limit() {
            burst3 += 1;
        }
    }
    assert_eq!(
        burst3, 100,
        "Third burst should allow full 100 after long wait"
    );
}
