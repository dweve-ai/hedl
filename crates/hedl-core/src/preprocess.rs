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

//! Input preprocessing for HEDL parsing.

use crate::error::{HedlError, HedlResult};
use crate::limits::Limits;
use std::borrow::Cow;

/// Preprocessed input ready for parsing.
/// Uses zero-copy design - stores normalized text and line offsets.
#[derive(Debug)]
pub struct PreprocessedInput {
    /// The normalized text (owned if CRLF conversion was needed, borrowed otherwise)
    text: String,
    /// Line boundaries: Vec of (line_number, start_offset, end_offset)
    line_offsets: Vec<(usize, usize, usize)>,
}

impl PreprocessedInput {
    /// Get lines as (line_num, &str) iterator - zero allocation
    #[inline]
    pub fn lines(&self) -> impl Iterator<Item = (usize, &str)> {
        self.line_offsets
            .iter()
            .map(move |&(num, start, end)| (num, &self.text[start..end]))
    }
}

/// Preprocess raw input bytes into lines.
///
/// This handles:
/// - UTF-8 validation
/// - BOM skipping
/// - CRLF normalization
/// - Bare CR rejection
/// - Control character validation
/// - Size and line length limits
pub fn preprocess(input: &[u8], limits: &Limits) -> HedlResult<PreprocessedInput> {
    // Check file size (don't reveal exact input size to avoid information disclosure)
    if input.len() > limits.max_file_size {
        return Err(HedlError::security(
            format!("file too large: exceeds limit of {} bytes", limits.max_file_size),
            0,
        ));
    }

    // Validate and decode UTF-8
    let text = std::str::from_utf8(input)
        .map_err(|e| HedlError::syntax(format!("invalid UTF-8 encoding: {}", e), 1))?;

    // Skip BOM if present
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);

    // Check for control characters (allow LF, CR, TAB)
    // Fast path: scan bytes for control chars (0x00-0x1F except 0x09, 0x0A, 0x0D)
    // P0 OPTIMIZATION: Track line number during scan (1000x speedup for errors deep in files)
    let bytes = text.as_bytes();
    let mut line_num = 1;
    for &b in bytes.iter() {
        if b == b'\n' {
            line_num += 1;
        } else if b < 0x20 && b != 0x09 && b != 0x0D {
            return Err(HedlError::syntax(
                format!("control character U+{:04X} not allowed", b),
                line_num,
            ));
        }
    }

    // Normalize line endings and check for bare CR
    // Use Cow to avoid allocation when no CRLF present
    let text: Cow<str> = if text.contains('\r') {
        let normalized = text.replace("\r\n", "\n");
        if normalized.contains('\r') {
            let line_num = normalized[..normalized.find('\r').unwrap()]
                .matches('\n')
                .count()
                + 1;
            return Err(HedlError::syntax(
                "bare CR (U+000D) not allowed - use LF or CRLF",
                line_num,
            ));
        }
        Cow::Owned(normalized)
    } else {
        Cow::Borrowed(text)
    };

    // Split into lines and validate lengths - zero copy using offsets
    // Pre-allocate with estimated line count (avoid reallocs)
    let text_ref = text.as_ref();
    let bytes = text_ref.as_bytes();
    let estimated_lines = bytes.iter().filter(|&&b| b == b'\n').count() + 1;
    let mut line_offsets = Vec::with_capacity(estimated_lines);

    let mut start = 0;
    let mut line_num = 1;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            let line_len = i - start;
            if line_len > limits.max_line_length {
                return Err(HedlError::security(
                    format!(
                        "line too long: exceeds limit of {} bytes",
                        limits.max_line_length
                    ),
                    line_num,
                ));
            }
            line_offsets.push((line_num, start, i));
            start = i + 1;
            line_num += 1;
        }
    }

    // Handle last line (no trailing newline)
    if start <= bytes.len() {
        let line_len = bytes.len() - start;
        if line_len > limits.max_line_length {
            return Err(HedlError::security(
                format!(
                    "line too long: exceeds limit of {} bytes",
                    limits.max_line_length
                ),
                line_num,
            ));
        }
        line_offsets.push((line_num, start, bytes.len()));
    }

    // Convert Cow to owned String for storage
    let text_owned = text.into_owned();

    Ok(PreprocessedInput {
        text: text_owned,
        line_offsets,
    })
}

/// Check if a line is blank (empty or whitespace only).
pub fn is_blank_line(line: &str) -> bool {
    line.trim().is_empty()
}

/// Check if a line is a comment (first non-whitespace is #).
pub fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_limits() -> Limits {
        Limits::default()
    }

    // ==================== Basic preprocessing tests ====================

    #[test]
    fn test_preprocess_simple() {
        let input = b"%VERSION: 1.0\n---\na: 1\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], (1, "%VERSION: 1.0"));
    }

    #[test]
    fn test_preprocess_single_line() {
        let input = b"hello";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], (1, "hello"));
    }

    #[test]
    fn test_preprocess_empty_input() {
        let input = b"";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], (1, ""));
    }

    #[test]
    fn test_preprocess_only_newline() {
        let input = b"\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], (1, ""));
        assert_eq!(lines[1], (2, ""));
    }

    #[test]
    fn test_preprocess_multiple_newlines() {
        let input = b"\n\n\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn test_preprocess_line_numbers() {
        let input = b"a\nb\nc\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines[0].0, 1);
        assert_eq!(lines[1].0, 2);
        assert_eq!(lines[2].0, 3);
    }

    // ==================== Line ending tests ====================

    #[test]
    fn test_preprocess_crlf() {
        let input = b"%VERSION: 1.0\r\n---\r\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines[0].1, "%VERSION: 1.0");
    }

    #[test]
    fn test_preprocess_mixed_line_endings() {
        let input = b"line1\nline2\r\nline3\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines[0].1, "line1");
        assert_eq!(lines[1].1, "line2");
        assert_eq!(lines[2].1, "line3");
    }

    #[test]
    fn test_preprocess_bare_cr_error() {
        let input = b"line1\rline2\n";
        let result = preprocess(input, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("bare CR"));
    }

    #[test]
    fn test_preprocess_cr_at_end_error() {
        let input = b"line1\r";
        let result = preprocess(input, &default_limits());
        assert!(result.is_err());
    }

    #[test]
    fn test_preprocess_crlf_only() {
        let input = b"\r\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    // ==================== BOM tests ====================

    #[test]
    fn test_preprocess_bom_skip() {
        let input = b"\xEF\xBB\xBF%VERSION: 1.0\n---\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines[0].1, "%VERSION: 1.0");
    }

    #[test]
    fn test_preprocess_bom_only() {
        let input = b"\xEF\xBB\xBF";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].1, "");
    }

    #[test]
    fn test_preprocess_bom_with_content() {
        let input = b"\xEF\xBB\xBFhello\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines[0].1, "hello");
    }

    // ==================== UTF-8 validation tests ====================

    #[test]
    fn test_preprocess_valid_utf8() {
        let input = "こんにちは\n".as_bytes();
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines[0].1, "こんにちは");
    }

    #[test]
    fn test_preprocess_emoji() {
        let input = "😀🎉🚀\n".as_bytes();
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines[0].1, "😀🎉🚀");
    }

    #[test]
    fn test_preprocess_invalid_utf8_error() {
        let input = b"\xFF\xFE";
        let result = preprocess(input, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("UTF-8"));
    }

    #[test]
    fn test_preprocess_truncated_utf8_error() {
        let input = b"\xC0"; // Incomplete UTF-8 sequence
        let result = preprocess(input, &default_limits());
        assert!(result.is_err());
    }

    // ==================== Control character tests ====================

    #[test]
    fn test_preprocess_tab_allowed() {
        let input = b"a\tb\tc\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert!(lines[0].1.contains('\t'));
    }

    #[test]
    fn test_preprocess_null_char_error() {
        let input = b"hello\x00world\n";
        let result = preprocess(input, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("U+0000"));
    }

    #[test]
    fn test_preprocess_bell_char_error() {
        let input = b"hello\x07world\n";
        let result = preprocess(input, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("U+0007"));
    }

    #[test]
    fn test_preprocess_backspace_char_error() {
        let input = b"hello\x08world\n";
        let result = preprocess(input, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("U+0008"));
    }

    #[test]
    fn test_preprocess_escape_char_error() {
        let input = b"hello\x1Bworld\n";
        let result = preprocess(input, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("U+001B"));
    }

    #[test]
    fn test_preprocess_control_char_line_number() {
        let input = b"line1\nline2\x00\n";
        let result = preprocess(input, &default_limits());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.line, 2);
    }

    // ==================== Size limit tests ====================

    #[test]
    fn test_preprocess_file_size_limit() {
        let limits = Limits {
            max_file_size: 10,
            ..Limits::default()
        };
        let input = b"12345678901"; // 11 bytes
        let result = preprocess(input, &limits);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("file too large"));
    }

    #[test]
    fn test_preprocess_file_size_at_limit() {
        let limits = Limits {
            max_file_size: 10,
            ..Limits::default()
        };
        let input = b"1234567890"; // exactly 10 bytes
        let result = preprocess(input, &limits);
        assert!(result.is_ok());
    }

    #[test]
    fn test_preprocess_line_length_limit() {
        let limits = Limits {
            max_line_length: 5,
            ..Limits::default()
        };
        let input = b"123456\n"; // 6 chars before newline
        let result = preprocess(input, &limits);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("line too long"));
    }

    #[test]
    fn test_preprocess_line_length_at_limit() {
        let limits = Limits {
            max_line_length: 5,
            ..Limits::default()
        };
        let input = b"12345\n"; // exactly 5 chars
        let result = preprocess(input, &limits);
        assert!(result.is_ok());
    }

    #[test]
    fn test_preprocess_last_line_length_limit() {
        let limits = Limits {
            max_line_length: 5,
            ..Limits::default()
        };
        let input = b"abc\n123456"; // last line too long
        let result = preprocess(input, &limits);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.line, 2);
    }

    // ==================== is_blank_line tests ====================

    #[test]
    fn test_is_blank_line() {
        assert!(is_blank_line(""));
        assert!(is_blank_line("   "));
        assert!(is_blank_line("\t  "));
        assert!(!is_blank_line("a"));
    }

    #[test]
    fn test_is_blank_line_with_tabs() {
        assert!(is_blank_line("\t"));
        assert!(is_blank_line("\t\t\t"));
        assert!(is_blank_line("  \t  "));
    }

    #[test]
    fn test_is_blank_line_with_content() {
        assert!(!is_blank_line("x"));
        assert!(!is_blank_line(" x "));
        assert!(!is_blank_line("\tx"));
    }

    #[test]
    fn test_is_blank_line_unicode() {
        // Regular space is blank
        assert!(is_blank_line("   "));
    }

    // ==================== is_comment_line tests ====================

    #[test]
    fn test_is_comment_line() {
        assert!(is_comment_line("# comment"));
        assert!(is_comment_line("  # indented comment"));
        assert!(!is_comment_line("a: 1 # inline"));
    }

    #[test]
    fn test_is_comment_line_hash_only() {
        assert!(is_comment_line("#"));
        assert!(is_comment_line("  #"));
    }

    #[test]
    fn test_is_comment_line_empty_comment() {
        assert!(is_comment_line("# "));
        assert!(is_comment_line("#\t"));
    }

    #[test]
    fn test_is_comment_line_not_comment() {
        assert!(!is_comment_line(""));
        assert!(!is_comment_line("   "));
        assert!(!is_comment_line("key: value"));
        assert!(!is_comment_line("key: #value")); // # not at start
    }

    #[test]
    fn test_is_comment_line_with_tabs() {
        assert!(is_comment_line("\t#comment"));
        assert!(is_comment_line("\t\t# comment"));
    }

    // ==================== PreprocessedInput tests ====================

    #[test]
    fn test_preprocessed_input_lines_iterator() {
        let input = b"line1\nline2\nline3\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn test_preprocessed_input_debug() {
        let input = b"test\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let debug = format!("{:?}", result);
        assert!(debug.contains("PreprocessedInput"));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_preprocess_very_long_line_ok() {
        let long_line = "x".repeat(1000);
        let input = format!("{}\n", long_line);
        let result = preprocess(input.as_bytes(), &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines[0].1.len(), 1000);
    }

    #[test]
    fn test_preprocess_many_lines() {
        let input = (0..100)
            .map(|i| format!("line{}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let result = preprocess(input.as_bytes(), &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 100);
    }

    #[test]
    fn test_preprocess_trailing_newline_preserved() {
        let input = b"line\n";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].1, "");
    }

    #[test]
    fn test_preprocess_no_trailing_newline() {
        let input = b"line";
        let result = preprocess(input, &default_limits()).unwrap();
        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].1, "line");
    }
}
