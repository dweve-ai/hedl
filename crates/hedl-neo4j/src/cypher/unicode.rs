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

//! Unicode security utilities for Cypher identifiers.
//!
//! This module provides Unicode normalization and dangerous character filtering
//! to prevent homograph attacks, injection attacks, and other Unicode-based exploits.

use unicode_normalization::UnicodeNormalization;

/// Normalize a string to NFC (Canonical Composition) form.
///
/// This prevents homograph attacks where visually similar Unicode characters
/// are used to bypass security checks. NFC normalization ensures that
/// characters like "é" (U+00E9) and "é" (U+0065 U+0301) are treated identically.
///
/// # Security
///
/// Unicode normalization is essential for:
/// - Preventing homograph attacks (e.g., Cyrillic 'а' vs Latin 'a')
/// - Ensuring consistent property name handling
/// - Avoiding duplicate keys that appear identical but have different byte representations
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::cypher::normalize_unicode;
/// // These two strings look identical but have different representations
/// let composed = "café";  // é is U+00E9
/// let decomposed = "café"; // é is U+0065 + U+0301
/// assert_eq!(normalize_unicode(composed), normalize_unicode(decomposed));
/// ```
#[must_use]
pub fn normalize_unicode(s: &str) -> String {
    s.nfc().collect()
}

/// Check if a character is dangerous Unicode that should be filtered.
///
/// This includes:
/// - Control characters (C0 and C1 control codes)
/// - Zero-width characters (ZWNJ, ZWJ, Zero-width space)
/// - Directional formatting (LTR, RTL overrides and marks)
/// - Other format characters that could be used for attacks
///
/// # Security
///
/// Filtering dangerous Unicode prevents:
/// - Invisible character injection (zero-width spaces)
/// - Text direction attacks (RTL/LTR overrides)
/// - Control character exploits (null bytes, newlines in identifiers)
/// - Format character abuse (soft hyphens, vowel separators)
///
/// # Note
///
/// This function is public for use in other cypher submodules (like escape.rs)
/// but is not re-exported at the crate level since it's an implementation detail.
#[must_use]
pub fn is_dangerous_unicode(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            // Zero-width characters
            '\u{200B}' // Zero-width space
            | '\u{200C}' // Zero-width non-joiner
            | '\u{200D}' // Zero-width joiner
            | '\u{FEFF}' // Zero-width no-break space
            // Directional formatting
            | '\u{202A}' // Left-to-right embedding
            | '\u{202B}' // Right-to-left embedding
            | '\u{202C}' // Pop directional formatting
            | '\u{202D}' // Left-to-right override
            | '\u{202E}' // Right-to-left override
            | '\u{2066}' // Left-to-right isolate
            | '\u{2067}' // Right-to-left isolate
            | '\u{2068}' // First strong isolate
            | '\u{2069}' // Pop directional isolate
            // Other potentially dangerous format characters
            | '\u{00AD}' // Soft hyphen
            | '\u{061C}' // Arabic letter mark
            | '\u{180E}' // Mongolian vowel separator
        )
}

/// Check if a character is dangerous Unicode for property values.
///
/// This is a less strict version of `is_dangerous_unicode` that preserves
/// harmless whitespace characters (newlines, tabs) which are legitimate
/// content in text property values.
///
/// Filters:
/// - Null bytes and other harmful control characters
/// - Zero-width characters (ZWNJ, ZWJ, Zero-width space)
/// - Directional formatting (LTR, RTL overrides and marks)
///
/// Preserves:
/// - Newlines (`\n`)
/// - Carriage returns (`\r`)
/// - Tabs (`\t`)
///
/// # Security
///
/// Property values need to preserve legitimate text formatting while still
/// filtering truly dangerous characters that could enable attacks.
#[must_use]
pub fn is_dangerous_unicode_for_values(c: char) -> bool {
    // Preserve common whitespace: newlines, carriage returns, tabs
    if matches!(c, '\n' | '\r' | '\t') {
        return false;
    }

    // Filter null bytes and other control characters
    if c.is_control() {
        return true;
    }

    // Filter dangerous format characters
    matches!(
        c,
        // Zero-width characters
        '\u{200B}' // Zero-width space
        | '\u{200C}' // Zero-width non-joiner
        | '\u{200D}' // Zero-width joiner
        | '\u{FEFF}' // Zero-width no-break space
        // Directional formatting
        | '\u{202A}' // Left-to-right embedding
        | '\u{202B}' // Right-to-left embedding
        | '\u{202C}' // Pop directional formatting
        | '\u{202D}' // Left-to-right override
        | '\u{202E}' // Right-to-left override
        | '\u{2066}' // Left-to-right isolate
        | '\u{2067}' // Right-to-left isolate
        | '\u{2068}' // First strong isolate
        | '\u{2069}' // Pop directional isolate
        // Other potentially dangerous format characters
        | '\u{00AD}' // Soft hyphen
        | '\u{061C}' // Arabic letter mark
        | '\u{180E}' // Mongolian vowel separator
    )
}

/// Sanitize an identifier by applying Unicode normalization and filtering dangerous characters.
///
/// This is the **canonical sanitization function** used by all identifier escaping functions.
/// It applies two security layers:
///
/// 1. **Unicode Normalization (NFC)**: Prevents homograph attacks
/// 2. **Dangerous Character Filtering**: Removes control chars, zero-width chars, and directional formatting
///
/// # Security
///
/// This function is security-critical for preventing:
/// - Homograph attacks (visually similar Unicode characters)
/// - Invisible character injection (zero-width spaces)
/// - Text direction manipulation (RTL/LTR overrides)
/// - Control character exploits (null bytes, newlines, tabs)
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::cypher::sanitize_identifier;
/// // Remove dangerous characters
/// assert_eq!(sanitize_identifier("name\u{200B}test"), "nametest");
///
/// // Normalize Unicode
/// let result = sanitize_identifier("café");
/// assert_eq!(result, "café"); // NFC normalized
///
/// // Filter control characters
/// assert_eq!(sanitize_identifier("name\x00test"), "nametest");
/// ```
#[must_use]
pub fn sanitize_identifier(s: &str) -> String {
    // Security Layer 1: Normalize Unicode to prevent homograph attacks
    let normalized = normalize_unicode(s);

    // Security Layer 2: Filter dangerous Unicode characters
    normalized
        .chars()
        .filter(|c| !is_dangerous_unicode(*c))
        .collect()
}

/// Sanitize a string value by applying Unicode normalization and filtering dangerous characters.
///
/// Unlike `sanitize_identifier`, this function preserves harmless whitespace characters
/// (newlines, tabs, carriage returns) which are legitimate content in text property values.
///
/// # Security
///
/// Applies two security layers with property-value-appropriate filtering:
/// 1. **Unicode Normalization (NFC)**: Prevents homograph attacks
/// 2. **Dangerous Character Filtering**: Removes null bytes, zero-width chars, and directional formatting
///
/// # Preserved Characters
///
/// - Newlines (`\n`) - legitimate text formatting
/// - Carriage returns (`\r`) - Windows line endings
/// - Tabs (`\t`) - text indentation
///
/// # Examples
///
/// ```
/// # use hedl_neo4j::cypher::unicode::sanitize_string_value;
/// // Sanitize simple value
/// let node_id = sanitize_string_value("user_123");
/// assert_eq!(node_id, "user_123");
///
/// // Remove dangerous characters but preserve whitespace
/// let value = sanitize_string_value("text\u{200B}value");
/// assert_eq!(value, "textvalue");
///
/// // Newlines are preserved
/// let multiline = sanitize_string_value("line1\nline2");
/// assert_eq!(multiline, "line1\nline2");
/// ```
#[must_use]
pub fn sanitize_string_value(s: &str) -> String {
    // Security Layer 1: Normalize Unicode to prevent homograph attacks
    let normalized = normalize_unicode(s);

    // Security Layer 2: Filter dangerous Unicode characters (but preserve whitespace)
    normalized
        .chars()
        .filter(|c| !is_dangerous_unicode_for_values(*c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_unicode_basic() {
        // ASCII strings should be unchanged
        assert_eq!(normalize_unicode("hello"), "hello");
        assert_eq!(normalize_unicode("test123"), "test123");
    }

    #[test]
    fn test_normalize_unicode_composed_vs_decomposed() {
        // NFC normalization: composed form (é as single character U+00E9)
        let composed = "café";
        let normalized = normalize_unicode(composed);

        // Verify it's in composed form (NFC)
        assert_eq!(normalized, "café");
        assert_eq!(normalized.chars().count(), 4); // c, a, f, é
    }

    #[test]
    fn test_normalize_unicode_homograph_prevention() {
        // Latin 'a' (U+0061)
        let latin_a = "name";
        // Cyrillic 'а' (U+0430) looks identical but is different
        let cyrillic_a = "nаme"; // Second character is Cyrillic а

        // They should NOT be equal (homograph attack prevention)
        // But normalization preserves their distinctness
        let norm_latin = normalize_unicode(latin_a);
        let norm_cyrillic = normalize_unicode(cyrillic_a);

        // Both are normalized, but they remain different
        assert_ne!(norm_latin, norm_cyrillic);

        // Verify Latin is still ASCII
        assert!(norm_latin.is_ascii());
        // Verify Cyrillic still contains non-ASCII
        assert!(!norm_cyrillic.is_ascii());
    }

    #[test]
    fn test_normalize_unicode_with_diacritics() {
        // Various diacritical marks
        let tests = vec![
            ("naïve", "naïve"),   // i with diaeresis
            ("résumé", "résumé"), // e with acute
            ("über", "über"),     // u with umlaut
            ("señor", "señor"),   // n with tilde
        ];

        for (input, expected) in tests {
            assert_eq!(normalize_unicode(input), expected);
        }
    }

    #[test]
    fn test_normalize_unicode_empty_string() {
        assert_eq!(normalize_unicode(""), "");
    }

    #[test]
    fn test_normalize_unicode_emoji() {
        // Emoji should be preserved
        let emoji = "test🔥data";
        let normalized = normalize_unicode(emoji);
        assert!(normalized.contains("🔥"));
    }

    #[test]
    fn test_is_dangerous_unicode_control_chars() {
        // Control characters
        assert!(is_dangerous_unicode('\x00')); // null
        assert!(is_dangerous_unicode('\n')); // newline
        assert!(is_dangerous_unicode('\r')); // carriage return
        assert!(is_dangerous_unicode('\t')); // tab

        // Safe characters
        assert!(!is_dangerous_unicode('a'));
        assert!(!is_dangerous_unicode('_'));
        assert!(!is_dangerous_unicode('1'));
    }

    #[test]
    fn test_is_dangerous_unicode_zero_width() {
        // Zero-width characters
        assert!(is_dangerous_unicode('\u{200B}')); // zero-width space
        assert!(is_dangerous_unicode('\u{200C}')); // zero-width non-joiner
        assert!(is_dangerous_unicode('\u{200D}')); // zero-width joiner
        assert!(is_dangerous_unicode('\u{FEFF}')); // zero-width no-break space
    }

    #[test]
    fn test_is_dangerous_unicode_directional() {
        // Directional formatting
        assert!(is_dangerous_unicode('\u{202A}')); // LTR embedding
        assert!(is_dangerous_unicode('\u{202B}')); // RTL embedding
        assert!(is_dangerous_unicode('\u{202E}')); // RTL override
        assert!(is_dangerous_unicode('\u{202D}')); // LTR override
    }

    #[test]
    fn test_sanitize_identifier_removes_dangerous() {
        // Zero-width space
        assert_eq!(sanitize_identifier("name\u{200B}test"), "nametest");

        // RTL override
        assert_eq!(sanitize_identifier("name\u{202E}test"), "nametest");

        // Null byte
        assert_eq!(sanitize_identifier("name\x00test"), "nametest");

        // Tab
        assert_eq!(sanitize_identifier("name\ttest"), "nametest");

        // Zero-width joiner
        assert_eq!(sanitize_identifier("name\u{200D}test"), "nametest");
    }

    #[test]
    fn test_sanitize_identifier_normalizes() {
        // Should normalize Unicode
        let result = sanitize_identifier("café");
        assert_eq!(result, "café");
        assert_eq!(result.chars().count(), 4);
    }

    #[test]
    fn test_sanitize_identifier_preserves_valid() {
        // Valid identifiers unchanged (except normalization)
        assert_eq!(sanitize_identifier("hello"), "hello");
        assert_eq!(sanitize_identifier("test_123"), "test_123");
        assert_eq!(sanitize_identifier("CamelCase"), "CamelCase");
    }

    #[test]
    fn test_sanitize_identifier_multiple_dangerous() {
        // Multiple dangerous characters
        let input = "name\u{200B}\x00\ttest\u{202E}";
        assert_eq!(sanitize_identifier(input), "nametest");
    }
}
