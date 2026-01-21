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

//! Async line reader for streaming parser.
//!
//! Provides buffered async line-by-line reading with line number tracking, peek support,
//! and the ability to push back lines for re-parsing.
//!
//! This module mirrors the synchronous [`LineReader`](crate::LineReader) but uses
//! tokio's async I/O primitives for non-blocking operation.
//!
//! # Examples
//!
//! ## Basic Async Line Reading
//!
//! ```rust,no_run
//! # #[cfg(feature = "async")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use hedl_stream::AsyncLineReader;
//! use tokio::io::AsyncReadExt;
//! use std::io::Cursor;
//!
//! let input = "line1\nline2\nline3";
//! let mut reader = AsyncLineReader::new(Cursor::new(input));
//!
//! assert_eq!(reader.next_line().await?, Some((1, "line1".to_string())));
//! assert_eq!(reader.next_line().await?, Some((2, "line2".to_string())));
//! assert_eq!(reader.next_line().await?, Some((3, "line3".to_string())));
//! assert_eq!(reader.next_line().await?, None);
//! # Ok(())
//! # }
//! ```
//!
//! ## Peeking and Push Back
//!
//! ```rust,no_run
//! # #[cfg(feature = "async")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use hedl_stream::AsyncLineReader;
//! use std::io::Cursor;
//!
//! let input = "line1\nline2";
//! let mut reader = AsyncLineReader::new(Cursor::new(input));
//!
//! // Peek without consuming
//! assert_eq!(reader.peek_line().await?, Some(&(1, "line1".to_string())));
//! assert_eq!(reader.peek_line().await?, Some(&(1, "line1".to_string())));
//!
//! // Now consume it
//! let line = reader.next_line().await?.unwrap();
//! assert_eq!(line, (1, "line1".to_string()));
//!
//! // Push it back
//! reader.push_back(line.0, line.1);
//!
//! // Read it again
//! assert_eq!(reader.next_line().await?, Some((1, "line1".to_string())));
//! # Ok(())
//! # }
//! ```

use crate::error::{StreamError, StreamResult};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

/// Simple memchr implementation - finds the first occurrence of a byte in a slice.
fn memchr_byte(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

/// Buffered async line reader with line number tracking.
///
/// Reads input line-by-line asynchronously, automatically handling different line endings
/// (LF, CRLF) and tracking the current line number for error reporting.
///
/// # Performance Characteristics
///
/// - **Buffering**: Configurable buffer size (default 64KB) for efficient I/O
/// - **Zero-Copy**: String allocations only for consumed lines
/// - **Async**: Non-blocking I/O suitable for high-concurrency scenarios
///
/// # When to Use Async vs Sync
///
/// **Use Async When:**
/// - Processing network streams or pipes
/// - High-concurrency scenarios (many parallel streams)
/// - Integration with async web servers or frameworks
/// - Need to process I/O without blocking threads
///
/// **Use Sync When:**
/// - Processing local files
/// - Single-threaded batch processing
/// - Simpler code without async complexity
/// - CPU-bound workloads with minimal I/O wait
///
/// # Examples
///
/// ## Reading from Async Source
///
/// ```rust,no_run
/// # #[cfg(feature = "async")]
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use hedl_stream::AsyncLineReader;
/// use tokio::fs::File;
///
/// let file = File::open("data.hedl").await?;
/// let mut reader = AsyncLineReader::new(file);
///
/// while let Some((line_num, line)) = reader.next_line().await? {
///     println!("{}: {}", line_num, line);
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## With Custom Buffer Size
///
/// ```rust
/// # #[cfg(feature = "async")]
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use hedl_stream::AsyncLineReader;
/// use std::io::Cursor;
///
/// let input = "line1\nline2";
/// let reader = AsyncLineReader::with_capacity(Cursor::new(input), 256 * 1024);
/// # Ok(())
/// # }
/// ```
pub struct AsyncLineReader<R: AsyncRead + Unpin> {
    reader: BufReader<R>,
    line_number: usize,
    buffer: String,
    peeked: Option<(usize, String)>,
    max_line_length: usize,
}

impl<R: AsyncRead + Unpin> AsyncLineReader<R> {
    /// Create a new async line reader with default buffer size (64KB) and max line length (1MB).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// use hedl_stream::AsyncLineReader;
    /// use std::io::Cursor;
    ///
    /// let input = "line1\nline2";
    /// let reader = AsyncLineReader::new(Cursor::new(input));
    /// ```
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_number: 0,
            buffer: String::new(),
            peeked: None,
            max_line_length: 1_000_000,
        }
    }

    /// Create an async line reader with a specific buffer capacity and default max line length (1MB).
    ///
    /// # Parameters
    ///
    /// - `reader`: The async readable source
    /// - `capacity`: Buffer size in bytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// use hedl_stream::AsyncLineReader;
    /// use std::io::Cursor;
    ///
    /// // Use a larger buffer for large files
    /// let reader = AsyncLineReader::with_capacity(
    ///     Cursor::new("data"),
    ///     256 * 1024  // 256KB
    /// );
    /// ```
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
    ///
    /// Returns 0 before any lines are read, then increments with each line.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::AsyncLineReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = AsyncLineReader::new(Cursor::new("line1\nline2"));
    ///
    /// assert_eq!(reader.line_number(), 0);
    /// reader.next_line().await?;
    /// assert_eq!(reader.line_number(), 1);
    /// reader.next_line().await?;
    /// assert_eq!(reader.line_number(), 2);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    /// Read the next line asynchronously.
    ///
    /// Returns `Ok(Some((line_num, line)))` if a line was read, `Ok(None)` at EOF,
    /// or `Err` on I/O errors.
    ///
    /// Trailing newlines (LF or CRLF) are automatically stripped.
    ///
    /// # Performance
    ///
    /// This method awaits on I/O and yields to the runtime if data is not available,
    /// allowing other tasks to run. It does not block the thread.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::AsyncLineReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = AsyncLineReader::new(Cursor::new("hello\nworld"));
    ///
    /// let (num, line) = reader.next_line().await?.unwrap();
    /// assert_eq!(num, 1);
    /// assert_eq!(line, "hello");
    ///
    /// let (num, line) = reader.next_line().await?.unwrap();
    /// assert_eq!(num, 2);
    /// assert_eq!(line, "world");
    ///
    /// assert_eq!(reader.next_line().await?, None);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn next_line(&mut self) -> StreamResult<Option<(usize, String)>> {
        // Return peeked line if available
        if let Some(peeked) = self.peeked.take() {
            return Ok(Some(peeked));
        }

        self.read_line_internal().await
    }

    /// Peek at the next line without consuming it.
    ///
    /// Returns a reference to the next line without advancing the reader.
    /// Subsequent calls to `peek_line()` return the same line. Call `next_line()`
    /// to consume it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::AsyncLineReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = AsyncLineReader::new(Cursor::new("line1\nline2"));
    ///
    /// // Peek multiple times
    /// assert_eq!(reader.peek_line().await?, Some(&(1, "line1".to_string())));
    /// assert_eq!(reader.peek_line().await?, Some(&(1, "line1".to_string())));
    ///
    /// // Consume
    /// reader.next_line().await?;
    ///
    /// // Next peek is the second line
    /// assert_eq!(reader.peek_line().await?, Some(&(2, "line2".to_string())));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn peek_line(&mut self) -> StreamResult<Option<&(usize, String)>> {
        if self.peeked.is_none() {
            self.peeked = self.read_line_internal().await?;
        }
        Ok(self.peeked.as_ref())
    }

    /// Push a line back to be read again.
    ///
    /// The next call to `next_line()` or `peek_line()` will return this line.
    /// Only one line can be pushed back at a time; subsequent calls overwrite
    /// the previously pushed line.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::AsyncLineReader;
    /// use std::io::Cursor;
    ///
    /// let mut reader = AsyncLineReader::new(Cursor::new("line1\nline2"));
    ///
    /// let line = reader.next_line().await?.unwrap();
    /// assert_eq!(line, (1, "line1".to_string()));
    ///
    /// // Push it back
    /// reader.push_back(line.0, line.1);
    ///
    /// // Read it again
    /// let line = reader.next_line().await?.unwrap();
    /// assert_eq!(line, (1, "line1".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn push_back(&mut self, line_num: usize, line: String) {
        self.peeked = Some((line_num, line));
    }

    async fn read_line_internal(&mut self) -> StreamResult<Option<(usize, String)>> {
        self.buffer.clear();

        loop {
            // Read from BufReader's internal buffer (zero-copy)
            let available = self.reader.fill_buf().await.map_err(StreamError::Io)?;

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
            if let Some(newline_pos) = memchr_byte(b'\n', available) {
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
                    self.skip_to_end_of_line().await?;

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
    async fn skip_to_end_of_line(&mut self) -> StreamResult<()> {
        loop {
            let available = self.reader.fill_buf().await.map_err(StreamError::Io)?;

            if available.is_empty() {
                // EOF reached, line is done
                return Ok(());
            }

            if let Some(newline_pos) = memchr_byte(b'\n', available) {
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

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_read_lines() {
        let input = "line1\nline2\nline3";
        let mut reader = AsyncLineReader::new(Cursor::new(input));

        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((1, "line1".to_string()))
        );
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((2, "line2".to_string()))
        );
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((3, "line3".to_string()))
        );
        assert_eq!(reader.next_line().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_peek_and_push_back() {
        let input = "line1\nline2";
        let mut reader = AsyncLineReader::new(Cursor::new(input));

        let peeked = reader.peek_line().await.unwrap().cloned();
        assert_eq!(peeked, Some((1, "line1".to_string())));

        // Should still return the same line
        let line = reader.next_line().await.unwrap();
        assert_eq!(line, Some((1, "line1".to_string())));

        // Push back
        reader.push_back(1, "line1".to_string());
        let line = reader.next_line().await.unwrap();
        assert_eq!(line, Some((1, "line1".to_string())));
    }

    #[tokio::test]
    async fn test_empty_input() {
        let input = "";
        let mut reader = AsyncLineReader::new(Cursor::new(input));
        assert_eq!(reader.next_line().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_single_empty_line() {
        let input = "\n";
        let mut reader = AsyncLineReader::new(Cursor::new(input));
        assert_eq!(reader.next_line().await.unwrap(), Some((1, String::new())));
        assert_eq!(reader.next_line().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_crlf_line_endings() {
        let input = "line1\r\nline2\r\nline3";
        let mut reader = AsyncLineReader::new(Cursor::new(input));
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((1, "line1".to_string()))
        );
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((2, "line2".to_string()))
        );
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((3, "line3".to_string()))
        );
    }

    #[tokio::test]
    async fn test_mixed_line_endings() {
        let input = "line1\nline2\r\nline3\nline4";
        let mut reader = AsyncLineReader::new(Cursor::new(input));
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((1, "line1".to_string()))
        );
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((2, "line2".to_string()))
        );
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((3, "line3".to_string()))
        );
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((4, "line4".to_string()))
        );
    }

    #[tokio::test]
    async fn test_line_number_tracking() {
        let input = "line1\nline2\nline3";
        let mut reader = AsyncLineReader::new(Cursor::new(input));

        assert_eq!(reader.line_number(), 0);

        reader.next_line().await.unwrap();
        assert_eq!(reader.line_number(), 1);

        reader.next_line().await.unwrap();
        assert_eq!(reader.line_number(), 2);

        reader.next_line().await.unwrap();
        assert_eq!(reader.line_number(), 3);
    }

    #[tokio::test]
    async fn test_peek_multiple_times() {
        let input = "line1\nline2";
        let mut reader = AsyncLineReader::new(Cursor::new(input));

        // Peek multiple times should return the same line
        assert_eq!(
            reader.peek_line().await.unwrap(),
            Some(&(1, "line1".to_string()))
        );
        assert_eq!(
            reader.peek_line().await.unwrap(),
            Some(&(1, "line1".to_string()))
        );
        assert_eq!(
            reader.peek_line().await.unwrap(),
            Some(&(1, "line1".to_string()))
        );

        // Consume it
        reader.next_line().await.unwrap();

        // Next peek should be the second line
        assert_eq!(
            reader.peek_line().await.unwrap(),
            Some(&(2, "line2".to_string()))
        );
    }

    #[tokio::test]
    async fn test_with_capacity() {
        let input = "line1\nline2";
        let mut reader = AsyncLineReader::with_capacity(Cursor::new(input), 1024);

        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((1, "line1".to_string()))
        );
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((2, "line2".to_string()))
        );
    }

    #[tokio::test]
    async fn test_unicode_content() {
        let input = "你好\n世界\n🎉";
        let mut reader = AsyncLineReader::new(Cursor::new(input));

        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((1, "你好".to_string()))
        );
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((2, "世界".to_string()))
        );
        assert_eq!(
            reader.next_line().await.unwrap(),
            Some((3, "🎉".to_string()))
        );
    }

    #[tokio::test]
    async fn test_long_line() {
        let long_line = "a".repeat(10000);
        let mut reader = AsyncLineReader::new(Cursor::new(long_line.clone()));
        assert_eq!(reader.next_line().await.unwrap(), Some((1, long_line)));
    }

    #[tokio::test]
    async fn test_many_lines() {
        let lines: Vec<String> = (0..1000).map(|i| format!("line{i}")).collect();
        let input = lines.join("\n");
        let mut reader = AsyncLineReader::new(Cursor::new(input));

        for (i, expected) in lines.iter().enumerate() {
            let result = reader.next_line().await.unwrap();
            assert_eq!(result, Some((i + 1, expected.clone())));
        }
        assert_eq!(reader.next_line().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_push_back_overwrites_peek() {
        let input = "line1\nline2";
        let mut reader = AsyncLineReader::new(Cursor::new(input));

        reader.peek_line().await.unwrap(); // Peek line1
        reader.push_back(42, "pushed".to_string());

        let line = reader.next_line().await.unwrap();
        assert_eq!(line, Some((42, "pushed".to_string())));
    }
}
