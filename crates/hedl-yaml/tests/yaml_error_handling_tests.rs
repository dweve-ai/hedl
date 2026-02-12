// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for `YamlError` types, error messages, suggestions,
//! and snippet extraction.

use hedl_yaml::error::{ErrorContext, Location, Span, YamlError};

// ==================== Location Tests ====================

#[test]
fn test_location_new() {
    let loc = Location::new(10, 5, 123);
    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 5);
    assert_eq!(loc.byte_offset, 123);
}

#[test]
fn test_location_display() {
    let loc = Location::new(42, 10, 456);
    assert_eq!(loc.to_string(), "line 42, column 10");
}

#[test]
fn test_location_clone() {
    let loc1 = Location::new(1, 2, 3);
    let loc2 = loc1.clone();
    assert_eq!(loc1.line, loc2.line);
    assert_eq!(loc1.column, loc2.column);
    assert_eq!(loc1.byte_offset, loc2.byte_offset);
}

#[test]
fn test_location_equality() {
    let loc1 = Location::new(5, 10, 50);
    let loc2 = Location::new(5, 10, 50);
    let loc3 = Location::new(5, 11, 50);

    assert_eq!(loc1, loc2);
    assert_ne!(loc1, loc3);
}

// ==================== Span Tests ====================

#[test]
fn test_span_new() {
    let start = Location::new(1, 1, 0);
    let end = Location::new(1, 10, 9);
    let span = Span::new(start.clone(), end.clone());

    assert_eq!(span.start, start);
    assert_eq!(span.end, end);
}

#[test]
fn test_span_clone() {
    let start = Location::new(1, 1, 0);
    let end = Location::new(1, 10, 9);
    let span1 = Span::new(start, end);
    let span2 = span1.clone();

    assert_eq!(span1.start, span2.start);
    assert_eq!(span1.end, span2.end);
}

// ==================== Error Type Tests ====================

#[test]
fn test_parse_error() {
    let err = YamlError::ParseError {
        message: "invalid syntax".to_string(),
        context: ErrorContext::boxed(Some(Location::new(3, 5, 20)), None),
    };

    let display = err.to_string();
    assert!(display.contains("YAML parse error"));
    assert!(display.contains("invalid syntax"));
}

#[test]
fn test_invalid_root_type_error() {
    let err = YamlError::InvalidRootType {
        found: "sequence".to_string(),
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Root must be a YAML mapping"));
    assert!(display.contains("sequence"));
}

#[test]
fn test_non_string_key_error() {
    let err = YamlError::NonStringKey {
        key_type: "number".to_string(),
        path: "root.config".to_string(),
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Non-string keys not supported"));
    assert!(display.contains("number"));
    assert!(display.contains("root.config"));
}

#[test]
fn test_invalid_number_error() {
    let err = YamlError::InvalidNumber {
        value: "not-a-number".to_string(),
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Invalid number format"));
    assert!(display.contains("not-a-number"));
}

#[test]
fn test_invalid_expression_error() {
    let err = YamlError::InvalidExpression {
        message: "unexpected token".to_string(),
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Invalid expression"));
    assert!(display.contains("unexpected token"));
}

#[test]
fn test_invalid_reference_error() {
    let err = YamlError::InvalidReference {
        message: "malformed reference".to_string(),
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Invalid reference format"));
    assert!(display.contains("malformed reference"));
}

#[test]
fn test_nested_object_in_scalar_error() {
    let err = YamlError::NestedObjectInScalar {
        path: "root.field".to_string(),
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Nested objects not allowed in scalar context"));
    assert!(display.contains("root.field"));
}

#[test]
fn test_invalid_tensor_element_error() {
    let err = YamlError::InvalidTensorElement {
        path: "matrix[0]".to_string(),
        expected: "number".to_string(),
        found: "string".to_string(),
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Invalid tensor element"));
    assert!(display.contains("matrix[0]"));
}

#[test]
fn test_resource_limit_exceeded_error() {
    let err = YamlError::ResourceLimitExceeded {
        limit_type: "array_length".to_string(),
        limit: 1000,
        actual: 2000,
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Resource limit exceeded"));
    assert!(display.contains("1000"));
    assert!(display.contains("2000"));
}

#[test]
fn test_max_depth_exceeded_error() {
    let err = YamlError::MaxDepthExceeded {
        max_depth: 100,
        actual_depth: 150,
        path: "root.deep.path".to_string(),
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Maximum nesting depth"));
    assert!(display.contains("100"));
    assert!(display.contains("150"));
    assert!(display.contains("root.deep.path"));
}

#[test]
fn test_document_too_large_error() {
    let err = YamlError::DocumentTooLarge {
        size: 20_000_000,
        max_size: 10_000_000,
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Document size"));
    assert!(display.contains("20000000"));
    assert!(display.contains("10000000"));
}

#[test]
fn test_array_too_long_error() {
    let err = YamlError::ArrayTooLong {
        length: 2000,
        max_length: 1000,
        path: "root.items".to_string(),
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Array length"));
    assert!(display.contains("2000"));
    assert!(display.contains("1000"));
    assert!(display.contains("root.items"));
}

#[test]
fn test_conversion_error() {
    let err = YamlError::Conversion {
        message: "conversion failed".to_string(),
        context: None,
    };

    let display = err.to_string();
    assert!(display.contains("Conversion error"));
    assert!(display.contains("conversion failed"));
}

#[test]
fn test_forward_reference_error() {
    let err = YamlError::ForwardReference {
        alias: "undefined".to_string(),
        line: 5,
    };

    assert_eq!(
        err.to_string(),
        "Forward reference: alias '*undefined' at line 5 references undefined anchor"
    );
}

#[test]
fn test_circular_reference_error() {
    let err = YamlError::CircularReference {
        cycle_path: "a -> b -> c -> a".to_string(),
        anchors: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        locations: vec![1, 5, 10],
    };

    assert_eq!(
        err.to_string(),
        "Circular anchor reference: a -> b -> c -> a"
    );
}

#[test]
fn test_invalid_anchor_name_error() {
    let err = YamlError::InvalidAnchorName {
        name: "__reserved".to_string(),
        reason: "Names starting with __ are reserved".to_string(),
    };

    assert_eq!(
        err.to_string(),
        "Invalid anchor name '__reserved': Names starting with __ are reserved"
    );
}

#[test]
fn test_anchor_redefinition_error() {
    let err = YamlError::AnchorRedefinition {
        name: "anchor1".to_string(),
        old_line: 5,
        new_line: 10,
    };

    assert_eq!(
        err.to_string(),
        "Anchor 'anchor1' redefined at line 10 (previously defined at line 5)"
    );
}

// ==================== Error Method Tests ====================

#[test]
fn test_error_location_method() {
    let loc = Location::new(5, 10, 50);
    let err = YamlError::ParseError {
        message: "test".to_string(),
        context: ErrorContext::boxed(Some(loc.clone()), None),
    };

    assert_eq!(err.location(), Some(&loc));
}

#[test]
fn test_error_location_method_none() {
    let err = YamlError::ForwardReference {
        alias: "test".to_string(),
        line: 1,
    };

    assert_eq!(err.location(), None);
}

#[test]
fn test_error_snippet_method() {
    let err = YamlError::ParseError {
        message: "test".to_string(),
        context: ErrorContext::boxed(None, Some("test snippet".to_string())),
    };

    assert_eq!(err.snippet(), Some("test snippet"));
}

#[test]
fn test_error_snippet_method_none() {
    let err = YamlError::ForwardReference {
        alias: "test".to_string(),
        line: 1,
    };

    assert_eq!(err.snippet(), None);
}

#[test]
fn test_error_path_method() {
    let err = YamlError::NonStringKey {
        key_type: "number".to_string(),
        path: "root.items".to_string(),
        context: None,
    };

    assert_eq!(err.path(), Some("root.items"));
}

#[test]
fn test_error_path_method_none() {
    let err = YamlError::ParseError {
        message: "test".to_string(),
        context: None,
    };

    assert_eq!(err.path(), None);
}

// ==================== Error Suggestion Tests ====================

#[test]
fn test_parse_error_suggestions() {
    let err = YamlError::ParseError {
        message: "test".to_string(),
        context: None,
    };

    let suggestions = err.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("syntax")));
}

#[test]
fn test_invalid_root_type_suggestions() {
    let err = YamlError::InvalidRootType {
        found: "sequence".to_string(),
        context: None,
    };

    let suggestions = err.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("mapping")));
}

#[test]
fn test_non_string_key_suggestions() {
    let err = YamlError::NonStringKey {
        key_type: "number".to_string(),
        path: "test".to_string(),
        context: None,
    };

    let suggestions = err.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("string")));
}

#[test]
fn test_max_depth_exceeded_suggestions() {
    let err = YamlError::MaxDepthExceeded {
        max_depth: 100,
        actual_depth: 150,
        path: "test".to_string(),
        context: None,
    };

    let suggestions = err.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("nesting")));
}

#[test]
fn test_document_too_large_suggestions() {
    let err = YamlError::DocumentTooLarge {
        size: 1000,
        max_size: 500,
        context: None,
    };

    let suggestions = err.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("size")));
}

#[test]
fn test_array_too_long_suggestions() {
    let err = YamlError::ArrayTooLong {
        length: 2000,
        max_length: 1000,
        path: "test".to_string(),
        context: None,
    };

    let suggestions = err.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions
        .iter()
        .any(|s| s.contains("array") || s.contains("split")));
}

#[test]
fn test_forward_reference_suggestions() {
    let err = YamlError::ForwardReference {
        alias: "undefined".to_string(),
        line: 5,
    };

    let suggestions = err.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.contains("anchor")));
}

#[test]
fn test_circular_reference_suggestions() {
    let err = YamlError::CircularReference {
        cycle_path: "a -> b -> a".to_string(),
        anchors: vec!["a".to_string(), "b".to_string()],
        locations: vec![1, 5],
    };

    let suggestions = err.suggestions();
    assert!(!suggestions.is_empty());
    assert!(suggestions
        .iter()
        .any(|s| s.contains("circular") || s.contains("Circular")));
}

// ==================== Error Clone and Equality Tests ====================

#[test]
fn test_error_clone() {
    let err1 = YamlError::ParseError {
        message: "test".to_string(),
        context: None,
    };
    let err2 = err1.clone();

    assert_eq!(err1, err2);
}

#[test]
fn test_error_equality() {
    let err1 = YamlError::ParseError {
        message: "test".to_string(),
        context: None,
    };
    let err2 = YamlError::ParseError {
        message: "test".to_string(),
        context: None,
    };
    let err3 = YamlError::ParseError {
        message: "different".to_string(),
        context: None,
    };

    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

// ==================== Error Conversion Tests ====================

#[test]
fn test_from_string() {
    let err: YamlError = "test error".to_string().into();

    match err {
        YamlError::Conversion { message, .. } => assert_eq!(message, "test error"),
        _ => panic!("Expected Conversion error"),
    }
}

#[test]
fn test_from_str() {
    let err: YamlError = "test error".into();

    match err {
        YamlError::Conversion { message, .. } => assert_eq!(message, "test error"),
        _ => panic!("Expected Conversion error"),
    }
}

#[test]
fn test_from_serde_yaml_error() {
    let yaml = "{ invalid: [";
    let result: Result<serde_yaml::Value, serde_yaml::Error> = serde_yaml::from_str(yaml);
    assert!(result.is_err());

    let serde_err = result.unwrap_err();
    let yaml_err: YamlError = serde_err.into();

    match yaml_err {
        YamlError::ParseError { message, .. } => {
            assert!(!message.is_empty());
        }
        _ => panic!("Expected ParseError"),
    }
}

// ==================== Error with All Fields Tests ====================

#[test]
fn test_error_with_complete_context() {
    let loc = Location::new(10, 5, 100);
    let err = YamlError::NonStringKey {
        key_type: "number".to_string(),
        path: "root.config".to_string(),
        context: ErrorContext::boxed(Some(loc.clone()), Some("  123: value".to_string())),
    };

    // Check message
    let display = err.to_string();
    assert!(display.contains("root.config"));
    assert!(display.contains("number"));

    // Check location
    let location = err.location().unwrap();
    assert_eq!(location.line, 10);
    assert_eq!(location.column, 5);

    // Check snippet
    assert_eq!(err.snippet().unwrap(), "  123: value");

    // Check path
    assert_eq!(err.path().unwrap(), "root.config");

    // Check suggestions
    let suggestions = err.suggestions();
    assert!(!suggestions.is_empty());
}

// ==================== Debug Display Tests ====================

#[test]
fn test_error_debug_display() {
    let err = YamlError::ParseError {
        message: "test".to_string(),
        context: None,
    };

    let debug = format!("{err:?}");
    assert!(debug.contains("ParseError"));
}

#[test]
fn test_location_debug_display() {
    let loc = Location::new(1, 2, 3);
    let debug = format!("{loc:?}");

    assert!(debug.contains("Location"));
    assert!(debug.contains('1'));
    assert!(debug.contains('2'));
    assert!(debug.contains('3'));
}

#[test]
fn test_span_debug_display() {
    let start = Location::new(1, 1, 0);
    let end = Location::new(1, 10, 9);
    let span = Span::new(start, end);
    let debug = format!("{span:?}");

    assert!(debug.contains("Span"));
}
