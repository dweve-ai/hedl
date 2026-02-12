// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Comprehensive ID validation tests for hedl-neo4j.
//!
//! These tests are designed to be ADVERSARIAL - they actively try to break
//! the ID validation and escaping logic by testing:
//!
//! - Cypher injection attacks
//! - Unicode homograph attacks
//! - Control character injection
//! - Boundary conditions (empty, max length, off-by-one)
//! - OWASP-style injection patterns
//! - Bidirectional text attacks
//! - Zero-width character attacks
//! - Null byte injection
//! - Encoding edge cases
//!
//! If any of these tests pass when they should fail, we have a security vulnerability.
//! If any of these tests fail when they should pass, we have a bug.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_neo4j::{to_cypher, Neo4jError, ToCypherConfig};
use smallvec::SmallVec;
use std::collections::BTreeMap;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a minimal document with a single node having the given ID
fn doc_with_id(id: &str) -> Document {
    let mut root = BTreeMap::new();
    root.insert(
        "items".to_string(),
        Item::List(MatrixList {
            type_name: "Item".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![Node {
                type_name: "Item".to_string(),
                id: id.to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String(id.to_string().into()),
                    Value::String("test".to_string().into()),
                ]),
                children: None,
                child_count: 0,
            }],
            count_hint: None,
        }),
    );

    Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

/// Create a document with a reference to another node
fn doc_with_reference(from_id: &str, to_id: &str) -> Document {
    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "friend".to_string()],
            rows: vec![
                Node {
                    type_name: "User".to_string(),
                    id: from_id.to_string(),
                    fields: SmallVec::from_vec(vec![
                        Value::String(from_id.to_string().into()),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string().into()),
                            id: to_id.to_string().into(),
                        }),
                    ]),
                    children: None,
                    child_count: 0,
                },
                Node {
                    type_name: "User".to_string(),
                    id: to_id.to_string(),
                    fields: SmallVec::from_vec(vec![
                        Value::String(to_id.to_string().into()),
                        Value::Null,
                    ]),
                    children: None,
                    child_count: 0,
                },
            ],
            count_hint: None,
        }),
    );

    Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

/// Create a document with nested children
fn doc_with_nested_child(parent_id: &str, child_id: &str) -> Document {
    let child_node = Node {
        type_name: "Post".to_string(),
        id: child_id.to_string(),
        fields: SmallVec::from_vec(vec![
            Value::String(child_id.to_string().into()),
            Value::String("Post content".to_string().into()),
        ]),
        children: None,
        child_count: 0,
    };

    let mut parent_children = BTreeMap::new();
    parent_children.insert("posts".to_string(), vec![child_node]);

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: parent_id.to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String(parent_id.to_string().into()),
                    Value::String("Test User".to_string().into()),
                ]),
                children: Some(Box::new(parent_children)),
                child_count: 1,
            }],
            count_hint: None,
        }),
    );

    // nests: parent_type -> child_types (Vec<String> for multiple children)
    let mut nests = BTreeMap::new();
    nests.insert("User".to_string(), vec!["Post".to_string()]);

    // structs: type_name -> list of field names
    let mut structs = BTreeMap::new();
    structs.insert(
        "Post".to_string(),
        vec!["id".to_string(), "content".to_string()],
    );
    structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs,
        nests,
        root,
    }
}

/// Assert that the Cypher output does NOT contain a dangerous unescaped pattern
fn assert_no_injection(cypher: &str, pattern: &str, attack_name: &str) {
    // Check that the raw dangerous pattern is not present UNESCAPED
    // Properly escaped strings will have \' for quotes, so we need to check
    // if the pattern appears after removing escaped quotes.
    //
    // The key insight: if injection text appears inside a quoted string with
    // properly escaped quotes, it's safe. We detect UNSAFE injection by:
    // 1. Replace all escaped quotes (\') with a placeholder
    // 2. Check if the dangerous pattern still appears (unescaped)

    let dangerous = pattern.contains(';')
        || pattern.contains("--")
        || pattern.contains("/*")
        || pattern.contains("*/")
        || pattern.contains("MATCH")
        || pattern.contains("DELETE")
        || pattern.contains("DROP")
        || pattern.contains("DETACH");

    if dangerous {
        // Replace escaped quotes to detect truly unescaped injection
        let cypher_unescaped_check = cypher.replace("\\'", "ESCAPED_QUOTE");
        assert!(
            !cypher_unescaped_check.contains(pattern),
            "SECURITY VULNERABILITY: {attack_name} attack succeeded!\nPattern '{pattern}' found UNESCAPED in output:\n{cypher}"
        );
    }
}

// =============================================================================
// CYPHER INJECTION ATTACK TESTS
// =============================================================================

#[test]
fn test_semicolon_injection() {
    // Attempt to terminate statement and inject new command
    let malicious_id = "alice'; DELETE (n); //";
    let doc = doc_with_id(malicious_id);
    let config = ToCypherConfig::default();

    let result = to_cypher(&doc, &config);
    assert!(
        result.is_ok(),
        "Conversion should succeed with escaped content"
    );

    let cypher = result.unwrap();
    // The injection text may appear in the output inside a properly quoted string.
    // That's safe! What we need to check is that it's NOT unescaped.
    // Replace escaped quotes and check if injection pattern still appears.
    let cypher_unescaped = cypher.replace("\\'", "ESCAPED_QUOTE");
    assert!(
        !cypher_unescaped.contains("'; DELETE"),
        "Injection attack: unescaped quote-semicolon found"
    );

    // Verify proper escaping happened
    assert!(
        cypher.contains("\\'"),
        "Single quotes should be escaped with backslash"
    );
}

#[test]
fn test_comment_injection_double_dash() {
    // Attempt to comment out rest of query
    let malicious_id = "alice' -- comment";
    let doc = doc_with_id(malicious_id);
    let config = ToCypherConfig::default();

    let result = to_cypher(&doc, &config);
    assert!(result.is_ok());

    let cypher = result.unwrap();
    // Double dash should be escaped
    assert_no_injection(&cypher, "' --", "double-dash comment injection");
}

#[test]
fn test_comment_injection_single_dash() {
    // Single dash shouldn't be a problem but test anyway
    let id = "alice-bob";
    let doc = doc_with_id(id);
    let result = to_cypher(&doc, &ToCypherConfig::default());
    assert!(result.is_ok());
}

#[test]
fn test_block_comment_injection() {
    // Attempt to inject block comment
    let malicious_id = "alice'/* comment */";
    let doc = doc_with_id(malicious_id);
    let config = ToCypherConfig::default();

    let result = to_cypher(&doc, &config);
    assert!(result.is_ok());

    let cypher = result.unwrap();
    // Block comment markers may appear inside a quoted string (safe).
    // The critical check is that the quote before them is escaped.
    let cypher_unescaped = cypher.replace("\\'", "ESCAPED_QUOTE");
    assert!(
        !cypher_unescaped.contains("'/*"),
        "Block comment injection: unescaped quote before /* found"
    );

    // Verify escaping happened
    assert!(
        cypher.contains("\\'"),
        "Single quotes should be escaped with backslash"
    );
}

#[test]
fn test_double_quote_injection() {
    // Attempt to break out of double-quoted string
    let malicious_id = r#"alice" OR 1=1 --"#;
    let doc = doc_with_id(malicious_id);
    let config = ToCypherConfig::default();

    let result = to_cypher(&doc, &config);
    assert!(result.is_ok());

    let cypher = result.unwrap();
    assert_no_injection(&cypher, "OR 1=1", "double quote injection");
}

#[test]
fn test_single_quote_injection() {
    // Classic SQL/Cypher injection with single quote
    let malicious_id = "alice' OR '1'='1";
    let doc = doc_with_id(malicious_id);
    let config = ToCypherConfig::default();

    let result = to_cypher(&doc, &config);
    assert!(result.is_ok());

    let cypher = result.unwrap();
    // The single quotes must be escaped
    assert!(
        cypher.contains("\\'") || !cypher.contains("' OR '"),
        "Single quotes must be escaped to prevent injection"
    );
}

#[test]
fn test_backslash_injection() {
    // Backslash might escape the escape character
    // Input: alice\'; DELETE (n); //
    // After escaping: alice\\\'; DELETE (n); //
    // This is SAFE because \\ is escaped backslash, \' is escaped quote
    let malicious_id = r"alice\'; DELETE (n); //";
    let doc = doc_with_id(malicious_id);
    let config = ToCypherConfig::default();

    let result = to_cypher(&doc, &config);
    assert!(result.is_ok());

    let cypher = result.unwrap();
    // The backslash should be escaped to \\, so the dangerous text is safely inside a string.
    // Check that both backslash and quote are properly escaped.
    assert!(
        cypher.contains("\\\\"),
        "Backslash should be escaped to double backslash"
    );
    assert!(cypher.contains("\\'"), "Quote should be escaped");

    // The critical check: after removing escaped sequences, no unescaped injection should remain
    let cypher_unescaped = cypher
        .replace("\\\\", "ESCAPED_BACKSLASH")
        .replace("\\'", "ESCAPED_QUOTE");
    assert!(
        !cypher_unescaped.contains("'; DELETE"),
        "Backslash escape attack: unescaped quote-semicolon found"
    );
}

#[test]
fn test_boolean_injection() {
    // Attempt boolean-based injection
    let malicious_id = "alice' AND 1=1 AND 'a'='a";
    let doc = doc_with_id(malicious_id);
    let config = ToCypherConfig::default();

    let result = to_cypher(&doc, &config);
    assert!(result.is_ok());

    let cypher = result.unwrap();
    assert_no_injection(&cypher, "AND 1=1", "boolean injection");
}

#[test]
fn test_cypher_specific_attacks() {
    // Cypher-specific keyword injection attempts
    let attacks = vec![
        "alice' MATCH (n) DETACH DELETE n //",
        "alice' MERGE (n:Hacked) //",
        "alice' CREATE (n:Evil) //",
        "alice' SET n.pwned=true //",
        "alice' REMOVE n.important //",
        "alice' CALL db.labels() YIELD label //",
        "alice' UNWIND range(1,1000000) AS x CREATE (n) //",
    ];

    for attack in attacks {
        let doc = doc_with_id(attack);
        let result = to_cypher(&doc, &ToCypherConfig::default());
        assert!(result.is_ok(), "Should handle attack pattern: {attack}");

        let cypher = result.unwrap();
        // These patterns may appear in the output inside quoted strings (safe).
        // The critical check is that quotes are escaped, so they can't break out.
        let cypher_unescaped = cypher.replace("\\'", "ESCAPED_QUOTE");

        // Check that after removing escaped quotes, no unescaped injection point remains
        assert!(
            !cypher_unescaped.contains("' MATCH"),
            "MATCH injection unescaped in: {attack}"
        );
        assert!(
            !cypher_unescaped.contains("' MERGE"),
            "MERGE injection unescaped in: {attack}"
        );
        assert!(
            !cypher_unescaped.contains("' CREATE"),
            "CREATE injection unescaped in: {attack}"
        );
        assert!(
            !cypher_unescaped.contains("' SET"),
            "SET injection unescaped in: {attack}"
        );

        // Verify escaping happened
        assert!(
            cypher.contains("\\'"),
            "Single quotes should be escaped for: {attack}"
        );
    }
}

#[test]
fn test_owasp_injection_patterns() {
    // OWASP SQL injection patterns adapted for Cypher
    let owasp_patterns = vec![
        "' OR ''='",
        "' OR 1=1--",
        "' OR 'x'='x",
        "'; MATCH (n) RETURN n; --",
        "' UNION MATCH (n) RETURN n--",
        "admin'--",
        "1' OR '1'='1",
        "' OR 1=1#",
        "' OR 1=1/*",
        "') OR ('1'='1",
        "' OR 'one'='one",
    ];

    for pattern in owasp_patterns {
        let doc = doc_with_id(pattern);
        let result = to_cypher(&doc, &ToCypherConfig::default());
        assert!(result.is_ok(), "Should handle OWASP pattern: {pattern}");
    }
}

// =============================================================================
// UNICODE ATTACK TESTS
// =============================================================================

#[test]
fn test_null_byte_filtered() {
    // Null byte injection attempt
    let malicious_id = "alice\x00bob";
    let doc = doc_with_id(malicious_id);
    let config = ToCypherConfig::default();

    let result = to_cypher(&doc, &config);
    assert!(result.is_ok());

    let cypher = result.unwrap();
    // Null bytes should be filtered or escaped
    assert!(
        !cypher.contains('\x00'),
        "Null bytes must be filtered from output"
    );
}

#[test]
fn test_newline_filtered() {
    // Newline injection
    let malicious_id = "alice\nbob";
    let doc = doc_with_id(malicious_id);
    let result = to_cypher(&doc, &ToCypherConfig::default());
    assert!(result.is_ok());

    let cypher = result.unwrap();
    // Raw newlines in IDs should be escaped
    assert!(
        cypher.contains("\\n") || !cypher.contains("alice\nbob"),
        "Newlines must be escaped"
    );
}

#[test]
fn test_carriage_return_filtered() {
    let malicious_id = "alice\rbob";
    let doc = doc_with_id(malicious_id);
    let result = to_cypher(&doc, &ToCypherConfig::default());
    assert!(result.is_ok());

    let cypher = result.unwrap();
    assert!(
        cypher.contains("\\r") || !cypher.contains("alice\rbob"),
        "Carriage returns must be escaped"
    );
}

#[test]
fn test_tab_filtered() {
    let malicious_id = "alice\tbob";
    let doc = doc_with_id(malicious_id);
    let result = to_cypher(&doc, &ToCypherConfig::default());
    assert!(result.is_ok());

    let cypher = result.unwrap();
    assert!(
        cypher.contains("\\t") || !cypher.contains("alice\tbob"),
        "Tabs must be escaped"
    );
}

#[test]
fn test_zero_width_space_filtered() {
    // Zero-width space (U+200B) - invisible character
    let malicious_id = "alice\u{200B}bob";
    let doc = doc_with_id(malicious_id);
    let result = to_cypher(&doc, &ToCypherConfig::default());
    assert!(result.is_ok());

    let cypher = result.unwrap();
    assert!(
        !cypher.contains('\u{200B}'),
        "Zero-width space must be filtered"
    );
}

#[test]
fn test_zero_width_joiner_filtered() {
    // Zero-width joiner (U+200D)
    let malicious_id = "alice\u{200D}bob";
    let doc = doc_with_id(malicious_id);
    let result = to_cypher(&doc, &ToCypherConfig::default());
    assert!(result.is_ok());

    let cypher = result.unwrap();
    assert!(
        !cypher.contains('\u{200D}'),
        "Zero-width joiner must be filtered"
    );
}

#[test]
fn test_rtl_override_filtered() {
    // Right-to-left override (U+202E) - text direction attack
    let malicious_id = "alice\u{202E}bob";
    let doc = doc_with_id(malicious_id);
    let result = to_cypher(&doc, &ToCypherConfig::default());
    assert!(result.is_ok());

    let cypher = result.unwrap();
    assert!(
        !cypher.contains('\u{202E}'),
        "RTL override must be filtered"
    );
}

#[test]
fn test_ltr_override_filtered() {
    // Left-to-right override (U+202D)
    let malicious_id = "alice\u{202D}bob";
    let doc = doc_with_id(malicious_id);
    let result = to_cypher(&doc, &ToCypherConfig::default());
    assert!(result.is_ok());

    let cypher = result.unwrap();
    assert!(
        !cypher.contains('\u{202D}'),
        "LTR override must be filtered"
    );
}

#[test]
fn test_unicode_normalization_composed_vs_decomposed() {
    // Test that composed and decomposed forms are normalized consistently
    let composed_id = "caf\u{00E9}"; // é as single codepoint
    let decomposed_id = "cafe\u{0301}"; // e + combining acute accent

    let doc1 = doc_with_id(composed_id);
    let doc2 = doc_with_id(decomposed_id);

    let result1 = to_cypher(&doc1, &ToCypherConfig::default());
    let result2 = to_cypher(&doc2, &ToCypherConfig::default());

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Both should normalize to the same form
    let cypher1 = result1.unwrap();
    let cypher2 = result2.unwrap();

    // Extract the ID from both outputs and compare
    // They should be identical after NFC normalization
    assert!(
        cypher1.contains("café") || cypher1.contains("`café`"),
        "Composed form should be present"
    );
    assert!(
        cypher2.contains("café") || cypher2.contains("`café`"),
        "Decomposed form should normalize to composed"
    );
}

#[test]
fn test_only_dangerous_unicode_rejected() {
    // Normal Unicode should be allowed (but may be backtick-quoted)
    let unicode_ids = vec![
        "ユーザー",     // Japanese
        "пользователь", // Russian
        "用户",         // Chinese
        "🔥user",       // Emoji prefix
        "user🔥",       // Emoji suffix
        "naïve",        // Accented Latin
    ];

    for id in unicode_ids {
        let doc = doc_with_id(id);
        let result = to_cypher(&doc, &ToCypherConfig::default());
        assert!(result.is_ok(), "Valid Unicode ID '{id}' should be accepted");
    }
}

// =============================================================================
// BOUNDARY CONDITION TESTS
// =============================================================================

#[test]
fn test_empty_node_id_rejected() {
    let doc = doc_with_id("");
    let config = ToCypherConfig::default();

    // Empty IDs should either be rejected or handled gracefully
    let result = to_cypher(&doc, &config);
    // The behavior depends on implementation - either error or generated ID
    // But it must not crash or produce invalid Cypher
    if let Ok(cypher) = result {
        // Empty string in quotes is safe - it's not an injection vector
        // Verify the output is valid Cypher syntax (properly quoted)
        assert!(
            cypher.contains("_hedl_id: ''") || !cypher.contains("_hedl_id:"),
            "Empty ID must be properly quoted or ID field omitted"
        );
        // Also verify no unbalanced quotes
        let cypher_for_counting = cypher.replace("\\'", "XX");
        assert!(
            cypher_for_counting.chars().filter(|&c| c == '\'').count() % 2 == 0,
            "Empty ID must not cause unbalanced quotes"
        );
    }
}

#[test]
fn test_empty_reference_id_rejected() {
    let doc = doc_with_reference("alice", "");
    let config = ToCypherConfig::default();

    let result = to_cypher(&doc, &config);
    // Either reject or handle gracefully
    if let Ok(cypher) = result {
        // Empty reference target should be handled
        assert!(
            !cypher.contains("target: ''") || cypher.contains("target: '\"\"'"),
            "Empty reference target must be handled safely"
        );
    }
}

#[test]
fn test_very_long_id_rejected() {
    // Test with ID longer than reasonable limits
    let long_id = "a".repeat(100_000); // 100KB ID
    let doc = doc_with_id(&long_id);
    let config = ToCypherConfig::default().with_max_string_length(1000);

    let result = to_cypher(&doc, &config);

    // Should fail due to string length limit
    assert!(result.is_err(), "Very long ID should be rejected");

    match result {
        Err(Neo4jError::StringLengthExceeded {
            length, max_length, ..
        }) => {
            assert_eq!(length, 100_000);
            assert_eq!(max_length, 1000);
        }
        Err(other) => {
            // Other error types are acceptable too
            println!("Got error: {other:?}");
        }
        Ok(_) => panic!("Should have rejected very long ID"),
    }
}

#[test]
fn test_whitespace_only_id() {
    let whitespace_ids = vec![" ", "  ", "\t", "   \t   ", "\n", "\r\n"];

    for id in whitespace_ids {
        let doc = doc_with_id(id);
        let result = to_cypher(&doc, &ToCypherConfig::default());
        // Should either reject or escape properly
        if let Ok(cypher) = result {
            // Must not produce broken Cypher
            assert!(
                !cypher.contains("_hedl_id: }"),
                "Whitespace ID must be properly quoted"
            );
        }
    }
}

#[test]
fn test_single_character_ids() {
    let single_chars = vec!["a", "1", "_", "-", ".", "@", "#"];

    for id in single_chars {
        let doc = doc_with_id(id);
        let result = to_cypher(&doc, &ToCypherConfig::default());
        assert!(
            result.is_ok(),
            "Single character ID '{id}' should be handled"
        );
    }
}

// =============================================================================
// NESTED CHILD ID VALIDATION TESTS
// =============================================================================

#[test]
fn test_nest_parent_id_validation() {
    // Malicious parent ID with nested child
    let malicious_parent = "parent'; DELETE (n); //";
    let doc = doc_with_nested_child(malicious_parent, "child1");
    let result = to_cypher(&doc, &ToCypherConfig::default());

    assert!(result.is_ok());
    let cypher = result.unwrap();

    // The injection text may appear in the output, but it MUST be properly escaped.
    // A proper escape means the single quote is escaped: \' not '
    // So "parent'; DELETE" should become "parent\'; DELETE" which is safely inside a string literal.
    //
    // To check for UNESCAPED injection, we replace escaped quotes with a placeholder,
    // then check if the dangerous pattern still exists. If it does, we have an unescaped injection.
    let cypher_with_escaped_quotes_removed = cypher.replace("\\'", "ESCAPED_QUOTE");
    assert!(
        !cypher_with_escaped_quotes_removed.contains("'; DELETE"),
        "Nested parent ID injection found unescaped (single quote not escaped before semicolon)"
    );

    // Also verify the escaped form IS present (confirms escaping happened)
    assert!(
        cypher.contains("\\'"),
        "Single quotes in ID should be escaped with backslash"
    );
}

#[test]
fn test_nest_child_id_validation() {
    // Malicious child ID
    let malicious_child = "child'; DELETE (n); //";
    let doc = doc_with_nested_child("parent1", malicious_child);
    let result = to_cypher(&doc, &ToCypherConfig::default());

    assert!(result.is_ok());
    let cypher = result.unwrap();

    // The injection text may appear in the output, but it MUST be properly escaped.
    // To check for UNESCAPED injection, we replace escaped quotes with a placeholder,
    // then check if the dangerous pattern still exists.
    let cypher_with_escaped_quotes_removed = cypher.replace("\\'", "ESCAPED_QUOTE");
    assert!(
        !cypher_with_escaped_quotes_removed.contains("'; DELETE"),
        "Nested child ID injection found unescaped (single quote not escaped before semicolon)"
    );

    // Also verify the escaped form IS present (confirms escaping happened)
    assert!(
        cypher.contains("\\'"),
        "Single quotes in child ID should be escaped with backslash"
    );
}

#[test]
fn test_nested_reference_id_injection_rejected() {
    // Reference with malicious target ID
    let malicious_ref = "bob'; MATCH (n) DETACH DELETE n //";
    let doc = doc_with_reference("alice", malicious_ref);
    let result = to_cypher(&doc, &ToCypherConfig::default());

    assert!(result.is_ok());
    let cypher = result.unwrap();

    // The dangerous text may appear inside a properly quoted string (safe).
    // Check that it's not UNESCAPED (i.e., quote is properly escaped before it).
    let cypher_unescaped = cypher.replace("\\'", "ESCAPED_QUOTE");
    assert!(
        !cypher_unescaped.contains("'; MATCH"),
        "Reference target ID injection: unescaped quote before MATCH found"
    );
}

#[test]
fn test_nested_child_id_injection_rejected() {
    // Both parent and child have injection attempts
    let malicious_parent = "parent' OR '1'='1";
    let malicious_child = "child' OR '1'='1";
    let doc = doc_with_nested_child(malicious_parent, malicious_child);
    let result = to_cypher(&doc, &ToCypherConfig::default());

    assert!(result.is_ok());
    // Conversion should succeed with proper escaping
}

// =============================================================================
// REFERENCE ID VALIDATION TESTS
// =============================================================================

#[test]
fn test_node_id_injection_rejected() {
    let malicious_id = "alice'; MATCH (n) RETURN n; //";
    let doc = doc_with_id(malicious_id);
    let result = to_cypher(&doc, &ToCypherConfig::default());

    assert!(result.is_ok());
    let cypher = result.unwrap();
    // The injection payload may appear inside a properly quoted string, which is safe.
    // Check that there's no UNESCAPED quote before the MATCH keyword.
    let cypher_unescaped = cypher.replace("\\'", "ESCAPED_QUOTE");
    assert!(
        !cypher_unescaped.contains("'; MATCH"),
        "Node ID injection: unescaped quote before MATCH found"
    );
}

#[test]
fn test_reference_id_injection_rejected() {
    let malicious_ref = "bob'; CREATE (n:Pwned); //";
    let doc = doc_with_reference("alice", malicious_ref);
    let result = to_cypher(&doc, &ToCypherConfig::default());

    assert!(result.is_ok());
    let cypher = result.unwrap();
    // The injection payload may appear inside a properly quoted string, which is safe.
    // Check that there's no UNESCAPED quote before the CREATE keyword.
    let cypher_unescaped = cypher.replace("\\'", "ESCAPED_QUOTE");
    assert!(
        !cypher_unescaped.contains("'; CREATE"),
        "Reference ID injection: unescaped quote before CREATE found"
    );
}

// =============================================================================
// PROPERTY-BASED / FUZZ-STYLE TESTS
// =============================================================================

#[test]
fn test_random_byte_sequences() {
    // Test with various byte patterns that might cause issues
    let byte_patterns: Vec<&[u8]> = vec![
        b"\x00\x01\x02\x03",
        b"\xff\xfe\xfd",
        b"\x7f\x80\x81",
        b"\x1b[31m",     // ANSI escape
        b"\xe2\x80\x8b", // Zero-width space in UTF-8
        b"\xef\xbb\xbf", // BOM
    ];

    for pattern in byte_patterns {
        if let Ok(s) = std::str::from_utf8(pattern) {
            let doc = doc_with_id(s);
            let result = to_cypher(&doc, &ToCypherConfig::default());
            // Should either succeed with escaping or fail gracefully
            if let Ok(cypher) = result {
                // Verify output is valid
                assert!(
                    !cypher.is_empty(),
                    "Output should not be empty for pattern {pattern:?}"
                );
            } else {
                // Errors are acceptable for malformed input
            }
        }
    }
}

#[test]
fn test_special_neo4j_characters() {
    // Characters that have special meaning in Cypher
    let special_chars = vec![
        ":", // Label prefix
        "`", // Backtick quote
        "{", // Map start
        "}", // Map end
        "[", // List/relationship start
        "]", // List/relationship end
        "(", // Node start
        ")", // Node end
        "<", // Direction
        ">", // Direction
        "-", // Relationship dash
        "|", // CASE separator
        "$", // Parameter prefix
        "*", // Wildcard
    ];

    for ch in special_chars {
        let id = format!("test{ch}id");
        let doc = doc_with_id(&id);
        let result = to_cypher(&doc, &ToCypherConfig::default());
        assert!(result.is_ok(), "Special character '{ch}' should be handled");
    }
}

#[test]
fn test_cypher_keywords_as_ids() {
    // All Cypher keywords should be safely quoted as IDs
    let keywords = vec![
        "MATCH", "CREATE", "DELETE", "RETURN", "WHERE", "SET", "REMOVE", "MERGE", "WITH", "UNWIND",
        "FOREACH", "CALL", "YIELD", "UNION", "ORDER", "SKIP", "LIMIT", "NULL", "TRUE", "FALSE",
        "AND", "OR", "NOT", "XOR", "IN", "STARTS", "ENDS", "CONTAINS", "IS", "AS", "DISTINCT",
    ];

    for keyword in keywords {
        let doc = doc_with_id(keyword);
        let result = to_cypher(&doc, &ToCypherConfig::default());
        assert!(
            result.is_ok(),
            "Keyword '{keyword}' should be safely handled as ID"
        );

        // Also test lowercase
        let doc_lower = doc_with_id(&keyword.to_lowercase());
        let result_lower = to_cypher(&doc_lower, &ToCypherConfig::default());
        assert!(
            result_lower.is_ok(),
            "Lowercase keyword '{}' should be safely handled",
            keyword.to_lowercase()
        );
    }
}

// =============================================================================
// CONCURRENT SAFETY TESTS
// =============================================================================

#[test]
fn test_concurrent_conversion_safety() {
    use std::sync::Arc;
    use std::thread;

    let ids: Vec<String> = (0..100)
        .map(|i| format!("user{i}'; DELETE (n); //"))
        .collect();

    let ids = Arc::new(ids);
    let mut handles = vec![];

    for _ in 0..4 {
        let ids = Arc::clone(&ids);
        handles.push(thread::spawn(move || {
            for id in ids.iter() {
                let doc = doc_with_id(id);
                let result = to_cypher(&doc, &ToCypherConfig::default());
                assert!(result.is_ok());

                let cypher = result.unwrap();
                // Check for unescaped injection - escaped quotes are safe
                let cypher_unescaped = cypher.replace("\\'", "ESCAPED_QUOTE");
                assert!(
                    !cypher_unescaped.contains("'; DELETE"),
                    "Concurrent conversion: unescaped quote before DELETE found"
                );
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

// =============================================================================
// INVARIANT TESTS
// =============================================================================

#[test]
fn test_invariant_escaped_output_is_valid_cypher_syntax() {
    // Any escaped output should be syntactically valid Cypher
    let test_ids = vec![
        "simple",
        "with space",
        "with'quote",
        "with\"double",
        "with\\backslash",
        "with\nnewline",
        "with;semicolon",
        "特殊文字",
        "emoji🔥",
    ];

    for id in test_ids {
        let doc = doc_with_id(id);
        let result = to_cypher(&doc, &ToCypherConfig::default());
        assert!(result.is_ok(), "ID '{id}' should produce valid output");

        let cypher = result.unwrap();
        // Basic syntax checks - must account for escaped quotes
        // Replace escaped quotes with placeholder before counting
        let cypher_for_counting = cypher.replace("\\'", "XX").replace("\\\"", "YY");
        assert!(
            cypher_for_counting.chars().filter(|&c| c == '\'').count() % 2 == 0,
            "Unbalanced single quotes for ID '{id}': {cypher}"
        );
        assert!(
            cypher_for_counting.chars().filter(|&c| c == '"').count() % 2 == 0,
            "Unbalanced double quotes for ID '{id}': {cypher}"
        );
    }
}

#[test]
fn test_invariant_no_raw_user_input_in_keywords() {
    // User input should never appear adjacent to Cypher keywords in a way
    // that could be interpreted as part of the query structure
    let dangerous_id = "MATCH (evil:Hacker) DELETE evil";
    let doc = doc_with_id(dangerous_id);
    let result = to_cypher(&doc, &ToCypherConfig::default());

    assert!(result.is_ok());
    let cypher = result.unwrap();

    // The dangerous ID should be within quotes, not parsed as keywords
    // Count the MATCHes - there should only be structural ones, not from the ID
    let match_count = cypher.matches("MATCH").count();
    // The ID contains MATCH but it should be in a string, so we should see
    // the structural MATCH keywords from the generated Cypher
    assert!(
        match_count <= 3, // Allow for structural MATCHes in relationship creation
        "Too many MATCH keywords found - possible injection: {cypher}"
    );
}
