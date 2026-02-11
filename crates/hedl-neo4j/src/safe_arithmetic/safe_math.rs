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

//! Safe arithmetic operations with explicit overflow handling.
//!
//! This module provides checked arithmetic operations that prevent integer overflow
//! vulnerabilities. All operations return a `Result` with detailed error context,
//! making it easy to track down overflow issues during development and testing.
//!
//! # Why Use These Functions?
//!
//! Rust's default arithmetic behavior differs between debug and release builds:
//! - **Debug builds**: Integer overflow causes a panic
//! - **Release builds**: Integer overflow wraps silently (potential security issue)
//!
//! These functions provide consistent, safe behavior across all build modes by using
//! Rust's checked arithmetic methods (`checked_add`, `checked_mul`, etc.).
//!
//! # Security Considerations
//!
//! Integer overflow can lead to serious vulnerabilities:
//! - **Buffer underflow**: `usize::MAX + 1` wraps to 0, allocating 0 bytes
//! - **Logic errors**: Incorrect calculations in batch processing
//! - **`DoS` attacks**: Crafted input causing panics in debug mode
//!
//! These functions ensure overflow is handled explicitly and safely.

use crate::error::{Neo4jError, Result};

/// Safely add two usize values, returning an error on overflow.
///
/// # Arguments
///
/// * `a` - First addend
/// * `b` - Second addend
/// * `context` - Description of where this addition occurs (for error reporting)
///
/// # Returns
///
/// * `Ok(sum)` - The sum if no overflow occurred
/// * `Err(IntegerOverflow)` - If the addition would overflow
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::safe_arithmetic::safe_math::checked_add;
/// // Normal case
/// let result = checked_add(100, 200, "test addition").unwrap();
/// assert_eq!(result, 300);
///
/// // Overflow case
/// let result = checked_add(usize::MAX, 1, "overflow test");
/// assert!(result.is_err());
/// ```
#[inline]
pub fn checked_add(a: usize, b: usize, context: &str) -> Result<usize> {
    a.checked_add(b).ok_or_else(|| Neo4jError::IntegerOverflow {
        context: format!("addition overflow in {context}: {a} + {b}"),
    })
}

/// Safely multiply two usize values, returning an error on overflow.
///
/// # Arguments
///
/// * `a` - First factor
/// * `b` - Second factor
/// * `context` - Description of where this multiplication occurs (for error reporting)
///
/// # Returns
///
/// * `Ok(product)` - The product if no overflow occurred
/// * `Err(IntegerOverflow)` - If the multiplication would overflow
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::safe_arithmetic::safe_math::checked_mul;
/// // Normal case
/// let result = checked_mul(100, 200, "test multiplication").unwrap();
/// assert_eq!(result, 20000);
///
/// // Overflow case
/// let result = checked_mul(usize::MAX, 2, "overflow test");
/// assert!(result.is_err());
/// ```
#[inline]
pub fn checked_mul(a: usize, b: usize, context: &str) -> Result<usize> {
    a.checked_mul(b).ok_or_else(|| Neo4jError::IntegerOverflow {
        context: format!("multiplication overflow in {context}: {a} * {b}"),
    })
}

/// Calculate ceiling division without overflow risk.
///
/// This function computes `ceil(dividend / divisor)` using a safe formula that
/// avoids the traditional `(a + b - 1) / b` pattern which can overflow when
/// `a` is close to `usize::MAX`.
///
/// # Algorithm
///
/// Instead of: `(dividend + divisor - 1) / divisor` (can overflow)
/// We use: `dividend / divisor + (dividend % divisor != 0) as usize` (safe)
///
/// # Arguments
///
/// * `dividend` - The numerator
/// * `divisor` - The denominator (must be non-zero)
///
/// # Panics
///
/// Panics if `divisor` is zero (division by zero).
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::safe_arithmetic::safe_math::ceiling_div;
/// // Normal cases
/// assert_eq!(ceiling_div(10, 3), 4);  // 10 / 3 = 3.33... → 4
/// assert_eq!(ceiling_div(9, 3), 3);   // 9 / 3 = 3 → 3
/// assert_eq!(ceiling_div(0, 5), 0);   // 0 / 5 = 0 → 0
///
/// // Large values (safe from overflow)
/// let result = ceiling_div(usize::MAX, 1000);
/// assert_eq!(result, usize::MAX / 1000 + 1);
/// ```
#[inline]
#[must_use]
pub fn ceiling_div(dividend: usize, divisor: usize) -> usize {
    assert!(divisor != 0, "division by zero in ceiling_div");
    // Safe pattern: a / b + (a % b != 0) as usize
    // This avoids the overflow risk of (a + b - 1) / b
    dividend / divisor + usize::from(dividend % divisor != 0)
}

/// Safely allocate capacity for string operations.
///
/// This function uses saturating addition to ensure that string capacity
/// calculations never overflow. If the sum would exceed `usize::MAX`, it
/// returns `usize::MAX` instead of wrapping around.
///
/// # Why Saturating?
///
/// For capacity reservations, saturating is safer than checked arithmetic:
/// - If we hit `usize::MAX`, the allocation will fail gracefully with OOM
/// - The string will auto-resize if needed, just with more reallocations
/// - This is preferable to panicking or returning an error for capacity hints
///
/// # Arguments
///
/// * `base` - The base size (typically `string.len()`)
/// * `overhead` - Additional capacity to reserve
///
/// # Returns
///
/// The sum of `base + overhead`, saturating at `usize::MAX` on overflow.
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::safe_arithmetic::safe_math::safe_string_capacity;
/// // Normal case
/// let capacity = safe_string_capacity(100, 10);
/// assert_eq!(capacity, 110);
///
/// // Overflow case - saturates at MAX
/// let capacity = safe_string_capacity(usize::MAX, 10);
/// assert_eq!(capacity, usize::MAX);
/// ```
#[inline]
#[must_use]
pub fn safe_string_capacity(base: usize, overhead: usize) -> usize {
    base.saturating_add(overhead)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checked_add_normal() {
        assert_eq!(checked_add(100, 200, "test").unwrap(), 300);
        assert_eq!(checked_add(0, 0, "test").unwrap(), 0);
        assert_eq!(checked_add(1, 1, "test").unwrap(), 2);
    }

    #[test]
    fn test_checked_add_overflow() {
        let result = checked_add(usize::MAX, 1, "test");
        assert!(result.is_err());

        let result = checked_add(usize::MAX, usize::MAX, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_checked_add_error_message() {
        let result = checked_add(usize::MAX, 1, "batch calculation");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("batch calculation"));
        assert!(msg.contains("overflow"));
    }

    #[test]
    fn test_checked_mul_normal() {
        assert_eq!(checked_mul(10, 20, "test").unwrap(), 200);
        assert_eq!(checked_mul(0, 100, "test").unwrap(), 0);
        assert_eq!(checked_mul(1, 1, "test").unwrap(), 1);
    }

    #[test]
    fn test_checked_mul_overflow() {
        let result = checked_mul(usize::MAX, 2, "test");
        assert!(result.is_err());

        let result = checked_mul(usize::MAX / 2 + 1, 2, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_checked_mul_error_message() {
        let result = checked_mul(usize::MAX, 2, "capacity calculation");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("capacity calculation"));
        assert!(msg.contains("overflow"));
    }

    #[test]
    fn test_ceiling_div_normal() {
        // Exact division
        assert_eq!(ceiling_div(10, 5), 2);
        assert_eq!(ceiling_div(100, 10), 10);

        // Ceiling required
        assert_eq!(ceiling_div(10, 3), 4); // 10 / 3 = 3.33... → 4
        assert_eq!(ceiling_div(11, 3), 4); // 11 / 3 = 3.66... → 4
        assert_eq!(ceiling_div(9, 3), 3); // 9 / 3 = 3 → 3

        // Edge cases
        assert_eq!(ceiling_div(0, 5), 0);
        assert_eq!(ceiling_div(1, 1), 1);
        assert_eq!(ceiling_div(1, 2), 1); // 1 / 2 = 0.5 → 1
    }

    #[test]
    fn test_ceiling_div_large_values() {
        // Test with large values that would overflow with (a + b - 1) / b
        let result = ceiling_div(usize::MAX, 1000);
        // Should be usize::MAX / 1000 + 1 (since MAX % 1000 != 0)
        assert_eq!(result, usize::MAX / 1000 + 1);

        let result = ceiling_div(usize::MAX, usize::MAX);
        assert_eq!(result, 1);

        let result = ceiling_div(usize::MAX - 1, 2);
        // (MAX - 1) is even, so (MAX - 1) / 2 with no extra 1 needed
        assert_eq!(result, (usize::MAX - 1) / 2);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_ceiling_div_zero_divisor() {
        ceiling_div(10, 0);
    }

    #[test]
    fn test_safe_string_capacity_normal() {
        assert_eq!(safe_string_capacity(100, 10), 110);
        assert_eq!(safe_string_capacity(0, 0), 0);
        assert_eq!(safe_string_capacity(1, 1), 2);
    }

    #[test]
    fn test_safe_string_capacity_overflow() {
        // Should saturate at MAX
        assert_eq!(safe_string_capacity(usize::MAX, 10), usize::MAX);
        assert_eq!(safe_string_capacity(usize::MAX, usize::MAX), usize::MAX);
        assert_eq!(safe_string_capacity(usize::MAX - 5, 10), usize::MAX);
    }

    #[test]
    fn test_saturating_behavior() {
        // Verify that saturating_add is used correctly
        let base = usize::MAX - 5;
        let overhead = 10;
        let result = safe_string_capacity(base, overhead);
        // Should saturate at MAX, not wrap around
        assert_eq!(result, usize::MAX);
        assert!(result >= base); // Never wraps to smaller value
    }

    #[test]
    fn test_ceiling_div_matches_expected_formula() {
        // Verify our formula matches expected ceiling division
        for dividend in [0, 1, 5, 10, 99, 100, 1000] {
            for divisor in [1, 2, 3, 5, 7, 10, 100] {
                let result = ceiling_div(dividend, divisor);
                let expected = (dividend + divisor - 1) / divisor;
                assert_eq!(
                    result, expected,
                    "ceiling_div({dividend}, {divisor}) = {result} != expected {expected}"
                );
            }
        }
    }

    #[test]
    fn test_ceiling_div_properties() {
        // Mathematical properties that should hold
        for dividend in [0, 1, 5, 10, 99, 100, 1000, 10000] {
            for divisor in [1, 2, 3, 5, 7, 10, 100] {
                let result = ceiling_div(dividend, divisor);

                // Property 1: result * divisor >= dividend
                assert!(
                    result * divisor >= dividend,
                    "Property 1 failed: {result} * {divisor} >= {dividend}"
                );

                // Property 2: (result - 1) * divisor < dividend (unless result == 0)
                if result > 0 {
                    assert!(
                        (result - 1) * divisor < dividend,
                        "Property 2 failed: ({result} - 1) * {divisor} < {dividend}"
                    );
                }
            }
        }
    }
}
