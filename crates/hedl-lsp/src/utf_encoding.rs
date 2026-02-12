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

//! Utility functions for safe string handling in LSP operations.

use tower_lsp::lsp_types::Position;

/// Safely get a string slice up to a character position, ensuring UTF-8 character boundaries.
///
/// # Security
///
/// This function prevents panics from slicing at invalid UTF-8 boundaries. If the requested
/// position falls in the middle of a multi-byte character, it rounds down to the nearest
/// valid character boundary.
///
/// # Arguments
///
/// * `s` - The string to slice
/// * `char_pos` - The desired character position (in bytes)
///
/// # Returns
///
/// A string slice from the start to the nearest valid position <= `char_pos`
///
/// # Example
///
/// ```
/// use hedl_lsp::utf_encoding::safe_slice_to;
///
/// let s = "Hello 世界";  // Multi-byte UTF-8 characters
/// assert_eq!(safe_slice_to(s, 6), "Hello ");
/// // Position 7 would be mid-character, so it rounds down to 6
/// assert_eq!(safe_slice_to(s, 7), "Hello ");
/// ```
#[must_use]
pub fn safe_slice_to(s: &str, char_pos: usize) -> &str {
    if char_pos >= s.len() {
        return s;
    }

    // Check if char_pos is already a valid UTF-8 boundary
    if s.is_char_boundary(char_pos) {
        &s[..char_pos]
    } else {
        // Find the nearest valid boundary before char_pos
        let mut pos = char_pos;
        while pos > 0 && !s.is_char_boundary(pos) {
            pos -= 1;
        }
        &s[..pos]
    }
}

/// Safely get a string slice from a character position, ensuring UTF-8 character boundaries.
///
/// # Security
///
/// This function prevents panics from slicing at invalid UTF-8 boundaries. If the requested
/// position falls in the middle of a multi-byte character, it rounds down to the nearest
/// valid character boundary.
///
/// # Arguments
///
/// * `s` - The string to slice
/// * `char_pos` - The desired starting character position (in bytes)
///
/// # Returns
///
/// A string slice from the nearest valid position <= `char_pos` to the end
///
/// # Example
///
/// ```
/// use hedl_lsp::utf_encoding::safe_slice_from;
///
/// let s = "Hello 世界";
/// assert_eq!(safe_slice_from(s, 6), "世界");
/// // Position 7 would be mid-character, so it rounds down to 6
/// assert_eq!(safe_slice_from(s, 7), "世界");
/// ```
#[must_use]
pub fn safe_slice_from(s: &str, char_pos: usize) -> &str {
    if char_pos >= s.len() {
        return "";
    }

    // Check if char_pos is already a valid UTF-8 boundary
    if s.is_char_boundary(char_pos) {
        &s[char_pos..]
    } else {
        // Find the nearest valid boundary before char_pos
        let mut pos = char_pos;
        while pos > 0 && !s.is_char_boundary(pos) {
            pos -= 1;
        }
        &s[pos..]
    }
}

/// Convert LSP UTF-16 code unit position to byte offset in a UTF-8 string.
///
/// LSP positions use UTF-16 code units, but Rust strings use UTF-8 bytes.
/// This function converts from UTF-16 column position to byte offset.
///
/// # Arguments
///
/// * `line` - The line content as a UTF-8 string
/// * `utf16_col` - The column position in UTF-16 code units
///
/// # Returns
///
/// The byte offset in the UTF-8 string
///
/// # Example
///
/// ```
/// use hedl_lsp::utf_encoding::utf16_col_to_byte_offset;
///
/// let line = "Hello 世界";  // "世" is 1 char, 3 bytes, 1 UTF-16 code unit
/// assert_eq!(utf16_col_to_byte_offset(line, 6), 6);  // After "Hello "
/// assert_eq!(utf16_col_to_byte_offset(line, 7), 9);  // After "世"
/// ```
#[must_use]
pub fn utf16_col_to_byte_offset(line: &str, utf16_col: u32) -> usize {
    let mut utf16_count = 0;
    let mut byte_offset = 0;

    for ch in line.chars() {
        if utf16_count >= utf16_col {
            break;
        }

        // Each char contributes 1 or 2 UTF-16 code units
        utf16_count += ch.len_utf16() as u32;
        byte_offset += ch.len_utf8();
    }

    byte_offset
}

/// Convert LSP Position (UTF-16) to byte offset in document content.
///
/// # Arguments
///
/// * `content` - The full document content
/// * `position` - The LSP position in UTF-16 code units
///
/// # Returns
///
/// The byte offset in the document content
#[must_use]
pub fn lsp_position_to_byte_offset(content: &str, position: Position) -> usize {
    let lines: Vec<&str> = content.lines().collect();
    let line_num = position.line as usize;

    if line_num >= lines.len() {
        return content.len();
    }

    // Calculate byte offset to start of line
    let mut byte_offset = 0;
    for i in 0..line_num {
        if i < lines.len() {
            byte_offset += lines[i].len() + 1; // +1 for newline
        }
    }

    // Add byte offset within the line
    let line = lines[line_num];
    byte_offset + utf16_col_to_byte_offset(line, position.character)
}

/// Get the line content and convert UTF-16 position to byte offset within that line.
///
/// # Arguments
///
/// * `content` - The full document content
/// * `position` - The LSP position in UTF-16 code units
///
/// # Returns
///
/// A tuple of (`line_content`, `byte_offset_in_line`)
#[must_use]
pub fn get_line_and_byte_offset(content: &str, position: Position) -> Option<(&str, usize)> {
    let lines: Vec<&str> = content.lines().collect();
    let line_num = position.line as usize;

    if line_num >= lines.len() {
        return None;
    }

    let line = lines[line_num];
    let byte_offset = utf16_col_to_byte_offset(line, position.character);

    Some((line, byte_offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_slice_to_ascii() {
        let s = "Hello, World!";
        assert_eq!(safe_slice_to(s, 5), "Hello");
        assert_eq!(safe_slice_to(s, 0), "");
        assert_eq!(safe_slice_to(s, 100), s);
    }

    #[test]
    fn test_safe_slice_to_utf8() {
        let s = "Hello 世界"; // "世" is 3 bytes at position 6
        assert_eq!(safe_slice_to(s, 6), "Hello ");
        // Position 7 is mid-character, should round down to 6
        assert_eq!(safe_slice_to(s, 7), "Hello ");
        // Position 8 is mid-character, should round down to 6
        assert_eq!(safe_slice_to(s, 8), "Hello ");
        // Position 9 is start of "界"
        assert_eq!(safe_slice_to(s, 9), "Hello 世");
    }

    #[test]
    fn test_safe_slice_from_ascii() {
        let s = "Hello, World!";
        assert_eq!(safe_slice_from(s, 7), "World!");
        assert_eq!(safe_slice_from(s, 0), s);
        assert_eq!(safe_slice_from(s, 100), "");
    }

    #[test]
    fn test_safe_slice_from_utf8() {
        let s = "Hello 世界";
        assert_eq!(safe_slice_from(s, 6), "世界");
        // Position 7 is mid-character, should round down to 6
        assert_eq!(safe_slice_from(s, 7), "世界");
        // Position 8 is mid-character, should round down to 6
        assert_eq!(safe_slice_from(s, 8), "世界");
        // Position 9 is start of "界"
        assert_eq!(safe_slice_from(s, 9), "界");
    }

    #[test]
    fn test_emoji_handling() {
        // Emoji are multi-byte
        let s = "Hi 👋 there"; // 👋 is 4 bytes
        assert_eq!(safe_slice_to(s, 3), "Hi ");
        // Positions 4-6 are mid-emoji, should round down to 3
        assert_eq!(safe_slice_to(s, 4), "Hi ");
        assert_eq!(safe_slice_to(s, 5), "Hi ");
        assert_eq!(safe_slice_to(s, 6), "Hi ");
        // Position 7 is after emoji
        assert_eq!(safe_slice_to(s, 7), "Hi 👋");
    }

    #[test]
    fn test_utf16_col_to_byte_offset_ascii() {
        let line = "Hello, World!";
        assert_eq!(utf16_col_to_byte_offset(line, 0), 0);
        assert_eq!(utf16_col_to_byte_offset(line, 5), 5);
        assert_eq!(utf16_col_to_byte_offset(line, 13), 13);
    }

    #[test]
    fn test_utf16_col_to_byte_offset_utf8() {
        // "世" is 1 char, 3 bytes, 1 UTF-16 code unit
        let line = "Hello 世界";
        assert_eq!(utf16_col_to_byte_offset(line, 0), 0);
        assert_eq!(utf16_col_to_byte_offset(line, 6), 6); // After "Hello "
        assert_eq!(utf16_col_to_byte_offset(line, 7), 9); // After "世"
        assert_eq!(utf16_col_to_byte_offset(line, 8), 12); // After "界"
    }

    #[test]
    fn test_utf16_col_to_byte_offset_emoji() {
        // 👋 is 1 char, 4 bytes, 2 UTF-16 code units (surrogate pair)
        let line = "Hi 👋 there";
        assert_eq!(utf16_col_to_byte_offset(line, 0), 0);
        assert_eq!(utf16_col_to_byte_offset(line, 3), 3); // After "Hi "
        assert_eq!(utf16_col_to_byte_offset(line, 5), 7); // After "👋" (2 UTF-16 units)
        assert_eq!(utf16_col_to_byte_offset(line, 6), 8); // After " "
    }

    #[test]
    fn test_lsp_position_to_byte_offset() {
        let content = "Hello\n世界\n👋";
        // Line 0, col 0
        assert_eq!(lsp_position_to_byte_offset(content, Position::new(0, 0)), 0);
        // Line 0, col 5
        assert_eq!(lsp_position_to_byte_offset(content, Position::new(0, 5)), 5);
        // Line 1, col 0 (start of "世界")
        assert_eq!(lsp_position_to_byte_offset(content, Position::new(1, 0)), 6);
        // Line 1, col 1 (after "世")
        assert_eq!(lsp_position_to_byte_offset(content, Position::new(1, 1)), 9);
        // Line 2, col 0 (start of "👋")
        assert_eq!(
            lsp_position_to_byte_offset(content, Position::new(2, 0)),
            13
        );
    }

    #[test]
    fn test_get_line_and_byte_offset() {
        let content = "Hello\n世界\n👋";

        let (line, offset) = get_line_and_byte_offset(content, Position::new(0, 5)).unwrap();
        assert_eq!(line, "Hello");
        assert_eq!(offset, 5);

        let (line, offset) = get_line_and_byte_offset(content, Position::new(1, 1)).unwrap();
        assert_eq!(line, "世界");
        assert_eq!(offset, 3); // 3 bytes for "世"

        let (line, offset) = get_line_and_byte_offset(content, Position::new(2, 2)).unwrap();
        assert_eq!(line, "👋");
        assert_eq!(offset, 4); // 4 bytes for "👋"
    }
}
