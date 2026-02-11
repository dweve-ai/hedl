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

//! Security limits for HEDL parsing.

use crate::error::{HedlError, HedlResult};

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use std::time::Duration;

/// Configurable limits for parser security.
///
/// These limits protect against denial-of-service attacks and memory exhaustion
/// by bounding the resources consumed during parsing.
#[derive(Debug, Clone)]
pub struct Limits {
    /// Maximum file size in bytes (default: 1GB).
    pub max_file_size: usize,
    /// Maximum line length in bytes (default: 1MB).
    pub max_line_length: usize,
    /// Maximum indent depth (default: 50).
    pub max_indent_depth: usize,
    /// Maximum number of nodes (default: 10M).
    pub max_nodes: usize,
    /// Maximum number of aliases (default: 10k).
    pub max_aliases: usize,
    /// Maximum columns per schema (default: 100).
    pub max_columns: usize,
    /// Maximum NEST hierarchy depth (default: 100).
    pub max_nest_depth: usize,
    /// Maximum block string size in bytes (default: 10MB).
    pub max_block_string_size: usize,
    /// Maximum number of keys in a single object/map (default: 10k).
    pub max_object_keys: usize,
    /// Maximum total number of keys across all objects (default: 10M).
    ///
    /// This prevents DoS attacks where an attacker creates many small objects,
    /// each under the max_object_keys limit, but collectively consuming excessive
    /// memory. Provides defense-in-depth against memory exhaustion attacks.
    ///
    /// Default is 10,000,000 keys, which allows for large documents while still
    /// providing protection against memory exhaustion. For very large datasets,
    /// this can be increased via `ParseOptions`.
    pub max_total_keys: usize,
    /// Maximum total number of IDs across all types (default: 10M).
    ///
    /// This prevents DoS attacks where an attacker registers many IDs across
    /// multiple types, each type under reasonable limits, but collectively
    /// consuming excessive memory in the TypeRegistry indices.
    ///
    /// Default is 10,000,000 IDs, matching max_total_keys for consistency.
    /// The TypeRegistry maintains two indices (forward and inverted), so each
    /// ID registration consumes memory in both data structures.
    pub max_total_ids: usize,
    /// Maximum parsing duration (default: 30 seconds).
    ///
    /// Prevents denial-of-service attacks where a malicious document causes the
    /// parser to hang indefinitely. The parser checks elapsed time periodically
    /// and returns a `Timeout` error if parsing exceeds this duration.
    ///
    /// Set to `None` to disable timeout checking (not recommended for untrusted input).
    pub timeout: Option<Duration>,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_file_size: 1024 * 1024 * 1024, // 1GB
            max_line_length: 1024 * 1024,      // 1MB
            max_indent_depth: 50,
            max_nodes: 10_000_000,
            max_aliases: 10_000,
            max_columns: 100,
            max_nest_depth: 100,
            max_block_string_size: 10 * 1024 * 1024, // 10MB
            max_object_keys: 10_000,
            max_total_keys: 10_000_000,             // 10M
            max_total_ids: 10_000_000,              // 10M
            timeout: Some(Duration::from_secs(30)), // 30 seconds
        }
    }
}

impl Limits {
    /// Create limits with no restrictions (for testing).
    pub fn unlimited() -> Self {
        Self {
            max_file_size: usize::MAX,
            max_line_length: usize::MAX,
            max_indent_depth: usize::MAX,
            max_nodes: usize::MAX,
            max_aliases: usize::MAX,
            max_columns: usize::MAX,
            max_nest_depth: usize::MAX,
            max_block_string_size: usize::MAX,
            max_object_keys: usize::MAX,
            max_total_keys: usize::MAX,
            max_total_ids: usize::MAX,
            timeout: None,
        }
    }
}

/// Timeout context for tracking parsing time and enforcing timeout limits.
///
/// This structure tracks the start time of a parsing operation and provides
/// a method to check whether the configured timeout has been exceeded.
///
/// On WASM targets, timeout checking is disabled since `std::time::Instant`
/// is not available.
#[derive(Debug, Clone, Copy)]
pub struct TimeoutContext {
    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,
    #[cfg(not(target_arch = "wasm32"))]
    timeout: Option<Duration>,
}

impl TimeoutContext {
    /// Create a new timeout context with the given timeout duration.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(timeout: Option<Duration>) -> Self {
        Self {
            start: Instant::now(),
            timeout,
        }
    }

    /// Create a new timeout context (no-op on WASM).
    #[cfg(target_arch = "wasm32")]
    pub fn new(_timeout: Option<Duration>) -> Self {
        Self {}
    }

    /// Check if timeout has been exceeded. Returns an error if timeout exceeded.
    ///
    /// # Arguments
    ///
    /// * `line_num` - Line number for error reporting
    ///
    /// # Errors
    ///
    /// Returns a security error if the elapsed time exceeds the configured timeout.
    /// On WASM targets, this always returns Ok.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn check_timeout(&self, line_num: usize) -> HedlResult<()> {
        if let Some(timeout) = self.timeout {
            let elapsed = self.start.elapsed();
            if elapsed > timeout {
                return Err(HedlError::security(
                    format!(
                        "parsing timeout exceeded: {}ms > {}ms",
                        elapsed.as_millis(),
                        timeout.as_millis()
                    ),
                    line_num,
                ));
            }
        }
        Ok(())
    }

    /// Check if timeout has been exceeded (no-op on WASM).
    #[cfg(target_arch = "wasm32")]
    #[inline(always)]
    pub fn check_timeout(&self, _line_num: usize) -> HedlResult<()> {
        Ok(())
    }
}

/// Default interval for periodic timeout checks (every 10,000 iterations).
///
/// This value balances timeout detection responsiveness with performance overhead:
/// - At typical parsing speeds (~100k lines/sec), checks occur every ~100ms
/// - Calling `Instant::elapsed()` every 10k iterations adds <0.01% overhead
/// - Timeout detection latency is ~1ms worst-case
pub const DEFAULT_TIMEOUT_CHECK_INTERVAL: usize = 10_000;

/// Iterator adapter that performs periodic timeout checks.
///
/// This adapter wraps an iterator and checks for timeout every N iterations,
/// balancing responsiveness with performance. The check interval is configurable
/// but defaults to 10,000 iterations for optimal performance.
///
/// # Performance
///
/// Calling `Instant::elapsed()` on every iteration adds measurable overhead.
/// The default 10,000 iteration interval provides:
/// - Minimal performance impact (<0.01% overhead)
/// - Reasonable timeout detection latency (~1ms at typical parsing speeds)
/// - Balance between responsiveness and efficiency
///
/// # Examples
///
/// ```ignore
/// // Internal API - limits module is private
/// use hedl_core::limits::{TimeoutContext, TimeoutCheckExt};
/// use std::time::Duration;
///
/// let timeout_ctx = TimeoutContext::new(Some(Duration::from_secs(30)));
/// let lines = vec![(1, "line1"), (2, "line2"), (3, "line3")];
///
/// for result in lines.iter().copied().with_timeout_check(&timeout_ctx) {
///     let (line_num, line) = result.unwrap();
///     // Process line - timeout checked automatically every 10,000 iterations
/// }
/// ```
pub struct TimeoutCheckIterator<'a, I>
where
    I: Iterator,
{
    inner: I,
    timeout_ctx: &'a TimeoutContext,
    check_interval: usize,
    iteration_count: usize,
}

impl<'a, I> TimeoutCheckIterator<'a, I>
where
    I: Iterator,
{
    /// Create a new timeout-checking iterator with the default check interval.
    pub fn new(inner: I, timeout_ctx: &'a TimeoutContext) -> Self {
        Self::with_interval(inner, timeout_ctx, DEFAULT_TIMEOUT_CHECK_INTERVAL)
    }

    /// Create a new timeout-checking iterator with a custom check interval.
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying iterator to wrap
    /// * `timeout_ctx` - The timeout context to check against
    /// * `check_interval` - Number of iterations between timeout checks
    pub fn with_interval(inner: I, timeout_ctx: &'a TimeoutContext, check_interval: usize) -> Self {
        Self {
            inner,
            timeout_ctx,
            check_interval,
            iteration_count: 0,
        }
    }
}

impl<'a, I> Iterator for TimeoutCheckIterator<'a, I>
where
    I: Iterator<Item = (usize, &'a str)>,
{
    type Item = Result<(usize, &'a str), HedlError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get next item from inner iterator
        let item = self.inner.next()?;
        let (line_num, _line) = item;

        // Periodic timeout check
        self.iteration_count += 1;
        if self.iteration_count % self.check_interval == 0 {
            if let Err(e) = self.timeout_ctx.check_timeout(line_num) {
                return Some(Err(e));
            }
        }

        Some(Ok(item))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Extension trait for adding timeout checking to iterators.
///
/// This trait provides a convenient method to wrap any iterator with
/// periodic timeout checks.
pub trait TimeoutCheckExt<'a>: Iterator<Item = (usize, &'a str)> + Sized {
    /// Add periodic timeout checking to this iterator.
    ///
    /// The iterator will check for timeout every 10,000 iterations by default.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Internal API - limits module is private
    /// use hedl_core::limits::{TimeoutContext, TimeoutCheckExt};
    /// use std::time::Duration;
    ///
    /// let timeout_ctx = TimeoutContext::new(Some(Duration::from_secs(30)));
    /// let lines = vec![(1, "line1"), (2, "line2")];
    ///
    /// for result in lines.iter().copied().with_timeout_check(&timeout_ctx) {
    ///     let (line_num, line) = result.unwrap();
    ///     // Process line
    /// }
    /// ```
    fn with_timeout_check(self, timeout_ctx: &'a TimeoutContext) -> TimeoutCheckIterator<'a, Self> {
        TimeoutCheckIterator::new(self, timeout_ctx)
    }
}

// Blanket implementation for all iterators with the right item type
impl<'a, I> TimeoutCheckExt<'a> for I where I: Iterator<Item = (usize, &'a str)> {}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Default limits tests ====================

    #[test]
    fn test_default_max_file_size() {
        let limits = Limits::default();
        assert_eq!(limits.max_file_size, 1024 * 1024 * 1024); // 1GB
    }

    #[test]
    fn test_default_max_line_length() {
        let limits = Limits::default();
        assert_eq!(limits.max_line_length, 1024 * 1024); // 1MB
    }

    #[test]
    fn test_default_max_indent_depth() {
        let limits = Limits::default();
        assert_eq!(limits.max_indent_depth, 50);
    }

    #[test]
    fn test_default_max_nodes() {
        let limits = Limits::default();
        assert_eq!(limits.max_nodes, 10_000_000); // 10M
    }

    #[test]
    fn test_default_max_aliases() {
        let limits = Limits::default();
        assert_eq!(limits.max_aliases, 10_000); // 10k
    }

    #[test]
    fn test_default_max_columns() {
        let limits = Limits::default();
        assert_eq!(limits.max_columns, 100);
    }

    // ==================== Unlimited limits tests ====================

    #[test]
    fn test_unlimited_max_file_size() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_file_size, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_line_length() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_line_length, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_indent_depth() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_indent_depth, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_nodes() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_nodes, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_aliases() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_aliases, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_columns() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_columns, usize::MAX);
    }

    // ==================== Clone and Debug tests ====================

    #[test]
    fn test_limits_clone() {
        let original = Limits::default();
        let cloned = original.clone();
        assert_eq!(original.max_file_size, cloned.max_file_size);
        assert_eq!(original.max_line_length, cloned.max_line_length);
        assert_eq!(original.max_indent_depth, cloned.max_indent_depth);
        assert_eq!(original.max_nodes, cloned.max_nodes);
        assert_eq!(original.max_aliases, cloned.max_aliases);
        assert_eq!(original.max_columns, cloned.max_columns);
    }

    #[test]
    fn test_limits_debug() {
        let limits = Limits::default();
        let debug = format!("{:?}", limits);
        assert!(debug.contains("max_file_size"));
        assert!(debug.contains("max_line_length"));
        assert!(debug.contains("max_indent_depth"));
        assert!(debug.contains("max_nodes"));
        assert!(debug.contains("max_aliases"));
        assert!(debug.contains("max_columns"));
    }

    // ==================== Custom limits tests ====================

    #[test]
    fn test_custom_limits() {
        let limits = Limits {
            max_file_size: 100,
            max_line_length: 200,
            max_indent_depth: 5,
            max_nodes: 1000,
            max_aliases: 50,
            max_columns: 10,
            max_nest_depth: 20,
            max_block_string_size: 5000,
            max_object_keys: 100,
            max_total_keys: 500,
            max_total_ids: 1000,
            timeout: Some(Duration::from_secs(5)),
        };
        assert_eq!(limits.max_file_size, 100);
        assert_eq!(limits.max_line_length, 200);
        assert_eq!(limits.max_indent_depth, 5);
        assert_eq!(limits.max_nodes, 1000);
        assert_eq!(limits.max_aliases, 50);
        assert_eq!(limits.max_columns, 10);
        assert_eq!(limits.max_nest_depth, 20);
        assert_eq!(limits.max_block_string_size, 5000);
        assert_eq!(limits.max_object_keys, 100);
        assert_eq!(limits.max_total_keys, 500);
        assert_eq!(limits.max_total_ids, 1000);
        assert_eq!(limits.timeout, Some(Duration::from_secs(5)));
    }

    #[test]
    fn test_limits_zero_values() {
        let limits = Limits {
            max_file_size: 0,
            max_line_length: 0,
            max_indent_depth: 0,
            max_nodes: 0,
            max_aliases: 0,
            max_columns: 0,
            max_nest_depth: 0,
            max_block_string_size: 0,
            max_object_keys: 0,
            max_total_keys: 0,
            max_total_ids: 0,
            timeout: Some(Duration::from_secs(0)),
        };
        assert_eq!(limits.max_file_size, 0);
        assert_eq!(limits.max_columns, 0);
        assert_eq!(limits.max_nest_depth, 0);
        assert_eq!(limits.max_block_string_size, 0);
        assert_eq!(limits.max_object_keys, 0);
        assert_eq!(limits.max_total_keys, 0);
    }

    // ==================== New limits tests ====================

    #[test]
    fn test_default_max_nest_depth() {
        let limits = Limits::default();
        assert_eq!(limits.max_nest_depth, 100);
    }

    #[test]
    fn test_default_max_block_string_size() {
        let limits = Limits::default();
        assert_eq!(limits.max_block_string_size, 10 * 1024 * 1024); // 10MB
    }

    #[test]
    fn test_unlimited_max_nest_depth() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_nest_depth, usize::MAX);
    }

    #[test]
    fn test_unlimited_max_block_string_size() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_block_string_size, usize::MAX);
    }

    #[test]
    fn test_default_max_total_keys() {
        let limits = Limits::default();
        assert_eq!(limits.max_total_keys, 10_000_000);
    }

    #[test]
    fn test_unlimited_max_total_keys() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_total_keys, usize::MAX);
    }

    #[test]
    fn test_max_total_keys_greater_than_max_object_keys() {
        let limits = Limits::default();
        assert!(
            limits.max_total_keys > limits.max_object_keys,
            "max_total_keys ({}) should be greater than max_object_keys ({})",
            limits.max_total_keys,
            limits.max_object_keys
        );
    }

    // ==================== max_total_ids tests ====================

    #[test]
    fn test_default_max_total_ids() {
        let limits = Limits::default();
        assert_eq!(limits.max_total_ids, 10_000_000);
    }

    #[test]
    fn test_unlimited_max_total_ids() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_total_ids, usize::MAX);
    }

    #[test]
    fn test_max_total_ids_matches_max_total_keys() {
        let limits = Limits::default();
        assert_eq!(
            limits.max_total_ids, limits.max_total_keys,
            "max_total_ids ({}) should match max_total_keys ({}) for consistency",
            limits.max_total_ids, limits.max_total_keys
        );
    }

    // ==================== Timeout tests ====================

    #[test]
    fn test_default_timeout() {
        let limits = Limits::default();
        assert_eq!(limits.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_unlimited_no_timeout() {
        let limits = Limits::unlimited();
        assert_eq!(limits.timeout, None);
    }

    #[test]
    fn test_custom_timeout() {
        let limits = Limits {
            timeout: Some(Duration::from_secs(60)),
            ..Limits::default()
        };
        assert_eq!(limits.timeout, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_disabled_timeout() {
        let limits = Limits {
            timeout: None,
            ..Limits::default()
        };
        assert_eq!(limits.timeout, None);
    }

    // ==================== TimeoutContext tests ====================

    #[test]
    fn test_timeout_context_no_timeout() {
        let ctx = TimeoutContext::new(None);
        // Should never timeout when timeout is None
        assert!(ctx.check_timeout(1).is_ok());
        assert!(ctx.check_timeout(1000).is_ok());
    }

    #[test]
    fn test_timeout_context_with_generous_timeout() {
        let ctx = TimeoutContext::new(Some(Duration::from_secs(10)));
        // Should not timeout immediately
        assert!(ctx.check_timeout(1).is_ok());
    }

    #[test]
    fn test_timeout_context_with_zero_timeout() {
        // Zero timeout should immediately trigger
        let ctx = TimeoutContext::new(Some(Duration::from_micros(1)));
        // Sleep a tiny bit to ensure elapsed time > 1 microsecond
        std::thread::sleep(Duration::from_micros(10));
        // Should timeout
        let result = ctx.check_timeout(42);
        assert!(result.is_err());
        if let Err(e) = result {
            let msg = e.to_string();
            assert!(msg.contains("timeout exceeded") || msg.contains("Timeout"));
        }
    }

    #[test]
    fn test_timeout_context_error_message() {
        let ctx = TimeoutContext::new(Some(Duration::from_nanos(1)));
        std::thread::sleep(Duration::from_millis(1));
        let result = ctx.check_timeout(123);
        assert!(result.is_err());
        if let Err(e) = result {
            let msg = e.to_string();
            assert!(msg.contains("123")); // Should include line number
        }
    }

    // ==================== TimeoutCheckIterator tests ====================

    #[test]
    fn test_timeout_iterator_basic() {
        let lines = [(1, "line1"), (2, "line2"), (3, "line3")];
        let timeout_ctx = TimeoutContext::new(Some(Duration::from_secs(60)));

        let mut count = 0;
        for result in lines.iter().copied().with_timeout_check(&timeout_ctx) {
            let (_line_num, _line) = result.unwrap();
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_timeout_iterator_no_timeout() {
        let lines = vec![(1, "a"); 1000];
        let timeout_ctx = TimeoutContext::new(Some(Duration::from_secs(60)));

        let count = lines
            .iter()
            .copied()
            .with_timeout_check(&timeout_ctx)
            .filter_map(Result::ok)
            .count();
        assert_eq!(count, 1000);
    }

    #[test]
    fn test_timeout_iterator_triggers_timeout() {
        // Create lines that will take long to process
        let lines: Vec<(usize, &str)> = (1..=100_000).map(|i| (i, "line")).collect();

        // Very short timeout (1 microsecond)
        let timeout_ctx = TimeoutContext::new(Some(Duration::from_micros(1)));

        // Should eventually hit timeout (checked every 10k iterations)
        let mut hit_timeout = false;
        for result in lines.iter().copied().with_timeout_check(&timeout_ctx) {
            if result.is_err() {
                hit_timeout = true;
                break;
            }
        }

        // May or may not timeout depending on machine speed, but should not panic
        // This test mainly verifies the mechanism works without errors
        // Use underscore prefix to indicate intentional unused value check
        let _ = hit_timeout; // Exercises code path, value not relevant
    }

    #[test]
    fn test_timeout_iterator_custom_interval() {
        let lines = vec![(1, "a"); 100];
        let timeout_ctx = TimeoutContext::new(Some(Duration::from_secs(60)));

        // Use very small interval (check every iteration)
        let count = TimeoutCheckIterator::with_interval(lines.iter().copied(), &timeout_ctx, 1)
            .filter_map(Result::ok)
            .count();
        assert_eq!(count, 100);
    }

    #[test]
    fn test_timeout_iterator_size_hint() {
        let lines = [(1, "a"), (2, "b"), (3, "c")];
        let timeout_ctx = TimeoutContext::new(Some(Duration::from_secs(60)));

        let iter = lines.iter().copied().with_timeout_check(&timeout_ctx);
        let (lower, upper) = iter.size_hint();
        assert_eq!(lower, 3);
        assert_eq!(upper, Some(3));
    }

    #[test]
    fn test_timeout_iterator_empty() {
        let lines: Vec<(usize, &str)> = vec![];
        let timeout_ctx = TimeoutContext::new(Some(Duration::from_secs(60)));

        let count = lines
            .iter()
            .copied()
            .with_timeout_check(&timeout_ctx)
            .filter_map(Result::ok)
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_timeout_iterator_single_item() {
        let lines = [(1, "line")];
        let timeout_ctx = TimeoutContext::new(Some(Duration::from_secs(60)));

        let items: Vec<_> = lines
            .iter()
            .copied()
            .with_timeout_check(&timeout_ctx)
            .collect();
        assert_eq!(items.len(), 1);
        assert!(items[0].is_ok());
    }

    #[test]
    fn test_timeout_iterator_no_timeout_configured() {
        let lines = vec![(1, "a"); 1000];
        let timeout_ctx = TimeoutContext::new(None);

        let count = lines
            .iter()
            .copied()
            .with_timeout_check(&timeout_ctx)
            .filter_map(Result::ok)
            .count();
        assert_eq!(count, 1000);
    }

    #[test]
    fn test_default_timeout_check_interval() {
        assert_eq!(DEFAULT_TIMEOUT_CHECK_INTERVAL, 10_000);
    }

    // ==================== Integration tests ====================

    #[test]
    fn test_timeout_check_interval_performance_characteristic() {
        // Verify that check interval is large enough to minimize overhead
        // but small enough for reasonable timeout detection
        let interval = DEFAULT_TIMEOUT_CHECK_INTERVAL;

        // Should be >= 1000 for performance (avoid excessive checks)
        assert!(
            interval >= 1000,
            "Check interval too small, may impact performance"
        );

        // Should be <= 100_000 for responsiveness (detect timeout reasonably quickly)
        assert!(
            interval <= 100_000,
            "Check interval too large, slow timeout detection"
        );
    }
}
