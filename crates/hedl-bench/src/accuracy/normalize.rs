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
//! - JSON object comparison
//! - Multiple valid answers
//! - Numeric ranges

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
/// ```no_run
/// use hedl_bench::accuracy::normalize::normalize;
/// use hedl_bench::accuracy::questions::AnswerType;
///
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
        AnswerType::String => Ok(normalize_string(&cleaned)),
        AnswerType::StringCaseSensitive => Ok(cleaned),
        AnswerType::Integer => normalize_integer(&cleaned),
        AnswerType::Number { decimals } => normalize_number(&cleaned, *decimals),
        AnswerType::Boolean => normalize_boolean(&cleaned),
        AnswerType::Date => normalize_date(&cleaned),
        AnswerType::DateTime => normalize_datetime(&cleaned),
        AnswerType::ListOrdered => normalize_list(&cleaned, true),
        AnswerType::ListUnordered => normalize_list(&cleaned, false),
        AnswerType::JsonObject => normalize_json_object(&cleaned),
        AnswerType::NumericRange { min, max } => normalize_numeric_range(&cleaned, *min, *max),
        AnswerType::MultipleValid(valid_answers) => {
            normalize_multiple_valid(&cleaned, valid_answers)
        }
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
/// ```no_run
/// use hedl_bench::accuracy::normalize::compare;
/// use hedl_bench::accuracy::questions::AnswerType;
///
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
        AnswerType::Number { decimals } => {
            // For numbers, compare with tolerance based on decimal precision
            let exp: f64 = norm_expected
                .parse()
                .map_err(|e| BenchError::ComparisonFailed {
                    reason: format!(
                        "Failed to parse expected value '{norm_expected}' as number: {e}"
                    ),
                })?;
            let act: f64 = norm_actual
                .parse()
                .map_err(|e| BenchError::ComparisonFailed {
                    reason: format!("Failed to parse actual value '{norm_actual}' as number: {e}"),
                })?;

            // Use relative tolerance (0.5% of expected value) or absolute tolerance
            // based on decimal precision, whichever is larger
            let abs_tolerance = 0.5 / 10_f64.powi(*decimals as i32);
            let rel_tolerance = exp.abs() * 0.005; // 0.5% relative
            let tolerance = abs_tolerance.max(rel_tolerance).max(1e-6);

            Ok((exp - act).abs() < tolerance)
        }
        AnswerType::NumericRange { min, max } => {
            // For numeric ranges, check if value is within bounds
            let val: f64 = norm_actual
                .parse()
                .map_err(|e| BenchError::ComparisonFailed {
                    reason: format!("Failed to parse actual value '{norm_actual}' as number: {e}"),
                })?;
            Ok(val >= *min && val <= *max)
        }
        AnswerType::MultipleValid(_) => {
            // For multiple valid answers, the normalized actual should be "valid"
            Ok(norm_actual == "valid")
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

    // Extract answer from verbose reasoning output
    // Look for patterns like "ANSWER: 42", "**Answer:** 42", "**Answer**: 42", "The answer is: 42"
    // First strip markdown bold markers to simplify pattern matching
    let stripped = s.replace("**", "");
    let lower = stripped.to_lowercase();

    // Try multiple answer patterns in order of specificity
    let answer_pos = lower
        .find("answer:")
        .or_else(|| lower.find("answer is:"))
        .or_else(|| lower.find("answer is\n"));

    if let Some(pos) = answer_pos {
        // Find where the actual answer starts (after the marker)
        let marker_end = if lower[pos..].starts_with("answer:") {
            pos + 7
        } else if lower[pos..].starts_with("answer is:") {
            pos + 10
        } else {
            pos + 9 // "answer is\n"
        };

        // Use stripped string since pos is relative to it
        let after_marker = &stripped[marker_end..];
        // Take first line after marker, strip formatting
        let extracted = after_marker
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .trim_start_matches('*')
            .trim_end_matches('*')
            .trim_start_matches('`')
            .trim_end_matches('`')
            .trim();
        if !extracted.is_empty() {
            s = extracted.to_string();
        }
    } else if s.lines().count() > 3 && s.len() > 100 {
        // For long multi-line responses without ANSWER marker,
        // try the last non-empty line as it often contains the final answer
        if let Some(last_line) = s.lines().rev().find(|l| !l.trim().is_empty()) {
            let last = last_line.trim();
            // Only use last line if it looks like an answer (short, not a sentence fragment)
            if last.len() < 100 && !last.ends_with("...") && !last.starts_with("- ") {
                s = last.to_string();
            }
        }
    }

    // Remove wrapping quotes
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s = s[1..s.len() - 1].to_string();
    }

    // Remove wrapping backticks
    if s.starts_with('`') && s.ends_with('`') {
        s = s[1..s.len() - 1].to_string();
    }

    // Remove markdown bold markers
    s = s
        .trim_start_matches("**")
        .trim_end_matches("**")
        .to_string();

    s.trim().to_string()
}

/// Normalize string values for comparison.
///
/// Handles case-insensitive comparison and normalizes whitespace around
/// brackets and commas for array-like strings. Also handles bare arrays
/// (e.g., `3,3` or `"3;3"` becomes `[3, 3]`).
///
/// # Arguments
/// * `s` - The string to normalize
///
/// # Returns
/// Normalized lowercase string with consistent spacing
///
/// # Examples
/// ```no_run
/// # fn normalize_string(s: &str) -> String { s.to_string() }
/// assert_eq!(normalize_string("[3,3]"), "[3, 3]");
/// assert_eq!(normalize_string("[ 3 , 3 ]"), "[3, 3]");
/// assert_eq!(normalize_string("3,3"), "[3, 3]");
/// assert_eq!(normalize_string("\"3;3\""), "[3, 3]");
/// assert_eq!(normalize_string("Hello World"), "hello world");
/// ```
fn normalize_string(s: &str) -> String {
    let mut result = s.to_lowercase();

    // Strip wrapping quotes if present
    if (result.starts_with('"') && result.ends_with('"'))
        || (result.starts_with('\'') && result.ends_with('\''))
    {
        result = result[1..result.len() - 1].to_string();
    }

    // Normalize semicolons to commas (for CSV format arrays like "3;3")
    if result.contains(';') && !result.contains(',') && !result.contains(' ') {
        result = result.replace(';', ",");
    }

    // Check if this looks like a bare array (digits/values separated by commas, no brackets)
    // e.g., "3,3" or "a,b,c" -> "[3, 3]" or "[a, b, c]"
    let is_bare_array = !result.starts_with('[')
        && result.contains(',')
        && result
            .split(',')
            .all(|part| part.trim().len() < 20 && !part.contains(' '));

    if is_bare_array {
        // Add brackets to make it a proper array format
        result = format!("[{}]", result);
    }

    // Normalize spacing in array-like strings: [a,b] -> [a, b]
    // Remove spaces after [ and before ]
    result = result.replace("[ ", "[").replace(" ]", "]");
    // Normalize comma spacing: ensure single space after comma
    result = result.replace(" ,", ",").replace(",  ", ", ");
    // If no space after comma, add one (for arrays)
    if result.contains('[') && result.contains(',') {
        let chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        let mut new_chars = Vec::with_capacity(chars.len() + 10);
        while i < chars.len() {
            new_chars.push(chars[i]);
            if chars[i] == ',' && i + 1 < chars.len() && chars[i + 1] != ' ' {
                new_chars.push(' ');
            }
            i += 1;
        }
        result = new_chars.into_iter().collect();
    }

    result
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
            reason: format!("Cannot parse as number: {e}"),
        })?;

    // Convert percentage to decimal (42% -> 0.42)
    let n = if is_percent { n / 100.0 } else { n };

    // Round to specified decimals
    let factor = 10_f64.powi(decimals as i32);
    let rounded = (n * factor).round() / factor;

    Ok(format!("{rounded:.decimals$}"))
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

/// Normalize datetime to ISO format (YYYY-MM-DDTHH:MM:SSZ).
///
/// Accepts various datetime formats and normalizes to ISO 8601.
///
/// # Arguments
/// * `s` - The datetime string to normalize
///
/// # Returns
/// * `Ok(String)` - DateTime in ISO format
/// * `Err(BenchError)` - Error if format not recognized
fn normalize_datetime(s: &str) -> Result<String> {
    // Already in ISO format
    if s.contains('T') && (s.contains('Z') || s.contains('+') || s.contains('-')) {
        return Ok(s.to_string());
    }

    // Try to parse common formats
    // Simple space-separated format: YYYY-MM-DD HH:MM:SS
    if s.len() >= 19 && s.chars().nth(10) == Some(' ') {
        let parts: Vec<&str> = s.split(' ').collect();
        if parts.len() >= 2 {
            return Ok(format!("{}T{}Z", parts[0], parts[1]));
        }
    }

    Err(BenchError::NormalizationFailed {
        value: s.to_string(),
        reason: "Cannot parse as datetime".to_string(),
    })
}

/// Normalize list (comma-separated values).
///
/// Splits on commas, trims whitespace, lowercases, and optionally sorts.
/// Also handles JSON array format `["a", "b"]` and newline-separated lists.
///
/// # Arguments
/// * `s` - Comma-separated list, JSON array, or newline-separated list
/// * `ordered` - If false, list will be sorted for order-independent comparison
///
/// # Returns
/// * `Ok(String)` - Normalized comma-separated list
/// * `Err(BenchError)` - Error if normalization fails
///
/// # Examples
/// ```no_run
/// # fn normalize_list(s: &str, ordered: bool) -> Result<String, hedl_bench::error::BenchError> { Ok(s.to_string()) }
/// assert_eq!(normalize_list("a, b, c", true).unwrap(), "a,b,c");
/// assert_eq!(normalize_list("c, a, b", false).unwrap(), "a,b,c");
/// assert_eq!(normalize_list("[\"a\", \"b\"]", false).unwrap(), "a,b");
/// ```
fn normalize_list(s: &str, ordered: bool) -> Result<String> {
    let mut input = s.to_string();

    // Handle JSON array format: ["a", "b", "c"]
    if input.starts_with('[') && input.ends_with(']') {
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&input) {
            input = arr
                .into_iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string().trim_matches('"').to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ");
        } else {
            // Not valid JSON, strip brackets and continue
            input = input[1..input.len() - 1].to_string();
        }
    }

    // Handle newline-separated lists
    let items: Vec<String> = if input.contains('\n') && !input.contains(',') {
        input
            .lines()
            .map(|item| item.trim().to_lowercase())
            .filter(|item| !item.is_empty())
            .collect()
    } else {
        input
            .split(',')
            .map(|item| item.trim().to_lowercase())
            .filter(|item| !item.is_empty())
            .collect()
    };

    if ordered {
        Ok(items.join(", "))
    } else {
        let mut sorted = items;
        sorted.sort();
        Ok(sorted.join(", "))
    }
}

/// Normalize JSON object for comparison.
///
/// Parses JSON and re-serializes in canonical form.
///
/// # Arguments
/// * `s` - The JSON string to normalize
///
/// # Returns
/// * `Ok(String)` - Normalized JSON
/// * `Err(BenchError)` - Error if JSON parsing fails
fn normalize_json_object(s: &str) -> Result<String> {
    // Try to parse as JSON
    let json_value: serde_json::Value =
        serde_json::from_str(s).map_err(|e| BenchError::NormalizationFailed {
            value: s.to_string(),
            reason: format!("Cannot parse as JSON: {e}"),
        })?;

    // Serialize back to canonical form (sorted keys, no whitespace)
    serde_json::to_string(&json_value).map_err(|e| BenchError::NormalizationFailed {
        value: s.to_string(),
        reason: format!("Cannot serialize JSON: {e}"),
    })
}

/// Normalize numeric range.
///
/// Validates that the value is a number within the specified range.
///
/// # Arguments
/// * `s` - The string to validate
/// * `min` - Minimum allowed value
/// * `max` - Maximum allowed value
///
/// # Returns
/// * `Ok(String)` - Normalized number as string
/// * `Err(BenchError)` - Error if not a number or out of range
fn normalize_numeric_range(s: &str, min: f64, max: f64) -> Result<String> {
    let cleaned = remove_formatting(s);

    let n: f64 = cleaned
        .parse()
        .map_err(|e| BenchError::NormalizationFailed {
            value: s.to_string(),
            reason: format!("Cannot parse as number: {e}"),
        })?;

    if n < min || n > max {
        return Err(BenchError::NormalizationFailed {
            value: s.to_string(),
            reason: format!("Value {n} is outside range [{min}, {max}]"),
        });
    }

    Ok(n.to_string())
}

/// Normalize multiple valid answers.
///
/// Checks if the answer matches any of the valid answers.
///
/// # Arguments
/// * `s` - The answer to check
/// * `valid_answers` - List of valid answers
///
/// # Returns
/// * `Ok("valid")` - If answer matches any valid answer
/// * `Err(BenchError)` - If answer doesn't match any valid answer
fn normalize_multiple_valid(s: &str, valid_answers: &[String]) -> Result<String> {
    let lower = s.to_lowercase();

    for valid in valid_answers {
        if lower == valid.to_lowercase() {
            return Ok("valid".to_string());
        }
    }

    Err(BenchError::NormalizationFailed {
        value: s.to_string(),
        reason: format!("Answer does not match any valid answers: {valid_answers:?}"),
    })
}

/// Remove formatting characters (currency symbols, thousand separators).
///
/// Filters string to keep only numeric characters, decimal point, sign,
/// and scientific notation (e/E). Removes percentage signs and currency symbols.
/// For plus signs at the start of positive numbers, they are stripped since
/// `12.3` and `+12.3` should be treated as equivalent.
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
/// assert_eq!(remove_formatting("+12.3"), "12.3");
/// ```
fn remove_formatting(s: &str) -> String {
    let result: String = s
        .chars()
        .filter(|c| {
            c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+' || *c == 'e' || *c == 'E'
        })
        .collect();

    // Strip leading + sign for positive numbers (12.3 == +12.3)
    result.strip_prefix('+').unwrap_or(&result).to_string()
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
    fn test_clean_answer_extracts_from_reasoning() {
        // Test ANSWER: extraction
        let verbose = "To find the total, we sum 100 + 200.\nANSWER: 300";
        assert_eq!(clean_answer(verbose), "300");

        // Test case-insensitive Answer:
        let verbose2 = "The calculation shows...\nAnswer: 42.5";
        assert_eq!(clean_answer(verbose2), "42.5");

        // Test markdown bold ANSWER
        let verbose3 = "Some reasoning...\n**ANSWER:** 123";
        assert_eq!(clean_answer(verbose3), "123");

        // Test backtick format: **Answer**: `4330.95`
        let verbose4 = "Step 1...\n**Answer**: `4330.95`";
        assert_eq!(clean_answer(verbose4), "4330.95");

        // Test "The answer is:" pattern
        let verbose5 = "Working through the problem...\nThe answer is: 42";
        assert_eq!(clean_answer(verbose5), "42");
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
    fn test_normalize_datetime() {
        assert_eq!(
            normalize_datetime("2024-01-15T10:30:00Z").unwrap(),
            "2024-01-15T10:30:00Z"
        );
        assert_eq!(
            normalize_datetime("2024-01-15 10:30:00").unwrap(),
            "2024-01-15T10:30:00Z"
        );
    }

    #[test]
    fn test_normalize_list() {
        assert_eq!(normalize_list("a, b, c", true).unwrap(), "a, b, c");
        assert_eq!(normalize_list("c, a, b", false).unwrap(), "a, b, c");
    }

    #[test]
    fn test_normalize_list_json_array() {
        // JSON array format should be normalized
        assert_eq!(
            normalize_list(r#"["BERT-Base-SQuAD", "EfficientNetB0-Flowers"]"#, false).unwrap(),
            "bert-base-squad, efficientnetb0-flowers"
        );
    }

    #[test]
    fn test_normalize_list_newline_separated() {
        // Newline-separated lists should be handled
        assert_eq!(
            normalize_list("item1\nitem2\nitem3", true).unwrap(),
            "item1, item2, item3"
        );
    }

    #[test]
    fn test_normalize_string_bare_array() {
        // Bare arrays should get brackets added
        assert_eq!(normalize_string("3,3"), "[3, 3]");
        assert_eq!(normalize_string("[3,3]"), "[3, 3]");
        assert_eq!(normalize_string("[ 3 , 3 ]"), "[3, 3]");
    }

    #[test]
    fn test_normalize_string_semicolon_array() {
        // Semicolon-separated values (CSV format) should be normalized
        assert_eq!(normalize_string("\"3;3\""), "[3, 3]");
        assert_eq!(normalize_string("3;3"), "[3, 3]");
    }

    #[test]
    fn test_number_tolerance() {
        // Numbers with small differences should match within tolerance
        let num_type = AnswerType::Number { decimals: 1 };

        // 3245.1 vs 3244.5 - difference of 0.6, tolerance is 0.5% of 3244.5 = 16.2
        assert!(compare("3244.5", "3245.1", &num_type).unwrap());
        assert!(compare("3244.5", "3245.15", &num_type).unwrap());

        // But significantly different values should not match
        assert!(!compare("3244.5", "3300.0", &num_type).unwrap());
    }

    #[test]
    fn test_normalize_json_object() {
        let json = r#"{"name": "John", "age": 30}"#;
        let normalized = normalize_json_object(json).unwrap();
        assert!(normalized.contains("\"name\""));
        assert!(normalized.contains("\"age\""));
    }

    #[test]
    fn test_normalize_numeric_range() {
        assert!(normalize_numeric_range("50", 0.0, 100.0).is_ok());
        assert!(normalize_numeric_range("150", 0.0, 100.0).is_err());
    }

    #[test]
    fn test_normalize_multiple_valid() {
        let valid = vec!["answer1".to_string(), "answer2".to_string()];
        assert_eq!(
            normalize_multiple_valid("answer1", &valid).unwrap(),
            "valid"
        );
        assert_eq!(
            normalize_multiple_valid("Answer2", &valid).unwrap(),
            "valid"
        );
        assert!(normalize_multiple_valid("answer3", &valid).is_err());
    }

    #[test]
    fn test_compare() {
        assert!(compare("42", "42", &AnswerType::Integer).unwrap());
        assert!(compare("yes", "true", &AnswerType::Boolean).unwrap());
        assert!(compare("3.5", "3.5", &AnswerType::Number { decimals: 2 }).unwrap());
    }

    #[test]
    fn test_string_case_sensitive() {
        let case_sensitive = AnswerType::StringCaseSensitive;
        let case_insensitive = AnswerType::String;

        // Case sensitive should preserve case
        assert_eq!(normalize("Hello", &case_sensitive).unwrap(), "Hello");

        // Case insensitive should lowercase
        assert_eq!(normalize("Hello", &case_insensitive).unwrap(), "hello");

        // Comparison should respect case sensitivity
        assert!(!compare("Hello", "hello", &case_sensitive).unwrap());
        assert!(compare("Hello", "hello", &case_insensitive).unwrap());
    }

    #[test]
    fn test_list_ordered_vs_unordered() {
        let ordered = AnswerType::ListOrdered;
        let unordered = AnswerType::ListUnordered;

        // Ordered lists should preserve order
        assert!(!compare("a,b,c", "c,b,a", &ordered).unwrap());

        // Unordered lists should match regardless of order
        assert!(compare("a,b,c", "c,b,a", &unordered).unwrap());
    }
}
