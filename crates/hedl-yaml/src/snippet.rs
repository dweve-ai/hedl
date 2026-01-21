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

//! Code snippet extraction for error messages.

use crate::error::Location;

/// Configuration for snippet extraction.
#[derive(Debug, Clone)]
pub struct SnippetConfig {
    /// Number of context lines before and after the error
    pub context_lines: usize,
    /// Maximum line length before truncation
    pub max_line_length: usize,
    /// Whether to show line numbers
    pub show_line_numbers: bool,
}

impl Default for SnippetConfig {
    fn default() -> Self {
        Self {
            context_lines: 2,
            max_line_length: 120,
            show_line_numbers: true,
        }
    }
}

/// Extracts a code snippet from the source at the given location.
///
/// # Arguments
///
/// * `source` - The YAML source text
/// * `location` - The location of the error
/// * `config` - Configuration for the snippet
///
/// # Returns
///
/// A formatted snippet showing the error location with context lines.
pub fn extract_snippet(source: &str, location: &Location, config: &SnippetConfig) -> String {
    let lines: Vec<&str> = source.lines().collect();

    if lines.is_empty() || location.line == 0 {
        return String::new();
    }

    // Convert to 0-indexed
    let error_line_idx = location.line.saturating_sub(1);

    if error_line_idx >= lines.len() {
        return String::new();
    }

    // Calculate start and end lines for context
    let start_line = error_line_idx.saturating_sub(config.context_lines);
    let end_line = (error_line_idx + config.context_lines + 1).min(lines.len());

    let mut result = String::new();

    // Calculate the maximum line number width for alignment
    let max_line_num = end_line;
    let line_num_width = max_line_num.to_string().len();

    for (idx, line_idx) in (start_line..end_line).enumerate() {
        let line = lines[line_idx];
        let line_num = line_idx + 1; // Convert back to 1-indexed for display

        // Truncate long lines
        let display_line = if line.len() > config.max_line_length {
            let mut truncated = line.chars().take(config.max_line_length).collect::<String>();
            truncated.push_str("...");
            truncated
        } else {
            line.to_string()
        };

        if config.show_line_numbers {
            result.push_str(&format!(
                " {:>width$} | {}\n",
                line_num,
                display_line,
                width = line_num_width
            ));
        } else {
            result.push_str(&format!("{}\n", display_line));
        }

        // Add error indicator on the line after the error line
        if line_idx == error_line_idx {
            if config.show_line_numbers {
                result.push_str(&format!(
                    " {:>width$} | ",
                    "",
                    width = line_num_width
                ));
            }

            // Add spaces to align with the error column
            let column = location.column.saturating_sub(1);
            for _ in 0..column {
                result.push(' ');
            }
            result.push_str("^^^ error here\n");
        }
    }

    // Remove trailing newline
    if result.ends_with('\n') {
        result.pop();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_snippet_single_line() {
        let source = "name: value\ncount: 42\nstatus: active\n";
        let location = Location::new(2, 8, 15);
        let config = SnippetConfig::default();

        let snippet = extract_snippet(source, &location, &config);
        assert!(snippet.contains("name: value"));
        assert!(snippet.contains("count: 42"));
        assert!(snippet.contains("status: active"));
        assert!(snippet.contains("^^^ error here"));
    }

    #[test]
    fn test_extract_snippet_with_context() {
        let source = "line1\nline2\nline3\nline4\nline5\n";
        let location = Location::new(3, 1, 12);
        let config = SnippetConfig {
            context_lines: 1,
            ..Default::default()
        };

        let snippet = extract_snippet(source, &location, &config);
        assert!(snippet.contains("line2"));
        assert!(snippet.contains("line3"));
        assert!(snippet.contains("line4"));
        assert!(!snippet.contains("line1"));
        assert!(!snippet.contains("line5"));
    }

    #[test]
    fn test_extract_snippet_at_start() {
        let source = "first\nsecond\nthird\n";
        let location = Location::new(1, 1, 0);
        let config = SnippetConfig::default();

        let snippet = extract_snippet(source, &location, &config);
        assert!(snippet.contains("first"));
        assert!(snippet.contains("second"));
        assert!(snippet.contains("third"));
    }

    #[test]
    fn test_extract_snippet_at_end() {
        let source = "first\nsecond\nthird\n";
        let location = Location::new(3, 1, 13);
        let config = SnippetConfig::default();

        let snippet = extract_snippet(source, &location, &config);
        assert!(snippet.contains("first"));
        assert!(snippet.contains("second"));
        assert!(snippet.contains("third"));
    }

    #[test]
    fn test_extract_snippet_empty_source() {
        let source = "";
        let location = Location::new(1, 1, 0);
        let config = SnippetConfig::default();

        let snippet = extract_snippet(source, &location, &config);
        assert!(snippet.is_empty());
    }

    #[test]
    fn test_extract_snippet_invalid_line() {
        let source = "line1\nline2\n";
        let location = Location::new(100, 1, 1000);
        let config = SnippetConfig::default();

        let snippet = extract_snippet(source, &location, &config);
        assert!(snippet.is_empty());
    }

    #[test]
    fn test_extract_snippet_no_line_numbers() {
        let source = "name: value\ncount: 42\n";
        let location = Location::new(1, 1, 0);
        let config = SnippetConfig {
            show_line_numbers: false,
            ..Default::default()
        };

        let snippet = extract_snippet(source, &location, &config);
        assert!(!snippet.contains(" | "));
        assert!(snippet.contains("name: value"));
    }

    #[test]
    fn test_extract_snippet_long_line() {
        let long_line = "x".repeat(200);
        let source = format!("short\n{}\nshort\n", long_line);
        let location = Location::new(2, 1, 6);
        let config = SnippetConfig::default();

        let snippet = extract_snippet(source, &location, &config);
        assert!(snippet.contains("..."));
        assert!(snippet.len() < source.len());
    }

    #[test]
    fn test_extract_snippet_column_alignment() {
        let source = "users:\n  123: invalid\n";
        let location = Location::new(2, 3, 9);
        let config = SnippetConfig::default();

        let snippet = extract_snippet(source, &location, &config);
        assert!(snippet.contains("^^^ error here"));

        // Count spaces before the error indicator
        let lines: Vec<&str> = snippet.lines().collect();
        let error_line = lines.iter().find(|l| l.contains("^^^ error here")).unwrap();

        // Find position of ^^^
        let spaces_before = error_line.chars().take_while(|&c| c == ' ').count();

        // Should align with column 3 (accounting for line number prefix)
        assert!(spaces_before >= 2);
    }

    #[test]
    fn test_snippet_config_default() {
        let config = SnippetConfig::default();
        assert_eq!(config.context_lines, 2);
        assert_eq!(config.max_line_length, 120);
        assert!(config.show_line_numbers);
    }

    #[test]
    fn test_snippet_config_custom() {
        let config = SnippetConfig {
            context_lines: 5,
            max_line_length: 80,
            show_line_numbers: false,
        };
        assert_eq!(config.context_lines, 5);
        assert_eq!(config.max_line_length, 80);
        assert!(!config.show_line_numbers);
    }
}
