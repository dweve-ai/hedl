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

//! Source position and range utilities for fix application

/// Source position with line and column
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePosition {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column offset in bytes (0-indexed)
    pub column: usize,
}

impl SourcePosition {
    /// Create a new source position
    #[must_use]
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Create a position at the start of a file
    #[must_use]
    pub fn start() -> Self {
        Self { line: 1, column: 0 }
    }

    /// Create a position at the end of a file (max values)
    #[must_use]
    pub fn end() -> Self {
        Self {
            line: usize::MAX,
            column: usize::MAX,
        }
    }
}

/// Source range for fix application
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRange {
    /// Start position (inclusive)
    pub start: SourcePosition,
    /// End position (exclusive)
    pub end: SourcePosition,
}

impl SourceRange {
    /// Create a new source range
    #[must_use]
    pub fn new(start: SourcePosition, end: SourcePosition) -> Self {
        Self { start, end }
    }

    /// Create a range spanning an entire line
    #[must_use]
    pub fn line(line: usize) -> Self {
        Self {
            start: SourcePosition::new(line, 0),
            end: SourcePosition::new(line + 1, 0),
        }
    }

    /// Create a single-point range (empty)
    #[must_use]
    pub fn point(pos: SourcePosition) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    /// Check if this range overlaps with another
    #[must_use]
    pub fn overlaps(&self, other: &SourceRange) -> bool {
        // Ranges overlap if they are not completely disjoint
        !(self.end <= other.start || other.end <= self.start)
    }

    /// Check if this range contains another range
    #[must_use]
    pub fn contains(&self, other: &SourceRange) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// Check if this range contains a position
    #[must_use]
    pub fn contains_position(&self, pos: &SourcePosition) -> bool {
        self.start <= *pos && *pos < self.end
    }

    /// Check if this range is empty (start == end)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Merge two ranges into the smallest range containing both
    #[must_use]
    pub fn merge(&self, other: &SourceRange) -> SourceRange {
        SourceRange {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Get the length in lines
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.end.line.saturating_sub(self.start.line)
    }

    /// Check if this is a valid range (start <= end)
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.start <= self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_position_new() {
        let pos = SourcePosition::new(42, 10);
        assert_eq!(pos.line, 42);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_source_position_start() {
        let pos = SourcePosition::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_source_position_end() {
        let pos = SourcePosition::end();
        assert_eq!(pos.line, usize::MAX);
        assert_eq!(pos.column, usize::MAX);
    }

    #[test]
    fn test_source_position_ordering() {
        let pos1 = SourcePosition::new(1, 5);
        let pos2 = SourcePosition::new(1, 10);
        let pos3 = SourcePosition::new(2, 0);

        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
        assert!(pos1 < pos3);
    }

    #[test]
    fn test_source_position_equality() {
        let pos1 = SourcePosition::new(5, 10);
        let pos2 = SourcePosition::new(5, 10);
        let pos3 = SourcePosition::new(5, 11);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_source_range_new() {
        let start = SourcePosition::new(1, 0);
        let end = SourcePosition::new(1, 10);
        let range = SourceRange::new(start, end);

        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_source_range_line() {
        let range = SourceRange::line(5);
        assert_eq!(range.start.line, 5);
        assert_eq!(range.start.column, 0);
        assert_eq!(range.end.line, 6);
        assert_eq!(range.end.column, 0);
    }

    #[test]
    fn test_source_range_point() {
        let pos = SourcePosition::new(3, 7);
        let range = SourceRange::point(pos);
        assert_eq!(range.start, pos);
        assert_eq!(range.end, pos);
        assert!(range.is_empty());
    }

    #[test]
    fn test_range_overlaps_true() {
        let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
        let range2 = SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15));

        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));
    }

    #[test]
    fn test_range_overlaps_false() {
        let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
        let range2 = SourceRange::new(SourcePosition::new(1, 10), SourcePosition::new(1, 15));

        assert!(!range1.overlaps(&range2));
        assert!(!range2.overlaps(&range1));
    }

    #[test]
    fn test_range_overlaps_adjacent() {
        let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
        let range2 = SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 10));

        // Adjacent ranges don't overlap (end is exclusive)
        assert!(!range1.overlaps(&range2));
    }

    #[test]
    fn test_range_overlaps_contained() {
        let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 20));
        let range2 = SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15));

        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));
    }

    #[test]
    fn test_range_contains_range() {
        let outer = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(3, 0));
        let inner = SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 10));

        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_range_contains_position() {
        let range = SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15));

        assert!(range.contains_position(&SourcePosition::new(1, 5)));
        assert!(range.contains_position(&SourcePosition::new(1, 10)));
        assert!(!range.contains_position(&SourcePosition::new(1, 15))); // end is exclusive
        assert!(!range.contains_position(&SourcePosition::new(1, 0)));
        assert!(!range.contains_position(&SourcePosition::new(2, 0)));
    }

    #[test]
    fn test_range_is_empty() {
        let empty = SourceRange::point(SourcePosition::new(1, 5));
        assert!(empty.is_empty());

        let non_empty = SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 10));
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_range_merge() {
        let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
        let range2 = SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(2, 0));

        let merged = range1.merge(&range2);
        assert_eq!(merged.start, SourcePosition::new(1, 0));
        assert_eq!(merged.end, SourcePosition::new(2, 0));
    }

    #[test]
    fn test_range_merge_disjoint() {
        let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
        let range2 = SourceRange::new(SourcePosition::new(3, 0), SourcePosition::new(3, 5));

        let merged = range1.merge(&range2);
        assert_eq!(merged.start, SourcePosition::new(1, 0));
        assert_eq!(merged.end, SourcePosition::new(3, 5));
    }

    #[test]
    fn test_range_line_count() {
        let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(5, 0));
        assert_eq!(range.line_count(), 4);

        let single_line = SourceRange::new(SourcePosition::new(3, 5), SourcePosition::new(3, 10));
        assert_eq!(single_line.line_count(), 0);
    }

    #[test]
    fn test_range_is_valid() {
        let valid = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(2, 0));
        assert!(valid.is_valid());

        let invalid = SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(1, 0));
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_range_equality() {
        let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
        let range2 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
        let range3 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 11));

        assert_eq!(range1, range2);
        assert_ne!(range1, range3);
    }

    #[test]
    fn test_multiline_range() {
        let range = SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(5, 10));

        assert!(range.is_valid());
        assert_eq!(range.line_count(), 4);
        assert!(range.contains_position(&SourcePosition::new(3, 0)));
        assert!(!range.contains_position(&SourcePosition::new(6, 0)));
    }
}
