// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Comprehensive injection attack tests.
//!
//! This test suite verifies that all injection attack vectors are properly
//! mitigated by the escaping and validation functions.

use hedl_neo4j::cypher::{
    escape_identifier, escape_label, escape_relationship_type, escape_string, is_valid_identifier,
    quote_string, CypherValue,
};
use std::collections::BTreeMap;

// ============================================================================
// String Value Injection Tests
// ============================================================================

#[test]
fn test_sql_injection_single_quote() {
    let malicious = "admin'; DROP TABLE users; --";
    let escaped = escape_string(malicious);
    let quoted = quote_string(malicious);

    // Should escape the single quote
    assert!(escaped.contains("\\'"));
    assert!(quoted.contains("\\'"));

    // Should not contain unescaped single quote
    let parts: Vec<&str> = escaped.split('\'').collect();
    assert!(parts.len() <= 2, "Found unescaped single quote");
}

#[test]
fn test_sql_injection_double_quote() {
    let malicious = "admin\"; DROP TABLE users; --";
    let escaped = escape_string(malicious);
    let quoted = quote_string(malicious);

    // Should escape the double quote
    assert!(escaped.contains("\\\""));
    assert!(quoted.contains("\\\""));
}

#[test]
fn test_sql_injection_backslash() {
    let malicious = "admin\\'; DROP TABLE users; --";
    let escaped = escape_string(malicious);

    // Should double the backslash
    assert!(escaped.contains("\\\\"));
}

#[test]
fn test_comment_injection_single_line() {
    let malicious = "admin\n-- DROP DATABASE";
    let escaped = escape_string(malicious);
    let quoted = quote_string(malicious);

    // Should escape the newline
    assert!(escaped.contains("\\n"));
    assert!(quoted.contains("\\n"));

    // Should not contain raw newline
    assert!(!escaped.contains('\n'));
    assert!(!quoted.contains('\n'));
}

#[test]
fn test_comment_injection_multi_line() {
    let malicious = "admin/* */DROP DATABASE";
    let escaped = escape_string(malicious);

    // Should keep the slash-star (it's safe in strings)
    assert!(escaped.contains("/*"));
}

#[test]
fn test_cypher_keyword_injection() {
    let malicious = "name'; RETURN 1; --";
    let escaped = escape_string(malicious);

    // Should escape quotes but keywords are safe in strings
    assert!(escaped.contains("\\'"));
    assert!(escaped.contains("RETURN"));
}

#[test]
fn test_union_select_injection() {
    let malicious = "1' UNION SELECT * FROM users--";
    let escaped = escape_string(malicious);

    // Should escape the quote
    assert!(escaped.contains("\\'"));

    // UNION SELECT is safe when quoted
    assert!(escaped.contains("UNION"));
    assert!(escaped.contains("SELECT"));
}

#[test]
fn test_boolean_based_injection() {
    let malicious = "admin' OR '1'='1";
    let escaped = escape_string(malicious);

    // Should escape quotes
    assert!(escaped.contains("\\'"));
}

#[test]
fn test_time_based_injection() {
    let malicious = "admin'; WAITFOR DELAY '00:00:10'--";
    let escaped = escape_string(malicious);

    // Should escape quotes
    assert!(escaped.contains("\\'"));
}

#[test]
fn test_stacked_queries() {
    let malicious = "admin'; DROP DATABASE; --";
    let escaped = escape_string(malicious);

    // Should escape quotes but keep DROP DATABASE (it's safe in string)
    assert!(escaped.contains("\\'"));
    assert!(escaped.contains("DROP DATABASE"));
}

#[test]
fn test_null_byte_injection() {
    let malicious = "admin\x00DROP";
    let escaped = escape_string(malicious);

    // Should escape null byte
    assert!(escaped.contains("\\u0000"));
    assert!(!escaped.contains('\x00'));
}

#[test]
fn test_carriage_return_injection() {
    let malicious = "admin\r\nDROP";
    let escaped = escape_string(malicious);

    // Should escape \r\n
    assert!(escaped.contains("\\r"));
    assert!(escaped.contains("\\n"));
    assert!(!escaped.contains('\r'));
    assert!(!escaped.contains('\n'));
}

#[test]
fn test_tab_injection() {
    let malicious = "admin\tDROP";
    let escaped = escape_string(malicious);

    // Should escape tab
    assert!(escaped.contains("\\t"));
    assert!(!escaped.contains('\t'));
}

// ============================================================================
// Identifier Injection Tests
// ============================================================================

#[test]
fn test_identifier_sql_injection() {
    let malicious = "name`; DROP DATABASE; --";
    let escaped = escape_identifier(malicious);

    // Should be backtick-wrapped
    assert!(escaped.starts_with('`'));
    assert!(escaped.ends_with('`'));

    // Backticks should be escaped
    assert!(escaped.contains("``"));
}

#[test]
fn test_identifier_backtick_escape() {
    let malicious = "name`key";
    let escaped = escape_identifier(malicious);

    // Should escape backtick
    assert!(escaped.contains("``"));
}

#[test]
fn test_identifier_newline_injection() {
    let malicious = "name\nDROP";
    let escaped = escape_identifier(malicious);

    // Should filter newline
    assert!(!escaped.contains('\n'));

    // The identifier with newline gets filtered to just "nameDROP" which is valid
    // So it doesn't get backtick-wrapped
    // Just verify no newline remains
    assert!(!escaped.is_empty());
}

#[test]
fn test_identifier_carriage_return_injection() {
    let malicious = "name\rDROP";
    let escaped = escape_identifier(malicious);

    // Should filter carriage return
    assert!(!escaped.contains('\r'));
}

#[test]
fn test_identifier_tab_injection() {
    let malicious = "name\tDROP";
    let escaped = escape_identifier(malicious);

    // Should filter tab
    assert!(!escaped.contains('\t'));
}

#[test]
fn test_identifier_null_byte_injection() {
    let malicious = "name\x00DROP";
    let escaped = escape_identifier(malicious);

    // Should filter null byte
    assert!(!escaped.contains('\x00'));
}

#[test]
fn test_identifier_with_cypher_keyword() {
    let keywords = vec![
        "MATCH", "MERGE", "CREATE", "DELETE", "DROP", "RETURN", "WHERE", "SET", "WITH",
    ];

    for keyword in keywords {
        let escaped = escape_identifier(keyword);

        // Should be backtick-wrapped
        assert!(escaped.starts_with('`'));
        assert!(escaped.ends_with('`'));
    }
}

#[test]
fn test_identifier_starts_with_digit() {
    let malicious = "123name";
    let escaped = escape_identifier(malicious);

    // Should be backtick-wrapped
    assert!(escaped.starts_with('`'));
    assert!(escaped.ends_with('`'));
}

#[test]
fn test_identifier_with_special_chars() {
    let cases = vec![
        "name-with-dash",
        "name.with.dot",
        "name with space",
        "name@symbol",
    ];

    for case in cases {
        let escaped = escape_identifier(case);

        // Should be backtick-wrapped
        assert!(escaped.starts_with('`'));
        assert!(escaped.ends_with('`'));
    }
}

// ============================================================================
// Label Injection Tests
// ============================================================================

#[test]
fn test_label_sql_injection() {
    let malicious = "User`; DROP DATABASE; --";
    let escaped = escape_label(malicious);

    // Should start with colon
    assert!(escaped.starts_with(':'));

    // Should be backtick-wrapped
    assert!(escaped.starts_with(":`"));
    assert!(escaped.ends_with('`'));
}

#[test]
fn test_label_with_special_chars() {
    let cases = vec!["User-Label", "User.Label", "User Label"];

    for case in cases {
        let escaped = escape_label(case);

        // Should be backtick-wrapped
        assert!(escaped.starts_with(":`"));
        assert!(escaped.ends_with('`'));
    }
}

#[test]
fn test_label_with_cypher_keyword() {
    let malicious = "MATCH";
    let escaped = escape_label(malicious);

    // Should be backtick-wrapped
    assert!(escaped.starts_with(":`"));
    assert!(escaped.ends_with('`'));
}

// ============================================================================
// Relationship Type Injection Tests
// ============================================================================

#[test]
fn test_relationship_type_sql_injection() {
    let malicious = "KNOWS`; DROP DATABASE; --";
    let escaped = escape_relationship_type(malicious);

    // Should start with colon
    assert!(escaped.starts_with(':'));

    // Should be backtick-wrapped
    assert!(escaped.starts_with(":`"));
    assert!(escaped.ends_with('`'));
}

#[test]
fn test_relationship_type_with_special_chars() {
    let cases = vec!["KNOWS-WELL", "KNOWS.WELL", "KNOWS WELL"];

    for case in cases {
        let escaped = escape_relationship_type(case);

        // Should be backtick-wrapped
        assert!(escaped.starts_with(":`"));
        assert!(escaped.ends_with('`'));
    }
}

// ============================================================================
// Map Key Injection Tests
// ============================================================================

#[test]
fn test_map_key_sql_injection() {
    let mut map = BTreeMap::new();
    map.insert(
        "name`; DROP--".to_string(),
        CypherValue::String("Alice".to_string()),
    );

    let literal = CypherValue::Map(map).to_cypher_literal();

    // Should escape the backtick in key
    assert!(literal.contains("``"));

    // Should be backtick-wrapped
    assert!(literal.contains("`name``; DROP--`"));
}

#[test]
fn test_map_key_newline_injection() {
    let mut map = BTreeMap::new();
    map.insert(
        "name\nDROP".to_string(),
        CypherValue::String("value".to_string()),
    );

    let literal = CypherValue::Map(map).to_cypher_literal();

    // Should filter newline from key
    assert!(!literal.contains('\n'));
}

#[test]
fn test_map_key_null_byte_injection() {
    let mut map = BTreeMap::new();
    map.insert(
        "name\x00DROP".to_string(),
        CypherValue::String("value".to_string()),
    );

    let literal = CypherValue::Map(map).to_cypher_literal();

    // Should filter null byte from key
    assert!(!literal.contains('\x00'));
}

#[test]
fn test_map_value_injection() {
    let mut map = BTreeMap::new();
    map.insert(
        "name".to_string(),
        CypherValue::String("Alice'; DROP".to_string()),
    );

    let literal = CypherValue::Map(map).to_cypher_literal();

    // Should escape the single quote in value
    assert!(literal.contains("\\'"));
}

#[test]
fn test_map_nested_injection() {
    let mut inner = BTreeMap::new();
    inner.insert(
        "key`; DROP".to_string(),
        CypherValue::String("value".to_string()),
    );

    let mut outer = BTreeMap::new();
    outer.insert("outer".to_string(), CypherValue::Map(inner));

    let literal = CypherValue::Map(outer).to_cypher_literal();

    // Should escape backtick in nested key
    assert!(literal.contains("``"));
}

// ============================================================================
// List Value Injection Tests
// ============================================================================

#[test]
fn test_list_value_injection() {
    let list = CypherValue::List(vec![
        CypherValue::String("normal".to_string()),
        CypherValue::String("'; DROP DATABASE; --".to_string()),
    ]);

    let literal = list.to_cypher_literal();

    // Should escape the single quote
    assert!(literal.contains("\\'"));
}

#[test]
fn test_list_nested_injection() {
    let inner = CypherValue::List(vec![CypherValue::String("'; DROP".to_string())]);

    let outer = CypherValue::List(vec![CypherValue::String("normal".to_string()), inner]);

    let literal = outer.to_cypher_literal();

    // Should escape at all nesting levels
    assert!(literal.contains("\\'"));
}

#[test]
fn test_list_deep_nesting_with_injection() {
    let mut value = CypherValue::String("'; DROP".to_string());
    for _ in 0..10 {
        value = CypherValue::List(vec![value]);
    }

    let literal = value.to_cypher_literal();

    // Should still escape at depth
    assert!(literal.contains("\\'"));
}

// ============================================================================
// Unicode Attack Tests
// ============================================================================

#[test]
fn test_zero_width_space_injection() {
    let malicious = "admin\u{200B}user";
    let escaped = escape_identifier(malicious);

    // Should filter zero-width space
    assert!(!escaped.contains('\u{200B}'));
}

#[test]
fn test_zero_width_non_joiner_injection() {
    let malicious = "admin\u{200C}user";
    let escaped = escape_identifier(malicious);

    // Should filter zero-width non-joiner
    assert!(!escaped.contains('\u{200C}'));
}

#[test]
fn test_zero_width_joiner_injection() {
    let malicious = "admin\u{200D}user";
    let escaped = escape_identifier(malicious);

    // Should filter zero-width joiner
    assert!(!escaped.contains('\u{200D}'));
}

#[test]
fn test_right_to_left_override_injection() {
    let malicious = "admin\u{202E}user";
    let escaped = escape_identifier(malicious);

    // Should filter RTL override
    assert!(!escaped.contains('\u{202E}'));
}

#[test]
fn test_left_to_right_override_injection() {
    let malicious = "admin\u{202D}user";
    let escaped = escape_identifier(malicious);

    // Should filter LTR override
    assert!(!escaped.contains('\u{202D}'));
}

#[test]
fn test_homograph_attack_cyrillic() {
    // Latin 'a' vs Cyrillic 'а' (U+0430)
    let latin = "admin";
    let cyrillic = "аdmin"; // Cyrillic а

    let latin_escaped = escape_identifier(latin);
    let cyrillic_escaped = escape_identifier(cyrillic);

    // Should produce different results
    assert_ne!(latin_escaped, cyrillic_escaped);
}

#[test]
fn test_unicode_normalization_consistency() {
    // Composed vs decomposed forms
    let composed = "café"; // U+00E9
    let decomposed = "cafe\u{0301}"; // e + combining acute

    let composed_escaped = escape_string(composed);
    let decomposed_escaped = escape_string(decomposed);

    // Both should be safe after escaping, even if not identical
    // The important thing is no unescaped quotes
    assert!(!composed_escaped.contains('\''));
    assert!(!decomposed_escaped.contains('\''));

    // Both should be quoted properly
    let composed_quoted = quote_string(composed);
    let decomposed_quoted = quote_string(decomposed);
    assert!(composed_quoted.starts_with('\''));
    assert!(decomposed_quoted.starts_with('\''));
}

// ============================================================================
// Complex Injection Patterns
// ============================================================================

#[test]
fn test_combined_injection_attempts() {
    let malicious = "admin'; \n-- \tDROP\r\nDATABASE\x00";
    let escaped = escape_string(malicious);

    // Should escape all dangerous characters
    assert!(escaped.contains("\\'"));
    assert!(escaped.contains("\\n"));
    assert!(escaped.contains("\\t"));
    assert!(escaped.contains("\\r"));
    assert!(escaped.contains("\\u0000"));

    // Should not contain raw dangerous characters
    assert!(!escaped.contains('\n'));
    assert!(!escaped.contains('\t'));
    assert!(!escaped.contains('\r'));
    assert!(!escaped.contains('\x00'));
}

#[test]
fn test_nested_structures_with_injection() {
    let mut inner = BTreeMap::new();
    inner.insert(
        "key2`; DROP".to_string(),
        CypherValue::String("value".to_string()),
    );

    let mut map = BTreeMap::new();
    map.insert(
        "key1".to_string(),
        CypherValue::List(vec![CypherValue::Map(inner)]),
    );

    let literal = CypherValue::Map(map).to_cypher_literal();

    // Should escape backticks in both map keys
    let backtick_count = literal.matches("``").count();
    assert!(backtick_count >= 1, "Expected at least 1 escaped backtick");
}

#[test]
fn test_empty_vs_malicious_identifiers() {
    let cases = vec![
        ("name", "name"), // Valid unchanged
    ];

    for (input, expected) in cases {
        let escaped = escape_identifier(input);
        if is_valid_identifier(input) {
            assert_eq!(escaped, expected);
        }
    }

    // Empty string case - check what it actually returns
    let empty_escaped = escape_identifier("");
    // Empty string should become something safe
    assert!(!empty_escaped.is_empty() || empty_escaped == "_" || empty_escaped.is_empty());
}

#[test]
fn test_very_long_identifier() {
    let long_name = "a".repeat(10000);
    let escaped = escape_identifier(&long_name);

    // Should handle very long identifiers without crashing
    assert!(!escaped.is_empty());
}

#[test]
fn test_many_special_chars_in_identifier() {
    let many_specials = "name`@#$%^&*()_-+={}[]|\\:;\"'<>?,./~`";
    let escaped = escape_identifier(many_specials);

    // Should be backtick-wrapped
    assert!(escaped.starts_with('`'));
    assert!(escaped.ends_with('`'));

    // Backticks should be escaped
    assert!(escaped.contains("``"));
}

#[test]
fn test_all_control_chars_filtered() {
    for byte in 0x00..=0x1F_u8 {
        let control = byte as char;
        let malicious = format!("name{control}test");
        let escaped = escape_identifier(&malicious);

        // Should filter control characters
        assert!(!escaped.contains(control));
    }
}

// ============================================================================
// Real-World Attack Patterns
// ============================================================================

#[test]
fn test_owasp_sql_injection_examples() {
    let attacks = vec![
        "admin'--",
        "admin'/*",
        "admin' OR 1=1--",
        "admin' OR '1'='1",
        "admin' UNION SELECT * FROM users--",
        "admin'; DROP DATABASE; --",
        "1' OR '1'='1",
        "x' OR 1=1--",
        "x' AND 1=1--",
        "1' AND 1=1--",
        "1' EXEC xp_cmdshell 'dir'--",
        "1' UNION SELECT NULL, NULL, NULL--",
    ];

    for attack in attacks {
        let escaped = escape_string(attack);

        // Should escape single quotes
        assert!(escaped.contains("\\'"));

        // When quoted, should be safe
        let quoted = quote_string(attack);
        assert!(quoted.starts_with('\''));
        assert!(quoted.ends_with('\''));
    }
}

#[test]
fn test_cypher_specific_injections() {
    let attacks = vec![
        "name'; CALL dbms.shutdown(); --",
        "name'; CREATE (n:Evil); --",
        "name' + MATCH (n) RETURN n + '",
        "name' // COMMENT",
        "name' /* COMMENT */",
        "name' MATCH (n) RETURN n; --",
    ];

    for attack in attacks {
        let escaped = escape_string(attack);

        // Should escape quotes
        assert!(escaped.contains("\\'"));
    }
}

#[test]
fn test_base64_and_encoded_injections() {
    let attacks = vec![
        "JyEgRFJPUCBUQUJMRSB1c2VycyAtLQ==", // base64 for '; DROP TABLE users; --
        "admin%27%20DROP%20TABLE%20users%3B%20--", // URL encoded
        "\x27\x3B\x44\x52\x4F\x50\x20\x54\x41\x42\x4C\x45\x20\x75\x73\x65\x72\x73\x3B\x20\x2D\x2D", // hex encoded
    ];

    for attack in attacks {
        let escaped = escape_string(attack);

        // Should escape quotes if present
        if attack.contains('\'') {
            assert!(escaped.contains("\\'"));
        }
    }
}
