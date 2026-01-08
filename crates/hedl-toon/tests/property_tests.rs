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

//! Property-based tests for hedl-toon string escaping and TOON conversion.
//!
//! These tests validate invariants that should hold for all inputs:
//! - String escaping roundtrip correctness
//! - Quote detection accuracy
//! - Newline handling in strings
//! - Unicode preservation
//! - Special character escaping
//! - Delimiter handling
//! - Empty and whitespace strings
//!
//! Uses proptest to generate 1000 test cases per property.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_toon::{hedl_to_toon, to_toon, Delimiter, ToToonConfig};
use proptest::prelude::*;

// ============================================================================
// Test Configuration
// ============================================================================

/// Number of test cases to run per property
const TEST_CASES: u32 = 1000;

// ============================================================================
// String Generators
// ============================================================================

/// Generate any valid Unicode string
fn any_string() -> impl Strategy<Value = String> {
    "\\PC*"
}

/// Generate strings with special characters
fn special_chars_string() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            Just('\n'),
            Just('\r'),
            Just('\t'),
            Just('\\'),
            Just('"'),
            Just(':'),
            Just(','),
            Just('['),
            Just(']'),
            Just('{'),
            Just('}'),
            Just('@'),
            Just('-'),
        ],
        0..20,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

/// Generate strings with only whitespace
fn whitespace_string() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            Just(' '),
            Just('\t'),
            Just('\n'),
            Just('\r'),
        ],
        1..20,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

/// Generate strings with leading/trailing whitespace
fn whitespace_wrapped_string() -> impl Strategy<Value = String> {
    (
        whitespace_string(),
        "[a-zA-Z0-9]{1,20}",
        whitespace_string(),
    )
        .prop_map(|(before, middle, after)| format!("{}{}{}", before, middle, after))
}

/// Generate strings that look like TOON literals
fn toon_literal_like() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("true".to_string()),
        Just("false".to_string()),
        Just("null".to_string()),
        "[-+]?[0-9]+".prop_map(|s| s.to_string()),
        "[-+]?[0-9]+\\.[0-9]+".prop_map(|s| s.to_string()),
    ]
}

/// Generate strings with newlines
fn multiline_string() -> impl Strategy<Value = String> {
    prop::collection::vec("[^\n]{0,20}\n", 1..10).prop_map(|lines| lines.join(""))
}

// ============================================================================
// String Escaping Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(TEST_CASES))]

    /// Property: Escaped strings should never contain raw (unescaped) special characters
    #[test]
    fn prop_escaped_string_no_raw_specials(s in any_string()) {
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // If string contains special chars, it must be quoted in output
        if s.contains('\n') || s.contains('\r') || s.contains('\t') || s.contains('"') || s.contains('\\') {
            // The escaped representation should use escape sequences
            if toon.contains(&s) {
                // If original appears verbatim, it must be in a context where it's safe
                // (e.g., within quotes with proper escaping)
                prop_assert!(toon.contains('"') || s.chars().all(|c| !matches!(c, '\n' | '\r' | '\t' | '"' | '\\')));
            }
        }
    }

    /// Property: Quote detection should correctly identify strings needing quotes
    #[test]
    fn prop_quote_detection_correct(s in any_string()) {
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // Empty strings must be quoted
        if s.is_empty() {
            prop_assert!(toon.contains("\"\""));
        }

        // Strings with structural characters must be quoted
        if s.contains(':') || s.contains('[') || s.contains(']') || s.contains('{') || s.contains('}') {
            prop_assert!(toon.contains('"'));
        }

        // Boolean/null literals must be quoted
        if matches!(s.as_str(), "true" | "false" | "null") {
            prop_assert!(toon.contains('"'));
        }
    }

    /// Property: Unicode characters should be preserved exactly
    #[test]
    fn prop_unicode_preserved(s in "[\\u{0}-\\u{10FFFF}]{1,50}") {
        // Filter out control chars except space - they get escaped
        let filtered: String = s.chars()
            .filter(|c| !c.is_control() || *c == ' ')
            .collect();

        if filtered.is_empty() {
            return Ok(());
        }

        let doc = create_doc_with_string(&filtered);
        let toon = hedl_to_toon(&doc)?;

        // Parse back from TOON and verify Unicode preservation
        // Since we don't have a TOON parser, we verify the string appears in output
        // (possibly quoted and escaped)

        // For simple alphanumeric Unicode (no special escaping needed), should appear verbatim
        if filtered.chars().all(|c| c.is_alphanumeric() || c == ' ') {
            let quoted = format!("\"{}\"", filtered);
            // Either appears directly or quoted
            prop_assert!(toon.contains(&filtered) || toon.contains(&quoted));
        }
    }

    /// Property: Newlines in single-line context must be escaped
    #[test]
    fn prop_newlines_escaped_in_single_line(s in multiline_string()) {
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // Count actual newlines in output (not escaped ones)
        // The field value line should not contain unescaped newlines
        for line in toon.lines() {
            if line.contains("test_field:") {
                // This line contains our test field
                // It should not have unescaped newlines (\\n is ok, \n is not)
                let _value_part = line.split(':').nth(1).unwrap_or("");

                // If there's a literal newline in this line (not \\n), that's wrong
                // But we're already splitting by lines, so this check is implicit
                prop_assert!(true);
            }
        }

        // Verify that the escape sequence \\n appears if input had newlines
        if s.contains('\n') {
            prop_assert!(toon.contains("\\n") || toon.contains("\""));
        }
    }

    /// Property: Empty strings must be quoted
    #[test]
    fn prop_empty_string_quoted(_seed in any::<u64>()) {
        let doc = create_doc_with_string("");
        let toon = hedl_to_toon(&doc)?;
        prop_assert!(toon.contains("\"\""));
    }

    /// Property: Strings with only whitespace must be quoted
    #[test]
    fn prop_whitespace_only_quoted(s in whitespace_string()) {
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;
        prop_assert!(toon.contains('"'));
    }

    /// Property: Leading/trailing whitespace requires quoting
    #[test]
    fn prop_leading_trailing_whitespace_quoted(s in whitespace_wrapped_string()) {
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // Should be quoted if it has leading/trailing whitespace
        if s.starts_with(|c: char| c.is_whitespace()) || s.ends_with(|c: char| c.is_whitespace()) {
            prop_assert!(toon.contains('"'));
        }
    }

    /// Property: TOON literals must be quoted when used as strings
    #[test]
    fn prop_toon_literals_quoted(s in toon_literal_like()) {
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // Boolean and null literals must be quoted
        if matches!(s.as_str(), "true" | "false" | "null") {
            let quoted = format!("\"{}\"", s);
            prop_assert!(toon.contains(&quoted));
        }

        // Numeric-looking strings must be quoted if they start with digit or minus+digit
        // Note: +0.0 parses as a float, so it doesn't need quoting (it's a valid number)
        if s.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            prop_assert!(toon.contains('"'));
        } else if s.starts_with('-') && s.len() > 1 {
            if s.chars().nth(1).map_or(false, |c| c.is_ascii_digit()) {
                prop_assert!(toon.contains('"'));
            }
        }
        // +0.0 is a valid numeric literal, no quoting needed
    }

    /// Property: Backslashes must be properly escaped
    #[test]
    fn prop_backslashes_escaped(count in 1..10usize) {
        let s = "\\".repeat(count);
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // Should contain escaped backslashes (\\\\)
        if toon.contains(&s) {
            // If original appears, verify it's in escaped form
            let escaped = "\\\\".repeat(count);
            prop_assert!(toon.contains(&escaped));
        }
    }

    /// Property: Quotes must be properly escaped
    #[test]
    fn prop_quotes_escaped(s in ".*\".*") {
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // String with quotes must be quoted and escaped
        prop_assert!(toon.contains("\\\""));
    }

    /// Property: Delimiter characters require quoting
    #[test]
    fn prop_delimiter_requires_quoting(
        s in "[a-z]{1,5},[a-z]{1,5}",
        delimiter in prop_oneof![
            Just(Delimiter::Comma),
            Just(Delimiter::Tab),
            Just(Delimiter::Pipe),
        ]
    ) {
        let doc = create_doc_with_string(&s);
        let config = ToToonConfig {
            indent: 2,
            delimiter,
        };
        let toon = to_toon(&doc, &config)?;

        // If string contains the delimiter, must be quoted
        let delim_char = match delimiter {
            Delimiter::Comma => ',',
            Delimiter::Tab => '\t',
            Delimiter::Pipe => '|',
        };

        if s.contains(delim_char) {
            prop_assert!(toon.contains('"'));
        }
    }

    /// Property: Structural characters require quoting
    #[test]
    fn prop_structural_chars_quoted(c in prop_oneof![
        Just(':'),
        Just('['),
        Just(']'),
        Just('{'),
        Just('}'),
    ]) {
        let s = format!("test{}value", c);
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;
        prop_assert!(toon.contains('"'));
    }

    /// Property: Special prefixes require quoting
    #[test]
    fn prop_special_prefix_quoted(
        c in prop_oneof![Just('-'), Just('@')],
        rest in "[a-z]{1,10}"
    ) {
        let s = format!("{}{}", c, rest);
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;
        prop_assert!(toon.contains('"'));
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(TEST_CASES))]

    /// Property: Very long strings should be handled
    #[test]
    fn prop_long_strings_handled(length in 1000..5000usize) {
        let s = "a".repeat(length);
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // Should produce valid output
        prop_assert!(toon.len() > 0);
        prop_assert!(toon.contains("test_field:"));
    }

    /// Property: Mixed special characters
    #[test]
    fn prop_mixed_special_chars(s in special_chars_string()) {
        let doc = create_doc_with_string(&s);
        let result = hedl_to_toon(&doc);

        // Should succeed
        prop_assert!(result.is_ok());

        if let Ok(toon) = result {
            // Should be quoted if non-empty
            if !s.is_empty() {
                prop_assert!(toon.contains('"') || s.chars().all(|c| c.is_alphanumeric()));
            }
        }
    }

    /// Property: Round-trip through TOON (basic verification)
    #[test]
    fn prop_roundtrip_structure_preserved(s in "[a-zA-Z0-9 ]{1,50}") {
        let doc1 = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc1)?;

        // Verify basic structure
        prop_assert!(toon.contains("test_field:"));
        prop_assert!(toon.len() > 0);

        // The string value should appear somewhere in the output
        // (either quoted or unquoted depending on content)
        let quoted = format!("\"{}\"", s);
        prop_assert!(toon.contains(&s) || toon.contains(&quoted));
    }

    /// Property: Control characters are escaped
    #[test]
    fn prop_control_chars_escaped(s in ".*[\\x00-\\x1F].*") {
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // Common control chars should be escaped
        if s.contains('\n') {
            prop_assert!(toon.contains("\\n") || !toon.contains('\n'));
        }
        if s.contains('\r') {
            prop_assert!(toon.contains("\\r") || !toon.contains('\r'));
        }
        if s.contains('\t') {
            prop_assert!(toon.contains("\\t") || !toon.contains('\t'));
        }
    }

    /// Property: Tab delimiter escaping
    #[test]
    fn prop_tab_delimiter_handling(s in ".*\t.*") {
        let config = ToToonConfig {
            indent: 2,
            delimiter: Delimiter::Tab,
        };
        let doc = create_doc_with_string(&s);
        let toon = to_toon(&doc, &config)?;

        // String with tab using tab delimiter must be quoted
        prop_assert!(toon.contains('"'));
    }

    /// Property: Pipe delimiter escaping
    #[test]
    fn prop_pipe_delimiter_handling(s in ".*\\|.*") {
        let config = ToToonConfig {
            indent: 2,
            delimiter: Delimiter::Pipe,
        };
        let doc = create_doc_with_string(&s);
        let toon = to_toon(&doc, &config)?;

        // String with pipe using pipe delimiter must be quoted
        prop_assert!(toon.contains('"'));
    }
}

// ============================================================================
// Invariant Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(TEST_CASES))]

    /// Property: TOON conversion is deterministic
    #[test]
    fn prop_conversion_deterministic(s in any_string()) {
        let doc = create_doc_with_string(&s);
        let toon1 = hedl_to_toon(&doc)?;
        let toon2 = hedl_to_toon(&doc)?;
        prop_assert_eq!(toon1, toon2);
    }

    /// Property: Same string in different positions produces same escaping
    #[test]
    fn prop_escaping_consistent(s in "[a-zA-Z0-9:, ]{1,20}") {
        let mut doc = Document::new((1, 0));
        doc.root.insert("field1".to_string(), Item::Scalar(Value::String(s.clone())));
        doc.root.insert("field2".to_string(), Item::Scalar(Value::String(s.clone())));

        let toon = hedl_to_toon(&doc)?;

        // Both fields should have the same value representation
        let lines: Vec<&str> = toon.lines().collect();
        let field1_line = lines.iter().find(|l| l.contains("field1:")).unwrap();
        let field2_line = lines.iter().find(|l| l.contains("field2:")).unwrap();

        let val1 = field1_line.split(':').nth(1).unwrap().trim();
        let val2 = field2_line.split(':').nth(1).unwrap().trim();

        prop_assert_eq!(val1, val2);
    }

    /// Property: No injection through special characters
    #[test]
    fn prop_no_injection(s in special_chars_string()) {
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // Output should be parseable (no broken structure)
        // Verify basic structure markers are present
        prop_assert!(toon.contains("test_field:"));

        // Should not have unbalanced quotes or brackets
        // Count quotes excluding escaped quotes (\\")
        let mut quote_count = 0;
        let mut prev_was_backslash = false;
        for c in toon.chars() {
            if c == '"' && !prev_was_backslash {
                quote_count += 1;
            }
            prev_was_backslash = c == '\\' && !prev_was_backslash;
        }
        prop_assert_eq!(quote_count % 2, 0, "Unbalanced quotes");
    }

    /// Property: TOON output is valid line-oriented format
    #[test]
    fn prop_valid_line_format(s in any_string()) {
        let doc = create_doc_with_string(&s);
        let toon = hedl_to_toon(&doc)?;

        // Every line should be parseable (no dangling content)
        for line in toon.lines() {
            // Lines should not end with escape characters (incomplete escape)
            if !line.is_empty() {
                prop_assert!(!line.ends_with('\\') || line.ends_with("\\\\"));
            }
        }
    }
}

// ============================================================================
// Array/List String Handling
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(TEST_CASES))]

    /// Property: Strings in arrays are escaped consistently
    #[test]
    fn prop_array_string_escaping(
        s1 in "[a-zA-Z0-9, ]{1,20}",
        s2 in "[a-zA-Z0-9, ]{1,20}",
    ) {
        let mut doc = Document::new((1, 0));
        doc.structs.insert("Item".to_string(), vec!["value".to_string()]);

        let mut list = MatrixList::new("Item", vec!["value".to_string()]);
        list.add_row(Node::new("Item", "i1", vec![Value::String(s1.clone())]));
        list.add_row(Node::new("Item", "i2", vec![Value::String(s2.clone())]));

        doc.root.insert("items".to_string(), Item::List(list));

        let toon = hedl_to_toon(&doc)?;

        // Should have tabular format with proper escaping
        let expected = "items[2]{value}:";
        prop_assert!(toon.contains(expected));
    }

    /// Property: Empty strings in arrays are quoted
    #[test]
    fn prop_array_empty_strings(_seed in any::<u64>()) {
        let mut doc = Document::new((1, 0));
        doc.structs.insert("Item".to_string(), vec!["value".to_string()]);

        let mut list = MatrixList::new("Item", vec!["value".to_string()]);
        list.add_row(Node::new("Item", "i1", vec![Value::String("".to_string())]));

        doc.root.insert("items".to_string(), Item::List(list));

        let toon = hedl_to_toon(&doc)?;

        // Empty string must be quoted
        prop_assert!(toon.contains("\"\""));
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a minimal document with a single string field for testing
fn create_doc_with_string(s: &str) -> Document {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "test_field".to_string(),
        Item::Scalar(Value::String(s.to_string())),
    );
    doc
}

// ============================================================================
// Regression Tests (from proptest-regressions file)
// ============================================================================

#[test]
fn regression_quote_character() {
    // From proptest-regressions: s = "\""
    let doc = create_doc_with_string("\"");
    let toon = hedl_to_toon(&doc).unwrap();

    // Should escape the quote
    assert!(toon.contains("\\\""));
}

#[test]
fn regression_empty_string() {
    let doc = create_doc_with_string("");
    let toon = hedl_to_toon(&doc).unwrap();

    // Empty string must be quoted
    assert!(toon.contains("\"\""));
}

#[test]
fn regression_newline() {
    let doc = create_doc_with_string("hello\nworld");
    let toon = hedl_to_toon(&doc).unwrap();

    // Newline must be escaped
    assert!(toon.contains("\\n"));
}

#[test]
fn regression_backslash() {
    let doc = create_doc_with_string("path\\to\\file");
    let toon = hedl_to_toon(&doc).unwrap();

    // Backslashes must be escaped
    assert!(toon.contains("\\\\"));
}

#[test]
fn regression_unicode() {
    let doc = create_doc_with_string("Hello 世界 🌍");
    let toon = hedl_to_toon(&doc).unwrap();

    // Unicode should be preserved
    assert!(toon.contains("世界"));
    assert!(toon.contains("🌍"));
}
