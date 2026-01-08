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

//! Source position and span tracking for HEDL lexical analysis.
//!
//! This module provides span information for AST nodes and error reporting,
//! enabling precise error messages with line and column numbers.
//!
//! # Examples
//!
//! ```
//! use hedl_core::lex::{SourcePos, Span};
//!
//! // Create a position at line 10, column 25
//! let pos = SourcePos::new(10, 25);
//! assert_eq!(pos.line(), 10);
//! assert_eq!(pos.column(), 25);
//!
//! // Create a span from start to end
//! let start = SourcePos::new(1, 5);
//! let end = SourcePos::new(1, 10);
//! let span = Span::new(start, end);
//! assert!(span.is_single_line());
//! ```

use std::fmt;

/// A position in source code (line and column).
///
/// Line and column numbers are 1-indexed by convention.
/// For zero-indexed positions, use `SourcePos::default()`.
///
/// # Examples
///
/// ```
/// use hedl_core::lex::SourcePos;
///
/// let pos = SourcePos::new(10, 25);
/// assert_eq!(pos.line(), 10);
/// assert_eq!(pos.column(), 25);
///
/// // Navigate positions
/// let mut pos = SourcePos::new(5, 10);
/// pos.advance_col();
/// assert_eq!(pos.column(), 11);
/// pos.next_line();
/// assert_eq!(pos.line(), 6);
/// assert_eq!(pos.column(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SourcePos {
    /// Line number (typically 1-indexed, but 0 is allowed for unknown positions).
    line: usize,
    /// Column number (typically 1-indexed, but 0 is allowed for unknown positions).
    column: usize,
}

impl SourcePos {
    /// Creates a new source position.
    ///
    /// # Arguments
    ///
    /// * `line` - The line number (typically 1-indexed).
    /// * `column` - The column number (typically 1-indexed).
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::SourcePos;
    ///
    /// let pos = SourcePos::new(10, 25);
    /// assert_eq!(pos.line(), 10);
    /// assert_eq!(pos.column(), 25);
    /// ```
    #[inline]
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Creates a position at the start of the file (line 1, column 1).
    #[inline]
    pub const fn start() -> Self {
        Self { line: 1, column: 1 }
    }

    /// Returns the line number.
    #[inline]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the column number.
    #[inline]
    pub const fn column(&self) -> usize {
        self.column
    }

    /// Advances the position by one column.
    #[inline]
    pub fn advance_col(&mut self) {
        self.column += 1;
    }

    /// Advances the position by n columns.
    #[inline]
    pub fn advance_cols(&mut self, n: usize) {
        self.column += n;
    }

    /// Moves to the next line (increments line, resets column to 1).
    #[inline]
    pub fn next_line(&mut self) {
        self.line += 1;
        self.column = 1;
    }

    /// Creates a new SourcePos from u32 values (for compatibility).
    #[inline]
    pub const fn from_u32(line: u32, column: u32) -> Self {
        Self {
            line: line as usize,
            column: column as usize,
        }
    }
}

impl fmt::Display for SourcePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// A span in source code (start and end positions).
///
/// Spans are half-open intervals [start, end), where start is inclusive
/// and end is exclusive.
///
/// # Examples
///
/// ```
/// use hedl_core::lex::{SourcePos, Span};
///
/// let start = SourcePos::new(1, 5);
/// let end = SourcePos::new(1, 10);
/// let span = Span::new(start, end);
///
/// assert_eq!(span.start().line(), 1);
/// assert_eq!(span.end().column(), 10);
/// assert!(span.is_single_line());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    /// Start position (inclusive).
    start: SourcePos,
    /// End position (exclusive).
    end: SourcePos,
}

impl Span {
    /// Creates a new span from start and end positions.
    #[inline]
    pub const fn new(start: SourcePos, end: SourcePos) -> Self {
        Self { start, end }
    }

    /// Creates a zero-width span at a single position.
    #[inline]
    pub const fn point(pos: SourcePos) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    /// Creates a span at the start of the file.
    #[inline]
    pub const fn file_start() -> Self {
        Self::point(SourcePos::start())
    }

    /// Gets the start position (inclusive).
    #[inline]
    pub const fn start(&self) -> SourcePos {
        self.start
    }

    /// Gets the end position (exclusive).
    #[inline]
    pub const fn end(&self) -> SourcePos {
        self.end
    }

    /// Checks if this span is on a single line.
    #[inline]
    pub const fn is_single_line(&self) -> bool {
        self.start.line == self.end.line
    }

    /// Combines two spans into a larger span covering both.
    ///
    /// The resulting span will start at the earlier position and end
    /// at the later position.
    pub fn merge(self, other: Span) -> Span {
        let start = if self.start.line < other.start.line
            || (self.start.line == other.start.line && self.start.column < other.start.column)
        {
            self.start
        } else {
            other.start
        };

        let end = if self.end.line > other.end.line
            || (self.end.line == other.end.line && self.end.column > other.end.column)
        {
            self.end
        } else {
            other.end
        };

        Span { start, end }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_single_line() {
            write!(
                f,
                "{}:{}-{}",
                self.start.line, self.start.column, self.end.column
            )
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== SourcePos tests ====================

    #[test]
    fn test_source_pos_new() {
        let pos = SourcePos::new(10, 25);
        assert_eq!(pos.line(), 10);
        assert_eq!(pos.column(), 25);
    }

    #[test]
    fn test_source_pos_start() {
        let pos = SourcePos::start();
        assert_eq!(pos.line(), 1);
        assert_eq!(pos.column(), 1);
    }

    #[test]
    fn test_source_pos_default() {
        let pos = SourcePos::default();
        assert_eq!(pos.line(), 0);
        assert_eq!(pos.column(), 0);
    }

    #[test]
    fn test_source_pos_advance_col() {
        let mut pos = SourcePos::new(5, 10);
        pos.advance_col();
        assert_eq!(pos.line(), 5);
        assert_eq!(pos.column(), 11);
    }

    #[test]
    fn test_source_pos_advance_cols() {
        let mut pos = SourcePos::new(5, 10);
        pos.advance_cols(5);
        assert_eq!(pos.line(), 5);
        assert_eq!(pos.column(), 15);
    }

    #[test]
    fn test_source_pos_next_line() {
        let mut pos = SourcePos::new(5, 42);
        pos.next_line();
        assert_eq!(pos.line(), 6);
        assert_eq!(pos.column(), 1);
    }

    #[test]
    fn test_source_pos_display() {
        let pos = SourcePos::new(10, 25);
        assert_eq!(format!("{}", pos), "line 10, column 25");
    }

    #[test]
    fn test_source_pos_equality() {
        let a = SourcePos::new(5, 10);
        let b = SourcePos::new(5, 10);
        let c = SourcePos::new(5, 11);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_source_pos_from_u32() {
        let pos = SourcePos::from_u32(42, 15);
        assert_eq!(pos.line(), 42);
        assert_eq!(pos.column(), 15);
    }

    // ==================== Span tests ====================

    #[test]
    fn test_span_new() {
        let start = SourcePos::new(1, 5);
        let end = SourcePos::new(1, 10);
        let span = Span::new(start, end);
        assert_eq!(span.start(), start);
        assert_eq!(span.end(), end);
    }

    #[test]
    fn test_span_point() {
        let pos = SourcePos::new(3, 7);
        let span = Span::point(pos);
        assert_eq!(span.start(), pos);
        assert_eq!(span.end(), pos);
    }

    #[test]
    fn test_span_file_start() {
        let span = Span::file_start();
        assert_eq!(span.start().line(), 1);
        assert_eq!(span.start().column(), 1);
    }

    #[test]
    fn test_span_default() {
        let span = Span::default();
        assert_eq!(span.start().line(), 0);
        assert_eq!(span.start().column(), 0);
    }

    #[test]
    fn test_span_is_single_line_true() {
        let span = Span::new(SourcePos::new(5, 10), SourcePos::new(5, 20));
        assert!(span.is_single_line());
    }

    #[test]
    fn test_span_is_single_line_false() {
        let span = Span::new(SourcePos::new(5, 10), SourcePos::new(6, 5));
        assert!(!span.is_single_line());
    }

    #[test]
    fn test_span_merge_same_line() {
        let span1 = Span::new(SourcePos::new(1, 5), SourcePos::new(1, 10));
        let span2 = Span::new(SourcePos::new(1, 15), SourcePos::new(1, 20));
        let merged = span1.merge(span2);
        assert_eq!(merged.start(), SourcePos::new(1, 5));
        assert_eq!(merged.end(), SourcePos::new(1, 20));
    }

    #[test]
    fn test_span_merge_different_lines() {
        let span1 = Span::new(SourcePos::new(1, 5), SourcePos::new(2, 10));
        let span2 = Span::new(SourcePos::new(3, 1), SourcePos::new(4, 5));
        let merged = span1.merge(span2);
        assert_eq!(merged.start(), SourcePos::new(1, 5));
        assert_eq!(merged.end(), SourcePos::new(4, 5));
    }

    #[test]
    fn test_span_display_single_line() {
        let span = Span::new(SourcePos::new(5, 10), SourcePos::new(5, 20));
        assert_eq!(format!("{}", span), "5:10-20");
    }

    #[test]
    fn test_span_display_multi_line() {
        let span = Span::new(SourcePos::new(5, 10), SourcePos::new(7, 5));
        assert_eq!(format!("{}", span), "line 5, column 10-line 7, column 5");
    }

    #[test]
    fn test_span_equality() {
        let a = Span::new(SourcePos::new(1, 1), SourcePos::new(1, 5));
        let b = Span::new(SourcePos::new(1, 1), SourcePos::new(1, 5));
        let c = Span::new(SourcePos::new(1, 1), SourcePos::new(1, 6));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_span_merge_identical() {
        let span = Span::new(SourcePos::new(1, 1), SourcePos::new(1, 5));
        let merged = span.merge(span);
        assert_eq!(merged, span);
    }
}
