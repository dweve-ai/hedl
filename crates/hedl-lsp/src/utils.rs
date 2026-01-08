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
/// A string slice from the start to the nearest valid position <= char_pos
///
/// # Example
///
/// ```
/// use hedl_lsp::utils::safe_slice_to;
///
/// let s = "Hello 世界";  // Multi-byte UTF-8 characters
/// assert_eq!(safe_slice_to(s, 6), "Hello ");
/// // Position 7 would be mid-character, so it rounds down to 6
/// assert_eq!(safe_slice_to(s, 7), "Hello ");
/// ```
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
/// A string slice from the nearest valid position <= char_pos to the end
///
/// # Example
///
/// ```
/// use hedl_lsp::utils::safe_slice_from;
///
/// let s = "Hello 世界";
/// assert_eq!(safe_slice_from(s, 6), "世界");
/// // Position 7 would be mid-character, so it rounds down to 6
/// assert_eq!(safe_slice_from(s, 7), "世界");
/// ```
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
}
