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

//! Block string parsing for multi-line string literals.
//!
//! This module handles the parsing of block strings (multi-line strings delimited by `"""`).
//! Block strings are used in HEDL for long text values that span multiple lines.
//!
//! # Format
//!
//! ```text
//! key: """
//!   Line 1
//!   Line 2
//!   Line 3
//! """
//! ```
//!
//! # Rules
//!
//! - Opening `"""` must be followed by a newline (single-line block strings are not allowed)
//! - Closing `"""` must be on its own line (only whitespace/comments allowed after)
//! - Content between delimiters is preserved as-is, including leading/trailing whitespace
//! - Size limits are enforced to prevent memory exhaustion

use crate::error::{HedlError, HedlResult};
use crate::lex::is_valid_key_token;
use crate::limits::Limits;

/// Result of trying to start a block string.
#[derive(Debug)]
pub(crate) enum BlockStringResult {
    /// Not a block string - parse as normal line.
    NotBlockString,
    /// Multi-line block string started - need to accumulate lines.
    MultiLineStarted(BlockStringState),
}

/// State for parsing multi-line block strings.
#[derive(Debug)]
pub(crate) struct BlockStringState {
    /// The key for the block string value.
    pub key: String,
    /// Accumulated content lines.
    pub content: Vec<String>,
    /// Starting line number (for error messages).
    pub start_line: usize,
    /// Indent level of the key.
    pub indent: usize,
    /// Total accumulated size in bytes.
    pub total_size: usize,
}

impl BlockStringState {
    /// Process a line while accumulating a block string.
    ///
    /// Returns `Some(String)` if the block string is complete (closing `"""` found),
    /// otherwise returns `None` and continues accumulation.
    pub fn process_line(
        &mut self,
        line: &str,
        line_num: usize,
        limits: &Limits,
    ) -> HedlResult<Option<String>> {
        // Check if this line contains the closing """
        if let Some(end_pos) = line.find("\"\"\"") {
            // Found closing - extract content before """
            let before_close = &line[..end_pos];

            // Check size limit before adding
            let new_size = self
                .total_size
                .checked_add(before_close.len())
                .ok_or_else(|| HedlError::security("block string size overflow", line_num))?;
            if new_size > limits.max_block_string_size {
                return Err(HedlError::security(
                    format!(
                        "block string size {} exceeds limit {}",
                        new_size, limits.max_block_string_size
                    ),
                    line_num,
                ));
            }
            self.total_size = new_size;
            self.content.push(before_close.to_string());

            // Validate nothing meaningful after closing """
            let after_close = line[end_pos + 3..].trim();
            if !after_close.is_empty() && !after_close.starts_with('#') {
                return Err(HedlError::syntax(
                    "unexpected content after closing \"\"\"",
                    line_num,
                ));
            }

            // Complete the block string
            let full_content = self.content.join("\n");
            Ok(Some(full_content))
        } else {
            // Check size limit before accumulating
            // Add 1 for the newline that will be added when joining
            let line_contribution = line.len().saturating_add(1);
            let new_size = self
                .total_size
                .checked_add(line_contribution)
                .ok_or_else(|| HedlError::security("block string size overflow", line_num))?;
            if new_size > limits.max_block_string_size {
                return Err(HedlError::security(
                    format!(
                        "block string size {} exceeds limit {}",
                        new_size, limits.max_block_string_size
                    ),
                    line_num,
                ));
            }
            self.total_size = new_size;
            self.content.push(line.to_string());
            Ok(None)
        }
    }
}

/// Try to start a block string if the line contains `key: """`.
///
/// # Arguments
///
/// * `content` - The line content (after indentation is stripped)
/// * `line_num` - Current line number for error reporting
///
/// # Returns
///
/// - `BlockStringResult::MultiLineStarted` if a block string was started
/// - `BlockStringResult::NotBlockString` if this is not a block string
///
/// # Errors
///
/// Returns an error if:
/// - The key is invalid
/// - Content appears on the same line as opening `"""`
pub(crate) fn try_start_block_string(
    content: &str,
    indent: usize,
    line_num: usize,
) -> HedlResult<BlockStringResult> {
    // Look for key: """
    let Some(colon_pos) = content.find(':') else {
        return Ok(BlockStringResult::NotBlockString);
    };

    let key = content[..colon_pos].trim();
    let after_colon = &content[colon_pos + 1..];

    // Need space after colon
    if !after_colon.is_empty() && !after_colon.starts_with(' ') {
        return Ok(BlockStringResult::NotBlockString);
    }

    let value_str = after_colon.trim();

    // Check for """ start
    if !value_str.starts_with("\"\"\"") {
        return Ok(BlockStringResult::NotBlockString);
    }

    // Validate key
    if !is_valid_key_token(key) {
        return Err(HedlError::syntax(
            format!("invalid key: '{}'", key),
            line_num,
        ));
    }

    let after_open = &value_str[3..]; // Skip opening """

    // Per SPEC Section 8.2: After opening """, there MUST be a newline.
    // Single-line block strings like """content""" are NOT valid HEDL.
    let after_open_trimmed = after_open.trim_start();
    if after_open_trimmed.is_empty() || after_open_trimmed.starts_with('#') {
        // OK: nothing on this line after """ (will start multi-line)
        // Preserve whatever is after """ on this line (usually empty or whitespace/comment)
        Ok(BlockStringResult::MultiLineStarted(BlockStringState {
            key: key.to_string(),
            content: vec![after_open.to_string()],
            start_line: line_num,
            indent,
            total_size: after_open.len(),
        }))
    } else {
        // ERROR: content on same line as opening """
        Err(HedlError::syntax(
            "block string must have newline after opening \"\"\" (single-line block strings are not allowed)",
            line_num,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_block_string_no_colon() {
        let result = try_start_block_string("just text", 0, 1).unwrap();
        assert!(matches!(result, BlockStringResult::NotBlockString));
    }

    #[test]
    fn test_not_block_string_no_triple_quote() {
        let result = try_start_block_string("key: value", 0, 1).unwrap();
        assert!(matches!(result, BlockStringResult::NotBlockString));
    }

    #[test]
    fn test_not_block_string_double_quote() {
        let result = try_start_block_string("key: \"value\"", 0, 1).unwrap();
        assert!(matches!(result, BlockStringResult::NotBlockString));
    }

    #[test]
    fn test_not_block_string_no_space_after_colon() {
        let result = try_start_block_string("key:\"\"\"", 0, 1).unwrap();
        assert!(matches!(result, BlockStringResult::NotBlockString));
    }

    #[test]
    fn test_valid_block_string_start() {
        let result = try_start_block_string("description: \"\"\"", 0, 1).unwrap();
        match result {
            BlockStringResult::MultiLineStarted(state) => {
                assert_eq!(state.key, "description");
                assert_eq!(state.start_line, 1);
                assert_eq!(state.indent, 0);
                assert_eq!(state.content.len(), 1);
            }
            BlockStringResult::NotBlockString => panic!("Expected block string to start"),
        }
    }

    #[test]
    fn test_valid_block_string_with_comment() {
        let result = try_start_block_string("key: \"\"\" # comment", 0, 1).unwrap();
        match result {
            BlockStringResult::MultiLineStarted(state) => {
                assert_eq!(state.key, "key");
            }
            BlockStringResult::NotBlockString => panic!("Expected block string to start"),
        }
    }

    #[test]
    fn test_invalid_key() {
        let result = try_start_block_string("123invalid: \"\"\"", 0, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid key"));
    }

    #[test]
    fn test_content_after_opening_quotes() {
        let result = try_start_block_string("key: \"\"\"content", 0, 1);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have newline after opening"));
    }

    #[test]
    fn test_process_line_accumulation() {
        let mut state = BlockStringState {
            key: "test".to_string(),
            content: vec!["".to_string()],
            start_line: 1,
            indent: 0,
            total_size: 0,
        };
        let limits = Limits::default();

        let result = state.process_line("  Line 1", 2, &limits).unwrap();
        assert!(result.is_none()); // Still accumulating
        assert_eq!(state.content.len(), 2);
        assert_eq!(state.content[1], "  Line 1");
    }

    #[test]
    fn test_process_line_closing() {
        let mut state = BlockStringState {
            key: "test".to_string(),
            content: vec!["".to_string(), "Line 1".to_string()],
            start_line: 1,
            indent: 0,
            total_size: 7,
        };
        let limits = Limits::default();

        let result = state.process_line("\"\"\"", 3, &limits).unwrap();
        assert!(result.is_some());
        let content = result.unwrap();
        assert_eq!(content, "\nLine 1\n");
    }

    #[test]
    fn test_process_line_closing_with_content_before() {
        let mut state = BlockStringState {
            key: "test".to_string(),
            content: vec!["".to_string(), "Line 1".to_string()],
            start_line: 1,
            indent: 0,
            total_size: 7,
        };
        let limits = Limits::default();

        let result = state.process_line("Line 2\"\"\"", 3, &limits).unwrap();
        assert!(result.is_some());
        let content = result.unwrap();
        assert_eq!(content, "\nLine 1\nLine 2");
    }

    #[test]
    fn test_process_line_closing_with_comment() {
        let mut state = BlockStringState {
            key: "test".to_string(),
            content: vec!["".to_string(), "Line 1".to_string()],
            start_line: 1,
            indent: 0,
            total_size: 7,
        };
        let limits = Limits::default();

        let result = state.process_line("\"\"\" # comment", 3, &limits).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_process_line_closing_with_invalid_content_after() {
        let mut state = BlockStringState {
            key: "test".to_string(),
            content: vec!["".to_string()],
            start_line: 1,
            indent: 0,
            total_size: 1,
        };
        let limits = Limits::default();

        let result = state.process_line("\"\"\" invalid", 3, &limits);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unexpected content after closing"));
    }

    #[test]
    fn test_size_limit_exceeded_during_accumulation() {
        let mut state = BlockStringState {
            key: "test".to_string(),
            content: vec!["".to_string()],
            start_line: 1,
            indent: 0,
            total_size: 0,
        };
        let mut limits = Limits::default();
        limits.max_block_string_size = 10; // Very small limit

        let result = state.process_line("This is a very long line that exceeds the limit", 2, &limits);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds limit"));
    }

    #[test]
    fn test_size_limit_exceeded_at_closing() {
        let mut state = BlockStringState {
            key: "test".to_string(),
            content: vec!["".to_string()],
            start_line: 1,
            indent: 0,
            total_size: 5,
        };
        let mut limits = Limits::default();
        limits.max_block_string_size = 10;

        let result = state.process_line("Long content before closing\"\"\"", 2, &limits);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds limit"));
    }

    #[test]
    fn test_preserve_empty_lines() {
        let mut state = BlockStringState {
            key: "test".to_string(),
            content: vec!["".to_string(), "Line 1".to_string()],
            start_line: 1,
            indent: 0,
            total_size: 7,
        };
        let limits = Limits::default();

        state.process_line("", 3, &limits).unwrap();
        state.process_line("Line 3", 4, &limits).unwrap();
        let result = state.process_line("\"\"\"", 5, &limits).unwrap();

        let content = result.unwrap();
        assert_eq!(content, "\nLine 1\n\nLine 3\n");
    }

    #[test]
    fn test_preserve_indentation() {
        let mut state = BlockStringState {
            key: "test".to_string(),
            content: vec!["".to_string()],
            start_line: 1,
            indent: 0,
            total_size: 1,
        };
        let limits = Limits::default();

        state.process_line("  indented", 2, &limits).unwrap();
        state.process_line("    more indented", 3, &limits).unwrap();
        let result = state.process_line("\"\"\"", 4, &limits).unwrap();

        let content = result.unwrap();
        assert_eq!(content, "\n  indented\n    more indented\n");
    }
}
