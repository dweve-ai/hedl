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

//! Rate limiting implementation using token bucket algorithm.
//!
//! Protects the MCP server from DoS attacks via request flooding by limiting
//! the rate at which requests can be processed.

use std::time::Instant;
use tracing::warn;

/// Token bucket rate limiter.
///
/// Implements the token bucket algorithm for rate limiting:
/// - Tokens are refilled at a constant rate
/// - Each request consumes one token
/// - Requests are rejected when the bucket is empty
///
/// # Algorithm
///
/// The token bucket algorithm provides smooth rate limiting with burst capacity:
///
/// 1. **Token Refill**: Tokens are added to the bucket at `refill_rate` per second
/// 2. **Burst Capacity**: Up to `max_tokens` can accumulate in the bucket
/// 3. **Request Handling**: Each request consumes one token
/// 4. **Rate Limit**: Requests are rejected when no tokens are available
///
/// # Advantages
///
/// - **Burst Tolerance**: Allows short bursts up to `max_tokens`
/// - **Smooth Rate Limiting**: Tokens refill continuously, not in discrete windows
/// - **Constant Memory**: O(1) space complexity regardless of request volume
/// - **Low Overhead**: Simple arithmetic operations per request
///
/// # Security
///
/// This rate limiter protects against:
/// - **DoS via Request Flooding**: Limits request rate to prevent resource exhaustion
/// - **Resource Starvation**: Ensures fair access across time windows
///
/// # Examples
///
/// ```
/// use hedl_mcp::RateLimiter;
///
/// // Allow 100 requests/second with burst of 200
/// let mut limiter = RateLimiter::new(200, 100);
///
/// // Check if request is allowed
/// if limiter.check_limit() {
///     // Process request
/// } else {
///     // Reject with 429 Too Many Requests
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Maximum number of tokens in the bucket (burst capacity).
    ///
    /// This allows short bursts of requests up to this limit even if the
    /// sustained rate exceeds the refill rate.
    max_tokens: usize,

    /// Current number of available tokens.
    ///
    /// Decremented on each request, refilled over time at `refill_rate`.
    /// Invariant: `0 <= tokens <= max_tokens`
    tokens: usize,

    /// Token refill rate (tokens per second).
    ///
    /// Determines the sustained request rate the limiter allows.
    /// For example, 100 tokens/sec allows 100 requests/sec sustained.
    refill_rate: usize,

    /// Timestamp of last token refill.
    ///
    /// Used to calculate how many tokens to add based on elapsed time.
    last_refill: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter with token bucket algorithm.
    ///
    /// # Arguments
    ///
    /// * `max_tokens` - Maximum bucket capacity (burst size)
    /// * `refill_rate` - Tokens added per second (sustained rate)
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_mcp::RateLimiter;
    ///
    /// // 100 req/sec sustained, burst of 200
    /// let limiter = RateLimiter::new(200, 100);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `max_tokens` or `refill_rate` is zero.
    pub fn new(max_tokens: usize, refill_rate: usize) -> Self {
        assert!(max_tokens > 0, "max_tokens must be positive");
        assert!(refill_rate > 0, "refill_rate must be positive");

        Self {
            max_tokens,
            tokens: max_tokens, // Start with full bucket
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Check if a request is allowed and consume a token.
    ///
    /// Returns `true` if the request is allowed (token consumed),
    /// `false` if rate limit is exceeded.
    ///
    /// # Algorithm
    ///
    /// 1. Refill tokens based on elapsed time
    /// 2. Check if tokens are available
    /// 3. Consume one token if available
    /// 4. Return success/failure
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_mcp::RateLimiter;
    ///
    /// let mut limiter = RateLimiter::new(100, 50);
    ///
    /// if limiter.check_limit() {
    ///     println!("Request allowed");
    /// } else {
    ///     println!("Rate limit exceeded");
    /// }
    /// ```
    pub fn check_limit(&mut self) -> bool {
        self.refill();

        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            warn!(
                "Rate limit exceeded: no tokens available (max={}, rate={}/s)",
                self.max_tokens, self.refill_rate
            );
            false
        }
    }

    /// Refill tokens based on elapsed time.
    ///
    /// Calculates how many tokens to add based on:
    /// - Time elapsed since last refill
    /// - Refill rate (tokens per second)
    ///
    /// Tokens are capped at `max_tokens` to maintain burst limit.
    ///
    /// # Algorithm
    ///
    /// ```text
    /// elapsed_secs = (now - last_refill).as_secs_f64()
    /// new_tokens = floor(elapsed_secs * refill_rate)
    /// tokens = min(tokens + new_tokens, max_tokens)
    /// ```
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        // Calculate new tokens to add
        let new_tokens = (elapsed.as_secs_f64() * self.refill_rate as f64) as usize;

        if new_tokens > 0 {
            // Add tokens, capped at max_tokens
            self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
            self.last_refill = now;
        }
    }

    /// Get current token count.
    ///
    /// Useful for monitoring and debugging. Triggers refill before returning.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_mcp::RateLimiter;
    ///
    /// let mut limiter = RateLimiter::new(100, 50);
    /// assert_eq!(limiter.tokens(), 100); // Starts full
    /// ```
    pub fn tokens(&mut self) -> usize {
        self.refill();
        self.tokens
    }

    /// Get maximum token capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_mcp::RateLimiter;
    ///
    /// let limiter = RateLimiter::new(200, 100);
    /// assert_eq!(limiter.max_tokens(), 200);
    /// ```
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }

    /// Get refill rate (tokens per second).
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_mcp::RateLimiter;
    ///
    /// let limiter = RateLimiter::new(200, 100);
    /// assert_eq!(limiter.refill_rate(), 100);
    /// ```
    pub fn refill_rate(&self) -> usize {
        self.refill_rate
    }

    /// Reset the rate limiter to full capacity.
    ///
    /// Useful for testing or when reconfiguring the server.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_mcp::RateLimiter;
    ///
    /// let mut limiter = RateLimiter::new(100, 50);
    /// limiter.check_limit(); // Consumes one token
    /// limiter.reset();
    /// assert_eq!(limiter.tokens(), 100);
    /// ```
    pub fn reset(&mut self) {
        self.tokens = self.max_tokens;
        self.last_refill = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_new_limiter_starts_full() {
        let mut limiter = RateLimiter::new(100, 50);
        assert_eq!(limiter.tokens(), 100);
        assert_eq!(limiter.max_tokens(), 100);
        assert_eq!(limiter.refill_rate(), 50);
    }

    #[test]
    #[should_panic(expected = "max_tokens must be positive")]
    fn test_new_limiter_zero_max_tokens_panics() {
        RateLimiter::new(0, 50);
    }

    #[test]
    #[should_panic(expected = "refill_rate must be positive")]
    fn test_new_limiter_zero_refill_rate_panics() {
        RateLimiter::new(100, 0);
    }

    #[test]
    fn test_check_limit_allows_requests_when_tokens_available() {
        let mut limiter = RateLimiter::new(10, 5);

        for i in 0..10 {
            assert!(
                limiter.check_limit(),
                "Request {} should be allowed",
                i + 1
            );
        }

        assert_eq!(limiter.tokens(), 0);
    }

    #[test]
    fn test_check_limit_rejects_when_no_tokens() {
        let mut limiter = RateLimiter::new(2, 1);

        assert!(limiter.check_limit()); // 1 token left
        assert!(limiter.check_limit()); // 0 tokens left
        assert!(!limiter.check_limit()); // No tokens, rejected
        assert!(!limiter.check_limit()); // Still rejected
    }

    #[test]
    fn test_refill_adds_tokens_over_time() {
        let mut limiter = RateLimiter::new(100, 50); // 50 tokens/sec

        // Consume all tokens
        for _ in 0..100 {
            limiter.check_limit();
        }
        assert_eq!(limiter.tokens(), 0);

        // Wait for refill (100ms = 5 tokens at 50/sec)
        thread::sleep(Duration::from_millis(100));

        // Should have refilled approximately 5 tokens
        let tokens = limiter.tokens();
        assert!(tokens >= 4 && tokens <= 6, "Expected ~5 tokens, got {}", tokens);
    }

    #[test]
    fn test_refill_caps_at_max_tokens() {
        let mut limiter = RateLimiter::new(10, 100); // 100 tokens/sec, max 10

        // Wait long enough to exceed max
        thread::sleep(Duration::from_millis(200)); // Would refill 20, but capped at 10

        assert_eq!(limiter.tokens(), 10); // Capped at max
    }

    #[test]
    fn test_reset_restores_full_capacity() {
        let mut limiter = RateLimiter::new(50, 25);

        // Consume some tokens
        for _ in 0..30 {
            limiter.check_limit();
        }
        assert_eq!(limiter.tokens(), 20);

        // Reset
        limiter.reset();
        assert_eq!(limiter.tokens(), 50);
    }

    #[test]
    fn test_burst_capacity() {
        let mut limiter = RateLimiter::new(200, 100); // 100/sec sustained, 200 burst

        // Should allow burst of 200 requests
        for i in 0..200 {
            assert!(
                limiter.check_limit(),
                "Burst request {} should be allowed",
                i + 1
            );
        }

        // 201st request should be rejected
        assert!(!limiter.check_limit());
    }

    #[test]
    fn test_sustained_rate() {
        let mut limiter = RateLimiter::new(10, 100); // 100/sec sustained, 10 burst

        // Consume initial burst
        for _ in 0..10 {
            limiter.check_limit();
        }

        // Wait 100ms (should refill ~10 tokens)
        thread::sleep(Duration::from_millis(100));

        // Should allow ~10 more requests
        let mut allowed = 0;
        for _ in 0..20 {
            if limiter.check_limit() {
                allowed += 1;
            }
        }

        assert!(
            allowed >= 8 && allowed <= 12,
            "Expected ~10 requests allowed, got {}",
            allowed
        );
    }
}
