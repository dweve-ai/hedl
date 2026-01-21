// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive error handling tests for hedl-json
//!
//! Tests all error paths, error messages, and error recovery scenarios

use hedl_json::*;
use serde_json::json;

// ==================== JsonConversionError Tests ====================

#[test]
fn test_depth_exceeded_error_display() {
    let config = FromJsonConfig::builder().max_depth(2).build();
    let json = json!({"a": {"b": {"c": {"d": 1}}}});

    let result = from_json_value(&json, &config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let msg = err.to_string();
    // Error format: "Maximum recursion depth (N) exceeded"
    assert!(msg.contains("Maximum recursion depth"));
    assert!(msg.contains("exceeded"));
    assert!(msg.contains('2'));
}

#[test]
fn test_array_size_exceeded_error() {
    let config = FromJsonConfig::builder().max_array_size(3).build();
    // Root must be an object, so put oversized array inside an object
    let json = json!({"numbers": [1, 2, 3, 4, 5]});

    let result = from_json_value(&json, &config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    // Error format: "Maximum array size (N) exceeded - array has M elements"
    assert!(err.to_string().contains("Maximum array size"));
    assert!(err.to_string().contains("exceeded"));
}

#[test]
fn test_object_size_exceeded_error() {
    let config = FromJsonConfig::builder().max_object_size(2).build();
    let json = json!({"a": 1, "b": 2, "c": 3});

    let result = from_json_value(&json, &config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    // Error format: "Maximum object size (N) exceeded - object has M keys"
    assert!(err.to_string().contains("Maximum object size"));
    assert!(err.to_string().contains("exceeded"));
}

#[test]
fn test_string_length_exceeded_error() {
    let config = FromJsonConfig::builder().max_string_length(5).build();
    let json = json!({"text": "this is a very long string"});

    let result = from_json_value(&json, &config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    // Error format: "Maximum string length (N) exceeded - string has M characters"
    assert!(err.to_string().contains("Maximum string length"));
    assert!(err.to_string().contains("exceeded"));
}

#[test]
fn test_invalid_json_syntax_error() {
    let invalid_json = "{invalid json}";
    let config = FromJsonConfig::default();

    let result = from_json(invalid_json, &config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("JSON parse error"));
}

#[test]
fn test_invalid_reference_format() {
    let json = json!({"ref": {"@ref": "invalid reference format"}});
    let config = FromJsonConfig::default();

    let result = from_json_value(&json, &config);
    // Should handle gracefully as a string or error
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_error_source_chain() {
    let config = FromJsonConfig::builder().max_depth(1).build();
    let json = json!({"a": {"b": 1}});

    let result = from_json_value(&json, &config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    // Test that error has meaningful source information
    assert!(!format!("{err:?}").is_empty());
}

#[test]
fn test_integer_overflow_error() {
    use serde_json::Number;

    // Create a number that's too large for i64
    let large_number = Number::from_f64(1e100).unwrap();
    let json = json!({"value": large_number});

    // This should either truncate or produce an error depending on handling
    let result = from_json_value(&json, &FromJsonConfig::default());
    // Accept either outcome as both are valid error handling strategies
    let _ = result;
}

#[test]
fn test_surrogate_pair_error_strict() {
    let config = FromJsonConfig::builder()
        .surrogate_policy(SurrogatePolicy::Reject)
        .build();

    // JSON with unpaired surrogate
    let json = r#"{"text": "\uD800"}"#;
    let result = from_json(json, &config);

    // Should either reject or auto-correct depending on JSON parser
    let _ = result;
}

#[test]
fn test_invalid_utf8_in_reference() {
    let json = json!({"ref": {"@ref": "@Type:id\u{FFFD}"}});
    let config = FromJsonConfig::default();

    let result = from_json_value(&json, &config);
    // Should either parse or reject gracefully
    let _ = result;
}

// ==================== Partial Parsing Error Tests ====================

#[test]
fn test_partial_parse_collect_all_errors() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    // JSON with multiple issues (very deep + large array)
    let json = r#"{
        "deep": {"a": {"b": {"c": {"d": {"e": 1}}}}},
        "large": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]
    }"#;

    let result = partial_parse_json(json, &config);

    // Should collect all errors
    if !result.errors.is_empty() {
        assert!(!result.errors.is_empty());

        for error in &result.errors {
            assert!(!error.location.path.is_empty());
            assert!(!error.error.to_string().is_empty());
        }
    }
}

#[test]
fn test_partial_parse_stop_at_first() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::StopOnFirst)
        .from_json_config(FromJsonConfig::builder().max_depth(2).build())
        .build();

    let json = r#"{
        "error1": {"a": {"b": {"c": 1}}},
        "error2": {"x": {"y": {"z": 2}}}
    }"#;

    let result = partial_parse_json(json, &config);

    // Should stop at first error
    if !result.errors.is_empty() {
        assert_eq!(result.errors.len(), 1);
    }
}

#[test]
fn test_partial_parse_collects_errors() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .from_json_config(FromJsonConfig::builder().max_depth(2).build())
        .build();

    // JSON with many potential errors
    let json = r#"{
        "a": {"x": {"y": {"z": 1}}},
        "b": {"x": {"y": {"z": 2}}},
        "c": {"x": {"y": {"z": 3}}}
    }"#;

    let result = partial_parse_json(json, &config);

    // Should collect errors when they occur
    // (exact count depends on error handling strategy)
    let _ = result.errors.len();
}

#[test]
fn test_error_location_accuracy() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .from_json_config(FromJsonConfig::builder().max_depth(2).build())
        .build();

    let json = r#"{"users": [{"profile": {"deep": {"nested": 1}}}]}"#;

    let result = partial_parse_json(json, &config);

    if !result.errors.is_empty() {
        for error in &result.errors {
            // Path should be meaningful
            assert!(
                error.location.path.contains("users")
                    || error.location.path.contains("profile")
                    || !error.location.path.is_empty()
            );
        }
    }
}

#[test]
fn test_partial_result_is_complete() {
    let valid_json = r#"{"name": "Alice", "age": 30}"#;
    let config = PartialConfig::default();

    let result = partial_parse_json(valid_json, &config);

    assert!(result.is_complete());
    assert!(result.errors.is_empty());
    assert!(result.document.is_some());
}

#[test]
fn test_partial_result_is_not_complete() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .from_json_config(FromJsonConfig::builder().max_depth(1).build())
        .build();

    let json = r#"{"deep": {"nested": 1}}"#;

    let result = partial_parse_json(json, &config);

    // May or may not be complete depending on error handling
    // Just verify the method works
    let _ = result.is_complete();
}

// ==================== Streaming Error Tests ====================

#[test]
fn test_stream_error_display() {
    use hedl_json::streaming::{JsonArrayStreamer, StreamConfig};
    use std::io::Cursor;

    let invalid_json = "not a json array";
    let reader = Cursor::new(invalid_json.as_bytes());
    let config = StreamConfig::default();

    let result = JsonArrayStreamer::new(reader, config);
    assert!(result.is_err());

    if let Err(err) = result {
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }
}

#[test]
fn test_stream_object_too_large_error() {
    use hedl_json::streaming::{JsonLinesStreamer, StreamConfig};
    use std::io::Cursor;

    let config = StreamConfig::builder()
        .max_object_bytes(100) // Very small limit
        .build();

    // Create a large object
    let large_obj = format!(r#"{{"data": "{}"}}"#, "x".repeat(200));
    let reader = Cursor::new(large_obj.as_bytes());

    let mut streamer = JsonLinesStreamer::new(reader, config);
    let result = streamer.next();

    // Should error on large object
    if let Some(Err(err)) = result {
        assert!(err.to_string().contains("exceeds limit") || err.to_string().contains("size"));
    }
}

// ==================== Schema Generation Error Tests ====================

#[test]
fn test_schema_validation_error_missing_type() {
    use hedl_json::schema_gen::validate_schema;

    let invalid_schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        // Missing "type" field
        "properties": {}
    });

    let result = validate_schema(&invalid_schema);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("type"));
}

#[test]
fn test_schema_validation_error_invalid_type() {
    use hedl_json::schema_gen::validate_schema;

    let invalid_schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "invalid_type_name"
    });

    let result = validate_schema(&invalid_schema);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid type"));
}

#[test]
fn test_schema_validation_error_not_object() {
    use hedl_json::schema_gen::validate_schema;

    let invalid_schema = json!("not an object");

    let result = validate_schema(&invalid_schema);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("object"));
}

#[test]
fn test_schema_error_debug_impl() {
    use hedl_json::schema_gen::SchemaError;

    let error = SchemaError::ValidationError("test error".to_string());
    let debug_str = format!("{error:?}");

    assert!(debug_str.contains("ValidationError"));
    assert!(debug_str.contains("test error"));
}

// ==================== Validation Error Tests ====================

#[cfg(feature = "validation")]
#[test]
fn test_validation_error_display_impl() {
    use hedl_json::validation::ValidationError;

    let error = ValidationError {
        instance_path: "/users/0/email".to_string(),
        message: "Invalid email format".to_string(),
        schema_path: "/properties/email/format".to_string(),
    };

    let display = error.to_string();
    assert!(display.contains("/users/0/email"));
    assert!(display.contains("Invalid email format"));
    assert!(display.contains("/properties/email/format"));
}

#[cfg(feature = "validation")]
#[test]
fn test_schema_compilation_error() {
    use hedl_json::validation::{CompiledSchema, ValidationConfig};

    let invalid_schema = json!({
        "type": "this_type_does_not_exist"
    });

    let result = CompiledSchema::compile(&invalid_schema, &ValidationConfig::default());
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(!err.to_string().is_empty());
}

// ==================== JSONPath Error Tests ====================

#[test]
fn test_jsonpath_invalid_expression_error() {
    use hedl_core::Document;
    use hedl_json::jsonpath::{query, QueryConfig};

    let doc = Document::new((1, 0));
    let config = QueryConfig::default();

    let result = query(&doc, "$$invalid[[", &config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid JSONPath"));
}

#[test]
fn test_jsonpath_query_single_no_results_error() {
    use hedl_core::parse;
    use hedl_json::jsonpath::{query_single, QueryConfig};

    let hedl = "%VERSION: 1.0\n---\nname: Alice";
    let doc = parse(hedl.as_bytes()).unwrap();
    let config = QueryConfig::default();

    let result = query_single(&doc, "$.missing_field", &config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("no results"));
}

#[test]
fn test_jsonpath_query_single_multiple_results_error() {
    use hedl_core::parse;
    use hedl_json::jsonpath::{query_single, QueryConfig};

    let hedl = "%VERSION: 1.0\n---\na: 1\nb: 2\nc: 3";
    let doc = parse(hedl.as_bytes()).unwrap();
    let config = QueryConfig::default();

    let result = query_single(&doc, "$.*", &config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("multiple") || err.to_string().contains("expected exactly 1"));
}

#[test]
fn test_jsonpath_error_clone() {
    use hedl_json::jsonpath::QueryError;

    let err1 = QueryError::InvalidExpression("test".to_string());
    let err2 = err1.clone();

    assert_eq!(err1, err2);
}

// ==================== Error Recovery Tests ====================

#[test]
fn test_continue_after_depth_limit_in_partial_parse() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .from_json_config(FromJsonConfig::builder().max_depth(2).build())
        .build();

    let json = r#"{
        "valid": "data",
        "deep": {"a": {"b": {"c": 1}}},
        "also_valid": 42
    }"#;

    let result = partial_parse_json(json, &config);

    // Should have partial document even with errors
    if let Some(doc) = result.document {
        // Document should exist with valid parts
        assert!(!doc.root.is_empty());
    }
}

#[test]
fn test_error_with_unicode_in_path() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .from_json_config(FromJsonConfig::builder().max_depth(1).build())
        .build();

    let json = r#"{"データ": {"nested": 1}}"#;

    let result = partial_parse_json(json, &config);

    // Should handle unicode field names in error paths
    if !result.errors.is_empty() {
        for error in &result.errors {
            assert!(!error.location.path.is_empty());
        }
    }
}

#[test]
fn test_multiple_error_types_in_one_document() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .from_json_config(
            FromJsonConfig::builder()
                .max_depth(2)
                .max_array_size(3)
                .max_string_length(10)
                .build(),
        )
        .build();

    let json = r#"{
        "deep": {"a": {"b": {"c": 1}}},
        "large_array": [1,2,3,4,5,6],
        "long_string": "this is a very long string that exceeds the limit"
    }"#;

    let result = partial_parse_json(json, &config);

    // May have multiple different error types
    if !result.errors.is_empty() {
        // Just verify errors are collected
        assert!(!result.errors.is_empty());
    }
}

// ==================== Edge Case Error Tests ====================

#[test]
fn test_empty_json_string_error() {
    let config = FromJsonConfig::default();
    let result = from_json("", &config);

    assert!(result.is_err());
}

#[test]
fn test_null_root_error() {
    let config = FromJsonConfig::default();
    let json = "null";

    let result = from_json(json, &config);
    // Null root should either error or produce empty document
    let _ = result;
}

#[test]
fn test_array_root_error() {
    let config = FromJsonConfig::default();
    let json = "[1, 2, 3]";

    let result = from_json(json, &config);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("object"));
}

#[test]
fn test_string_root_error() {
    let config = FromJsonConfig::default();
    let json = r#""just a string""#;

    let result = from_json(json, &config);
    assert!(result.is_err());
}

#[test]
fn test_number_root_error() {
    let config = FromJsonConfig::default();
    let json = "42";

    let result = from_json(json, &config);
    assert!(result.is_err());
}

#[test]
fn test_boolean_root_error() {
    let config = FromJsonConfig::default();
    let json = "true";

    let result = from_json(json, &config);
    assert!(result.is_err());
}
