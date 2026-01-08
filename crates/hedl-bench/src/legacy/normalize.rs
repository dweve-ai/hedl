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

//! Answer normalization for type-aware comparison.
//!
//! Normalizes LLM responses for deterministic grading without needing an LLM judge.
//! Handles:
//! - Currency symbols ($, €, £, ¥)
//! - Percentages (42% → 0.42)
//! - Thousand separators (1,234 → 1234)
//! - Scientific notation (1.5e-3)
//! - Boolean variations (yes/no/true/false/y/n/1/0)
//! - Quote stripping
//! - Code fence removal

use super::questions::AnswerType;
use crate::error::{BenchError, Result};

/// Normalize an answer string based on its expected type.
///
/// This function performs type-aware normalization to enable deterministic
/// comparison of LLM responses without requiring an LLM judge.
///
/// # Arguments
/// * `answer` - The raw answer string from LLM or ground truth
/// * `answer_type` - The expected type for normalization rules
///
/// # Returns
/// * `Ok(String)` - Normalized answer string
/// * `Err(BenchError)` - Error if normalization fails
///
/// # Examples
/// ```text
/// use hedl_bench::legacy::normalize::normalize;
/// use hedl_bench::legacy::questions::AnswerType;
/// // Integer normalization
/// assert_eq!(normalize("1,234", &AnswerType::Integer).unwrap(), "1234");
///
/// // Boolean normalization
/// assert_eq!(normalize("yes", &AnswerType::Boolean).unwrap(), "true");
///
/// // Number with percentage
/// let result = normalize("42%", &AnswerType::Number { decimals: 2 }).unwrap();
/// assert_eq!(result, "0.42");
/// ```
pub fn normalize(answer: &str, answer_type: &AnswerType) -> Result<String> {
    let cleaned = clean_answer(answer);

    match answer_type {
        AnswerType::String => Ok(cleaned.to_lowercase()),
        AnswerType::Integer => normalize_integer(&cleaned),
        AnswerType::Number { decimals } => normalize_number(&cleaned, *decimals),
        AnswerType::Boolean => normalize_boolean(&cleaned),
        AnswerType::Date => normalize_date(&cleaned),
        AnswerType::CsvListOrdered => normalize_csv_list(&cleaned, true),
        AnswerType::CsvListUnordered => normalize_csv_list(&cleaned, false),
    }
}

/// Compare two answers with type-aware normalization.
///
/// Normalizes both expected and actual answers according to the answer type,
/// then performs appropriate comparison (exact match for most types, tolerance
/// for floating point numbers).
///
/// # Arguments
/// * `expected` - The ground truth answer
/// * `actual` - The LLM's response
/// * `answer_type` - The type for normalization and comparison rules
///
/// # Returns
/// * `Ok(true)` - Answers match after normalization
/// * `Ok(false)` - Answers differ after normalization
/// * `Err(BenchError)` - Error if normalization or comparison fails
///
/// # Examples
/// ```text
/// use hedl_bench::legacy::normalize::compare;
/// use hedl_bench::legacy::questions::AnswerType;
/// // Integer comparison with formatting differences
/// assert!(compare("1234", "$1,234", &AnswerType::Integer).unwrap());
///
/// // Boolean comparison with variations
/// assert!(compare("true", "yes", &AnswerType::Boolean).unwrap());
///
/// // Number comparison with tolerance
/// let num_type = AnswerType::Number { decimals: 2 };
/// assert!(compare("3.14", "3.14159", &num_type).unwrap());
/// ```
pub fn compare(expected: &str, actual: &str, answer_type: &AnswerType) -> Result<bool> {
    let norm_expected = normalize(expected, answer_type)?;
    let norm_actual = normalize(actual, answer_type)?;

    match answer_type {
        AnswerType::Number { decimals: _ } => {
            // For numbers, compare with tolerance
            let exp: f64 = norm_expected
                .parse()
                .map_err(|e| BenchError::ComparisonFailed {
                    reason: format!(
                        "Failed to parse expected value '{}' as number: {}",
                        norm_expected, e
                    ),
                })?;
            let act: f64 = norm_actual
                .parse()
                .map_err(|e| BenchError::ComparisonFailed {
                    reason: format!(
                        "Failed to parse actual value '{}' as number: {}",
                        norm_actual, e
                    ),
                })?;
            Ok((exp - act).abs() < 1e-6)
        }
        _ => Ok(norm_expected == norm_actual),
    }
}

/// Clean common artifacts from LLM responses.
///
/// Removes code fences (```), wrapping quotes, and trims whitespace.
/// This preprocessing step ensures consistent formatting before type-specific
/// normalization.
///
/// # Arguments
/// * `answer` - The raw answer string from LLM
///
/// # Returns
/// Cleaned answer string with artifacts removed
///
/// # Examples
/// ```no_run
/// # fn clean_answer(s: &str) -> String { s.trim().to_string() }
/// assert_eq!(clean_answer("\"hello\""), "hello");
/// assert_eq!(clean_answer("```\n42\n```"), "42");
/// ```
fn clean_answer(answer: &str) -> String {
    let mut s = answer.trim().to_string();

    // Remove code fences
    if s.starts_with("```") {
        if let Some(end) = s.rfind("```") {
            if end > 3 {
                // Find the actual content between fences
                let start = s.find('\n').unwrap_or(3) + 1;
                s = s[start..end].trim().to_string();
            }
        }
    }

    // Remove wrapping quotes
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s = s[1..s.len() - 1].to_string();
    }

    s.trim().to_string()
}

/// Normalize integer values.
///
/// Strips currency symbols, thousand separators, and other formatting.
/// Handles both integer and floating point strings (truncating decimals).
///
/// # Arguments
/// * `s` - The string to normalize
///
/// # Returns
/// * `Ok(String)` - Normalized integer as string
/// * `Err(BenchError)` - Error if parsing fails
///
/// # Examples
/// ```no_run
/// # fn normalize_integer(s: &str) -> Result<String, hedl_bench::error::BenchError> { Ok(s.to_string()) }
/// assert_eq!(normalize_integer("1,234").unwrap(), "1234");
/// assert_eq!(normalize_integer("$1,000").unwrap(), "1000");
/// assert_eq!(normalize_integer("42.7").unwrap(), "42");
/// ```
fn normalize_integer(s: &str) -> Result<String> {
    let cleaned = remove_formatting(s);

    // Try to parse as integer
    if let Ok(n) = cleaned.parse::<i64>() {
        return Ok(n.to_string());
    }

    // Try to parse as float and truncate
    if let Ok(n) = cleaned.parse::<f64>() {
        return Ok((n as i64).to_string());
    }

    Err(BenchError::NormalizationFailed {
        value: s.to_string(),
        reason: "Cannot parse as integer".to_string(),
    })
}

/// Normalize number values with decimal precision.
///
/// # Arguments
/// * `s` - The string to normalize (may contain %, $, commas, etc.)
/// * `decimals` - Number of decimal places to round to
///
/// # Returns
/// * `Ok(String)` - Normalized number as a string with specified decimal precision
/// * `Err(BenchError)` - Error if parsing fails
///
/// # Examples
/// ```no_run
/// # fn normalize_number(s: &str, decimals: usize) -> Result<String, hedl_bench::error::BenchError> { Ok(s.to_string()) }
/// assert_eq!(normalize_number("42%", 2).unwrap(), "0.42");
/// assert_eq!(normalize_number("$1,234.56", 2).unwrap(), "1234.56");
/// assert_eq!(normalize_number("3.14159", 2).unwrap(), "3.14");
/// ```
fn normalize_number(s: &str, decimals: usize) -> Result<String> {
    // CRITICAL FIX (P0): Check for percentage BEFORE stripping formatting
        let is_percent = s.contains('%');

    let cleaned = remove_formatting(s);

    let n: f64 = cleaned
        .parse()
        .map_err(|e| BenchError::NormalizationFailed {
            value: s.to_string(),
            reason: format!("Cannot parse as number: {}", e),
        })?;

    // Convert percentage to decimal (42% -> 0.42)
    let n = if is_percent { n / 100.0 } else { n };

    // Round to specified decimals
    let factor = 10_f64.powi(decimals as i32);
    let rounded = (n * factor).round() / factor;

    Ok(format!("{:.1$}", rounded, decimals))
}

/// Normalize boolean values.
///
/// Accepts various boolean representations and normalizes to "true" or "false".
/// Case-insensitive matching.
///
/// # Arguments
/// * `s` - The string to normalize
///
/// # Returns
/// * `Ok("true")` - For yes/y/true/1/on
/// * `Ok("false")` - For no/n/false/0/off
/// * `Err(BenchError)` - Error if not a recognized boolean value
///
/// # Examples
/// ```no_run
/// # fn normalize_boolean(s: &str) -> Result<String, hedl_bench::error::BenchError> { Ok(s.to_string()) }
/// assert_eq!(normalize_boolean("yes").unwrap(), "true");
/// assert_eq!(normalize_boolean("NO").unwrap(), "false");
/// assert_eq!(normalize_boolean("1").unwrap(), "true");
/// ```
fn normalize_boolean(s: &str) -> Result<String> {
    let lower = s.to_lowercase();

    match lower.as_str() {
        "yes" | "y" | "true" | "1" | "on" => Ok("true".to_string()),
        "no" | "n" | "false" | "0" | "off" => Ok("false".to_string()),
        _ => Err(BenchError::NormalizationFailed {
            value: s.to_string(),
            reason: "Cannot parse as boolean".to_string(),
        }),
    }
}

/// Normalize date to ISO format (YYYY-MM-DD).
///
/// Accepts ISO format (YYYY-MM-DD) and US format (MM/DD/YYYY).
///
/// # Arguments
/// * `s` - The date string to normalize
///
/// # Returns
/// * `Ok(String)` - Date in ISO format (YYYY-MM-DD)
/// * `Err(BenchError)` - Error if format not recognized
///
/// # Examples
/// ```no_run
/// # fn normalize_date(s: &str) -> Result<String, hedl_bench::error::BenchError> { Ok(s.to_string()) }
/// assert_eq!(normalize_date("2024-01-15").unwrap(), "2024-01-15");
/// assert_eq!(normalize_date("01/15/2024").unwrap(), "2024-01-15");
/// ```
fn normalize_date(s: &str) -> Result<String> {
    // Already in ISO format
    if s.len() >= 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
        return Ok(s[..10].to_string());
    }

    // Try common formats
    // MM/DD/YYYY
    if s.len() >= 10 && s.chars().nth(2) == Some('/') && s.chars().nth(5) == Some('/') {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() >= 3 {
            return Ok(format!("{}-{:0>2}-{:0>2}", parts[2], parts[0], parts[1]));
        }
    }

    Err(BenchError::NormalizationFailed {
        value: s.to_string(),
        reason: "Cannot parse as date".to_string(),
    })
}

/// Normalize CSV list.
///
/// Splits on commas, trims whitespace, lowercases, and optionally sorts.
///
/// # Arguments
/// * `s` - Comma-separated list
/// * `ordered` - If false, list will be sorted for order-independent comparison
///
/// # Returns
/// * `Ok(String)` - Normalized comma-separated list
/// * `Err(BenchError)` - Error if normalization fails
///
/// # Examples
/// ```no_run
/// # fn normalize_csv_list(s: &str, ordered: bool) -> Result<String, hedl_bench::error::BenchError> { Ok(s.to_string()) }
/// assert_eq!(normalize_csv_list("a, b, c", true).unwrap(), "a,b,c");
/// assert_eq!(normalize_csv_list("c, a, b", false).unwrap(), "a,b,c");
/// ```
fn normalize_csv_list(s: &str, ordered: bool) -> Result<String> {
    let items: Vec<String> = s
        .split(',')
        .map(|item| item.trim().to_lowercase())
        .filter(|item| !item.is_empty())
        .collect();

    if ordered {
        Ok(items.join(","))
    } else {
        let mut sorted = items;
        sorted.sort();
        Ok(sorted.join(","))
    }
}

/// Remove formatting characters (currency symbols, thousand separators).
///
/// Filters string to keep only numeric characters, decimal point, sign,
/// and scientific notation (e/E). Removes percentage signs and currency symbols.
/// Used as preprocessing before parsing numeric values.
///
/// # Arguments
/// * `s` - The string to clean
///
/// # Returns
/// String with only numeric characters that can be parsed as a number
///
/// # Examples
/// ```no_run
/// # fn remove_formatting(s: &str) -> String { s.to_string() }
/// assert_eq!(remove_formatting("$1,234.56"), "1234.56");
/// assert_eq!(remove_formatting("€42%"), "42");
/// assert_eq!(remove_formatting("1,000"), "1000");
/// ```
fn remove_formatting(s: &str) -> String {
    s.chars()
        .filter(|c| {
            c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+' || *c == 'e' || *c == 'E'
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_answer_quotes() {
        assert_eq!(clean_answer("\"hello\""), "hello");
        assert_eq!(clean_answer("'world'"), "world");
    }

    #[test]
    fn test_clean_answer_code_fence() {
        let answer = "```\n42\n```";
        assert_eq!(clean_answer(answer), "42");
    }

    #[test]
    fn test_normalize_integer() {
        assert_eq!(normalize_integer("42").unwrap(), "42");
        assert_eq!(normalize_integer("1,234").unwrap(), "1234");
        assert_eq!(normalize_integer("$1,000").unwrap(), "1000");
        assert_eq!(normalize_integer("42.7").unwrap(), "42");
    }

    #[test]
    fn test_normalize_number() {
        assert_eq!(normalize_number("3.56789", 2).unwrap(), "3.57");
        assert_eq!(normalize_number("42%", 2).unwrap(), "0.42");
        assert_eq!(normalize_number("$1,234.56", 2).unwrap(), "1234.56");
    }

    #[test]
    fn test_normalize_number_percentage_fix() {
        // P0 BUG FIX VERIFICATION: Percentage normalization
    

        // Basic percentage
        assert_eq!(normalize_number("42%", 2).unwrap(), "0.42");
        assert_eq!(normalize_number("100%", 2).unwrap(), "1.00");
        assert_eq!(normalize_number("0.5%", 4).unwrap(), "0.0050");

        // Percentage with currency/formatting (should still work)
        assert_eq!(normalize_number("$42%", 2).unwrap(), "0.42");

        // Decimal percentage
        assert_eq!(normalize_number("3.14%", 3).unwrap(), "0.031");

        // Large percentage
        assert_eq!(normalize_number("250%", 2).unwrap(), "2.50");

        // Non-percentage (control cases)
        assert_eq!(normalize_number("42", 2).unwrap(), "42.00");
        assert_eq!(normalize_number("0.42", 2).unwrap(), "0.42");
    }

    #[test]
    fn test_compare_percentage() {
        // Verify percentage normalization works in compare()
        let num_type = AnswerType::Number { decimals: 2 };

        // LLM might return "42%" while ground truth is "0.42"
        assert!(compare("0.42", "42%", &num_type).unwrap());
        assert!(compare("42%", "0.42", &num_type).unwrap());

        // Or both as percentages
        assert!(compare("42%", "42%", &num_type).unwrap());
    }

    #[test]
    fn test_remove_formatting() {
        // Verify remove_formatting strips all non-numeric chars including '%'
        assert_eq!(remove_formatting("42%"), "42");
        assert_eq!(remove_formatting("$1,234.56%"), "1234.56");
        assert_eq!(remove_formatting("€100%"), "100");
        assert_eq!(remove_formatting("$1,000"), "1000");
        assert_eq!(remove_formatting("1.5e-3"), "1.5e-3");
    }

    #[test]
    fn test_normalize_boolean() {
        assert_eq!(normalize_boolean("yes").unwrap(), "true");
        assert_eq!(normalize_boolean("YES").unwrap(), "true");
        assert_eq!(normalize_boolean("no").unwrap(), "false");
        assert_eq!(normalize_boolean("false").unwrap(), "false");
        assert_eq!(normalize_boolean("1").unwrap(), "true");
        assert_eq!(normalize_boolean("0").unwrap(), "false");
    }

    #[test]
    fn test_normalize_date() {
        assert_eq!(normalize_date("2024-01-15").unwrap(), "2024-01-15");
        assert_eq!(normalize_date("01/15/2024").unwrap(), "2024-01-15");
    }

    #[test]
    fn test_normalize_csv_list() {
        assert_eq!(normalize_csv_list("a, b, c", true).unwrap(), "a,b,c");
        assert_eq!(normalize_csv_list("c, a, b", false).unwrap(), "a,b,c");
    }

    #[test]
    fn test_compare() {
        assert!(compare("42", "42", &AnswerType::Integer).unwrap());
        assert!(compare("yes", "true", &AnswerType::Boolean).unwrap());
        assert!(compare("3.5", "3.5", &AnswerType::Number { decimals: 2 }).unwrap());
    }
}
