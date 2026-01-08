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
}

impl<R: Read> LineReader<R> {
    /// Create a new line reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_number: 0,
            buffer: String::new(),
            peeked: None,
        }
    }

    /// Create with a specific buffer capacity.
    pub fn with_capacity(reader: R, capacity: usize) -> Self {
        Self {
            reader: BufReader::with_capacity(capacity, reader),
            line_number: 0,
            buffer: String::new(),
            peeked: None,
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

        self.buffer.clear();

        match self.reader.read_line(&mut self.buffer) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                self.line_number += 1;

                // Remove trailing newline
                if self.buffer.ends_with('\n') {
                    self.buffer.pop();
                    if self.buffer.ends_with('\r') {
                        self.buffer.pop();
                    }
                }

                Ok(Some((self.line_number, self.buffer.clone())))
            }
            Err(e) => Err(StreamError::Io(e)),
        }
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

        match self.reader.read_line(&mut self.buffer) {
            Ok(0) => Ok(None),
            Ok(_) => {
                self.line_number += 1;

                if self.buffer.ends_with('\n') {
                    self.buffer.pop();
                    if self.buffer.ends_with('\r') {
                        self.buffer.pop();
                    }
                }

                Ok(Some((self.line_number, self.buffer.clone())))
            }
            Err(e) => Err(StreamError::Io(e)),
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
        assert_eq!(reader.next_line().unwrap(), Some((1, "".to_string())));
        assert_eq!(reader.next_line().unwrap(), None);
    }

    #[test]
    fn test_multiple_empty_lines() {
        let input = "\n\n\n";
        let mut reader = LineReader::new(Cursor::new(input));
        assert_eq!(reader.next_line().unwrap(), Some((1, "".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((2, "".to_string())));
        assert_eq!(reader.next_line().unwrap(), Some((3, "".to_string())));
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

        let lines: Vec<_> = reader.filter_map(|r| r.ok()).collect();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], (1, "line1".to_string()));
        assert_eq!(lines[1], (2, "line2".to_string()));
        assert_eq!(lines[2], (3, "line3".to_string()));
    }

    #[test]
    fn test_iterator_empty() {
        let reader = LineReader::new(Cursor::new(""));
        let lines: Vec<_> = reader.filter_map(|r| r.ok()).collect();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_iterator_single_line() {
        let reader = LineReader::new(Cursor::new("single"));
        let lines: Vec<_> = reader.filter_map(|r| r.ok()).collect();
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
        let lines: Vec<String> = (0..1000).map(|i| format!("line{}", i)).collect();
        let input = lines.join("\n");
        let mut reader = LineReader::new(Cursor::new(input));

        for (i, expected) in lines.iter().enumerate() {
            let result = reader.next_line().unwrap();
            assert_eq!(result, Some((i + 1, expected.clone())));
        }
        assert_eq!(reader.next_line().unwrap(), None);
    }
}
