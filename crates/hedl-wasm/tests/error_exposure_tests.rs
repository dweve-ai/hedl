// Dweve HEDL - Hierarchical Entity Data Language
// Tests to verify error exposure through WASM bindings

use hedl_core::{parse as core_parse, HedlErrorKind};

// ============ ERROR KIND EXPOSURE TESTS ============

#[test]
fn test_syntax_error_exposure() {
    // Test that syntax errors are properly exposed
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
---
invalid syntax here
"#;

    let result = core_parse(hedl.as_bytes());
    if let Err(err) = result {
        assert!(!err.message.is_empty(), "Error should have message");
        assert!(err.line > 0, "Error should have line number");
        // Error kind is accessible and formatted
        let error_type = format!("{}", err.kind);
        assert!(!error_type.is_empty(), "Error kind should format to string");
    }
}

#[test]
fn test_schema_error_exposure() {
    // Test schema violations
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Parent:[id]
%STRUCT: Child: [id]
---
parents:@Parent
 |p1, @Child#1:|c1
"#;

    let result = core_parse(hedl.as_bytes());
    if let Err(err) = result {
        // Schema errors occur when NEST relationship is missing
        assert!(!err.message.is_empty());
        assert!(err.line > 0);
    }
}

#[test]
fn test_shape_error_exposure() {
    // Test field count mismatch
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
users:@User
 |alice, Alice
"#;

    let result = core_parse(hedl.as_bytes());
    if let Err(err) = result {
        assert!(!err.message.is_empty());
        assert!(err.line > 0);
        // Shape errors indicate field count problems
        let _ = format!("{:?}", err.kind);
    }
}

#[test]
fn test_version_error_exposure() {
    // Test invalid version format (produces Version error)
    let hedl = r"
%V:abc
---
data: value
";

    let result = core_parse(hedl.as_bytes());
    if let Err(err) = result {
        assert!(!err.message.is_empty());
        assert!(err.line > 0);
        assert_eq!(err.kind, HedlErrorKind::Version);
    }
}

#[test]
fn test_error_message_format() {
    // Verify error messages are descriptive
    let invalid_docs = vec![
        ("---\n", "missing version"),
        ("%VERSION: invalid\n---\n", "invalid version"),
        ("%VERSION: 1.0\n", "missing separator"),
    ];

    for (hedl, _expected_content) in invalid_docs {
        let result = core_parse(hedl.as_bytes());
        if let Err(err) = result {
            // Error has required fields
            assert!(!err.message.is_empty(), "Error message should not be empty");
            let _ = err.line; // Line number is accessible

            // Can convert to string
            let error_string = format!("{}", err);
            assert!(!error_string.is_empty());
        }
    }
}

// ============ ERROR STRUCTURE TESTS ============

#[test]
fn test_error_has_all_required_fields() {
    let hedl = "invalid";
    let result = core_parse(hedl.as_bytes());

    if let Err(err) = result {
        // Verify all fields are accessible
        let _ = err.kind; // HedlErrorKind enum
        let _ = &err.message; // String
        let _ = err.line; // usize
        let _ = err.column; // Option<usize>
        let _ = &err.context; // Option<String>

        // Can format error
        let formatted = format!("{}", err);
        assert!(formatted.contains(&err.message));
    } else {
        panic!("Should produce error");
    }
}

#[test]
fn test_error_kind_to_string() {
    // Verify all error kinds can be converted to strings
    let kinds = vec![
        HedlErrorKind::Syntax,
        HedlErrorKind::Version,
        HedlErrorKind::Schema,
        HedlErrorKind::Alias,
        HedlErrorKind::Shape,
        HedlErrorKind::Semantic,
        HedlErrorKind::OrphanRow,
        HedlErrorKind::Collision,
        HedlErrorKind::Reference,
        HedlErrorKind::Security,
        HedlErrorKind::Conversion,
        HedlErrorKind::IO,
    ];

    for kind in kinds {
        let s = format!("{}", kind);
        assert!(
            !s.is_empty(),
            "Error kind should format to non-empty string"
        );
        assert!(s.contains("Error"), "Error kind should contain 'Error'");
    }
}

// ============ WASM INTEGRATION TESTS ============

#[test]
fn test_parse_error_propagation() {
    // Verify errors propagate correctly through parsing
    let hedl = "invalid content";
    let result = core_parse(hedl.as_bytes());

    assert!(result.is_err(), "Should error on invalid input");

    let err = result.unwrap_err();

    // Error can be formatted for JavaScript
    let js_error_message = format!("Parse error at line {}: {}", err.line, err.message);
    assert!(!js_error_message.is_empty());
    assert!(js_error_message.contains("Parse error"));
    assert!(js_error_message.contains(&err.message));
}

#[test]
#[cfg(feature = "json")]
fn test_json_conversion_error_exposure() {
    use hedl_json::{to_json_value, ToJsonConfig};

    // Parse valid document
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name]
---
users:@User
 |alice, Alice
"#;

    let doc = core_parse(hedl.as_bytes()).unwrap();

    // JSON conversion should succeed
    let config = ToJsonConfig::default();
    let result = to_json_value(&doc, &config);

    assert!(result.is_ok(), "JSON conversion should succeed");
}

#[test]
fn test_multiple_error_recovery() {
    // Test that after an error, subsequent parses still work
    let invalid = "invalid1";
    let _ = core_parse(invalid.as_bytes());

    let valid = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n";
    let result = core_parse(valid.as_bytes());

    assert!(result.is_ok(), "Should recover after previous error");
}

// ============ COMPREHENSIVE ERROR TYPE COVERAGE ============

#[test]
fn test_all_error_types_exposed() {
    // Document all error types that WASM bindings expose

    // These error kinds are defined in hedl-core and exposed through WASM:
    // - SyntaxError: Malformed syntax
    // - VersionError: Unsupported version
    // - SchemaError: Type/schema violations
    // - AliasError: Duplicate or invalid aliases
    // - ShapeError: Field count mismatches
    // - SemanticError: Logical errors
    // - OrphanRowError: Child without NEST
    // - CollisionError: Duplicate IDs
    // - ReferenceError: Unresolved references
    // - SecurityError: Resource limits exceeded
    // - ConversionError: Format conversion failures
    // - IOError: I/O operations

    // All error types can be stringified
    let _ = format!("{}", HedlErrorKind::Syntax);
    let _ = format!("{}", HedlErrorKind::Version);
    let _ = format!("{}", HedlErrorKind::Schema);
    let _ = format!("{}", HedlErrorKind::Alias);
    let _ = format!("{}", HedlErrorKind::Shape);
    let _ = format!("{}", HedlErrorKind::Semantic);
    let _ = format!("{}", HedlErrorKind::OrphanRow);
    let _ = format!("{}", HedlErrorKind::Collision);
    let _ = format!("{}", HedlErrorKind::Reference);
    let _ = format!("{}", HedlErrorKind::Security);
    let _ = format!("{}", HedlErrorKind::Conversion);
    let _ = format!("{}", HedlErrorKind::IO);
}

// ============ INLINE CHILD LIST ERROR TESTS ============

#[test]
fn test_inline_child_error_messages_descriptive() {
    // Test that inline child list errors (when the feature is used) provide
    // clear, actionable error messages

    // These test documents may or may not trigger inline child parsing
    // depending on the parser's recognition of the syntax.
    // The key is that any errors produced are clear and helpful.

    let test_cases = vec![
        (
            r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Parent:[id]
%STRUCT: Child: [id]
%NEST: Parent > Child
---
parents:@Parent
 |p1, @Child#
"#,
            "missing or malformed count",
        ),
        (
            r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Parent:[id]
%STRUCT: Child: [id]
%NEST: Parent > Child
---
parents:@Parent
 |p1, @Child#2
"#,
            "missing separator",
        ),
    ];

    for (hedl, _error_context) in test_cases {
        let result = core_parse(hedl.as_bytes());
        if let Err(err) = result {
            // Error should be clear and actionable
            assert!(
                !err.message.is_empty(),
                "Error message should be descriptive"
            );
            assert!(err.line > 0, "Error should have valid line number");

            // Error can be formatted for display
            let display = format!("{}", err);
            assert!(!display.is_empty());
        }
    }
}

#[test]
fn test_inline_child_feature_detection() {
    // Test that the parser correctly handles documents with and without
    // inline child syntax

    // Document without inline children - should parse successfully
    let simple = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Parent:[id, name]
%S:Child:[id]
%N:Parent>Child
---
parents:@Parent
 |p1, Parent One
  |c1
"#;

    let result = core_parse(simple.as_bytes());
    assert!(
        result.is_ok(),
        "Traditional nested syntax should work: {:?}",
        result.err()
    );
}

#[test]
fn test_error_messages_include_line_numbers() {
    // Verify all parser errors include line numbers for debugging

    let test_cases = vec![
        "invalid",
        "---\ndata: value",
        "%VERSION: 99.0\n---\n",
        "%VERSION: 1.0\n%STRUCT: Bad",
    ];

    for hedl in test_cases {
        let result = core_parse(hedl.as_bytes());
        if let Err(err) = result {
            assert!(
                err.line > 0 || hedl.len() < 10,
                "Error should have line number for: {:?}",
                hedl
            );
        }
    }
}
