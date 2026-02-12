// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Parsing utilities for parenthesized count hints.
//!
//! HEDL supports count hints in two contexts:
//! 1. `%STRUCT: TypeName (N): [columns]` - Struct instance count hints
//! 2. `name(N):@Type` (DEPRECATED) - List declaration count hints
//!
//! This module provides a unified parser with configurable validation rules.

use crate::error::{HedlError, HedlResult};

/// Configuration for parenthesized count parsing behavior.
///
/// Different contexts require different validation rules:
/// - Struct hints allow zero counts and validate formatting strictly
/// - List hints (deprecated) require non-zero counts and are more lenient
#[derive(Debug, Clone, Copy)]
pub struct CountParsingConfig {
    /// Whether to allow count = 0
    /// - true: Allow zero counts (structs can have 0 instances)
    /// - false: Require count > 0 (lists must have at least one row)
    pub allow_zero: bool,

    /// Whether to validate against leading zeros (e.g., "01" is invalid)
    /// - true: Reject leading zeros to prevent ambiguity
    /// - false: Accept any valid integer representation
    pub check_leading_zeros: bool,

    /// Whether to validate no trailing content after ')'
    /// - true: Reject "name(10) extra" as malformed
    /// - false: Accept and ignore trailing content
    pub check_trailing_content: bool,

    /// Whether to use rightmost '(' for parsing
    /// - true: Use rfind('(') to handle nested or multiple parentheses
    /// - false: Use find('(') for first occurrence
    pub use_rightmost_paren: bool,
}

impl CountParsingConfig {
    /// Configuration for deprecated list count hints: `teams(3):@Team`
    ///
    /// Lenient validation for backward compatibility:
    /// - Requires non-zero counts
    /// - Accepts any formatting
    /// - First parenthesis wins
    pub const LIST_HINT: Self = Self {
        allow_zero: false,
        check_leading_zeros: false,
        check_trailing_content: false,
        use_rightmost_paren: false,
    };

    /// Configuration for struct count hints: `%STRUCT: Company (10): [id, name]`
    ///
    /// Strict validation for new syntax:
    /// - Allows zero counts (no instances is valid)
    /// - Validates formatting (no leading zeros, no trailing content)
    /// - Rightmost parenthesis for robustness
    pub const STRUCT_HINT: Self = Self {
        allow_zero: true,
        check_leading_zeros: true,
        check_trailing_content: true,
        use_rightmost_paren: true,
    };
}

/// Parse a string that may contain a parenthesized count hint.
///
/// # Format
///
/// - `"name"` → `("name", None)` - No count hint
/// - `"name(5)"` → `("name", Some(5))` - Count hint present
/// - `"name (5)"` → `("name", Some(5))` - Spaces allowed (trimmed)
///
/// # Configuration
///
/// The `config` parameter controls validation behavior. Use predefined
/// constants for common cases:
/// - `CountParsingConfig::LIST_HINT` - Deprecated list syntax
/// - `CountParsingConfig::STRUCT_HINT` - Struct declaration syntax
///
/// # Errors
///
/// Returns `HedlError::Syntax` for:
/// - Unclosed parentheses: `"name(5"`
/// - Invalid count format: `"name(abc)"`
/// - Zero count (if `!config.allow_zero`): `"name(0)"`
/// - Leading zeros (if `config.check_leading_zeros`): `"name(01)"`
/// - Trailing content (if `config.check_trailing_content`): `"name(5) extra"`
///
/// # Examples
///
/// ```
/// # use hedl_core::lex::count_hint::{parse_parenthesized_count, CountParsingConfig};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Struct syntax - allows zero
/// let (name, count) = parse_parenthesized_count(
///     "Company (0)",
///     CountParsingConfig::STRUCT_HINT,
///     1
/// )?;
/// assert_eq!(name, "Company");
/// assert_eq!(count, Some(0));
///
/// // List syntax - rejects zero
/// let result = parse_parenthesized_count(
///     "teams(0)",
///     CountParsingConfig::LIST_HINT,
///     1
/// );
/// assert!(result.is_err());
/// # Ok(())
/// # }
/// ```
pub fn parse_parenthesized_count(
    input: &str,
    config: CountParsingConfig,
    line_num: usize,
) -> HedlResult<(String, Option<usize>)> {
    // Find opening parenthesis (first or rightmost based on config)
    let paren_start = if config.use_rightmost_paren {
        input.rfind('(')
    } else {
        input.find('(')
    };

    let Some(paren_start) = paren_start else {
        // No parenthesis found - return input as-is
        return Ok((input.to_string(), None));
    };

    // Extract name part (before parenthesis)
    let name_part = input[..paren_start].trim();

    // Extract content after opening parenthesis
    let after_paren = &input[paren_start + 1..];

    // Find closing parenthesis
    let paren_end = after_paren
        .find(')')
        .ok_or_else(|| HedlError::syntax("unclosed count hint parenthesis", line_num))?;

    // Extract count string (inside parentheses)
    let count_str = after_paren[..paren_end].trim();

    // Check for trailing content after ')' if configured
    if config.check_trailing_content {
        let trailing = after_paren[paren_end + 1..].trim();
        if !trailing.is_empty() {
            return Err(HedlError::syntax(
                format!("unexpected content after count: {}", trailing),
                line_num,
            ));
        }
    }

    // Parse count as usize
    let count_val: usize = count_str
        .parse()
        .map_err(|_| HedlError::syntax(format!("invalid count value: {}", count_str), line_num))?;

    // Validate against leading zeros if configured
    if config.check_leading_zeros && count_str.len() > 1 && count_str.starts_with('0') {
        return Err(HedlError::syntax(
            "leading zeros not allowed in count",
            line_num,
        ));
    }

    // Validate against zero count if configured
    if !config.allow_zero && count_val == 0 {
        return Err(HedlError::syntax(
            "count hint must be greater than zero",
            line_num,
        ));
    }

    Ok((name_part.to_string(), Some(count_val)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Basic Parsing Tests ====================

    #[test]
    fn test_no_parenthesis_returns_input() {
        let (name, count) =
            parse_parenthesized_count("simple", CountParsingConfig::STRUCT_HINT, 1).unwrap();
        assert_eq!(name, "simple");
        assert_eq!(count, None);
    }

    #[test]
    fn test_simple_count_hint() {
        let (name, count) =
            parse_parenthesized_count("name(5)", CountParsingConfig::STRUCT_HINT, 1).unwrap();
        assert_eq!(name, "name");
        assert_eq!(count, Some(5));
    }

    #[test]
    fn test_count_with_spaces() {
        let (name, count) =
            parse_parenthesized_count("name (10)", CountParsingConfig::STRUCT_HINT, 1).unwrap();
        assert_eq!(name, "name");
        assert_eq!(count, Some(10));
    }

    #[test]
    fn test_count_with_internal_spaces() {
        let (name, count) =
            parse_parenthesized_count("name( 10 )", CountParsingConfig::STRUCT_HINT, 1).unwrap();
        assert_eq!(name, "name");
        assert_eq!(count, Some(10));
    }

    // ==================== Configuration Tests ====================

    #[test]
    fn test_struct_hint_allows_zero() {
        let (name, count) =
            parse_parenthesized_count("Empty(0)", CountParsingConfig::STRUCT_HINT, 1).unwrap();
        assert_eq!(name, "Empty");
        assert_eq!(count, Some(0));
    }

    #[test]
    fn test_list_hint_rejects_zero() {
        let result = parse_parenthesized_count("teams(0)", CountParsingConfig::LIST_HINT, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("greater than zero"));
    }

    #[test]
    fn test_struct_hint_rejects_leading_zeros() {
        let result = parse_parenthesized_count("Company(01)", CountParsingConfig::STRUCT_HINT, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("leading zeros"));
    }

    #[test]
    fn test_list_hint_allows_leading_zeros() {
        // Deprecated syntax is lenient
        let (name, count) =
            parse_parenthesized_count("teams(01)", CountParsingConfig::LIST_HINT, 1).unwrap();
        assert_eq!(name, "teams");
        assert_eq!(count, Some(1));
    }

    #[test]
    fn test_struct_hint_rejects_trailing_content() {
        let result =
            parse_parenthesized_count("Company(10) extra", CountParsingConfig::STRUCT_HINT, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("unexpected content"));
    }

    #[test]
    fn test_list_hint_ignores_trailing_content() {
        // Deprecated syntax doesn't check trailing
        let (name, count) =
            parse_parenthesized_count("teams(5) extra", CountParsingConfig::LIST_HINT, 1).unwrap();
        assert_eq!(name, "teams");
        assert_eq!(count, Some(5));
    }

    // ==================== Parenthesis Position Tests ====================

    #[test]
    fn test_struct_hint_uses_rightmost_paren() {
        // Should use rightmost '(' for type names with parens
        let (name, count) =
            parse_parenthesized_count("Func(x)(10)", CountParsingConfig::STRUCT_HINT, 1).unwrap();
        assert_eq!(name, "Func(x)");
        assert_eq!(count, Some(10));
    }

    #[test]
    fn test_list_hint_uses_first_paren() {
        // Should use first '(' - parsing stops at first parenthesis
        let (name, count) =
            parse_parenthesized_count("func(5)extra(10)", CountParsingConfig::LIST_HINT, 1)
                .unwrap();
        assert_eq!(name, "func");
        assert_eq!(count, Some(5));
    }

    // ==================== Error Cases ====================

    #[test]
    fn test_unclosed_parenthesis_error() {
        let result = parse_parenthesized_count("name(10", CountParsingConfig::STRUCT_HINT, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("unclosed"));
    }

    #[test]
    fn test_invalid_number_error() {
        let result = parse_parenthesized_count("name(abc)", CountParsingConfig::STRUCT_HINT, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid count"));
    }

    #[test]
    fn test_empty_parentheses_error() {
        let result = parse_parenthesized_count("name()", CountParsingConfig::STRUCT_HINT, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid count"));
    }

    #[test]
    fn test_negative_number_error() {
        let result = parse_parenthesized_count("name(-5)", CountParsingConfig::STRUCT_HINT, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid count"));
    }

    #[test]
    fn test_overflow_number_error() {
        // usize::MAX + 1
        let overflow = format!("name({}0)", usize::MAX);
        let result = parse_parenthesized_count(&overflow, CountParsingConfig::STRUCT_HINT, 1);
        assert!(result.is_err());
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_large_valid_count() {
        let (name, count) =
            parse_parenthesized_count("name(999999)", CountParsingConfig::STRUCT_HINT, 1).unwrap();
        assert_eq!(name, "name");
        assert_eq!(count, Some(999999));
    }

    #[test]
    fn test_single_digit_count() {
        let (name, count) =
            parse_parenthesized_count("x(1)", CountParsingConfig::STRUCT_HINT, 1).unwrap();
        assert_eq!(name, "x");
        assert_eq!(count, Some(1));
    }

    #[test]
    fn test_empty_name_with_count() {
        let (name, count) =
            parse_parenthesized_count("(5)", CountParsingConfig::STRUCT_HINT, 1).unwrap();
        assert_eq!(name, "");
        assert_eq!(count, Some(5));
    }

    #[test]
    fn test_whitespace_only_name() {
        let (name, count) =
            parse_parenthesized_count("   (10)", CountParsingConfig::STRUCT_HINT, 1).unwrap();
        assert_eq!(name, "");
        assert_eq!(count, Some(10));
    }

    // ==================== Line Number Propagation ====================

    #[test]
    fn test_error_includes_line_number() {
        let result = parse_parenthesized_count("name(abc)", CountParsingConfig::STRUCT_HINT, 42);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.line, 42);
    }
}
