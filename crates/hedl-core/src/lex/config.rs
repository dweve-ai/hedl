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

//! Configuration for HEDL lexical analysis with resource limits.
//!
//! This module provides configuration options to prevent resource exhaustion
//! attacks and ensure bounded parsing behavior. Limits are fully configurable
//! and default to high values suitable for trusted input processing. For
//! untrusted input, use `LexConfig::strict()` or customize limits as needed.

/// Configuration for HEDL lexical analysis with resource limits.
///
/// These limits prevent resource exhaustion from malicious or malformed input.
/// Defaults are set high (100MB, 10K recursion, 10M fields) for trusted data
/// processing. For untrusted input, use `LexConfig::strict()`.
///
/// # Examples
///
/// ```
/// use hedl_core::lex::LexConfig;
///
/// // Use default limits (high values for trusted input)
/// let config = LexConfig::default();
/// assert_eq!(config.max_string_length(), 100 * 1024 * 1024); // 100 MB
///
/// // Use strict limits for untrusted input
/// let strict = LexConfig::strict();
/// assert_eq!(strict.max_string_length(), 64 * 1024); // 64 KB
///
/// // Customize limits for specific requirements
/// let custom = LexConfig::new()
///     .with_max_string_length(500 * 1024 * 1024) // 500 MB
///     .with_max_recursion_depth(50_000);          // 50K levels
///
/// // For maximum throughput with trusted data, use very high limits
/// let unlimited = LexConfig::new()
///     .with_max_string_length(usize::MAX)
///     .with_max_recursion_depth(usize::MAX)
///     .with_max_field_count(usize::MAX);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexConfig {
    /// Maximum length of a single string literal in bytes.
    ///
    /// Prevents memory exhaustion from extremely long strings.
    /// Default: 100 MB (104,857,600 bytes).
    max_string_length: usize,

    /// Maximum recursion depth for nested expressions.
    ///
    /// Prevents stack overflow from deeply nested `$(...)` expressions.
    /// Default: 10,000 levels.
    max_recursion_depth: usize,

    /// Maximum number of fields in a CSV row or struct.
    ///
    /// Prevents memory exhaustion from rows with excessive fields.
    /// Default: 10,000,000 fields.
    max_field_count: usize,

    /// Maximum nesting depth for parentheses in expressions.
    ///
    /// Prevents stack overflow from expressions like `$(((((...)))))`
    /// Default: 1,000 levels.
    max_paren_depth: usize,
}

impl LexConfig {
    /// Default maximum string length (100 MB).
    ///
    /// This high default supports large data processing while still preventing
    /// unbounded memory consumption. For untrusted input, use `LexConfig::strict()`.
    pub const DEFAULT_MAX_STRING_LENGTH: usize = 100 * 1024 * 1024;

    /// Default maximum recursion depth (10,000 levels).
    ///
    /// This high default supports deeply nested structures while preventing stack overflow.
    /// For untrusted input, use `LexConfig::strict()`.
    pub const DEFAULT_MAX_RECURSION_DEPTH: usize = 10_000;

    /// Default maximum field count (10 million fields).
    ///
    /// This high default supports large datasets while preventing unbounded memory.
    /// For untrusted input, use `LexConfig::strict()`.
    pub const DEFAULT_MAX_FIELD_COUNT: usize = 10_000_000;

    /// Default maximum parenthesis depth (1,000 levels).
    ///
    /// This high default supports complex expressions while preventing stack overflow.
    /// For untrusted input, use `LexConfig::strict()`.
    pub const DEFAULT_MAX_PAREN_DEPTH: usize = 1_000;

    /// Create a new configuration with default limits.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum string length in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::LexConfig;
    ///
    /// let config = LexConfig::new().with_max_string_length(10_000);
    /// assert_eq!(config.max_string_length(), 10_000);
    /// ```
    #[inline]
    pub fn with_max_string_length(mut self, max: usize) -> Self {
        self.max_string_length = max;
        self
    }

    /// Set the maximum recursion depth.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::LexConfig;
    ///
    /// let config = LexConfig::new().with_max_recursion_depth(32);
    /// assert_eq!(config.max_recursion_depth(), 32);
    /// ```
    #[inline]
    pub fn with_max_recursion_depth(mut self, max: usize) -> Self {
        self.max_recursion_depth = max;
        self
    }

    /// Set the maximum field count.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::LexConfig;
    ///
    /// let config = LexConfig::new().with_max_field_count(1000);
    /// assert_eq!(config.max_field_count(), 1000);
    /// ```
    #[inline]
    pub fn with_max_field_count(mut self, max: usize) -> Self {
        self.max_field_count = max;
        self
    }

    /// Set the maximum parenthesis depth.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::LexConfig;
    ///
    /// let config = LexConfig::new().with_max_paren_depth(32);
    /// assert_eq!(config.max_paren_depth(), 32);
    /// ```
    #[inline]
    pub fn with_max_paren_depth(mut self, max: usize) -> Self {
        self.max_paren_depth = max;
        self
    }

    /// Get the maximum string length in bytes.
    #[inline]
    pub fn max_string_length(&self) -> usize {
        self.max_string_length
    }

    /// Get the maximum recursion depth.
    #[inline]
    pub fn max_recursion_depth(&self) -> usize {
        self.max_recursion_depth
    }

    /// Get the maximum field count.
    #[inline]
    pub fn max_field_count(&self) -> usize {
        self.max_field_count
    }

    /// Get the maximum parenthesis depth.
    #[inline]
    pub fn max_paren_depth(&self) -> usize {
        self.max_paren_depth
    }

    /// Create a configuration with strict (small) limits for untrusted input.
    ///
    /// Useful for parsing untrusted data where security is paramount.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::LexConfig;
    ///
    /// let config = LexConfig::strict();
    /// assert_eq!(config.max_string_length(), 65_536); // 64 KB
    /// assert_eq!(config.max_recursion_depth(), 32);
    /// ```
    #[inline]
    pub fn strict() -> Self {
        Self {
            max_string_length: 64 * 1024,     // 64 KB
            max_recursion_depth: 32,          // 32 levels
            max_field_count: 1_000,           // 1,000 fields
            max_paren_depth: 16,              // 16 levels
        }
    }

    /// Create a configuration with permissive (large) limits for trusted input.
    ///
    /// Useful for parsing trusted data where performance matters more than security.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::LexConfig;
    ///
    /// let config = LexConfig::permissive();
    /// assert_eq!(config.max_string_length(), 104_857_600); // 100 MB
    /// ```
    #[inline]
    pub fn permissive() -> Self {
        Self {
            max_string_length: 100 * 1024 * 1024, // 100 MB
            max_recursion_depth: 1_000,            // 1,000 levels
            max_field_count: 1_000_000,            // 1 million fields
            max_paren_depth: 256,                  // 256 levels
        }
    }

    /// Check if a string length is within limits.
    #[inline]
    pub fn check_string_length(&self, length: usize) -> bool {
        length <= self.max_string_length
    }

    /// Check if a recursion depth is within limits.
    #[inline]
    pub fn check_recursion_depth(&self, depth: usize) -> bool {
        depth <= self.max_recursion_depth
    }

    /// Check if a field count is within limits.
    #[inline]
    pub fn check_field_count(&self, count: usize) -> bool {
        count <= self.max_field_count
    }

    /// Check if a parenthesis depth is within limits.
    #[inline]
    pub fn check_paren_depth(&self, depth: usize) -> bool {
        depth <= self.max_paren_depth
    }
}

impl Default for LexConfig {
    fn default() -> Self {
        Self {
            max_string_length: Self::DEFAULT_MAX_STRING_LENGTH,
            max_recursion_depth: Self::DEFAULT_MAX_RECURSION_DEPTH,
            max_field_count: Self::DEFAULT_MAX_FIELD_COUNT,
            max_paren_depth: Self::DEFAULT_MAX_PAREN_DEPTH,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Construction tests ====================

    #[test]
    fn test_config_new() {
        let config = LexConfig::new();
        assert_eq!(config.max_string_length(), LexConfig::DEFAULT_MAX_STRING_LENGTH);
        assert_eq!(config.max_recursion_depth(), LexConfig::DEFAULT_MAX_RECURSION_DEPTH);
        assert_eq!(config.max_field_count(), LexConfig::DEFAULT_MAX_FIELD_COUNT);
        assert_eq!(config.max_paren_depth(), LexConfig::DEFAULT_MAX_PAREN_DEPTH);
    }

    #[test]
    fn test_config_default() {
        let config = LexConfig::default();
        assert_eq!(config.max_string_length(), 100 * 1024 * 1024);
        assert_eq!(config.max_recursion_depth(), 10_000);
        assert_eq!(config.max_field_count(), 10_000_000);
        assert_eq!(config.max_paren_depth(), 1_000);
    }

    #[test]
    fn test_config_strict() {
        let config = LexConfig::strict();
        assert_eq!(config.max_string_length(), 64 * 1024);
        assert_eq!(config.max_recursion_depth(), 32);
        assert_eq!(config.max_field_count(), 1_000);
        assert_eq!(config.max_paren_depth(), 16);
    }

    #[test]
    fn test_config_permissive() {
        let config = LexConfig::permissive();
        assert_eq!(config.max_string_length(), 100 * 1024 * 1024);
        assert_eq!(config.max_recursion_depth(), 1_000);
        assert_eq!(config.max_field_count(), 1_000_000);
        assert_eq!(config.max_paren_depth(), 256);
    }

    // ==================== Builder pattern tests ====================

    #[test]
    fn test_with_max_string_length() {
        let config = LexConfig::new().with_max_string_length(5000);
        assert_eq!(config.max_string_length(), 5000);
    }

    #[test]
    fn test_with_max_recursion_depth() {
        let config = LexConfig::new().with_max_recursion_depth(50);
        assert_eq!(config.max_recursion_depth(), 50);
    }

    #[test]
    fn test_with_max_field_count() {
        let config = LexConfig::new().with_max_field_count(500);
        assert_eq!(config.max_field_count(), 500);
    }

    #[test]
    fn test_with_max_paren_depth() {
        let config = LexConfig::new().with_max_paren_depth(32);
        assert_eq!(config.max_paren_depth(), 32);
    }

    #[test]
    fn test_builder_chaining() {
        let config = LexConfig::new()
            .with_max_string_length(1000)
            .with_max_recursion_depth(20)
            .with_max_field_count(100)
            .with_max_paren_depth(16);

        assert_eq!(config.max_string_length(), 1000);
        assert_eq!(config.max_recursion_depth(), 20);
        assert_eq!(config.max_field_count(), 100);
        assert_eq!(config.max_paren_depth(), 16);
    }

    // ==================== Check methods tests ====================

    #[test]
    fn test_check_string_length_within_limit() {
        let config = LexConfig::new().with_max_string_length(100);
        assert!(config.check_string_length(50));
        assert!(config.check_string_length(100));
    }

    #[test]
    fn test_check_string_length_exceeds_limit() {
        let config = LexConfig::new().with_max_string_length(100);
        assert!(!config.check_string_length(101));
        assert!(!config.check_string_length(1000));
    }

    #[test]
    fn test_check_recursion_depth_within_limit() {
        let config = LexConfig::new().with_max_recursion_depth(10);
        assert!(config.check_recursion_depth(5));
        assert!(config.check_recursion_depth(10));
    }

    #[test]
    fn test_check_recursion_depth_exceeds_limit() {
        let config = LexConfig::new().with_max_recursion_depth(10);
        assert!(!config.check_recursion_depth(11));
        assert!(!config.check_recursion_depth(100));
    }

    #[test]
    fn test_check_field_count_within_limit() {
        let config = LexConfig::new().with_max_field_count(1000);
        assert!(config.check_field_count(500));
        assert!(config.check_field_count(1000));
    }

    #[test]
    fn test_check_field_count_exceeds_limit() {
        let config = LexConfig::new().with_max_field_count(1000);
        assert!(!config.check_field_count(1001));
        assert!(!config.check_field_count(10000));
    }

    #[test]
    fn test_check_paren_depth_within_limit() {
        let config = LexConfig::new().with_max_paren_depth(32);
        assert!(config.check_paren_depth(16));
        assert!(config.check_paren_depth(32));
    }

    #[test]
    fn test_check_paren_depth_exceeds_limit() {
        let config = LexConfig::new().with_max_paren_depth(32);
        assert!(!config.check_paren_depth(33));
        assert!(!config.check_paren_depth(100));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_zero_limits() {
        let config = LexConfig::new()
            .with_max_string_length(0)
            .with_max_recursion_depth(0)
            .with_max_field_count(0)
            .with_max_paren_depth(0);

        assert_eq!(config.max_string_length(), 0);
        assert_eq!(config.max_recursion_depth(), 0);
        assert_eq!(config.max_field_count(), 0);
        assert_eq!(config.max_paren_depth(), 0);

        // Zero limits reject everything
        assert!(!config.check_string_length(1));
        assert!(!config.check_recursion_depth(1));
        assert!(!config.check_field_count(1));
        assert!(!config.check_paren_depth(1));
    }

    #[test]
    fn test_max_usize_limits() {
        let config = LexConfig::new()
            .with_max_string_length(usize::MAX)
            .with_max_recursion_depth(usize::MAX)
            .with_max_field_count(usize::MAX)
            .with_max_paren_depth(usize::MAX);

        assert_eq!(config.max_string_length(), usize::MAX);
        assert_eq!(config.max_recursion_depth(), usize::MAX);
        assert_eq!(config.max_field_count(), usize::MAX);
        assert_eq!(config.max_paren_depth(), usize::MAX);

        // Max limits accept everything (except usize::MAX + 1, which overflows)
        assert!(config.check_string_length(usize::MAX));
        assert!(config.check_recursion_depth(usize::MAX));
        assert!(config.check_field_count(usize::MAX));
        assert!(config.check_paren_depth(usize::MAX));
    }

    #[test]
    fn test_check_boundary_values() {
        let config = LexConfig::new().with_max_string_length(100);
        assert!(config.check_string_length(0));
        assert!(config.check_string_length(100));
        assert!(!config.check_string_length(101));
    }

    // ==================== Equality and clone tests ====================

    #[test]
    fn test_config_equality() {
        let a = LexConfig::new();
        let b = LexConfig::new();
        assert_eq!(a, b);

        let c = LexConfig::new().with_max_string_length(5000);
        assert_ne!(a, c);
    }

    #[test]
    fn test_config_clone() {
        let original = LexConfig::new().with_max_string_length(5000);
        let cloned = original;
        assert_eq!(original, cloned);
    }

    // ==================== Debug and display ====================

    #[test]
    fn test_config_debug() {
        let config = LexConfig::new();
        let debug = format!("{:?}", config);
        assert!(debug.contains("LexConfig"));
    }

    // ==================== Preset comparison ====================

    #[test]
    fn test_strict_vs_default() {
        let strict = LexConfig::strict();
        let default = LexConfig::default();

        assert!(strict.max_string_length() < default.max_string_length());
        assert!(strict.max_recursion_depth() < default.max_recursion_depth());
        assert!(strict.max_field_count() < default.max_field_count());
        assert!(strict.max_paren_depth() < default.max_paren_depth());
    }

    #[test]
    fn test_permissive_vs_default() {
        let permissive = LexConfig::permissive();
        let default = LexConfig::default();

        assert!(permissive.max_string_length() == default.max_string_length());
        assert!(permissive.max_recursion_depth() < default.max_recursion_depth());
        assert!(permissive.max_field_count() < default.max_field_count());
        assert!(permissive.max_paren_depth() < default.max_paren_depth());
    }

    // ==================== Custom high limit configuration ====================

    #[test]
    fn test_custom_high_limits() {
        // Demonstrate configuring very high limits for trusted data
        let config = LexConfig::new()
            .with_max_string_length(500 * 1024 * 1024) // 500 MB
            .with_max_recursion_depth(50_000)           // 50K levels
            .with_max_field_count(100_000_000)          // 100M fields
            .with_max_paren_depth(10_000);              // 10K levels

        assert_eq!(config.max_string_length(), 500 * 1024 * 1024);
        assert_eq!(config.max_recursion_depth(), 50_000);
        assert_eq!(config.max_field_count(), 100_000_000);
        assert_eq!(config.max_paren_depth(), 10_000);

        // Verify these high limits work correctly
        assert!(config.check_string_length(500 * 1024 * 1024));
        assert!(config.check_recursion_depth(50_000));
        assert!(config.check_field_count(100_000_000));
        assert!(config.check_paren_depth(10_000));
    }

    #[test]
    fn test_unlimited_config() {
        // Demonstrate setting unlimited (max) values
        let config = LexConfig::new()
            .with_max_string_length(usize::MAX)
            .with_max_recursion_depth(usize::MAX)
            .with_max_field_count(usize::MAX)
            .with_max_paren_depth(usize::MAX);

        // Verify unlimited config accepts any value
        assert!(config.check_string_length(usize::MAX));
        assert!(config.check_recursion_depth(usize::MAX));
        assert!(config.check_field_count(usize::MAX));
        assert!(config.check_paren_depth(usize::MAX));

        // Even very large values are accepted
        assert!(config.check_string_length(1_000_000_000));
        assert!(config.check_recursion_depth(1_000_000));
    }

    #[test]
    fn test_use_case_large_datasets() {
        // Configuration for processing large trusted datasets
        let config = LexConfig::new()
            .with_max_string_length(1024 * 1024 * 1024) // 1 GB strings
            .with_max_field_count(50_000_000);           // 50M fields

        assert!(config.check_string_length(500 * 1024 * 1024)); // 500 MB ok
        assert!(config.check_field_count(30_000_000));           // 30M ok
    }

    #[test]
    fn test_use_case_deep_nesting() {
        // Configuration for deeply nested trusted structures
        let config = LexConfig::new()
            .with_max_recursion_depth(100_000)  // 100K levels
            .with_max_paren_depth(100_000);     // 100K parens

        assert!(config.check_recursion_depth(75_000));
        assert!(config.check_paren_depth(75_000));
    }
}
