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

//! CSV row parsing state machine for HEDL matrix rows.
//!
//! This module implements the normative CSV parsing algorithm from the HEDL spec,
//! handling quoted fields, escaped quotes, and expression regions.
//!
//! # Features
//!
//! - Quoted fields with `""` or `\"` escape for literal quotes
//! - Expression regions `$(...)` where commas don't delimit
//! - Tensor literals `[1, 2, 3]` where commas don't delimit
//! - Whitespace trimming for unquoted fields
//! - Escape sequences in quoted fields: `\n`, `\t`, `\r`, `\\`, `\"`
//! - Full UTF-8 support

use crate::lex::error::LexError;

/// A parsed CSV field with metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvField {
    /// The field value (unquoted, with escapes processed).
    pub value: String,
    /// Whether the field was enclosed in quotes.
    pub is_quoted: bool,
}

impl CsvField {
    /// Creates a field from a borrowed string slice.
    #[inline]
    fn from_borrowed(value: &str, is_quoted: bool) -> Self {
        Self {
            value: value.to_string(),
            is_quoted,
        }
    }

    /// Creates a field from an owned String.
    #[inline]
    fn from_owned(value: String, is_quoted: bool) -> Self {
        Self { value, is_quoted }
    }

    /// Returns `true` if the field value is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl AsRef<str> for CsvField {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for CsvField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_quoted {
            write!(f, "\"{}\"", self.value)
        } else {
            write!(f, "{}", self.value)
        }
    }
}

/// Parser state machine states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    StartField,
    InUnquotedField,
    InQuotedField,
    AfterQuote,
    InExpression,
}

/// Optimized field finalization that minimizes allocations.
#[inline]
fn finalize_unquoted_field(mut field: String) -> Result<String, LexError> {
    let original_len = field.len();
    let trimmed = field.trim();

    if trimmed.contains('"') {
        return Err(LexError::QuoteInUnquotedField(trimmed.to_string()));
    }

    if trimmed.len() == original_len {
        Ok(field)
    } else if trimmed.is_empty() {
        field.clear();
        Ok(field)
    } else {
        Ok(trimmed.to_string())
    }
}

/// Parses a CSV string into a list of fields.
///
/// This implements the normative algorithm from the HEDL spec.
/// It handles:
/// - Quoted fields with `""` escape for literal quotes
/// - Expression regions `$(...)` where commas don't delimit
/// - Tensor literals `[...]` where commas don't delimit
/// - Whitespace trimming for unquoted fields
/// - Full UTF-8 support for multi-byte characters
/// - Escape sequences: `\n`, `\t`, `\r`, `\\`, `\"`
///
/// # Examples
///
/// ```
/// use hedl_core::lex::parse_csv_row;
///
/// // Simple fields
/// let fields = parse_csv_row("a, b, c").unwrap();
/// assert_eq!(fields.len(), 3);
/// assert_eq!(fields[0].value, "a");
///
/// // Quoted field with comma
/// let fields = parse_csv_row(r#""hello, world", other"#).unwrap();
/// assert_eq!(fields[0].value, "hello, world");
/// assert!(fields[0].is_quoted);
///
/// // Expression region
/// let fields = parse_csv_row("id, $(a, b), value").unwrap();
/// assert_eq!(fields[1].value, "$(a, b)");
///
/// // Tensor literal
/// let fields = parse_csv_row("id, [1, 2, 3]").unwrap();
/// assert_eq!(fields[1].value, "[1, 2, 3]");
/// ```
///
/// # Errors
///
/// Returns error for:
/// - Trailing comma
/// - Unclosed quoted string
/// - Unclosed expression
/// - Quote character in unquoted field
pub fn parse_csv_row(csv_string: &str) -> Result<Vec<CsvField>, LexError> {
    if csv_string.is_empty() {
        return Ok(Vec::new());
    }

    // Check for trailing comma
    if csv_string.trim_end().ends_with(',') {
        return Err(LexError::TrailingComma);
    }

    // Pre-allocate based on estimated field count
    let estimated_fields = csv_string.bytes().filter(|&b| b == b',').count() + 1;
    let mut fields = Vec::with_capacity(estimated_fields);

    let estimated_field_capacity = (csv_string.len() / estimated_fields.max(1)).max(16);
    let mut current_field = String::with_capacity(estimated_field_capacity);
    let mut _current_is_quoted = false;
    let mut state = State::StartField;
    let mut expression_depth: usize = 0;
    let mut bracket_depth: usize = 0;

    let mut chars = csv_string.chars().peekable();

    while let Some(ch) = chars.next() {
        match state {
            State::StartField => {
                _current_is_quoted = false;
                if ch.is_ascii_whitespace() {
                    continue;
                } else if ch == ',' {
                    fields.push(CsvField::from_borrowed("", false));
                } else if ch == '"' {
                    _current_is_quoted = true;
                    state = State::InQuotedField;
                } else if ch == '$' && chars.peek() == Some(&'(') {
                    chars.next();
                    current_field.push_str("$(");
                    state = State::InExpression;
                    expression_depth = 1;
                } else if ch == '[' {
                    bracket_depth = 1;
                    current_field.push(ch);
                    state = State::InUnquotedField;
                } else {
                    state = State::InUnquotedField;
                    current_field.push(ch);
                }
            }

            State::InUnquotedField => {
                if ch == '[' {
                    bracket_depth += 1;
                    current_field.push(ch);
                } else if ch == ']' {
                    bracket_depth = bracket_depth.saturating_sub(1);
                    current_field.push(ch);
                } else if ch == ',' && bracket_depth == 0 {
                    let value = finalize_unquoted_field(std::mem::take(&mut current_field))?;
                    fields.push(CsvField::from_owned(value, false));
                    bracket_depth = 0;
                    state = State::StartField;
                } else {
                    current_field.push(ch);
                }
            }

            State::InQuotedField => {
                if ch == '"' {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        current_field.push('"');
                    } else {
                        state = State::AfterQuote;
                    }
                } else if ch == '\\' {
                    if let Some(&next_ch) = chars.peek() {
                        match next_ch {
                            'n' => {
                                chars.next();
                                current_field.push('\n');
                            }
                            't' => {
                                chars.next();
                                current_field.push('\t');
                            }
                            'r' => {
                                chars.next();
                                current_field.push('\r');
                            }
                            '\\' => {
                                chars.next();
                                current_field.push('\\');
                            }
                            '"' => {
                                chars.next();
                                current_field.push('"');
                            }
                            _ => {
                                current_field.push(ch);
                            }
                        }
                    } else {
                        current_field.push(ch);
                    }
                } else {
                    current_field.push(ch);
                }
            }

            State::AfterQuote => {
                if ch.is_ascii_whitespace() {
                    continue;
                } else if ch == ',' {
                    fields.push(CsvField::from_owned(
                        std::mem::take(&mut current_field),
                        true,
                    ));
                    state = State::StartField;
                } else {
                    return Err(LexError::ExpectedCommaAfterQuote(ch));
                }
            }

            State::InExpression => {
                current_field.push(ch);
                if ch == '(' {
                    expression_depth += 1;
                } else if ch == ')' {
                    expression_depth = expression_depth.saturating_sub(1);
                    if expression_depth == 0 {
                        state = State::InUnquotedField;
                    }
                }
            }
        }
    }

    // Handle end of string
    match state {
        State::InQuotedField => {
            return Err(LexError::UnclosedQuote {
                pos: crate::lex::error::SourcePos::default(),
            });
        }
        State::InExpression => {
            return Err(LexError::UnclosedExpression {
                pos: crate::lex::error::SourcePos::default(),
            });
        }
        State::AfterQuote => {
            fields.push(CsvField::from_owned(current_field, true));
        }
        State::InUnquotedField | State::StartField => {
            if !current_field.is_empty() || state == State::InUnquotedField {
                let value = finalize_unquoted_field(current_field)?;
                fields.push(CsvField::from_owned(value, false));
            }
        }
    }

    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Basic field tests ====================

    #[test]
    fn test_simple_fields() {
        let fields = parse_csv_row("a, b, c").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "a");
        assert_eq!(fields[1].value, "b");
        assert_eq!(fields[2].value, "c");
        assert!(!fields[0].is_quoted);
    }

    #[test]
    fn test_single_field() {
        let fields = parse_csv_row("hello").unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].value, "hello");
    }

    #[test]
    fn test_empty_input() {
        let fields = parse_csv_row("").unwrap();
        assert!(fields.is_empty());
    }

    // ==================== Quoted field tests ====================

    #[test]
    fn test_quoted_field() {
        let fields = parse_csv_row(r#""hello, world", other"#).unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].value, "hello, world");
        assert!(fields[0].is_quoted);
        assert_eq!(fields[1].value, "other");
        assert!(!fields[1].is_quoted);
    }

    #[test]
    fn test_escaped_quote() {
        let fields = parse_csv_row(r#""say ""hello""""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].value, r#"say "hello""#);
    }

    #[test]
    fn test_backslash_escapes() {
        let fields = parse_csv_row(r#""line1\nline2""#).unwrap();
        assert_eq!(fields[0].value, "line1\nline2");

        let fields = parse_csv_row(r#""col1\tcol2""#).unwrap();
        assert_eq!(fields[0].value, "col1\tcol2");

        let fields = parse_csv_row(r#""path\\file""#).unwrap();
        assert_eq!(fields[0].value, "path\\file");

        let fields = parse_csv_row(r#""say \"hi\"""#).unwrap();
        assert_eq!(fields[0].value, "say \"hi\"");
    }

    // ==================== Expression tests ====================

    #[test]
    fn test_expression() {
        let fields = parse_csv_row("id, $(a, b), value").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[1].value, "$(a, b)");
    }

    #[test]
    fn test_nested_expression() {
        let fields = parse_csv_row("$((a + b))").unwrap();
        assert_eq!(fields[0].value, "$((a + b))");
    }

    // ==================== Tensor literal tests ====================

    #[test]
    fn test_tensor_literal() {
        let fields = parse_csv_row("id, [1, 2, 3]").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[1].value, "[1, 2, 3]");
    }

    #[test]
    fn test_nested_tensor() {
        let fields = parse_csv_row("id, [[1, 2], [3, 4]]").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[1].value, "[[1, 2], [3, 4]]");
    }

    // ==================== Empty field tests ====================

    #[test]
    fn test_empty_fields() {
        let fields = parse_csv_row("a,,b").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "a");
        assert_eq!(fields[1].value, "");
        assert_eq!(fields[2].value, "b");
    }

    // ==================== Error tests ====================

    #[test]
    fn test_trailing_comma_error() {
        assert!(matches!(
            parse_csv_row("a, b,"),
            Err(LexError::TrailingComma)
        ));
    }

    #[test]
    fn test_unclosed_quote_error() {
        assert!(matches!(
            parse_csv_row(r#""unclosed"#),
            Err(LexError::UnclosedQuote { .. })
        ));
    }

    #[test]
    fn test_unclosed_expression_error() {
        assert!(matches!(
            parse_csv_row("$(unclosed"),
            Err(LexError::UnclosedExpression { .. })
        ));
    }

    #[test]
    fn test_quote_in_unquoted_error() {
        assert!(matches!(
            parse_csv_row(r#"hello"world"#),
            Err(LexError::QuoteInUnquotedField(_))
        ));
    }

    // ==================== Unicode tests ====================

    #[test]
    fn test_unicode() {
        let fields = parse_csv_row("hello, wörld, 日本語").unwrap();
        assert_eq!(fields[0].value, "hello");
        assert_eq!(fields[1].value, "wörld");
        assert_eq!(fields[2].value, "日本語");
    }

    // ==================== CsvField tests ====================

    #[test]
    fn test_csv_field_is_empty() {
        let field = CsvField::from_borrowed("", false);
        assert!(field.is_empty());

        let field = CsvField::from_borrowed("hello", false);
        assert!(!field.is_empty());
    }

    #[test]
    fn test_csv_field_as_ref() {
        let field = CsvField::from_borrowed("hello", false);
        let s: &str = field.as_ref();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_csv_field_display() {
        let field = CsvField::from_borrowed("hello", false);
        assert_eq!(format!("{}", field), "hello");

        let field = CsvField::from_borrowed("hello", true);
        assert_eq!(format!("{}", field), "\"hello\"");
    }
}
