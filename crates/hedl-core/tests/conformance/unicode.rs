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

//! Unicode Edge Case Tests
//!
//! Comprehensive tests for Unicode handling in HEDL, covering:
//! - Right-to-Left (RTL) text (Arabic, Hebrew)
//! - Combining characters and accents
//! - Emoji and emoji modifiers
//! - Unicode normalization (NFD vs NFC)
//! - Edge cases (null chars, control chars, surrogates)
//! - String escaping and block strings
//! - BOM handling
//! - 4-byte UTF-8 sequences

use hedl_core::{parse, HedlErrorKind, Item, Value};

// =============================================================================
// 1. Right-to-Left (RTL) Text Tests
// =============================================================================

/// Test Arabic text in values
#[test]
fn test_rtl_arabic_value() {
    let doc = "%VERSION: 1.0\n---\ngreeting: مرحبا بك\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let greeting = doc.root.get("greeting").unwrap();
    if let Item::Scalar(Value::String(s)) = greeting {
        assert_eq!(s, "مرحبا بك");
    } else {
        panic!("Expected string value");
    }
}

/// Test Hebrew text in values
#[test]
fn test_rtl_hebrew_value() {
    let doc = "%VERSION: 1.0\n---\ngreeting: שלום\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let greeting = doc.root.get("greeting").unwrap();
    if let Item::Scalar(Value::String(s)) = greeting {
        assert_eq!(s, "שלום");
    } else {
        panic!("Expected string value");
    }
}

/// Test mixed RTL/LTR text in same value
#[test]
fn test_rtl_mixed_in_value() {
    let doc = "%VERSION: 1.0\n---\nmessage: Hello مرحبا World שלום\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let message = doc.root.get("message").unwrap();
    if let Item::Scalar(Value::String(s)) = message {
        assert_eq!(s, "Hello مرحبا World שלום");
    } else {
        panic!("Expected string value");
    }
}

/// Test RTL text in quoted strings
#[test]
fn test_rtl_arabic_quoted() {
    let doc = "%VERSION: 1.0\n---\ngreeting: \"مرحبا بك\"\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let greeting = doc.root.get("greeting").unwrap();
    if let Item::Scalar(Value::String(s)) = greeting {
        assert_eq!(s, "مرحبا بك");
    } else {
        panic!("Expected string value");
    }
}

/// Test RTL text in keys (should work for valid identifiers)
#[test]
fn test_rtl_in_matrix_list() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id, text]\n---\ndata: @T\n  | item1, مرحبا\n  | item2, שלום\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let data = doc.root.get("data").unwrap();
    if let Item::List(list) = data {
        assert_eq!(list.rows.len(), 2);
        if let Value::String(s) = &list.rows[0].fields[1] {
            assert_eq!(s, "مرحبا");
        }
        if let Value::String(s) = &list.rows[1].fields[1] {
            assert_eq!(s, "שלום");
        }
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// 2. Combining Characters Tests
// =============================================================================

/// Test accented characters (precomposed)
#[test]
fn test_combining_accents_precomposed() {
    let doc = "%VERSION: 1.0\n---\nname: café\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let name = doc.root.get("name").unwrap();
    if let Item::Scalar(Value::String(s)) = name {
        assert!(s.contains('é'));
    } else {
        panic!("Expected string value");
    }
}

/// Test combining diacritics
#[test]
fn test_combining_diacritics() {
    // e + combining acute accent
    let doc = "%VERSION: 1.0\n---\nname: cafe\u{0301}\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let name = doc.root.get("name").unwrap();
    if let Item::Scalar(Value::String(s)) = name {
        // Should contain the combining character sequence
        assert!(s.contains('\u{0301}'));
    } else {
        panic!("Expected string value");
    }
}

/// Test multiple combining characters
#[test]
fn test_multiple_combining_chars() {
    let doc = "%VERSION: 1.0\n---\ntext: a\u{0301}\u{0308}\n"; // a with acute and umlaut
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
}

/// Test Spanish ñ character
#[test]
fn test_combining_spanish_n() {
    let doc = "%VERSION: 1.0\n---\nword: niño\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let word = doc.root.get("word").unwrap();
    if let Item::Scalar(Value::String(s)) = word {
        assert_eq!(s, "niño");
    } else {
        panic!("Expected string value");
    }
}

/// Test German umlauts
#[test]
fn test_combining_german_umlauts() {
    let doc = "%VERSION: 1.0\n---\nword: Müller\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let word = doc.root.get("word").unwrap();
    if let Item::Scalar(Value::String(s)) = word {
        assert_eq!(s, "Müller");
    } else {
        panic!("Expected string value");
    }
}

// =============================================================================
// 3. Emoji and Modifiers Tests
// =============================================================================

/// Test basic emoji
#[test]
fn test_emoji_basic() {
    let doc = "%VERSION: 1.0\n---\nreaction: 🎉\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let reaction = doc.root.get("reaction").unwrap();
    if let Item::Scalar(Value::String(s)) = reaction {
        assert_eq!(s, "🎉");
    } else {
        panic!("Expected string value");
    }
}

/// Test emoji with skin tone modifier
#[test]
fn test_emoji_skin_tone_modifier() {
    let doc = "%VERSION: 1.0\n---\nthumb: 👍🏿\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let thumb = doc.root.get("thumb").unwrap();
    if let Item::Scalar(Value::String(s)) = thumb {
        assert_eq!(s, "👍🏿");
    } else {
        panic!("Expected string value");
    }
}

/// Test emoji with Zero Width Joiner (ZWJ) sequences
#[test]
fn test_emoji_zwj_family() {
    // Family emoji: 👨‍👩‍👧‍👦
    let doc = "%VERSION: 1.0\n---\nfamily: 👨\u{200D}👩\u{200D}👧\u{200D}👦\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let family = doc.root.get("family").unwrap();
    if let Item::Scalar(Value::String(s)) = family {
        // Should contain ZWJ characters
        assert!(s.contains('\u{200D}'));
    } else {
        panic!("Expected string value");
    }
}

/// Test multiple emoji in sequence
#[test]
fn test_emoji_sequence() {
    let doc = "%VERSION: 1.0\n---\nemojis: 🎉🎊🎈\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let emojis = doc.root.get("emojis").unwrap();
    if let Item::Scalar(Value::String(s)) = emojis {
        assert_eq!(s, "🎉🎊🎈");
    } else {
        panic!("Expected string value");
    }
}

/// Test emoji in matrix list
#[test]
fn test_emoji_in_matrix() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id, emoji]\n---\ndata: @T\n  | e1, 🎉\n  | e2, 👍🏿\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let data = doc.root.get("data").unwrap();
    if let Item::List(list) = data {
        assert_eq!(list.rows.len(), 2);
        if let Value::String(s) = &list.rows[0].fields[1] {
            assert_eq!(s, "🎉");
        }
        if let Value::String(s) = &list.rows[1].fields[1] {
            assert_eq!(s, "👍🏿");
        }
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// 4. Unicode Normalization Tests
// =============================================================================

/// Test NFC (precomposed) form - café
#[test]
fn test_normalization_nfc() {
    // NFC: single character é (U+00E9)
    let doc = "%VERSION: 1.0\n---\nword: café\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let word = doc.root.get("word").unwrap();
    if let Item::Scalar(Value::String(s)) = word {
        // Should parse correctly regardless of form
        assert!(s.contains("caf"));
    } else {
        panic!("Expected string value");
    }
}

/// Test NFD (decomposed) form - café
#[test]
fn test_normalization_nfd() {
    // NFD: e + combining acute (U+0065 + U+0301)
    let doc = "%VERSION: 1.0\n---\nword: cafe\u{0301}\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let word = doc.root.get("word").unwrap();
    if let Item::Scalar(Value::String(s)) = word {
        // Should parse correctly regardless of form
        assert!(s.starts_with("caf"));
    } else {
        panic!("Expected string value");
    }
}

/// Test that both NFC and NFD forms parse successfully
#[test]
fn test_normalization_both_forms_valid() {
    let doc_nfc = "%VERSION: 1.0\n---\nword: café\n";
    let doc_nfd = "%VERSION: 1.0\n---\nword: cafe\u{0301}\n";

    assert!(parse(doc_nfc.as_bytes()).is_ok());
    assert!(parse(doc_nfd.as_bytes()).is_ok());
}

/// Test mixed normalization forms in same document
#[test]
fn test_normalization_mixed_forms() {
    let doc = "%VERSION: 1.0\n---\nnfc: café\nnfd: cafe\u{0301}\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    assert!(doc.root.contains_key("nfc"));
    assert!(doc.root.contains_key("nfd"));
}

// =============================================================================
// 5. Edge Cases and Error Handling
// =============================================================================

/// Test null character should be rejected
#[test]
fn test_null_character_rejected() {
    let doc = "%VERSION: 1.0\n---\ndata: test\0value\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// Test control character (SOH) should be rejected
#[test]
fn test_control_character_soh_rejected() {
    let doc = "%VERSION: 1.0\n---\ndata: test\x01value\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// Test control character (STX) should be rejected
#[test]
fn test_control_character_stx_rejected() {
    let doc = "%VERSION: 1.0\n---\ndata: test\x02value\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// Test vertical tab should be rejected
#[test]
fn test_control_character_vtab_rejected() {
    let doc = "%VERSION: 1.0\n---\ndata: test\x0Bvalue\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

/// Test form feed should be rejected
#[test]
fn test_control_character_ff_rejected() {
    let doc = "%VERSION: 1.0\n---\ndata: test\x0Cvalue\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, HedlErrorKind::Syntax));
}

// =============================================================================
// 6. 4-Byte UTF-8 Sequences
// =============================================================================

/// Test rare CJK character (4-byte UTF-8)
#[test]
fn test_4byte_utf8_cjk() {
    // U+20000 (4-byte UTF-8: F0 A0 80 80)
    let doc = "%VERSION: 1.0\n---\nchar: 𠀀\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let char_val = doc.root.get("char").unwrap();
    if let Item::Scalar(Value::String(s)) = char_val {
        assert_eq!(s, "𠀀");
    } else {
        panic!("Expected string value");
    }
}

/// Test emoji (4-byte UTF-8)
#[test]
fn test_4byte_utf8_emoji() {
    // U+1F600 GRINNING FACE (4-byte UTF-8)
    let doc = "%VERSION: 1.0\n---\nface: 😀\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let face = doc.root.get("face").unwrap();
    if let Item::Scalar(Value::String(s)) = face {
        assert_eq!(s, "😀");
    } else {
        panic!("Expected string value");
    }
}

/// Test musical symbols (4-byte UTF-8)
#[test]
fn test_4byte_utf8_musical() {
    // U+1D11E MUSICAL SYMBOL G CLEF (4-byte UTF-8)
    let doc = "%VERSION: 1.0\n---\nmusic: 𝄞\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let music = doc.root.get("music").unwrap();
    if let Item::Scalar(Value::String(s)) = music {
        assert_eq!(s, "𝄞");
    } else {
        panic!("Expected string value");
    }
}

/// Test multiple 4-byte sequences
#[test]
fn test_4byte_utf8_multiple() {
    let doc = "%VERSION: 1.0\n---\ndata: 𠀀𝄞😀\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let data = doc.root.get("data").unwrap();
    if let Item::Scalar(Value::String(s)) = data {
        assert_eq!(s, "𠀀𝄞😀");
    } else {
        panic!("Expected string value");
    }
}

// =============================================================================
// 7. BOM Handling
// =============================================================================

/// Test BOM at start of file is skipped
#[test]
fn test_bom_skipped() {
    let doc = "\u{FEFF}%VERSION: 1.0\n---\ndata: test\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let data = doc.root.get("data").unwrap();
    if let Item::Scalar(Value::String(s)) = data {
        assert_eq!(s, "test");
    } else {
        panic!("Expected string value");
    }
}

/// Test BOM in middle of file is preserved
#[test]
fn test_bom_in_value() {
    let doc = "%VERSION: 1.0\n---\ndata: test\u{FEFF}value\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let data = doc.root.get("data").unwrap();
    if let Item::Scalar(Value::String(s)) = data {
        // BOM in value should be preserved
        assert!(s.contains('\u{FEFF}'));
    } else {
        panic!("Expected string value");
    }
}

// =============================================================================
// 8. String Escaping and Unicode
// =============================================================================

/// Test Unicode in quoted strings
#[test]
fn test_unicode_in_quoted_string() {
    let doc = "%VERSION: 1.0\n---\ntext: \"Hello 世界 🌍\"\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let text = doc.root.get("text").unwrap();
    if let Item::Scalar(Value::String(s)) = text {
        assert_eq!(s, "Hello 世界 🌍");
    } else {
        panic!("Expected string value");
    }
}

/// Test Unicode in block strings
#[test]
fn test_unicode_in_block_string() {
    let doc = "%VERSION: 1.0\n---\ntext: \"\"\"\nHello مرحبا\nWorld 世界\n🌍 🎉\n\"\"\"\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let text = doc.root.get("text").unwrap();
    if let Item::Scalar(Value::String(s)) = text {
        assert!(s.contains("مرحبا"));
        assert!(s.contains("世界"));
        assert!(s.contains("🌍"));
        assert!(s.contains("🎉"));
    } else {
        panic!("Expected string value");
    }
}

/// Test RTL in block strings
#[test]
fn test_rtl_in_block_string() {
    let doc = "%VERSION: 1.0\n---\ngreeting: \"\"\"\nمرحبا بك\nשלום\n\"\"\"\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let greeting = doc.root.get("greeting").unwrap();
    if let Item::Scalar(Value::String(s)) = greeting {
        assert!(s.contains("مرحبا بك"));
        assert!(s.contains("שלום"));
    } else {
        panic!("Expected string value");
    }
}

// =============================================================================
// 9. CJK Characters
// =============================================================================

/// Test Chinese characters
#[test]
fn test_cjk_chinese() {
    let doc = "%VERSION: 1.0\n---\ngreeting: 你好世界\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let greeting = doc.root.get("greeting").unwrap();
    if let Item::Scalar(Value::String(s)) = greeting {
        assert_eq!(s, "你好世界");
    } else {
        panic!("Expected string value");
    }
}

/// Test Japanese characters (Hiragana, Katakana, Kanji)
#[test]
fn test_cjk_japanese() {
    let doc = "%VERSION: 1.0\n---\ntext: こんにちは カタカナ 漢字\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let text = doc.root.get("text").unwrap();
    if let Item::Scalar(Value::String(s)) = text {
        assert_eq!(s, "こんにちは カタカナ 漢字");
    } else {
        panic!("Expected string value");
    }
}

/// Test Korean characters
#[test]
fn test_cjk_korean() {
    let doc = "%VERSION: 1.0\n---\ngreeting: 안녕하세요\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let greeting = doc.root.get("greeting").unwrap();
    if let Item::Scalar(Value::String(s)) = greeting {
        assert_eq!(s, "안녕하세요");
    } else {
        panic!("Expected string value");
    }
}

/// Test mixed CJK in matrix list
#[test]
fn test_cjk_in_matrix() {
    let doc = "%VERSION: 1.0\n%STRUCT: T: [id, text]\n---\ndata: @T\n  | t1, 你好\n  | t2, こんにちは\n  | t3, 안녕하세요\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let data = doc.root.get("data").unwrap();
    if let Item::List(list) = data {
        assert_eq!(list.rows.len(), 3);
        if let Value::String(s) = &list.rows[0].fields[1] {
            assert_eq!(s, "你好");
        }
        if let Value::String(s) = &list.rows[1].fields[1] {
            assert_eq!(s, "こんにちは");
        }
        if let Value::String(s) = &list.rows[2].fields[1] {
            assert_eq!(s, "안녕하세요");
        }
    } else {
        panic!("Expected list");
    }
}

// =============================================================================
// 10. Zero-Width Characters
// =============================================================================

/// Test Zero Width Joiner (ZWJ)
#[test]
fn test_zero_width_joiner() {
    let doc = "%VERSION: 1.0\n---\ntext: a\u{200D}b\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let text = doc.root.get("text").unwrap();
    if let Item::Scalar(Value::String(s)) = text {
        assert!(s.contains('\u{200D}'));
    } else {
        panic!("Expected string value");
    }
}

/// Test Zero Width Non-Joiner (ZWNJ)
#[test]
fn test_zero_width_non_joiner() {
    let doc = "%VERSION: 1.0\n---\ntext: a\u{200C}b\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let text = doc.root.get("text").unwrap();
    if let Item::Scalar(Value::String(s)) = text {
        assert!(s.contains('\u{200C}'));
    } else {
        panic!("Expected string value");
    }
}

/// Test Zero Width Space
#[test]
fn test_zero_width_space() {
    let doc = "%VERSION: 1.0\n---\ntext: a\u{200B}b\n";
    let result = parse(doc.as_bytes());
    assert!(result.is_ok());
    let doc = result.unwrap();

    let text = doc.root.get("text").unwrap();
    if let Item::Scalar(Value::String(s)) = text {
        assert!(s.contains('\u{200B}'));
    } else {
        panic!("Expected string value");
    }
}
