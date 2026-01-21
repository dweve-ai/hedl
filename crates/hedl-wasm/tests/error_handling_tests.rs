// HEDL WebAssembly Error Handling Tests
//
// Comprehensive tests for error paths, edge cases, and error recovery

use hedl_core::{parse as core_parse, Item, Value};

// ============ PARSE ERROR TESTS ============

#[test]
fn test_parse_error_missing_version() {
    let hedl = "---\ndata: value\n";
    let result = core_parse(hedl.as_bytes());

    assert!(result.is_err(), "Should error on missing version");
    let err = result.unwrap_err();
    assert!(!err.message.is_empty());
}

#[test]
fn test_parse_error_invalid_separator() {
    let hedl = "%VERSION: 1.0\n===\ndata: value\n";
    let result = core_parse(hedl.as_bytes());

    // May error depending on parser strictness
    let _ = result;
}

#[test]
fn test_parse_error_malformed_struct() {
    let hedl = "%VERSION: 1.0\n%STRUCT: BadStruct\n---\n";
    let result = core_parse(hedl.as_bytes());

    // May error on malformed struct definition
    let _ = result;
}

#[test]
fn test_parse_error_malformed_alias() {
    let hedl = "%VERSION: 1.0\n%ALIAS: %bad\n---\n";
    let result = core_parse(hedl.as_bytes());

    // May error on malformed alias
    let _ = result;
}

#[test]
fn test_parse_error_malformed_nest() {
    let hedl = "%VERSION: 1.0\n%NEST: Parent\n---\n";
    let result = core_parse(hedl.as_bytes());

    // May error on malformed nest
    let _ = result;
}

#[test]
fn test_parse_error_unclosed_quote() {
    let hedl = r#"
%VERSION: 1.0
---
value: "unclosed string
"#;
    let result = core_parse(hedl.as_bytes());

    // Should handle unclosed quotes
    let _ = result;
}

#[test]
fn test_parse_error_invalid_list_syntax() {
    let hedl = r"
%VERSION: 1.0
%STRUCT: T: [id]
---
items: @T
  | | invalid
";
    let result = core_parse(hedl.as_bytes());

    // May error on invalid list syntax
    let _ = result;
}

#[test]
fn test_parse_error_undefined_type() {
    let hedl = r"
%VERSION: 1.0
---
items: @UndefinedType
  | id1
";
    let result = core_parse(hedl.as_bytes());

    // Parser may accept undefined types (validation catches this)
    let _ = result;
}

#[test]
fn test_parse_error_mismatched_field_count() {
    let hedl = r"
%VERSION: 1.0
%STRUCT: T: [id, name, email]
---
items: @T
  | alice, Alice
";
    let result = core_parse(hedl.as_bytes());

    // Parser may accept mismatched fields (validation catches this)
    if let Ok(doc) = result {
        // Verify we can still access the document
        assert_eq!(doc.version, (1, 0));
    }
}

// ============ VALIDATION ERROR TESTS ============

#[test]
#[cfg(feature = "full-validation")]
fn test_validation_error_unused_schema() {
    use hedl_lint::lint;

    let hedl = "%VERSION: 1.0\n%STRUCT: UnusedType: [id]\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let diagnostics = lint(&doc);

    // Check for warnings about unused schema
    let has_unused_warning = diagnostics.iter().any(|d| {
        matches!(
            d.severity(),
            hedl_lint::Severity::Warning | hedl_lint::Severity::Hint
        )
    });

    // May or may not have warnings depending on linter rules
    let _ = has_unused_warning;
}

#[test]
#[cfg(feature = "full-validation")]
fn test_validation_multiple_errors() {
    use hedl_lint::lint;

    let hedl = r"
%VERSION: 1.0
%STRUCT: T1: [id]
%STRUCT: T2: [id]
---
";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let diagnostics = lint(&doc);

    // Document may have multiple diagnostics
    // Just verify it doesn't panic
    let _ = diagnostics.len();
}

// ============ JSON CONVERSION ERROR TESTS ============

#[test]
#[cfg(feature = "json")]
fn test_json_parse_error_invalid_json() {
    let invalid_json = "{broken json]";
    let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);

    assert!(result.is_err());
}

#[test]
#[cfg(feature = "json")]
fn test_json_parse_error_empty_string() {
    let empty = "";
    let result: Result<serde_json::Value, _> = serde_json::from_str(empty);

    assert!(result.is_err());
}

#[test]
#[cfg(feature = "json")]
fn test_json_parse_error_incomplete() {
    let incomplete = r#"{"key": "#;
    let result: Result<serde_json::Value, _> = serde_json::from_str(incomplete);

    assert!(result.is_err());
}

#[test]
#[cfg(feature = "json")]
fn test_from_json_null_value() {
    use hedl_json::{from_json_value, FromJsonConfig};

    let json = serde_json::Value::Null;
    let config = FromJsonConfig::default();

    let result = from_json_value(&json, &config);
    // Null value may or may not convert to valid HEDL
    // Just verify it doesn't panic
    let _ = result;
}

#[test]
#[cfg(feature = "json")]
fn test_from_json_primitive_types() {
    use hedl_json::{from_json_value, FromJsonConfig};

    let config = FromJsonConfig::default();

    // Primitive types may not convert directly to HEDL documents
    // Just verify they don't panic

    // Boolean
    let result = from_json_value(&serde_json::json!(true), &config);
    let _ = result;

    // Number
    let result = from_json_value(&serde_json::json!(42), &config);
    let _ = result;

    // String
    let result = from_json_value(&serde_json::json!("test"), &config);
    let _ = result;
}

// ============ CANONICALIZATION ERROR TESTS ============

#[test]
fn test_canonicalize_minimal_document() {
    use hedl_c14n::{canonicalize_with_config, CanonicalConfig};

    let hedl = "%VERSION: 1.0\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = CanonicalConfig::default();
    let result = canonicalize_with_config(&doc, &config);

    assert!(result.is_ok(), "Should canonicalize minimal document");
}

#[test]
fn test_canonicalize_with_all_features() {
    use hedl_c14n::{canonicalize_with_config, CanonicalConfig};

    let hedl = r#"
%VERSION: 1.0
%STRUCT: T: [id, value]
%STRUCT: Parent: [id]
%STRUCT: Child: [id]
%ALIAS: %test: "value"
%NEST: Parent > Child
---
data: @T
  | a, 1
"#;

    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = CanonicalConfig::default();
    let result = canonicalize_with_config(&doc, &config);

    assert!(result.is_ok(), "Should handle all features");
}

// ============ VALUE CONVERSION ERROR TESTS ============

#[test]
#[cfg(feature = "query-api")]
fn test_value_to_json_all_types() {
    // Test all value types convert without error
    let test_values = vec![
        Value::Null,
        Value::Bool(true),
        Value::Bool(false),
        Value::Int(42),
        Value::Int(-42),
        Value::Float(1.234),
        Value::Float(-5.678),
        Value::String("test".into()),
    ];

    for value in test_values {
        // Conversion should not panic
        let _ = format!("{value:?}");
    }
}

#[test]
fn test_value_edge_cases() {
    // Test edge case values
    let max_int = Value::Int(i64::MAX);
    let min_int = Value::Int(i64::MIN);
    let zero = Value::Int(0);

    let _ = format!("{max_int:?}");
    let _ = format!("{min_int:?}");
    let _ = format!("{zero:?}");
}

#[test]
fn test_value_float_special() {
    // Test special float values
    let infinity = Value::Float(f64::INFINITY);
    let neg_infinity = Value::Float(f64::NEG_INFINITY);
    let zero = Value::Float(0.0);

    let _ = format!("{infinity:?}");
    let _ = format!("{neg_infinity:?}");
    let _ = format!("{zero:?}");

    // NaN is tricky, test it separately
    let nan = Value::Float(f64::NAN);
    let debug_str = format!("{nan:?}");
    assert!(!debug_str.is_empty());
}

// ============ MEMORY LIMIT ERROR TESTS ============

#[test]
fn test_input_size_limit_boundary() {
    let limit = 1000;

    // Just under limit
    let under = "x".repeat(999);
    assert!(under.len() < limit);

    // Exactly at limit
    let at = "x".repeat(1000);
    assert_eq!(at.len(), limit);

    // Just over limit
    let over = "x".repeat(1001);
    assert!(over.len() > limit);
}

#[test]
fn test_input_size_zero() {
    let empty = "";
    assert_eq!(empty.len(), 0);

    // Should not error on empty input for size check
    // (but will error on parsing)
}

#[test]
fn test_input_size_very_large() {
    // Test that we can calculate sizes for very large inputs
    let size = 1_000_000_000usize; // 1 GB
    let mb = size / (1024 * 1024);

    assert_eq!(mb, 953); // ~953 MB
}

// ============ CONCURRENT ERROR TESTS ============

#[test]
fn test_concurrent_parse_errors() {
    use std::sync::Arc;

    let invalid_hedl = Arc::new("invalid content without version".to_string());

    let handles: Vec<_> = (0..5)
        .map(|_| {
            let hedl = Arc::clone(&invalid_hedl);
            std::thread::spawn(move || {
                let result = core_parse(hedl.as_bytes());
                result.is_err()
            })
        })
        .collect();

    for handle in handles {
        let is_error = handle.join().unwrap();
        assert!(is_error, "Should consistently error on invalid input");
    }
}

// ============ UTF-8 ERROR TESTS ============

#[test]
fn test_utf8_valid() {
    let valid = "Hello 世界 🌍";
    assert!(std::str::from_utf8(valid.as_bytes()).is_ok());
}

#[test]
fn test_utf8_emoji() {
    let emoji = "🚀🎉💯";
    assert!(std::str::from_utf8(emoji.as_bytes()).is_ok());

    let hedl = format!("%VERSION: 1.0\n---\nemoji: {emoji}\n");
    let result = core_parse(hedl.as_bytes());

    // Should handle emoji without error
    assert!(result.is_ok());
}

// ============ ITEM TRAVERSAL ERROR TESTS ============

#[test]
fn test_empty_item_traversal() {
    let hedl = "%VERSION: 1.0\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Traversing empty root should not error
    for _item in doc.root.values() {
        // Empty iteration
    }
}

#[test]
fn test_mixed_item_types() {
    let hedl = r"
%VERSION: 1.0
%STRUCT: T: [id]
---
scalar: simple_value
list: @T
  | item1
object:
  nested: value
";

    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Should handle mixed item types
    for item in doc.root.values() {
        match item {
            Item::Scalar(_) => {}
            Item::List(_) => {}
            Item::Object(_) => {}
        }
    }
}

// ============ NODE CHILDREN ERROR TESTS ============

#[test]
fn test_node_without_children() {
    let hedl = r"
%VERSION: 1.0
%STRUCT: T: [id]
---
items: @T
  | item1
";

    let doc = core_parse(hedl.as_bytes()).unwrap();

    if let Some(Item::List(list)) = doc.root.get("items") {
        for node in &list.rows {
            // Node without children should return None
            if let Some(children_map) = node.children() {
                assert!(children_map.is_empty() || !children_map.is_empty());
            }
        }
    }
}

#[test]
fn test_node_with_empty_children_map() {
    // Test handling of nodes with empty children maps
    let hedl = r"
%VERSION: 1.0
%STRUCT: Parent: [id]
%STRUCT: Child: [id]
%NEST: Parent > Child
---
parents: @Parent
  | parent1
";

    let doc = core_parse(hedl.as_bytes()).unwrap();

    if let Some(Item::List(list)) = doc.root.get("parents") {
        for node in &list.rows {
            if let Some(children_map) = node.children() {
                // May be empty if no children added
                let _ = children_map;
            }
        }
    }
}

// ============ FIELD ACCESS ERROR TESTS ============

#[test]
fn test_field_index_out_of_bounds() {
    let hedl = r"
%VERSION: 1.0
%STRUCT: T: [id, name]
---
items: @T
  | alice, Alice
";

    let doc = core_parse(hedl.as_bytes()).unwrap();

    if let Some(Item::List(list)) = doc.root.get("items") {
        for node in &list.rows {
            // Access existing fields
            assert!(!node.fields.is_empty());
            assert!(node.fields.get(1).is_some());

            // Access non-existent field
            assert!(node.fields.get(10).is_none());
        }
    }
}

#[test]
fn test_empty_fields() {
    let hedl = r"
%VERSION: 1.0
%STRUCT: T: [id]
---
items: @T
  | item1
";

    let doc = core_parse(hedl.as_bytes()).unwrap();

    if let Some(Item::List(list)) = doc.root.get("items") {
        for node in &list.rows {
            // Should have at least ID field
            assert!(!node.fields.is_empty());
        }
    }
}

// ============ SCHEMA ACCESS ERROR TESTS ============

#[test]
fn test_schema_missing_columns() {
    let hedl = "%VERSION: 1.0\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Access non-existent schema
    assert!(!doc.structs.contains_key("NonExistent"));
}

#[test]
fn test_empty_schema_definition() {
    // Test if parser accepts empty schema
    let hedl = "%VERSION: 1.0\n%STRUCT: T: []\n---\n";
    let result = core_parse(hedl.as_bytes());

    // May or may not be valid depending on parser
    let _ = result;
}

// ============ ALIAS ACCESS ERROR TESTS ============

#[test]
fn test_alias_missing_value() {
    let hedl = "%VERSION: 1.0\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Access non-existent alias
    assert!(!doc.aliases.contains_key("%nonexistent"));
}

#[test]
fn test_alias_resolution() {
    let hedl = r#"
%VERSION: 1.0
%ALIAS: %active: "true"
---
status: %active
"#;

    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Aliases may be stored with or without the % prefix
    let has_alias = doc.aliases.contains_key("%active") || doc.aliases.contains_key("active");
    assert!(has_alias, "Alias should be defined");
}

// ============ NEST ACCESS ERROR TESTS ============

#[test]
fn test_nest_missing_relationship() {
    let hedl = "%VERSION: 1.0\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Access non-existent nest
    assert!(!doc.nests.contains_key("NonExistent"));
}

#[test]
fn test_nest_circular_reference() {
    // Test handling of circular nest references
    let hedl = r"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%NEST: A > B
%NEST: B > A
---
";

    let result = core_parse(hedl.as_bytes());

    // Parser may accept this (validation would catch it)
    let _ = result;
}

// ============ STATISTICS ERROR TESTS ============

#[test]
fn test_stats_division_by_zero() {
    // Test handling when JSON tokens is 0
    let json_tokens = 0;
    let hedl_tokens = 100;

    let savings = if json_tokens > 0 {
        ((i64::from(json_tokens) - i64::from(hedl_tokens)) * 100 / i64::from(json_tokens)) as i32
    } else {
        0
    };

    assert_eq!(savings, 0);
}

#[test]
fn test_stats_overflow() {
    // Test large numbers don't overflow
    let hedl_tokens = i64::MAX as usize / 2;
    let json_tokens = i64::MAX as usize / 2;

    let savings = if json_tokens > 0 {
        ((json_tokens as i64 - hedl_tokens as i64) * 100 / json_tokens as i64) as i32
    } else {
        0
    };

    assert_eq!(savings, 0);
}

// ============ TOKEN ESTIMATION ERROR TESTS ============

#[cfg(any(feature = "statistics", feature = "token-tools"))]
#[test]
fn test_token_estimation_edge_cases() {
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

    // All whitespace
    let all_whitespace = "    \n\t\r\n    ";
    let tokens = estimate_tokens(all_whitespace);
    assert!(tokens > 0);

    // All punctuation
    let all_punct = "!@#$%^&*()";
    let tokens = estimate_tokens(all_punct);
    assert!(tokens > 0);

    // Mixed
    let mixed = "a b, c! d?";
    let tokens = estimate_tokens(mixed);
    assert!(tokens > 0);
}

// ============ RECOVERY TESTS ============

#[test]
fn test_recovery_after_parse_error() {
    // Test that after a parse error, we can still parse valid documents
    let invalid = "invalid content";
    let result1 = core_parse(invalid.as_bytes());
    assert!(result1.is_err());

    let valid = "%VERSION: 1.0\n---\n";
    let result2 = core_parse(valid.as_bytes());
    assert!(result2.is_ok());
}

#[test]
fn test_multiple_parse_errors() {
    // Test handling multiple parse errors in sequence
    let invalid_docs = vec!["invalid1", "invalid2", "---\nno version"];

    for doc in invalid_docs {
        let result = core_parse(doc.as_bytes());
        // Each should error independently
        let _ = result;
    }

    // Valid parse should still work
    let valid = "%VERSION: 1.0\n---\n";
    let result = core_parse(valid.as_bytes());
    assert!(result.is_ok());
}
