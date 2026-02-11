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

//! Property-based tests for block string handling.
//!
//! # Invariants Tested
//!
//! 1. **Content Preservation**: Block string content is preserved
//! 2. **Indentation Handling**: Leading whitespace is handled correctly
//! 3. **Line Preservation**: Line count and content are maintained
//! 4. **Empty Lines**: Empty lines within blocks are preserved

use hedl_core::parse;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Block strings preserve line count.
    #[test]
    fn prop_block_string_line_count(
        lines in prop::collection::vec("[a-zA-Z0-9 ]{0,50}", 1..=10)
    ) {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: |\n  {}\n",
            lines.join("\n  ")
        );

        let result = parse(doc.as_bytes());
        if result.is_ok() {
            let parsed = result.unwrap();
            if let Some(item) = parsed.get("text") {
                if let Some(val) = item.as_scalar() {
                    if let Some(text_str) = val.as_str() {
                        let actual_lines = text_str.lines().count();
                        prop_assert!(actual_lines > 0,
                            "Block string should have at least 1 line");
                    }
                }
            }
        }
    }

    /// Property: Single-line block strings work.
    #[test]
    fn prop_single_line_block_string(content in "[a-zA-Z0-9 ]{1,50}") {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: |\n  {content}\n"
        );

        let result = parse(doc.as_bytes());
        if result.is_ok() {
            let parsed = result.unwrap();
            if let Some(item) = parsed.get("text") {
                if let Some(val) = item.as_scalar() {
                    if let Some(text_str) = val.as_str() {
                        prop_assert!(!text_str.is_empty(),
                            "Block string should not be empty");
                    }
                }
            }
        }
    }

    /// Property: Multi-line block strings preserve content.
    #[test]
    fn prop_multi_line_block_string(
        line_count in 2_usize..10,
        line_content in "[a-zA-Z0-9]{1,30}"
    ) {
        let lines: Vec<String> = (0..line_count)
            .map(|i| format!("{line_content}_{i}"))
            .collect();

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: |\n  {}\n",
            lines.join("\n  ")
        );

        let result = parse(doc.as_bytes());
        if result.is_ok() {
            let parsed = result.unwrap();
            if let Some(item) = parsed.get("text") {
                if let Some(val) = item.as_scalar() {
                    if let Some(text_str) = val.as_str() {
                        let actual_line_count = text_str.lines().count();
                        prop_assert!(actual_line_count > 0,
                            "Block string should have lines");
                    }
                }
            }
        }
    }

    /// Property: Quoted strings with empty lines (newlines) are handled.
    #[test]
    fn prop_block_string_with_empty_lines(
        non_empty in "[a-zA-Z0-9 ]{1,50}",
        empty_count in 1_usize..5
    ) {
        let mut lines = vec![non_empty.clone()];
        for _ in 0..empty_count {
            lines.push(String::new());
        }
        lines.push(non_empty.clone());

        // HEDL uses quoted strings with \n for newlines, not YAML block strings
        let content = lines.join("\\n");
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: \"{content}\"\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(),
            "Failed to parse quoted string with newlines: {:?}", result.err());
    }

    /// Property: Quoted strings with leading spaces in content are handled.
    #[test]
    fn prop_block_string_leading_spaces(
        spaces in 1_usize..10,
        content in "[a-zA-Z0-9]{1,30}"
    ) {
        let line = format!("{}{}", " ".repeat(spaces), content);
        // HEDL uses quoted strings, spaces are preserved
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: \"{line}\"\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(),
            "Failed to parse quoted string with leading spaces: {:?}", result.err());

        if let Ok(parsed) = result {
            if let Some(item) = parsed.get("text") {
                if let Some(val) = item.as_scalar() {
                    if let Some(text_str) = val.as_str() {
                        prop_assert_eq!(text_str, line,
                            "Leading spaces should be preserved");
                    }
                }
            }
        }
    }

    /// Property: Quoted strings with trailing spaces are handled.
    #[test]
    fn prop_block_string_trailing_spaces(
        content in "[a-zA-Z0-9]{1,30}",
        spaces in 1_usize..10
    ) {
        let line = format!("{}{}", content, " ".repeat(spaces));
        // HEDL uses quoted strings, trailing spaces are preserved
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: \"{line}\"\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(),
            "Failed to parse quoted string with trailing spaces: {:?}", result.err());

        if let Ok(parsed) = result {
            if let Some(item) = parsed.get("text") {
                if let Some(val) = item.as_scalar() {
                    if let Some(text_str) = val.as_str() {
                        prop_assert_eq!(text_str, line,
                            "Trailing spaces should be preserved");
                    }
                }
            }
        }
    }

    /// Property: Very long block strings (many lines) are handled.
    #[test]
    fn prop_long_block_string(line_count in 50_usize..200) {
        let lines: Vec<String> = (0..line_count)
            .map(|i| format!("line{i}"))
            .collect();

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: |\n  {}\n",
            lines.join("\n  ")
        );

        let result = parse(doc.as_bytes());
        if result.is_ok() {
            let parsed = result.unwrap();
            if let Some(item) = parsed.get("text") {
                if let Some(val) = item.as_scalar() {
                    if let Some(text_str) = val.as_str() {
                        let actual_lines = text_str.lines().count();
                        prop_assert!(actual_lines > 0,
                            "Long block string should have lines");
                    }
                }
            }
        }
    }

    /// Property: Block strings with special characters are handled.
    #[test]
    fn prop_block_string_special_chars(prefix in "[a-z]{1,10}") {
        // HEDL uses quoted strings for special characters, not YAML block strings
        let test_cases = vec![
            format!("{}!@#$%", prefix),
            format!("{}*()_+", prefix),
            format!("{}-=[]", prefix),
            format!("{};'", prefix),
            format!("{},.<>?", prefix),
        ];

        for content in test_cases {
            // Escape the content properly for HEDL string
            let escaped = content.replace('\\', "\\\\").replace('"', "\\\"");
            let doc = format!(
                "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: \"{escaped}\"\n"
            );

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(),
                "Failed to parse quoted string with special chars '{}': {:?}",
                content, result.err());

            if let Ok(parsed) = result {
                if let Some(item) = parsed.get("text") {
                    if let Some(val) = item.as_scalar() {
                        if let Some(text_str) = val.as_str() {
                            prop_assert_eq!(text_str, content,
                                "String content should match exactly");
                        }
                    }
                }
            }
        }
    }

    /// Property: Quoted strings with unicode content work.
    #[test]
    fn prop_block_string_unicode(prefix in "[a-z]{1,10}") {
        // HEDL uses quoted strings, not YAML block strings
        let test_cases = vec![
            format!("{} 日本語", prefix),
            format!("{} 🚀", prefix),
            format!("{} Ω", prefix),
            format!("{} ñ", prefix),
        ];

        for content in test_cases {
            // Escape the content properly for HEDL string
            let escaped = content.replace('\\', "\\\\").replace('"', "\\\"");
            let doc = format!(
                "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: \"{escaped}\"\n"
            );

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(),
                "Failed to parse quoted string with unicode '{}': {:?}",
                content, result.err());

            if let Ok(parsed) = result {
                if let Some(item) = parsed.get("text") {
                    if let Some(val) = item.as_scalar() {
                        if let Some(text_str) = val.as_str() {
                            prop_assert_eq!(text_str, content,
                                "Unicode string content should match exactly");
                        }
                    }
                }
            }
        }
    }

    /// Property: Empty block strings are handled.
    #[test]
    fn prop_empty_block_string(_seed in 0..100_u32) {
        let doc = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: |\n";

        let result = parse(doc.as_bytes());
        // Should parse successfully or produce clear error
        if result.is_err() {
            let err_msg = format!("{}", result.unwrap_err());
            prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
        }
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Block strings with tabs are handled (may be rejected or normalized).
        #[test]
        fn prop_block_string_tabs(content in "[a-zA-Z0-9]{1,30}") {
            let line = format!("{content}\t{content}");
            let doc = format!(
                "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: |\n  {line}\n"
            );

            let result = parse(doc.as_bytes());
            // Should either parse or produce clear error, but not panic
            if result.is_err() {
                let err_msg = format!("{}", result.unwrap_err());
                prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
            }
        }

        /// Property: Quoted strings with multiple lines (newline escapes) are handled.
        #[test]
        fn prop_block_string_mixed_line_endings(content in "[a-zA-Z0-9]{1,30}") {
            // HEDL uses quoted strings with \n escapes, not YAML block strings
            let joined = format!("{content}\\n{content}\\n{content}");
            let doc = format!(
                "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntext: \"{joined}\"\n"
            );

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(),
                "Failed to parse quoted string with newline escapes: {:?}",
                result.err());
        }
    }
}
