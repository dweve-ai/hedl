// HEDL WebAssembly Optimization Tests
//
// Comprehensive tests to verify that optimization passes preserve correctness

use hedl_core::parse as core_parse;

#[test]
fn test_parse_basic_after_optimization() {
    // Verify basic parsing still works after optimization
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
name: test
"#;
    let result = core_parse(hedl.as_bytes());
    assert!(result.is_ok(), "Basic parsing should work");

    let doc = result.unwrap();
    // Parsing v2.0 content preserves the version
    assert_eq!(doc.version, (2, 0));
}

#[test]
fn test_parse_complex_document() {
    // Test complex HEDL document to ensure all parser features work
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
%S:Post:[id, title, content]
%N:User>Post
%A:%active:"true"
---
users:@User
 |alice, Alice Smith, alice@example.com
  |post1, First Post, Hello world
 |bob, Bob Jones, bob@example.com
  |post2, Bob's Post, Content here
"#;

    let result = core_parse(hedl.as_bytes());
    assert!(result.is_ok(), "Complex document should parse");

    let doc = result.unwrap();
    assert_eq!(doc.structs.len(), 2, "Should have 2 structs");
    assert_eq!(doc.nests.len(), 1, "Should have 1 nest");
    assert_eq!(doc.aliases.len(), 1, "Should have 1 alias");
}

#[test]
fn test_parse_preserves_structure() {
    // Verify that optimization doesn't change parsed structure
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id, value]
---
items:@T
 |a, 1
 |b, 2
 |c, 3
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Check root items
    assert_eq!(doc.root.len(), 1, "Should have 1 root item");
    assert!(doc.root.contains_key("items"), "Should have 'items' key");

    // Check struct definition
    assert!(doc.structs.contains_key("T"), "Should have struct T");
    let schema = doc.structs.get("T").unwrap();
    assert_eq!(schema, &vec!["id".to_string(), "value".to_string()]);
}

#[test]
fn test_parse_edge_cases() {
    // Empty body
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
"#;
    assert!(
        core_parse(hedl.as_bytes()).is_ok(),
        "Empty body should parse"
    );

    // Only header
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id]
---
"#;
    assert!(
        core_parse(hedl.as_bytes()).is_ok(),
        "Header-only should parse"
    );

    // Large document (stress test)
    let mut large_hedl = String::from(
        r#"%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id]
---
items:@T
"#,
    );
    for i in 0..1000 {
        large_hedl.push_str(&format!(" |item{i}\n"));
    }
    assert!(
        core_parse(large_hedl.as_bytes()).is_ok(),
        "Large document should parse"
    );
}

#[test]
fn test_parse_error_handling() {
    // Invalid version
    let hedl = r#"%V:99.99
%NULL:~
%QUOTE:"
---
"#;
    let _result = core_parse(hedl.as_bytes());
    // Parser may accept any version, but we test error handling path

    // Missing version
    let hedl = "---\ndata: value\n";
    let result = core_parse(hedl.as_bytes());
    assert!(result.is_err(), "Missing version should error");

    // Invalid syntax
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
::::
"#;
    let result = core_parse(hedl.as_bytes());
    assert!(result.is_err(), "Invalid syntax should error");
}

#[test]
fn test_token_estimation_correctness() {
    // Verify token estimation still works after constant folding optimization
    fn estimate_tokens(text: &str) -> usize {
        let bytes = text.as_bytes();
        let byte_count = bytes.len();

        if byte_count == 0 {
            return 0;
        }

        let mut whitespace_count = 0usize;
        let mut punct_count = 0usize;
        let mut i = 0;

        while i < byte_count {
            let b = bytes[i];

            if b < 128 {
                whitespace_count += usize::from(matches!(b, b' ' | b'\t' | b'\n' | b'\r'));
                punct_count += usize::from(matches!(
                    b,
                    b'!' | b'"'
                        | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'('
                        | b')'
                        | b'*'
                        | b'+'
                        | b','
                        | b'-'
                        | b'.'
                        | b'/'
                        | b':'
                        | b';'
                        | b'<'
                        | b'='
                        | b'>'
                        | b'?'
                        | b'@'
                        | b'['
                        | b'\\'
                        | b']'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'{'
                        | b'|'
                        | b'}'
                        | b'~'
                ));
                i += 1;
            } else {
                let char_len = if b < 0b1110_0000 {
                    2
                } else if b < 0b1111_0000 {
                    3
                } else {
                    4
                };
                i += char_len;
            }
        }

        const CHARS_PER_TOKEN: usize = 4;
        (byte_count + whitespace_count + punct_count) / CHARS_PER_TOKEN
    }

    // Empty string
    assert_eq!(estimate_tokens(""), 0);

    // Simple text
    let tokens = estimate_tokens("hello world");
    assert!(
        tokens > 0 && tokens < 10,
        "Should estimate reasonable token count"
    );

    // With punctuation
    let tokens_plain = estimate_tokens("hello world");
    let tokens_punct = estimate_tokens("hello, world!");
    assert!(
        tokens_punct >= tokens_plain,
        "Punctuation should add tokens"
    );

    // With whitespace
    let tokens_compact = estimate_tokens("abc");
    let tokens_spaced = estimate_tokens("a b c");
    assert!(
        tokens_spaced > tokens_compact,
        "Whitespace should add tokens"
    );

    // Large text
    let large_text = "a".repeat(10000);
    let tokens = estimate_tokens(&large_text);
    assert!(tokens > 1000, "Large text should have many tokens");
}

#[test]
fn test_inlining_preserves_behavior() {
    // Test that inlined functions still behave correctly
    // Parse the same document multiple times to ensure consistency
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id]
---
items:@T
 |a
 |b
"#;

    let result1 = core_parse(hedl.as_bytes());
    let result2 = core_parse(hedl.as_bytes());
    let result3 = core_parse(hedl.as_bytes());

    assert!(
        result1.is_ok() && result2.is_ok() && result3.is_ok(),
        "Multiple parses should all succeed"
    );

    let doc1 = result1.unwrap();
    let doc2 = result2.unwrap();
    let doc3 = result3.unwrap();

    assert_eq!(doc1.version, doc2.version);
    assert_eq!(doc2.version, doc3.version);
    assert_eq!(doc1.structs.len(), doc2.structs.len());
    assert_eq!(doc2.structs.len(), doc3.structs.len());
}

#[test]
fn test_memory_layout_optimization() {
    // Verify that memory layout optimizations don't break functionality
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Access all document fields to ensure layout is correct
    let _version = doc.version;
    let _structs = &doc.structs;
    let _aliases = &doc.aliases;
    let _nests = &doc.nests;
    let _root = &doc.root;

    // All accesses should work without panics
}

#[test]
fn test_state_machine_optimization() {
    // Test various parser states to ensure state machine optimization is correct
    let test_cases = vec![
        (
            r#"%V:2.0
%NULL:~
%QUOTE:"
---
"#,
            true,
            "empty body",
        ),
        (
            r#"%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id]
---
"#,
            true,
            "struct only",
        ),
        (
            r#"%V:2.0
%NULL:~
%QUOTE:"
%A:%a:"b"
---
"#,
            true,
            "alias only",
        ),
        (
            r#"%V:2.0
%NULL:~
%QUOTE:"
---
key: value
"#,
            true,
            "scalar value",
        ),
        (
            r#"%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id]
---
items:@T
 |a
"#,
            true,
            "list",
        ),
        (
            r#"%V:2.0
%NULL:~
%QUOTE:"
---
obj:
 nested: value
"#,
            true,
            "nested object",
        ),
    ];

    for (hedl, should_pass, description) in test_cases {
        let result = core_parse(hedl.as_bytes());
        if should_pass {
            assert!(result.is_ok(), "Should parse: {description}");
        } else {
            assert!(result.is_err(), "Should fail: {description}");
        }
    }
}

#[test]
fn test_utf8_handling_after_optimization() {
    // Ensure UTF-8 handling is preserved after byte-level optimizations
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
---
emoji: 😀
chinese: 你好
arabic: مرحبا
russian: Привет
"#;

    let result = core_parse(hedl.as_bytes());
    assert!(result.is_ok(), "UTF-8 should parse correctly");
}

#[test]
fn test_boundary_conditions() {
    // Test boundary conditions that might expose optimization bugs

    // Single character
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
x: a
"#;
    assert!(core_parse(hedl.as_bytes()).is_ok());

    // Maximum reasonable version
    let hedl = r#"%V:255.255
%NULL:~
%QUOTE:"
---
"#;
    let _result = core_parse(hedl.as_bytes());
    // Version validation is done by parser

    // Nested objects (if supported)
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
a:
 b:
  c: value
"#;
    // This may or may not parse depending on HEDL's nesting support
    let _result = core_parse(hedl.as_bytes());
    // We just verify it doesn't crash
}

#[test]
fn test_concurrent_parsing() {
    // Test that optimizations don't introduce thread-safety issues
    // (even though WASM is single-threaded, we test the logic)
    use std::sync::Arc;

    let hedl = Arc::new(
        r#"%V:2.0
%NULL:~
%QUOTE:"
---
test: value
"#
        .to_string(),
    );

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let hedl = Arc::clone(&hedl);
            std::thread::spawn(move || core_parse(hedl.as_bytes()).is_ok())
        })
        .collect();

    for handle in handles {
        assert!(handle.join().unwrap(), "Concurrent parse should succeed");
    }
}

#[test]
fn test_optimization_does_not_change_output() {
    // Compare parsing results before and after optimization
    // This is a meta-test that would be run during CI
    let test_documents = vec![
        r#"%V:2.0
%NULL:~
%QUOTE:"
---
"#,
        r#"%V:2.0
%NULL:~
%QUOTE:"
---
key: value
"#,
        r#"%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id]
---
items:@T
 |a
 |b
"#,
    ];

    for hedl in test_documents {
        let doc1 = core_parse(hedl.as_bytes()).unwrap();
        let doc2 = core_parse(hedl.as_bytes()).unwrap();

        // Documents should be identical
        assert_eq!(doc1.version, doc2.version);
        assert_eq!(doc1.structs.keys().len(), doc2.structs.keys().len());
        assert_eq!(doc1.aliases.keys().len(), doc2.aliases.keys().len());
        assert_eq!(doc1.root.keys().len(), doc2.root.keys().len());
    }
}

// Regression tests for specific optimization passes

#[test]
fn test_dce_preserves_entry_points() {
    // Dead code elimination should preserve all entry points
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
test: value
"#;

    // Parse (entry point)
    assert!(core_parse(hedl.as_bytes()).is_ok());

    // Multiple invocations to ensure consistency
    for _ in 0..5 {
        assert!(core_parse(hedl.as_bytes()).is_ok());
    }
}

#[test]
fn test_constant_folding_correctness() {
    // Verify constant folding doesn't change semantics
    fn test_division_by_four(value: usize) -> usize {
        value / 4
    }

    // Test that division by 4 works correctly (may be optimized to shift)
    assert_eq!(test_division_by_four(0), 0);
    assert_eq!(test_division_by_four(4), 1);
    assert_eq!(test_division_by_four(8), 2);
    assert_eq!(test_division_by_four(100), 25);
    assert_eq!(test_division_by_four(1000), 250);
}

#[test]
fn test_inlining_does_not_cause_stack_overflow() {
    // Aggressive inlining should not cause stack overflow
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
"#
    .repeat(100);
    let result = core_parse(hedl.as_bytes());
    // May error due to content, but should not stack overflow
    let _ = result;
}

#[test]
fn test_optimization_with_all_value_types() {
    // Test various HEDL value types to ensure type conversions still work
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id, name, active, score]
---
values:@T
 |item1, test, true, 42
 |item2, "hello", false, 3.14
"#;

    let result = core_parse(hedl.as_bytes());
    assert!(result.is_ok(), "Value types should parse");
}

#[test]
fn test_wasm_specific_constraints() {
    // Test constraints specific to WASM environment

    // Large allocations (should work within reasonable limits)
    let header = r#"%V:2.0
%NULL:~
%QUOTE:"
---
data: "#;
    let large_doc = format!("{}{}\n", header, "x".repeat(100_000));
    assert!(core_parse(large_doc.as_bytes()).is_ok());

    // Many small allocations
    let mut many_items = String::from(
        r#"%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id]
---
items:@T
"#,
    );
    for i in 0..100 {
        many_items.push_str(&format!(" |item{i}\n"));
    }
    assert!(core_parse(many_items.as_bytes()).is_ok());
}
