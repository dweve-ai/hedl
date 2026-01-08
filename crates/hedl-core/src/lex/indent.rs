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

//! Indentation handling for HEDL.

use super::error::LexError;
use super::span::SourcePos;

/// Information about a line's indentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndentInfo {
    /// Number of leading spaces.
    pub spaces: usize,
    /// Calculated indent level (spaces / 2).
    pub level: usize,
}

/// Calculate indentation info from a line.
///
/// Returns `None` if the line is blank (only whitespace).
/// Returns error if indentation uses tabs or odd number of spaces.
///
/// # Parameters
/// - `line`: The line to analyze
/// - `line_num`: Line number (1-indexed) for error reporting
pub fn calculate_indent(line: &str, line_num: u32) -> Result<Option<IndentInfo>, LexError> {
    let bytes = line.as_bytes();
    let mut spaces = 0;

    // Count leading spaces and detect tabs in indentation
    for &b in bytes {
        match b {
            b' ' => spaces += 1,
            b'\t' => {
                // Tab found - check if line is blank after this point (use bytes for speed)
                if bytes[spaces..].iter().all(|&b| b.is_ascii_whitespace()) {
                    return Ok(None);
                }
                return Err(LexError::TabInIndentation {
                    pos: SourcePos::new(line_num as usize, spaces + 1),
                });
            }
            _ => break,
        }
    }

    // Check if line is blank (only spaces or all whitespace) - use bytes for speed
    if spaces == bytes.len() || bytes[spaces..].iter().all(|&b| b.is_ascii_whitespace()) {
        return Ok(None);
    }

    // Validate even number of spaces
    if spaces % 2 != 0 {
        return Err(LexError::InvalidIndentation {
            spaces,
            pos: SourcePos::new(line_num as usize, 1),
        });
    }

    Ok(Some(IndentInfo {
        spaces,
        level: spaces / 2,
    }))
}

/// Validate that indent level doesn't exceed maximum.
///
/// # Parameters
/// - `info`: Indentation information to validate
/// - `max_depth`: Maximum allowed indentation depth
/// - `line_num`: Line number (1-indexed) for error reporting
pub fn validate_indent(
    info: IndentInfo,
    max_depth: usize,
    line_num: u32,
) -> Result<(), LexError> {
    if info.level > max_depth {
        return Err(LexError::IndentTooDeep {
            depth: info.level,
            max: max_depth,
            pos: SourcePos::new(line_num as usize, 1),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== calculate_indent: valid cases ====================

    #[test]
    fn test_calculate_indent_zero() {
        let result = calculate_indent("hello", 1).unwrap().unwrap();
        assert_eq!(result.spaces, 0);
        assert_eq!(result.level, 0);
    }

    #[test]
    fn test_calculate_indent_level_1() {
        let result = calculate_indent("  hello", 5).unwrap().unwrap();
        assert_eq!(result.spaces, 2);
        assert_eq!(result.level, 1);
    }

    #[test]
    fn test_calculate_indent_level_2() {
        let result = calculate_indent("    hello", 10).unwrap().unwrap();
        assert_eq!(result.spaces, 4);
        assert_eq!(result.level, 2);
    }

    #[test]
    fn test_calculate_indent_deep_nesting() {
        let result = calculate_indent("          hello", 15).unwrap().unwrap();
        assert_eq!(result.spaces, 10);
        assert_eq!(result.level, 5);

        let result = calculate_indent("                    hello", 20)
            .unwrap()
            .unwrap();
        assert_eq!(result.spaces, 20);
        assert_eq!(result.level, 10);
    }

    #[test]
    fn test_calculate_indent_various_content() {
        // Content with special characters
        assert_eq!(calculate_indent("  key: value", 1).unwrap().unwrap().level, 1);
        assert_eq!(calculate_indent("  | row, data", 1).unwrap().unwrap().level, 1);
        assert_eq!(calculate_indent("  @reference", 1).unwrap().unwrap().level, 1);
        assert_eq!(calculate_indent("  # comment", 1).unwrap().unwrap().level, 1);
    }

    #[test]
    fn test_calculate_indent_unicode_content() {
        // Unicode content should work fine
        let result = calculate_indent("  日本語", 3).unwrap().unwrap();
        assert_eq!(result.spaces, 2);
        assert_eq!(result.level, 1);

        let result = calculate_indent("    émoji 😀", 7).unwrap().unwrap();
        assert_eq!(result.spaces, 4);
        assert_eq!(result.level, 2);
    }

    // ==================== calculate_indent: blank lines ====================

    #[test]
    fn test_blank_line_empty() {
        assert!(calculate_indent("", 1).unwrap().is_none());
    }

    #[test]
    fn test_blank_line_spaces_only() {
        assert!(calculate_indent("   ", 2).unwrap().is_none());
        assert!(calculate_indent("  ", 3).unwrap().is_none());
        assert!(calculate_indent(" ", 4).unwrap().is_none());
        assert!(calculate_indent("          ", 5).unwrap().is_none());
    }

    #[test]
    fn test_blank_line_mixed_whitespace() {
        // Tab after spaces in a blank line should be treated as blank
        assert!(calculate_indent("  \t  ", 6).unwrap().is_none());
        assert!(calculate_indent("    \t", 7).unwrap().is_none());
    }

    #[test]
    fn test_blank_line_with_trailing_whitespace() {
        // Trailing whitespace only
        assert!(calculate_indent("     \t   ", 8).unwrap().is_none());
    }

    // ==================== calculate_indent: odd indentation errors ====================

    #[test]
    fn test_odd_indent_1_space() {
        let result = calculate_indent(" hello", 42);
        assert!(matches!(
            result,
            Err(LexError::InvalidIndentation { spaces: 1, pos }) if pos.line() == 42
        ));
    }

    #[test]
    fn test_odd_indent_3_spaces() {
        let result = calculate_indent("   hello", 15);
        assert!(matches!(
            result,
            Err(LexError::InvalidIndentation { spaces: 3, pos }) if pos.line() == 15
        ));
    }

    #[test]
    fn test_odd_indent_5_spaces() {
        let result = calculate_indent("     hello", 20);
        assert!(matches!(
            result,
            Err(LexError::InvalidIndentation { spaces: 5, pos }) if pos.line() == 20
        ));
    }

    #[test]
    fn test_odd_indent_various() {
        for odd in [1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21] {
            let line = format!("{}hello", " ".repeat(odd));
            let result = calculate_indent(&line, 100);
            assert!(
                matches!(result, Err(LexError::InvalidIndentation { spaces, pos }) if spaces == odd && pos.line() == 100),
                "Expected InvalidIndentation for {} spaces at line 100",
                odd
            );
        }
    }

    // ==================== calculate_indent: tab errors ====================

    #[test]
    fn test_tab_at_start() {
        let result = calculate_indent("\thello", 10);
        assert!(matches!(
            result,
            Err(LexError::TabInIndentation { pos }) if pos.line() == 10 && pos.column() == 1
        ));
    }

    #[test]
    fn test_tab_after_spaces() {
        let result = calculate_indent("  \thello", 25);
        assert!(matches!(
            result,
            Err(LexError::TabInIndentation { pos }) if pos.line() == 25 && pos.column() == 3
        ));
    }

    #[test]
    fn test_tab_mixed_with_spaces() {
        let result = calculate_indent(" \t hello", 30);
        assert!(matches!(
            result,
            Err(LexError::TabInIndentation { pos }) if pos.line() == 30 && pos.column() == 2
        ));
    }

    #[test]
    fn test_multiple_tabs() {
        let result = calculate_indent("\t\thello", 50);
        assert!(matches!(
            result,
            Err(LexError::TabInIndentation { pos }) if pos.line() == 50
        ));
    }

    // ==================== validate_indent ====================

    #[test]
    fn test_validate_indent_within_max() {
        let info = IndentInfo {
            spaces: 4,
            level: 2,
        };
        assert!(validate_indent(info, 10, 5).is_ok());
        assert!(validate_indent(info, 2, 10).is_ok());
    }

    #[test]
    fn test_validate_indent_at_max() {
        let info = IndentInfo {
            spaces: 20,
            level: 10,
        };
        assert!(validate_indent(info, 10, 15).is_ok());
    }

    #[test]
    fn test_validate_indent_exceeds_max() {
        let info = IndentInfo {
            spaces: 22,
            level: 11,
        };
        let result = validate_indent(info, 10, 42);
        assert!(matches!(
            result,
            Err(LexError::IndentTooDeep { depth: 11, max: 10, pos }) if pos.line() == 42
        ));
    }

    #[test]
    fn test_validate_indent_zero_max() {
        let info = IndentInfo {
            spaces: 0,
            level: 0,
        };
        assert!(validate_indent(info, 0, 1).is_ok());

        let info = IndentInfo {
            spaces: 2,
            level: 1,
        };
        let result = validate_indent(info, 0, 8);
        assert!(matches!(
            result,
            Err(LexError::IndentTooDeep { depth: 1, max: 0, pos }) if pos.line() == 8
        ));
    }

    // ==================== IndentInfo struct ====================

    #[test]
    fn test_indent_info_equality() {
        let a = IndentInfo {
            spaces: 4,
            level: 2,
        };
        let b = IndentInfo {
            spaces: 4,
            level: 2,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_indent_info_inequality() {
        let a = IndentInfo {
            spaces: 4,
            level: 2,
        };
        let b = IndentInfo {
            spaces: 2,
            level: 1,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_indent_info_clone() {
        let a = IndentInfo {
            spaces: 6,
            level: 3,
        };
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_indent_info_debug() {
        let info = IndentInfo {
            spaces: 4,
            level: 2,
        };
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("spaces: 4"));
        assert!(debug_str.contains("level: 2"));
    }
}
