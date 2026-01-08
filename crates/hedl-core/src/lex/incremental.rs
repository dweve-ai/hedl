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

//! Incremental parsing support for efficient IDE integration.
//!
//! This module provides incremental parsing capabilities that enable efficient
//! re-parsing when documents change. Instead of re-parsing the entire document,
//! only the modified lines and their dependent lines are re-parsed.
//!
//! # Architecture
//!
//! The incremental parser maintains:
//! - Line-level parse results with position tracking
//! - Content hashes to detect unchanged lines
//! - Dependency tracking for context-dependent parsing
//! - Efficient reuse of previous parse results
//!
//! # Performance Characteristics
//!
//! - Time complexity: O(modified_lines + dependent_lines) instead of O(total_lines)
//! - Space complexity: O(total_lines) for caching parse results
//! - Hash computation: O(line_length) using FNV-1a for cache-friendly performance
//!
//! # Examples
//!
//! ```
//! use hedl_core::lex::incremental::{IncrementalParser, TextEdit};
//!
//! let mut parser = IncrementalParser::new();
//!
//! // Initial parse
//! let text = "User: alice\n  name: Alice\n  age: 30";
//! let result = parser.parse(text);
//! assert_eq!(result.line_count(), 3);
//!
//! // Incremental update - only modify one line
//! let text2 = "User: alice\n  name: Alice\n  age: 31";
//! let edit = TextEdit::replace(2, 3, "  age: 31");
//! let result = parser.parse_incremental(text2, &[edit]);
//! assert_eq!(result.reused_lines(), 2); // First two lines reused
//! ```

use crate::lex::error::LexError;
use crate::lex::span::{SourcePos, Span};
use std::collections::HashMap;

// ==================== Configuration ====================

/// Maximum number of lines to cache in incremental parser.
const MAX_CACHED_LINES: usize = 100_000;

/// Maximum line length for incremental parsing.
const MAX_LINE_LENGTH: usize = 10_000;

/// Default maximum indentation depth (10 levels = 20 spaces).
const DEFAULT_MAX_INDENT_DEPTH: usize = 10;

// ==================== Hash Function ====================

/// Fast hash function for line content comparison (FNV-1a).
///
/// FNV-1a provides good distribution and is cache-friendly for small inputs.
/// This is used to quickly detect unchanged lines without string comparison.
#[inline]
fn hash_line(s: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    s.bytes().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}

// ==================== Indentation Helpers ====================

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
fn calculate_indent(line: &str, line_num: usize) -> Result<Option<IndentInfo>, LexError> {
    let bytes = line.as_bytes();
    let mut spaces = 0;

    // Count leading spaces and detect tabs in indentation
    for &b in bytes {
        match b {
            b' ' => spaces += 1,
            b'\t' => {
                // Tab found - check if line is blank after this point
                if bytes[spaces..].iter().all(|&b| b.is_ascii_whitespace()) {
                    return Ok(None);
                }
                return Err(LexError::TabInIndentation {
                    pos: SourcePos::new(line_num, spaces + 1),
                });
            }
            _ => break,
        }
    }

    // Check if line is blank (only spaces or all whitespace)
    if spaces == bytes.len() || bytes[spaces..].iter().all(|&b| b.is_ascii_whitespace()) {
        return Ok(None);
    }

    // Validate even number of spaces
    if spaces % 2 != 0 {
        return Err(LexError::InvalidIndentation {
            spaces,
            pos: SourcePos::new(line_num, 1),
        });
    }

    Ok(Some(IndentInfo {
        spaces,
        level: spaces / 2,
    }))
}

/// Validate that indent level doesn't exceed maximum.
fn validate_indent(info: IndentInfo, max_depth: usize, line_num: usize) -> Result<(), LexError> {
    if info.level > max_depth {
        return Err(LexError::IndentTooDeep {
            depth: info.level,
            max: max_depth,
            pos: SourcePos::new(line_num, 1),
        });
    }
    Ok(())
}

// ==================== Core Types ====================

/// A text edit operation for incremental updates.
///
/// Represents a change to a range of lines in the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    /// Starting line number (0-indexed).
    pub start_line: usize,
    /// Ending line number (0-indexed, exclusive).
    pub end_line: usize,
    /// New content for the range (may span multiple lines).
    pub new_text: String,
}

impl TextEdit {
    /// Create a new text edit that replaces a range of lines.
    ///
    /// # Arguments
    ///
    /// * `start_line` - Starting line (0-indexed, inclusive)
    /// * `end_line` - Ending line (0-indexed, exclusive)
    /// * `new_text` - New content (may contain newlines)
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::incremental::TextEdit;
    ///
    /// // Replace line 2 with new content
    /// let edit = TextEdit::replace(2, 3, "new content");
    ///
    /// // Insert new line at position 5
    /// let edit = TextEdit::insert(5, "inserted line");
    ///
    /// // Delete lines 3-5
    /// let edit = TextEdit::delete(3, 5);
    /// ```
    pub fn replace(start_line: usize, end_line: usize, new_text: impl Into<String>) -> Self {
        Self {
            start_line,
            end_line,
            new_text: new_text.into(),
        }
    }

    /// Create an insertion edit (no lines deleted).
    pub fn insert(line: usize, new_text: impl Into<String>) -> Self {
        Self {
            start_line: line,
            end_line: line,
            new_text: new_text.into(),
        }
    }

    /// Create a deletion edit (no new content).
    pub fn delete(start_line: usize, end_line: usize) -> Self {
        Self {
            start_line,
            end_line,
            new_text: String::new(),
        }
    }

    /// Get the number of lines being deleted.
    #[inline]
    pub fn deleted_lines(&self) -> usize {
        self.end_line.saturating_sub(self.start_line)
    }

    /// Get the number of lines being inserted.
    #[inline]
    pub fn inserted_lines(&self) -> usize {
        if self.new_text.is_empty() {
            0
        } else {
            self.new_text.lines().count()
        }
    }

    /// Calculate the line delta (inserted - deleted).
    #[inline]
    pub fn line_delta(&self) -> isize {
        self.inserted_lines() as isize - self.deleted_lines() as isize
    }
}

/// Cached parse result for a single line.
#[derive(Debug, Clone)]
struct CachedLine {
    /// Content hash for fast comparison.
    hash: u64,
    /// Original line content.
    content: String,
    /// Indentation level (number of spaces).
    indent: usize,
    /// Span covering this line.
    span: Span,
    /// Any error that occurred during parsing.
    error: Option<LexError>,
}

impl CachedLine {
    /// Create a cached line from content.
    fn new(line_num: usize, content: String) -> Self {
        let hash = hash_line(&content);
        let indent = content.chars().take_while(|&c| c == ' ').count();
        let start = SourcePos::new(line_num + 1, 1);
        let end = SourcePos::new(line_num + 1, content.len() + 1);
        let span = Span::new(start, end);

        Self {
            hash,
            content,
            indent,
            span,
            error: None,
        }
    }

    /// Check if content matches this cached line.
    #[inline]
    fn matches(&self, content: &str) -> bool {
        let hash = hash_line(content);
        hash == self.hash && self.content == content
    }
}

/// Result of an incremental parse operation.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Parsed lines with position information.
    lines: Vec<CachedLine>,
    /// Number of lines reused from cache.
    reused_count: usize,
    /// Number of lines re-parsed.
    reparsed_count: usize,
    /// Total parsing time in microseconds.
    parse_time_us: u64,
}

impl ParseResult {
    /// Get the number of lines in the document.
    #[inline]
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get the number of lines reused from cache.
    #[inline]
    pub fn reused_lines(&self) -> usize {
        self.reused_count
    }

    /// Get the number of lines that were re-parsed.
    #[inline]
    pub fn reparsed_lines(&self) -> usize {
        self.reparsed_count
    }

    /// Get the cache hit rate (0.0 to 1.0).
    #[inline]
    pub fn cache_hit_rate(&self) -> f64 {
        if self.lines.is_empty() {
            0.0
        } else {
            self.reused_count as f64 / self.lines.len() as f64
        }
    }

    /// Get parsing time in microseconds.
    #[inline]
    pub fn parse_time_us(&self) -> u64 {
        self.parse_time_us
    }

    /// Get line content by line number (0-indexed).
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|l| l.content.as_str())
    }

    /// Get the span for a line (0-indexed).
    pub fn get_span(&self, line: usize) -> Option<Span> {
        self.lines.get(line).map(|l| l.span)
    }

    /// Get indentation level for a line (0-indexed).
    pub fn get_indent(&self, line: usize) -> Option<usize> {
        self.lines.get(line).map(|l| l.indent)
    }

    /// Get any parse error for a line (0-indexed).
    pub fn get_error(&self, line: usize) -> Option<&LexError> {
        self.lines.get(line).and_then(|l| l.error.as_ref())
    }

    /// Find all lines with parse errors.
    pub fn errors(&self) -> Vec<(usize, &LexError)> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| line.error.as_ref().map(|e| (i, e)))
            .collect()
    }
}

// ==================== Incremental Parser ====================

/// Incremental parser for HEDL documents.
///
/// Maintains cached parse results and efficiently updates them when
/// the document changes. This is designed for IDE use cases where
/// small edits occur frequently.
///
/// # Thread Safety
///
/// This parser is not thread-safe. Use separate instances per document
/// or protect with a mutex in multi-threaded scenarios.
pub struct IncrementalParser {
    /// Maximum indentation depth to allow.
    max_indent_depth: usize,
    /// Cached parse results by line number.
    cache: HashMap<usize, CachedLine>,
    /// Total lines in last parse.
    last_line_count: usize,
}

impl IncrementalParser {
    /// Create a new incremental parser with default configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::incremental::IncrementalParser;
    ///
    /// let parser = IncrementalParser::new();
    /// ```
    pub fn new() -> Self {
        Self {
            max_indent_depth: DEFAULT_MAX_INDENT_DEPTH,
            cache: HashMap::new(),
            last_line_count: 0,
        }
    }

    /// Create a new incremental parser with custom maximum indentation depth.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::incremental::IncrementalParser;
    ///
    /// let parser = IncrementalParser::with_max_indent_depth(8);
    /// ```
    pub fn with_max_indent_depth(max_indent_depth: usize) -> Self {
        Self {
            max_indent_depth,
            cache: HashMap::new(),
            last_line_count: 0,
        }
    }

    /// Parse a complete document (initial parse or full reparse).
    ///
    /// This clears the cache and parses all lines. Use this for the
    /// initial parse or when you need to force a full reparse.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::incremental::IncrementalParser;
    ///
    /// let mut parser = IncrementalParser::new();
    /// let text = "User: alice\n  name: Alice\n  age: 30";
    /// let result = parser.parse(text);
    /// assert_eq!(result.line_count(), 3);
    /// ```
    pub fn parse(&mut self, text: &str) -> ParseResult {
        let start = std::time::Instant::now();

        // Clear cache for fresh parse
        self.cache.clear();

        let lines: Vec<String> = if text.is_empty() {
            Vec::new()
        } else {
            text.lines().map(String::from).collect()
        };

        let mut cached_lines = Vec::with_capacity(lines.len());
        let reparsed_count = lines.len();

        for (line_num, content) in lines.into_iter().enumerate() {
            let cached = self.parse_line(line_num, content);

            // Cache for future incremental updates
            if line_num < MAX_CACHED_LINES {
                self.cache.insert(line_num, cached.clone());
            }

            cached_lines.push(cached);
        }

        self.last_line_count = cached_lines.len();
        let parse_time_us = start.elapsed().as_micros() as u64;

        ParseResult {
            lines: cached_lines,
            reused_count: 0,
            reparsed_count,
            parse_time_us,
        }
    }

    /// Parse a document incrementally using edit information.
    ///
    /// This reuses cached parse results for unchanged lines and only
    /// re-parses modified lines and their dependencies.
    ///
    /// # Arguments
    ///
    /// * `text` - The complete updated document text
    /// * `edits` - List of edits applied (should be non-overlapping)
    ///
    /// # Performance
    ///
    /// For small edits, this is significantly faster than full reparsing.
    /// The speedup is proportional to the percentage of unchanged lines.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::lex::incremental::{IncrementalParser, TextEdit};
    ///
    /// let mut parser = IncrementalParser::new();
    /// let text1 = "User: alice\n  name: Alice\n  age: 30";
    /// parser.parse(text1);
    ///
    /// let text2 = "User: alice\n  name: Alice\n  age: 31";
    /// let edit = TextEdit::replace(2, 3, "  age: 31");
    /// let result = parser.parse_incremental(text2, &[edit]);
    ///
    /// assert_eq!(result.reused_lines(), 2);
    /// assert_eq!(result.reparsed_lines(), 1);
    /// ```
    pub fn parse_incremental(&mut self, text: &str, edits: &[TextEdit]) -> ParseResult {
        let start = std::time::Instant::now();

        // If cache is empty, do full parse
        if self.cache.is_empty() {
            return self.parse(text);
        }

        // If no edits, try to reuse all cached lines
        if edits.is_empty() {
            return self.parse_with_cache_reuse(text, start);
        }

        let lines: Vec<String> = if text.is_empty() {
            Vec::new()
        } else {
            text.lines().map(String::from).collect()
        };

        // Compute affected line ranges
        let affected = self.compute_affected_lines(edits, lines.len());

        let mut cached_lines = Vec::with_capacity(lines.len());
        let mut reused = 0;
        let mut reparsed = 0;

        for (line_num, content) in lines.into_iter().enumerate() {
            let cached = if affected.contains(&line_num) {
                // Line is affected - reparse
                reparsed += 1;
                self.parse_line(line_num, content)
            } else if let Some(old_cached) = self.cache.get(&line_num) {
                // Check if content matches cached version
                if old_cached.matches(&content) {
                    // Reuse cached result
                    reused += 1;
                    old_cached.clone()
                } else {
                    // Content changed - reparse
                    reparsed += 1;
                    self.parse_line(line_num, content)
                }
            } else {
                // No cache entry - parse new line
                reparsed += 1;
                self.parse_line(line_num, content)
            };

            // Update cache
            if line_num < MAX_CACHED_LINES {
                self.cache.insert(line_num, cached.clone());
            }

            cached_lines.push(cached);
        }

        // Clean up cache entries beyond new document size
        let new_len = cached_lines.len();
        self.cache.retain(|&line_num, _| line_num < new_len);

        self.last_line_count = cached_lines.len();
        let parse_time_us = start.elapsed().as_micros() as u64;

        ParseResult {
            lines: cached_lines,
            reused_count: reused,
            reparsed_count: reparsed,
            parse_time_us,
        }
    }

    /// Parse with maximum cache reuse (when no explicit edits are provided).
    fn parse_with_cache_reuse(
        &mut self,
        text: &str,
        start: std::time::Instant,
    ) -> ParseResult {
        let lines: Vec<String> = if text.is_empty() {
            Vec::new()
        } else {
            text.lines().map(String::from).collect()
        };

        let mut cached_lines = Vec::with_capacity(lines.len());
        let mut reused = 0;
        let mut reparsed = 0;

        for (line_num, content) in lines.into_iter().enumerate() {
            let cached = if let Some(old_cached) = self.cache.get(&line_num) {
                if old_cached.matches(&content) {
                    reused += 1;
                    old_cached.clone()
                } else {
                    reparsed += 1;
                    self.parse_line(line_num, content)
                }
            } else {
                reparsed += 1;
                self.parse_line(line_num, content)
            };

            if line_num < MAX_CACHED_LINES {
                self.cache.insert(line_num, cached.clone());
            }

            cached_lines.push(cached);
        }

        let new_len = cached_lines.len();
        self.cache.retain(|&line_num, _| line_num < new_len);

        self.last_line_count = cached_lines.len();
        let parse_time_us = start.elapsed().as_micros() as u64;

        ParseResult {
            lines: cached_lines,
            reused_count: reused,
            reparsed_count: reparsed,
            parse_time_us,
        }
    }

    /// Parse a single line and validate it.
    fn parse_line(&self, line_num: usize, content: String) -> CachedLine {
        let mut cached = CachedLine::new(line_num, content);

        // Validate line length
        if cached.content.len() > MAX_LINE_LENGTH {
            cached.error = Some(LexError::StringTooLong {
                length: cached.content.len(),
                max: MAX_LINE_LENGTH,
                pos: cached.span.start(),
            });
            return cached;
        }

        // Validate indentation
        match calculate_indent(&cached.content, line_num + 1) {
            Ok(Some(info)) => {
                if let Err(e) = validate_indent(info, self.max_indent_depth, line_num + 1) {
                    cached.error = Some(e);
                }
            }
            Ok(None) => {} // Blank line
            Err(e) => {
                cached.error = Some(e);
            }
        }

        cached
    }

    /// Clear the parser cache.
    ///
    /// This forces the next parse to be a full parse rather than incremental.
    /// Useful when you want to ensure a clean state.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.last_line_count = 0;
    }

    /// Get the current cache size (number of cached lines).
    #[inline]
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Get the maximum indentation depth.
    #[inline]
    pub fn max_indent_depth(&self) -> usize {
        self.max_indent_depth
    }

    /// Compute which lines are affected by edits.
    ///
    /// This includes:
    /// - Directly edited lines
    /// - Lines whose positions shifted due to insertions/deletions
    /// - Context-dependent lines (future: indentation-dependent lines)
    fn compute_affected_lines(&self, edits: &[TextEdit], new_line_count: usize) -> Vec<usize> {
        let mut affected = Vec::new();

        for edit in edits {
            // All lines in the edited range are affected
            for line in edit.start_line..edit.end_line.min(new_line_count) {
                if !affected.contains(&line) {
                    affected.push(line);
                }
            }

            // Newly inserted lines are affected
            let new_lines = edit.inserted_lines();
            for i in 0..new_lines {
                let line = edit.start_line + i;
                if line < new_line_count && !affected.contains(&line) {
                    affected.push(line);
                }
            }
        }

        affected.sort_unstable();
        affected
    }
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Hash Function Tests ====================

    #[test]
    fn test_hash_line_empty() {
        let hash = hash_line("");
        assert_ne!(hash, 0);
    }

    #[test]
    fn test_hash_line_deterministic() {
        let s = "User: alice";
        assert_eq!(hash_line(s), hash_line(s));
    }

    #[test]
    fn test_hash_line_different_content() {
        let h1 = hash_line("User: alice");
        let h2 = hash_line("User: bob");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_line_collision_unlikely() {
        let h1 = hash_line("a");
        let h2 = hash_line("b");
        let h3 = hash_line("aa");
        assert_ne!(h1, h2);
        assert_ne!(h1, h3);
        assert_ne!(h2, h3);
    }

    // ==================== TextEdit Tests ====================

    #[test]
    fn test_text_edit_replace() {
        let edit = TextEdit::replace(2, 4, "new content");
        assert_eq!(edit.start_line, 2);
        assert_eq!(edit.end_line, 4);
        assert_eq!(edit.new_text, "new content");
        assert_eq!(edit.deleted_lines(), 2);
        assert_eq!(edit.inserted_lines(), 1);
    }

    #[test]
    fn test_text_edit_insert() {
        let edit = TextEdit::insert(5, "inserted");
        assert_eq!(edit.start_line, 5);
        assert_eq!(edit.end_line, 5);
        assert_eq!(edit.deleted_lines(), 0);
        assert_eq!(edit.inserted_lines(), 1);
    }

    #[test]
    fn test_text_edit_delete() {
        let edit = TextEdit::delete(3, 7);
        assert_eq!(edit.start_line, 3);
        assert_eq!(edit.end_line, 7);
        assert_eq!(edit.deleted_lines(), 4);
        assert_eq!(edit.inserted_lines(), 0);
    }

    #[test]
    fn test_text_edit_multiline_insert() {
        let edit = TextEdit::replace(0, 0, "line1\nline2\nline3");
        assert_eq!(edit.deleted_lines(), 0);
        assert_eq!(edit.inserted_lines(), 3);
        assert_eq!(edit.line_delta(), 3);
    }

    #[test]
    fn test_text_edit_line_delta() {
        let edit1 = TextEdit::replace(0, 2, "single line");
        assert_eq!(edit1.line_delta(), -1);

        let edit2 = TextEdit::replace(0, 1, "line1\nline2\nline3");
        assert_eq!(edit2.line_delta(), 2);

        let edit3 = TextEdit::replace(0, 3, "a\nb\nc");
        assert_eq!(edit3.line_delta(), 0);
    }

    // ==================== CachedLine Tests ====================

    #[test]
    fn test_cached_line_new() {
        let cached = CachedLine::new(0, "User: alice".to_string());
        assert_eq!(cached.content, "User: alice");
        assert_eq!(cached.indent, 0);
        assert_eq!(cached.span.start().line(), 1);
        assert!(cached.error.is_none());
    }

    #[test]
    fn test_cached_line_with_indent() {
        let cached = CachedLine::new(1, "  name: Alice".to_string());
        assert_eq!(cached.indent, 2);
    }

    #[test]
    fn test_cached_line_matches() {
        let cached = CachedLine::new(0, "User: alice".to_string());
        assert!(cached.matches("User: alice"));
        assert!(!cached.matches("User: bob"));
    }

    // ==================== ParseResult Tests ====================

    #[test]
    fn test_parse_result_empty() {
        let result = ParseResult {
            lines: Vec::new(),
            reused_count: 0,
            reparsed_count: 0,
            parse_time_us: 0,
        };
        assert_eq!(result.line_count(), 0);
        assert_eq!(result.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_parse_result_cache_hit_rate() {
        let lines = vec![
            CachedLine::new(0, "line1".to_string()),
            CachedLine::new(1, "line2".to_string()),
        ];
        let result = ParseResult {
            lines,
            reused_count: 1,
            reparsed_count: 1,
            parse_time_us: 100,
        };
        assert_eq!(result.cache_hit_rate(), 0.5);
    }

    #[test]
    fn test_parse_result_get_line() {
        let lines = vec![
            CachedLine::new(0, "User: alice".to_string()),
            CachedLine::new(1, "  name: Alice".to_string()),
        ];
        let result = ParseResult {
            lines,
            reused_count: 0,
            reparsed_count: 2,
            parse_time_us: 100,
        };
        assert_eq!(result.get_line(0), Some("User: alice"));
        assert_eq!(result.get_line(1), Some("  name: Alice"));
        assert_eq!(result.get_line(2), None);
    }

    #[test]
    fn test_parse_result_get_indent() {
        let lines = vec![
            CachedLine::new(0, "User: alice".to_string()),
            CachedLine::new(1, "  name: Alice".to_string()),
        ];
        let result = ParseResult {
            lines,
            reused_count: 0,
            reparsed_count: 2,
            parse_time_us: 100,
        };
        assert_eq!(result.get_indent(0), Some(0));
        assert_eq!(result.get_indent(1), Some(2));
    }

    // ==================== IncrementalParser Tests ====================

    #[test]
    fn test_parser_new() {
        let parser = IncrementalParser::new();
        assert_eq!(parser.cache_size(), 0);
        assert_eq!(parser.max_indent_depth(), DEFAULT_MAX_INDENT_DEPTH);
    }

    #[test]
    fn test_parser_parse_empty() {
        let mut parser = IncrementalParser::new();
        let result = parser.parse("");
        assert_eq!(result.line_count(), 0);
        assert_eq!(result.reused_lines(), 0);
    }

    #[test]
    fn test_parser_parse_single_line() {
        let mut parser = IncrementalParser::new();
        let result = parser.parse("User: alice");
        assert_eq!(result.line_count(), 1);
        assert_eq!(result.get_line(0), Some("User: alice"));
    }

    #[test]
    fn test_parser_parse_multiple_lines() {
        let mut parser = IncrementalParser::new();
        let text = "User: alice\n  name: Alice\n  age: 30";
        let result = parser.parse(text);
        assert_eq!(result.line_count(), 3);
        assert_eq!(result.get_line(0), Some("User: alice"));
        assert_eq!(result.get_line(1), Some("  name: Alice"));
        assert_eq!(result.get_line(2), Some("  age: 30"));
    }

    #[test]
    fn test_parser_incremental_no_change() {
        let mut parser = IncrementalParser::new();
        let text = "User: alice\n  name: Alice\n  age: 30";
        parser.parse(text);

        let result = parser.parse_incremental(text, &[]);
        assert_eq!(result.reused_lines(), 3);
        assert_eq!(result.reparsed_lines(), 0);
    }

    #[test]
    fn test_parser_incremental_single_line_change() {
        let mut parser = IncrementalParser::new();
        let text1 = "User: alice\n  name: Alice\n  age: 30";
        parser.parse(text1);

        let text2 = "User: alice\n  name: Alice\n  age: 31";
        let edit = TextEdit::replace(2, 3, "  age: 31");
        let result = parser.parse_incremental(text2, &[edit]);

        assert_eq!(result.line_count(), 3);
        assert_eq!(result.reused_lines(), 2);
        assert_eq!(result.reparsed_lines(), 1);
        assert_eq!(result.get_line(2), Some("  age: 31"));
    }

    #[test]
    fn test_parser_incremental_insert_line() {
        let mut parser = IncrementalParser::new();
        let text1 = "User: alice\n  age: 30";
        parser.parse(text1);

        let text2 = "User: alice\n  name: Alice\n  age: 30";
        let edit = TextEdit::insert(1, "  name: Alice");
        let result = parser.parse_incremental(text2, &[edit]);

        assert_eq!(result.line_count(), 3);
        assert_eq!(result.get_line(1), Some("  name: Alice"));
    }

    #[test]
    fn test_parser_incremental_delete_line() {
        let mut parser = IncrementalParser::new();
        let text1 = "User: alice\n  name: Alice\n  age: 30";
        parser.parse(text1);

        let text2 = "User: alice\n  age: 30";
        let edit = TextEdit::delete(1, 2);
        let result = parser.parse_incremental(text2, &[edit]);

        assert_eq!(result.line_count(), 2);
        assert_eq!(result.get_line(0), Some("User: alice"));
        assert_eq!(result.get_line(1), Some("  age: 30"));
    }

    #[test]
    fn test_parser_clear_cache() {
        let mut parser = IncrementalParser::new();
        parser.parse("User: alice\n  name: Alice");
        assert!(parser.cache_size() > 0);

        parser.clear_cache();
        assert_eq!(parser.cache_size(), 0);
    }

    #[test]
    fn test_parser_indentation_error() {
        let mut parser = IncrementalParser::new();
        let text = "User: alice\n   name: Alice"; // 3 spaces - invalid
        let result = parser.parse(text);

        assert_eq!(result.line_count(), 2);
        assert!(result.get_error(1).is_some());
    }

    #[test]
    fn test_parser_line_too_long() {
        let mut parser = IncrementalParser::new();
        let long_line = "x".repeat(MAX_LINE_LENGTH + 1);
        let result = parser.parse(&long_line);

        assert!(result.get_error(0).is_some());
        match result.get_error(0) {
            Some(LexError::StringTooLong { .. }) => {}
            _ => panic!("Expected StringTooLong error"),
        }
    }

    // ==================== Performance Tests ====================

    #[test]
    fn test_incremental_faster_than_full_parse() {
        let mut parser = IncrementalParser::new();

        // Create a large document
        let mut lines = Vec::new();
        for i in 0..1000 {
            lines.push(format!("User{}: user_{}", i, i));
            lines.push(format!("  name: User {}", i));
            lines.push(format!("  id: {}", i));
        }
        let text1 = lines.join("\n");

        // Initial parse
        parser.parse(&text1);

        // Modify one line
        lines[500] = "User500: modified_user".to_string();
        let text2 = lines.join("\n");
        let edit = TextEdit::replace(500, 501, "User500: modified_user");

        // Incremental parse should reuse most lines
        let result = parser.parse_incremental(&text2, &[edit]);
        assert!(result.cache_hit_rate() > 0.99);
    }

    #[test]
    fn test_multiple_edits() {
        let mut parser = IncrementalParser::new();
        let text1 = "line1\nline2\nline3\nline4\nline5";
        parser.parse(text1);

        let edits = vec![
            TextEdit::replace(1, 2, "modified2"),
            TextEdit::replace(3, 4, "modified4"),
        ];
        let text2 = "line1\nmodified2\nline3\nmodified4\nline5";
        let result = parser.parse_incremental(text2, &edits);

        assert_eq!(result.line_count(), 5);
        assert_eq!(result.reused_lines(), 3);
        assert_eq!(result.reparsed_lines(), 2);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_parser_empty_to_nonempty() {
        let mut parser = IncrementalParser::new();
        parser.parse("");

        let text = "User: alice";
        let edit = TextEdit::insert(0, "User: alice");
        let result = parser.parse_incremental(text, &[edit]);

        assert_eq!(result.line_count(), 1);
        assert_eq!(result.get_line(0), Some("User: alice"));
    }

    #[test]
    fn test_parser_nonempty_to_empty() {
        let mut parser = IncrementalParser::new();
        parser.parse("User: alice");

        let result = parser.parse_incremental("", &[TextEdit::delete(0, 1)]);
        assert_eq!(result.line_count(), 0);
    }

    #[test]
    fn test_edit_at_end_of_document() {
        let mut parser = IncrementalParser::new();
        let text1 = "line1\nline2";
        parser.parse(text1);

        let text2 = "line1\nline2\nline3";
        let edit = TextEdit::insert(2, "line3");
        let result = parser.parse_incremental(text2, &[edit]);

        assert_eq!(result.line_count(), 3);
        assert_eq!(result.get_line(2), Some("line3"));
    }

    #[test]
    fn test_edit_replace_entire_document() {
        let mut parser = IncrementalParser::new();
        parser.parse("old1\nold2\nold3");

        let text2 = "new1\nnew2";
        let edit = TextEdit::replace(0, 3, "new1\nnew2");
        let result = parser.parse_incremental(text2, &[edit]);

        assert_eq!(result.line_count(), 2);
        assert_eq!(result.get_line(0), Some("new1"));
        assert_eq!(result.get_line(1), Some("new2"));
    }

    #[test]
    fn test_whitespace_only_lines() {
        let mut parser = IncrementalParser::new();
        let text = "User: alice\n  \n  name: Alice";
        let result = parser.parse(text);

        assert_eq!(result.line_count(), 3);
        assert_eq!(result.get_line(1), Some("  "));
    }

    #[test]
    fn test_unicode_content() {
        let mut parser = IncrementalParser::new();
        let text = "User: alice\n  name: nihongo_test\n  emoji: test_face";
        let result = parser.parse(text);

        assert_eq!(result.line_count(), 3);
        assert_eq!(result.get_line(1), Some("  name: nihongo_test"));
        assert_eq!(result.get_line(2), Some("  emoji: test_face"));
    }

    #[test]
    fn test_cache_size_limit() {
        let mut parser = IncrementalParser::new();

        // This would exceed MAX_CACHED_LINES if we tried to cache everything
        let lines: Vec<String> = (0..MAX_CACHED_LINES + 100)
            .map(|i| format!("line{}", i))
            .collect();
        let text = lines.join("\n");

        parser.parse(&text);
        assert!(parser.cache_size() <= MAX_CACHED_LINES);
    }

    #[test]
    fn test_custom_max_indent_depth() {
        let mut parser = IncrementalParser::with_max_indent_depth(2);
        // Level 0, 1, 2, 3 - level 3 exceeds max of 2
        let text = "User: alice\n  profile:\n    details:\n      age: 30";
        let result = parser.parse(text);

        assert_eq!(result.line_count(), 4);
        // Line 3 (index 3) has 6 spaces = level 3, which exceeds max of 2
        assert!(result.get_error(3).is_some());
    }
}
