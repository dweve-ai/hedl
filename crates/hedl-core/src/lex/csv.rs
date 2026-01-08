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

//! CSV parsing for HEDL matrix rows.
//!
//! Implements the normative CSV parsing algorithm from SPEC Section 9.2.

use super::error::LexError;
use super::span::SourcePos;

/// A parsed CSV field with its content and quoting status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvField {
    /// The field content (unquoted).
    pub value: String,
    /// Whether this field was enclosed in quotes.
    pub is_quoted: bool,
}

/// Parse a CSV row into fields.
///
/// Follows the normative algorithm from SPEC Section 9.2:
/// - Delimiter: comma
/// - Quoting: double quotes only
/// - Escaping: "" inside quotes → literal "
/// - Whitespace: unquoted fields trimmed
/// - Trailing comma: error
///
/// # Examples
///
/// ```
/// use hedl_core::lex::parse_csv_row;
///
/// let fields = parse_csv_row("a, b, c").unwrap();
/// assert_eq!(fields.len(), 3);
/// assert_eq!(fields[0].value, "a");
/// assert!(!fields[0].is_quoted);
///
/// let fields = parse_csv_row(r#""quoted, value", other"#).unwrap();
/// assert_eq!(fields[0].value, "quoted, value");
/// assert!(fields[0].is_quoted);
/// ```
pub fn parse_csv_row(s: &str) -> Result<Vec<CsvField>, LexError> {
    if s.is_empty() {
        return Ok(Vec::new());
    }

    // Check for trailing comma
    if s.trim_end().ends_with(',') {
        return Err(LexError::InvalidToken {
            message: "trailing comma not allowed in CSV row".to_string(),
            pos: SourcePos::new(1, 1),
        });
    }

    let mut fields = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();

    while i < chars.len() {
        // Skip leading whitespace
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        if i >= chars.len() {
            break;
        }

        // Parse one field
        let (field, new_i) = parse_field(&chars, i)?;
        fields.push(field);
        i = new_i;

        // Skip trailing whitespace after field
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        if i < chars.len() {
            if chars[i] == ',' {
                i += 1; // Skip comma
            } else {
                return Err(LexError::InvalidToken {
                    message: format!("expected comma or end of line, got '{}'", chars[i]),
                    pos: SourcePos::new(1, i + 1),
                });
            }
        }
    }

    Ok(fields)
}

/// Parse a single field starting at position i.
/// Returns the field and the position after it.
fn parse_field(chars: &[char], start: usize) -> Result<(CsvField, usize), LexError> {
    let mut i = start;

    // Skip leading whitespace
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }

    if i >= chars.len() {
        // Empty field
        return Ok((
            CsvField {
                value: String::new(),
                is_quoted: false,
            },
            i,
        ));
    }

    if chars[i] == '"' {
        // Quoted field
        parse_quoted_field(chars, i)
    } else {
        // Unquoted field
        parse_unquoted_field(chars, i)
    }
}

/// Parse a quoted field starting with ".
/// Supports escape sequences: `\n` (newline), `\t` (tab), `\\` (backslash), `\"` (quote).
/// Also supports CSV-style `""` for embedded quotes.
fn parse_quoted_field(chars: &[char], start: usize) -> Result<(CsvField, usize), LexError> {
    let mut i = start + 1; // Skip opening quote
    let mut value = String::new();

    while i < chars.len() {
        if chars[i] == '"' {
            // Check for escaped quote (CSV-style "")
            if i + 1 < chars.len() && chars[i + 1] == '"' {
                value.push('"');
                i += 2;
            } else {
                // End of quoted field
                i += 1; // Skip closing quote
                return Ok((
                    CsvField {
                        value,
                        is_quoted: true,
                    },
                    i,
                ));
            }
        } else if chars[i] == '\\' && i + 1 < chars.len() {
            // Handle escape sequences
            let next = chars[i + 1];
            match next {
                'n' => {
                    value.push('\n');
                    i += 2;
                }
                't' => {
                    value.push('\t');
                    i += 2;
                }
                'r' => {
                    value.push('\r');
                    i += 2;
                }
                '\\' => {
                    value.push('\\');
                    i += 2;
                }
                '"' => {
                    value.push('"');
                    i += 2;
                }
                _ => {
                    // Unknown escape - keep as-is
                    value.push(chars[i]);
                    i += 1;
                }
            }
        } else {
            value.push(chars[i]);
            i += 1;
        }
    }

    // Hit end of line without closing quote
    Err(LexError::UnclosedQuote {
        pos: SourcePos::new(1, start + 1),
    })
}

/// Parse an unquoted field.
fn parse_unquoted_field(chars: &[char], start: usize) -> Result<(CsvField, usize), LexError> {
    let mut i = start;
    let mut value = String::new();
    let mut expr_depth = 0;
    let mut in_expr_quotes = false;

    while i < chars.len() {
        let ch = chars[i];

        // Check if we hit a comma (and not in expression)
        if ch == ',' && expr_depth == 0 {
            break;
        }

        value.push(ch);

        // Track expression depth
        if ch == '$' && i + 1 < chars.len() && chars[i + 1] == '(' {
            expr_depth += 1;
            value.push(chars[i + 1]);
            i += 2;
            continue;
        }

        // Handle quotes inside expressions
        if expr_depth > 0 {
            if ch == '"' {
                if in_expr_quotes {
                    // Check for escaped quote
                    if i + 1 < chars.len() && chars[i + 1] == '"' {
                        value.push(chars[i + 1]);
                        i += 2;
                        continue;
                    } else {
                        in_expr_quotes = false;
                    }
                } else {
                    in_expr_quotes = true;
                }
            } else if !in_expr_quotes {
                if ch == '(' {
                    expr_depth += 1;
                } else if ch == ')' {
                    expr_depth -= 1;
                }
            }
        }

        i += 1;
    }

    if expr_depth > 0 {
        return Err(LexError::UnclosedExpression {
            pos: SourcePos::new(1, start + 1),
        });
    }

    let trimmed = value.trim().to_string();

    // Check for quotes in unquoted field (but not if it's an expression)
    if trimmed.contains('"') && !trimmed.starts_with("$(") {
        return Err(LexError::InvalidToken {
            message: format!("quote character '\"' found in unquoted field: '{}'", trimmed),
            pos: SourcePos::new(1, start + 1),
        });
    }

    Ok((
        CsvField {
            value: trimmed,
            is_quoted: false,
        },
        i,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_simple() {
        let fields = parse_csv_row("a, b, c").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "a");
        assert_eq!(fields[1].value, "b");
        assert_eq!(fields[2].value, "c");
        assert!(!fields[0].is_quoted);
    }

    #[test]
    fn test_parse_csv_no_spaces() {
        let fields = parse_csv_row("a,b,c").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "a");
    }

    #[test]
    fn test_parse_csv_quoted_field() {
        let fields = parse_csv_row(r#""quoted, value", other"#).unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].value, "quoted, value");
        assert!(fields[0].is_quoted);
        assert_eq!(fields[1].value, "other");
        assert!(!fields[1].is_quoted);
    }

    #[test]
    fn test_parse_csv_escaped_quote() {
        // Input: "escaped ""quote""" (with proper closing quote)
        // Expected output: escaped "quote"
        let input = "\"escaped \"\"quote\"\"\"";
        let fields = parse_csv_row(input).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].value, "escaped \"quote\"");
        assert!(fields[0].is_quoted);
    }

    #[test]
    fn test_parse_csv_empty_fields() {
        let fields = parse_csv_row("a,,c").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "a");
        assert_eq!(fields[1].value, "");
        assert_eq!(fields[2].value, "c");
    }

    #[test]
    fn test_parse_csv_whitespace_preserved_quoted() {
        let fields = parse_csv_row(r#""  spaced  ", other"#).unwrap();
        assert_eq!(fields[0].value, "  spaced  ");
        assert!(fields[0].is_quoted);
    }

    #[test]
    fn test_parse_csv_whitespace_trimmed_unquoted() {
        let fields = parse_csv_row("  spaced  ,  other  ").unwrap();
        assert_eq!(fields[0].value, "spaced");
        assert_eq!(fields[1].value, "other");
    }

    #[test]
    fn test_parse_csv_empty_string() {
        let fields = parse_csv_row("").unwrap();
        assert!(fields.is_empty());
    }

    #[test]
    fn test_parse_csv_trailing_comma_error() {
        let result = parse_csv_row("a, b,");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_csv_unclosed_quote_error() {
        let result = parse_csv_row(r#""unclosed"#);
        assert!(matches!(result, Err(LexError::UnclosedQuote { .. })));
    }

    #[test]
    fn test_parse_csv_quote_in_unquoted_field_error() {
        let result = parse_csv_row(r#"hello"world"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_csv_with_expression() {
        let fields = parse_csv_row("$(x + 1), other").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].value, "$(x + 1)");
        assert!(!fields[0].is_quoted);
    }

    #[test]
    fn test_parse_csv_expression_with_comma() {
        let fields = parse_csv_row("$(a, b), other").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].value, "$(a, b)");
    }

    #[test]
    fn test_parse_csv_expression_with_nested_parens() {
        let fields = parse_csv_row("$((a + b)), other").unwrap();
        assert_eq!(fields[0].value, "$((a + b))");
    }

    #[test]
    fn test_parse_csv_expression_with_quotes() {
        let fields = parse_csv_row(r#"$(concat("a", "b")), other"#).unwrap();
        assert_eq!(fields[0].value, r#"$(concat("a", "b"))"#);
    }

    #[test]
    fn test_parse_csv_unclosed_expression_error() {
        let result = parse_csv_row("$(incomplete, other");
        assert!(matches!(result, Err(LexError::UnclosedExpression { .. })));
    }

    #[test]
    fn test_parse_csv_single_field() {
        let fields = parse_csv_row("single").unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].value, "single");
    }

    #[test]
    fn test_parse_csv_all_quoted() {
        let fields = parse_csv_row(r#""a", "b", "c""#).unwrap();
        assert_eq!(fields.len(), 3);
        assert!(fields.iter().all(|f| f.is_quoted));
    }

    #[test]
    fn test_parse_csv_escape_newline() {
        // \n becomes actual newline
        let fields = parse_csv_row(r#""line1\nline2""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].value, "line1\nline2");
        assert!(fields[0].value.contains('\n'));
    }

    #[test]
    fn test_parse_csv_escape_tab() {
        // \t becomes actual tab
        let fields = parse_csv_row(r#""col1\tcol2""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].value, "col1\tcol2");
        assert!(fields[0].value.contains('\t'));
    }

    #[test]
    fn test_parse_csv_escape_backslash() {
        // \\ becomes single backslash
        let fields = parse_csv_row(r#""path\\to\\file""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].value, r"path\to\file");
    }

    #[test]
    fn test_parse_csv_escape_quote() {
        // \" becomes quote (alternative to "")
        let fields = parse_csv_row(r#""say \"hello\"""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].value, "say \"hello\"");
    }

    #[test]
    fn test_parse_csv_multiple_escapes() {
        // Multiple escape sequences in one field
        let fields = parse_csv_row(r#""line1\nline2\ttabbed""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert!(fields[0].value.contains('\n'));
        assert!(fields[0].value.contains('\t'));
    }

    #[test]
    fn test_parse_csv_carriage_return() {
        // \r becomes carriage return
        let fields = parse_csv_row(r#""windows\r\nline""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert!(fields[0].value.contains('\r'));
        assert!(fields[0].value.contains('\n'));
    }

    // ==================== Additional edge cases ====================

    #[test]
    fn test_parse_csv_unicode_unquoted() {
        let fields = parse_csv_row("日本語, émoji, über").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "日本語");
        assert_eq!(fields[1].value, "émoji");
        assert_eq!(fields[2].value, "über");
    }

    #[test]
    fn test_parse_csv_unicode_quoted() {
        let fields = parse_csv_row(r#""日本語", "émoji 🎉", "über""#).unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "日本語");
        assert!(fields[0].is_quoted);
        assert!(fields[1].value.contains('🎉'));
    }

    #[test]
    fn test_parse_csv_only_whitespace_between_commas() {
        let fields = parse_csv_row("a,   ,c").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "a");
        assert_eq!(fields[1].value, "");
        assert_eq!(fields[2].value, "c");
    }

    #[test]
    fn test_parse_csv_numbers() {
        let fields = parse_csv_row("123, 45.67, -89, 0").unwrap();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0].value, "123");
        assert_eq!(fields[1].value, "45.67");
        assert_eq!(fields[2].value, "-89");
        assert_eq!(fields[3].value, "0");
    }

    #[test]
    fn test_parse_csv_booleans() {
        let fields = parse_csv_row("true, false, null").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "true");
        assert_eq!(fields[1].value, "false");
        assert_eq!(fields[2].value, "null");
    }

    #[test]
    fn test_parse_csv_references() {
        let fields = parse_csv_row("@User:123, @Post:456").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].value, "@User:123");
        assert_eq!(fields[1].value, "@Post:456");
    }

    #[test]
    fn test_parse_csv_mixed_types() {
        let fields = parse_csv_row(r#"123, "hello", true, null, @Ref:1"#).unwrap();
        assert_eq!(fields.len(), 5);
        assert_eq!(fields[0].value, "123");
        assert_eq!(fields[1].value, "hello");
        assert!(fields[1].is_quoted);
        assert_eq!(fields[2].value, "true");
        assert_eq!(fields[3].value, "null");
        assert_eq!(fields[4].value, "@Ref:1");
    }

    #[test]
    fn test_parse_csv_many_fields() {
        let input = (0..20)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let fields = parse_csv_row(&input).unwrap();
        assert_eq!(fields.len(), 20);
        for (i, field) in fields.iter().enumerate() {
            assert_eq!(field.value, i.to_string());
        }
    }

    #[test]
    fn test_parse_csv_expression_nested_quotes() {
        // Expression with nested quoted strings
        let fields = parse_csv_row(r#"$(concat("a""b", "c")), other"#).unwrap();
        assert_eq!(fields.len(), 2);
        assert!(fields[0].value.starts_with("$("));
    }

    #[test]
    fn test_parse_csv_multiple_expressions() {
        let fields = parse_csv_row("$(a), $(b), $(c)").unwrap();
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "$(a)");
        assert_eq!(fields[1].value, "$(b)");
        assert_eq!(fields[2].value, "$(c)");
    }

    #[test]
    fn test_parse_csv_expression_with_nested_calls() {
        let fields = parse_csv_row("$(outer(inner(x))), other").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].value, "$(outer(inner(x)))");
    }

    #[test]
    fn test_parse_csv_quoted_empty() {
        let fields = parse_csv_row(r#""""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].value, "");
        assert!(fields[0].is_quoted);
    }

    #[test]
    fn test_parse_csv_quoted_only_spaces() {
        let fields = parse_csv_row(r#""   ""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].value, "   ");
        assert!(fields[0].is_quoted);
    }

    #[test]
    fn test_parse_csv_unknown_escape() {
        // Unknown escape sequences are kept as-is (backslash preserved)
        let fields = parse_csv_row(r#""\x""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert!(fields[0].value.contains('\\'));
    }

    #[test]
    fn test_parse_csv_double_quotes_in_expression() {
        // Expression can contain escaped quotes
        let fields = parse_csv_row(r#"$(say("hello""world")), other"#).unwrap();
        assert_eq!(fields.len(), 2);
        assert!(fields[0].value.contains("hello\"\"world"));
    }

    #[test]
    fn test_parse_csv_special_chars_in_quotes() {
        let fields = parse_csv_row(r#""!@#$%^&*()[]{}|;:'<>?/""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert!(fields[0].is_quoted);
        assert!(fields[0].value.contains('#'));
        assert!(fields[0].value.contains('!'));
    }

    #[test]
    fn test_parse_csv_comma_in_quoted() {
        let fields = parse_csv_row(r#""a,b,c""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert!(fields[0].is_quoted);
        assert!(fields[0].value.contains(','));
        assert_eq!(fields[0].value, "a,b,c");
    }

    #[test]
    fn test_csv_field_equality() {
        let a = CsvField {
            value: "test".to_string(),
            is_quoted: true,
        };
        let b = CsvField {
            value: "test".to_string(),
            is_quoted: true,
        };
        let c = CsvField {
            value: "test".to_string(),
            is_quoted: false,
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_csv_field_clone() {
        let original = CsvField {
            value: "test".to_string(),
            is_quoted: true,
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_csv_field_debug() {
        let field = CsvField {
            value: "test".to_string(),
            is_quoted: true,
        };
        let debug = format!("{:?}", field);
        assert!(debug.contains("test"));
        assert!(debug.contains("is_quoted"));
    }

    #[test]
    fn test_parse_csv_trailing_comma_with_spaces() {
        // Trailing comma after spaces should still error
        let result = parse_csv_row("a, b,   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_csv_leading_comma_error() {
        // Leading comma gives empty first field
        let fields = parse_csv_row(",a").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].value, "");
        assert_eq!(fields[1].value, "a");
    }

    #[test]
    fn test_parse_csv_multiple_consecutive_commas() {
        let fields = parse_csv_row("a,,,b").unwrap();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0].value, "a");
        assert_eq!(fields[1].value, "");
        assert_eq!(fields[2].value, "");
        assert_eq!(fields[3].value, "b");
    }

    #[test]
    fn test_parse_csv_tab_in_unquoted_field() {
        // Tab is whitespace, so it gets trimmed in unquoted field
        let fields = parse_csv_row("a\tb, c").unwrap();
        assert_eq!(fields.len(), 2);
        // "a\tb" gets trimmed at edges but tab in middle stays
        assert!(fields[0].value.contains('\t') || fields[0].value == "a\tb");
    }

    #[test]
    fn test_parse_csv_tab_in_quoted_field() {
        // Tab in quoted field is preserved
        let fields = parse_csv_row("\"a\tb\", c").unwrap();
        assert_eq!(fields.len(), 2);
        assert!(fields[0].value.contains('\t'));
    }

    #[test]
    fn test_parse_csv_very_long_field() {
        let long_value = "x".repeat(10000);
        let input = format!("\"{}\", other", long_value);
        let fields = parse_csv_row(&input).unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].value.len(), 10000);
    }

    #[test]
    fn test_parse_csv_all_escapes_together() {
        let fields = parse_csv_row(r#""line1\nline2\ttab\r\nwindows\\path\"""#).unwrap();
        assert_eq!(fields.len(), 1);
        assert!(fields[0].value.contains('\n'));
        assert!(fields[0].value.contains('\t'));
        assert!(fields[0].value.contains('\r'));
        assert!(fields[0].value.contains('\\'));
        assert!(fields[0].value.contains('"'));
    }
}
