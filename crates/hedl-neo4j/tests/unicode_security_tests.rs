// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive Unicode security tests.

use hedl_neo4j::cypher::unicode::{normalize_unicode, sanitize_identifier, sanitize_string_value};

#[test]
fn test_sanitize_string_value_preserves_whitespace() {
    // Newlines should be preserved
    let result = sanitize_string_value("line1\nline2\nline3");
    assert_eq!(result, "line1\nline2\nline3");

    // Carriage returns should be preserved
    let result = sanitize_string_value("line1\r\nline2");
    assert_eq!(result, "line1\r\nline2");

    // Tabs should be preserved
    let result = sanitize_string_value("col1\tcol2\tcol3");
    assert_eq!(result, "col1\tcol2\tcol3");
}

#[test]
fn test_sanitize_string_value_removes_dangerous_unicode() {
    // Zero-width space
    let result = sanitize_string_value("text\u{200B}value");
    assert_eq!(result, "textvalue");

    // Zero-width non-joiner
    let result = sanitize_string_value("word\u{200C}boundary");
    assert_eq!(result, "wordboundary");

    // Zero-width joiner
    let result = sanitize_string_value("emoji\u{200D}sequence");
    assert_eq!(result, "emojisequence");

    // Zero-width no-break space (BOM)
    let result = sanitize_string_value("start\u{FEFF}middle");
    assert_eq!(result, "startmiddle");
}

#[test]
fn test_sanitize_string_value_removes_directional_formatting() {
    // Left-to-right embedding
    let result = sanitize_string_value("text\u{202A}embedded");
    assert_eq!(result, "textembedded");

    // Right-to-left embedding
    let result = sanitize_string_value("text\u{202B}rtl");
    assert_eq!(result, "textrtl");

    // Pop directional formatting
    let result = sanitize_string_value("text\u{202C}pop");
    assert_eq!(result, "textpop");

    // Left-to-right override
    let result = sanitize_string_value("text\u{202D}ltr");
    assert_eq!(result, "textltr");

    // Right-to-left override
    let result = sanitize_string_value("text\u{202E}rto");
    assert_eq!(result, "textrto");

    // Isolates
    let result = sanitize_string_value("text\u{2066}\u{2067}\u{2068}\u{2069}isolate");
    assert_eq!(result, "textisolate");
}

#[test]
fn test_sanitize_string_value_removes_other_dangerous_chars() {
    // Soft hyphen
    let result = sanitize_string_value("word\u{00AD}break");
    assert_eq!(result, "wordbreak");

    // Arabic letter mark
    let result = sanitize_string_value("text\u{061C}mark");
    assert_eq!(result, "textmark");

    // Mongolian vowel separator
    let result = sanitize_string_value("word\u{180E}separator");
    assert_eq!(result, "wordseparator");
}

#[test]
fn test_sanitize_string_value_removes_null_bytes() {
    let result = sanitize_string_value("before\x00after");
    assert_eq!(result, "beforeafter");
}

#[test]
fn test_sanitize_string_value_removes_other_control_chars() {
    // But NOT newlines, carriage returns, or tabs
    let result = sanitize_string_value("text\x01\x02\x03control");
    assert_eq!(result, "textcontrol");

    // Bell character
    let result = sanitize_string_value("text\x07bell");
    assert_eq!(result, "textbell");

    // Escape character
    let result = sanitize_string_value("text\x1Bescape");
    assert_eq!(result, "textescape");
}

#[test]
fn test_sanitize_string_value_normalizes_unicode() {
    // Composed vs decomposed should normalize to same
    let composed = sanitize_string_value("café");
    let decomposed = sanitize_string_value("cafe\u{0301}");
    assert_eq!(composed, decomposed);
    assert_eq!(composed, "café");
}

#[test]
fn test_sanitize_string_value_preserves_emoji() {
    let result = sanitize_string_value("Test 🔥 emoji 👍 text");
    assert_eq!(result, "Test 🔥 emoji 👍 text");
}

#[test]
fn test_sanitize_string_value_preserves_cjk() {
    let result = sanitize_string_value("中文 日本語 한국어");
    assert_eq!(result, "中文 日本語 한국어");
}

#[test]
fn test_sanitize_string_value_preserves_symbols() {
    let result = sanitize_string_value("Math: ∀∃∈∉ Arrows: ←→↑↓");
    assert_eq!(result, "Math: ∀∃∈∉ Arrows: ←→↑↓");
}

#[test]
fn test_sanitize_string_value_empty() {
    assert_eq!(sanitize_string_value(""), "");
}

#[test]
fn test_sanitize_string_value_only_dangerous_chars() {
    let result = sanitize_string_value("\u{200B}\u{200C}\u{202E}");
    assert_eq!(result, "");
}

#[test]
fn test_sanitize_string_value_mixed_content() {
    let result = sanitize_string_value("Normal\u{200B}text\nwith\ttabs\u{202E}and\rdangerous");
    assert_eq!(result, "Normaltext\nwith\ttabsand\rdangerous");
}

#[test]
fn test_sanitize_identifier_vs_sanitize_string_value_whitespace() {
    // sanitize_identifier removes ALL control chars including newlines
    let id_result = sanitize_identifier("text\nwith\nnewlines");
    assert_eq!(id_result, "textwithnewlines");

    // sanitize_string_value preserves newlines/tabs/CR
    let val_result = sanitize_string_value("text\nwith\nnewlines");
    assert_eq!(val_result, "text\nwith\nnewlines");
}

#[test]
fn test_normalize_unicode_ascii_unchanged() {
    assert_eq!(normalize_unicode("hello"), "hello");
    assert_eq!(normalize_unicode("test_123"), "test_123");
    assert_eq!(normalize_unicode("UPPER"), "UPPER");
}

#[test]
fn test_normalize_unicode_combining_marks() {
    // Combining acute accent (U+0301)
    let decomposed = "e\u{0301}"; // e + combining acute
    let composed = "é"; // precomposed
    assert_eq!(normalize_unicode(decomposed), normalize_unicode(composed));
}

#[test]
fn test_normalize_unicode_ligatures() {
    // Some characters that might have decomposed forms
    let input = "ﬁ"; // fi ligature (U+FB01)
    let normalized = normalize_unicode(input);
    assert!(!normalized.is_empty());
}

#[test]
fn test_normalize_unicode_hangul() {
    // Korean Hangul can be composed or decomposed
    let hangul = "한글";
    let normalized = normalize_unicode(hangul);
    assert_eq!(normalized, "한글");
}

#[test]
fn test_normalize_unicode_preserves_length_or_shorter() {
    // Normalization should not make strings longer (for most cases)
    let inputs = vec!["test", "café", "naïve", "über", "señor", "中文", "😀"];

    for input in inputs {
        let normalized = normalize_unicode(input);
        // Normalized form should exist
        assert!(!normalized.is_empty());
        // For NFC, the character count is usually same or less
        assert!(normalized.chars().count() <= input.chars().count() + 1);
    }
}

#[test]
fn test_homograph_attack_detection() {
    // Latin 'a' vs Cyrillic 'а'
    let latin = "name";
    let cyrillic = "nаme"; // Second character is Cyrillic а (U+0430)

    // They should remain different after normalization
    assert_ne!(normalize_unicode(latin), normalize_unicode(cyrillic));

    // Latin should still be ASCII
    assert!(normalize_unicode(latin).is_ascii());

    // Cyrillic should still contain non-ASCII
    assert!(!normalize_unicode(cyrillic).is_ascii());
}

#[test]
fn test_sanitize_identifier_comprehensive() {
    // Test removing all dangerous characters
    let dangerous = "name\x00\n\r\t\u{200B}\u{200C}\u{200D}\u{202E}\u{202D}test";
    let result = sanitize_identifier(dangerous);
    assert_eq!(result, "nametest");

    // No dangerous characters should remain
    assert!(!result.contains('\x00'));
    assert!(!result.contains('\n'));
    assert!(!result.contains('\r'));
    assert!(!result.contains('\t'));
    assert!(!result.contains('\u{200B}'));
    assert!(!result.contains('\u{202E}'));
}

#[test]
fn test_sanitize_string_value_comprehensive() {
    // Test removing dangerous characters but preserving safe whitespace
    let input = "text\x00with\u{200B}dangerous\u{202E}but\nwith\twhitespace\r";
    let result = sanitize_string_value(input);
    assert_eq!(result, "textwithdangerousbut\nwith\twhitespace\r");

    // Dangerous characters removed
    assert!(!result.contains('\x00'));
    assert!(!result.contains('\u{200B}'));
    assert!(!result.contains('\u{202E}'));

    // Safe whitespace preserved
    assert!(result.contains('\n'));
    assert!(result.contains('\t'));
    assert!(result.contains('\r'));
}

#[test]
fn test_unicode_normalization_idempotent() {
    let inputs = vec!["café", "naïve", "über"];

    for input in inputs {
        let once = normalize_unicode(input);
        let twice = normalize_unicode(&once);
        assert_eq!(once, twice, "Normalization should be idempotent");
    }
}

#[test]
fn test_sanitization_idempotent() {
    let input = "test\u{200B}value\nwith\ttabs";

    let id_once = sanitize_identifier(input);
    let id_twice = sanitize_identifier(&id_once);
    assert_eq!(id_once, id_twice);

    let val_once = sanitize_string_value(input);
    let val_twice = sanitize_string_value(&val_once);
    assert_eq!(val_once, val_twice);
}

#[test]
fn test_real_world_attack_vectors() {
    // Common attack patterns

    // 1. Zero-width space injection in identifiers
    let attack1 = "admin\u{200B}";
    let sanitized1 = sanitize_identifier(attack1);
    assert_eq!(sanitized1, "admin");

    // 2. RTL override for visual spoofing
    let attack2 = "user\u{202E}nimda"; // Appears as "useradmin" visually
    let sanitized2 = sanitize_identifier(attack2);
    assert_eq!(sanitized2, "usernimda");

    // 3. Null byte injection
    let attack3 = "admin\x00injected";
    let sanitized3 = sanitize_identifier(attack3);
    assert_eq!(sanitized3, "admininjected");

    // 4. Combining multiple attack vectors
    let attack4 = "user\u{200B}\x00\u{202E}test";
    let sanitized4 = sanitize_identifier(attack4);
    assert_eq!(sanitized4, "usertest");
}

#[test]
fn test_preserve_legitimate_unicode() {
    // These should all be preserved
    let legitimate = vec![
        ("José", "José"),
        ("François", "François"),
        ("Björk", "Björk"),
        ("Müller", "Müller"),
        ("Владимир", "Владимир"),
        ("李明", "李明"),
        ("محمد", "محمد"),
    ];

    for (input, expected) in legitimate {
        let result = sanitize_identifier(input);
        // Should normalize but preserve the characters
        assert_eq!(
            normalize_unicode(result.as_str()),
            normalize_unicode(expected)
        );
    }
}
