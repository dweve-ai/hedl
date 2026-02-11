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

//! Line reader for streaming parser.
//!
//! Provides buffered line-by-line reading with line number tracking, peek support,
//! and the ability to push back lines for re-parsing.
//!
//! This module is primarily an internal implementation detail of the streaming
//! parser, but is exposed for advanced use cases.

use crate::error::{StreamError, StreamResult};
use memchr::memchr;
use std::io::{BufRead, BufReader, Read};

/// Buffered line reader with line number tracking.
///
/// Reads input line-by-line, automatically handling different line endings
/// (LF, CRLF) and tracking the current line number for error reporting.
///
/// # Features
///
/// - **Buffered I/O**: Efficient reading with configurable buffer size
/// - **Line Number Tracking**: Automatic tracking for error messages
/// - **Peek Support**: Look ahead without consuming lines
/// - **Push Back**: Re-read previously consumed lines
/// - **Iterator**: Standard Rust iterator interface
///
/// # Examples
///
/// ## Basic Line Reading
///
/// ```rust
/// use hedl_stream::LineReader;
/// use std::io::Cursor;
///
/// let input = "line1\nline2\nline3";
/// let mut reader = LineReader::new(Cursor::new(input));
///
/// assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
/// assert_eq!(reader.next_line().unwrap(), Some((2, "line2".to_string())));
/// assert_eq!(reader.next_line().unwrap(), Some((3, "line3".to_string())));
/// assert_eq!(reader.next_line().unwrap(), None);
/// ```
///
/// ## Peeking Ahead
///
/// ```rust
/// use hedl_stream::LineReader;
/// use std::io::Cursor;
///
/// let input = "line1\nline2";
/// let mut reader = LineReader::new(Cursor::new(input));
///
/// // Peek without consuming
/// assert_eq!(reader.peek_line().unwrap(), Some(&(1, "line1".to_string())));
/// assert_eq!(reader.peek_line().unwrap(), Some(&(1, "line1".to_string())));
///
/// // Now consume it
/// assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
/// ```
///
/// ## Push Back for Re-parsing
///
/// ```rust
/// use hedl_stream::LineReader;
/// use std::io::Cursor;
///
/// let input = "line1\nline2";
/// let mut reader = LineReader::new(Cursor::new(input));
///
/// let line = reader.next_line().unwrap().unwrap();
/// assert_eq!(line, (1, "line1".to_string()));
///
/// // Push it back
/// reader.push_back(line.0, line.1);
///
/// // Read it again
/// assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
/// ```
pub struct LineReader<R: Read> {
    reader: BufReader<R>,
    line_number: usize,
    buffer: String,
    peeked: Option<(usize, String)>,
    max_line_length: usize,
}

impl<R: Read> LineReader<R> {
    /// Create a new line reader with default max line length (1MB).
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_number: 0,
            buffer: String::new(),
            peeked: None,
            max_line_length: 1_000_000,
        }
    }

    /// Create with a specific buffer capacity and default max line length (1MB).
    pub fn with_capacity(reader: R, capacity: usize) -> Self {
        Self {
            reader: BufReader::with_capacity(capacity, reader),
            line_number: 0,
            buffer: String::new(),
            peeked: None,
            max_line_length: 1_000_000,
        }
    }

    /// Create with a specific max line length.
    pub fn with_max_length(reader: R, max_line_length: usize) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_number: 0,
            buffer: String::new(),
            peeked: None,
            max_line_length,
        }
    }

    /// Create with a specific buffer capacity and max line length.
    pub fn with_capacity_and_max_length(
        reader: R,
        capacity: usize,
        max_line_length: usize,
    ) -> Self {
        Self {
            reader: BufReader::with_capacity(capacity, reader),
            line_number: 0,
            buffer: String::new(),
            peeked: None,
            max_line_length,
        }
    }

    /// Get the current line number.
    #[inline]
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    /// Read the next line.
    pub fn next_line(&mut self) -> StreamResult<Option<(usize, String)>> {
        // Return peeked line if available
        if let Some(peeked) = self.peeked.take() {
            return Ok(Some(peeked));
        }

        self.read_line_internal()
    }

    /// Peek at the next line without consuming it.
    pub fn peek_line(&mut self) -> StreamResult<Option<&(usize, String)>> {
        if self.peeked.is_none() {
            self.peeked = self.read_line_internal()?;
        }
        Ok(self.peeked.as_ref())
    }

    /// Push a line back to be read again.
    #[inline]
    pub fn push_back(&mut self, line_num: usize, line: String) {
        self.peeked = Some((line_num, line));
    }

    fn read_line_internal(&mut self) -> StreamResult<Option<(usize, String)>> {
        self.buffer.clear();

        loop {
            // Read from BufReader's internal buffer (zero-copy)
            let available = match self.reader.fill_buf() {
                Ok(buf) => buf,
                Err(e) => return Err(StreamError::Io(e)),
            };

            if available.is_empty() {
                // EOF
                if self.buffer.is_empty() {
                    return Ok(None);
                }
                // Return partial line (no trailing newline)
                self.line_number += 1;
                return Ok(Some((self.line_number, self.buffer.clone())));
            }

            // Find newline in available data
            if let Some(newline_pos) = memchr(b'\n', available) {
                // Check limit BEFORE appending
                if self.buffer.len() + newline_pos > self.max_line_length {
                    // CRITICAL: Consume the oversized line data to prevent infinite loop
                    // Consume up to and including the newline character
                    self.reader.consume(newline_pos + 1);
                    let total_length = self.buffer.len() + newline_pos;
                    self.line_number += 1;
                    self.buffer.clear();
                    return Err(StreamError::LineTooLong {
                        line: self.line_number,
                        length: total_length,
                        limit: self.max_line_length,
                    });
                }

                // Append up to newline (excluding the newline itself)
                let _line_data = &available[..newline_pos];
                let mut line_end = newline_pos;

                // Handle CRLF: if newline is preceded by CR, exclude it too
                if newline_pos > 0 && available[newline_pos - 1] == b'\r' {
                    line_end = newline_pos - 1;
                }

                let to_append = &available[..line_end];

                // Validate UTF-8 before appending
                let line_str =
                    std::str::from_utf8(to_append).map_err(|e| StreamError::InvalidUtf8 {
                        line: self.line_number + 1,
                        error: e,
                    })?;

                self.buffer.push_str(line_str);

                // Consume bytes including newline
                self.reader.consume(newline_pos + 1);

                self.line_number += 1;
                return Ok(Some((self.line_number, self.buffer.clone())));
            } else {
                // No newline yet, check if adding entire buffer exceeds limit
                if self.buffer.len() + available.len() > self.max_line_length {
                    // CRITICAL: Consume all available data and skip to end of line
                    // to prevent infinite loop on subsequent reads
                    let accumulated = self.buffer.len() + available.len();
                    let consumed = available.len();
                    self.reader.consume(consumed);

                    // Continue reading and discarding until we find the end of line
                    self.skip_to_end_of_line()?;

                    self.line_number += 1;
                    self.buffer.clear();
                    return Err(StreamError::LineTooLong {
                        line: self.line_number,
                        length: accumulated,
                        limit: self.max_line_length,
                    });
                }

                // Validate UTF-8 before appending
                let chunk_str =
                    std::str::from_utf8(available).map_err(|e| StreamError::InvalidUtf8 {
                        line: self.line_number + 1,
                        error: e,
                    })?;

                // Append entire buffer and continue reading
                self.buffer.push_str(chunk_str);

                let len = available.len();
                self.reader.consume(len);
            }
        }
    }

    /// Skip to end of line when handling oversized line errors.
    /// Consumes data until a newline is found or EOF is reached.
    fn skip_to_end_of_line(&mut self) -> StreamResult<()> {
        loop {
            let available = match self.reader.fill_buf() {
                Ok(buf) => buf,
                Err(e) => return Err(StreamError::Io(e)),
            };

            if available.is_empty() {
                // EOF reached, line is done
                return Ok(());
            }

            if let Some(newline_pos) = memchr(b'\n', available) {
                // Found newline, consume up to and including it
                self.reader.consume(newline_pos + 1);
                return Ok(());
            } else {
                // No newline, consume all and continue
                let len = available.len();
                self.reader.consume(len);
            }
        }
    }
}

impl<R: Read> Iterator for LineReader<R> {
    type Item = StreamResult<(usize, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_line() {
            Ok(Some(line)) => Some(Ok(line)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_lines() {
        let input = "line1\nline2\nline3";
        let mut reader = LineReader::new(Cursor::new(input));

        assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((2, "line2".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((3, "line3".to_string())));
        assert_eq!(reader.next_line().unwrap(), None);
    }

    #[test]
    fn test_peek_and_push_back() {
        let input = "line1\nline2";
        let mut reader = LineReader::new(Cursor::new(input));

        let peeked = reader.peek_line().unwrap().cloned();
        assert_eq!(peeked, Some((1, "line1".to_string())));

        // Should still return the same line
        let line = reader.next_line().unwrap();
        assert_eq!(line, Some((1, "line1".to_string())));

        // Push back
        reader.push_back(1, "line1".to_string());
        let line = reader.next_line().unwrap();
        assert_eq!(line, Some((1, "line1".to_string())));
    }

    // ==================== Empty input tests ====================

    #[test]
    fn test_empty_input() {
        let input = "";
        let mut reader = LineReader::new(Cursor::new(input));
        assert_eq!(reader.next_line().unwrap(), None);
    }

    #[test]
    fn test_single_empty_line() {
        let input = "\n";
        let mut reader = LineReader::new(Cursor::new(input));
        assert_eq!(reader.next_line().unwrap(), Some((1, String::new())));
        assert_eq!(reader.next_line().unwrap(), None);
    }

    #[test]
    fn test_multiple_empty_lines() {
        let input = "\n\n\n";
        let mut reader = LineReader::new(Cursor::new(input));
        assert_eq!(reader.next_line().unwrap(), Some((1, String::new())));
        assert_eq!(reader.next_line().unwrap(), Some((2, String::new())));
        assert_eq!(reader.next_line().unwrap(), Some((3, String::new())));
        assert_eq!(reader.next_line().unwrap(), None);
    }

    // ==================== Line ending tests ====================

    #[test]
    fn test_crlf_line_endings() {
        let input = "line1\r\nline2\r\nline3";
        let mut reader = LineReader::new(Cursor::new(input));
        assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((2, "line2".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((3, "line3".to_string())));
    }

    #[test]
    fn test_mixed_line_endings() {
        let input = "line1\nline2\r\nline3\nline4";
        let mut reader = LineReader::new(Cursor::new(input));
        assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((2, "line2".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((3, "line3".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((4, "line4".to_string())));
    }

    #[test]
    fn test_trailing_newline() {
        let input = "line1\n";
        let mut reader = LineReader::new(Cursor::new(input));
        assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
        assert_eq!(reader.next_line().unwrap(), None);
    }

    #[test]
    fn test_no_trailing_newline() {
        let input = "line1";
        let mut reader = LineReader::new(Cursor::new(input));
        assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
        assert_eq!(reader.next_line().unwrap(), None);
    }

    // ==================== Line number tests ====================

    #[test]
    fn test_line_number_initial() {
        let reader: LineReader<Cursor<&str>> = LineReader::new(Cursor::new("test"));
        assert_eq!(reader.line_number(), 0);
    }

    #[test]
    fn test_line_number_after_read() {
        let input = "line1\nline2\nline3";
        let mut reader = LineReader::new(Cursor::new(input));

        reader.next_line().unwrap();
        assert_eq!(reader.line_number(), 1);

        reader.next_line().unwrap();
        assert_eq!(reader.line_number(), 2);

        reader.next_line().unwrap();
        assert_eq!(reader.line_number(), 3);
    }

    #[test]
    fn test_line_number_after_eof() {
        let input = "line1";
        let mut reader = LineReader::new(Cursor::new(input));

        reader.next_line().unwrap();
        assert_eq!(reader.line_number(), 1);

        reader.next_line().unwrap(); // EOF
        assert_eq!(reader.line_number(), 1); // Line number unchanged
    }

    // ==================== Peek tests ====================

    #[test]
    fn test_peek_empty_input() {
        let mut reader = LineReader::new(Cursor::new(""));
        assert_eq!(reader.peek_line().unwrap(), None);
    }

    #[test]
    fn test_peek_multiple_times() {
        let input = "line1\nline2";
        let mut reader = LineReader::new(Cursor::new(input));

        // Peek multiple times should return the same line
        assert_eq!(reader.peek_line().unwrap(), Some(&(1, "line1".to_string())));
        assert_eq!(reader.peek_line().unwrap(), Some(&(1, "line1".to_string())));
        assert_eq!(reader.peek_line().unwrap(), Some(&(1, "line1".to_string())));

        // Consume it
        reader.next_line().unwrap();

        // Next peek should be the second line
        assert_eq!(reader.peek_line().unwrap(), Some(&(2, "line2".to_string())));
    }

    #[test]
    fn test_peek_then_read() {
        let input = "line1\nline2";
        let mut reader = LineReader::new(Cursor::new(input));

        reader.peek_line().unwrap();
        let line = reader.next_line().unwrap();
        assert_eq!(line, Some((1, "line1".to_string())));
    }

    // ==================== Push back tests ====================

    #[test]
    fn test_push_back_with_different_line_number() {
        let input = "line1\nline2";
        let mut reader = LineReader::new(Cursor::new(input));

        reader.next_line().unwrap(); // line1
        reader.push_back(99, "pushed".to_string());

        let line = reader.next_line().unwrap();
        assert_eq!(line, Some((99, "pushed".to_string())));
    }

    #[test]
    fn test_push_back_overwrites_peek() {
        let input = "line1\nline2";
        let mut reader = LineReader::new(Cursor::new(input));

        reader.peek_line().unwrap(); // Peek line1
        reader.push_back(42, "pushed".to_string());

        let line = reader.next_line().unwrap();
        assert_eq!(line, Some((42, "pushed".to_string())));
    }

    // ==================== Iterator tests ====================

    #[test]
    fn test_iterator() {
        let input = "line1\nline2\nline3";
        let reader = LineReader::new(Cursor::new(input));

        let lines: Vec<_> = reader.filter_map(std::result::Result::ok).collect();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], (1, "line1".to_string()));
        assert_eq!(lines[1], (2, "line2".to_string()));
        assert_eq!(lines[2], (3, "line3".to_string()));
    }

    #[test]
    fn test_iterator_empty() {
        let reader = LineReader::new(Cursor::new(""));
        let lines: Vec<_> = reader.filter_map(std::result::Result::ok).collect();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_iterator_single_line() {
        let reader = LineReader::new(Cursor::new("single"));
        let lines: Vec<_> = reader.filter_map(std::result::Result::ok).collect();
        assert_eq!(lines, vec![(1, "single".to_string())]);
    }

    // ==================== With capacity tests ====================

    #[test]
    fn test_with_capacity() {
        let input = "line1\nline2";
        let mut reader = LineReader::with_capacity(Cursor::new(input), 1024);

        assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((2, "line2".to_string())));
    }

    #[test]
    fn test_with_small_capacity() {
        let input = "line1\nline2";
        let mut reader = LineReader::with_capacity(Cursor::new(input), 1);

        assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((2, "line2".to_string())));
    }

    // ==================== Unicode tests ====================

    #[test]
    fn test_unicode_content() {
        let input = "你好\n世界\n🎉";
        let mut reader = LineReader::new(Cursor::new(input));

        assert_eq!(reader.next_line().unwrap(), Some((1, "你好".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((2, "世界".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((3, "🎉".to_string())));
    }

    #[test]
    fn test_unicode_line_with_emoji() {
        let input = "Hello 🌍 World";
        let mut reader = LineReader::new(Cursor::new(input));
        assert_eq!(
            reader.next_line().unwrap(),
            Some((1, "Hello 🌍 World".to_string()))
        );
    }

    // ==================== Whitespace tests ====================

    #[test]
    fn test_line_with_spaces() {
        let input = "  indented  \n\ttabbed\t";
        let mut reader = LineReader::new(Cursor::new(input));

        assert_eq!(
            reader.next_line().unwrap(),
            Some((1, "  indented  ".to_string()))
        );
        assert_eq!(
            reader.next_line().unwrap(),
            Some((2, "\ttabbed\t".to_string()))
        );
    }

    #[test]
    fn test_only_whitespace_lines() {
        let input = "   \n\t\t\n  \t  ";
        let mut reader = LineReader::new(Cursor::new(input));

        assert_eq!(reader.next_line().unwrap(), Some((1, "   ".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((2, "\t\t".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((3, "  \t  ".to_string())));
    }

    // ==================== Long line tests ====================

    #[test]
    fn test_long_line() {
        let long_line = "a".repeat(10000);
        let mut reader = LineReader::new(Cursor::new(long_line.clone()));
        assert_eq!(reader.next_line().unwrap(), Some((1, long_line)));
    }

    #[test]
    fn test_many_lines() {
        let lines: Vec<String> = (0..1000).map(|i| format!("line{i}")).collect();
        let input = lines.join("\n");
        let mut reader = LineReader::new(Cursor::new(input));

        for (i, expected) in lines.iter().enumerate() {
            let result = reader.next_line().unwrap();
            assert_eq!(result, Some((i + 1, expected.clone())));
        }
        assert_eq!(reader.next_line().unwrap(), None);
    }

    // ==================== Security: Line length enforcement tests ====================

    #[test]
    fn test_line_length_limit_enforced() {
        let config_max = 100;

        // Create a line with 101 characters (exceeds limit of 100)
        let long_line = format!("data: {}\n", "A".repeat(95)); // "data: " (6 chars) + 95 A's + newline = 102 total
        let input = Cursor::new(long_line.as_str());
        let mut reader = LineReader::with_max_length(input, config_max);

        let result = reader.next_line();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, StreamError::LineTooLong { .. }));

        if let StreamError::LineTooLong {
            line,
            length,
            limit,
        } = err
        {
            assert_eq!(line, 1);
            assert!(length > 100);
            assert_eq!(limit, 100);
        }
    }

    #[test]
    fn test_line_length_limit_exactly_at_boundary() {
        let config_max = 100;
        // Exactly 100 characters (should succeed)
        let line = format!("data: {}\n", "A".repeat(93)); // "data: " + 93 A's + newline = 100 chars
        let mut reader = LineReader::with_max_length(Cursor::new(line), config_max);

        let result = reader.next_line();
        assert!(result.is_ok());
        let (line_num, content) = result.unwrap().unwrap();
        assert_eq!(line_num, 1);
        assert_eq!(content.len(), 99); // "data: " + 93 A's (without newline)
    }

    #[test]
    fn test_line_length_limit_one_over_boundary() {
        let config_max = 100;
        // Exactly 101 characters WITHOUT newline (should fail)
        // "data: " (6 chars) + 95 A's = 101 chars (no newline yet)
        let line = format!("data: {}", "A".repeat(95));
        let mut reader =
            LineReader::with_capacity_and_max_length(Cursor::new(line), 64, config_max);

        let result = reader.next_line();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StreamError::LineTooLong { .. }
        ));
    }

    #[test]
    fn test_default_limit_allows_reasonable_lines() {
        // 100KB line (well under 1MB default)
        let line = format!("data: {}\n", "A".repeat(100_000));
        let mut reader = LineReader::new(Cursor::new(line));

        let result = reader.next_line();
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_limit_rejects_huge_lines() {
        // 2MB line (exceeds 1MB default)
        let line = format!("data: {}\n", "A".repeat(2_000_000));
        let mut reader = LineReader::new(Cursor::new(line));

        let result = reader.next_line();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StreamError::LineTooLong { .. }
        ));
    }

    #[test]
    fn test_multiple_long_lines() {
        let config_max = 50;
        let input = format!(
            "version: 1.0\nid: 1\ndata: {}\nid: 2\ndata: {}\n",
            "A".repeat(60), // First line OK, second line exceeds
            "B".repeat(60)
        );
        let mut reader = LineReader::with_max_length(Cursor::new(input), config_max);

        // Should successfully read lines that are within the limit
        assert!(reader.next_line().is_ok()); // version
        assert!(reader.next_line().is_ok()); // id: 1

        // Should fail on first overly long line
        let result = reader.next_line();
        assert!(result.is_err());
        if let Err(StreamError::LineTooLong { line, .. }) = result {
            assert_eq!(line, 3); // Third line (the first long data line)
        }
    }

    #[test]
    fn test_line_without_newline_checked() {
        let config_max = 100;
        // No trailing newline, 101 chars
        let input = format!("data: {}", "A".repeat(95));
        let mut reader = LineReader::with_max_length(Cursor::new(input), config_max);

        let result = reader.next_line();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StreamError::LineTooLong { .. }
        ));
    }

    #[test]
    fn test_very_long_single_line_without_newline() {
        let config_max = 1000;
        // Create a line that's way over the limit without any newline
        let input = "A".repeat(10_000);
        let mut reader = LineReader::with_max_length(Cursor::new(input), config_max);

        let result = reader.next_line();
        assert!(result.is_err());

        if let Err(StreamError::LineTooLong {
            line,
            length,
            limit,
        }) = result
        {
            assert_eq!(line, 1);
            // The actual length detected will be at least the limit
            // It could be detected at the buffer boundary (8192) or the full length
            assert!(length >= config_max);
            assert_eq!(limit, 1000);
        }
    }

    #[test]
    fn test_line_length_limit_with_crlf() {
        let config_max = 50;
        // Test with CRLF line endings
        let line = format!("data: {}\r\n", "A".repeat(60)); // Exceeds limit
        let mut reader = LineReader::with_max_length(Cursor::new(line), config_max);

        let result = reader.next_line();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StreamError::LineTooLong { .. }
        ));
    }

    #[test]
    fn test_line_length_across_buffer_boundaries() {
        let config_max = 1000;
        // Create a line that's larger than typical buffer sizes
        let long_line = format!("data: {}\n", "A".repeat(2000));
        let mut reader =
            LineReader::with_capacity_and_max_length(Cursor::new(long_line), 64, config_max);

        let result = reader.next_line();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StreamError::LineTooLong { .. }
        ));
    }

    #[test]
    fn test_unlimited_config_allows_any_length() {
        // Use usize::MAX to simulate unlimited config
        let config_max = usize::MAX;
        let line = format!("data: {}\n", "A".repeat(1_000_000));
        let mut reader = LineReader::with_max_length(Cursor::new(line), config_max);

        let result = reader.next_line();
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_line_respects_limit() {
        let config_max = 10;
        let input = "\n"; // Empty line should be fine
        let mut reader = LineReader::with_max_length(Cursor::new(input), config_max);

        let result = reader.next_line();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some((1, String::new())));
    }

    #[test]
    fn test_zero_length_limit() {
        let config_max = 0;
        let input = "x\n";
        let mut reader = LineReader::with_max_length(Cursor::new(input), config_max);

        let result = reader.next_line();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StreamError::LineTooLong { .. }
        ));
    }

    #[test]
    fn test_line_too_long_error_message() {
        let config_max = 100;
        let line = format!("data: {}\n", "A".repeat(200));
        let mut reader = LineReader::with_max_length(Cursor::new(line), config_max);

        let result = reader.next_line();
        let err = result.unwrap_err();
        let msg = format!("{err}");

        assert!(msg.contains("exceeds maximum length"));
        assert!(msg.contains("100"));
    }

    #[test]
    fn test_mixed_valid_and_invalid_lines() {
        let config_max = 50;
        let input = format!(
            "short\n{}\nanother short\n{}\n",
            "A".repeat(100), // Too long
            "B".repeat(200)  // Also too long
        );
        let mut reader = LineReader::with_max_length(Cursor::new(input.as_str()), config_max);

        // First line OK
        assert!(reader.next_line().is_ok());

        // Second line should fail
        let result = reader.next_line();
        assert!(result.is_err());
        if let Err(StreamError::LineTooLong { line, .. }) = result {
            assert_eq!(line, 2);
        }
    }

    #[test]
    fn test_line_length_checked_before_memory_allocation() {
        // This test verifies that we check the limit BEFORE allocating excessive memory.
        // The key insight is that with chunk-based reading using fill_buf, we check
        // the length as we read chunks, not after reading everything into memory.
        let config_max = 100;

        // Create a line that's way over the limit
        let huge_line = "A".repeat(10_000_000); // 10MB
        let input = format!("{huge_line}\n");

        let mut reader =
            LineReader::with_capacity_and_max_length(Cursor::new(input), 64, config_max);

        // This should fail quickly without allocating 10MB
        let result = reader.next_line();
        assert!(result.is_err());

        if let Err(StreamError::LineTooLong { length, limit, .. }) = result {
            assert!(length > limit);
            assert_eq!(limit, 100);
        }
    }

    // ==================== Security: UTF-8 validation tests ====================

    #[test]
    fn test_invalid_utf8_rejected() {
        let mut reader = LineReader::new(Cursor::new(vec![0xFF, 0xFE, 0xFD, 0x0A]));

        let result = reader.next_line();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StreamError::InvalidUtf8 { .. }
        ));
    }

    #[test]
    fn test_invalid_utf8_in_middle_of_line() {
        let mut input = vec![];
        input.extend_from_slice(b"valid start");
        input.extend_from_slice(&[0xFF, 0xFE]); // Invalid UTF-8
        input.extend_from_slice(b" end\n");

        let mut reader = LineReader::new(Cursor::new(input));

        let result = reader.next_line();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StreamError::InvalidUtf8 { .. }
        ));
    }

    #[test]
    fn test_invalid_utf8_error_includes_line_number() {
        let mut reader = LineReader::new(Cursor::new(vec![0xFF, 0xFE, 0x0A]));

        let result = reader.next_line();
        if let Err(StreamError::InvalidUtf8 { line, .. }) = result {
            assert_eq!(line, 1);
        } else {
            panic!("Expected InvalidUtf8 error");
        }
    }

    #[test]
    fn test_valid_utf8_multibyte_characters() {
        // Test with valid multibyte UTF-8
        let input = "Hello 世界 🎉\n";
        let mut reader = LineReader::new(Cursor::new(input));

        let result = reader.next_line();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some((1, "Hello 世界 🎉".to_string())));
    }

    // ==================== With max_length constructors tests ====================

    #[test]
    fn test_with_max_length_constructor() {
        let input = "test\n";
        let reader = LineReader::with_max_length(Cursor::new(input), 500);
        assert_eq!(reader.line_number(), 0);
    }

    #[test]
    fn test_with_capacity_and_max_length_constructor() {
        let input = "test\n";
        let reader = LineReader::with_capacity_and_max_length(Cursor::new(input), 1024, 500);
        assert_eq!(reader.line_number(), 0);
    }
}
